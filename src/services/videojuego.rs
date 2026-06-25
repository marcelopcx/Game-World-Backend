use sqlx::PgPool;

use crate::config::IgdbConfig;
use crate::models::catalogo::{Genero, Plataforma};
use crate::models::pagination::{PaginatedResponse, PaginationParams};
use crate::models::videojuego::{
    CreateVideojuegoRequest, UpdateVideojuegoRequest, VideojuegoListQuery, VideojuegoResponse,
    VideojuegoResumenPuntajes, VideojuegoRow,
};

#[derive(Debug, thiserror::Error)]
pub enum VideojuegoError {
    #[error("solicitud inválida: {0}")]
    InvalidRequest(String),

    #[error("recurso no encontrado")]
    NotFound,

    #[error("conflicto: id externa duplicada")]
    Conflict,

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),
}

fn conflicto_id_externa(err: sqlx::Error) -> VideojuegoError {
    if let sqlx::Error::Database(db) = &err {
        if db.constraint().is_some() {
            return VideojuegoError::Conflict;
        }
    }
    VideojuegoError::Database(err)
}

const VIDEOJUEGO_SELECT: &str = r#"
    SELECT
        id_videojuego,
        id_externa,
        titulo,
        sinopsis,
        fecha_lanzamiento,
        url_caratula,
        desarrollador,
        editor,
        promedio_puntaje_usuarios::float8 AS promedio_puntaje_usuarios,
        promedio_puntaje_criticos::float8 AS promedio_puntaje_criticos,
        cantidad_opiniones_usuarios,
        cantidad_opiniones_criticos
    FROM videojuegos
"#;

pub async fn contar(pool: &PgPool) -> Result<i64, VideojuegoError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM videojuegos")
        .fetch_one(pool)
        .await?;
    Ok(total)
}

pub async fn listar(
    pool: &PgPool,
    query: &VideojuegoListQuery,
    igdb: Option<&IgdbConfig>,
) -> Result<PaginatedResponse<VideojuegoResponse>, VideojuegoError> {
    let mut resultado = listar_desde_db(pool, query).await?;

    let busca_externo = query
        .q
        .as_deref()
        .is_some_and(|s| !s.trim().is_empty())
        && resultado.total == 0;

    if busca_externo {
        if let Some(config) = igdb.filter(|c| !c.client_id.is_empty()) {
            let termino = query.q.as_deref().unwrap_or("").trim();
            let limite = query.limit.unwrap_or(20);
            let _ = crate::services::igdb::buscar_y_persistir(pool, config, termino, limite).await;
            resultado = listar_desde_db(pool, query).await?;
        }
    }

    Ok(resultado)
}

