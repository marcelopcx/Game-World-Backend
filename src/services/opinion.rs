use sqlx::PgPool;

use crate::models::opinion::{
    CreateOpinionRequest, OpinionDetalleResponse, OpinionListQuery, OpinionPlataformaInfo,
    OpinionPropiaListItem, OpinionRow, OpinionUsuarioInfo, UpdateOpinionRequest,
};
use crate::models::pagination::{PaginatedResponse, PaginationParams};

#[derive(Debug, thiserror::Error)]
pub enum OpinionError {
    #[error("solicitud inválida: {0}")]
    InvalidRequest(String),

    #[error("no autorizado")]
    Forbidden,

    #[error("recurso no encontrado")]
    NotFound,

    #[error("conflicto: ya existe una opinión para este juego")]
    Conflict,

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),
}

fn conflicto_opinion(err: sqlx::Error) -> OpinionError {
    if let sqlx::Error::Database(db) = &err {
        if db.constraint().is_some() {
            return OpinionError::Conflict;
        }
    }
    OpinionError::Database(err)
}

fn validar_puntaje(puntaje: i32) -> Result<(), OpinionError> {
    if !(1..=5).contains(&puntaje) {
        return Err(OpinionError::InvalidRequest(
            "el puntaje debe estar entre 1 y 5".into(),
        ));
    }
    Ok(())
}

#[derive(sqlx::FromRow)]
struct OpinionDetalleRow {
    id_opinion: i32,
    puntaje: i32,
    comentario: String,
    fecha_publicacion: chrono::DateTime<chrono::Utc>,
    id_usuario: i32,
    username: String,
    url_avatar: Option<String>,
    es_critico: bool,
    id_plataforma: Option<i32>,
    nombre_plataforma: Option<String>,
}

impl OpinionDetalleRow {
    fn into_response(self) -> OpinionDetalleResponse {
        OpinionDetalleResponse {
            id_opinion: self.id_opinion,
            puntaje: self.puntaje,
            comentario: self.comentario,
            fecha_publicacion: self.fecha_publicacion,
            usuario: OpinionUsuarioInfo {
                id_usuario: self.id_usuario,
                username: self.username,
                url_avatar: self.url_avatar,
                es_critico: self.es_critico,
            },
            plataforma: self.id_plataforma.map(|id_plataforma| OpinionPlataformaInfo {
                id_plataforma,
                nombre: self.nombre_plataforma.unwrap_or_default(),
            }),
        }
    }
}

#[derive(sqlx::FromRow)]
struct OpinionPropiaRow {
    id_opinion: i32,
    puntaje: i32,
    comentario: String,
    fecha_publicacion: chrono::DateTime<chrono::Utc>,
    id_videojuego: i32,
    titulo_videojuego: String,
    id_plataforma: Option<i32>,
    nombre_plataforma: Option<String>,
}

pub async fn listar_por_videojuego(
    pool: &PgPool,
    id_videojuego: i32,
    query: &OpinionListQuery,
) -> Result<PaginatedResponse<OpinionDetalleResponse>, OpinionError> {
    let existe: Option<i32> = sqlx::query_scalar(
        "SELECT id_videojuego FROM videojuegos WHERE id_videojuego = $1",
    )
    .bind(id_videojuego)
    .fetch_optional(pool)
    .await?;

    if existe.is_none() {
        return Err(OpinionError::NotFound);
    }

    let pagination = PaginationParams::from_query(query.page, query.limit);
    let sort_col = match query.sort.as_deref() {
        Some("puntaje") => "o.puntaje",
        _ => "o.fecha_publicacion",
    };
    let order = match query.order.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::bigint
        FROM opinion o
        INNER JOIN usuarios u ON u.id_usuario = o.id_usuario
        WHERE o.id_videojuego = $1
          AND ($2::bool IS NULL OR u.es_critico = $2)
        "#,
    )
    .bind(id_videojuego)
    .bind(query.es_critico)
    .fetch_one(pool)
    .await?;

    let sql = format!(
        r#"
        SELECT
            o.id_opinion,
            o.puntaje,
            o.comentario,
            o.fecha_publicacion,
            u.id_usuario,
            u.username,
            u.url_avatar,
            u.es_critico,
            pl.id_plataforma,
            pl.nombre AS nombre_plataforma
        FROM opinion o
        INNER JOIN usuarios u ON u.id_usuario = o.id_usuario
        LEFT JOIN plataformas pl ON pl.id_plataforma = o.id_plataforma
        WHERE o.id_videojuego = $1
          AND ($2::bool IS NULL OR u.es_critico = $2)
        ORDER BY {sort_col} {order}
        LIMIT $3 OFFSET $4
        "#
    );

    let rows = sqlx::query_as::<_, OpinionDetalleRow>(&sql)
        .bind(id_videojuego)
        .bind(query.es_critico)
        .bind(pagination.limit)
        .bind(pagination.offset())
        .fetch_all(pool)
        .await?;

    let data = rows.into_iter().map(OpinionDetalleRow::into_response).collect();

    Ok(PaginatedResponse {
        data,
        page: pagination.page,
        limit: pagination.limit,
        total,
    })
}

