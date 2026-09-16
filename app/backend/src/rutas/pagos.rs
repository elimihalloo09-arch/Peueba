use axum::{extract::State, Json};

use crate::{error::{Error, Resultado}, modelos::*, Estado};

pub async fn listar(State(e): State<Estado>) -> Resultado<Json<Vec<Pago>>> {
    let filas = sqlx::query_as::<_, Pago>(
        "SELECT id, acreedor_id, concepto, monto, fecha FROM pagos ORDER BY fecha DESC, creado_en DESC",
    )
    .fetch_all(&e.bd)
    .await?;
    Ok(Json(filas))
}

pub async fn crear(State(e): State<Estado>, Json(nuevo): Json<NuevoPago>) -> Resultado<Json<Pago>> {
    if nuevo.monto <= 0 {
        return Err(Error::Invalido("el monto tiene que ser mayor a cero".into()));
    }
    let fila = sqlx::query_as::<_, Pago>(
        "INSERT INTO pagos (acreedor_id, concepto, monto, fecha)
         VALUES ($1,$2,$3, COALESCE($4, CURRENT_DATE))
         RETURNING id, acreedor_id, concepto, monto, fecha",
    )
    .bind(nuevo.acreedor_id)
    .bind(nuevo.concepto)
    .bind(nuevo.monto)
    .bind(nuevo.fecha)
    .fetch_one(&e.bd)
    .await?;
    Ok(Json(fila))
}
