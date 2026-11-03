//! Terrain.java: tipos de celda (el valor es también el índice de tile en tiles0.png) y sus flags.
//! Los nombres y descripciones para examinar llegan con la UI (R4).

pub const CHASM: u8 = 0;
pub const EMPTY: u8 = 1;
pub const GRASS: u8 = 2;
pub const EMPTY_WELL: u8 = 3;
pub const WALL: u8 = 4;
pub const DOOR: u8 = 5;
pub const OPEN_DOOR: u8 = 6;
pub const ENTRANCE: u8 = 7;
pub const EXIT: u8 = 8;
pub const EMBERS: u8 = 9;
pub const LOCKED_DOOR: u8 = 10;
pub const PEDESTAL: u8 = 11;
pub const WALL_DECO: u8 = 12;
pub const BARRICADE: u8 = 13;
pub const EMPTY_SP: u8 = 14;
pub const HIGH_GRASS: u8 = 15;
pub const EMPTY_DECO: u8 = 24;
pub const LOCKED_EXIT: u8 = 25;
pub const UNLOCKED_EXIT: u8 = 26;
pub const SIGN: u8 = 29;
pub const WELL: u8 = 34;
pub const STATUE: u8 = 35;
pub const STATUE_SP: u8 = 36;
pub const BOOKSHELF: u8 = 41;
pub const ALCHEMY: u8 = 42;
pub const CHASM_FLOOR: u8 = 43;
pub const CHASM_FLOOR_SP: u8 = 44;
pub const CHASM_WALL: u8 = 45;
pub const CHASM_WATER: u8 = 46;

pub const SECRET_DOOR: u8 = 16;
pub const TOXIC_TRAP: u8 = 17;
pub const SECRET_TOXIC_TRAP: u8 = 18;
pub const FIRE_TRAP: u8 = 19;
pub const SECRET_FIRE_TRAP: u8 = 20;
pub const PARALYTIC_TRAP: u8 = 21;
pub const SECRET_PARALYTIC_TRAP: u8 = 22;
pub const INACTIVE_TRAP: u8 = 23;
pub const POISON_TRAP: u8 = 27;
pub const SECRET_POISON_TRAP: u8 = 28;
pub const ALARM_TRAP: u8 = 30;
pub const SECRET_ALARM_TRAP: u8 = 31;
pub const LIGHTNING_TRAP: u8 = 32;
pub const SECRET_LIGHTNING_TRAP: u8 = 33;
pub const GRIPPING_TRAP: u8 = 37;
pub const SECRET_GRIPPING_TRAP: u8 = 38;
pub const SUMMONING_TRAP: u8 = 39;
pub const SECRET_SUMMONING_TRAP: u8 = 40;

pub const WATER_TILES: u8 = 48; // 48..63: agua "cosida" con sus bordes (Level.getWaterTile)
pub const WATER: u8 = 63;

pub const PASSABLE: u16 = 0x01;
pub const LOS_BLOCKING: u16 = 0x02;
pub const FLAMABLE: u16 = 0x04;
pub const SECRET: u16 = 0x08;
pub const SOLID: u16 = 0x10;
pub const AVOID: u16 = 0x20;
pub const LIQUID: u16 = 0x40;
pub const PIT: u16 = 0x80;
pub const UNSTITCHABLE: u16 = 0x100;

/// Trampa visible y su versión oculta.
pub const TRAPS: [(u8, u8); 8] = [
    (TOXIC_TRAP, SECRET_TOXIC_TRAP),
    (FIRE_TRAP, SECRET_FIRE_TRAP),
    (PARALYTIC_TRAP, SECRET_PARALYTIC_TRAP),
    (POISON_TRAP, SECRET_POISON_TRAP),
    (ALARM_TRAP, SECRET_ALARM_TRAP),
    (LIGHTNING_TRAP, SECRET_LIGHTNING_TRAP),
    (GRIPPING_TRAP, SECRET_GRIPPING_TRAP),
    (SUMMONING_TRAP, SECRET_SUMMONING_TRAP),
];

