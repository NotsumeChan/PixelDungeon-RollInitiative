//! Campo de visión propio: *symmetric shadowcasting* (Albert Ford, 2021, dominio público). Simétrico: si A ve a B, B
//! ve a A (encaja con 5e: si ves al mob, te ve). Las paredes que tocan lo visible también se ven. Pendientes como
//! fracciones enteras (num, den): sin floats, igual en todas las máquinas.

/// Radio de visión (en casillas): hasta acá llega la vista sin luz (ver "Luz y visión 5e" en el roadmap).
pub const MAX_DISTANCE: i32 = 8;

/// Una pendiente num / den (den > 0).
#[derive(Clone, Copy)]
struct Slope(i32, i32);

/// Una fila de un cuadrante a `depth` casillas del centro, entre dos pendientes.
#[derive(Clone, Copy)]
struct Row {
    depth: i32,
    start: Slope,
    end: Slope,
}

impl Row {
    /// Primera y última columna de la fila: redondeo hacia arriba y hacia abajo en los empates (`depth * pendiente`).
    fn cols(&self) -> (i32, i32) {
        let (d, Slope(sn, sd), Slope(en, ed)) = (self.depth, self.start, self.end);
        let first = (2 * d * sn + sd).div_euclid(2 * sd);
        let last = -((ed - 2 * d * en).div_euclid(2 * ed));
        (first, last)
    }

    fn next(&self) -> Row {
        Row { depth: self.depth + 1, ..*self }
    }

    /// ¿La casilla está dentro de la fila (no sólo tocada)? Es lo que la hace simétrica.
    fn symmetric(&self, col: i32) -> bool {
        let (Slope(sn, sd), Slope(en, ed)) = (self.start, self.end);
        col * sd >= self.depth * sn && col * ed <= self.depth * en
    }
}

/// Pendiente del borde izquierdo de una casilla: (2 * col - 1) / (2 * depth).
fn slope(depth: i32, col: i32) -> Slope {
    Slope(2 * col - 1, 2 * depth)
}

struct Scan<'a> {
    fov: Vec<bool>,
    los: &'a [bool],
    w: i32,
    h: i32,
    origin: (i32, i32),
    radius: i32,
    /// Cuadrante: (depth, col) -> (dx, dy).
    quadrant: u8,
}

impl Scan<'_> {
    fn cell(&self, depth: i32, col: i32) -> Option<usize> {
        let (dx, dy) = match self.quadrant {
            0 => (col, -depth), // norte
            1 => (depth, col),  // este
            2 => (col, depth),  // sur
            _ => (-depth, col), // oeste
        };
        let (x, y) = (self.origin.0 + dx, self.origin.1 + dy);
        ((0..self.w).contains(&x) && (0..self.h).contains(&y)).then(|| (x + y * self.w) as usize)
    }

    /// Fuera del mapa cuenta como pared.
    fn wall(&self, c: Option<usize>) -> bool {
        c.is_none_or(|c| self.los[c])
    }

    fn reveal(&mut self, c: Option<usize>, depth: i32, col: i32) {
        // círculo: como un radio redondeado (r² + r), así la vista no es un rombo ni un cuadrado
        if let Some(c) = c
            && depth * depth + col * col <= self.radius * self.radius + self.radius
        {
            self.fov[c] = true;
        }
    }

    fn scan(&mut self, mut row: Row) {
        if row.depth > self.radius {
            return;
        }
        let (first, last) = row.cols();
        let mut prev: Option<bool> = None; // ¿la casilla anterior era pared?
        for col in first..=last {
            let c = self.cell(row.depth, col);
            let wall = self.wall(c);
            if wall || row.symmetric(col) {
                self.reveal(c, row.depth, col);
            }
            if prev == Some(true) && !wall {
                row.start = slope(row.depth, col);
            }
            if prev == Some(false) && wall {
                let mut next = row.next();
                next.end = slope(row.depth, col);
                self.scan(next);
            }
            prev = Some(wall);
        }
        if prev == Some(false) {
            self.scan(row.next());
        }
    }
}

/// Celdas visibles desde la celda `pos` de un mapa de `w` x `h` (`los_blocking`: lo que tapa la vista), hasta
/// `distance` casillas.
pub fn cast_shadow(pos: i32, w: i32, h: i32, los_blocking: &[bool], distance: i32) -> Vec<bool> {
    let mut s = Scan { fov: vec![false; (w * h) as usize], los: los_blocking, w, h, origin: (pos % w, pos / w), radius: distance, quadrant: 0 };
    s.fov[pos as usize] = true;
    for q in 0..4 {
        s.quadrant = q;
        s.scan(Row { depth: 1, start: Slope(-1, 1), end: Slope(1, 1) });
    }
    s.fov
}
