use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no encontrado")]
    NoEncontrado,
    #[error("dato inválido: {0}")]
    Invalido(String),
    #[error("no autorizado")]
    NoAutorizado,
    #[error(transparent)]
    Base(#[from] sqlx::Error),
}

pub type Resultado<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (codigo, mensaje) = match &self {
            Error::NoEncontrado => (StatusCode::NOT_FOUND, self.to_string()),
            Error::Invalido(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Error::NoAutorizado => (StatusCode::UNAUTHORIZED, self.to_string()),
            Error::Base(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "no encontrado".to_string())
            }
            Error::Base(e) => {
                tracing::error!("error de base de datos: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, "error interno".to_string())
            }
        };
        (codigo, Json(json!({ "error": mensaje }))).into_response()
    }
}
