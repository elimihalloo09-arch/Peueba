use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use avances_api::dominio::{lector::{BloqueLeido, PasoLeido}, EstadoAvance};

use crate::error::{Error, Resultado};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Proyecto {
    pub id: Uuid,
    pub nombre: String,
    pub equipo: String,
    pub orden: i32,
    pub activo: bool,
}

#[derive(Debug, Deserialize)]
pub struct NuevoProyecto {
    pub nombre: String,
    pub equipo: Option<String>,
    pub orden: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CambioProyecto {
    pub nombre: Option<String>,
    pub equipo: Option<String>,
    pub orden: Option<i32>,
    pub activo: Option<bool>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Corte {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub titulo: String,
    pub presenta: String,
    pub siguiente: Option<NaiveDate>,
}

/// En la lista de cortes, con cuántos puntos y proyectos lleva cada uno.
#[derive(Debug, Serialize, FromRow)]
pub struct CorteEnLista {
    pub id: Uuid,
    pub fecha: NaiveDate,
    pub titulo: String,
    pub presenta: String,
    pub siguiente: Option<NaiveDate>,
    pub puntos: i64,
    pub proyectos: i64,
}

#[derive(Debug, Deserialize)]
pub struct NuevoCorte {
    pub fecha: NaiveDate,
    pub titulo: Option<String>,
    pub presenta: Option<String>,
    pub siguiente: Option<NaiveDate>,
    /// Copia lo que quedó en proceso o pendiente del corte anterior.
    #[serde(default)]
    pub arrastrar: bool,
}

/// La cabecera completa: se manda toda, así se puede quitar la fecha siguiente.
#[derive(Debug, Deserialize)]
pub struct CabeceraCorte {
    pub fecha: NaiveDate,
    pub titulo: String,
    pub presenta: String,
    pub siguiente: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Avance {
    pub id: Uuid,
    pub proyecto_id: Uuid,
    pub texto: String,
    pub estado: Option<String>,
    pub orden: i32,
}

#[derive(Debug, Deserialize)]
pub struct CambioAvance {
    pub texto: Option<String>,
    /// `""` quita el estado.
    pub estado: Option<String>,
    pub orden: Option<i32>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Paso {
    pub id: Uuid,
    pub proyecto_id: Option<Uuid>,
    pub texto: String,
    pub orden: i32,
}

#[derive(Debug, Deserialize)]
pub struct NuevoPaso {
    pub proyecto_id: Option<Uuid>,
    pub texto: String,
}

#[derive(Debug, Deserialize)]
pub struct CambioPaso {
    pub texto: Option<String>,
    pub proyecto_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ProyectoEnCorte {
    #[serde(flatten)]
    pub proyecto: Proyecto,
    pub avances: Vec<Avance>,
}

#[derive(Debug, Serialize)]
pub struct DetalleCorte {
    #[serde(flatten)]
    pub corte: Corte,
    pub proyectos: Vec<ProyectoEnCorte>,
    pub pasos: Vec<Paso>,
}

#[derive(Debug, Deserialize)]
pub struct PedidoLectura {
    pub texto: String,
    /// A qué proyecto van los puntos que llegan sin encabezado.
    pub proyecto: Option<String>,
}

/// Lo leído, ya revisado en pantalla, para sumarlo al corte.
#[derive(Debug, Deserialize)]
pub struct Suma {
    pub bloques: Vec<BloqueLeido>,
    #[serde(default)]
    pub pasos: Vec<PasoLeido>,
}

/// Valida un estado que llega de afuera. `""` y `null` son "sin estado".
pub fn estado_valido(e: Option<&str>) -> Resultado<Option<String>> {
    match e.map(str::trim) {
        None | Some("") => Ok(None),
        Some(s) => EstadoAvance::desde_clave(s)
            .map(|x| Some(x.clave().to_string()))
            .ok_or_else(|| Error::Invalido(format!("estado desconocido: {s}"))),
    }
}
