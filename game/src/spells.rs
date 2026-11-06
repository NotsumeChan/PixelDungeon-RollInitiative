//! Conjuros de D&D 5e (`game/spells.py`; el original no tiene un sistema así). Son datos (`data/spells.json`): un mod
//! combina los tipos que existen (`kind`), nunca apunta a código. Y los rasgos de clase de 5e: Segundo Aliento, Ataque
//! Furtivo (en `roll_damage`) y Enemigo predilecto (Marca del cazador gratis).
//!
//! Campos: `level` (0 = truco: sin espacio, los dados se multiplican en los niveles 5, 11 y 17; 1+ = gasta el menor
//! espacio de ese nivel o mayor), `kind` (attack: tirada de ataque de conjuro contra la CA, `damage` y `buff` opcional;
//! auto: `darts` dardos que aciertan solos; sleep: reserva de `pool` PV, duerme a los de menos vida a `radius`;
//! heal: cura `damage` + mod, o `amount` fijo; mark: pone `buff` al objetivo; save: el objetivo tira la salvación
//! `save` contra la CD y si falla recibe `damage` (con `half`, la mitad si se salva); buff: pone `buff` a uno mismo),
//! `target` (enemy | cell | self | touch), `upcast` (por cada nivel de espacio extra: darts / pool / damage), `bonus`
//! (acción adicional: no gasta tiempo), `fx`.
//! Rasgos con usos (`features` de la clase: [conjuro, base, cada]): se lanzan como conjuros de nivel 1 pero con usos
//! por descanso (base + (nivel - 1) / cada) en vez de espacios, y sin pedir competencia con la armadura.
//! ponytail: sin proyectiles ni partículas de conjuros (el log y los dados avisan); sin concentración ni foco
use serde_json::Value;

use crate::actor::ActorId;

use crate::dice;
use crate::dungeon::{Dungeon, Fx, Pending, Tone};
use crate::hero::Hero;
use crate::mobs::State;
use crate::registry::text;
use crate::{tr, trf};

/// Espacios de conjuro por nivel de personaje (5e): lanzador completo (Mago) y medio (Explorador, reglas 2024).
const FULL: [&[i32]; 10] = [&[2], &[3], &[4, 2], &[4, 3], &[4, 3, 2], &[4, 3, 3], &[4, 3, 3, 1], &[4, 3, 3, 2], &[4, 3, 3, 3, 1], &[4, 3, 3, 3, 2]];
const HALF: [&[i32]; 10] = [&[2], &[2], &[3], &[3], &[4, 2], &[4, 2], &[4, 3], &[4, 3], &[4, 3, 2], &[4, 3, 2]];
/// Magia de pacto (Brujo): pocos espacios, todos del nivel más alto que tiene.
const PACT: [&[i32]; 10] = [&[1], &[2], &[0, 2], &[0, 2], &[0, 0, 2], &[0, 0, 2], &[0, 0, 0, 2], &[0, 0, 0, 2], &[0, 0, 0, 0, 2], &[0, 0, 0, 0, 2]];
pub const TIME_TO_CAST: f64 = 1.0;

/// Rasgos que dan lanzamientos gratis: (rasgo, conjuro); usos = 2 + (nivel - 1) / 4.
const FREE_CASTS: [(&str, &str); 1] = [("pdri:favored_enemy", "pdri:hunters_mark")];

impl Hero {
    pub fn has_trait(&self, t: &str) -> bool {
        self.cls.get("traits").and_then(Value::as_array).is_some_and(|a| a.iter().any(|v| v.as_str() == Some(t)))
    }

    pub fn spellcasting(&self) -> Option<&Value> {
        self.cls.get("spellcasting")
    }

    /// Los conjuros de la clase y después sus rasgos con usos, en orden (el índice es el de `Action::Cast`).
    pub fn spell_ids(&self) -> Vec<String> {
        let list = self.spellcasting().and_then(|s| s.get("spells")).and_then(Value::as_array);
        let spells = list.into_iter().flatten().filter_map(Value::as_str);
        let feats = self.features().into_iter().map(|f| f.0);
        spells.map(str::to_string).chain(feats).collect()
    }

