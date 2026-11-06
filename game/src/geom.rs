//! Vecinos de una celda y Rect (com.watabou.utils.Rect). El mapa es plano: celda = x + y * ancho. Cada nivel tiene su
//! tamaño (`Level::width` / `height`, M1a): no hay tamaño global.

/// Mayor ancho o alto de una zona (la niebla de la GPU se reserva para esto).
pub const MAX_SIZE: i32 = 128;

pub fn n4(w: i32) -> [i32; 4] {
    [-w, 1, w, -1]
}

pub fn n8(w: i32) -> [i32; 8] {
    [1, -1, w, -w, 1 + w, 1 - w, -1 + w, -1 - w]
}

pub fn n9(w: i32) -> [i32; 9] {
    [0, 1, -1, w, -w, 1 + w, 1 - w, -1 + w, -1 - w]
}

/// Level.distance: distancia de Chebyshev (las diagonales cuentan 1) en un mapa de ancho `w`.
pub fn distance(w: i32, a: i32, b: i32) -> i32 {
    ((a % w) - (b % w)).abs().max(((a / w) - (b / w)).abs())
}

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
