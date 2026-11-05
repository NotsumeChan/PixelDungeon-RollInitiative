//! Un solo atlas de texturas: las hojas PNG apiladas en vertical. Todo el frame se dibuja con esta textura.
use crate::gpu::Sprite;

pub struct Atlas {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
    /// Dónde quedó cada hoja: [x, y, ancho, alto], en el orden en que se pasaron.
    pub rects: Vec<[u32; 4]>,
    /// Un pixel blanco (debajo de las hojas): rectángulos de color liso con el mismo lote.
    white: (u32, u32),
}

/// PNG -> RGBA8 (paleta, gris, 16 bits y sin alfa se normalizan).
/// Un PNG que se puede usar de ícono de mod (se decodifica y mide 64x64 o menos).
pub fn icon_ok(bytes: &[u8]) -> bool {
    png::Decoder::new(std::io::Cursor::new(bytes)).read_info().is_ok_and(|r| r.info().width <= 64 && r.info().height <= 64)
}

pub fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16 | png::Transformations::ALPHA);
    let mut reader = dec.read_info().expect("png");
    let mut buf = vec![0; reader.output_buffer_size().expect("png demasiado grande")];
    let info = reader.next_frame(&mut buf).expect("png");
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::GrayscaleAlpha => buf.chunks(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        other => panic!("formato de png no esperado: {other:?}"),
    };
    (info.width, info.height, rgba)
}

impl Atlas {
    /// ponytail: apilado vertical simple; si alguna GPU se queda corta de alto (max_texture_dimension_2d),
    /// empaquetar en estantes.
    pub fn build(sheets: &[&[u8]]) -> Self {
        let images: Vec<_> = sheets.iter().map(|b| decode(b)).collect();
        let w = images.iter().map(|i| i.0).max().unwrap_or(1);
        let h = images.iter().map(|i| i.1).sum::<u32>() + 1; // + la fila del pixel blanco
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        let mut rects = Vec::new();
        let mut y = 0;
        for (iw, ih, px) in &images {
            for row in 0..*ih {
                let src = (row * iw * 4) as usize;
                let dst = (((y + row) * w) * 4) as usize;
                rgba[dst..dst + (*iw * 4) as usize].copy_from_slice(&px[src..src + (*iw * 4) as usize]);
            }
            rects.push([0, y, *iw, *ih]);
            y += ih;
        }
        let white = (0, y);
        rgba[((y * w) * 4) as usize..((y * w) * 4 + 4) as usize].copy_from_slice(&[255; 4]);
        Self { w, h, rgba, rects, white }
    }

    /// Alfa del pixel (x, y) del atlas.
    pub fn alpha(&self, x: u32, y: u32) -> u8 {
        self.rgba[((y * self.w + x) * 4 + 3) as usize]
    }

    /// Un rectángulo de color liso (el pixel blanco teñido).
    pub fn rect(&self, pos: [f32; 2], size: [f32; 2], color: [f32; 4]) -> Sprite {
        let (u, v) = ((self.white.0 as f32 + 0.5) / self.w as f32, (self.white.1 as f32 + 0.5) / self.h as f32);
        Sprite { pos, size, uv: [u, v, u, v], color }
    }

    /// Un recorte del atlas dibujado en `pos` (pixeles del juego), del mismo tamaño que el recorte.
    pub fn sprite(&self, x: u32, y: u32, w: u32, h: u32, pos: [f32; 2], color: [f32; 4]) -> Sprite {
        let (aw, ah) = (self.w as f32, self.h as f32);
        Sprite {
            pos,
            size: [w as f32, h as f32],
            uv: [x as f32 / aw, y as f32 / ah, (x + w) as f32 / aw, (y + h) as f32 / ah],
            color,
        }
    }
}