    /// Rasgos con usos de la clase: (conjuro, base, cada cuántos niveles suma uno).
    fn features(&self) -> Vec<(String, i32, i32)> {
        let list = self.cls.get("features").and_then(Value::as_array);
        list.into_iter()
            .flatten()
            .filter_map(|f| Some((f[0].as_str()?.to_string(), f[1].as_i64().unwrap_or(1) as i32, f[2].as_i64().unwrap_or(0) as i32)))
            .collect()
    }

    pub fn is_feature(&self, sid: &str) -> bool {
        self.features().iter().any(|f| f.0 == sid)
    }

    /// ¿Tiene libro (conjuros o rasgos con usos)? Lo abre F / el botón de la barra.
    pub fn has_book(&self) -> bool {
        !self.spell_ids().is_empty()
    }

    /// Bono de ataque de conjuro: competencia + modificador de la característica de lanzamiento.
    pub fn spell_bonus(&self) -> i32 {
        let ability = self.spellcasting().and_then(|s| s["ability"].as_str()).unwrap_or("INT");
        self.proficiency() + self.ability_mod(ability)
    }

    pub fn spell_dc(&self) -> i32 {
        8 + self.spell_bonus()
    }

    pub fn max_slots(&self, lvl: i32) -> Vec<i32> {
        let Some(kind) = self.spellcasting().and_then(|s| s["slots"].as_str()) else { return Vec::new() };
        let table = match kind {
            "half" => &HALF,
            "pact" => &PACT,
            _ => &FULL,
        };
        table[(lvl.clamp(1, 10) - 1) as usize].to_vec()
    }

    pub fn max_free(&self, lvl: i32) -> Vec<(String, i32)> {
        let free = FREE_CASTS.iter().filter(|(t, _)| self.has_trait(t)).map(|(_, s)| (s.to_string(), 2 + (lvl - 1) / 4));
        let feats = self.features().into_iter().map(|(s, base, every)| (s, base + if every > 0 { (lvl - 1) / every } else { 0 }));
        free.chain(feats).collect()
    }

    pub fn free_casts_of(&self, sid: &str) -> i32 {
        self.free_casts.iter().find(|f| f.0 == sid).map_or(0, |f| f.1)
    }

    /// Descanso (5e): Segundo Aliento, espacios de conjuro y usos gratis.
    /// ponytail: no hay descansos todavía; se recupera al llegar a un piso nuevo (como el prototipo)
    pub fn recover(&mut self) {
        self.second_wind = true;
        self.slots = self.max_slots(self.lvl);
        self.free_casts = self.max_free(self.lvl);
    }

    /// Al subir de nivel se suman los espacios y usos nuevos (los gastados siguen gastados).
    pub fn level_up_spells(&mut self) {
        let (old, new) = (self.max_slots(self.lvl - 1), self.max_slots(self.lvl));
        self.slots.resize(new.len().max(self.slots.len()), 0);
        for (k, n) in new.iter().enumerate() {
            // el pacto pasa sus espacios a un nivel mayor: los del nivel viejo desaparecen (nunca quedan negativos)
            self.slots[k] = (self.slots[k] + n - old.get(k).copied().unwrap_or(0)).max(0);
        }
        let old_free = self.max_free(self.lvl - 1);
        for (sid, n) in self.max_free(self.lvl) {
            let before = old_free.iter().find(|f| f.0 == sid).map_or(0, |f| f.1);
            match self.free_casts.iter_mut().find(|f| f.0 == sid) {
                Some(f) => f.1 += n - before,
                None => self.free_casts.push((sid, n - before)),
            }
        }
    }

    /// El menor nivel de espacio disponible >= level (0 si no hay).
    fn slot_for(&self, level: i32) -> i32 {
        (level.max(1) as usize - 1..self.slots.len()).find(|&k| self.slots[k] > 0).map_or(0, |k| k as i32 + 1)
    }

    /// Ataque Furtivo: 1d6 en nivel 1, +1d6 cada 2 niveles.
    pub fn sneak_dice(&self) -> i32 {
        (self.lvl + 1) / 2
    }

    fn cantrip_mult(&self) -> i32 {
        1 + (self.lvl >= 5) as i32 + (self.lvl >= 11) as i32 + (self.lvl >= 17) as i32
    }
}

/// "5d8" + 2 veces "2d8" -> "9d8" (mismo dado).
fn more(notation: &str, extra: &str, times: i32) -> String {
    let (mut n, sides, b) = dice::parse(notation).unwrap_or((1, 4, 0));
    if times > 0 {
        n += dice::parse(extra).map_or(0, |e| e.0) * times;
    }
    if b != 0 { format!("{n}d{sides}{b:+}") } else { format!("{n}d{sides}") }
}

