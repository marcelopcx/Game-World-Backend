use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct OpinionRow {
    pub id_opinion: i32,
    pub puntaje: i32,
    pub comentario: String,
    pub fecha_publicacion: DateTime<Utc>,
    pub id_usuario: i32,
    pub id_videojuego: i32,
    pub id_plataforma: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct OpinionUsuarioInfo {
    pub id_usuario: i32,
    pub username: String,
    pub url_avatar: Option<String>,
    pub es_critico: bool,
}

#[derive(Debug, Serialize)]
pub struct OpinionPlataformaInfo {
    pub id_plataforma: i32,
    pub nombre: String,
}

#[derive(Debug, Serialize)]
pub struct OpinionDetalleResponse {
    pub id_opinion: i32,
    pub puntaje: i32,
    pub comentario: String,
    pub fecha_publicacion: DateTime<Utc>,
    pub usuario: OpinionUsuarioInfo,
    pub plataforma: Option<OpinionPlataformaInfo>,
}

#[derive(Debug, Serialize)]
pub struct OpinionPropiaListItem {
    pub id_opinion: i32,
    pub puntaje: i32,
    pub comentario: String,
    pub fecha_publicacion: DateTime<Utc>,
    pub id_videojuego: i32,
    pub titulo_videojuego: String,
    pub id_plataforma: Option<i32>,
    pub nombre_plataforma: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOpinionRequest {
    pub puntaje: i32,
    pub comentario: String,
    pub id_plataforma: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOpinionRequest {
    pub puntaje: Option<i32>,
    pub comentario: Option<String>,
    pub id_plataforma: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct OpinionListQuery {
    pub es_critico: Option<bool>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
