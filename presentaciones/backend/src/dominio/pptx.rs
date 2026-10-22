//! Dibuja la maqueta como PowerPoint editable (.pptx).
//!
//! Con plantilla, las diapositivas se meten dentro de la plantilla misma: el
//! archivo conserva su maestro, diseños, tema, fondos y logos, y cada
//! diapositiva usa su diseño ("Portada", "Interior 2"…). Sin plantilla, se arma
//! un paquete mínimo con la misma paleta y fuente.
//!
//! Cada viñeta va en su propia caja de texto, como en los decks del área, y
//! el interlineado va fijo en puntos para que PowerPoint ponga cada renglón
//! donde lo puso la maqueta.

use std::io::{Cursor, Write};

use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use super::{
    estilo::*,
    maqueta::{self, Bloque, Contenido, Diapositiva, Trozo},
    pdf::{ancho_nombre_resumen, columnas_resumen},
    plantilla::Plantilla,
    EstadoAvance,
};

const EMU: f32 = 12_700.0;

fn emu(pt: f32) -> i64 {
    (pt * EMU).round() as i64
}

fn escapar(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            c if (c as u32) < 0x20 && c != '\t' => {}
            c => o.push(c),
        }
    }
    o
}

const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#;
const XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#;
const GRUPO: &str = r#"<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>"#;
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const TIPO_DIAPO: &str = "application/vnd.openxmlformats-officedocument.presentationml.slide+xml";

#[derive(Clone, Copy)]
enum Alinear {
    Izq,
    Centro,
}

/// Junta las figuras de una diapositiva con ids consecutivos.
struct Hoja {
    figuras: String,
    id: u32,
}

impl Hoja {
    fn new() -> Self {
        Self { figuras: String::new(), id: 2 }
    }

    fn siguiente(&mut self) -> u32 {
        self.id += 1;
        self.id - 1
    }

    fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: Color) {
        let id = self.siguiente();
        self.figuras.push_str(&format!(
            r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="Forma {id}"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val="{}"/></a:solidFill><a:ln><a:noFill/></a:ln></p:spPr></p:sp>"#,
            emu(x), emu(y), emu(w), emu(h), c.hex()
        ));
    }

    /// Caja de texto sin márgenes internos. `parrafos` ya es XML de `<a:p>`.
    fn caja(&mut self, nombre: &str, x: f32, y: f32, w: f32, h: f32, parrafos: &str) {
        let id = self.siguiente();
        self.figuras.push_str(&format!(
            r#"<p:sp><p:nvSpPr><p:cNvPr id="{id}" name="{} {id}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:noFill/></p:spPr><p:txBody><a:bodyPr wrap="square" lIns="0" tIns="0" rIns="0" bIns="0" rtlCol="0" anchor="t"><a:noAutofit/></a:bodyPr><a:lstStyle/>{parrafos}</p:txBody></p:sp>"#,
            escapar(nombre), emu(x), emu(y), emu(w), emu(h.max(1.0))
        ));
    }

    fn texto(&mut self, nombre: &str, s: &str, tam: f32, x: f32, y: f32, w: f32, peso: Peso, c: Color, al: Alinear) {
        let p = parrafo(&[Trozo::new(s, peso, c)], tam, al, None);
        self.caja(nombre, x, y, w, alto_renglon(tam), &p);
    }

    /// Un renglón por párrafo: así PowerPoint no lo vuelve a partir distinto.
    fn bloque(&mut self, nombre: &str, b: &Bloque) {
        let al = if b.centrado { Alinear::Centro } else { Alinear::Izq };
        let ps: String = b.renglones.iter().map(|r| parrafo(&[Trozo::new(r.as_str(), b.peso, b.color)], b.tam, al, None)).collect();
        // Holgura a lo ancho por si PowerPoint mide un poco distinto.
        self.caja(nombre, b.x, b.y, b.ancho + 8.0, b.alto(), &ps);
    }
}

