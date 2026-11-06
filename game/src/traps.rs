//! levels/traps/*.java + Level.press / mobPress (`game/traps.py`): las 8 trampas de las Alcantarillas, con
//! salvaciones de 5e. Los efectos aplican estados de `data/buffs.json` por id.
//! ponytail: el gas tóxico, el paralizante y el fuego no forman nubes que se expanden (Blob): afectan una vez al área
//! 3x3 alrededor de la trampa. Agregar Blob cuando lleguen las pociones de gas.
use crate::actor::ActorId;
use crate::dice::{ability_names, ABILITIES};
use crate::dungeon::{Dungeon, Pending, Tone};
use crate::mobs::{pick_mob, State};
use crate::terrain as t;

/// Los invocados actúan después de este tiempo.
const SUMMON_DELAY: f64 = 2.0;

/// CD de las salvaciones contra trampas: 11 en los pisos 2-3, 12 en el 4 (trampas "molestas" de la DMG).
pub fn trap_dc(depth: i32) -> i32 {
    10 + depth / 2
}

/// Lo que hace una trampa según la salvación, cuando el dado ya se vio. No usa el rng.
#[derive(Clone, Copy, Debug)]
pub enum TrapEffect {
    /// Gas tóxico: falla = envenenado n turnos, supera = la mitad (si la regla `save_half` lo permite).
    Toxic(i32),
    Fire,
    /// Gas paralizante: falla = paralizado (repite la salvación con esta CD cada turno).
    Paralysis(i32),
    /// Dardo envenenado: falla = envenenado n turnos.
    Dart(i32),
    /// Descarga: daño; supera = la mitad (o nada, según `save_half`).
    Lightning(i32),
    /// Garras: falla = sangrando (nivel) y lisiado.
    Gripping(i32),
}

impl TrapEffect {
    /// Con "mitad si supera" (regla de la partida `save_half`).
    fn half(self) -> bool {
        matches!(self, TrapEffect::Toxic(_) | TrapEffect::Lightning(_))
    }
}

impl Dungeon {
    /// Level.press (héroe) / mobPress (mobs): los mobs sólo activan trampas visibles. La trampa queda inactiva.
    pub(crate) fn press_trap(&mut self, cell: i32, who: ActorId) {
        let terr = self.level().map[cell as usize];
        let trap = t::discover(terr);
        if !matches!(
            trap,
            t::TOXIC_TRAP | t::FIRE_TRAP | t::PARALYTIC_TRAP | t::POISON_TRAP | t::ALARM_TRAP | t::LIGHTNING_TRAP | t::GRIPPING_TRAP | t::SUMMONING_TRAP
        ) {
            return;
        }
        let hero = matches!(who, ActorId::Hero(_));
        if terr != trap {
            if !hero {
                return;
            }
            self.log(crate::tr!("Una placa oculta hace click", "A hidden pressure plate clicks"));
        }
        self.floor_mut().level.set(cell, t::INACTIVE_TRAP);
        if let ActorId::Hero(i) = who {
            self.heroes[i].action = None;
        }
        // ponytail: sin partículas de trampas todavía (gas, llamas, rayo); el log y los dados avisan
        let depth = self.depth;
        match trap {
            t::TOXIC_TRAP => {
                self.say(cell, crate::tr!("Una nube de gas tóxico brota de la trampa", "A cloud of toxic gas bursts from the trap"));
                for c in self.area(cell) {
                    self.trap_save(c, "CON", TrapEffect::Toxic(4 + depth));
                }
            }
            t::FIRE_TRAP => {
                self.say(cell, crate::tr!("La trampa estalla en llamas", "The trap bursts into flames"));
                for c in self.area(cell) {
                    self.trap_save(c, "DEX", TrapEffect::Fire);
                }
            }
            t::PARALYTIC_TRAP => {
                self.say(cell, crate::tr!("Una nube de gas paralizante brota de la trampa", "A cloud of paralytic gas bursts from the trap"));
                for c in self.area(cell) {
                    self.trap_save(c, "CON", TrapEffect::Paralysis(trap_dc(depth)));
                }
            }
            t::POISON_TRAP => {
                self.say(cell, crate::tr!("Un dardo envenenado sale disparado", "A poison dart shoots out"));
                self.trap_save(who, "CON", TrapEffect::Dart(4 + depth / 2));
            }
            t::ALARM_TRAP => {
                // sin salvación: todos los mobs del piso van hacia la trampa (Mob.beckon)
                for m in &mut self.floor_mut().mobs {
                    if ActorId::Mob(m.id) != who {
                        if m.state != State::Hunting {
                            m.state = State::Wandering;
                        }
                        m.target = cell;
                    }
                }
                self.say(cell, crate::tr!("La trampa suelta un chillido que resuena por toda la mazmorra", "The trap emits a piercing shriek that echoes through the dungeon"));
            }
            t::LIGHTNING_TRAP => {
                // entre 1/3 y 2/3 de la vida actual
                let hp = self.ch(who).map_or(0, |c| c.hp);
                let dmg = self.rng.int_range_incl(hp / 3, 2 * hp / 3).max(1);
                self.say(cell, crate::tr!("Una descarga eléctrica sale de la trampa", "A bolt of lightning leaps from the trap"));
                self.trap_save(who, "DEX", TrapEffect::Lightning(dmg));
            }
            t::GRIPPING_TRAP => {
                self.say(cell, crate::tr!("Unas garras de metal se cierran sobre la trampa", "Metal claws snap shut over the trap"));
                self.trap_save(who, "DEX", TrapEffect::Gripping(depth + 3));
            }
            t::SUMMONING_TRAP => {
                // sin salvación: 1 a 3 mobs del piso aparecen alrededor, despiertos
                let mut n = 1;
                if self.rng.int(2) == 0 {
                    n += 1;
                    if self.rng.int(2) == 0 {
                        n += 1;
                    }
                }
                let level = self.level();
                let mut cands: Vec<i32> = level
                    .n8()
                    .iter()
                    .map(|o| cell + o)
                    .filter(|&p| self.find_char(p).is_none() && (level.passable[p as usize] || level.avoid[p as usize]))
                    .collect();
                while n > 0 && !cands.is_empty() {
                    let p = cands.remove(self.rng.int(cands.len() as i32) as usize);
                    let key = pick_mob(&self.bestiary, &mut self.rng, depth);
                    self.add_mob_delayed(&key, p, State::Wandering, SUMMON_DELAY);
                    n -= 1;
                }
                self.say(cell, crate::tr!("Aparecen monstruos de la nada", "Monsters appear out of nowhere"));
            }
            _ => {}
        }
    }

