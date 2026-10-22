use axum::{extract::{Path, State}, Json};
use uuid::Uuid;

use avances_api::dominio::lector::{self, Lectura};

use crate::{error::{Error, Resultado}, modelos::*, rutas::cortes::detalle, Estado};

/// Lee el texto de un equipo sin guardar nada: devuelve lo que entendió para
/// revisarlo en pantalla antes de sumarlo.
pub async fn leer(State(e): State<Estado>, Json(pedido): Json<PedidoLectura>) -> Resultado<Json<Lectura>> {
    if pedido.texto.trim().is_empty() {
        return Err(Error::Invalido("pega el texto que mandó el equipo".into()));
    }
    let catalogo: Vec<(String, String)> = sqlx::query_as("SELECT nombre, equipo FROM proyectos ORDER BY orden")
        .fetch_all(&e.bd)
        .await?;
    let conocidos: Vec<String> = catalogo.iter().map(|(n, _)| n.clone()).collect();
    let mut l = lector::leer(&pedido.texto, &conocidos, pedido.proyecto.as_deref());
    // Si el texto no dice el equipo, se toma el del catálogo.
    for b in l.bloques.iter_mut().filter(|b| b.equipo.is_empty()) {
        if let Some((_, eq)) = catalogo.iter().find(|(n, _)| *n == b.proyecto) {
            b.equipo = eq.clone();
        }
    }
    Ok(Json(l))
}

/// Suma al corte lo leído y revisado. Los proyectos que no existen se dan de
/// alta; los puntos nuevos se agregan después de los que ya tenía el corte y
/// los repetidos solo actualizan su estado.
pub async fn sumar(
    State(e): State<Estado>,
    Path(corte): Path<Uuid>,
    Json(suma): Json<Suma>,
) -> Resultado<Json<DetalleCorte>> {
    let mut tx = e.bd.begin().await?;
    let existe: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cortes WHERE id = $1")
        .bind(corte)
        .fetch_optional(&mut *tx)
        .await?;
    existe.ok_or(Error::NoEncontrado)?;

    let catalogo: Vec<(Uuid, String)> = sqlx::query_as("SELECT id, nombre FROM proyectos").fetch_all(&mut *tx).await?;
    let mut nombres: Vec<String> = catalogo.iter().map(|(_, n)| n.clone()).collect();
    let mut ids: Vec<Uuid> = catalogo.iter().map(|(i, _)| *i).collect();

    for b in &suma.bloques {
        let nombre = b.proyecto.trim();
        if nombre.is_empty() || b.puntos.is_empty() {
            continue;
        }
        let proyecto = match lector::coincidir(nombre, &nombres).and_then(|n| nombres.iter().position(|x| x == n)) {
            Some(i) => {
                if !b.equipo.trim().is_empty() {
                    sqlx::query("UPDATE proyectos SET equipo = $2 WHERE id = $1 AND equipo = ''")
                        .bind(ids[i])
                        .bind(b.equipo.trim())
                        .execute(&mut *tx)
                        .await?;
                }
                ids[i]
            }
            None => {
                let id: Uuid = sqlx::query_scalar(
                    "INSERT INTO proyectos (nombre, equipo, orden)
                     VALUES ($1, $2, (SELECT COALESCE(max(orden), 0) + 1 FROM proyectos)) RETURNING id",
                )
                .bind(nombre)
                .bind(b.equipo.trim())
                .fetch_one(&mut *tx)
                .await?;
                nombres.push(nombre.to_string());
                ids.push(id);
                id
            }
        };

        // Un punto que ya está en el corte (por ejemplo, arrastrado del anterior)
        // no se repite: se actualiza su estado.
        let existentes: Vec<(Uuid, String, i32)> = sqlx::query_as(
            "SELECT id, texto, orden FROM avances WHERE corte_id = $1 AND proyecto_id = $2",
        )
        .bind(corte)
        .bind(proyecto)
        .fetch_all(&mut *tx)
        .await?;
        let mut orden = existentes.iter().map(|(_, _, o)| *o).max().unwrap_or(0);
        for p in &b.puntos {
            if p.texto.trim().is_empty() {
                continue;
            }
            let clave = lector::clave(&p.texto);
            if let Some((id, _, _)) = existentes.iter().find(|(_, t, _)| lector::clave(t) == clave) {
                if let Some(estado) = p.estado {
                    sqlx::query("UPDATE avances SET estado = $2 WHERE id = $1")
                        .bind(id)
                        .bind(estado.clave())
                        .execute(&mut *tx)
                        .await?;
                }
                continue;
            }
            orden += 1;
            sqlx::query("INSERT INTO avances (corte_id, proyecto_id, texto, estado, orden) VALUES ($1,$2,$3,$4,$5)")
                .bind(corte)
                .bind(proyecto)
                .bind(p.texto.trim())
                .bind(p.estado.map(|x| x.clave()))
                .bind(orden)
                .execute(&mut *tx)
                .await?;
        }
    }

    let mut orden: i32 = sqlx::query_scalar("SELECT COALESCE(max(orden), 0) FROM pasos WHERE corte_id = $1")
        .bind(corte)
        .fetch_one(&mut *tx)
        .await?;
    for p in &suma.pasos {
        if p.texto.trim().is_empty() {
            continue;
        }
        let proyecto = p
            .proyecto
            .as_deref()
            .and_then(|n| lector::coincidir(n, &nombres))
            .and_then(|n| nombres.iter().position(|x| x == n))
            .map(|i| ids[i]);
        orden += 1;
        sqlx::query("INSERT INTO pasos (corte_id, proyecto_id, texto, orden) VALUES ($1,$2,$3,$4)")
            .bind(corte)
            .bind(proyecto)
            .bind(p.texto.trim())
            .bind(orden)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Json(detalle(&e.bd, corte).await?))
}

