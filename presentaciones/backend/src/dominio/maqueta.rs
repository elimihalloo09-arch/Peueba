//! Decide qué va en cada diapositiva, en qué diseño de la plantilla y dónde.
//! El PDF y el PowerPoint dibujan esta misma maqueta, así que salen iguales.
//!
//! Orden del deck: portada, resumen (si hay más de un proyecto), una
//! diapositiva por proyecto —o más, si no cabe—, próximos pasos y cierre.

use chrono::NaiveDate;

use super::{
    estilo::{self, *},
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

/// Los diseños de la plantilla, por el nombre que tienen en PowerPoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Diseno {
    /// Grecas, logos y línea dorada; título centrado.
    Portada,
    /// Logo de Margarita Maza abajo a la izquierda y línea guinda abajo.
    Interior1,
    /// Barra guinda junto al título; la parte de abajo queda libre.
    Interior2,
    /// Ilustración con la bandera y franja blanca para el texto de cierre.
    Cierre,
}

impl Diseno {
    pub const TODOS: [Diseno; 4] = [Diseno::Portada, Diseno::Interior1, Diseno::Interior2, Diseno::Cierre];

    pub fn nombre(self) -> &'static str {
        match self {
            Diseno::Portada => "Portada",
            Diseno::Interior1 => "Interior 1",
            Diseno::Interior2 => "Interior 2",
            Diseno::Cierre => "Cierre",
        }
    }

    /// Hasta dónde puede bajar el texto sin pisar lo que trae el fondo.
    pub fn limite(self) -> f32 {
        match self {
            Diseno::Interior1 => 440.0,
            _ => 505.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trozo {
    pub texto: String,
    pub peso: Peso,
    pub color: Color,
}

impl Trozo {
    pub fn new(texto: impl Into<String>, peso: Peso, color: Color) -> Self {
        Self { texto: texto.into(), peso, color }
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
        alto_renglon(self.tam)
    }
}

/// Un texto de un solo estilo, ya partido en renglones, con su caja.
#[derive(Debug, Clone)]
pub struct Bloque {
    pub renglones: Vec<String>,
    pub tam: f32,
    pub peso: Peso,
    pub color: Color,
    pub x: f32,
    pub y: f32,
    pub ancho: f32,
    pub centrado: bool,
}

impl Bloque {
    pub fn alto(&self) -> f32 {
        self.renglones.len() as f32 * alto_renglon(self.tam)
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
pub enum Contenido {
    /// Bloques sueltos: la portada y el cierre.
    Libre(Vec<Bloque>),
    Resumen {
        titulo: Bloque,
        filas: Vec<Fila>,
    },
    Lista {
        titulo: Bloque,
        subtitulo: Option<Bloque>,
        cuerpo: Cuerpo,
        nota: Option<Bloque>,
    },
}

#[derive(Debug, Clone)]
pub struct Diapositiva {
    pub diseno: Diseno,
    pub contenido: Contenido,
}

// Medidas de la plantilla.
const Y_TITULO: f32 = 19.8;
const Y_CUERPO_MIN: f32 = 144.0;
pub const Y_TABLA: f32 = 150.0;
pub const ALTO_FILA: f32 = 46.0;
const FILAS_POR_HOJA: usize = 6;
const TAMANOS: [f32; 3] = [20.0, 18.0, 16.0];

pub fn armar(p: &Presentacion) -> Vec<Diapositiva> {
    let mut d = vec![portada(p)];

    if p.proyectos.len() >= 2 {
        d.extend(resumen(p));
    }

    for pr in &p.proyectos {
        let parrafos: Vec<Vec<Trozo>> = pr.puntos.iter().map(|(t, e)| trozos_punto(t, *e)).collect();
        let equipo = (!pr.equipo.trim().is_empty()).then(|| pr.equipo.trim().to_string());
        d.extend(lista(Diseno::Interior2, &pr.nombre, equipo, &parrafos, None));
    }

    let pasos = pasos_o_resumen(p);
    let nota = p.siguiente.map(|s| format!("Siguiente corte: {}.", fechas::con_dia(s)));
    if !pasos.is_empty() || nota.is_some() {
        d.extend(lista(Diseno::Interior1, "Próximos pasos", None, &pasos, nota));
    }

    d.push(Diapositiva {
        diseno: Diseno::Cierre,
        contenido: Contenido::Libre(vec![Bloque {
            renglones: vec!["Gracias".into()],
            tam: 40.0,
            peso: Peso::Seminegrita,
            color: GUINDA_CIERRE,
            x: 363.0,
            y: 214.0,
            ancho: 520.0,
            centrado: false,
        }]),
    });
    d
}

fn portada(p: &Presentacion) -> Diapositiva {
    // El título va centrado en el cuadro de la plantilla, en uno o dos "pisos",
    // pegado abajo, justo encima de la línea dorada del fondo.
    let (renglones, tam) = ajustar(p.titulo.trim(), 828.0, 44.0, 32.0, 2, Peso::Negrita);
    let mut titulo = Bloque { renglones, tam, peso: Peso::Negrita, color: GUINDA, x: 66.0, y: 0.0, ancho: 828.0, centrado: true };
    titulo.y = 250.0 - titulo.alto();

    let mut bloques = vec![
        titulo,
        Bloque {
            renglones: vec![format!("Corte al {}", fechas::larga(p.fecha))],
            tam: 20.0,
            peso: Peso::Medio,
            color: GUINDA_OSCURO,
            x: 66.0,
            y: 284.0,
            ancho: 828.0,
            centrado: true,
        },
    ];
    let mut y = 318.0;
    if !p.presenta.trim().is_empty() {
        bloques.push(Bloque {
            renglones: vec![p.presenta.trim().to_string()],
            tam: 16.0,
            peso: Peso::Regular,
            color: GUINDA_OSCURO,
            x: 66.0,
            y,
            ancho: 828.0,
            centrado: true,
        });
        y += 30.0;
    }
    // Los nombres cortos: "Asistente virtual", sin "(Fase 2)".
    let nombres: Vec<String> = p.proyectos.iter().map(|x| corto(&x.nombre).to_string()).collect();
    if !nombres.is_empty() {
        let linea = vec![Trozo::new(nombres.join(" · "), Peso::Regular, GRIS)];
        let renglones: Vec<String> = partir(&linea, 14.0, 760.0)
            .into_iter()
            .take(3)
            .map(|r| r.into_iter().map(|t| t.texto).collect())
            .collect();
        bloques.push(Bloque { renglones, tam: 14.0, peso: Peso::Regular, color: GRIS, x: 66.0, y, ancho: 828.0, centrado: true });
    }
    Diapositiva { diseno: Diseno::Portada, contenido: Contenido::Libre(bloques) }
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
    let mut hojas: Vec<Vec<Fila>> = vec![];
    while !filas.is_empty() {
        let resto = filas.split_off(filas.len().min(FILAS_POR_HOJA));
        hojas.push(std::mem::replace(&mut filas, resto));
    }
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
            Diapositiva { diseno: Diseno::Interior2, contenido: Contenido::Resumen { titulo: titulo(&t), filas } }
        })
        .collect()
}

/// Una o varias diapositivas de viñetas: si no cabe, se reparte en (1/2), (2/2).
fn lista(diseno: Diseno, t: &str, subtitulo: Option<String>, parrafos: &[Vec<Trozo>], nota: Option<String>) -> Vec<Diapositiva> {
    let mut tit = titulo(t);
    let mut sub = subtitulo.map(|s| Bloque {
        renglones: vec![s],
        tam: 20.0,
        peso: Peso::Medio,
        color: GUINDA_OSCURO,
        x: MARGEN,
        y: tit.y + tit.alto() + 6.0,
        ancho: ANCHO_TITULO,
        centrado: false,
    });
    let fin_cabeza = sub.as_ref().map(|s| s.y + s.alto()).unwrap_or(tit.y + tit.alto());
    let y0 = (fin_cabeza + 22.0).max(Y_CUERPO_MIN);
    let limite = if nota.is_some() { diseno.limite() - 34.0 } else { diseno.limite() };
    let hojas = repartir(parrafos, y0, limite);
    let n = hojas.len();
    let mut v = vec![];
    for (i, cuerpo) in hojas.into_iter().enumerate() {
        if n > 1 {
            tit = titulo(&format!("{t} ({}/{n})", i + 1));
            if let Some(s) = sub.as_mut() {
                s.y = tit.y + tit.alto() + 6.0;
            }
        }
        let nota = (i + 1 == n).then(|| nota.clone()).flatten().map(|txt| Bloque {
            renglones: vec![txt],
            tam: 16.0,
            peso: Peso::Medio,
            color: GUINDA_OSCURO,
            x: MARGEN,
            y: diseno.limite() - 20.0,
            ancho: ANCHO_TEXTO,
            centrado: false,
        });
        v.push(Diapositiva {
            diseno,
            contenido: Contenido::Lista { titulo: tit.clone(), subtitulo: sub.clone(), cuerpo, nota },
        });
    }
    v
}

/// Los próximos pasos escritos; si no hay, uno por proyecto con lo que sigue abierto.
fn pasos_o_resumen(p: &Presentacion) -> Vec<Vec<Trozo>> {
    if !p.pasos.is_empty() {
        return p
            .pasos
            .iter()
            .map(|x| match &x.proyecto {
                Some(pr) => vec![
                    Trozo::new(format!("{}: ", corto(pr)), Peso::Negrita, GUINDA),
                    Trozo::new(en_minuscula(&x.texto), Peso::Regular, TINTA),
                ],
                None => vec![Trozo::new(x.texto.clone(), Peso::Regular, TINTA)],
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
            Some(vec![
                Trozo::new(format!("{}: ", corto(&pr.nombre)), Peso::Negrita, GUINDA),
                Trozo::new(format!("dar seguimiento a {}.", partes.join(" y ")), Peso::Regular, TINTA),
            ])
        })
        .collect()
}

fn trozos_punto(texto: &str, estado: Option<EstadoAvance>) -> Vec<Trozo> {
    let mut v = vec![Trozo::new(texto.trim(), Peso::Regular, TINTA)];
    if let Some(e) = estado {
        // Espacios duros: "Estado: En proceso." no se parte entre renglones.
        v.push(Trozo::new(" Estado:\u{a0}", Peso::Regular, GRIS));
        v.push(Trozo::new(format!("{}.", e.etiqueta().replace(' ', "\u{a0}")), Peso::Negrita, e.color()));
    }
    v
}

/// "Asistente virtual (Fase 2)" → "Asistente virtual".
fn corto(nombre: &str) -> &str {
    nombre.split(" (").next().unwrap_or(nombre).trim()
}

/// "Cerrar las pruebas" → "cerrar las pruebas", pero "QA revisará" se queda.
fn en_minuscula(s: &str) -> String {
    let mut cs = s.chars();
    match (cs.next(), cs.next()) {
        (Some(a), Some(b)) if a.is_uppercase() && b.is_lowercase() => a.to_lowercase().collect::<String>() + &s[a.len_utf8()..],
        _ => s.to_string(),
    }
}

/// Título de diapositiva interior: 40 pt como la plantilla, en uno o dos
/// renglones antes de llegar a los logos; si no cabe, baja de tamaño.
fn titulo(t: &str) -> Bloque {
    let (renglones, tam) = ajustar(t, ANCHO_TITULO, 40.0, 26.0, 2, Peso::Negrita);
    Bloque { renglones, tam, peso: Peso::Negrita, color: GUINDA, x: MARGEN, y: Y_TITULO, ancho: ANCHO_TITULO, centrado: false }
}

/// Parte `t` en a lo más `max` renglones, bajando de 2 en 2 puntos desde `tam`
/// hasta `minimo`. Al mínimo usa los renglones que hagan falta: nunca se sale
/// de su ancho, que en los títulos es lo que lo separa de los logos.
fn ajustar(t: &str, ancho: f32, tam: f32, minimo: f32, max: usize, peso: Peso) -> (Vec<String>, f32) {
    let mut tam = tam;
    loop {
        let r = partir(&[Trozo::new(t, peso, TINTA)], tam, ancho);
        if r.len() <= max || tam <= minimo {
            return (r.into_iter().map(|x| x.into_iter().map(|t| t.texto).collect()).collect(), tam);
        }
        tam -= 2.0;
    }
}

/// Acomoda los párrafos: primero intenta que todo quepa en una hoja bajando un
/// poco la letra (20 → 16 pt); si ni así, reparte en varias hojas.
fn repartir(parrafos: &[Vec<Trozo>], y0: f32, limite: f32) -> Vec<Cuerpo> {
    let alto_disponible = limite - y0;
    let ancho = ANCHO_TEXTO - SANGRIA;
    for &tam in &TAMANOS {
        let partidos: Vec<Vec<Vec<Trozo>>> = parrafos.iter().map(|p| partir(p, tam, ancho)).collect();
        let lh = alto_renglon(tam);
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
            let pieza = Trozo::new(palabra, t.peso, t.color);
            match unidades.last_mut() {
                Some(u) if pegada => u.push(pieza),
                _ => unidades.push(vec![pieza]),
            }
            pegada = false;
        }
    }

    let espacio = estilo::medir(" ", tam, Peso::Regular);
    let mut renglones: Vec<Vec<Trozo>> = vec![];
    let mut renglon: Vec<Trozo> = vec![];
    let mut x = 0.0;
    for u in unidades {
        let w: f32 = u.iter().map(|p| medir(&p.texto, tam, p.peso)).sum();
        if !renglon.is_empty() && x + espacio + w > ancho {
            renglones.push(std::mem::take(&mut renglon));
            x = 0.0;
        }
        let sp = if renglon.is_empty() { 0.0 } else { espacio };
        for (j, p) in u.into_iter().enumerate() {
            let texto = if j == 0 && sp > 0.0 { format!(" {}", p.texto) } else { p.texto };
            match renglon.last_mut() {
                Some(r) if r.peso == p.peso && r.color == p.color => r.texto.push_str(&texto),
                _ => renglon.push(Trozo::new(texto, p.peso, p.color)),
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
                        .map(|i| (format!("Avance número {i} con un texto de largo regular para ver cómo se acomoda en la diapositiva y si parte bien."), Some(EstadoAvance::EnProceso)))
                        .collect(),
                },
                ProyectoPres {
                    nombre: "Asistente virtual (Fase 2)".into(),
                    equipo: String::new(),
                    puntos: vec![("Se corrigió el flujo de citas.".into(), Some(EstadoAvance::Concluido)), ("Falta la sesión.".into(), Some(EstadoAvance::Pendiente))],
                },
            ],
            pasos: vec![],
        }
    }

    fn texto(b: &Bloque) -> String {
        b.renglones.join(" ")
    }

    #[test]
    fn arma_portada_resumen_proyectos_pasos_y_cierre() {
        let d = armar(&pres(3));
        let disenos: Vec<Diseno> = d.iter().map(|x| x.diseno).collect();
        use Diseno::*;
        assert_eq!(disenos, vec![Portada, Interior2, Interior2, Interior2, Interior1, Cierre]);
        let Contenido::Libre(b) = &d[0].contenido else { panic!() };
        assert_eq!(texto(&b[0]), "Avance de proyectos");
        assert_eq!(texto(&b[1]), "Corte al 24 de septiembre de 2026");
        assert_eq!(texto(&b[3]), "Portal de Trámites · Asistente virtual");
        let Contenido::Lista { nota, cuerpo, titulo, .. } = &d[4].contenido else { panic!() };
        assert_eq!(texto(titulo), "Próximos pasos");
        assert_eq!(nota.as_ref().map(texto).as_deref(), Some("Siguiente corte: jueves 1 de octubre de 2026."));
        // Sin pasos escritos, se arman con lo que sigue abierto.
        assert_eq!(cuerpo.parrafos.len(), 2);
    }

    #[test]
    fn muchos_puntos_se_reparten_sin_salirse() {
        let d = armar(&pres(14));
        let hojas: Vec<&Diapositiva> = d
            .iter()
            .filter(|x| matches!(&x.contenido, Contenido::Lista { titulo, .. } if texto(titulo).contains("Portal")))
            .collect();
        assert!(hojas.len() >= 2);
        for h in hojas {
            let Contenido::Lista { cuerpo, titulo, .. } = &h.contenido else { panic!() };
            assert!(texto(titulo).contains('/'));
            let ultimo = cuerpo.parrafos.last().unwrap();
            assert!(ultimo.y + ultimo.renglones.len() as f32 * cuerpo.interlineado() <= h.diseno.limite() + 0.1);
        }
    }

    #[test]
    fn un_titulo_largo_no_pisa_los_logos() {
        for nombre in [
            "Sistema de seguimiento de requerimientos",
            "Sistema de seguimiento de requerimientos de información de las administraciones desconcentradas (1/2)",
        ] {
            let t = titulo(nombre);
            for r in &t.renglones {
                assert!(medir(r, t.tam, Peso::Negrita) <= ANCHO_TITULO + 0.5, "«{r}» se mete bajo los logos");
            }
        }
    }

    #[test]
    fn parte_renglones_dentro_del_ancho() {
        let t = trozos_punto(&"palabra ".repeat(40), Some(EstadoAvance::Pendiente));
        let r = partir(&t, 16.0, 300.0);
        assert!(r.len() > 3);
        for renglon in &r {
            let w: f32 = renglon.iter().map(|x| medir(&x.texto, 16.0, x.peso)).sum();
            assert!(w <= 300.5, "renglón de {w}");
        }
        let ultimo = r.last().unwrap();
        assert_eq!(ultimo.last().unwrap().peso, Peso::Negrita);
        assert_eq!(ultimo.last().unwrap().texto.trim(), "Pendiente.");
    }

    #[test]
    fn la_etiqueta_de_estado_baja_completa_si_no_cabe() {
        let t = trozos_punto("Las respuestas de horarios se unificarán en un solo flujo para todas las oficinas.", Some(EstadoAvance::EnProceso));
        let ancho = medir("Las respuestas de horarios se unificarán en un solo flujo para todas las oficinas. Estado:", 20.0, Peso::Regular) + 5.0;
        let r = partir(&t, 20.0, ancho);
        assert_eq!(r.len(), 2);
        let segundo: String = r[1].iter().map(|x| x.texto.as_str()).collect();
        assert_eq!(segundo, "Estado:\u{a0}En\u{a0}proceso.");
    }
}