impl Dungeon {
    pub fn spell(&self, sid: &str) -> Option<&Value> {
        self.spells.get(sid)
    }

    pub fn can_cast(&self, h: &Hero, sid: &str) -> bool {
        let Some(s) = self.spell(sid) else { return false };
        if !h.spell_ids().iter().any(|x| x == sid) || !(h.proficient_armor() || h.is_feature(sid)) {
            return false;
        }
        let level = s["level"].as_i64().unwrap_or(0) as i32;
        level == 0 || h.free_casts_of(sid) > 0 || h.slot_for(level) > 0
    }

    /// Segundo Aliento (Guerrero): recupera 1d10 + nivel PV, una vez hasta recargarse. Acción adicional (sin tiempo).
    pub(crate) fn second_wind(&mut self, i: usize) {
        if !self.heroes[i].has_trait("pdri:second_wind") {
            return;
        }
        if !self.heroes[i].second_wind {
            self.tell(i, tr!("Ya usaste el Segundo Aliento", "You already used Second Wind"));
            return;
        }
        self.heroes[i].second_wind = false;
        let (roll, _, rolled, _) = dice::roll(&mut self.rng, "1d10", false);
        let lvl = self.heroes[i].lvl;
        let heal = self.heal(i, roll + lvl);
        self.roll_event("1d10", rolled[0], 1, 10, None);
        self.log(&trf!("Segundo Aliento: +{heal} PV", "Second Wind: +{heal} HP"));
        self.status(ActorId::Hero(i), &format!("+{heal}"), Tone::Pos);
    }

    /// La línea de tiro de un héroe (proyectil o conjuro): pasa por los otros héroes sin golpearlos (no hay fuego
    /// amigo) y se frena en el primer enemigo o pared. Devuelve (dónde se frena, cobertura): con un aliado en medio,
    /// media cobertura = -2 a la tirada (5e: +2 a la CA del objetivo).
    pub fn shot(&self, from: i32, to: i32) -> (i32, i32) {
        let ally = std::cell::Cell::new(false);
        let stop = crate::ballistica::cast(self.level(), from, to, |c| match self.find_char(c) {
            Some(ActorId::Hero(_)) => {
                ally.set(true);
                false
            }
            Some(_) => true,
            None => false,
        });
        (stop, if ally.get() { 2 } else { 0 })
    }

