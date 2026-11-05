//! actors/buffs/*.java (`game/buffs.py`): estados alterados (veneno, sangrado, quemadura, parálisis, lisiado...) y
//! las tiradas de salvación de 5e. Son datos (`data/buffs.json`) + un `Buff` genérico: un mod combina lo que existe
//! (ticks por nombre, daño con dados, velocidad, flags, salvación para terminarlo). Cada estado es un actor del
//! planificador (`ActorId::Buff`) y vive en el personaje (`Char::buffs`); el del héroe viaja con él.
//!
//! Campos de data/buffs.json (todos opcionales salvo name): `icon` (buffs.png, 7x7), `duration` (sin duración dura
//! hasta que su tick lo termine), `tick` (poison, bleeding, burning), `damage` (dados por turno), `speed`
//! (multiplicador), `flags` ("paralysed": no actúa, falla sola las salvaciones de FUE y DES y le atacan con ventaja),
//! `fx` ("flame"), `save_ends` (característica de la salvación que se repite al final de cada turno contra `dc`).
use serde_json::Value;

use crate::actor::{ActorId, TICK};
use crate::dice::{self, ABILITIES};
use crate::dungeon::{Dungeon, Tone};
use crate::registry::text;

#[derive(Clone, Debug)]
pub struct Buff {
    pub uid: u32,
    pub id: String,
    pub data: Value,
    /// Turnos restantes (None = sin límite).
    pub left: Option<f64>,
    /// Intensidad (sangrado).
    pub level: i32,
    /// CD de `save_ends`.
    pub dc: i32,
}

impl Buff {
    pub fn name(&self) -> String {
        text(&self.data["name"])
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.data.get("flags").and_then(Value::as_array).is_some_and(|f| f.iter().any(|v| v.as_str() == Some(flag)))
    }

    pub fn speed(&self) -> f64 {
        self.data.get("speed").and_then(Value::as_f64).unwrap_or(1.0)
    }

    /// Índice en buffs.png (None = no se muestra).
    pub fn icon(&self) -> Option<u32> {
        self.data.get("icon").and_then(Value::as_i64).filter(|&i| i >= 0).map(|i| i as u32)
    }
}

impl Dungeon {
    fn buffs_of(&self, a: ActorId) -> &[Buff] {
        self.ch(a).map_or(&[], |c| &c.buffs)
    }

    pub fn has_flag(&self, a: ActorId, flag: &str) -> bool {
        self.buffs_of(a).iter().any(|b| b.has_flag(flag))
    }

    /// Multiplicador de velocidad de todos los estados (Lentitud = la mitad).
    pub fn buff_speed(&self, a: ActorId) -> f64 {
        self.buffs_of(a).iter().map(Buff::speed).product()
    }

    /// Buff.affect + Buff.prolong: si ya lo tiene, se queda con lo mayor (duración, nivel, CD).
    pub(crate) fn add_buff(&mut self, a: ActorId, id: &str, duration: Option<f64>, level: i32, dc: i32) {
        let Some(data) = self.buffs.get(id).cloned() else { return };
        let default = data.get("duration").and_then(Value::as_f64);
        let Some(c) = self.ch_mut(a) else { return };
        if let Some(b) = c.buffs.iter_mut().find(|b| b.id == id) {
            if let Some(left) = b.left {
                b.left = duration.or(default).map(|new| left.max(new));
            }
            b.level = b.level.max(level);
            b.dc = b.dc.max(dc);
            return;
        }
        let uid = self.next_buff;
        self.next_buff += 1;
        let b = Buff { uid, id: id.to_string(), data, left: duration.or(default), level, dc };
        let name = b.name().to_lowercase();
        let c = self.ch_mut(a).unwrap();
        c.buffs.push(b);
        let (cname, pos) = (c.name.clone(), c.pos);
        self.floor_mut().scheduler.add(ActorId::Buff(uid), 0.0);
        if pos >= 0 && self.level().visible[pos as usize] {
            self.log(&format!("{cname}: {name}"));
        }
        self.status(a, &name, Tone::Neg);
    }

