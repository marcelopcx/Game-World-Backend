use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::catalogo::{Genero, Plataforma};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct VideojuegoRow {
    pub id_videojuego: i32,
    pub id_externa: Option<String>,
    pub titulo: String,
    pub sinopsis: Option<String>,
    pub fecha_lanzamiento: Option<NaiveDate>,
    pub url_caratula: Option<String>,
    pub desarrollador: Option<String>,
    pub editor: Option<String>,
    pub promedio_puntaje_usuarios: Option<f64>,
    pub promedio_puntaje_criticos: Option<f64>,
    pub cantidad_opiniones_usuarios: i32,
    pub cantidad_opiniones_criticos: i32,
}

#[derive(Debug, Serialize)]
pub struct VideojuegoResponse {
    pub id_videojuego: i32,
    pub id_externa: Option<String>,
    pub titulo: String,
    pub sinopsis: Option<String>,
    pub fecha_lanzamiento: Option<NaiveDate>,
    pub url_caratula: Option<String>,
    pub desarrollador: Option<String>,
    pub editor: Option<String>,
    pub promedio_puntaje_usuarios: Option<f64>,
    pub promedio_puntaje_criticos: Option<f64>,
    pub cantidad_opiniones_usuarios: i32,
    pub cantidad_opiniones_criticos: i32,
    pub generos: Vec<Genero>,
    pub plataformas: Vec<Plataforma>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct VideojuegoResumenPuntajes {
    pub id_videojuego: i32,
    pub promedio_puntaje_usuarios: Option<f64>,
    pub promedio_puntaje_criticos: Option<f64>,
    pub cantidad_opiniones_usuarios: i32,
    pub cantidad_opiniones_criticos: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateVideojuegoRequest {
    pub id_externa: Option<String>,
    pub titulo: String,
    pub sinopsis: Option<String>,
    pub fecha_lanzamiento: Option<NaiveDate>,
    pub url_caratula: Option<String>,
    pub desarrollador: Option<String>,
    pub editor: Option<String>,
    pub generos: Option<Vec<i32>>,
    pub plataformas: Option<Vec<i32>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVideojuegoRequest {
    pub id_externa: Option<String>,
    pub titulo: Option<String>,
    pub sinopsis: Option<String>,
    pub fecha_lanzamiento: Option<NaiveDate>,
    pub url_caratula: Option<String>,
    pub desarrollador: Option<String>,
    pub editor: Option<String>,
    pub generos: Option<Vec<i32>>,
    pub plataformas: Option<Vec<i32>>,
}

#[derive(Debug, Deserialize)]
pub struct VideojuegoListQuery {
    pub q: Option<String>,
    pub genero: Option<i32>,
    pub plataforma: Option<i32>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    pub puntaje_min: Option<f64>,
    pub tipo_puntaje: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl VideojuegoRow {
    pub fn into_response(self, generos: Vec<Genero>, plataformas: Vec<Plataforma>) -> VideojuegoResponse {
        VideojuegoResponse {
            id_videojuego: self.id_videojuego,
            id_externa: self.id_externa,
            titulo: self.titulo,
            sinopsis: self.sinopsis,
            fecha_lanzamiento: self.fecha_lanzamiento,
            url_caratula: self.url_caratula,
            desarrollador: self.desarrollador,
            editor: self.editor,
            promedio_puntaje_usuarios: self.promedio_puntaje_usuarios,
            promedio_puntaje_criticos: self.promedio_puntaje_criticos,
            cantidad_opiniones_usuarios: self.cantidad_opiniones_usuarios,
            cantidad_opiniones_criticos: self.cantidad_opiniones_criticos,
            generos,
            plataformas,
        }
    }
}
