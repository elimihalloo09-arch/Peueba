mod error;
mod modelos;
mod rutas;

use axum::{
    extract::Request,
    http::{header, HeaderValue},
    middleware::{self, Next},
    response::Response,
    routing::{get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use rutas::{avances, cortes, documentos, proyectos};

#[derive(Clone)]
pub struct Estado {
    pub bd: sqlx::PgPool,
    pub llave: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "avances_api=info,tower_http=warn".into()),
        )
        .init();

    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://avances:avances@localhost:5433/avances".into());
    let bd = PgPoolOptions::new().max_connections(5).connect(&url).await?;
    sqlx::migrate!("./migrations").run(&bd).await?;
    tracing::info!("migraciones aplicadas");

    let llave = std::env::var("APP_API_KEY").ok().filter(|k| !k.trim().is_empty());
    if llave.is_none() {
        tracing::warn!("APP_API_KEY vacía: la API queda abierta. Ponle una antes de exponerla fuera de tu red.");
    }

    let estado = Estado { bd, llave };

    let origen = std::env::var("ORIGEN_PERMITIDO").unwrap_or_else(|_| "http://localhost:4200".into());
    let cors = match origen.parse::<HeaderValue>() {
        Ok(o) => CorsLayer::new().allow_origin(o),
        Err(_) => CorsLayer::new().allow_origin(Any),
    }
    .allow_methods(Any)
    .allow_headers(Any)
    .expose_headers([header::CONTENT_DISPOSITION]);

    let api = Router::new()
        .route("/proyectos", get(proyectos::listar).post(proyectos::crear))
        .route("/proyectos/:id", put(proyectos::actualizar).delete(proyectos::borrar))
        .route("/cortes", get(cortes::listar).post(cortes::crear))
        .route("/cortes/:id", get(cortes::obtener).put(cortes::actualizar).delete(cortes::borrar))
        .route("/cortes/:id/sumar", post(avances::sumar))
        .route("/cortes/:id/pasos", post(avances::crear_paso))
        .route("/cortes/:id/pdf", get(documentos::pdf))
        .route("/cortes/:id/pptx", get(documentos::pptx))
        .route("/leer", post(avances::leer))
        .route("/avances/:id", put(avances::actualizar).delete(avances::borrar))
        .route("/pasos/:id", put(avances::actualizar_paso).delete(avances::borrar_paso))
        .route_layer(middleware::from_fn_with_state(estado.clone(), guardia))
        .with_state(estado.clone());

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/api", api)
        .layer(cors);

    let puerto: u16 = std::env::var("PUERTO").ok().and_then(|p| p.parse().ok()).unwrap_or(3001);
    let dir = SocketAddr::from(([0, 0, 0, 0], puerto));
    tracing::info!("escuchando en http://{dir}");
    let escucha = tokio::net::TcpListener::bind(dir).await?;
    axum::serve(escucha, app).await?;
    Ok(())
}

/// Deja pasar solo si el header X-API-Key coincide. Si no hay llave configurada,
/// no estorba: es una app de una sola persona corriendo en su máquina.
async fn guardia(
    axum::extract::State(estado): axum::extract::State<Estado>,
    peticion: Request,
    siguiente: Next,
) -> Result<Response, error::Error> {
    let Some(esperada) = estado.llave.as_ref() else {
        return Ok(siguiente.run(peticion).await);
    };
    let recibida = peticion
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if recibida == esperada {
        Ok(siguiente.run(peticion).await)
    } else {
        Err(error::Error::NoAutorizado)
    }
}