    /// Quién tiene el estado `uid` (en el piso en contexto).
    fn buff_owner(&self, uid: u32) -> Option<ActorId> {
        self.chars().into_iter().map(|c| c.0).find(|&a| self.buffs_of(a).iter().any(|b| b.uid == uid))
    }

    fn buff(&self, a: ActorId, uid: u32) -> Option<&Buff> {
        self.buffs_of(a).iter().find(|b| b.uid == uid)
    }

    fn buff_mut(&mut self, a: ActorId, uid: u32) -> Option<&mut Buff> {
        self.ch_mut(a)?.buffs.iter_mut().find(|b| b.uid == uid)
    }

    /// Una vez por turno: su efecto, después la duración y la salvación para librarse.
    pub(crate) fn buff_act(&mut self, uid: u32) {
        let Some(a) = self.buff_owner(uid) else {
            self.floor_mut().scheduler.remove(ActorId::Buff(uid)); // el dueño ya no está (un mob muerto)
            return;
        };
        let me = Some(ActorId::Buff(uid));
        let b = self.buff(a, uid).unwrap().clone();
        if self.alive(a) {
            match b.data.get("tick").and_then(Value::as_str) {
                // Poison.java: (turnos restantes / 3) + 1 de daño por turno
                Some("poison") => self.damage(a, (b.left.unwrap_or(0.0) / 3.0).floor() as i32 + 1, me),
                // Bleeding.java: cada turno baja al azar entre la mitad y el total; en 0 se termina
                Some("bleeding") => {
                    let level = self.rng.int_range_incl(b.level / 2, b.level);
                    self.buff_mut(a, uid).unwrap().level = level;
                    if level > 0 {
                        self.damage(a, level, me);
                    } else {
                        self.buff_mut(a, uid).unwrap().left = Some(0.0);
                    }
                }
                // Burning.java: 1-5 por turno; se apaga en el agua o al azar (más fácil cuanta menos vida queda)
                // ponytail: no quema pergaminos de la mochila ni prende el pasto (falta Fire / Blob)
                Some("burning") => {
                    let dmg = self.rng.int_range_incl(1, 5);
                    self.damage(a, dmg, me);
                    // si lo mató, el mob ya no está: el rng se gasta igual (como el prototipo, con 0 PV)
                    let (water, ratio) = self.ch(a).map_or((false, 0.0), |c| {
                        let water = c.pos >= 0 && self.level().water[c.pos as usize] && !c.flying;
                        (water, c.hp.max(0) as f64 / c.ht.max(1) as f64)
                    });
                    if (water || self.rng.float() > (2.0 + ratio) / 3.0)
                        && let Some(b) = self.buff_mut(a, uid)
                    {
                        b.left = Some(0.0);
                    }
                }
                _ => {}
            }
            if let Some(dmg) = b.data.get("damage").and_then(Value::as_str)
                && self.alive(a)
            {
                let (n, _, _, _) = dice::roll(&mut self.rng, dmg, false);
                self.damage(a, n, me);
            }
        }
        self.floor_mut().scheduler.spend(ActorId::Buff(uid), TICK);
        let Some(b) = self.buff_mut(a, uid) else {
            self.floor_mut().scheduler.remove(ActorId::Buff(uid)); // lo mató y ya no está
            return;
        };
        if let Some(l) = b.left.as_mut() {
            *l -= TICK;
        }
        let (left, dc, name) = (b.left, b.dc, b.name().to_lowercase());
        if let Some(ability) = b.data.get("save_ends").and_then(Value::as_str).map(str::to_string)
            && self.alive(a)
            && left.is_none_or(|l| l > 0.0)
            && self.saving_throw(a, &ability, dc, &name).0
            && let Some(b) = self.buff_mut(a, uid)
        {
            b.left = Some(0.0);
        }
        let ended = self.buff(a, uid).is_some_and(|b| b.left.is_some_and(|l| l <= 0.0));
        if !self.alive(a) || ended {
            self.detach_buff(a, uid);
        }
    }