fn corrida(t: &Trozo, tam: f32) -> String {
    let (fuente, negrita) = t.peso.fuente();
    format!(
        r#"<a:r><a:rPr lang="es-MX" sz="{}" b="{}" dirty="0"><a:solidFill><a:srgbClr val="{}"/></a:solidFill><a:latin typeface="{fuente}"/><a:ea typeface="{fuente}"/><a:cs typeface="{fuente}"/></a:rPr><a:t>{}</a:t></a:r>"#,
        (tam * 100.0).round() as i64,
        if negrita { 1 } else { 0 },
        t.color.hex(),
        escapar(&t.texto)
    )
}

fn parrafo(trozos: &[Trozo], tam: f32, al: Alinear, vineta: Option<Color>) -> String {
    let algn = match al {
        Alinear::Izq => "l",
        Alinear::Centro => "ctr",
    };
    let interlineado = format!(r#"<a:lnSpc><a:spcPts val="{}"/></a:lnSpc><a:spcBef><a:spcPts val="0"/></a:spcBef>"#, (alto_renglon(tam) * 100.0).round() as i64);
    let ppr = match vineta {
        Some(c) => format!(
            r#"<a:pPr marL="{s}" indent="-{s}" algn="{algn}">{interlineado}<a:buClr><a:srgbClr val="{}"/></a:buClr><a:buFont typeface="Arial"/><a:buChar char="•"/></a:pPr>"#,
            c.hex(),
            s = emu(SANGRIA)
        ),
        None => format!(r#"<a:pPr marL="0" indent="0" algn="{algn}">{interlineado}<a:buNone/></a:pPr>"#),
    };
    let corridas: String = trozos.iter().map(|t| corrida(t, tam)).collect();
    format!(r#"<a:p>{ppr}{corridas}<a:endParaRPr lang="es-MX" sz="{}" dirty="0"/></a:p>"#, (tam * 100.0).round() as i64)
}

fn diapositiva(d: &Diapositiva) -> String {
    let mut h = Hoja::new();
    match &d.contenido {
        Contenido::Libre(bloques) => {
            for (i, b) in bloques.iter().enumerate() {
                h.bloque(if i == 0 { "Título" } else { "Texto" }, b);
            }
        }
        Contenido::Resumen { titulo, filas } => {
            h.bloque("Título", titulo);
            let cols = columnas_resumen();
            let y0 = maqueta::Y_TABLA;
            h.texto("Encabezado", "PROYECTO", 11.0, MARGEN + 8.0, y0, 200.0, Peso::Negrita, GRIS, Alinear::Izq);
            for (i, e) in EstadoAvance::TODOS.iter().enumerate() {
                h.texto("Encabezado", &e.etiqueta().to_uppercase(), 11.0, cols[i] - 60.0, y0, 120.0, Peso::Negrita, GRIS, Alinear::Centro);
            }
            h.rect(MARGEN, y0 + 20.0, ANCHO_TEXTO, 1.5, GUINDA);
            let mut y = y0 + 30.0;
            for f in filas {
                if f.total {
                    h.rect(MARGEN, y - 6.0, ANCHO_TEXTO, maqueta::ALTO_FILA, CREMA);
                }
                let tam = tam_que_cabe(&f.nombre, ancho_nombre_resumen(), 18.0, 13.0, Peso::Negrita);
                let y_nombre = if f.equipo.is_empty() { y + 7.0 } else { y };
                let ancho = ancho_nombre_resumen() + 8.0;
                h.texto("Proyecto", &f.nombre, tam, MARGEN + 8.0, y_nombre, ancho, Peso::Negrita, if f.total { GUINDA } else { TINTA }, Alinear::Izq);
                if !f.equipo.is_empty() {
                    h.texto("Equipo", &f.equipo, 12.0, MARGEN + 8.0, y + 22.0, ancho, Peso::Regular, GRIS, Alinear::Izq);
                }
                for (i, e) in EstadoAvance::TODOS.iter().enumerate() {
                    let n = f.cuentas[i];
                    let (s, c) = if n == 0 { ("—".to_string(), GRIS_CLARO) } else { (n.to_string(), e.color()) };
                    h.texto(e.etiqueta(), &s, 24.0, cols[i] - 60.0, y + 1.0, 120.0, Peso::Negrita, c, Alinear::Centro);
                }
                y += maqueta::ALTO_FILA;
                if !f.total {
                    h.rect(MARGEN, y - 6.0, ANCHO_TEXTO, 0.75, LINEA);
                }
            }
        }
        Contenido::Lista { titulo, subtitulo, cuerpo, nota } => {
            h.bloque("Título", titulo);
            if let Some(s) = subtitulo {
                h.bloque("Equipo", s);
            }
            let lh = cuerpo.interlineado();
            for (i, p) in cuerpo.parrafos.iter().enumerate() {
                let xml = parrafo(&p.trozos, cuerpo.tam, Alinear::Izq, Some(GUINDA));
                h.caja(&format!("Punto {}", i + 1), MARGEN, p.y, ANCHO_TEXTO + 8.0, p.renglones.len() as f32 * lh, &xml);
            }
            if let Some(n) = nota {
                h.bloque("Siguiente corte", n);
            }
        }
    }
    format!(
        r#"{XML}<p:sld {NS}><p:cSld><p:spTree>{GRUPO}{}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"#,
        h.figuras
    )
}

pub fn generar(diapositivas: &[Diapositiva], titulo: &str, autor: &str, plantilla: Option<&Plantilla>) -> Result<Vec<u8>, String> {
    let n = diapositivas.len();
    let mut archivos: Vec<(String, Vec<u8>)> = match plantilla {
        Some(p) => sobre_plantilla(p, n),
        None => paquete_propio(n),
    };
    archivos.push(("docProps/core.xml".into(), core(titulo, autor).into_bytes()));
    archivos.push((
        "docProps/app.xml".into(),
        format!(r#"{XML}<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Avance de proyectos</Application><Slides>{n}</Slides></Properties>"#).into_bytes(),
    ));
    for (i, d) in diapositivas.iter().enumerate() {
        archivos.push((format!("ppt/slides/slide{}.xml", i + 1), diapositiva(d).into_bytes()));
        let diseno = match plantilla {
            Some(p) => format!("../{}", p.diseno(d.diseno).ruta.trim_start_matches("ppt/")),
            None => "../slideLayouts/slideLayout1.xml".into(),
        };
        archivos.push((
            format!("ppt/slides/_rels/slide{}.xml.rels", i + 1),
            format!(r#"{XML}<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{REL}/slideLayout" Target="{diseno}"/></Relationships>"#).into_bytes(),
        ));
    }

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opciones = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    // [Content_Types].xml va primero, como lo deja PowerPoint.
    archivos.sort_by_key(|(r, _)| r != "[Content_Types].xml");
    for (ruta, contenido) in archivos {
        zip.start_file(ruta, opciones).map_err(|e| e.to_string())?;
        zip.write_all(&contenido).map_err(|e| e.to_string())?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}

/// Todo lo de la plantilla menos sus diapositivas de ejemplo; la lista de
/// diapositivas, sus relaciones y los tipos se rehacen con las nuestras.
fn sobre_plantilla(p: &Plantilla, n: usize) -> Vec<(String, Vec<u8>)> {
    let mut v = vec![];
    for (ruta, datos) in p.partes() {
        let texto = || String::from_utf8_lossy(datos).into_owned();
        match ruta.as_str() {
            r if r.starts_with("ppt/slides/") || r.starts_with("ppt/notesSlides/") => {}
            "docProps/core.xml" | "docProps/app.xml" => {}
            "[Content_Types].xml" => {
                let t = quitar_elementos(&texto(), "<Override ", |e| e.contains("/ppt/slides/") || e.contains("/ppt/notesSlides/"));
                let nuevas: String = (1..=n)
                    .map(|i| format!(r#"<Override PartName="/ppt/slides/slide{i}.xml" ContentType="{TIPO_DIAPO}"/>"#))
                    .collect();
                v.push((ruta.clone(), t.replace("</Types>", &format!("{nuevas}</Types>")).into_bytes()));
            }
            "ppt/_rels/presentation.xml.rels" => {
                let t = quitar_elementos(&texto(), "<Relationship ", |e| e.contains("/relationships/slide\""));
                let nuevas: String = (1..=n)
                    .map(|i| format!(r#"<Relationship Id="rIdDiapo{i}" Type="{REL}/slide" Target="slides/slide{i}.xml"/>"#))
                    .collect();
                v.push((ruta.clone(), t.replace("</Relationships>", &format!("{nuevas}</Relationships>")).into_bytes()));
            }
            "ppt/presentation.xml" => {
                let t = texto();
                let lista = format!(
                    "<p:sldIdLst>{}</p:sldIdLst>",
                    (1..=n).map(|i| format!(r#"<p:sldId id="{}" r:id="rIdDiapo{i}"/>"#, 255 + i)).collect::<String>()
                );
                let t = match (t.find("<p:sldIdLst>"), t.find("</p:sldIdLst>")) {
                    (Some(a), Some(b)) => format!("{}{lista}{}", &t[..a], &t[b + "</p:sldIdLst>".len()..]),
                    _ => t.replacen("<p:sldSz", &format!("{lista}<p:sldSz"), 1),
                };
                v.push((ruta.clone(), t.into_bytes()));
            }
            _ => v.push((ruta.clone(), datos.clone())),
        }
    }
    v
}

/// Quita los elementos vacíos `<Etiqueta …/>` que cumplan `quitar`.
fn quitar_elementos(xml: &str, inicio: &str, quitar: impl Fn(&str) -> bool) -> String {
    let mut o = String::with_capacity(xml.len());
    let mut resto = xml;
    while let Some(i) = resto.find(inicio) {
        let Some(f) = resto[i..].find("/>") else { break };
        let elemento = &resto[i..i + f + 2];
        o.push_str(&resto[..i]);
        if !quitar(elemento) {
            o.push_str(elemento);
        }
        resto = &resto[i + f + 2..];
    }
    o.push_str(resto);
    o
}

/// Sin plantilla: maestro, un diseño en blanco y el tema con la paleta y la fuente.
fn paquete_propio(n: usize) -> Vec<(String, Vec<u8>)> {
    let p = "application/vnd.openxmlformats-officedocument.presentationml";
    let diapos: String = (1..=n).map(|i| format!(r#"<Override PartName="/ppt/slides/slide{i}.xml" ContentType="{TIPO_DIAPO}"/>"#)).collect();
    let tipos = format!(
        r#"{XML}<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/ppt/presentation.xml" ContentType="{p}.presentation.main+xml"/><Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="{p}.slideMaster+xml"/><Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="{p}.slideLayout+xml"/><Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/><Override PartName="/ppt/presProps.xml" ContentType="{p}.presProps+xml"/><Override PartName="/ppt/viewProps.xml" ContentType="{p}.viewProps+xml"/><Override PartName="/ppt/tableStyles.xml" ContentType="{p}.tableStyles+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>{diapos}</Types>"#
    );
    let ids: String = (1..=n).map(|i| format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i + 2)).collect();
    let presentacion = format!(
        r#"{XML}<p:presentation {NS} saveSubsetFonts="1"><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst><p:sldIdLst>{ids}</p:sldIdLst><p:sldSz cx="{}" cy="{}"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#,
        emu(ANCHO),
        emu(ALTO)
    );
    let rels_diapos: String = (1..=n).map(|i| format!(r#"<Relationship Id="rId{}" Type="{REL}/slide" Target="slides/slide{i}.xml"/>"#, i + 2)).collect();
    let rels_pres = format!(
        r#"{XML}<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{REL}/slideMaster" Target="slideMasters/slideMaster1.xml"/><Relationship Id="rId2" Type="{REL}/theme" Target="theme/theme1.xml"/>{rels_diapos}<Relationship Id="rId{a}" Type="{REL}/presProps" Target="presProps.xml"/><Relationship Id="rId{b}" Type="{REL}/viewProps" Target="viewProps.xml"/><Relationship Id="rId{c}" Type="{REL}/tableStyles" Target="tableStyles.xml"/></Relationships>"#,
        a = n + 3,
        b = n + 4,
        c = n + 5
    );
    [
        ("[Content_Types].xml", tipos),
        ("_rels/.rels", RELS_RAIZ.into()),
        ("ppt/presentation.xml", presentacion),
        ("ppt/_rels/presentation.xml.rels", rels_pres),
        ("ppt/presProps.xml", format!("{XML}<p:presentationPr {NS}/>")),
        ("ppt/viewProps.xml", format!("{XML}<p:viewPr {NS}/>")),
        ("ppt/tableStyles.xml", format!(r#"{XML}<a:tblStyleLst xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" def="{{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}}"/>"#)),
        ("ppt/slideMasters/slideMaster1.xml", MAESTRO.replace("{XML}", XML).replace("{NS}", NS).replace("{GRUPO}", GRUPO)),
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels", RELS_MAESTRO.into()),
        ("ppt/slideLayouts/slideLayout1.xml", format!(r#"{XML}<p:sldLayout {NS} type="blank" preserve="1"><p:cSld name="En blanco"><p:spTree>{GRUPO}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>"#)),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels", RELS_DISENO.into()),
        ("ppt/theme/theme1.xml", tema()),
    ]
    .into_iter()
    .map(|(r, c)| (r.to_string(), c.into_bytes()))
    .collect()
}

fn core(titulo: &str, autor: &str) -> String {
    format!(
        r#"{XML}<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{}</dc:title><dc:creator>{}</dc:creator><cp:lastModifiedBy>{}</cp:lastModifiedBy></cp:coreProperties>"#,
        escapar(titulo),
        escapar(autor),
        escapar(autor)
    )
}

/// El tema con la paleta y la fuente de la plantilla, para lo que se agregue a mano.
fn tema() -> String {
    let fill = r#"<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>"#;
    let ln = |w: u32| format!(r#"<a:ln w="{w}"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>"#);
    format!(
        r#"{XML}<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Institucional"><a:themeElements><a:clrScheme name="Institucional"><a:dk1><a:srgbClr val="212121"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="691B32"/></a:dk2><a:lt2><a:srgbClr val="A02041"/></a:lt2><a:accent1><a:srgbClr val="EDD899"/></a:accent1><a:accent2><a:srgbClr val="AE8320"/></a:accent2><a:accent3><a:srgbClr val="235C4E"/></a:accent3><a:accent4><a:srgbClr val="11322C"/></a:accent4><a:accent5><a:srgbClr val="98989A"/></a:accent5><a:accent6><a:srgbClr val="FCF7EC"/></a:accent6><a:hlink><a:srgbClr val="A02142"/></a:hlink><a:folHlink><a:srgbClr val="691B32"/></a:folHlink></a:clrScheme><a:fontScheme name="Institucional"><a:majorFont><a:latin typeface="Noto Sans"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Noto Sans"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Institucional"><a:fillStyleLst>{fill}{fill}{fill}</a:fillStyleLst><a:lnStyleLst>{l1}{l2}{l3}</a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst>{fill}{fill}{fill}</a:bgFillStyleLst></a:fmtScheme></a:themeElements><a:objectDefaults/><a:extraClrSchemeLst/></a:theme>"#,
        l1 = ln(6350),
        l2 = ln(12700),
        l3 = ln(19050),
    )
}

const RELS_RAIZ: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#;

const RELS_MAESTRO: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/></Relationships>"#;

const RELS_DISENO: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/></Relationships>"#;

const MAESTRO: &str = r#"{XML}<p:sldMaster {NS}><p:cSld><p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg><p:spTree>{GRUPO}</p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle><a:lvl1pPr><a:defRPr sz="4000" b="1"><a:solidFill><a:srgbClr val="A02142"/></a:solidFill><a:latin typeface="+mj-lt"/></a:defRPr></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr><a:defRPr sz="2000"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:bodyStyle><p:otherStyle><a:lvl1pPr><a:defRPr sz="1800"><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:otherStyle></p:txStyles></p:sldMaster>"#;

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn quita_solo_los_elementos_pedidos() {
        let xml = r#"<Types><Override PartName="/ppt/slides/slide1.xml" ContentType="x"/><Override PartName="/ppt/theme/theme1.xml" ContentType="y"/></Types>"#;
        let t = quitar_elementos(xml, "<Override ", |e| e.contains("/ppt/slides/"));
        assert_eq!(t, r#"<Types><Override PartName="/ppt/theme/theme1.xml" ContentType="y"/></Types>"#);
    }
}