const fn build_flags() -> [u16; 256] {
    let mut f = [0u16; 256];
    f[CHASM as usize] = AVOID | PIT | UNSTITCHABLE;
    f[EMPTY as usize] = PASSABLE;
    f[GRASS as usize] = PASSABLE | FLAMABLE;
    f[EMPTY_WELL as usize] = PASSABLE;
    f[WATER as usize] = PASSABLE | LIQUID | UNSTITCHABLE;
    f[WALL as usize] = LOS_BLOCKING | SOLID | UNSTITCHABLE;
    f[DOOR as usize] = PASSABLE | LOS_BLOCKING | FLAMABLE | SOLID | UNSTITCHABLE;
    f[OPEN_DOOR as usize] = PASSABLE | FLAMABLE | UNSTITCHABLE;
    f[ENTRANCE as usize] = PASSABLE;
    f[EXIT as usize] = PASSABLE;
    f[EMBERS as usize] = PASSABLE;
    f[LOCKED_DOOR as usize] = LOS_BLOCKING | SOLID | UNSTITCHABLE;
    f[PEDESTAL as usize] = PASSABLE | UNSTITCHABLE;
    f[WALL_DECO as usize] = f[WALL as usize];
    f[BARRICADE as usize] = FLAMABLE | SOLID | LOS_BLOCKING;
    f[EMPTY_SP as usize] = f[EMPTY as usize] | UNSTITCHABLE;
    f[HIGH_GRASS as usize] = PASSABLE | LOS_BLOCKING | FLAMABLE;
    f[EMPTY_DECO as usize] = f[EMPTY as usize];
    f[LOCKED_EXIT as usize] = SOLID;
    f[UNLOCKED_EXIT as usize] = PASSABLE;
    f[SIGN as usize] = PASSABLE | FLAMABLE;
    f[WELL as usize] = AVOID;
    f[STATUE as usize] = SOLID;
    f[STATUE_SP as usize] = f[STATUE as usize] | UNSTITCHABLE;
    f[BOOKSHELF as usize] = f[BARRICADE as usize] | UNSTITCHABLE;
    f[ALCHEMY as usize] = PASSABLE;
    f[CHASM_WALL as usize] = f[CHASM as usize];
    f[CHASM_FLOOR as usize] = f[CHASM as usize];
    f[CHASM_FLOOR_SP as usize] = f[CHASM as usize];
    f[CHASM_WATER as usize] = f[CHASM as usize];
    f[SECRET_DOOR as usize] = f[WALL as usize] | SECRET | UNSTITCHABLE;
    f[INACTIVE_TRAP as usize] = f[EMPTY as usize];
    let mut i = 0;
    while i < TRAPS.len() {
        f[TRAPS[i].0 as usize] = AVOID;
        f[TRAPS[i].1 as usize] = f[EMPTY as usize] | SECRET;
        i += 1;
    }
    let mut w = WATER_TILES as usize;
    while w < WATER_TILES as usize + 16 {
        f[w] = f[WATER as usize];
        w += 1;
    }
    f
}

pub const FLAGS: [u16; 256] = build_flags();

pub fn flags(t: u8) -> u16 {
    FLAGS[t as usize]
}

/// Terreno secreto -> su versión descubierta.
pub fn discover(t: u8) -> u8 {
    if t == SECRET_DOOR {
        return DOOR;
    }
    TRAPS.iter().find(|(_, s)| *s == t).map_or(t, |(v, _)| *v)
}

/// Level.Feeling: tema del piso, elegido al crear el nivel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feeling {
    None,
    Chasm,
    Water,
    Grass,
}

impl Feeling {
    pub fn name(self) -> &'static str {
        match self {
            Feeling::None => "NONE",
            Feeling::Chasm => "CHASM",
            Feeling::Water => "WATER",
            Feeling::Grass => "GRASS",
        }
    }
}
