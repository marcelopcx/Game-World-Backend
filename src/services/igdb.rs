use chrono::{NaiveDate, TimeZone, Utc};
use serde::Deserialize;
use serde::Serialize;
use sqlx::PgPool;

use crate::config::IgdbConfig;
use crate::models::videojuego::{CreateVideojuegoRequest, VideojuegoResponse};
use crate::services::catalogo;
use crate::services::videojuego;

#[derive(Debug, thiserror::Error)]
pub enum IgdbError {
    #[error("credenciales IGDB no configuradas")]
    NotConfigured,

    #[error("solicitud inválida: {0}")]
    InvalidRequest(String),

    #[error("recurso no encontrado en IGDB")]
    NotFound,

    #[error("error de red: {0}")]
    Http(String),

    #[error("error de base de datos")]
    Database(#[from] sqlx::Error),

    #[error("error de videojuego")]
    Videojuego(#[from] videojuego::VideojuegoError),

    #[error("error de catálogo")]
    Catalogo(#[from] catalogo::CatalogoError),
}

#[derive(Debug, Serialize)]
pub struct IgdbJuegoResponse {
    pub id_externa: String,
    pub titulo: String,
    pub sinopsis: Option<String>,
    pub fecha_lanzamiento: Option<NaiveDate>,
    pub url_caratula: Option<String>,
    pub desarrollador: Option<String>,
    pub editor: Option<String>,
    pub generos: Vec<String>,
    pub plataformas: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct IgdbBuscarResponse {
    pub data: Vec<IgdbJuegoResponse>,
}

#[derive(Debug, Serialize)]
pub struct SincronizarResponse {
    #[serde(flatten)]
    pub videojuego: VideojuegoResponse,
    pub sincronizado: bool,
}

#[derive(Debug, Deserialize)]
struct TwitchTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct IgdbGame {
    id: Option<i64>,
    name: Option<String>,
    summary: Option<String>,
    first_release_date: Option<i64>,
    cover: Option<IgdbCover>,
    involved_companies: Option<Vec<IgdbInvolvedCompany>>,
    genres: Option<Vec<IgdbNamed>>,
    platforms: Option<Vec<IgdbNamed>>,
}

#[derive(Debug, Deserialize)]
struct IgdbCover {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IgdbNamed {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IgdbInvolvedCompany {
    developer: Option<bool>,
    publisher: Option<bool>,
    company: Option<IgdbNamed>,
}

async fn obtener_token(config: &IgdbConfig) -> Result<String, IgdbError> {
    if config.client_id.is_empty() || config.client_secret.is_empty() {
        return Err(IgdbError::NotConfigured);
    }

    let url = format!(
        "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials",
        config.client_id, config.client_secret
    );

    let resp = reqwest::Client::new()
        .post(&url)
        .send()
        .await
        .map_err(|e| IgdbError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(IgdbError::Http(format!(
            "token IGDB falló: {}",
            resp.status()
        )));
    }

    let data: TwitchTokenResponse = resp
        .json()
        .await
        .map_err(|e| IgdbError::Http(e.to_string()))?;

    Ok(data.access_token)
}

async fn consultar_igdb(
    config: &IgdbConfig,
    endpoint: &str,
    body: &str,
) -> Result<Vec<IgdbGame>, IgdbError> {
    let token = obtener_token(config).await?;

    let resp = reqwest::Client::new()
        .post(format!("https://api.igdb.com/v4/{endpoint}"))
        .header("Client-ID", &config.client_id)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| IgdbError::Http(e.to_string()))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(IgdbError::Http(format!("IGDB {status}: {text}")));
    }

    resp.json()
        .await
        .map_err(|e| IgdbError::Http(e.to_string()))
}

fn normalizar_url(url: Option<String>) -> Option<String> {
    url.map(|u| {
        if u.starts_with("//") {
            format!("https:{u}")
        } else {
            u
        }
    })
}

fn timestamp_a_fecha(ts: Option<i64>) -> Option<NaiveDate> {
    ts.and_then(|value| Utc.timestamp_opt(value, 0).single())
        .map(|dt| dt.date_naive())
}

fn mapear_juego(game: IgdbGame) -> Option<IgdbJuegoResponse> {
    let id = game.id?;
    let titulo = game.name?;

    let desarrollador = game
        .involved_companies
        .as_ref()
        .and_then(|items| {
            items
                .iter()
                .find(|c| c.developer.unwrap_or(false))
                .and_then(|c| c.company.as_ref())
                .and_then(|c| c.name.clone())
        });

    let editor = game
        .involved_companies
        .as_ref()
        .and_then(|items| {
            items
                .iter()
                .find(|c| c.publisher.unwrap_or(false))
                .and_then(|c| c.company.as_ref())
                .and_then(|c| c.name.clone())
        });

    let generos = game
        .genres
        .unwrap_or_default()
        .into_iter()
        .filter_map(|g| g.name)
        .collect();

    let plataformas = game
        .platforms
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| p.name)
        .collect();

    Some(IgdbJuegoResponse {
        id_externa: id.to_string(),
        titulo,
        sinopsis: game.summary,
        fecha_lanzamiento: timestamp_a_fecha(game.first_release_date),
        url_caratula: normalizar_url(game.cover.and_then(|c| c.url)),
        desarrollador,
        editor,
        generos,
        plataformas,
    })
}

const CAMPOS_JUEGO: &str = "name, summary, cover.url, first_release_date, involved_companies.company.name, involved_companies.developer, involved_companies.publisher, genres.name, platforms.name";
const LOTE_IGDB: i64 = 500;

fn escapar_termino_busqueda(termino: &str) -> String {
    termino.replace('"', r#"\""#)
}

/// Guarda un juego de IGDB en la BD local (idempotente por id_externa).
pub async fn persistir_juego(
    pool: &PgPool,
    igdb: &IgdbJuegoResponse,
) -> Result<VideojuegoResponse, IgdbError> {
    if let Some(existente) = videojuego::obtener_por_id_externa(pool, &igdb.id_externa).await? {
        return Ok(existente);
    }

    let mut genero_ids = Vec::new();
    for nombre in &igdb.generos {
        genero_ids.push(catalogo::obtener_o_crear_genero(pool, nombre).await?);
    }

    let mut plataforma_ids = Vec::new();
    for nombre in &igdb.plataformas {
        plataforma_ids.push(catalogo::obtener_o_crear_plataforma(pool, nombre).await?);
    }

    let request = CreateVideojuegoRequest {
        id_externa: Some(igdb.id_externa.clone()),
        titulo: igdb.titulo.clone(),
        sinopsis: igdb.sinopsis.clone(),
        fecha_lanzamiento: igdb.fecha_lanzamiento,
        url_caratula: igdb.url_caratula.clone(),
        desarrollador: igdb.desarrollador.clone(),
        editor: igdb.editor.clone(),
        generos: Some(genero_ids),
        plataformas: Some(plataforma_ids),
    };

    Ok(videojuego::crear(pool, &request).await?)
}

async fn listar_populares(
    config: &IgdbConfig,
    limit: i64,
    offset: i64,
) -> Result<Vec<IgdbJuegoResponse>, IgdbError> {
    let body = format!(
        "fields {CAMPOS_JUEGO}; where version_parent = null & cover != null; sort total_rating_count desc; limit {limit}; offset {offset};"
    );
    let games = consultar_igdb(config, "games", &body).await?;
    Ok(games.into_iter().filter_map(mapear_juego).collect())
}

/// Completa el catálogo local hasta `min_juegos` usando IGDB (al arrancar el servidor).
pub async fn poblar_catalogo(
    pool: &PgPool,
    config: &IgdbConfig,
    min_juegos: i64,
) -> Result<(), IgdbError> {
    let actual = videojuego::contar(pool).await?;
    if actual >= min_juegos {
        println!("Catálogo de videojuegos OK ({actual} juegos en BD).");
        return Ok(());
    }

    if config.client_id.is_empty() || config.client_secret.is_empty() {
        eprintln!(
            "ADVERTENCIA: IGDB no configurado; hay {actual} juegos (mínimo {min_juegos})."
        );
        return Ok(());
    }

    println!(
        "Poblando catálogo desde IGDB ({actual} → {min_juegos} juegos)..."
    );

    let mut offset = 0i64;
    loop {
        let actual = videojuego::contar(pool).await?;
        if actual >= min_juegos {
            break;
        }

        let faltan = min_juegos - actual;
        let limite = faltan.min(LOTE_IGDB);
        let juegos = listar_populares(config, limite, offset).await?;
        if juegos.is_empty() {
            eprintln!("IGDB no devolvió más juegos (offset {offset}). Total en BD: {actual}.");
            break;
        }

        for juego in &juegos {
            persistir_juego(pool, juego).await?;
        }

        offset += juegos.len() as i64;
        actix_web::rt::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    let final_count = videojuego::contar(pool).await?;
    println!("Catálogo de videojuegos listo ({final_count} juegos en BD).");
    Ok(())
}

/// Busca en IGDB y persiste todos los resultados en la BD local.
pub async fn buscar_y_persistir(
    pool: &PgPool,
    config: &IgdbConfig,
    q: &str,
    limit: i64,
) -> Result<usize, IgdbError> {
    let respuesta = buscar(config, q, limit).await?;
    let mut guardados = 0usize;

    for juego in &respuesta.data {
        persistir_juego(pool, juego).await?;
        guardados += 1;
    }

    Ok(guardados)
}

pub async fn buscar(
    config: &IgdbConfig,
    q: &str,
    limit: i64,
) -> Result<IgdbBuscarResponse, IgdbError> {
    let termino = q.trim();
    if termino.is_empty() {
        return Err(IgdbError::InvalidRequest(
            "el parámetro q es obligatorio".into(),
        ));
    }

    let limite = limit.clamp(1, 50);
    let termino_esc = escapar_termino_busqueda(termino);
    let body = format!(
        r#"search "{termino_esc}"; fields {CAMPOS_JUEGO}; where version_parent = null; limit {limite};"#
    );

    let games = consultar_igdb(config, "games", &body).await?;
    let data = games.into_iter().filter_map(mapear_juego).collect();

    Ok(IgdbBuscarResponse { data })
}

pub async fn obtener_detalle(
    config: &IgdbConfig,
    id_externa: &str,
) -> Result<IgdbJuegoResponse, IgdbError> {
    let body = format!("fields {CAMPOS_JUEGO}; where id = {id_externa};");

    let games = consultar_igdb(config, "games", &body).await?;
    games
        .into_iter()
        .next()
        .and_then(mapear_juego)
        .ok_or(IgdbError::NotFound)
}

pub async fn sincronizar(
    pool: &PgPool,
    config: &IgdbConfig,
    id_externa: &str,
) -> Result<SincronizarResponse, IgdbError> {
    if let Some(existente) = videojuego::obtener_por_id_externa(pool, id_externa).await? {
        return Ok(SincronizarResponse {
            videojuego: existente,
            sincronizado: false,
        });
    }

    let igdb = obtener_detalle(config, id_externa).await?;
    let creado = persistir_juego(pool, &igdb).await?;

    Ok(SincronizarResponse {
        videojuego: creado,
        sincronizado: true,
    })
}
