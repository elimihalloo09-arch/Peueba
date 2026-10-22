//! Dibuja la maqueta en PDF con Montserrat incrustada: se ve igual en
//! cualquier computadora, tenga o no la fuente instalada.

use std::io::Cursor;

use printpdf::{
    path::PaintMode, Color as ColorPdf, IndirectFontRef, Mm, PdfDocument, PdfLayerReference, Pt, Rect, Rgb,
};

use super::{
    estilo::*,
    maqueta::{self, Contenido, Diapositiva, Trozo},
};

struct Lienzo<'a> {
    capa: PdfLayerReference,
    regular: &'a IndirectFontRef,
    negrita: &'a IndirectFontRef,
}

impl Lienzo<'_> {
    fn color(&self, c: Color) {
        let (r, g, b) = c.rgb();
        self.capa.set_fill_color(ColorPdf::Rgb(Rgb::new(r, g, b, None)));
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        self.color(c);
        let r = Rect::new(mm(x), mm(ALTO - y - h), mm(x + w), mm(ALTO - y)).with_mode(PaintMode::Fill);
        self.capa.add_rect(r);
    }

    /// `y` es la parte de arriba del renglón, como en las cajas de texto.
    fn texto(&self, s: &str, tam: f32, x: f32, y: f32, negrita: bool, c: Color) {
        if s.is_empty() {
            return;
        }
        self.color(c);
        let fuente = if negrita { self.negrita } else { self.regular };
        self.capa.use_text(s, tam, mm(x), mm(ALTO - y - tam * 0.95), fuente);
    }

    fn renglon(&self, trozos: &[Trozo], tam: f32, x: f32, y: f32) {
        let mut x = x;
        for t in trozos {
            self.texto(&t.texto, tam, x, y, t.negrita, t.color);
            x += medir(&t.texto, tam, t.negrita);
        }
    }

    fn derecha(&self, s: &str, tam: f32, x_der: f32, y: f32, negrita: bool, c: Color) {
        self.texto(s, tam, x_der - medir(s, tam, negrita), y, negrita, c);
    }

    fn centrado(&self, s: &str, tam: f32, x_centro: f32, y: f32, negrita: bool, c: Color) {
        self.texto(s, tam, x_centro - medir(s, tam, negrita) / 2.0, y, negrita, c);
    }
}

fn mm(pt: f32) -> Mm {
    Mm::from(Pt(pt))
}

pub fn generar(diapositivas: &[Diapositiva], titulo: &str) -> Result<Vec<u8>, String> {
    let (doc, pag, capa) = PdfDocument::new(titulo, mm(ANCHO), mm(ALTO), "diapositiva");
    let regular = doc.add_external_font(Cursor::new(REGULAR)).map_err(|e| e.to_string())?;
    let negrita = doc.add_external_font(Cursor::new(NEGRITA)).map_err(|e| e.to_string())?;

    for (i, d) in diapositivas.iter().enumerate() {
        let (p, c) = if i == 0 { (pag, capa) } else { doc.add_page(mm(ANCHO), mm(ALTO), "diapositiva") };
        let l = Lienzo { capa: doc.get_page(p).get_layer(c), regular: &regular, negrita: &negrita };
        dibujar(&l, d);
    }
    doc.save_to_bytes().map_err(|e| e.to_string())
}

