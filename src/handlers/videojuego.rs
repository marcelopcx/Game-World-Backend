use actix_web::{delete, get, patch, post, web, HttpResponse};
use sqlx::PgPool;

use crate::auth::AuthenticatedUser;
use crate::error::ApiError;
use crate::models::opinion::{CreateOpinionRequest, OpinionListQuery, UpdateOpinionRequest};
use crate::models::videojuego::{
    CreateVideojuegoRequest, UpdateVideojuegoRequest, VideojuegoListQuery,
};
use crate::services::{opinion, videojuego};

#[get("/videojuegos")]
pub async fn listar_videojuegos(
    pool: web::Data<PgPool>,
    config: web::Data<crate::config::AppConfig>,
    query: web::Query<VideojuegoListQuery>,
) -> Result<HttpResponse, ApiError> {
    let juegos = videojuego::listar(pool.get_ref(), &query, Some(&config.igdb)).await?;
    Ok(HttpResponse::Ok().json(juegos))
}

#[get("/videojuegos/{id}")]
pub async fn obtener_videojuego(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let juego = videojuego::obtener(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(juego))
}

#[post("/videojuegos")]
pub async fn crear_videojuego(
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
    body: web::Json<CreateVideojuegoRequest>,
) -> Result<HttpResponse, ApiError> {
    let juego = videojuego::crear(pool.get_ref(), &body).await?;
    Ok(HttpResponse::Created().json(juego))
}

#[patch("/videojuegos/{id}")]
pub async fn actualizar_videojuego(
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
    path: web::Path<i32>,
    body: web::Json<UpdateVideojuegoRequest>,
) -> Result<HttpResponse, ApiError> {
    let juego = videojuego::actualizar(pool.get_ref(), path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(juego))
}

#[delete("/videojuegos/{id}")]
pub async fn eliminar_videojuego(
    pool: web::Data<PgPool>,
    _user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    videojuego::eliminar(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/videojuegos/{id}/opiniones")]
pub async fn listar_opiniones_videojuego(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    query: web::Query<OpinionListQuery>,
) -> Result<HttpResponse, ApiError> {
    let opiniones =
        opinion::listar_por_videojuego(pool.get_ref(), path.into_inner(), &query).await?;
    Ok(HttpResponse::Ok().json(opiniones))
}

#[get("/videojuegos/{id}/opiniones/resumen")]
pub async fn resumen_opiniones_videojuego(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let resumen = videojuego::resumen_puntajes(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(resumen))
}

#[post("/videojuegos/{id}/opiniones")]
pub async fn crear_opinion_videojuego(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
    body: web::Json<CreateOpinionRequest>,
) -> Result<HttpResponse, ApiError> {
    let op = opinion::crear(pool.get_ref(), user.user_id, path.into_inner(), &body).await?;
    Ok(HttpResponse::Created().json(op))
}

#[get("/opiniones/{id}")]
pub async fn obtener_opinion(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let op = opinion::obtener(pool.get_ref(), path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(op))
}

#[patch("/opiniones/{id}")]
pub async fn actualizar_opinion(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
    body: web::Json<UpdateOpinionRequest>,
) -> Result<HttpResponse, ApiError> {
    let op = opinion::actualizar(pool.get_ref(), user.user_id, path.into_inner(), &body).await?;
    Ok(HttpResponse::Ok().json(op))
}

#[delete("/opiniones/{id}")]
pub async fn eliminar_opinion(
    pool: web::Data<PgPool>,
    user: AuthenticatedUser,
    path: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    opinion::eliminar(pool.get_ref(), user.user_id, path.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
