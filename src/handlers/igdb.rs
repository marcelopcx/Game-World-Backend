use actix_web::{get, post, web, HttpResponse};

use crate::auth::AuthenticatedUser;
use crate::config::AppConfig;
use crate::error::ApiError;
use crate::services::igdb;

#[derive(serde::Deserialize)]
pub struct IgdbBuscarQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[get("/igdb/buscar")]
pub async fn buscar_igdb(
    config: web::Data<AppConfig>,
    query: web::Query<IgdbBuscarQuery>,
) -> Result<HttpResponse, ApiError> {
    let resultado = igdb::buscar(
        &config.igdb,
        &query.q,
        query.limit.unwrap_or(10),
    )
    .await?;
    Ok(HttpResponse::Ok().json(resultado))
}

#[get("/igdb/juegos/{id_externa}")]
pub async fn detalle_igdb(
    config: web::Data<AppConfig>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let juego = igdb::obtener_detalle(&config.igdb, &path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(juego))
}

#[post("/igdb/sincronizar/{id_externa}")]
pub async fn sincronizar_igdb(
    pool: web::Data<sqlx::PgPool>,
    config: web::Data<AppConfig>,
    _user: AuthenticatedUser,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let resultado =
        igdb::sincronizar(pool.get_ref(), &config.igdb, &path.into_inner()).await?;

    let status = if resultado.sincronizado {
        actix_web::http::StatusCode::CREATED
    } else {
        actix_web::http::StatusCode::OK
    };

    Ok(HttpResponse::build(status).json(resultado))
}