fn dibujar(l: &Lienzo, d: &Diapositiva) {
    match &d.contenido {
        Contenido::Portada { titulo, subtitulo, proyectos, presenta } => {
            l.rect(0.0, 0.0, ANCHO, ALTO, GUINDA);
            l.rect(0.0, ALTO - 6.0, ANCHO, 6.0, DORADO);
            l.texto(&titulo.texto, titulo.tam, MARGEN, 77.0, true, BLANCO);
            l.rect(MARGEN, 138.0, 56.0, 3.0, DORADO);
            l.texto(subtitulo, 22.0, MARGEN, 158.0, false, BLANCO);
            let mut y = 247.0;
            for r in proyectos {
                l.texto(r, 16.0, MARGEN, y, false, ROSA_PALIDO);
                y += 20.0;
            }
            l.texto(presenta, 16.0, MARGEN, y.max(267.0) + 28.0, false, ROSA_PALIDO);
        }
        Contenido::Resumen { titulo, filas } => {
            l.texto(&titulo.texto, titulo.tam, MARGEN, maqueta::Y_TITULO, true, GUINDA);
            let cols = columnas_resumen();
            let y0 = maqueta::Y_TABLA;
            l.texto("PROYECTO", 9.0, MARGEN, y0, true, GRIS);
            for (i, e) in crate::dominio::EstadoAvance::TODOS.iter().enumerate() {
                l.centrado(&e.etiqueta().to_uppercase(), 9.0, cols[i], y0, true, GRIS);
            }
            l.rect(MARGEN, y0 + 16.0, ANCHO_TEXTO, 1.2, GUINDA);
            let mut y = y0 + 24.0;
            for f in filas {
                if f.total {
                    l.rect(MARGEN, y - 4.0, ANCHO_TEXTO, ALTO_FILA_TOTAL, Color(0xF6, 0xF1, 0xEA));
                }
                l.texto(&f.nombre, 13.0, MARGEN + 8.0, if f.equipo.is_empty() { y + 8.0 } else { y + 2.0 }, true, if f.total { GUINDA } else { TINTA });
                if !f.equipo.is_empty() {
                    l.texto(&f.equipo, 9.5, MARGEN + 8.0, y + 19.0, false, GRIS);
                }
                for (i, e) in crate::dominio::EstadoAvance::TODOS.iter().enumerate() {
                    let n = f.cuentas[i];
                    let (s, c) = if n == 0 { ("—".to_string(), GRIS_CLARO) } else { (n.to_string(), e.color()) };
                    l.centrado(&s, 16.0, cols[i], y + 6.0, true, c);
                }
                y += maqueta::ALTO_FILA;
                if !f.total {
                    l.rect(MARGEN, y - 5.0, ANCHO_TEXTO, 0.6, LINEA);
                }
            }
        }
        Contenido::Lista { titulo, subtitulo, cuerpo, nota } => {
            l.texto(&titulo.texto, titulo.tam, MARGEN, maqueta::Y_TITULO, true, GUINDA);
            if let Some(s) = subtitulo {
                l.texto(s, 16.0, MARGEN, maqueta::Y_SUBTITULO, false, GRIS);
            }
            let lh = cuerpo.interlineado();
            for p in &cuerpo.parrafos {
                l.texto("•", cuerpo.tam, MARGEN, p.y, true, GUINDA);
                for (i, r) in p.renglones.iter().enumerate() {
                    l.renglon(r, cuerpo.tam, MARGEN + SANGRIA, p.y + i as f32 * lh);
                }
            }
            if let Some(n) = nota {
                l.texto(n, 16.0, MARGEN, LIMITE - 16.0, false, GRIS);
            }
        }
    }
    if let Some((texto, numero)) = &d.pie {
        l.rect(MARGEN, Y_PIE - 8.0, ANCHO_TEXTO, 0.6, LINEA);
        l.rect(MARGEN, Y_PIE - 8.6, 40.0, 1.8, DORADO);
        l.texto(texto, 8.0, MARGEN, Y_PIE, false, GRIS);
        l.derecha(numero, 8.0, MARGEN + ANCHO_TEXTO, Y_PIE, true, GUINDA);
    }
}

pub const ALTO_FILA_TOTAL: f32 = 34.0;

/// Centro de las columnas Concluido, En proceso y Pendiente del resumen.
pub fn columnas_resumen() -> [f32; 3] {
    let fin = MARGEN + ANCHO_TEXTO;
    [fin - 250.0, fin - 150.0, fin - 50.0]
}