    pub(crate) fn detach_buff(&mut self, a: ActorId, uid: u32) {
        if let Some(c) = self.ch_mut(a)
            && let Some(k) = c.buffs.iter().position(|b| b.uid == uid)
        {
            let b = c.buffs.remove(k);
            let (cname, pos) = (c.name.clone(), c.pos);
            if self.alive(a) && pos >= 0 && self.level().visible[pos as usize] {
                self.log(&crate::trf!("{cname}: ya no está {}", "{cname}: no longer {}", b.name().to_lowercase()));
            }
        }
        self.floor_mut().scheduler.remove(ActorId::Buff(uid));
    }

    /// Bono de salvación: héroe = mod + competencia (`save_prof`); mob = `saves` en data/mobs.json o precisión / 4.
    pub fn save_bonus(&self, a: ActorId, ability: &str) -> i32 {
        match a {
            ActorId::Hero(i) => self.heroes[i].save_bonus(ability),
            ActorId::Mob(id) => self.mob(id).map_or(0, |m| {
                let s = &m.stats;
                s.get("saves").and_then(|v| v.get(ability)).and_then(Value::as_i64).map_or_else(
                    || s.get("attack").and_then(Value::as_i64).unwrap_or(0).div_euclid(4) as i32,
                    |v| v as i32,
                )
            }),
            _ => 0,
        }
    }

    /// Tirada de salvación (5e): d20 + bono contra la CD, sin éxito ni fallo automático con 20 o 1. Paralizado: falla
    /// sola las de FUE y DES. `why` = de qué es ("trampa", "paralizado"): va en el recuadro del dado.
    /// Devuelve (superada, detalle para el log).
    pub(crate) fn saving_throw(&mut self, a: ActorId, ability: &str, dc: i32, why: &str) -> (bool, String) {
        if self.has_flag(a, "paralysed") && (ability == "STR" || ability == "DEX") {
            return (false, crate::tr!("paralizado: falla", "paralysed: fails").into());
        }
        let (nat, txt) = dice::d20(&mut self.rng, 0);
        let bonus = self.save_bonus(a, ability);
        let ok = nat + bonus >= dc;
        let pos = self.ch(a).map_or(-1, |c| c.pos);
        if pos >= 0 && self.level().visible[pos as usize] {
            let short = ABILITIES.iter().position(|x| *x == ability).map_or(ability, |k| dice::ability_names()[k]);
            let label = if why.is_empty() { crate::trf!("{short} CD{dc}", "{short} DC{dc}") } else { crate::trf!("{short} CD{dc} {why}", "{short} DC{dc} {why}") };
            self.roll_event(&label, nat + bonus, 1 + bonus, 20 + bonus, Some(ok));
        }
        (ok, crate::trf!("d20 {txt}{bonus:+}={} vs CD {dc}", "d20 {txt}{bonus:+}={} vs DC {dc}", nat + bonus))
    }

    /// Nombre de un origen de daño (el log y "Has muerto (...)").
    pub fn source_name(&self, src: ActorId) -> Option<String> {
        match src {
            ActorId::Hero(_) | ActorId::Mob(_) => self.ch(src).map(|c| c.name.clone()),
            ActorId::Buff(uid) => {
                self.chars().into_iter().find_map(|(a, _)| self.buff(a, uid).map(|b| b.name().to_lowercase()))
            }
            ActorId::Hunger(_) => Some(crate::tr!("hambre", "hunger").into()),
            ActorId::Trap => Some(crate::tr!("trampa eléctrica", "lightning trap").into()),
            _ => None,
        }
    }
}
