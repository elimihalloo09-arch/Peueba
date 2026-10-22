use axum::{
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use avances_api::dominio::{
    maqueta::{self, PasoPres, Presentacion, ProyectoPres},
    pdf, pptx, EstadoAvance,
};

use crate::{error::{Error, Resultado}, modelos::DetalleCorte, rutas::cortes::detalle, Estado};

pub async fn pdf(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Response> {
    let d = detalle(&e.bd, id).await?;
    let p = presentacion(&d);
    let bytes = pdf::generar(&maqueta::armar(&p), &p.titulo).map_err(Error::Documento)?;
    Ok(archivo(bytes, "application/pdf", &nombre(&d, "pdf")))
}

pub async fn pptx(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Response> {
    let d = detalle(&e.bd, id).await?;
    let p = presentacion(&d);
    let bytes = pptx::generar(&maqueta::armar(&p), &p.titulo, &p.presenta).map_err(Error::Documento)?;
    let tipo = "application/vnd.openxmlformats-officedocument.presentationml.presentation";
    Ok(archivo(bytes, tipo, &nombre(&d, "pptx")))
}

fn presentacion(d: &DetalleCorte) -> Presentacion {
    let nombre_de = |id: Option<Uuid>| {
        id.and_then(|id| d.proyectos.iter().find(|p| p.proyecto.id == id))
            .map(|p| p.proyecto.nombre.clone())
    };
    Presentacion {
        titulo: d.corte.titulo.clone(),
        fecha: d.corte.fecha,
        presenta: d.corte.presenta.clone(),
        siguiente: d.corte.siguiente,
        proyectos: d
            .proyectos
            .iter()
            .map(|p| ProyectoPres {
                nombre: p.proyecto.nombre.clone(),
                equipo: p.proyecto.equipo.clone(),
                puntos: p
                    .avances
                    .iter()
                    .map(|a| (a.texto.clone(), a.estado.as_deref().and_then(EstadoAvance::desde_clave)))
                    .collect(),
            })
            .collect(),
        pasos: d.pasos.iter().map(|p| PasoPres { proyecto: nombre_de(p.proyecto_id), texto: p.texto.clone() }).collect(),
    }
}

/// "avance-de-proyectos-2026-09-24.pdf": sin acentos ni espacios, para que el
/// nombre sobreviva a cualquier correo o carpeta compartida.
fn nombre(d: &DetalleCorte, ext: &str) -> String {
    let base: String = avances_api::dominio::lector::plegar(&d.corte.titulo)
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let base = base.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    format!("{}-{}.{ext}", if base.is_empty() { "avance" } else { &base }, d.corte.fecha)
}

fn archivo(bytes: Vec<u8>, tipo: &str, nombre: &str) -> Response {
    (
        [
            (header::CONTENT_TYPE, tipo.to_string()),
            (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{nombre}\"")),
        ],
        bytes,
    )
        .into_response()
}
