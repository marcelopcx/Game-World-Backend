use bcrypt::BcryptError;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::pagination::{PaginatedResponse, PaginationParams};
use crate::models::usuario::{
    CreateUsuarioRequest, LoginRequest, PerfilResponse, RegisterRequest, UpdateMeRequest,
    UpdateUsuarioRequest, Usuario, UsuarioListItem, UsuarioListQuery, UsuarioPublicoResponse,
};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("credenciales inválidas")]
    InvalidCredentials,

    #[error("no autorizado")]
    Unauthorized,

    #[error("usuario no encontrado")]
    NotFound,

    #[error("conflicto: usuario o correo ya registrado")]
    Conflict,

    #[error("no autorizado")]
    Forbidden,

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),

    #[error("error al verificar contraseña")]
    PasswordHash(#[from] BcryptError),

    #[error("error al generar token")]
    Token(#[from] jsonwebtoken::errors::Error),
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: i32,
    exp: usize,
}

#[derive(sqlx::FromRow)]
struct UsuarioPasswordRow {
    id_usuario: i32,
    username: String,
    email: String,
    url_avatar: Option<String>,
    es_critico: bool,
    password: String,
}

#[derive(sqlx::FromRow)]
struct UsuarioListRow {
    id_usuario: i32,
    username: String,
    email: String,
    url_avatar: Option<String>,
    es_critico: bool,
    fecha_registro: chrono::DateTime<Utc>,
    nombre: Option<String>,
    apellido: Option<String>,
}

fn optional_trim(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn resolver_avatar(body_url: Option<&String>, default_avatar_url: &str) -> String {
    body_url
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(default_avatar_url)
        .to_string()
}

fn conflicto_duplicado(err: sqlx::Error) -> AuthError {
    if let sqlx::Error::Database(db) = &err {
        if db.constraint().is_some() {
            return AuthError::Conflict;
        }
    }
    AuthError::Database(err)
}

pub fn user_id_from_token(token: &str, secret: &str) -> Result<i32, AuthError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AuthError::Unauthorized)?;

    Ok(data.claims.sub)
}

pub fn create_jwt(
    user_id: i32,
    secret: &str,
    expiration_hours: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = (Utc::now() + Duration::hours(expiration_hours)).timestamp() as usize;
    let claims = Claims { sub: user_id, exp };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub async fn login(
    pool: &PgPool,
    jwt_secret: &str,
    jwt_expiration_hours: i64,
    body: &LoginRequest,
) -> Result<(String, Usuario), AuthError> {
    let identificador = body.username.trim();

    let user = sqlx::query_as::<_, UsuarioPasswordRow>(
        r#"
        SELECT
            id_usuario,
            username,
            email,
            url_avatar,
            es_critico,
            password
        FROM usuarios
        WHERE LOWER(username) = LOWER($1) OR LOWER(email) = LOWER($1)
        "#,
    )
    .bind(identificador)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::InvalidCredentials)?;

    let valid = bcrypt::verify(&body.password, &user.password)?;
    if !valid {
        return Err(AuthError::InvalidCredentials);
    }

    let token = create_jwt(user.id_usuario, jwt_secret, jwt_expiration_hours)?;
    Ok((
        token,
        Usuario {
            id_usuario: user.id_usuario,
            username: user.username,
            email: user.email,
            url_avatar: user.url_avatar,
            es_critico: user.es_critico,
        },
    ))
}

pub async fn register(
    pool: &PgPool,
    body: &RegisterRequest,
    default_avatar_url: &str,
) -> Result<Usuario, AuthError> {
    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)?;
    let avatar = resolver_avatar(None, default_avatar_url);

    let user = sqlx::query_as::<_, Usuario>(
        r#"
        INSERT INTO usuarios (username, email, password, nombre, apellido, url_avatar)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id_usuario, username, email, url_avatar, es_critico
        "#,
    )
    .bind(body.username.trim())
    .bind(body.email.trim())
    .bind(hash)
    .bind(optional_trim(&body.nombre))
    .bind(optional_trim(&body.apellido))
    .bind(&avatar)
    .fetch_one(pool)
    .await
    .map_err(conflicto_duplicado)?;

    Ok(user)
}

