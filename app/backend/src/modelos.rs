use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Etapas por las que pasa la negociación con un acreedor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Etapa {
    SinContactar,
    Contactado,
    ConOferta,
    Convenio,
    Pagado,
    Finiquito,
}

impl Etapa {
    pub fn desde_texto(s: &str) -> Option<Self> {
        Some(match s {
            "sin_contactar" => Etapa::SinContactar,
            "contactado" => Etapa::Contactado,
            "con_oferta" => Etapa::ConOferta,
            "convenio" => Etapa::Convenio,
            "pagado" => Etapa::Pagado,
            "finiquito" => Etapa::Finiquito,
            _ => return None,
        })
    }
    /// Una deuda deja de contar como viva cuando ya se pagó.
    pub fn liquidada(&self) -> bool {
        matches!(self, Etapa::Pagado | Etapa::Finiquito)
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Acreedor {
    pub id: Uuid,
    pub nombre: String,
    pub saldo_original: i64,
    pub monto_a_pagar: i64,
    pub tasa_anual: f64,
    pub pago_mensual: i64,
    pub etapa: String,
    pub admite_quita: bool,
    pub por_nomina: bool,
    pub tiene_convenio: bool,
    pub tiene_comprobante: bool,
    pub tiene_finiquito: bool,
    pub nota: String,
    pub creado_en: DateTime<Utc>,
    pub actualizado_en: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct NuevoAcreedor {
    pub nombre: String,
    #[serde(default)]
    pub saldo_original: i64,
    #[serde(default)]
    pub monto_a_pagar: i64,
    #[serde(default)]
    pub tasa_anual: f64,
    #[serde(default)]
    pub pago_mensual: i64,
    #[serde(default)]
    pub admite_quita: Option<bool>,
    #[serde(default)]
    pub por_nomina: Option<bool>,
    #[serde(default)]
    pub nota: Option<String>,
}

/// Todos los campos son opcionales: se actualiza solo lo que venga.
#[derive(Debug, Deserialize)]
pub struct CambioAcreedor {
    pub nombre: Option<String>,
    pub saldo_original: Option<i64>,
    pub monto_a_pagar: Option<i64>,
    pub tasa_anual: Option<f64>,
    pub pago_mensual: Option<i64>,
    pub etapa: Option<String>,
    pub admite_quita: Option<bool>,
    pub por_nomina: Option<bool>,
    pub tiene_convenio: Option<bool>,
    pub tiene_comprobante: Option<bool>,
    pub tiene_finiquito: Option<bool>,
    pub nota: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Pago {
    pub id: Uuid,
    pub acreedor_id: Option<Uuid>,
    pub concepto: String,
    pub monto: i64,
    pub fecha: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct NuevoPago {
    pub acreedor_id: Option<Uuid>,
    pub concepto: String,
    pub monto: i64,
    pub fecha: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Pendiente {
    pub id: Uuid,
    pub texto: String,
    pub vence: Option<NaiveDate>,
    pub hecho: bool,
}

#[derive(Debug, Deserialize)]
pub struct NuevoPendiente {
    pub texto: String,
    pub vence: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CambioPendiente {
    pub texto: Option<String>,
    pub vence: Option<NaiveDate>,
    pub hecho: Option<bool>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MovimientoPersonal {
    pub id: Uuid,
    pub persona: String,
    pub concepto: String,
    pub monto: i64,
    pub tipo: String,
    pub fecha: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct NuevoMovimiento {
    pub persona: String,
    pub concepto: String,
    pub monto: i64,
    pub tipo: String,
    pub fecha: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Ingreso {
    pub id: Uuid,
    pub concepto: String,
    pub neto_mensual: i64,
    pub activo: bool,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Gasto {
    pub id: Uuid,
    pub concepto: String,
    pub categoria: String,
    pub mensual: i64,
    pub recortable: bool,
}

#[derive(Debug, Serialize)]
pub struct Resumen {
    pub ingreso_mensual: i64,
    pub gasto_mensual: i64,
    pub gasto_recortable: i64,
    pub pagos_mensuales_deuda: i64,
    pub deuda_total: i64,
    pub deuda_liquidada: i64,
    pub deuda_por_pagar: i64,
    pub ahorro_por_quitas: i64,
    pub capacidad_mensual: i64,
    pub saldo_con_personas: i64,
}
