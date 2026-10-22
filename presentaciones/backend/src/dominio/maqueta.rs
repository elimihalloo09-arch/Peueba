//! Decide qué va en cada diapositiva y dónde. El PDF y el PowerPoint dibujan
//! esta misma maqueta, así que salen iguales.
//!
//! Orden del deck: portada, resumen (si hay más de un proyecto), una
//! diapositiva por proyecto —o más, si no cabe— y próximos pasos.

use chrono::NaiveDate;

use super::{
    estilo::{self, Color, *},
    fechas, EstadoAvance,
};

pub struct ProyectoPres {
    pub nombre: String,
    pub equipo: String,
    pub puntos: Vec<(String, Option<EstadoAvance>)>,
}

pub struct PasoPres {
    pub proyecto: Option<String>,
    pub texto: String,
}

pub struct Presentacion {
    pub titulo: String,
    pub fecha: NaiveDate,
    pub presenta: String,
    pub siguiente: Option<NaiveDate>,
    pub proyectos: Vec<ProyectoPres>,
    pub pasos: Vec<PasoPres>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trozo {
    pub texto: String,
    pub negrita: bool,
    pub color: Color,
}

impl Trozo {
    fn new(texto: impl Into<String>, negrita: bool, color: Color) -> Self {
        Self { texto: texto.into(), negrita, color }
    }
}

/// Un párrafo con viñeta, ya partido en renglones y con su altura en la página.
#[derive(Debug, Clone)]
pub struct Parrafo {
    pub trozos: Vec<Trozo>,
    pub renglones: Vec<Vec<Trozo>>,
    pub y: f32,
}

#[derive(Debug, Clone)]
pub struct Cuerpo {
    pub tam: f32,
    pub parrafos: Vec<Parrafo>,
}

impl Cuerpo {
    pub fn interlineado(&self) -> f32 {
        self.tam * 1.25
    }
}

#[derive(Debug, Clone)]
pub struct Fila {
    pub nombre: String,
    pub equipo: String,
    /// Concluido, en proceso, pendiente.
    pub cuentas: [usize; 3],
    pub total: bool,
}

#[derive(Debug, Clone)]
pub struct Titulo {
    pub texto: String,
    pub tam: f32,
}

#[derive(Debug, Clone)]
pub enum Contenido {
    Portada {
        titulo: Titulo,
        subtitulo: String,
        proyectos: Vec<String>,
        presenta: String,
    },
    Resumen {
        titulo: Titulo,
        filas: Vec<Fila>,
    },
    Lista {
        titulo: Titulo,
        subtitulo: Option<String>,
        cuerpo: Cuerpo,
        nota: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Diapositiva {
    pub contenido: Contenido,
    /// Texto del pie y "n / total". La portada no lleva pie.
    pub pie: Option<(String, String)>,
}

// Posiciones verticales, tomadas del deck original.
pub const Y_TITULO: f32 = 48.0;
pub const Y_SUBTITULO: f32 = 108.0;
pub const Y_CUERPO_CON_SUBTITULO: f32 = 168.0;
pub const Y_CUERPO: f32 = 112.0;
pub const Y_TABLA: f32 = 112.0;
pub const ALTO_FILA: f32 = 34.0;
const FILAS_POR_HOJA: usize = 6;
const TAMANOS: [f32; 3] = [16.0, 15.0, 14.0];

pub fn armar(p: &Presentacion) -> Vec<Diapositiva> {
    let mut d = vec![portada(p)];

    if p.proyectos.len() >= 2 {
        d.extend(resumen(p));
    }

    for pr in &p.proyectos {
        let titulo = format!("PROYECTO: {}", pr.nombre.to_uppercase());
        let parrafos: Vec<Vec<Trozo>> = pr.puntos.iter().map(|(t, e)| trozos_punto(t, *e)).collect();
        let subtitulo = (!pr.equipo.trim().is_empty()).then(|| pr.equipo.trim().to_string());
        let y0 = if subtitulo.is_some() { Y_CUERPO_CON_SUBTITULO } else { Y_CUERPO };
        let hojas = repartir(&parrafos, y0, LIMITE);
        let n = hojas.len();
        for (i, cuerpo) in hojas.into_iter().enumerate() {
            let t = if n > 1 { format!("{titulo} ({}/{n})", i + 1) } else { titulo.clone() };
            d.push(Diapositiva {
                contenido: Contenido::Lista { titulo: titulo_ajustado(&t), subtitulo: subtitulo.clone(), cuerpo, nota: None },
                pie: None,
            });
        }
    }

    let pasos = pasos_o_resumen(p);
    let nota = p.siguiente.map(|s| format!("Siguiente corte: {}.", fechas::con_dia(s)));
    if !pasos.is_empty() || nota.is_some() {
        let limite = if nota.is_some() { LIMITE - 30.0 } else { LIMITE };
        let hojas = repartir(&pasos, Y_CUERPO, limite);
        let n = hojas.len();
        for (i, cuerpo) in hojas.into_iter().enumerate() {
            let t = if n > 1 { format!("Próximos pasos ({}/{n})", i + 1) } else { "Próximos pasos".into() };
            d.push(Diapositiva {
                contenido: Contenido::Lista { titulo: titulo_ajustado(&t), subtitulo: None, cuerpo, nota: if i + 1 == n { nota.clone() } else { None } },
                pie: None,
            });
        }
    }

    let total = d.len();
    let pie = format!("{} · Corte al {}", p.titulo.trim(), fechas::corta(p.fecha));
    for (i, dia) in d.iter_mut().enumerate().skip(1) {
        dia.pie = Some((pie.clone(), format!("{} / {}", i + 1, total)));
    }
    d
}

fn portada(p: &Presentacion) -> Diapositiva {
    let mut titulo = Titulo { texto: p.titulo.trim().to_string(), tam: 44.0 };
    while titulo.tam > 28.0 && medir(&titulo.texto, titulo.tam, true) > ANCHO_TEXTO {
        titulo.tam -= 2.0;
    }
    // En la portada van los nombres cortos: "Asistente virtual", sin "(Fase 2)".
    let nombres: Vec<String> = p
        .proyectos
        .iter()
        .map(|x| x.nombre.split(" (").next().unwrap_or(&x.nombre).trim().to_string())
        .collect();
    let linea = vec![Trozo::new(nombres.join(" · "), false, ROSA_PALIDO)];
    let proyectos = partir(&linea, 16.0, ANCHO_TEXTO).into_iter().map(|r| r.into_iter().map(|t| t.texto).collect()).collect();
    Diapositiva {
        contenido: Contenido::Portada {
            titulo,
            subtitulo: format!("Corte al {}", fechas::larga(p.fecha)),
            proyectos,
            presenta: p.presenta.trim().to_string(),
        },
        pie: None,
    }
}

fn resumen(p: &Presentacion) -> Vec<Diapositiva> {
    let mut filas: Vec<Fila> = p
        .proyectos
        .iter()
        .map(|pr| {
            let mut c = [0; 3];
            for (_, e) in &pr.puntos {
                match e {
                    Some(EstadoAvance::Concluido) => c[0] += 1,
                    Some(EstadoAvance::EnProceso) => c[1] += 1,
                    Some(EstadoAvance::Pendiente) => c[2] += 1,
                    None => {}
                }
            }
            Fila { nombre: pr.nombre.clone(), equipo: pr.equipo.clone(), cuentas: c, total: false }
        })
        .collect();
    let mut total = [0; 3];
    for f in &filas {
        for i in 0..3 {
            total[i] += f.cuentas[i];
        }
    }
    let puntos: usize = p.proyectos.iter().map(|x| x.puntos.len()).sum();
    let hojas: Vec<Vec<Fila>> = {
        let mut v = vec![];
        while !filas.is_empty() {
            let resto = filas.split_off(filas.len().min(FILAS_POR_HOJA));
            v.push(std::mem::replace(&mut filas, resto));
        }
        v
    };
    let n = hojas.len();
    hojas
        .into_iter()
        .enumerate()
        .map(|(i, mut filas)| {
            if i + 1 == n {
                filas.push(Fila {
                    nombre: "Total".into(),
                    equipo: format!("{} proyectos · {} puntos", p.proyectos.len(), puntos),
                    cuentas: total,
                    total: true,
                });
            }
            let t = if n > 1 { format!("Resumen del corte ({}/{n})", i + 1) } else { "Resumen del corte".into() };
            Diapositiva { contenido: Contenido::Resumen { titulo: titulo_ajustado(&t), filas }, pie: None }
        })
        .collect()
}

/// Los próximos pasos escritos; si no hay, uno por proyecto con lo que sigue abierto.
fn pasos_o_resumen(p: &Presentacion) -> Vec<Vec<Trozo>> {
    if !p.pasos.is_empty() {
        return p
            .pasos
            .iter()
            .map(|x| match &x.proyecto {
                Some(pr) => {
                    let corto = pr.split(" (").next().unwrap_or(pr).trim();
                    vec![Trozo::new(format!("{corto}: "), true, GUINDA), Trozo::new(en_minuscula(&x.texto), false, TINTA)]
                }
                None => vec![Trozo::new(x.texto.clone(), false, TINTA)],
            })
            .collect();
    }
    p.proyectos
        .iter()
        .filter_map(|pr| {
            let proceso = pr.puntos.iter().filter(|(_, e)| *e == Some(EstadoAvance::EnProceso)).count();
            let pendientes = pr.puntos.iter().filter(|(_, e)| *e == Some(EstadoAvance::Pendiente)).count();
            if proceso + pendientes == 0 {
                return None;
            }
            let mut partes = vec![];
            if proceso > 0 {
                partes.push(format!("{proceso} {} en proceso", if proceso == 1 { "punto" } else { "puntos" }));
            }
            if pendientes > 0 {
                partes.push(format!("{pendientes} {}", if pendientes == 1 { "pendiente" } else { "pendientes" }));
            }
            let corto = pr.nombre.split(" (").next().unwrap_or(&pr.nombre).trim();
            Some(vec![
                Trozo::new(format!("{corto}: "), true, GUINDA),
                Trozo::new(format!("dar seguimiento a {}.", partes.join(" y ")), false, TINTA),
            ])
        })
        .collect()
}

fn trozos_punto(texto: &str, estado: Option<EstadoAvance>) -> Vec<Trozo> {
    let mut v = vec![Trozo::new(texto.trim(), false, TINTA)];
    if let Some(e) = estado {
        // Espacios duros: "Estado: En proceso." no se parte entre renglones.
        v.push(Trozo::new(" Estado:\u{a0}", false, GRIS));
        v.push(Trozo::new(format!("{}.", e.etiqueta().replace(' ', "\u{a0}")), true, e.color()));
    }
    v
}

/// "Cerrar las pruebas" → "cerrar las pruebas", pero "QA revisará" se queda.
fn en_minuscula(s: &str) -> String {
    let mut cs = s.chars();
    match (cs.next(), cs.next()) {
        (Some(a), Some(b)) if a.is_uppercase() && b.is_lowercase() => a.to_lowercase().collect::<String>() + &s[a.len_utf8()..],
        _ => s.to_string(),
    }
}

fn titulo_ajustado(t: &str) -> Titulo {
    let mut tam = 28.0;
    while tam > 18.0 && medir(t, tam, true) > ANCHO_TEXTO {
        tam -= 1.0;
    }
    Titulo { texto: t.to_string(), tam }
}

/// Acomoda los párrafos: primero intenta que todo quepa en una hoja bajando un
/// poco la letra (16 → 14 pt); si ni así, reparte en varias hojas.
fn repartir(parrafos: &[Vec<Trozo>], y0: f32, limite: f32) -> Vec<Cuerpo> {
    let alto_disponible = limite - y0;
    let ancho = ANCHO_TEXTO - SANGRIA;
    for &tam in &TAMANOS {
        let partidos: Vec<Vec<Vec<Trozo>>> = parrafos.iter().map(|p| partir(p, tam, ancho)).collect();
        let lh = tam * 1.25;
        let sep = tam * 0.6;
        let alto: f32 = partidos.iter().map(|r| r.len() as f32 * lh).sum::<f32>() + sep * partidos.len().saturating_sub(1) as f32;
        let ultimo = tam == TAMANOS[TAMANOS.len() - 1];
        if alto <= alto_disponible || ultimo {
            let mut hojas = vec![];
            let mut actual: Vec<Parrafo> = vec![];
            let mut y = y0;
            for (trozos, renglones) in parrafos.iter().zip(partidos) {
                let h = renglones.len() as f32 * lh;
                if !actual.is_empty() && y + h > limite {
                    hojas.push(Cuerpo { tam, parrafos: std::mem::take(&mut actual) });
                    y = y0;
                }
                actual.push(Parrafo { trozos: trozos.clone(), renglones, y });
                y += h + sep;
            }
            hojas.push(Cuerpo { tam, parrafos: actual });
            return hojas;
        }
    }
    unreachable!("el último tamaño siempre reparte")
}

/// Parte en renglones a lo ancho, palabra por palabra, respetando el estilo de
/// cada trozo. Un renglón queda como trozos consecutivos del mismo estilo.
pub fn partir(trozos: &[Trozo], tam: f32, ancho: f32) -> Vec<Vec<Trozo>> {
    // Unidades que no se pueden partir: una palabra, o varias pegadas sin
    // espacio entre trozos ("Estado:\u{a0}" + "En\u{a0}proceso.").
    let mut unidades: Vec<Vec<Trozo>> = vec![];
    let mut habia_espacio = true;
    for t in trozos {
        // Solo el espacio normal separa palabras; el duro (U+00A0) las mantiene juntas.
        let mut pegada = !habia_espacio && !t.texto.starts_with(' ');
        habia_espacio = t.texto.ends_with(' ');
        for palabra in t.texto.split(' ').filter(|w| !w.is_empty()) {
            let pieza = Trozo::new(palabra, t.negrita, t.color);
            match unidades.last_mut() {
                Some(u) if pegada => u.push(pieza),
                _ => unidades.push(vec![pieza]),
            }
            pegada = false;
        }
    }

    let espacio = estilo::medir(" ", tam, false);
    let mut renglones: Vec<Vec<Trozo>> = vec![];
    let mut renglon: Vec<Trozo> = vec![];
    let mut x = 0.0;
    for u in unidades {
        let w: f32 = u.iter().map(|p| medir(&p.texto, tam, p.negrita)).sum();
        if !renglon.is_empty() && x + espacio + w > ancho {
            renglones.push(std::mem::take(&mut renglon));
            x = 0.0;
        }
        let sp = if renglon.is_empty() { 0.0 } else { espacio };
        for (j, p) in u.into_iter().enumerate() {
            let texto = if j == 0 && sp > 0.0 { format!(" {}", p.texto) } else { p.texto };
            match renglon.last_mut() {
                Some(r) if r.negrita == p.negrita && r.color == p.color => r.texto.push_str(&texto),
                _ => renglon.push(Trozo::new(texto, p.negrita, p.color)),
            }
        }
        x += sp + w;
    }
    if !renglon.is_empty() {
        renglones.push(renglon);
    }
    if renglones.is_empty() {
        renglones.push(vec![]);
    }
    renglones
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn pres(puntos: usize) -> Presentacion {
        Presentacion {
            titulo: "Avance de proyectos".into(),
            fecha: NaiveDate::from_ymd_opt(2026, 9, 24).unwrap(),
            presenta: "Responsable del área".into(),
            siguiente: NaiveDate::from_ymd_opt(2026, 10, 1),
            proyectos: vec![
                ProyectoPres {
                    nombre: "Portal de Trámites".into(),
                    equipo: "Equipo jurídico".into(),
                    puntos: (0..puntos)
                        .map(|i| (format!("Avance número {i} con un texto de largo regular para ver cómo se acomoda en la diapositiva."), Some(EstadoAvance::EnProceso)))
                        .collect(),
                },
                ProyectoPres {
                    nombre: "Asistente virtual (AOS)".into(),
                    equipo: String::new(),
                    puntos: vec![("Se corrigió la VPN.".into(), Some(EstadoAvance::Concluido)), ("Falta la sesión.".into(), Some(EstadoAvance::Pendiente))],
                },
            ],
            pasos: vec![],
        }
    }

    #[test]
    fn arma_portada_resumen_proyectos_y_pasos() {
        let d = armar(&pres(3));
        assert_eq!(d.len(), 5);
        assert!(matches!(d[0].contenido, Contenido::Portada { .. }));
        assert!(d[0].pie.is_none());
        assert!(matches!(d[1].contenido, Contenido::Resumen { .. }));
        let Contenido::Lista { nota, cuerpo, .. } = &d[4].contenido else { panic!() };
        assert_eq!(nota.as_deref(), Some("Siguiente corte: jueves 1 de octubre de 2026."));
        // Sin pasos escritos, se arman con lo que sigue abierto.
        assert_eq!(cuerpo.parrafos.len(), 2);
        assert_eq!(d[4].pie.as_ref().unwrap().1, "5 / 5");
        let Contenido::Portada { proyectos, .. } = &d[0].contenido else { panic!() };
        assert_eq!(proyectos, &vec!["Portal de Trámites · Asistente virtual".to_string()]);
    }

    #[test]
    fn muchos_puntos_se_reparten_sin_salirse() {
        let d = armar(&pres(14));
        let hojas: Vec<&Diapositiva> = d.iter().filter(|x| matches!(&x.contenido, Contenido::Lista { titulo, .. } if titulo.texto.contains("PORTAL"))).collect();
        assert!(hojas.len() >= 2);
        for h in hojas {
            let Contenido::Lista { cuerpo, titulo, .. } = &h.contenido else { panic!() };
            assert!(titulo.texto.contains('/'));
            let ultimo = cuerpo.parrafos.last().unwrap();
            assert!(ultimo.y + ultimo.renglones.len() as f32 * cuerpo.interlineado() <= LIMITE + 0.1);
        }
    }

    #[test]
    fn parte_renglones_dentro_del_ancho() {
        let t = trozos_punto(&"palabra ".repeat(40), Some(EstadoAvance::Pendiente));
        let r = partir(&t, 16.0, 300.0);
        assert!(r.len() > 3);
        for renglon in &r {
            let w: f32 = renglon.iter().map(|x| medir(&x.texto, 16.0, x.negrita)).sum();
            assert!(w <= 300.5, "renglón de {w}");
        }
        let ultimo = r.last().unwrap();
        assert!(ultimo.last().unwrap().negrita);
        assert_eq!(ultimo.last().unwrap().texto.trim(), "Pendiente.");
    }

    #[test]
    fn la_etiqueta_de_estado_baja_completa_si_no_cabe() {
        let t = trozos_punto("El flujo de Centro de Datos se unificará con el Flujo Estándar.", Some(EstadoAvance::EnProceso));
        let r = partir(&t, 16.0, ANCHO_TEXTO - SANGRIA);
        assert_eq!(r.len(), 2);
        let w: f32 = r[0].iter().map(|x| medir(&x.texto, 16.0, x.negrita)).sum();
        assert!(w <= ANCHO_TEXTO - SANGRIA);
        let segundo: String = r[1].iter().map(|x| x.texto.as_str()).collect();
        assert_eq!(segundo, "Estado:\u{a0}En\u{a0}proceso.");
    }
}
