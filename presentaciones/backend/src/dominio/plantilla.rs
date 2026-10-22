//! Lee la plantilla institucional (.pptx o .potx) para generar sobre ella.
//!
//! - El PowerPoint se arma dentro de la plantilla misma: conserva su maestro,
//!   sus diseños, su tema, fondos y logos; solo se cambian las diapositivas.
//! - El PDF dibuja, debajo del texto, el fondo de cada diseño con sus logos
//!   encima, ya juntos en una sola imagen.
//!
//! Los diseños se reconocen por nombre: "Portada", "Interior 1", "Interior 2"
//! y "Cierre", como vienen en Plantilla_Presentacion2026.

use std::{
    collections::HashMap,
    io::{Cursor, Read},
};

use image::{codecs::jpeg::JpegEncoder, imageops, imageops::FilterType, RgbaImage};

use super::{lector::plegar, maqueta::Diseno};

/// La diapositiva de 13.33 × 7.5 pulgadas en EMU: las medidas de la maqueta.
const ANCHO_EMU: f64 = 12_192_000.0;
const ALTO_EMU: f64 = 6_858_000.0;
/// Resolución del fondo en el PDF: 1920 × 1080 se ve nítido en proyector.
const ANCHO_FONDO: u32 = 1920;
const ALTO_FONDO: u32 = 1080;

pub struct Fondo {
    pub jpeg: Vec<u8>,
    pub ancho: u32,
    pub alto: u32,
}

pub struct DisenoPlantilla {
    /// "ppt/slideLayouts/slideLayout2.xml"
    pub ruta: String,
    pub fondo: Option<Fondo>,
}

pub struct Plantilla {
    pub nombre: String,
    partes: Vec<(String, Vec<u8>)>,
    disenos: HashMap<Diseno, DisenoPlantilla>,
}

impl Plantilla {
    pub fn cargar(nombre: &str, archivo: &[u8]) -> Result<Self, String> {
        let mut zip = zip::ZipArchive::new(Cursor::new(archivo)).map_err(|_| "no es un .pptx válido".to_string())?;
        let mut partes = vec![];
        for i in 0..zip.len() {
            let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
            if f.is_dir() {
                continue;
            }
            let mut datos = vec![];
            f.read_to_end(&mut datos).map_err(|e| e.to_string())?;
            partes.push((f.name().to_string(), datos));
        }
        let parte = |ruta: &str| partes.iter().find(|(n, _)| n == ruta).map(|(_, d)| d.as_slice());
        let texto = |ruta: &str| parte(ruta).map(|d| String::from_utf8_lossy(d).into_owned());

        let pres = texto("ppt/presentation.xml").ok_or("le falta ppt/presentation.xml")?;
        let tam = etiqueta(&pres, "<p:sldSz").ok_or("no dice el tamaño de diapositiva")?;
        let (cx, cy) = (atributo(tam, "cx").unwrap_or(""), atributo(tam, "cy").unwrap_or(""));
        if cx != "12192000" || cy != "6858000" {
            return Err("la plantilla debe ser panorámica de 13.33 × 7.5 pulgadas (16:9), como la de 2026".into());
        }

        let mut disenos = HashMap::new();
        for (ruta, datos) in &partes {
            if !ruta.starts_with("ppt/slideLayouts/") || !ruta.ends_with(".xml") {
                continue;
            }
            let xml = String::from_utf8_lossy(datos);
            let nombre_diseno = etiqueta(&xml, "<p:cSld").and_then(|e| atributo(e, "name")).unwrap_or("");
            let Some(diseno) = Diseno::TODOS.into_iter().find(|d| plegar(d.nombre()) == plegar(nombre_diseno.trim())) else {
                continue;
            };
            let carpeta = ruta.rsplit_once('/').map(|(c, _)| c).unwrap_or("");
            let archivo = ruta.rsplit_once('/').map(|(_, a)| a).unwrap_or(ruta);
            let rels = texto(&format!("{carpeta}/_rels/{archivo}.rels")).unwrap_or_default();
            let destinos = relaciones(&rels, carpeta);
            let imagen = |id: &str| destinos.get(id).and_then(|r| parte(r));

            let fondo = segmentos(&xml, "<p:bg>", "</p:bg>")
                .first()
                .and_then(|bg| atributo(bg, "r:embed"))
                .and_then(imagen)
                .map(|bg| {
                    let logos: Vec<(&[u8], [f64; 4])> = segmentos(&xml, "<p:pic>", "</p:pic>")
                        .into_iter()
                        .filter_map(|pic| Some((imagen(atributo(pic, "r:embed")?)?, caja(pic)?)))
                        .collect();
                    componer(bg, &logos)
                })
                .transpose()
                .map_err(|e| format!("no se pudo leer el fondo de «{nombre_diseno}»: {e}"))?;
            disenos.insert(diseno, DisenoPlantilla { ruta: ruta.clone(), fondo });
        }
        if disenos.is_empty() {
            return Err("no encontré los diseños Portada, Interior 1, Interior 2 ni Cierre".into());
        }
        Ok(Self { nombre: nombre.to_string(), partes, disenos })
    }

