//! Dibuja la maqueta como PowerPoint editable (.pptx), armado a mano: es XML
//! dentro de un zip. Se abre en PowerPoint, Keynote y Google Slides. Cada
//! viñeta va en su propia caja de texto, como en el deck original, para que
//! quede exactamente donde el PDF la pone.

use std::io::{Cursor, Write};

use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

use super::{
    estilo::*,
    maqueta::{self, Contenido, Diapositiva, Trozo},
    pdf::{columnas_resumen, ALTO_FILA_TOTAL},
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

#[derive(Clone, Copy)]
enum Alinear {
    Izq,
    Centro,
    Der,
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

    fn texto(&mut self, nombre: &str, s: &str, tam: f32, x: f32, y: f32, w: f32, negrita: bool, c: Color, al: Alinear) {
        let p = parrafo(&[Trozo { texto: s.to_string(), negrita, color: c }], tam, al, None);
        self.caja(nombre, x, y, w, tam * 1.3, &p);
    }
}

fn corrida(t: &Trozo, tam: f32) -> String {
    format!(
        r#"<a:r><a:rPr lang="es-MX" sz="{}" b="{}" dirty="0"><a:solidFill><a:srgbClr val="{}"/></a:solidFill><a:latin typeface="{FUENTE}"/><a:cs typeface="{FUENTE}"/></a:rPr><a:t>{}</a:t></a:r>"#,
        (tam * 100.0).round() as i64,
        if t.negrita { 1 } else { 0 },
        t.color.hex(),
        escapar(&t.texto)
    )
}

/// Interlineado: Montserrat a renglón sencillo mide 1.219 del tamaño; la
/// maqueta usa 1.25.
fn parrafo(trozos: &[Trozo], tam: f32, al: Alinear, vineta: Option<Color>) -> String {
    let algn = match al {
        Alinear::Izq => "l",
        Alinear::Centro => "ctr",
        Alinear::Der => "r",
    };
    let ppr = match vineta {
        Some(c) => format!(
            r#"<a:pPr marL="{s}" indent="-{s}" algn="{algn}"><a:lnSpc><a:spcPct val="102500"/></a:lnSpc><a:buClr><a:srgbClr val="{}"/></a:buClr><a:buFont typeface="Arial"/><a:buChar char="•"/></a:pPr>"#,
            c.hex(),
            s = emu(SANGRIA)
        ),
        None => format!(r#"<a:pPr algn="{algn}"><a:lnSpc><a:spcPct val="102500"/></a:lnSpc><a:buNone/></a:pPr>"#),
    };
    let corridas: String = trozos.iter().map(|t| corrida(t, tam)).collect();
    format!(r#"<a:p>{ppr}{corridas}<a:endParaRPr lang="es-MX" sz="{}" dirty="0"><a:latin typeface="{FUENTE}"/></a:endParaRPr></a:p>"#, (tam * 100.0).round() as i64)
}

fn diapositiva(d: &Diapositiva) -> String {
    let mut h = Hoja::new();
    let mut fondo = String::new();
    match &d.contenido {
        Contenido::Portada { titulo, subtitulo, proyectos, presenta } => {
            fondo = format!(r#"<p:bg><p:bgPr><a:solidFill><a:srgbClr val="{}"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>"#, GUINDA.hex());
            h.rect(0.0, ALTO - 6.0, ANCHO, 6.0, DORADO);
            h.texto("Título", &titulo.texto, titulo.tam, MARGEN, 77.0, ANCHO_TEXTO, true, BLANCO, Alinear::Izq);
            h.rect(MARGEN, 138.0, 56.0, 3.0, DORADO);
            h.texto("Fecha de corte", subtitulo, 22.0, MARGEN, 158.0, ANCHO_TEXTO, false, BLANCO, Alinear::Izq);
            let linea: String = proyectos
                .iter()
                .map(|r| parrafo(&[Trozo { texto: r.clone(), negrita: false, color: ROSA_PALIDO }], 16.0, Alinear::Izq, None))
                .collect();
            if !proyectos.is_empty() {
                h.caja("Proyectos", MARGEN, 247.0, ANCHO_TEXTO + 6.0, proyectos.len() as f32 * 20.0, &linea);
            }
            let y = (247.0 + proyectos.len() as f32 * 20.0).max(267.0) + 28.0;
            if !presenta.is_empty() {
                h.texto("Presenta", presenta, 16.0, MARGEN, y, ANCHO_TEXTO, false, ROSA_PALIDO, Alinear::Izq);
            }
        }
        Contenido::Resumen { titulo, filas } => {
            h.texto("Título", &titulo.texto, titulo.tam, MARGEN, maqueta::Y_TITULO, ANCHO_TEXTO + 6.0, true, GUINDA, Alinear::Izq);
            let cols = columnas_resumen();
            let y0 = maqueta::Y_TABLA;
            h.texto("Encabezado", "PROYECTO", 9.0, MARGEN, y0, 200.0, true, GRIS, Alinear::Izq);
            for (i, e) in EstadoAvance::TODOS.iter().enumerate() {
                h.texto("Encabezado", &e.etiqueta().to_uppercase(), 9.0, cols[i] - 50.0, y0, 100.0, true, GRIS, Alinear::Centro);
            }
            h.rect(MARGEN, y0 + 16.0, ANCHO_TEXTO, 1.2, GUINDA);
            let mut y = y0 + 24.0;
            for f in filas {
                if f.total {
                    h.rect(MARGEN, y - 4.0, ANCHO_TEXTO, ALTO_FILA_TOTAL, Color(0xF6, 0xF1, 0xEA));
                }
                h.texto("Proyecto", &f.nombre, 13.0, MARGEN + 8.0, if f.equipo.is_empty() { y + 8.0 } else { y + 2.0 }, cols[0] - 60.0 - MARGEN, true, if f.total { GUINDA } else { TINTA }, Alinear::Izq);
                if !f.equipo.is_empty() {
                    h.texto("Equipo", &f.equipo, 9.5, MARGEN + 8.0, y + 19.0, cols[0] - 60.0 - MARGEN, false, GRIS, Alinear::Izq);
                }
                for (i, e) in EstadoAvance::TODOS.iter().enumerate() {
                    let n = f.cuentas[i];
                    let (s, c) = if n == 0 { ("—".to_string(), GRIS_CLARO) } else { (n.to_string(), e.color()) };
                    h.texto(e.etiqueta(), &s, 16.0, cols[i] - 50.0, y + 6.0, 100.0, true, c, Alinear::Centro);
                }
                y += maqueta::ALTO_FILA;
                if !f.total {
                    h.rect(MARGEN, y - 5.0, ANCHO_TEXTO, 0.6, LINEA);
                }
            }
        }
        Contenido::Lista { titulo, subtitulo, cuerpo, nota } => {
            h.texto("Título", &titulo.texto, titulo.tam, MARGEN, maqueta::Y_TITULO, ANCHO_TEXTO + 6.0, true, GUINDA, Alinear::Izq);
            if let Some(s) = subtitulo {
                h.texto("Equipo", s, 16.0, MARGEN, maqueta::Y_SUBTITULO, ANCHO_TEXTO, false, GRIS, Alinear::Izq);
            }
            let lh = cuerpo.interlineado();
            for (i, p) in cuerpo.parrafos.iter().enumerate() {
                let xml = parrafo(&p.trozos, cuerpo.tam, Alinear::Izq, Some(GUINDA));
                // Un poco de holgura a lo ancho por si PowerPoint mide distinto.
                h.caja(&format!("Punto {}", i + 1), MARGEN, p.y, ANCHO_TEXTO + 6.0, p.renglones.len() as f32 * lh, &xml);
            }
            if let Some(n) = nota {
                h.texto("Siguiente corte", n, 16.0, MARGEN, LIMITE - 16.0, ANCHO_TEXTO, false, GRIS, Alinear::Izq);
            }
        }
    }
    if let Some((texto, numero)) = &d.pie {
        h.rect(MARGEN, Y_PIE - 8.0, ANCHO_TEXTO, 0.6, LINEA);
        h.rect(MARGEN, Y_PIE - 8.6, 40.0, 1.8, DORADO);
        h.texto("Pie", texto, 8.0, MARGEN, Y_PIE, 480.0, false, GRIS, Alinear::Izq);
        h.texto("Número", numero, 8.0, MARGEN + ANCHO_TEXTO - 100.0, Y_PIE, 100.0, true, GUINDA, Alinear::Der);
    }
    format!(
        r#"{XML}<p:sld {NS}><p:cSld>{fondo}<p:spTree>{GRUPO}{}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"#,
        h.figuras
    )
}

pub fn generar(diapositivas: &[Diapositiva], titulo: &str, autor: &str) -> Result<Vec<u8>, String> {
    let n = diapositivas.len();
    let mut archivos: Vec<(String, String)> = vec![
        ("[Content_Types].xml".into(), tipos(n)),
        ("_rels/.rels".into(), RELS_RAIZ.into()),
        ("docProps/core.xml".into(), core(titulo, autor)),
        ("docProps/app.xml".into(), format!(r#"{XML}<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties"><Application>Avances</Application><Slides>{n}</Slides></Properties>"#)),
        ("ppt/presentation.xml".into(), presentacion(n)),
        ("ppt/_rels/presentation.xml.rels".into(), rels_presentacion(n)),
        ("ppt/presProps.xml".into(), format!("{XML}<p:presentationPr {NS}/>")),
        ("ppt/viewProps.xml".into(), format!("{XML}<p:viewPr {NS}/>")),
        ("ppt/tableStyles.xml".into(), format!(r#"{XML}<a:tblStyleLst xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" def="{{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}}"/>"#)),
        ("ppt/slideMasters/slideMaster1.xml".into(), MAESTRO.replace("{XML}", XML).replace("{NS}", NS).replace("{GRUPO}", GRUPO)),
        ("ppt/slideMasters/_rels/slideMaster1.xml.rels".into(), RELS_MAESTRO.into()),
        ("ppt/slideLayouts/slideLayout1.xml".into(), format!(r#"{XML}<p:sldLayout {NS} type="blank" preserve="1"><p:cSld name="En blanco"><p:spTree>{GRUPO}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>"#)),
        ("ppt/slideLayouts/_rels/slideLayout1.xml.rels".into(), RELS_DISENO.into()),
        ("ppt/theme/theme1.xml".into(), tema()),
    ];
    for (i, d) in diapositivas.iter().enumerate() {
        archivos.push((format!("ppt/slides/slide{}.xml", i + 1), diapositiva(d)));
        archivos.push((format!("ppt/slides/_rels/slide{}.xml.rels", i + 1), RELS_DISENO.into()));
    }

    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let opciones = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (ruta, contenido) in archivos {
        zip.start_file(ruta, opciones).map_err(|e| e.to_string())?;
        zip.write_all(contenido.as_bytes()).map_err(|e| e.to_string())?;
    }
    Ok(zip.finish().map_err(|e| e.to_string())?.into_inner())
}

fn tipos(n: usize) -> String {
    let p = "application/vnd.openxmlformats-officedocument.presentationml";
    let diapos: String = (1..=n)
        .map(|i| format!(r#"<Override PartName="/ppt/slides/slide{i}.xml" ContentType="{p}.slide+xml"/>"#))
        .collect();
    format!(
        r#"{XML}<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/ppt/presentation.xml" ContentType="{p}.presentation.main+xml"/><Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="{p}.slideMaster+xml"/><Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="{p}.slideLayout+xml"/><Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/><Override PartName="/ppt/presProps.xml" ContentType="{p}.presProps+xml"/><Override PartName="/ppt/viewProps.xml" ContentType="{p}.viewProps+xml"/><Override PartName="/ppt/tableStyles.xml" ContentType="{p}.tableStyles+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/>{diapos}</Types>"#
    )
}

fn presentacion(n: usize) -> String {
    let ids: String = (1..=n).map(|i| format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, 255 + i, i + 2)).collect();
    format!(
        r#"{XML}<p:presentation {NS} saveSubsetFonts="1"><p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst><p:sldIdLst>{ids}</p:sldIdLst><p:sldSz cx="{}" cy="{}"/><p:notesSz cx="6858000" cy="9144000"/></p:presentation>"#,
        emu(ANCHO),
        emu(ALTO)
    )
}

fn rels_presentacion(n: usize) -> String {
    let r = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
    let diapos: String = (1..=n)
        .map(|i| format!(r#"<Relationship Id="rId{}" Type="{r}/slide" Target="slides/slide{i}.xml"/>"#, i + 2))
        .collect();
    format!(
        r#"{XML}<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="{r}/slideMaster" Target="slideMasters/slideMaster1.xml"/><Relationship Id="rId2" Type="{r}/theme" Target="theme/theme1.xml"/>{diapos}<Relationship Id="rId{a}" Type="{r}/presProps" Target="presProps.xml"/><Relationship Id="rId{b}" Type="{r}/viewProps" Target="viewProps.xml"/><Relationship Id="rId{c}" Type="{r}/tableStyles" Target="tableStyles.xml"/></Relationships>"#,
        a = n + 3,
        b = n + 4,
        c = n + 5
    )
}

fn core(titulo: &str, autor: &str) -> String {
    format!(
        r#"{XML}<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{}</dc:title><dc:creator>{}</dc:creator><cp:lastModifiedBy>{}</cp:lastModifiedBy></cp:coreProperties>"#,
        escapar(titulo),
        escapar(autor),
        escapar(autor)
    )
}

/// El tema lleva la paleta y Montserrat: lo que se agregue a mano en
/// PowerPoint sale con la identidad institucional.
fn tema() -> String {
    let fill = |c: &str| format!(r#"<a:solidFill><a:schemeClr val="{c}"/></a:solidFill>"#);
    let ln = |w: u32| format!(r#"<a:ln w="{w}"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>"#);
    format!(
        r#"{XML}<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Institucional"><a:themeElements><a:clrScheme name="Institucional"><a:dk1><a:srgbClr val="1F1F1F"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="{g}"/></a:dk2><a:lt2><a:srgbClr val="F6F1EA"/></a:lt2><a:accent1><a:srgbClr val="{g}"/></a:accent1><a:accent2><a:srgbClr val="{d}"/></a:accent2><a:accent3><a:srgbClr val="{v}"/></a:accent3><a:accent4><a:srgbClr val="{gc}"/></a:accent4><a:accent5><a:srgbClr val="98989A"/></a:accent5><a:accent6><a:srgbClr val="E6D194"/></a:accent6><a:hlink><a:srgbClr val="{gc}"/></a:hlink><a:folHlink><a:srgbClr val="{g}"/></a:folHlink></a:clrScheme><a:fontScheme name="Institucional"><a:majorFont><a:latin typeface="{FUENTE}"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="{FUENTE}"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme><a:fmtScheme name="Institucional"><a:fillStyleLst>{f}{f}{f}</a:fillStyleLst><a:lnStyleLst>{l1}{l2}{l3}</a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst>{f}{f}{f}</a:bgFillStyleLst></a:fmtScheme></a:themeElements><a:objectDefaults/><a:extraClrSchemeLst/></a:theme>"#,
        g = GUINDA.hex(),
        gc = GUINDA_CLARO.hex(),
        d = DORADO.hex(),
        v = VERDE.hex(),
        f = fill("phClr"),
        l1 = ln(6350),
        l2 = ln(12700),
        l3 = ln(19050),
    )
}

const RELS_RAIZ: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#;

const RELS_MAESTRO: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/></Relationships>"#;

/// Lo usan el diseño (apunta al maestro) y cada diapositiva (apunta al diseño).
const RELS_DISENO: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/></Relationships>"#;

const MAESTRO: &str = r#"{XML}<p:sldMaster {NS}><p:cSld><p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg><p:spTree>{GRUPO}</p:spTree></p:cSld><p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/><p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle><a:lvl1pPr><a:defRPr sz="2800" b="1"><a:solidFill><a:schemeClr val="tx2"/></a:solidFill><a:latin typeface="+mj-lt"/></a:defRPr></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr><a:defRPr sz="1600"><a:solidFill><a:schemeClr val="tx1"/></a:solidFill><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:bodyStyle><p:otherStyle><a:lvl1pPr><a:defRPr sz="1600"><a:latin typeface="+mn-lt"/></a:defRPr></a:lvl1pPr></p:otherStyle></p:txStyles></p:sldMaster>"#;
