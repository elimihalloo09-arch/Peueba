use axum::{extract::{Query, State}, Json};
use serde::Deserialize;

use crate::{
    dominio::simulador::{simular, DeudaSimulada, Metodo, Simulacion},
    error::Resultado,
    modelos::{Acreedor, Etapa},
    Estado,
};

#[derive(Debug, Deserialize)]
pub struct Parametros {
    /// Todo el dinero mensual destinado a deuda, en centavos. Incluye los mínimos.
    pub capacidad: Option<i64>,
    pub metodo: Option<String>,
    #[serde(default)]
    pub apoyo_mensual: i64,
    #[serde(default)]
    pub meses_apoyo: u32,
    /// Si es true, usa el saldo original en vez del monto negociado.
    #[serde(default)]
    pub sin_quitas: bool,
}

pub async fn simulacion(
    State(e): State<Estado>,
    Query(p): Query<Parametros>,
) -> Resultado<Json<Simulacion>> {
    let acreedores = sqlx::query_as::<_, Acreedor>("SELECT * FROM acreedores").fetch_all(&e.bd).await?;

    let deudas: Vec<DeudaSimulada> = acreedores
        .iter()
        .filter(|a| {
            let liquidada = Etapa::desde_texto(&a.etapa).map(|e| e.liquidada()).unwrap_or(false);
            !liquidada && !a.por_nomina
        })
        .map(|a| DeudaSimulada {
            nombre: a.nombre.clone(),
            saldo: if p.sin_quitas { a.saldo_original } else { a.monto_a_pagar },
            pago_mensual: a.pago_mensual,
            tasa_anual: a.tasa_anual,
            // Lo que ya no cobra intereses pero tiene fecha límite se marca urgente.
            urgente: !a.admite_quita && a.tasa_anual == 0.0,
        })
        .collect();

    let metodo = match p.metodo.as_deref() {
        Some("bola_de_nieve") | Some("bola") => Metodo::BolaDeNieve,
        _ => Metodo::Avalancha,
    };

    Ok(Json(simular(
        &deudas,
        p.capacidad.unwrap_or(0).max(0),
        metodo,
        p.apoyo_mensual.max(0),
        p.meses_apoyo,
    )))
}