    /// El diseño pedido, o el más parecido que tenga la plantilla.
    pub fn diseno(&self, d: Diseno) -> &DisenoPlantilla {
        let alternos: &[Diseno] = match d {
            Diseno::Interior1 => &[Diseno::Interior1, Diseno::Interior2],
            Diseno::Interior2 => &[Diseno::Interior2, Diseno::Interior1],
            Diseno::Portada => &[Diseno::Portada, Diseno::Interior2, Diseno::Interior1],
            Diseno::Cierre => &[Diseno::Cierre, Diseno::Portada, Diseno::Interior2],
        };
        alternos
            .iter()
            .find_map(|x| self.disenos.get(x))
            .or_else(|| self.disenos.values().next())
            .expect("la plantilla tiene al menos un diseño")
    }

    pub fn disenos(&self) -> Vec<&'static str> {
        Diseno::TODOS.into_iter().filter(|d| self.disenos.contains_key(d)).map(|d| d.nombre()).collect()
    }

    /// Todas las partes del paquete, tal cual vienen.
    pub fn partes(&self) -> &[(String, Vec<u8>)] {
        &self.partes
    }
}

/// Junta el fondo con las imágenes que el diseño pone encima (los logos) y lo
/// deja como un solo JPEG de 1920 × 1080.
fn componer(fondo: &[u8], encima: &[(&[u8], [f64; 4])]) -> Result<Fondo, String> {
    let fondo = image::load_from_memory(fondo).map_err(|e| e.to_string())?;
    let mut lienzo: RgbaImage = fondo.resize_exact(ANCHO_FONDO, ALTO_FONDO, FilterType::Triangle).to_rgba8();
    let ex = ANCHO_FONDO as f64 / ANCHO_EMU;
    let ey = ALTO_FONDO as f64 / ALTO_EMU;
    let mut puestas: Vec<[f64; 4]> = vec![];
    for (datos, [x, y, w, h]) in encima {
        // La plantilla repite el mismo logo dos veces en el mismo lugar.
        if puestas.iter().any(|p| p == &[*x, *y, *w, *h]) {
            continue;
        }
        puestas.push([*x, *y, *w, *h]);
        let Ok(img) = image::load_from_memory(datos) else { continue };
        let (w, h) = (((w * ex).round() as u32).max(1), ((h * ey).round() as u32).max(1));
        let img = img.resize_exact(w, h, FilterType::Lanczos3).to_rgba8();
        imageops::overlay(&mut lienzo, &img, (x * ex).round() as i64, (y * ey).round() as i64);
    }
    let rgb = image::DynamicImage::ImageRgba8(lienzo).to_rgb8();
    let mut jpeg = vec![];
    JpegEncoder::new_with_quality(&mut jpeg, 85).encode_image(&rgb).map_err(|e| e.to_string())?;
    Ok(Fondo { jpeg, ancho: ANCHO_FONDO, alto: ALTO_FONDO })
}

/// Posición y tamaño en EMU de una figura: `<a:off x y/><a:ext cx cy/>`.
fn caja(xml: &str) -> Option<[f64; 4]> {
    // Dentro de <a:xfrm>: antes puede venir un <a:ext uri="…"> de extLst.
    let xfrm = &xml[xml.find("<a:xfrm")?..];
    let off = etiqueta(xfrm, "<a:off")?;
    let ext = etiqueta(xfrm, "<a:ext ")?;
    let n = |e: &str, a: &str| atributo(e, a)?.parse::<f64>().ok();
    Some([n(off, "x")?, n(off, "y")?, n(ext, "cx")?, n(ext, "cy")?])
}

