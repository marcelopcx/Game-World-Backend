use sqlx::PgPool;

use crate::models::catalogo::{Genero, Plataforma};

#[derive(Debug, thiserror::Error)]
pub enum CatalogoError {
    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),
}

fn nombre_genero_es(nombre: &str) -> String {
    match nombre {
        "Racing" => "Carreras".to_string(),
        "Sport" => "Deportes".to_string(),
        "Platform" => "Plataformas".to_string(),
        "Adventure" => "Aventura".to_string(),
        "Role-playing (RPG)" => "RPG".to_string(),
        "Strategy" => "Estrategia".to_string(),
        "Simulator" => "Simulación".to_string(),
        "Fighting" => "Lucha".to_string(),
        "Shooter" => "Disparos".to_string(),
        "Puzzle" => "Puzzle".to_string(),
        "Indie" => "Indie".to_string(),
        other => other.to_string(),
    }
}

pub async fn listar_generos(pool: &PgPool) -> Result<Vec<Genero>, CatalogoError> {
    let mut generos = sqlx::query_as::<_, Genero>(
        r#"
        SELECT g.id_genero, g.nombre
        FROM generos g
        WHERE EXISTS (
            SELECT 1
            FROM videojuegos_generos vg
            WHERE vg.id_genero = g.id_genero
        )
        ORDER BY LOWER(g.nombre)
        "#,
    )
    .fetch_all(pool)
    .await?;

    for genero in &mut generos {
        genero.nombre = nombre_genero_es(&genero.nombre);
    }

    Ok(generos)
}

pub async fn listar_plataformas(pool: &PgPool) -> Result<Vec<Plataforma>, CatalogoError> {
    let plataformas = sqlx::query_as::<_, Plataforma>(
        r#"
        SELECT p.id_plataforma, p.nombre
        FROM plataformas p
        WHERE EXISTS (
            SELECT 1
            FROM videojuegos_plataformas vp
            WHERE vp.id_plataforma = p.id_plataforma
        )
        ORDER BY LOWER(p.nombre)
        "#,
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