pub async fn actualizar(
    State(e): State<Estado>,
    Path(id): Path<Uuid>,
    Json(cambio): Json<CambioAvance>,
) -> Resultado<Json<Avance>> {
    if cambio.texto.as_deref().map(|t| t.trim().is_empty()).unwrap_or(false) {
        return Err(Error::Invalido("el avance no puede ir vacío".into()));
    }
    let estado = estado_valido(cambio.estado.as_deref())?;
    let fila = sqlx::query_as::<_, Avance>(
        "UPDATE avances SET
            texto  = COALESCE($2, texto),
            estado = CASE WHEN $3 THEN $4 ELSE estado END,
            orden  = COALESCE($5, orden)
         WHERE id = $1 RETURNING id, proyecto_id, texto, estado, orden",
    )
    .bind(id)
    .bind(cambio.texto.as_deref().map(str::trim))
    .bind(cambio.estado.is_some())
    .bind(estado)
    .bind(cambio.orden)
    .fetch_optional(&e.bd)
    .await?
    .ok_or(Error::NoEncontrado)?;
    Ok(Json(fila))
}

pub async fn borrar(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM avances WHERE id = $1").bind(id).execute(&e.bd).await?;
    Ok(Json(serde_json::json!({ "borrado": true })))
}

pub async fn crear_paso(
    State(e): State<Estado>,
    Path(corte): Path<Uuid>,
    Json(nuevo): Json<NuevoPaso>,
) -> Resultado<Json<Paso>> {
    if nuevo.texto.trim().is_empty() {
        return Err(Error::Invalido("el paso no puede ir vacío".into()));
    }
    let fila = sqlx::query_as::<_, Paso>(
        "INSERT INTO pasos (corte_id, proyecto_id, texto, orden)
         VALUES ($1, $2, $3, (SELECT COALESCE(max(orden), 0) + 1 FROM pasos WHERE corte_id = $1))
         RETURNING id, proyecto_id, texto, orden",
    )
    .bind(corte)
    .bind(nuevo.proyecto_id)
    .bind(lector::limpiar(&nuevo.texto))
    .fetch_one(&e.bd)
    .await?;
    Ok(Json(fila))
}

pub async fn actualizar_paso(
    State(e): State<Estado>,
    Path(id): Path<Uuid>,
    Json(cambio): Json<CambioPaso>,
) -> Resultado<Json<Paso>> {
    if cambio.texto.as_deref().map(|t| t.trim().is_empty()).unwrap_or(false) {
        return Err(Error::Invalido("el paso no puede ir vacío".into()));
    }
    let fila = sqlx::query_as::<_, Paso>(
        "UPDATE pasos SET texto = COALESCE($2, texto), proyecto_id = COALESCE($3, proyecto_id)
         WHERE id = $1 RETURNING id, proyecto_id, texto, orden",
    )
    .bind(id)
    .bind(cambio.texto.as_deref().map(str::trim))
    .bind(cambio.proyecto_id)
    .fetch_optional(&e.bd)
    .await?
    .ok_or(Error::NoEncontrado)?;
    Ok(Json(fila))
}

pub async fn borrar_paso(State(e): State<Estado>, Path(id): Path<Uuid>) -> Resultado<Json<serde_json::Value>> {
    sqlx::query("DELETE FROM pasos WHERE id = $1").bind(id).execute(&e.bd).await?;
    Ok(Json(serde_json::json!({ "borrado": true })))
}
