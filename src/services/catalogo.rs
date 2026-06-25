use sqlx::PgPool;

use crate::models::catalogo::{Genero, Plataforma};

#[derive(Debug, thiserror::Error)]
pub enum CatalogoError {
    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),
}

pub async fn listar_generos(pool: &PgPool) -> Result<Vec<Genero>, CatalogoError> {
    let generos = sqlx::query_as::<_, Genero>(
        "SELECT id_genero, nombre FROM generos ORDER BY LOWER(nombre)",
    )
    .fetch_all(pool)
    .await?;

    Ok(generos)
}

pub async fn listar_plataformas(pool: &PgPool) -> Result<Vec<Plataforma>, CatalogoError> {
    let plataformas = sqlx::query_as::<_, Plataforma>(
        "SELECT id_plataforma, nombre FROM plataformas ORDER BY LOWER(nombre)",
    )
    .fetch_all(pool)
    .await?;

    Ok(plataformas)
}

pub async fn obtener_o_crear_genero(pool: &PgPool, nombre: &str) -> Result<i32, CatalogoError> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO generos (nombre)
        VALUES ($1)
        ON CONFLICT (nombre) DO UPDATE SET nombre = EXCLUDED.nombre
        RETURNING id_genero
        "#,
    )
    .bind(nombre.trim())
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn obtener_o_crear_plataforma(pool: &PgPool, nombre: &str) -> Result<i32, CatalogoError> {
    let id: i32 = sqlx::query_scalar(
        r#"
        INSERT INTO plataformas (nombre)
        VALUES ($1)
        ON CONFLICT (nombre) DO UPDATE SET nombre = EXCLUDED.nombre
        RETURNING id_plataforma
        "#,
    )
    .bind(nombre.trim())
    .fetch_one(pool)
    .await?;

    Ok(id)
}
