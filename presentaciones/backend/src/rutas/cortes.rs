use axum::{extract::{Path, State}, Json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::{Error, Resultado}, modelos::*, Estado};

pub async fn listar(State(e): State<Estado>) -> Resultado<Json<Vec<CorteEnLista>>> {
    let filas = sqlx::query_as::<_, CorteEnLista>(
        "SELECT c.id, c.fecha, c.titulo, c.presenta, c.siguiente,
                count(a.id) AS puntos,
                count(DISTINCT a.proyecto_id) AS proyectos
         FROM cortes c LEFT JOIN avances a ON a.corte_id = c.id
         GROUP BY c.id
         ORDER BY c.fecha DESC, c.creado_en DESC",
    )
    .fetch_all(&e.bd)
    .await?;
    Ok(Json(filas))
}

pub async fn crear(State(e): State<Estado>, Json(nuevo): Json<NuevoCorte>) -> Resultado<Json<DetalleCorte>> {
    let mut tx = e.bd.begin().await?;
    // Sin título ni quién presenta, se toman del corte más reciente.
    let previo: Option<Corte> = sqlx::query_as(
        "SELECT id, fecha, titulo, presenta, siguiente FROM cortes
         ORDER BY (fecha <= $1) DESC, fecha DESC, creado_en DESC LIMIT 1",
    )
    .bind(nuevo.fecha)
    .fetch_optional(&mut *tx)
    .await?;

    let titulo = nuevo
        .titulo
        .filter(|t| !t.trim().is_empty())
        .or_else(|| previo.as_ref().map(|p| p.titulo.clone()))
        .unwrap_or_else(|| "Avance de proyectos".into());
    let presenta = nuevo
        .presenta
        .or_else(|| previo.as_ref().map(|p| p.presenta.clone()))
        .unwrap_or_default();

    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO cortes (fecha, titulo, presenta, siguiente) VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(nuevo.fecha)
    .bind(titulo.trim())
    .bind(presenta.trim())
    .bind(nuevo.siguiente)
    .fetch_one(&mut *tx)
    .await?;

    if let (true, Some(p)) = (nuevo.arrastrar, previo) {
        sqlx::query(
            "INSERT INTO avances (corte_id, proyecto_id, texto, estado, orden)
             SELECT $1, proyecto_id, texto, estado, orden FROM avances
             WHERE corte_id = $2 AND estado IS DISTINCT FROM 'concluido'",
        )
        .bind(id)
        .bind(p.id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(Json(detalle(&e.bd, id).await?))
}

pub async fn obtener(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Json<DetalleCorte>> {
    Ok(Json(detalle(&e.bd, id).await?))
}

pub async fn actualizar(
    State(e): State<Estado>,
    Path(id): Path<Uuid>,
    Json(c): Json<CabeceraCorte>,
) -> Resultado<Json<DetalleCorte>> {
    if c.titulo.trim().is_empty() {
        return Err(Error::Invalido("el título no puede ir vacío".into()));
    }
    let r = sqlx::query("UPDATE cortes SET fecha = $2, titulo = $3, presenta = $4, siguiente = $5 WHERE id = $1")
        .bind(id)
        .bind(c.fecha)
        .bind(c.titulo.trim())
        .bind(c.presenta.trim())
        .bind(c.siguiente)
        .execute(&e.bd)
        .await?;
    if r.rows_affected() == 0 {
        return Err(Error::NoEncontrado);
    }
    Ok(Json(detalle(&e.bd, id).await?))
}

pub async fn borrar(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM cortes WHERE id = $1").bind(id).execute(&e.bd).await?;
    Ok(Json(serde_json::json!({ "borrado": true })))
}

/// El corte con sus proyectos en orden —solo los que tienen avances— y los pasos.
pub async fn detalle(bd: &PgPool, id: Uuid) -> Resultado<DetalleCorte> {
    let corte: Corte = sqlx::query_as("SELECT id, fecha, titulo, presenta, siguiente FROM cortes WHERE id = $1")
        .bind(id)
        .fetch_optional(bd)
        .await?
        .ok_or(Error::NoEncontrado)?;
    let proyectos: Vec<Proyecto> = sqlx::query_as(
        "SELECT p.id, p.nombre, p.equipo, p.orden, p.activo FROM proyectos p
         WHERE EXISTS (SELECT 1 FROM avances a WHERE a.corte_id = $1 AND a.proyecto_id = p.id)
         ORDER BY p.orden, lower(p.nombre)",
    )
    .bind(id)
    .fetch_all(bd)
    .await?;
    let avances: Vec<Avance> = sqlx::query_as(
        "SELECT id, proyecto_id, texto, estado, orden FROM avances WHERE corte_id = $1 ORDER BY orden, creado_en",
    )
    .bind(id)
    .fetch_all(bd)
    .await?;
    let pasos: Vec<Paso> = sqlx::query_as(
        "SELECT id, proyecto_id, texto, orden FROM pasos WHERE corte_id = $1 ORDER BY orden, creado_en",
    )
    .bind(id)
    .fetch_all(bd)
    .await?;

    let proyectos = proyectos
        .into_iter()
        .map(|p| ProyectoEnCorte {
            avances: avances.iter().filter(|a| a.proyecto_id == p.id).cloned().collect(),
            proyecto: p,
        })
        .collect();
    Ok(DetalleCorte { corte, proyectos, pasos })
}
