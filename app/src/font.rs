//! Fuente bitmap del original (BitmapText.Font.colorMarked): glifos en una tira, separados por columnas
//! transparentes. Las columnas vacías del principio definen el ancho del espacio. Port de `core/Font.py`.
use crate::atlas::Atlas;
use crate::gpu::Sprite;

const CHARS: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\x7f";
const TRACKING: f32 = 1.0;

pub struct Font {
    aw: f32, // tamaño del atlas, para las coordenadas de textura
    ah: f32,
    x0: u32,
    y0: u32,
    h: u32,
    space: u32,
    glyphs: [Option<(u32, u32)>; 128], // ascii -> (x en la tira, ancho)
}

/// font1x sólo tiene ASCII: los acentos se quitan (á -> a, ñ -> n).
fn ascii(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'Á' => 'A', 'É' => 'E', 'Í' => 'I', 'Ó' => 'O', 'Ú' => 'U', 'Ñ' => 'N',
        '¡' => '!', '¿' => '?',
        c => c,
    }
}

impl Font {
    pub fn new(atlas: &Atlas, sheet: usize) -> Self {
        let [x0, y0, w, h] = atlas.rects[sheet];
        let empty: Vec<bool> = (0..w).map(|x| (0..h).all(|y| atlas.alpha(x0 + x, y0 + y) == 0)).collect();
        let mut glyphs = [None; 128];
        let mut pos = 0;
        while pos < w && empty[pos as usize] {
            pos += 1;
        }
        let space = pos;
        for c in CHARS.chars().skip(1) {
            if pos >= w {
                break;
            }
            let mut sep = pos + 1;
            while sep < w && !empty[sep as usize] {
                sep += 1;
            }
            glyphs[c as usize] = Some((pos, sep - pos));
            pos = sep + 1;
        }
        Self { aw: atlas.w as f32, ah: atlas.h as f32, x0, y0, h, space, glyphs }
    }

    /// Agrega el texto como sprites (uno por glifo) en (x, y), pixeles del juego. Devuelve el ancho.
    pub fn text(&self, out: &mut Vec<Sprite>, text: &str, x: f32, y: f32, color: [f32; 4]) -> f32 {
        self.text_scaled(out, text, x, y, color, 1.0)
    }

    /// `text` con cada pixel de la fuente de `scale` x `scale` (entero: nítido).
    pub fn text_scaled(&self, out: &mut Vec<Sprite>, text: &str, x: f32, y: f32, color: [f32; 4], scale: f32) -> f32 {
        let mut cx = x;
        for c in text.chars().map(ascii) {
            if c == ' ' {
                cx += (self.space as f32 + TRACKING) * scale;
                continue;
            }
            let Some((gx, gw)) = self.glyphs.get(c as usize).copied().flatten() else { continue };
            let (u, v) = ((self.x0 + gx) as f32 / self.aw, self.y0 as f32 / self.ah);
            out.push(Sprite {
                pos: [cx, y],
                size: [gw as f32 * scale, self.h as f32 * scale],
                uv: [u, v, u + gw as f32 / self.aw, v + self.h as f32 / self.ah],
                color,
            });
            cx += (gw as f32 + TRACKING) * scale;
        }
        cx - x - TRACKING * scale
    }

    /// Ancho del texto en pixeles del juego (sin dibujarlo).
    pub fn width(&self, text: &str) -> f32 {
        let mut w = 0.0;
        for c in text.chars().map(ascii) {
            if c == ' ' {
                w += self.space as f32 + TRACKING;
            } else if let Some((_, gw)) = self.glyphs.get(c as usize).copied().flatten() {
                w += gw as f32 + TRACKING;
            }
        }
        (w - TRACKING).max(0.0)
    }

    pub fn height(&self) -> f32 {
        self.h as f32
    }
}
