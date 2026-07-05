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
    pub generos: Option<String>,
    pub plataforma: Option<i32>,
    pub plataformas: Option<String>,
    pub fecha_desde: Option<NaiveDate>,
    pub fecha_hasta: Option<NaiveDate>,
    pub puntaje_min: Option<f64>,
    pub tipo_puntaje: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl VideojuegoListQuery {
    fn parse_id_list(value: &Option<String>) -> Option<Vec<i32>> {
        let raw = value.as_ref()?.trim();
        if raw.is_empty() {
            return None;
        }

        let ids: Vec<i32> = raw
            .split(',')
            .filter_map(|part| part.trim().parse::<i32>().ok())
            .collect();

        if ids.is_empty() {
            None
        } else {
            Some(ids)
        }
    }

    fn expand_genero_ids(ids: Vec<i32>) -> Vec<i32> {
        use std::collections::HashSet;

        const ALIAS: &[(i32, &[i32])] = &[
            (2, &[16]),   // Aventura -> Adventure
            (3, &[20]),   // RPG -> Role-playing (RPG)
            (4, &[124]),  // Estrategia -> Strategy
            (5, &[96]),   // Simulación -> Simulator
            (6, &[267]),  // Deportes -> Sport
            (7, &[18]),   // Carreras -> Racing
            (9, &[22]),   // Plataformas -> Platform
            (11, &[29]),  // Lucha -> Fighting
        ];

        let mut expanded = HashSet::new();

        for id in ids {
            expanded.insert(id);

            for (legacy_id, aliases) in ALIAS {
                if id == *legacy_id {
                    for alias in *aliases {
                        expanded.insert(*alias);
                    }
                }
            }
        }

        let mut result: Vec<i32> = expanded.into_iter().collect();
        result.sort_unstable();
        result
    }

    pub fn generos_filtro(&self) -> Option<Vec<i32>> {
        if let Some(ids) = Self::parse_id_list(&self.generos) {
            return Some(Self::expand_genero_ids(ids));
        }

        self.genero
            .map(|id| Self::expand_genero_ids(vec![id]))
    }

    pub fn plataformas_filtro(&self) -> Option<Vec<i32>> {
        if let Some(ids) = Self::parse_id_list(&self.plataformas) {
            return Some(ids);
        }

        self.plataforma.map(|id| vec![id])
    }
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
