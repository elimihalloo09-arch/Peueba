use axum::{extract::{Path, State}, Json};
use uuid::Uuid;

use crate::{error::{Error, Resultado}, modelos::*, Estado};

pub async fn listar(State(e): State<Estado>) -> Resultado<Json<Vec<Acreedor>>> {
    let filas = sqlx::query_as::<_, Acreedor>(
        "SELECT * FROM acreedores ORDER BY
            CASE etapa WHEN 'finiquito' THEN 2 WHEN 'pagado' THEN 1 ELSE 0 END,
            saldo_original DESC",
    )
    .fetch_all(&e.bd)
    .await?;
    Ok(Json(filas))
}

pub async fn crear(
    State(e): State<Estado>,
    Json(nuevo): Json<NuevoAcreedor>,
) -> Resultado<Json<Acreedor>> {
    if nuevo.nombre.trim().is_empty() {
        return Err(Error::Invalido("el nombre no puede ir vacío".into()));
    }
    let monto = if nuevo.monto_a_pagar > 0 { nuevo.monto_a_pagar } else { nuevo.saldo_original };
    let fila = sqlx::query_as::<_, Acreedor>(
        "INSERT INTO acreedores
            (nombre, saldo_original, monto_a_pagar, tasa_anual, pago_mensual,
             admite_quita, por_nomina, nota)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *",
    )
    .bind(nuevo.nombre.trim())
    .bind(nuevo.saldo_original.max(0))
    .bind(monto.max(0))
    .bind(nuevo.tasa_anual.max(0.0))
    .bind(nuevo.pago_mensual.max(0))
    .bind(nuevo.admite_quita.unwrap_or(true))
    .bind(nuevo.por_nomina.unwrap_or(false))
    .bind(nuevo.nota.unwrap_or_default())
    .fetch_one(&e.bd)
    .await?;
    Ok(Json(fila))
}

pub async fn actualizar(
    State(e): State<Estado>,
    Path(id): Path<Uuid>,
    Json(cambio): Json<CambioAcreedor>,
) -> Resultado<Json<Acreedor>> {
    if let Some(etapa) = &cambio.etapa {
        if Etapa::desde_texto(etapa).is_none() {
            return Err(Error::Invalido(format!("etapa desconocida: {etapa}")));
        }
    }
    // COALESCE deja pasar solo lo que viene en el cuerpo; lo demás se queda igual.
    let fila = sqlx::query_as::<_, Acreedor>(
        "UPDATE acreedores SET
            nombre            = COALESCE($2, nombre),
            saldo_original    = COALESCE($3, saldo_original),
            monto_a_pagar     = COALESCE($4, monto_a_pagar),
            tasa_anual        = COALESCE($5, tasa_anual),
            pago_mensual      = COALESCE($6, pago_mensual),
            etapa             = COALESCE($7, etapa),
            admite_quita      = COALESCE($8, admite_quita),
            por_nomina        = COALESCE($9, por_nomina),
            tiene_convenio    = COALESCE($10, tiene_convenio),
            tiene_comprobante = COALESCE($11, tiene_comprobante),
            tiene_finiquito   = COALESCE($12, tiene_finiquito),
            nota              = COALESCE($13, nota),
            actualizado_en    = now()
         WHERE id = $1 RETURNING *",
    )
    .bind(id)
    .bind(cambio.nombre)
    .bind(cambio.saldo_original)
    .bind(cambio.monto_a_pagar)
    .bind(cambio.tasa_anual)
    .bind(cambio.pago_mensual)
    .bind(cambio.etapa)
    .bind(cambio.admite_quita)
    .bind(cambio.por_nomina)
    .bind(cambio.tiene_convenio)
    .bind(cambio.tiene_comprobante)
    .bind(cambio.tiene_finiquito)
    .bind(cambio.nota)
    .fetch_optional(&e.bd)
    .await?
    .ok_or(Error::NoEncontrado)?;
    Ok(Json(fila))
}

pub async fn borrar(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Json<serde_json::Value>> {
    let r = sqlx::query("DELETE FROM acreedores WHERE id = $1")
        .bind(id)
        .execute(&e.bd)
        .await?;
    if r.rows_affected() == 0 {
        return Err(Error::NoEncontrado);
    }
    Ok(Json(serde_json::json!({ "borrado": true })))
}
