//! Genera una presentación de ejemplo sin base de datos:
//!
//!     cargo run --example muestra -- /tmp/muestra
//!
//! Deja muestra.pdf y muestra.pptx para revisar cómo se ven.

use avances_api::dominio::{
    lector,
    maqueta::{self, PasoPres, Presentacion, ProyectoPres},
    pdf, pptx,
};
use chrono::NaiveDate;

const TEXTO: &str = "\
PROYECTO: PORTAL DE TRÁMITES
Equipo jurídico
• Continúan las pruebas del checklist y de las fechas automáticas que ya se implementaron. Estado: En proceso.
• Estamos en espera de la retroalimentación de jurídica sobre el rol de administrador. Estado: Pendiente.

Expediente digital - Equipo forense
- Los módulos de Captura y Consulta ya están validados y se están atendiendo las observaciones de QA
- El equipo forense tiene dos semanas para dar su retroalimentación sobre Centro de Control y Cierre (en proceso)
- Ya se recibió el almacenamiento para el proyecto; queda pendiente su configuración y uso

Credenciales:
1. El aplicativo ya está desplegado y QA lo aprobó.
2. Falta separar los roles del aplicativo antes de liberarlo - en proceso
3. Una vez separados los roles, se compartirá con el área usuaria para su uso.

Asistente virtual (AOS)
- Continúan las correcciones de la conexión VPN.
- Sesión el lunes para revisar el flujo Gestión a Gestión. Estado: pendiente
- El flujo de Centro de Datos se unificará con el Flujo Estándar. Estado: en proceso
- Continúa la implementación del flujo con el bus de servicios.
- Falta agendar una sesión con AOS para validar los flujos.

Próximos pasos:
- Portal de Trámites: cerrar las pruebas del checklist y dar seguimiento a la respuesta de jurídica.
- Expediente digital: recibir la retroalimentación del equipo forense y dejar listo el almacenamiento.
- Credenciales: terminar la separación de roles y entregarlo al área usuaria.
- Asistente virtual: sesión de Gestión a Gestión y agendar la sesión con AOS.
";

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());
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
    std::fs::write(format!("{dir}/muestra.pdf"), pdf::generar(&d, &p.titulo).unwrap()).unwrap();
    std::fs::write(format!("{dir}/muestra.pptx"), pptx::generar(&d, &p.titulo, &p.presenta).unwrap()).unwrap();
    println!("{} diapositivas en {dir}/muestra.pdf y {dir}/muestra.pptx", d.len());
}