async fn listar_desde_db(
    pool: &PgPool,
    query: &VideojuegoListQuery,
) -> Result<PaginatedResponse<VideojuegoResponse>, VideojuegoError> {
    let pagination = PaginationParams::from_query(query.page, query.limit);
    let tipo_puntaje = query.tipo_puntaje.as_deref().unwrap_or("usuarios");
    let puntaje_col = if tipo_puntaje == "criticos" {
        "v2.promedio_puntaje_criticos"
    } else {
        "v2.promedio_puntaje_usuarios"
    };

    let sort_col = match query.sort.as_deref() {
        Some("fecha") => "v.fecha_lanzamiento",
        Some("puntaje_criticos") => "v.promedio_puntaje_criticos",
        Some("puntaje_usuarios") => "v.promedio_puntaje_usuarios",
        _ => "LOWER(v.titulo)",
    };
    let order = match query.order.as_deref() {
        Some("desc") => "DESC NULLS LAST",
        _ => "ASC NULLS LAST",
    };

    let q_pattern = query
        .q
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!("%{}%", s.trim().to_lowercase()));

    let count_sql = format!(
        r#"
        SELECT COUNT(DISTINCT v2.id_videojuego)::bigint
        FROM videojuegos v2
        LEFT JOIN videojuegos_generos vg ON vg.id_videojuego = v2.id_videojuego
        LEFT JOIN videojuegos_plataformas vp ON vp.id_videojuego = v2.id_videojuego
        WHERE ($1::text IS NULL OR LOWER(v2.titulo) LIKE $1)
          AND ($2::int IS NULL OR vg.id_genero = $2)
          AND ($3::int IS NULL OR vp.id_plataforma = $3)
          AND ($4::date IS NULL OR v2.fecha_lanzamiento >= $4)
          AND ($5::date IS NULL OR v2.fecha_lanzamiento <= $5)
          AND ($6::float8 IS NULL OR {puntaje_col} >= $6)
        "#
    );

    let puntaje_min = query.puntaje_min;
    let total: i64 = sqlx::query_scalar(&count_sql)
        .bind(&q_pattern)
        .bind(query.genero)
        .bind(query.plataforma)
        .bind(query.fecha_desde)
        .bind(query.fecha_hasta)
        .bind(puntaje_min)
        .fetch_one(pool)
        .await?;

    let list_sql = format!(
        r#"
        SELECT
            v.id_videojuego,
            v.id_externa,
            v.titulo,
            v.sinopsis,
            v.fecha_lanzamiento,
            v.url_caratula,
            v.desarrollador,
            v.editor,
            v.promedio_puntaje_usuarios::float8 AS promedio_puntaje_usuarios,
            v.promedio_puntaje_criticos::float8 AS promedio_puntaje_criticos,
            v.cantidad_opiniones_usuarios,
            v.cantidad_opiniones_criticos
        FROM videojuegos v
        WHERE v.id_videojuego IN (
            SELECT DISTINCT v2.id_videojuego
            FROM videojuegos v2
            LEFT JOIN videojuegos_generos vg ON vg.id_videojuego = v2.id_videojuego
            LEFT JOIN videojuegos_plataformas vp ON vp.id_videojuego = v2.id_videojuego
            WHERE ($1::text IS NULL OR LOWER(v2.titulo) LIKE $1)
              AND ($2::int IS NULL OR vg.id_genero = $2)
              AND ($3::int IS NULL OR vp.id_plataforma = $3)
              AND ($4::date IS NULL OR v2.fecha_lanzamiento >= $4)
              AND ($5::date IS NULL OR v2.fecha_lanzamiento <= $5)
              AND ($6::float8 IS NULL OR {puntaje_col} >= $6)
        )
        ORDER BY {sort_col} {order}
        LIMIT $7 OFFSET $8
        "#
    );

    let rows = sqlx::query_as::<_, VideojuegoRow>(&list_sql)
        .bind(&q_pattern)
        .bind(query.genero)
        .bind(query.plataforma)
        .bind(query.fecha_desde)
        .bind(query.fecha_hasta)
        .bind(puntaje_min)
        .bind(pagination.limit)
        .bind(pagination.offset())
        .fetch_all(pool)
        .await?;

    let mut data = Vec::with_capacity(rows.len());
    for row in rows {
        let (generos, plataformas) = cargar_relaciones(pool, row.id_videojuego).await?;
        data.push(row.into_response(generos, plataformas));
    }

    Ok(PaginatedResponse {
        data,
        page: pagination.page,
        limit: pagination.limit,
        total,
    })
}

pub async fn obtener(pool: &PgPool, id: i32) -> Result<VideojuegoResponse, VideojuegoError> {
    let sql = format!("{VIDEOJUEGO_SELECT} WHERE id_videojuego = $1");
    let row = sqlx::query_as::<_, VideojuegoRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(VideojuegoError::NotFound)?;

    let (generos, plataformas) = cargar_relaciones(pool, id).await?;
    Ok(row.into_response(generos, plataformas))
}

pub async fn obtener_por_id_externa(
    pool: &PgPool,
    id_externa: &str,
) -> Result<Option<VideojuegoResponse>, VideojuegoError> {
    let sql = format!("{VIDEOJUEGO_SELECT} WHERE id_externa = $1");
    let row = sqlx::query_as::<_, VideojuegoRow>(&sql)
        .bind(id_externa)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => {
            let (generos, plataformas) = cargar_relaciones(pool, r.id_videojuego).await?;
            Ok(Some(r.into_response(generos, plataformas)))
        }
        None => Ok(None),
    }
}