#[derive(sqlx::FromRow)]
struct PerfilRow {
    id_usuario: i32,
    username: String,
    email: String,
    url_avatar: Option<String>,
    es_critico: bool,
    fecha_registro: chrono::DateTime<Utc>,
    nombre: Option<String>,
    apellido: Option<String>,
}

pub async fn get_profile(pool: &PgPool, user_id: i32) -> Result<PerfilResponse, AuthError> {
    let row = sqlx::query_as::<_, PerfilRow>(
        r#"
        SELECT
            id_usuario,
            username,
            email,
            url_avatar,
            es_critico,
            fecha_registro,
            nombre,
            apellido
        FROM usuarios
        WHERE id_usuario = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::NotFound)?;

    Ok(PerfilResponse {
        id_usuario: row.id_usuario,
        username: row.username,
        email: row.email,
        url_avatar: row.url_avatar,
        es_critico: row.es_critico,
        fecha_registro: row.fecha_registro,
        nombre: row.nombre,
        apellido: row.apellido,
    })
}

pub async fn update_profile(
    pool: &PgPool,
    user_id: i32,
    body: &UpdateMeRequest,
) -> Result<PerfilResponse, AuthError> {
    update_usuario_internal(pool, user_id, body, None).await
}

pub async fn delete_account(pool: &PgPool, user_id: i32) -> Result<(), AuthError> {
    let result = sqlx::query("DELETE FROM usuarios WHERE id_usuario = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AuthError::NotFound);
    }

    Ok(())
}

pub async fn get_public_profile(
    pool: &PgPool,
    user_id: i32,
) -> Result<UsuarioPublicoResponse, AuthError> {
    let row = sqlx::query_as::<_, UsuarioPublicoResponse>(
        r#"
        SELECT
            id_usuario,
            username,
            url_avatar,
            es_critico,
            nombre,
            apellido
        FROM usuarios
        WHERE id_usuario = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AuthError::NotFound)?;

    Ok(row)
}

pub async fn listar_usuarios(
    pool: &PgPool,
    query: &UsuarioListQuery,
) -> Result<PaginatedResponse<UsuarioListItem>, AuthError> {
    let pagination = PaginationParams::from_query(query.page, query.limit);
    let sort_col = match query.sort.as_deref() {
        Some("fecha_registro") => "fecha_registro",
        _ => "LOWER(username)",
    };
    let order = match query.order.as_deref() {
        Some("desc") => "DESC",
        _ => "ASC",
    };

    let q_pattern = query
        .q
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("%{}%", s.trim().to_lowercase()));

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM usuarios
        WHERE ($1::text IS NULL OR LOWER(username) LIKE $1)
          AND ($2::bool IS NULL OR es_critico = $2)
        "#,
    )
    .bind(&q_pattern)
    .bind(query.es_critico)
    .fetch_one(pool)
    .await?;

    let sql = format!(
        r#"
        SELECT
            id_usuario,
            username,
            email,
            url_avatar,
            es_critico,
            fecha_registro,
            nombre,
            apellido
        FROM usuarios
        WHERE ($1::text IS NULL OR LOWER(username) LIKE $1)
          AND ($2::bool IS NULL OR es_critico = $2)
        ORDER BY {sort_col} {order}
        LIMIT $3 OFFSET $4
        "#
    );

    let rows = sqlx::query_as::<_, UsuarioListRow>(&sql)
        .bind(&q_pattern)
        .bind(query.es_critico)
        .bind(pagination.limit)
        .bind(pagination.offset())
        .fetch_all(pool)
        .await?;

    let data = rows
        .into_iter()
        .map(|r| UsuarioListItem {
            id_usuario: r.id_usuario,
            username: r.username,
            email: r.email,
            url_avatar: r.url_avatar,
            es_critico: r.es_critico,
            fecha_registro: r.fecha_registro,
            nombre: r.nombre,
            apellido: r.apellido,
        })
        .collect();

    Ok(PaginatedResponse {
        data,
        page: pagination.page,
        limit: pagination.limit,
        total,
    })
}

