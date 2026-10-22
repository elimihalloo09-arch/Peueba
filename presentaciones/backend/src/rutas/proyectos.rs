use axum::{extract::{Path, State}, Json};
use uuid::Uuid;

use crate::{error::{Error, Resultado}, modelos::*, Estado};

const COLUMNAS: &str = "id, nombre, equipo, orden, activo";

pub async fn listar(State(e): State<Estado>) -> Resultado<Json<Vec<Proyecto>>> {
    let filas = sqlx::query_as::<_, Proyecto>(&format!(
        "SELECT {COLUMNAS} FROM proyectos ORDER BY activo DESC, orden, lower(nombre)"
    ))
    .fetch_all(&e.bd)
    .await?;
    Ok(Json(filas))
}

pub async fn crear(State(e): State<Estado>, Json(nuevo): Json<NuevoProyecto>) -> Resultado<Json<Proyecto>> {
    let nombre = nuevo.nombre.trim();
    if nombre.is_empty() {
        return Err(Error::Invalido("el proyecto necesita nombre".into()));
    }
    let fila = sqlx::query_as::<_, Proyecto>(&format!(
        "INSERT INTO proyectos (nombre, equipo, orden)
         VALUES ($1, $2, COALESCE($3, (SELECT COALESCE(max(orden), 0) + 1 FROM proyectos)))
         RETURNING {COLUMNAS}"
    ))
    .bind(nombre)
    .bind(nuevo.equipo.as_deref().unwrap_or("").trim())
    .bind(nuevo.orden)
    .fetch_one(&e.bd)
    .await?;
    Ok(Json(fila))
}

pub async fn actualizar(
    State(e): State<Estado>,
    Path(id): Path<Uuid>,
    Json(cambio): Json<CambioProyecto>,
) -> Resultado<Json<Proyecto>> {
    if cambio.nombre.as_deref().map(|n| n.trim().is_empty()).unwrap_or(false) {
        return Err(Error::Invalido("el proyecto necesita nombre".into()));
    }
    let fila = sqlx::query_as::<_, Proyecto>(&format!(
        "UPDATE proyectos SET
            nombre = COALESCE($2, nombre),
            equipo = COALESCE($3, equipo),
            orden  = COALESCE($4, orden),
            activo = COALESCE($5, activo)
         WHERE id = $1 RETURNING {COLUMNAS}"
    ))
    .bind(id)
    .bind(cambio.nombre.as_deref().map(str::trim))
    .bind(cambio.equipo.as_deref().map(str::trim))
    .bind(cambio.orden)
    .bind(cambio.activo)
    .fetch_optional(&e.bd)
    .await?
    .ok_or(Error::NoEncontrado)?;
    Ok(Json(fila))
}

/// Borrar un proyecto borra sus avances en todos los cortes: para dejar de
/// verlo sin perder historia, mejor marcarlo como inactivo.
pub async fn borrar(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM proyectos WHERE id = $1").bind(id).execute(&e.bd).await?;
    Ok(Json(serde_json::json!({ "borrado": true })))
}