/// Id → ruta completa dentro del paquete, resolviendo "../media/x.png".
fn relaciones(rels: &str, carpeta: &str) -> HashMap<String, String> {
    rels.split("<Relationship ")
        .skip(1)
        .filter_map(|r| {
            // El corte se comió el espacio de antes del primer atributo.
            let r = format!(" {r}");
            let id = atributo(&r, "Id")?;
            let destino = atributo(&r, "Target")?;
            Some((id.to_string(), resolver(carpeta, destino)))
        })
        .collect()
}

pub fn resolver(carpeta: &str, destino: &str) -> String {
    if let Some(abs) = destino.strip_prefix('/') {
        return abs.to_string();
    }
    let mut partes: Vec<&str> = carpeta.split('/').filter(|s| !s.is_empty()).collect();
    for p in destino.split('/') {
        match p {
            ".." => {
                partes.pop();
            }
            "." | "" => {}
            otro => partes.push(otro),
        }
    }
    partes.join("/")
}

/// La primera etiqueta que empieza con `inicio`, hasta su `>`.
pub fn etiqueta<'a>(xml: &'a str, inicio: &str) -> Option<&'a str> {
    let i = xml.find(inicio)?;
    let fin = xml[i..].find('>')?;
    Some(&xml[i..i + fin + 1])
}

pub fn atributo<'a>(xml: &'a str, nombre: &str) -> Option<&'a str> {
    let patron = format!(" {nombre}=\"");
    let i = xml.find(&patron)? + patron.len();
    let fin = xml[i..].find('"')?;
    Some(&xml[i..i + fin])
}

fn segmentos<'a>(xml: &'a str, abre: &str, cierra: &str) -> Vec<&'a str> {
    let mut v = vec![];
    let mut resto = xml;
    while let Some(i) = resto.find(abre) {
        let Some(f) = resto[i..].find(cierra) else { break };
        v.push(&resto[i..i + f + cierra.len()]);
        resto = &resto[i + f + cierra.len()..];
    }
    v
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn resuelve_rutas_relativas() {
        assert_eq!(resolver("ppt/slideLayouts", "../media/image1.jpg"), "ppt/media/image1.jpg");
        assert_eq!(resolver("ppt", "slides/slide1.xml"), "ppt/slides/slide1.xml");
        assert_eq!(resolver("ppt/slides", "/ppt/media/a.png"), "ppt/media/a.png");
    }

    #[test]
    fn lee_atributos_y_cajas() {
        let pic = r#"<p:pic><p:cNvPr id="3" name="Imagen 2"><a:extLst><a:ext uri="{FF2B5EF4}"/></a:extLst></p:cNvPr><p:blipFill><a:blip r:embed="rId3"/></p:blipFill><p:spPr><a:xfrm><a:off x="7771966" y="213996"/><a:ext cx="4079284" cy="540000"/></a:xfrm></p:spPr></p:pic>"#;
        assert_eq!(atributo(pic, "r:embed"), Some("rId3"));
        assert_eq!(caja(pic), Some([7771966.0, 213996.0, 4079284.0, 540000.0]));
        let rels = r#"<Relationships><Relationship Id="rId2" Type="x/image" Target="../media/image1.jpg"/></Relationships>"#;
        assert_eq!(relaciones(rels, "ppt/slideLayouts").get("rId2").map(String::as_str), Some("ppt/media/image1.jpg"));
    }

    #[test]
    fn rechaza_lo_que_no_es_pptx() {
        assert!(Plantilla::cargar("x", b"no soy un zip").is_err());
    }

    /// Con la plantilla real, si está en `../plantilla/` (no viaja en el repo).
    #[test]
    fn lee_la_plantilla_institucional_si_esta() {
        let Some(ruta) = std::fs::read_dir("../plantilla").ok().and_then(|d| {
            d.filter_map(Result::ok).map(|e| e.path()).find(|p| p.extension().is_some_and(|x| x == "pptx" || x == "potx"))
        }) else {
            return;
        };
        let p = Plantilla::cargar("plantilla", &std::fs::read(ruta).unwrap()).unwrap();
        assert_eq!(p.disenos(), vec!["Portada", "Interior 1", "Interior 2", "Cierre"]);
        for d in Diseno::TODOS {
            assert!(p.diseno(d).fondo.is_some(), "{} sin fondo", d.nombre());
        }
    }
}