    /// Valida y lanza el conjuro `k` de la clase sobre `cell`. Si no se puede, avisa y no gasta tiempo.
    pub(crate) fn cast(&mut self, i: usize, k: usize, cell: i32) {
        let me = ActorId::Hero(i);
        let Some(sid) = self.heroes[i].spell_ids().get(k).cloned() else { return };
        let Some(s) = self.spell(&sid).cloned() else { return };
        if !self.heroes[i].proficient_armor() && !self.heroes[i].is_feature(&sid) {
            self.tell(i, tr!("No puedes lanzar conjuros con una armadura sin competencia", "You can't cast spells in armor you lack proficiency with"));
            return;
        }
        let pos = self.heroes[i].ch.pos;
        let tgt = s["target"].as_str().unwrap_or("self");
        let mut target = None;
        let mut cover = 0;
        match tgt {
            "enemy" => {
                let t = self.find_char(cell).filter(|&t| t != me && self.heroes[i].fov[cell as usize]);
                let Some(t) = t else {
                    self.tell(i, tr!("Necesitas un enemigo a la vista", "You need an enemy in sight"));
                    return;
                };
                let (stop, c) = self.shot(pos, cell);
                if stop != cell {
                    self.tell(i, tr!("No hay línea de tiro", "No clear line of fire"));
                    return;
                }
                target = Some(t);
                cover = c;
            }
            "cell" if !self.heroes[i].fov[cell as usize] => {
                self.tell(i, tr!("Necesitas una casilla a la vista", "You need a tile in sight"));
                return;
            }
            "touch" => {
                // uno mismo o un aliado al lado (también caído)
                let t = self.find_char(cell);
                if t != Some(me) && !(matches!(t, Some(ActorId::Hero(_))) && self.level().adjacent(pos, cell)) {
                    self.tell(i, tr!("Tócate a ti o a un aliado que esté al lado", "Touch yourself or an adjacent ally"));
                    return;
                }
                target = t;
            }
            _ => {}
        }
        let level = s["level"].as_i64().unwrap_or(0) as i32;
        let mut extra = 0;
        if level > 0 {
            let h = &mut self.heroes[i];
            if let Some(f) = h.free_casts.iter_mut().find(|f| f.0 == sid && f.1 > 0) {
                f.1 -= 1;
            } else {
                let slot = if h.is_feature(&sid) { 0 } else { h.slot_for(level) };
                if slot == 0 {
                    let msg = if h.is_feature(&sid) {
                        tr!("No te quedan usos (se recuperan al descansar)", "No uses left (they recover on a rest)")
                    } else {
                        tr!("No te quedan espacios de conjuro", "No spell slots left")
                    };
                    self.tell(i, msg);
                    return;
                }
                h.slots[slot as usize - 1] -= 1;
                extra = slot - level;
            }
        }
        let anim_cell = if tgt == "self" { pos } else { cell };
        let kind = if tgt == "self" || tgt == "touch" { "operate" } else { "attack" }; // HeroSprite: zap = ataque
        self.effect(Fx::Anim { who: me, kind, cell: anim_cell });
        let name = text(&s["name"]);
        match s["kind"].as_str().unwrap_or("") {
            "attack" => self.spell_attack(i, &s, &name, target.unwrap(), cover),
            "auto" => self.spell_auto(i, &s, &name, target.unwrap(), extra),
            "sleep" => self.spell_sleep(&s, &name, cell, extra),
            "heal" => self.spell_heal(i, &s, &name, target.unwrap_or(me), extra),
            "mark" => self.spell_mark(i, &s, &name, target.unwrap()),
            "save" => self.spell_save(i, &s, &name, target.unwrap(), extra),
            "buff" => {
                let id = s["buff"][0].as_str().unwrap_or("").to_string();
                self.add_buff(me, &id, s["buff"][1].as_f64(), 0, 0);
                let who = self.heroes[i].ch.name.clone();
                self.log(&format!("{who}: {name}"));
                self.status(me, &name.to_lowercase(), Tone::Pos);
            }
            _ => {}
        }
        if !s.get("bonus").and_then(Value::as_bool).unwrap_or(false) {
            self.spend(me, TIME_TO_CAST);
        }
    }

    fn spell_attack(&mut self, i: usize, s: &Value, name: &str, target: ActorId, cover: i32) {
        let me = ActorId::Hero(i);
        let adv = self.target_advantage(target); // sólo lo del objetivo (desprevenido, paralizado, caído)
        let (nat, txt) = dice::d20(&mut self.rng, adv);
        let bonus = self.heroes[i].spell_bonus() - cover;
        let ac = self.armor_class_of(target);
        let hit = nat == 20 || (nat != 1 && nat + bonus >= ac);
        self.roll_event("d20", nat, 1, 20, None);
        let line = trf!("{name}: d20 {txt}{bonus:+}={} vs CA {ac}", "{name}: d20 {txt}{bonus:+}={} vs AC {ac}", nat + bonus);
        if !hit {
            let p = Pending::Attack { attacker: me, target, hit: false, dmg: 0, detail: trf!("{line}, falla", "{line}, miss"), simple: trf!("{name} falla", "{name} misses") };
            self.defer(p);
            return;
        }
        let (n, sides, b) = dice::parse(s["damage"].as_str().unwrap_or("1d4")).unwrap_or((1, 4, 0));
        let lvl0 = s["level"].as_i64() == Some(0);
        let n = n * if lvl0 { self.heroes[i].cantrip_mult() } else { 1 };
        let notation = if b != 0 { format!("{n}d{sides}{b:+}") } else { format!("{n}d{sides}") };
        let (total, dtext, rolled, _) = dice::roll(&mut self.rng, &notation, nat == 20);
        let k = rolled.len() as i32;
        self.roll_event(&format!("{k}d{sides}"), rolled.iter().sum(), k, k * sides, None);
        let tname = self.ch(target).map_or(String::new(), |c| c.name.clone());
        let crit = if nat == 20 { tr!(" (crítico)", " (critical)") } else { "" };
        let p = Pending::Attack {
            attacker: me,
            target,
            hit: true,
            dmg: total,
            detail: format!("{line}, {dtext}={total}"),
            simple: trf!("{name} golpea a {tname}: {total}{crit}", "{name} hits {tname}: {total}{crit}"),
        };
        self.defer(p);
        if let Some(buff) = s.get("buff").and_then(Value::as_array) {
            let id = buff[0].as_str().unwrap_or("").to_string();
            self.defer(Pending::Buff { target, id, duration: buff[1].as_f64() });
        }
    }