pub async fn resumen_puntajes(
    pool: &PgPool,
    id: i32,
) -> Result<VideojuegoResumenPuntajes, VideojuegoError> {
    let row = sqlx::query_as::<_, VideojuegoResumenPuntajes>(
        r#"
        SELECT
            id_videojuego,
            promedio_puntaje_usuarios::float8 AS promedio_puntaje_usuarios,
            promedio_puntaje_criticos::float8 AS promedio_puntaje_criticos,
            cantidad_opiniones_usuarios,
            cantidad_opiniones_criticos
        FROM videojuegos
        WHERE id_videojuego = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(VideojuegoError::NotFound)?;

    Ok(row)
}

pub async fn crear(
    pool: &PgPool,
    body: &CreateVideojuegoRequest,
) -> Result<VideojuegoResponse, VideojuegoError> {
    if body.titulo.trim().is_empty() {
        return Err(VideojuegoError::InvalidRequest(
            "el título es obligatorio".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO videojuegos (
            id_externa, titulo, sinopsis, fecha_lanzamiento,
            url_caratula, desarrollador, editor
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id_videojuego
        "#,
    )
    .bind(&body.id_externa)
    .bind(body.titulo.trim())
    .bind(&body.sinopsis)
    .bind(body.fecha_lanzamiento)
    .bind(&body.url_caratula)
    .bind(&body.desarrollador)
    .bind(&body.editor)
    .fetch_one(&mut *tx)
    .await
    .map_err(conflicto_id_externa)?;

    vincular_relaciones(&mut tx, id, body.generos.as_deref(), body.plataformas.as_deref()).await?;

    tx.commit().await?;
    obtener(pool, id).await
}

pub async fn actualizar(
    pool: &PgPool,
    id: i32,
    body: &UpdateVideojuegoRequest,
) -> Result<VideojuegoResponse, VideojuegoError> {
    let existe: Option<i32> = sqlx::query_scalar(
        "SELECT id_videojuego FROM videojuegos WHERE id_videojuego = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    if existe.is_none() {
        return Err(VideojuegoError::NotFound);
    }

    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        UPDATE videojuegos
        SET
            id_externa = COALESCE($1, id_externa),
            titulo = COALESCE($2, titulo),
            sinopsis = COALESCE($3, sinopsis),
            fecha_lanzamiento = COALESCE($4, fecha_lanzamiento),
            url_caratula = COALESCE($5, url_caratula),
            desarrollador = COALESCE($6, desarrollador),
            editor = COALESCE($7, editor)
        WHERE id_videojuego = $8
        "#,
    )
    .bind(&body.id_externa)
    .bind(body.titulo.as_deref().map(str::trim))
    .bind(&body.sinopsis)
    .bind(body.fecha_lanzamiento)
    .bind(&body.url_caratula)
    .bind(&body.desarrollador)
    .bind(&body.editor)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(conflicto_id_externa)?;

    if body.generos.is_some() {
        sqlx::query("DELETE FROM videojuegos_generos WHERE id_videojuego = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    if body.plataformas.is_some() {
        sqlx::query("DELETE FROM videojuegos_plataformas WHERE id_videojuego = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    vincular_relaciones(&mut tx, id, body.generos.as_deref(), body.plataformas.as_deref()).await?;

    tx.commit().await?;
    obtener(pool, id).await
}

pub async fn eliminar(pool: &PgPool, id: i32) -> Result<(), VideojuegoError> {
    let result = sqlx::query("DELETE FROM videojuegos WHERE id_videojuego = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(VideojuegoError::NotFound);
    }

    Ok(())
}

async fn vincular_relaciones(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    id: i32,
    generos: Option<&[i32]>,
    plataformas: Option<&[i32]>,
) -> Result<(), VideojuegoError> {
    if let Some(generos) = generos {
        for id_genero in generos {
            sqlx::query(
                r#"
                INSERT INTO videojuegos_generos (id_videojuego, id_genero)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(id)
            .bind(id_genero)
            .execute(&mut **tx)
            .await?;
        }
    }

    if let Some(plataformas) = plataformas {
        for id_plataforma in plataformas {
            sqlx::query(
                r#"
                INSERT INTO videojuegos_plataformas (id_videojuego, id_plataforma)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(id)
            .bind(id_plataforma)
            .execute(&mut **tx)
            .await?;
        }
    }

    Ok(())
}

async fn cargar_relaciones(
    pool: &PgPool,
    id_videojuego: i32,
) -> Result<(Vec<Genero>, Vec<Plataforma>), VideojuegoError> {
    let generos = sqlx::query_as::<_, Genero>(
        r#"
        SELECT g.id_genero, g.nombre
        FROM generos g
        INNER JOIN videojuegos_generos vg ON vg.id_genero = g.id_genero
        WHERE vg.id_videojuego = $1
        ORDER BY LOWER(g.nombre)
        "#,
    )
    .bind(id_videojuego)
    .fetch_all(pool)
    .await?;

    let plataformas = sqlx::query_as::<_, Plataforma>(
        r#"
        SELECT p.id_plataforma, p.nombre
        FROM plataformas p
        INNER JOIN videojuegos_plataformas vp ON vp.id_plataforma = p.id_plataforma
        WHERE vp.id_videojuego = $1
        ORDER BY LOWER(p.nombre)
        "#,
    )
    .bind(id_videojuego)
    .fetch_all(pool)
    .await?;

    Ok((generos, plataformas))
}
