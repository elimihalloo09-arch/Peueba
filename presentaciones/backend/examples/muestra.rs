//! Genera una presentación de ejemplo sin base de datos:
//!
//!     cargo run --example muestra -- /tmp/muestra [plantilla.pptx]
//!
//! Deja muestra.pdf y muestra.pptx para revisar cómo se ven. Sin plantilla
//! busca la primera que haya en ../plantilla/.

use avances_api::dominio::{
    lector,
    maqueta::{self, PasoPres, Presentacion, ProyectoPres},
    pdf,
    plantilla::Plantilla,
    pptx,
};
use chrono::NaiveDate;

const TEXTO: &str = "\
PROYECTO: PORTAL DE TRÁMITES
Equipo de desarrollo
• Continúan las pruebas de usabilidad del formulario de registro con usuarios del piloto. Estado: En proceso.
• En espera de la validación del área normativa sobre los textos de ayuda. Estado: Pendiente.

Tablero de indicadores - Equipo de datos
- La carga de datos de agosto ya está validada y se están corrigiendo las diferencias de redondeo
- El área usuaria tiene una semana para revisar las gráficas del resumen mensual (en proceso)
- Ya llegó el servidor de pruebas; queda pendiente su instalación

Mesa de ayuda:
1. El catálogo de servicios ya está publicado y lo aprobó calidad.
2. Falta capacitar a los agentes del segundo turno - en proceso
3. Una vez capacitados, se abrirá la línea a todas las oficinas.

Asistente virtual (Fase 2)
- Continúan los ajustes a las respuestas sobre citas.
- Reunión el lunes para revisar las preguntas frecuentes nuevas. Estado: pendiente
- Las respuestas de horarios se unificarán en un solo flujo. Estado: en proceso
- Continúa la integración con el sistema de citas.
- Falta agendar la prueba con usuarios para validar los flujos.

Próximos pasos:
- Portal de Trámites: cerrar las pruebas de usabilidad y dar seguimiento a la validación normativa.
- Tablero de indicadores: recibir los comentarios del área usuaria e instalar el servidor de pruebas.
- Mesa de ayuda: terminar la capacitación y abrir la línea a todas las oficinas.
- Asistente virtual: reunión de preguntas frecuentes y agendar la prueba con usuarios.
";

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let ruta = std::env::args().nth(2).map(Into::into).or_else(|| {
        std::fs::read_dir("../plantilla").ok()?.filter_map(Result::ok).map(|e| e.path()).find(|p| {
            p.extension().is_some_and(|x| x == "pptx" || x == "potx")
        })
    });
    let plantilla = ruta.map(|r: std::path::PathBuf| Plantilla::cargar("plantilla", &std::fs::read(&r).unwrap()).unwrap());
    eprintln!("plantilla: {}", if plantilla.is_some() { "sí" } else { "no, con el diseño propio" });
    let l = lector::leer(TEXTO, &[], None);
    for a in &l.avisos {
        eprintln!("aviso: {a}");
    }
    let p = Presentacion {
        titulo: "Avance de proyectos".into(),
        fecha: NaiveDate::from_ymd_opt(2026, 9, 24).unwrap(),
        presenta: "Nombre de quien presenta".into(),
        siguiente: NaiveDate::from_ymd_opt(2026, 10, 1),
        proyectos: l
            .bloques
            .iter()
            .map(|b| ProyectoPres {
                nombre: b.proyecto.clone(),
                equipo: b.equipo.clone(),
                puntos: b.puntos.iter().map(|x| (x.texto.clone(), x.estado)).collect(),
            })
            .collect(),
        pasos: l.pasos.iter().map(|x| PasoPres { proyecto: x.proyecto.clone(), texto: x.texto.clone() }).collect(),
    };
    let d = maqueta::armar(&p);
    std::fs::write(format!("{dir}/muestra.pdf"), pdf::generar(&d, &p.titulo, plantilla.as_ref()).unwrap()).unwrap();
    std::fs::write(format!("{dir}/muestra.pptx"), pptx::generar(&d, &p.titulo, &p.presenta, plantilla.as_ref()).unwrap()).unwrap();
    println!("{} diapositivas en {dir}/muestra.pdf y {dir}/muestra.pptx", d.len());
}