    /// Conjuro de salvación: el objetivo tira `save` contra tu CD; si falla recibe el daño (con `half`, la mitad si se
    /// salva) y el `buff`.
    fn spell_save(&mut self, i: usize, s: &Value, name: &str, target: ActorId, extra: i32) {
        let ability = s["save"].as_str().unwrap_or("DEX");
        let dc = self.heroes[i].spell_dc();
        let (saved, stxt) = self.saving_throw(target, ability, dc, name);
        let half = s.get("half").and_then(Value::as_bool).unwrap_or(false);
        let tname = self.ch(target).map_or(String::new(), |c| c.name.clone());
        let me = ActorId::Hero(i);
        if saved && !half {
            let p = Pending::Attack { attacker: me, target, hit: false, dmg: 0, detail: trf!("{name}: {stxt}, se salva", "{name}: {stxt}, saved"), simple: trf!("{tname} resiste {name}", "{tname} resists {name}") };
            self.defer(p);
            return;
        }
        let base = s["damage"].as_str().unwrap_or("1d4");
        let up = s.get("upcast").and_then(|u| u.get("damage")).and_then(Value::as_str).unwrap_or(base);
        let (n, sides, b) = dice::parse(&more(base, up, extra)).unwrap_or((1, 4, 0));
        let n = n * if s["level"].as_i64() == Some(0) { self.heroes[i].cantrip_mult() } else { 1 };
        let notation = if b != 0 { format!("{n}d{sides}{b:+}") } else { format!("{n}d{sides}") };
        let (total, dtext, rolled, _) = dice::roll(&mut self.rng, &notation, false);
        let k = rolled.len() as i32;
        self.roll_event(&format!("{k}d{sides}"), rolled.iter().sum(), k, k * sides, None);
        let dmg = if saved { total / 2 } else { total };
        let p = Pending::Attack {
            attacker: me,
            target,
            hit: true,
            dmg,
            detail: format!("{name}: {stxt}, {dtext}={dmg}"),
            simple: trf!("{name} a {tname}: {dmg}", "{name} on {tname}: {dmg}"),
        };
        self.defer(p);
        if !saved && let Some(buff) = s.get("buff").and_then(Value::as_array) {
            let id = buff[0].as_str().unwrap_or("").to_string();
            self.defer(Pending::Buff { target, id, duration: buff[1].as_f64() });
        }
    }

    fn spell_auto(&mut self, i: usize, s: &Value, name: &str, target: ActorId, extra: i32) {
        let up = s.get("upcast").and_then(|u| u.get("darts")).and_then(Value::as_i64).unwrap_or(0) as i32;
        let darts = s["darts"].as_i64().unwrap_or(1) as i32 + extra * up;
        let dmg = s["damage"].as_str().unwrap_or("1d4");
        let rolls: Vec<i32> = (0..darts).map(|_| dice::roll(&mut self.rng, dmg, false).0).collect();
        let total: i32 = rolls.iter().sum();
        let (n, sides, b) = dice::parse(dmg).unwrap_or((1, 4, 0));
        self.roll_event(&format!("{darts}x{dmg}"), total, darts * (n + b), darts * (n * sides + b), None);
        let tname = self.ch(target).map_or(String::new(), |c| c.name.clone());
        let list: Vec<String> = rolls.iter().map(i32::to_string).collect();
        let p = Pending::Attack {
            attacker: ActorId::Hero(i),
            target,
            hit: true,
            dmg: total,
            detail: trf!("{name}: {darts} dardos ({})={total}", "{name}: {darts} darts ({})={total}", list.join("+")),
            simple: trf!("{name}: {total} a {tname}", "{name}: {total} to {tname}"),
        };
        self.defer(p);
    }

