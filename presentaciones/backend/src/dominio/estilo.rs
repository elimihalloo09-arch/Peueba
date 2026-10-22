//! La identidad institucional, sacada del deck "Avance de proyectos" que ya se
//! presenta: Montserrat, portada guinda con texto blanco, títulos guinda en
//! diapositivas blancas. Paleta del Gobierno de México 2024–2030.
//!
//! Todas las medidas van en puntos sobre una diapositiva de 16:9 a 10 × 5.625
//! pulgadas, el mismo tamaño que Google Slides.

use std::sync::OnceLock;

pub const ANCHO: f32 = 720.0;
pub const ALTO: f32 = 405.0;
pub const MARGEN: f32 = 48.0;
/// Ancho útil del texto: el mismo de las cajas del deck original.
pub const ANCHO_TEXTO: f32 = 624.0;
/// Sangría colgante de las viñetas.
pub const SANGRIA: f32 = 18.0;
/// Hasta dónde puede bajar el contenido antes del pie.
pub const LIMITE: f32 = 362.0;
pub const Y_PIE: f32 = 380.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub fn hex(self) -> String {
        format!("{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
    pub fn rgb(self) -> (f32, f32, f32) {
        (self.0 as f32 / 255.0, self.1 as f32 / 255.0, self.2 as f32 / 255.0)
    }
}

pub const GUINDA: Color = Color(0x61, 0x12, 0x32);
pub const GUINDA_CLARO: Color = Color(0x9B, 0x22, 0x47);
pub const DORADO: Color = Color(0xA5, 0x7F, 0x2C);
/// Dorado oscurecido para texto: el #A57F2C no llega a 4.5:1 sobre blanco.
pub const DORADO_TEXTO: Color = Color(0x86, 0x66, 0x1F);
pub const VERDE: Color = Color(0x1E, 0x5B, 0x4F);
pub const BLANCO: Color = Color(0xFF, 0xFF, 0xFF);
/// Texto secundario sobre la portada guinda.
pub const ROSA_PALIDO: Color = Color(0xE8, 0xD9, 0xDF);
pub const TINTA: Color = Color(0x1F, 0x1F, 0x1F);
pub const GRIS: Color = Color(0x5F, 0x5F, 0x5F);
pub const GRIS_CLARO: Color = Color(0x98, 0x98, 0x9A);
pub const LINEA: Color = Color(0xE3, 0xDD, 0xD3);

pub const FUENTE: &str = "Montserrat";
pub static REGULAR: &[u8] = include_bytes!("../../fuentes/Montserrat-Regular.ttf");
pub static NEGRITA: &[u8] = include_bytes!("../../fuentes/Montserrat-Bold.ttf");

/// Mide texto con las métricas reales de Montserrat, para partir renglones
/// igual en el PDF que en el PowerPoint.
pub fn medir(texto: &str, tam: f32, negrita: bool) -> f32 {
    static CARAS: OnceLock<(ttf_parser::Face<'static>, ttf_parser::Face<'static>)> = OnceLock::new();
    let (reg, neg) = CARAS.get_or_init(|| {
        (
            ttf_parser::Face::parse(REGULAR, 0).expect("Montserrat Regular"),
            ttf_parser::Face::parse(NEGRITA, 0).expect("Montserrat Bold"),
        )
    });
    let cara = if negrita { neg } else { reg };
    let em = cara.units_per_em() as f32;
    let unidades: f32 = texto
        .chars()
        .map(|c| {
            cara.glyph_index(c)
                .and_then(|g| cara.glyph_hor_advance(g))
                .map(f32::from)
                .unwrap_or(em * 0.6)
        })
        .sum();
    unidades / em * tam
}