pub async fn listar_por_usuario(
    pool: &PgPool,
    user_id: i32,
    query: &OpinionListQuery,
) -> Result<PaginatedResponse<OpinionPropiaListItem>, OpinionError> {
    let pagination = PaginationParams::from_query(query.page, query.limit);

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM opinion WHERE id_usuario = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query_as::<_, OpinionPropiaRow>(
        r#"
        SELECT
            o.id_opinion,
            o.puntaje,
            o.comentario,
            o.fecha_publicacion,
            o.id_videojuego,
            v.titulo AS titulo_videojuego,
            o.id_plataforma,
            pl.nombre AS nombre_plataforma
        FROM opinion o
        INNER JOIN videojuegos v ON v.id_videojuego = o.id_videojuego
        LEFT JOIN plataformas pl ON pl.id_plataforma = o.id_plataforma
        WHERE o.id_usuario = $1
        ORDER BY o.fecha_publicacion DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(pagination.limit)
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;

    let data = rows
        .into_iter()
        .map(|r| OpinionPropiaListItem {
            id_opinion: r.id_opinion,
            puntaje: r.puntaje,
            comentario: r.comentario,
            fecha_publicacion: r.fecha_publicacion,
            id_videojuego: r.id_videojuego,
            titulo_videojuego: r.titulo_videojuego,
            id_plataforma: r.id_plataforma,
            nombre_plataforma: r.nombre_plataforma,
        })
        .collect();

    Ok(PaginatedResponse {
        data,
        page: pagination.page,
        limit: pagination.limit,
        total,
    })
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<OpinionRow, OpinionError> {
    sqlx::query_as::<_, OpinionRow>(
        r#"
        SELECT
            id_opinion,
            puntaje,
            comentario,
            fecha_publicacion,
            id_usuario,
            id_videojuego,
            id_plataforma
        FROM opinion
        WHERE id_opinion = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(OpinionError::NotFound)
}

pub async fn obtener_detalle(pool: &PgPool, id: i32) -> Result<OpinionDetalleResponse, OpinionError> {
    let row = sqlx::query_as::<_, OpinionDetalleRow>(
        r#"
        SELECT
            o.id_opinion,
            o.puntaje,
            o.comentario,
            o.fecha_publicacion,
            u.id_usuario,
            u.username,
            u.url_avatar,
            u.es_critico,
            pl.id_plataforma,
            pl.nombre AS nombre_plataforma
        FROM opinion o
        INNER JOIN usuarios u ON u.id_usuario = o.id_usuario
        LEFT JOIN plataformas pl ON pl.id_plataforma = o.id_plataforma
        WHERE o.id_opinion = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(OpinionError::NotFound)?;

    Ok(row.into_response())
}

pub async fn crear(
    pool: &PgPool,
    user_id: i32,
    id_videojuego: i32,
    body: &CreateOpinionRequest,
) -> Result<OpinionRow, OpinionError> {
    validar_puntaje(body.puntaje)?;

    if body.comentario.trim().is_empty() {
        return Err(OpinionError::InvalidRequest(
            "el comentario es obligatorio".into(),
        ));
    }

    let existe: Option<i32> = sqlx::query_scalar(
        "SELECT id_videojuego FROM videojuegos WHERE id_videojuego = $1",
    )
    .bind(id_videojuego)
    .fetch_optional(pool)
    .await?;

    if existe.is_none() {
        return Err(OpinionError::NotFound);
    }

    sqlx::query_as::<_, OpinionRow>(
        r#"
        INSERT INTO opinion (puntaje, comentario, id_usuario, id_videojuego, id_plataforma)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id_opinion,
            puntaje,
            comentario,
            fecha_publicacion,
            id_usuario,
            id_videojuego,
            id_plataforma
        "#,
    )
    .bind(body.puntaje)
    .bind(body.comentario.trim())
    .bind(user_id)
    .bind(id_videojuego)
    .bind(body.id_plataforma)
    .fetch_one(pool)
    .await
    .map_err(conflicto_opinion)
}

pub async fn actualizar(
    pool: &PgPool,
    user_id: i32,
    id: i32,
    body: &UpdateOpinionRequest,
) -> Result<OpinionRow, OpinionError> {
    let owner: Option<i32> = sqlx::query_scalar(
        "SELECT id_usuario FROM opinion WHERE id_opinion = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let owner = owner.ok_or(OpinionError::NotFound)?;
    if owner != user_id {
        return Err(OpinionError::Forbidden);
    }

    if let Some(puntaje) = body.puntaje {
        validar_puntaje(puntaje)?;
    }

    sqlx::query_as::<_, OpinionRow>(
        r#"
        UPDATE opinion
        SET
            puntaje = COALESCE($1, puntaje),
            comentario = COALESCE($2, comentario),
            id_plataforma = COALESCE($3, id_plataforma)
        WHERE id_opinion = $4
        RETURNING
            id_opinion,
            puntaje,
            comentario,
            fecha_publicacion,
            id_usuario,
            id_videojuego,
            id_plataforma
        "#,
    )
    .bind(body.puntaje)
    .bind(body.comentario.as_deref().map(str::trim))
    .bind(body.id_plataforma)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}

pub async fn eliminar(pool: &PgPool, user_id: i32, id: i32) -> Result<(), OpinionError> {
    let owner: Option<i32> = sqlx::query_scalar(
        "SELECT id_usuario FROM opinion WHERE id_opinion = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let owner = owner.ok_or(OpinionError::NotFound)?;
    if owner != user_id {
        return Err(OpinionError::Forbidden);
    }

    sqlx::query("DELETE FROM opinion WHERE id_opinion = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
