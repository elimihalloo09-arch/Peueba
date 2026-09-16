use axum::{extract::State, Json};
use serde::Serialize;

use crate::{error::{Error, Resultado}, modelos::*, Estado};

#[derive(Serialize)]
pub struct CuentaPersona {
    pub persona: String,
    pub entregado: i64,
    pub pagado: i64,
    pub saldo: i64,
    pub movimientos: Vec<MovimientoPersonal>,
}

pub async fn listar(State(e): State<Estado>) -> Resultado<Json<Vec<CuentaPersona>>> {
    let movs = sqlx::query_as::<_, MovimientoPersonal>(
        "SELECT id, persona, concepto, monto, tipo, fecha
         FROM movimientos_personales ORDER BY fecha ASC",
    )
    .fetch_all(&e.bd)
    .await?;

    let mut cuentas: Vec<CuentaPersona> = Vec::new();
    for m in movs {
        let pos = cuentas.iter().position(|c| c.persona == m.persona);
        let i = match pos {
            Some(i) => i,
            None => {
                cuentas.push(CuentaPersona {
                    persona: m.persona.clone(),
                    entregado: 0,
                    pagado: 0,
                    saldo: 0,
                    movimientos: Vec::new(),
                });
                cuentas.len() - 1
            }
        };
        if m.tipo == "entrega" {
            cuentas[i].entregado += m.monto;
        } else {
            cuentas[i].pagado += m.monto;
        }
        cuentas[i].movimientos.push(m);
    }
    for c in cuentas.iter_mut() {
        c.saldo = (c.entregado - c.pagado).max(0);
    }
    Ok(Json(cuentas))
}

pub async fn crear(
    State(e): State<Estado>,
    Json(nuevo): Json<NuevoMovimiento>,
) -> Resultado<Json<MovimientoPersonal>> {
    if nuevo.tipo != "entrega" && nuevo.tipo != "pago" {
        return Err(Error::Invalido("el tipo debe ser entrega o pago".into()));
    }
    if nuevo.monto <= 0 {
        return Err(Error::Invalido("el monto tiene que ser mayor a cero".into()));
    }
    let fila = sqlx::query_as::<_, MovimientoPersonal>(
        "INSERT INTO movimientos_personales (persona, concepto, monto, tipo, fecha)
         VALUES ($1,$2,$3,$4, COALESCE($5, CURRENT_DATE))
         RETURNING id, persona, concepto, monto, tipo, fecha",
    )
    .bind(nuevo.persona)
    .bind(nuevo.concepto)
    .bind(nuevo.monto)
    .bind(nuevo.tipo)
    .bind(nuevo.fecha)
    .fetch_one(&e.bd)
    .await?;
    Ok(Json(fila))
}
