//! Dibuja la maqueta en PDF con Noto Sans incrustada y, si hay plantilla, el
//! fondo de cada diseño con sus logos: se ve igual en cualquier computadora.

use std::{collections::HashMap, io::Cursor};

use printpdf::{
    path::PaintMode, Color as ColorPdf, ColorBits, ColorSpace, Image, ImageFilter, ImageTransform, ImageXObject,
    IndirectFontRef, Mm, PdfDocument, PdfLayerReference, Pt, Px, Rect, Rgb,
};

use super::{
    estilo::*,
    maqueta::{self, Bloque, Contenido, Diapositiva, Trozo},
    plantilla::{Fondo, Plantilla},
    EstadoAvance,
};

struct Lienzo<'a> {
    capa: PdfLayerReference,
    fuentes: &'a HashMap<Peso, IndirectFontRef>,
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
    fn texto(&self, s: &str, tam: f32, x: f32, y: f32, peso: Peso, c: Color) {
        if s.is_empty() {
            return;
        }
        self.color(c);
        self.capa.use_text(s, tam, mm(x), mm(ALTO - y - linea_base(tam)), &self.fuentes[&peso]);
    }

    fn renglon(&self, trozos: &[Trozo], tam: f32, x: f32, y: f32) {
        let mut x = x;
        for t in trozos {
            self.texto(&t.texto, tam, x, y, t.peso, t.color);
            x += medir(&t.texto, tam, t.peso);
        }
    }

    fn centrado(&self, s: &str, tam: f32, x_centro: f32, y: f32, peso: Peso, c: Color) {
        self.texto(s, tam, x_centro - medir(s, tam, peso) / 2.0, y, peso, c);
    }

    fn bloque(&self, b: &Bloque) {
        for (i, r) in b.renglones.iter().enumerate() {
            let y = b.y + i as f32 * alto_renglon(b.tam);
            if b.centrado {
                self.centrado(r, b.tam, b.x + b.ancho / 2.0, y, b.peso, b.color);
            } else {
                self.texto(r, b.tam, b.x, y, b.peso, b.color);
            }
        }
    }

    /// El fondo a toda la página: ya trae los logos encima.
    fn fondo(&self, f: &Fondo) {
        let x = ImageXObject {
            width: Px(f.ancho as usize),
            height: Px(f.alto as usize),
            color_space: ColorSpace::Rgb,
            bits_per_component: ColorBits::Bit8,
            interpolate: true,
            image_data: f.jpeg.clone(),
            image_filter: Some(ImageFilter::DCT),
            smask: None,
            clipping_bbox: None,
        };
        let t = ImageTransform { dpi: Some(f.ancho as f32 * 72.0 / ANCHO), ..Default::default() };
        Image::from(x).add_to_layer(self.capa.clone(), t);
    }
}

fn mm(pt: f32) -> Mm {
    Mm::from(Pt(pt))
}

pub fn generar(diapositivas: &[Diapositiva], titulo: &str, plantilla: Option<&Plantilla>) -> Result<Vec<u8>, String> {
    let (doc, pag, capa) = PdfDocument::new(titulo, mm(ANCHO), mm(ALTO), "diapositiva");
    let mut fuentes = HashMap::new();
    for p in Peso::TODOS {
        fuentes.insert(p, doc.add_external_font(Cursor::new(p.datos())).map_err(|e| e.to_string())?);
    }

    for (i, d) in diapositivas.iter().enumerate() {
        let (p, c) = if i == 0 { (pag, capa) } else { doc.add_page(mm(ANCHO), mm(ALTO), "diapositiva") };
        let l = Lienzo { capa: doc.get_page(p).get_layer(c), fuentes: &fuentes };
        if let Some(f) = plantilla.and_then(|x| x.diseno(d.diseno).fondo.as_ref()) {
            l.fondo(f);
        }
        dibujar(&l, d);
    }
    doc.save_to_bytes().map_err(|e| e.to_string())
}

fn dibujar(l: &Lienzo, d: &Diapositiva) {
    match &d.contenido {
        Contenido::Libre(bloques) => bloques.iter().for_each(|b| l.bloque(b)),
        Contenido::Resumen { titulo, filas } => {
            l.bloque(titulo);
            let cols = columnas_resumen();
            let y0 = maqueta::Y_TABLA;
            l.texto("PROYECTO", 11.0, MARGEN + 8.0, y0, Peso::Negrita, GRIS);
            for (i, e) in EstadoAvance::TODOS.iter().enumerate() {
                l.centrado(&e.etiqueta().to_uppercase(), 11.0, cols[i], y0, Peso::Negrita, GRIS);
            }
            l.rect(MARGEN, y0 + 20.0, ANCHO_TEXTO, 1.5, GUINDA);
            let mut y = y0 + 30.0;
            for f in filas {
                if f.total {
                    l.rect(MARGEN, y - 6.0, ANCHO_TEXTO, maqueta::ALTO_FILA, CREMA);
                }
                let tam = tam_que_cabe(&f.nombre, ancho_nombre_resumen(), 18.0, 13.0, Peso::Negrita);
                let y_nombre = if f.equipo.is_empty() { y + 7.0 } else { y };
                l.texto(&f.nombre, tam, MARGEN + 8.0, y_nombre, Peso::Negrita, if f.total { GUINDA } else { TINTA });
                if !f.equipo.is_empty() {
                    l.texto(&f.equipo, 12.0, MARGEN + 8.0, y + 22.0, Peso::Regular, GRIS);
                }
                for (i, e) in EstadoAvance::TODOS.iter().enumerate() {
                    let n = f.cuentas[i];
                    let (s, c) = if n == 0 { ("—".to_string(), GRIS_CLARO) } else { (n.to_string(), e.color()) };
                    l.centrado(&s, 24.0, cols[i], y + 1.0, Peso::Negrita, c);
                }
                y += maqueta::ALTO_FILA;
                if !f.total {
                    l.rect(MARGEN, y - 6.0, ANCHO_TEXTO, 0.75, LINEA);
                }
            }
        }
        Contenido::Lista { titulo, subtitulo, cuerpo, nota } => {
            l.bloque(titulo);
            if let Some(s) = subtitulo {
                l.bloque(s);
            }
            let lh = cuerpo.interlineado();
            for p in &cuerpo.parrafos {
                l.texto("•", cuerpo.tam, MARGEN, p.y, Peso::Negrita, GUINDA);
                for (i, r) in p.renglones.iter().enumerate() {
                    l.renglon(r, cuerpo.tam, MARGEN + SANGRIA, p.y + i as f32 * lh);
                }
            }
            if let Some(n) = nota {
                l.bloque(n);
            }
        }
    }
}

/// Centro de las columnas Concluido, En proceso y Pendiente del resumen.
pub fn columnas_resumen() -> [f32; 3] {
    [DERECHA - 330.0, DERECHA - 200.0, DERECHA - 70.0]
}

/// Lo que puede medir el nombre del proyecto antes de la primera columna.
pub fn ancho_nombre_resumen() -> f32 {
    columnas_resumen()[0] - 70.0 - MARGEN
}
