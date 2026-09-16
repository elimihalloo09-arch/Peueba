use axum::{extract::{Path, State}, Json};
use uuid::Uuid;

use crate::{error::{Error, Resultado}, modelos::*, Estado};

pub async fn listar(State(e): State<Estado>) -> Resultado<Json<Vec<Pendiente>>> {
    let filas = sqlx::query_as::<_, Pendiente>(
        "SELECT id, texto, vence, hecho FROM pendientes
         ORDER BY hecho ASC, vence ASC NULLS LAST, creado_en ASC",
    )
    .fetch_all(&e.bd)
    .await?;
    Ok(Json(filas))
}

pub async fn crear(
    State(e): State<Estado>,
    Json(nuevo): Json<NuevoPendiente>,
) -> Resultado<Json<Pendiente>> {
    if nuevo.texto.trim().is_empty() {
        return Err(Error::Invalido("el pendiente no puede ir vacío".into()));
    }
    let fila = sqlx::query_as::<_, Pendiente>(
        "INSERT INTO pendientes (texto, vence) VALUES ($1,$2)
         RETURNING id, texto, vence, hecho",
    )
    .bind(nuevo.texto.trim())
    .bind(nuevo.vence)
    .fetch_one(&e.bd)
    .await?;
    Ok(Json(fila))
}

pub async fn actualizar(
    State(e): State<Estado>,
    Path(id): Path<Uuid>,
    Json(cambio): Json<CambioPendiente>,
) -> Resultado<Json<Pendiente>> {
    let fila = sqlx::query_as::<_, Pendiente>(
        "UPDATE pendientes SET
            texto = COALESCE($2, texto),
            vence = COALESCE($3, vence),
            hecho = COALESCE($4, hecho)
         WHERE id = $1 RETURNING id, texto, vence, hecho",
    )
    .bind(id)
    .bind(cambio.texto)
    .bind(cambio.vence)
    .bind(cambio.hecho)
    .fetch_optional(&e.bd)
    .await?
    .ok_or(Error::NoEncontrado)?;
    Ok(Json(fila))
}

pub async fn borrar(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM pendientes WHERE id = $1").bind(id).execute(&e.bd).await?;
    Ok(Json(serde_json::json!({ "borrado": true })))
}
