pub mod estilo;
pub mod fechas;
pub mod lector;
pub mod maqueta;
pub mod pdf;
pub mod pptx;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoAvance {
    Concluido,
    EnProceso,
    Pendiente,
}

impl EstadoAvance {
    pub const TODOS: [EstadoAvance; 3] = [Self::Concluido, Self::EnProceso, Self::Pendiente];

    pub fn clave(self) -> &'static str {
        match self {
            Self::Concluido => "concluido",
            Self::EnProceso => "en_proceso",
            Self::Pendiente => "pendiente",
        }
    }

    pub fn desde_clave(s: &str) -> Option<Self> {
        Self::TODOS.into_iter().find(|e| e.clave() == s)
    }

    pub fn etiqueta(self) -> &'static str {
        match self {
            Self::Concluido => "Concluido",
            Self::EnProceso => "En proceso",
            Self::Pendiente => "Pendiente",
        }
    }

    pub fn color(self) -> estilo::Color {
        match self {
            Self::Concluido => estilo::VERDE,
            Self::EnProceso => estilo::DORADO_TEXTO,
            Self::Pendiente => estilo::GUINDA_CLARO,
        }
    }
}
