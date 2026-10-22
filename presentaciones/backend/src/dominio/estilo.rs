//! La identidad de la plantilla institucional 2026 (Plantilla_Presentacion2026):
//! Noto Sans, títulos guinda #A02142, subtítulos #691B32, fondos con grecas y
//! logos de Hacienda | SAT. Las medidas salen de sus cuadros de texto.
//!
//! Todo va en puntos sobre la diapositiva de 13.33 × 7.5 pulgadas de la plantilla.

use std::sync::OnceLock;

pub const ANCHO: f32 = 960.0;
pub const ALTO: f32 = 540.0;
/// Margen izquierdo de los títulos y el texto en la plantilla.
pub const MARGEN: f32 = 23.1;
/// Hasta dónde llega el contenido a la derecha: donde terminan los logos.
pub const DERECHA: f32 = 933.0;
pub const ANCHO_TEXTO: f32 = DERECHA - MARGEN;
/// El título no puede pasar debajo de los logos, que empiezan en x = 612.
pub const ANCHO_TITULO: f32 = 580.0;
/// Sangría colgante de las viñetas.
pub const SANGRIA: f32 = 22.0;

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

/// Títulos.
pub const GUINDA: Color = Color(0xA0, 0x21, 0x42);
/// Fecha de la portada y subtítulos.
pub const GUINDA_OSCURO: Color = Color(0x69, 0x1B, 0x32);
/// Texto de cierre.
pub const GUINDA_CIERRE: Color = Color(0x9C, 0x21, 0x48);
pub const DORADO: Color = Color(0xAE, 0x83, 0x20);
/// Dorado oscurecido para texto: el #AE8320 no llega a 4.5:1 sobre el fondo claro.
pub const DORADO_TEXTO: Color = Color(0x85, 0x63, 0x14);
pub const VERDE: Color = Color(0x23, 0x5C, 0x4E);
pub const TINTA: Color = Color(0x21, 0x21, 0x21);
pub const GRIS: Color = Color(0x5F, 0x5F, 0x5F);
pub const GRIS_CLARO: Color = Color(0x98, 0x98, 0x9A);
pub const LINEA: Color = Color(0xD9, 0xD2, 0xC8);
/// Fondo de la fila de totales: el crema de la paleta.
pub const CREMA: Color = Color(0xFC, 0xF7, 0xEC);

/// Los cuatro pesos de Noto Sans que usa la plantilla.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Peso {
    Regular,
    Medio,
    Seminegrita,
    Negrita,
}

impl Peso {
    /// Cómo se llama en PowerPoint, y si además lleva negrita.
    pub fn fuente(self) -> (&'static str, bool) {
        match self {
            Peso::Regular => ("Noto Sans", false),
            Peso::Medio => ("Noto Sans Medium", false),
            Peso::Seminegrita => ("Noto Sans SemiBold", false),
            Peso::Negrita => ("Noto Sans", true),
        }
    }

    pub fn datos(self) -> &'static [u8] {
        match self {
            Peso::Regular => REGULAR,
            Peso::Medio => MEDIO,
            Peso::Seminegrita => SEMINEGRITA,
            Peso::Negrita => NEGRITA,
        }
    }

    pub const TODOS: [Peso; 4] = [Peso::Regular, Peso::Medio, Peso::Seminegrita, Peso::Negrita];
}

pub static REGULAR: &[u8] = include_bytes!("../../fuentes/NotoSans-Regular.ttf");
pub static MEDIO: &[u8] = include_bytes!("../../fuentes/NotoSans-Medium.ttf");
pub static SEMINEGRITA: &[u8] = include_bytes!("../../fuentes/NotoSans-SemiBold.ttf");
pub static NEGRITA: &[u8] = include_bytes!("../../fuentes/NotoSans-Bold.ttf");

/// Noto Sans a renglón sencillo mide 1.362 del tamaño (ascendente 1.069 +
/// descendente 0.293). La plantilla usa interlineado de 90 %.
pub const INTERLINEADO: f32 = 0.9;

/// Alto de un renglón a `tam` con el interlineado de la plantilla.
pub fn alto_renglon(tam: f32) -> f32 {
    tam * 1.362 * INTERLINEADO
}

/// Distancia de la parte de arriba del renglón a la línea base.
pub fn linea_base(tam: f32) -> f32 {
    alto_renglon(tam) - tam * 0.293
}

/// Mide texto con las métricas reales de Noto Sans, para partir renglones
/// igual en el PDF que en el PowerPoint.
pub fn medir(texto: &str, tam: f32, peso: Peso) -> f32 {
    static CARAS: OnceLock<Vec<ttf_parser::Face<'static>>> = OnceLock::new();
    let caras = CARAS.get_or_init(|| {
        Peso::TODOS.iter().map(|p| ttf_parser::Face::parse(p.datos(), 0).expect("Noto Sans")).collect()
    });
    let cara = &caras[Peso::TODOS.iter().position(|p| *p == peso).unwrap_or(0)];
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

/// El tamaño más grande, de `tam` a `minimo`, en el que `texto` cabe en `ancho`.
pub fn tam_que_cabe(texto: &str, ancho: f32, tam: f32, minimo: f32, peso: Peso) -> f32 {
    let mut t = tam;
    while t > minimo && medir(texto, t, peso) > ancho {
        t -= 1.0;
    }
    t
}
