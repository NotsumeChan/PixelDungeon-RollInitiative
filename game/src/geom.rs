//! Tamaño del mapa (Level.java) y Rect (com.watabou.utils.Rect). El mapa es plano: celda = x + y * WIDTH.

pub const WIDTH: i32 = 32;
pub const HEIGHT: i32 = 32;
pub const LENGTH: i32 = WIDTH * HEIGHT;

pub const NEIGHBOURS4: [i32; 4] = [-WIDTH, 1, WIDTH, -1];
pub const NEIGHBOURS8: [i32; 8] = [1, -1, WIDTH, -WIDTH, 1 + WIDTH, 1 - WIDTH, -1 + WIDTH, -1 - WIDTH];
pub const NEIGHBOURS9: [i32; 9] = [0, 1, -1, WIDTH, -WIDTH, 1 + WIDTH, 1 - WIDTH, -1 + WIDTH, -1 - WIDTH];

/// Ojo: right/bottom son coordenadas de celda (la pared): width = right - left.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn square(&self) -> i32 {
        self.width() * self.height()
    }

    pub fn intersect(&self, o: &Rect) -> Rect {
        Rect::new(self.left.max(o.left), self.top.max(o.top), self.right.min(o.right), self.bottom.min(o.bottom))
    }
}
