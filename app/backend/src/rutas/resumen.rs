use axum::{extract::State, Json};

use crate::{error::Resultado, modelos::*, Estado};

pub async fn obtener(State(e): State<Estado>) -> Resultado<Json<Resumen>> {
    let acreedores = sqlx::query_as::<_, Acreedor>("SELECT * FROM acreedores").fetch_all(&e.bd).await?;
    let ingresos = sqlx::query_as::<_, Ingreso>("SELECT * FROM ingresos WHERE activo").fetch_all(&e.bd).await?;
    let gastos = sqlx::query_as::<_, Gasto>("SELECT * FROM gastos").fetch_all(&e.bd).await?;
    let (entregado, pagado): (i64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(monto) FILTER (WHERE tipo = 'entrega'), 0)::bigint,
            COALESCE(SUM(monto) FILTER (WHERE tipo = 'pago'), 0)::bigint
         FROM movimientos_personales",
    )
    .fetch_one(&e.bd)
    .await?;

    let ingreso_mensual: i64 = ingresos.iter().map(|i| i.neto_mensual).sum();
    let gasto_mensual: i64 = gastos.iter().map(|g| g.mensual).sum();
    let gasto_recortable: i64 = gastos.iter().filter(|g| g.recortable).map(|g| g.mensual).sum();

    let vivas: Vec<&Acreedor> = acreedores
        .iter()
        .filter(|a| !Etapa::desde_texto(&a.etapa).map(|e| e.liquidada()).unwrap_or(false))
        .collect();

    // Los descuentos de nómina no compiten por el efectivo: ya vienen restados del neto.
    let pagos_mensuales_deuda: i64 = vivas.iter().filter(|a| !a.por_nomina).map(|a| a.pago_mensual).sum();
    let deuda_total: i64 = acreedores.iter().map(|a| a.monto_a_pagar).sum();
    let deuda_liquidada: i64 = acreedores
        .iter()
        .filter(|a| Etapa::desde_texto(&a.etapa).map(|e| e.liquidada()).unwrap_or(false))
        .map(|a| a.monto_a_pagar)
        .sum();
    let ahorro_por_quitas: i64 = acreedores
        .iter()
        .map(|a| (a.saldo_original - a.monto_a_pagar).max(0))
        .sum();

    Ok(Json(Resumen {
        ingreso_mensual,
        gasto_mensual,
        gasto_recortable,
        pagos_mensuales_deuda,
        deuda_total,
        deuda_liquidada,
        deuda_por_pagar: deuda_total - deuda_liquidada,
        ahorro_por_quitas,
        capacidad_mensual: ingreso_mensual - gasto_mensual,
        saldo_con_personas: (entregado - pagado).max(0),
    }))
}