pub async fn crear_usuario(
    pool: &PgPool,
    body: &CreateUsuarioRequest,
    default_avatar_url: &str,
) -> Result<Usuario, AuthError> {
    let hash = bcrypt::hash(&body.password, bcrypt::DEFAULT_COST)?;
    let avatar = resolver_avatar(body.url_avatar.as_ref(), default_avatar_url);

    let user = sqlx::query_as::<_, Usuario>(
        r#"
        INSERT INTO usuarios (username, email, password, nombre, apellido, url_avatar, es_critico)
        VALUES ($1, $2, $3, $4, $5, $6, COALESCE($7, FALSE))
        RETURNING id_usuario, username, email, url_avatar, es_critico
        "#,
    )
    .bind(body.username.trim())
    .bind(body.email.trim())
    .bind(hash)
    .bind(optional_trim(&body.nombre))
    .bind(optional_trim(&body.apellido))
    .bind(&avatar)
    .bind(body.es_critico)
    .fetch_one(pool)
    .await
    .map_err(conflicto_duplicado)?;

    Ok(user)
}

pub async fn actualizar_usuario(
    pool: &PgPool,
    user_id: i32,
    body: &UpdateUsuarioRequest,
) -> Result<PerfilResponse, AuthError> {
    let me_body = UpdateMeRequest {
        username: body.username.clone(),
        email: body.email.clone(),
        password: body.password.clone(),
        url_avatar: body.url_avatar.clone(),
        nombre: body.nombre.clone(),
        apellido: body.apellido.clone(),
    };
    update_usuario_internal(pool, user_id, &me_body, body.es_critico).await
}

pub async fn eliminar_usuario(pool: &PgPool, user_id: i32) -> Result<(), AuthError> {
    delete_account(pool, user_id).await
}

async fn update_usuario_internal(
    pool: &PgPool,
    user_id: i32,
    body: &UpdateMeRequest,
    es_critico: Option<bool>,
) -> Result<PerfilResponse, AuthError> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM usuarios WHERE id_usuario = $1)",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if !exists {
        return Err(AuthError::NotFound);
    }

    let mut tx = pool.begin().await?;

    if let Some(username) = &body.username {
        sqlx::query("UPDATE usuarios SET username = $1 WHERE id_usuario = $2")
            .bind(username.trim())
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(conflicto_duplicado)?;
    }

    if let Some(email) = &body.email {
        sqlx::query("UPDATE usuarios SET email = $1 WHERE id_usuario = $2")
            .bind(email.trim())
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(conflicto_duplicado)?;
    }

    if let Some(url_avatar) = &body.url_avatar {
        sqlx::query("UPDATE usuarios SET url_avatar = $1 WHERE id_usuario = $2")
            .bind(url_avatar)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(es_critico) = es_critico {
        sqlx::query("UPDATE usuarios SET es_critico = $1 WHERE id_usuario = $2")
            .bind(es_critico)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(password) = &body.password {
        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
        sqlx::query("UPDATE usuarios SET password = $1 WHERE id_usuario = $2")
            .bind(hash)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    if body.nombre.is_some() || body.apellido.is_some() {
        sqlx::query(
            r#"
            UPDATE usuarios
            SET
                nombre = COALESCE($1, nombre),
                apellido = COALESCE($2, apellido)
            WHERE id_usuario = $3
            "#,
        )
        .bind(optional_trim(&body.nombre))
        .bind(optional_trim(&body.apellido))
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    get_profile(pool, user_id).await
}