    fn spell_sleep(&mut self, s: &Value, name: &str, cell: i32, extra: i32) {
        let pool_dice = s["pool"].as_str().unwrap_or("5d8");
        let up = s.get("upcast").and_then(|u| u.get("pool")).and_then(Value::as_str).unwrap_or(pool_dice);
        let notation = more(pool_dice, up, extra);
        let (pool, _, rolled, sides) = dice::roll(&mut self.rng, &notation, false);
        let k = rolled.len() as i32;
        self.roll_event(&notation, pool, k, k * sides, None);
        let radius = s["radius"].as_i64().unwrap_or(2) as i32;
        // de menor a mayor vida; en empate, el orden de la lista (determinista)
        let mut victims: Vec<(i32, usize, u32)> = self
            .floor()
            .mobs
            .iter()
            .enumerate()
            .filter(|(_, m)| m.ch.hp > 0 && self.level().distance(m.ch.pos, cell) <= radius)
            .map(|(k, m)| (m.ch.hp, k, m.id))
            .collect();
        victims.sort();
        let victims = victims.into_iter().map(|v| v.2).collect();
        self.defer(Pending::Sleep { victims, pool, name: name.to_string() });
    }

    pub(crate) fn apply_sleep(&mut self, victims: Vec<u32>, pool: i32, name: &str) {
        let mut left = pool;
        let mut slept = Vec::new();
        for id in victims {
            let Some(m) = self.mob_mut(id) else { continue };
            if m.ch.hp > left {
                break;
            }
            left -= m.ch.hp;
            m.state = State::Sleeping;
            m.enemy_seen = false;
            m.alerted = false;
            slept.push(m.ch.name.clone());
            self.status(ActorId::Mob(id), tr!("duerme", "asleep"), Tone::Neutral);
        }
        let msg = if slept.is_empty() {
            tr!("nadie se duerme", "nobody falls asleep").to_string()
        } else {
            trf!("{} se duermen", "{} fall asleep", slept.join(", "))
        };
        self.log(&format!("{name}: {msg}"));
    }

    fn spell_heal(&mut self, i: usize, s: &Value, name: &str, who: ActorId, extra: i32) {
        if let Some(amount) = s.get("amount").and_then(Value::as_i64) {
            self.defer(Pending::Heal { caster: i, who, amount: amount as i32, name: name.to_string() }); // fijo, sin dados
            return;
        }
        let base = s["damage"].as_str().unwrap_or("1d8");
        let up = s.get("upcast").and_then(|u| u.get("damage")).and_then(Value::as_str).unwrap_or(base);
        let notation = more(base, up, extra);
        let (roll, _, rolled, sides) = dice::roll(&mut self.rng, &notation, false);
        let ability = self.heroes[i].spellcasting().and_then(|s| s["ability"].as_str()).unwrap_or("WIS").to_string();
        let m = self.heroes[i].ability_mod(&ability);
        let k = rolled.len() as i32;
        self.roll_event(&notation, rolled.iter().sum(), k, k * sides, None);
        self.defer(Pending::Heal { caster: i, who, amount: roll + m, name: name.to_string() });
    }

    pub(crate) fn apply_heal(&mut self, caster: usize, who: ActorId, amount: i32, name: &str) {
        let ActorId::Hero(j) = who else { return };
        if !self.heroes[j].is_alive() {
            return;
        }
        let heal = self.heal(j, amount);
        self.status(who, &format!("+{heal}"), Tone::Pos);
        let to = if j != caster { trf!(" a {}", " to {}", self.heroes[j].ch.name) } else { String::new() };
        self.log(&trf!("{name}: +{heal} PV{to}", "{name}: +{heal} HP{to}"));
    }

    fn spell_mark(&mut self, i: usize, s: &Value, name: &str, target: ActorId) {
        let id = s["buff"][0].as_str().unwrap_or("").to_string();
        let duration = s["buff"][1].as_f64();
        if let Some(old) = self.heroes[i].marked
            && old != target
            && let Some(uid) = self.ch(old).and_then(|c| c.buffs.iter().find(|b| b.id == id)).map(|b| b.uid)
        {
            self.detach_buff(old, uid);
        }
        self.add_buff(target, &id, duration, 0, 0);
        self.heroes[i].marked = Some(target);
        let tname = self.ch(target).map_or(String::new(), |c| c.name.clone());
        self.log(&trf!("{name}: marcas a {tname}", "{name}: {tname} is marked"));
    }
}
