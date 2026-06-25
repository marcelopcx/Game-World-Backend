use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Usuario {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
    pub es_critico: bool,
}

pub struct UsuarioPassword {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
    pub es_critico: bool,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct PerfilResponse {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
    pub es_critico: bool,
    pub fecha_registro: DateTime<Utc>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UsuarioListItem {
    pub id_usuario: i32,
    pub username: String,
    pub email: String,
    pub url_avatar: Option<String>,
    pub es_critico: bool,
    pub fecha_registro: DateTime<Utc>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UsuarioPublicoResponse {
    pub id_usuario: i32,
    pub username: String,
    pub url_avatar: Option<String>,
    pub es_critico: bool,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: Usuario,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUsuarioRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub es_critico: Option<bool>,
    pub url_avatar: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user: Usuario,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub url_avatar: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUsuarioRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub url_avatar: Option<String>,
    pub nombre: Option<String>,
    pub apellido: Option<String>,
    pub es_critico: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UsuarioListQuery {
    pub q: Option<String>,
    pub es_critico: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<String>,
}