    fn say(&mut self, cell: i32, msg: &str) {
        if self.level().visible[cell as usize] {
            self.log(msg);
        }
    }

    /// Los personajes vivos en el 3x3 alrededor de la celda.
    fn area(&self, cell: i32) -> Vec<ActorId> {
        self.chars().into_iter().filter(|&(a, p)| self.alive(a) && p >= 0 && self.level().distance(p, cell) <= 1).map(|(a, _)| a).collect()
    }

    /// Tira la salvación ya (el orden del rng no depende del render) y aplica el efecto cuando la animación del dado
    /// la muestra (pending); lo que no se ve, en el acto.
    fn trap_save(&mut self, who: ActorId, ability: &'static str, effect: TrapEffect) {
        let (ok, detail) = self.saving_throw(who, ability, crate::traps::trap_dc(self.depth), crate::tr!("trampa", "trap"));
        let pos = self.ch(who).map_or(-1, |c| c.pos);
        let seen = pos >= 0 && self.sees(pos);
        let p = Pending::TrapSave { who, ability, ok, detail, seen, effect };
        if seen {
            self.pending.push(p);
        } else {
            self.apply_trap_save(p);
        }
    }

    pub(crate) fn apply_trap_save(&mut self, p: Pending) {
        let Pending::TrapSave { who, ability, ok, detail, seen, effect } = p else { return };
        if !self.alive(who) {
            return;
        }
        let short = ABILITIES.iter().position(|x| *x == ability).map_or(ability, |k| ability_names()[k]);
        let half = self.rules.save_half;
        if seen {
            let name = self.ch(who).map_or(String::new(), |c| c.name.clone());
            let mut msg = if ok {
                crate::trf!("{name} supera la salvación de {short}", "{name} succeeds on a {short} saving throw")
            } else {
                crate::trf!("{name} falla la salvación de {short}", "{name} fails a {short} saving throw")
            };
            if ok && effect.half() {
                msg += if half { crate::tr!(": la mitad del efecto", ": half effect") } else { crate::tr!(": sin efecto", ": no effect") };
            }
            let line = if self.verbose_rolls { format!("{msg} ({detail})") } else { msg };
            self.log(&line);
            let tone = if ok { Tone::Pos } else { Tone::Neg };
            self.status(who, &if ok { crate::trf!("{short} supera", "{short} save") } else { crate::trf!("{short} falla", "{short} fail") }, tone);
        }
        match effect {
            TrapEffect::Toxic(n) if !ok || half => self.add_buff(who, "pdri:poison", Some(if ok { n / 2 } else { n } as f64), 0, 0),
            TrapEffect::Fire if !ok => self.add_buff(who, "pdri:burning", None, 0, 0),
            TrapEffect::Paralysis(dc) if !ok => self.add_buff(who, "pdri:paralysis", None, 0, dc),
            TrapEffect::Dart(n) if !ok => self.add_buff(who, "pdri:poison", Some(n as f64), 0, 0),
            TrapEffect::Lightning(dmg) => {
                let d = if !ok { dmg } else if half { dmg / 2 } else { 0 };
                self.damage(who, d, Some(ActorId::Trap));
            }
            TrapEffect::Gripping(level) if !ok => {
                self.add_buff(who, "pdri:bleeding", None, level, 0);
                self.add_buff(who, "pdri:cripple", None, 0, 0);
            }
            _ => {}
        }
    }
}
