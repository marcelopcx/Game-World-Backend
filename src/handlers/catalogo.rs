use actix_web::{get, HttpResponse};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::services::catalogo;

#[get("/generos")]
pub async fn listar_generos(pool: actix_web::web::Data<PgPool>) -> Result<HttpResponse, ApiError> {
    let generos = catalogo::listar_generos(pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(generos))
}

#[get("/plataformas")]
pub async fn listar_plataformas(
    pool: actix_web::web::Data<PgPool>,
) -> Result<HttpResponse, ApiError> {
    let plataformas = catalogo::listar_plataformas(pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(plataformas))
}
