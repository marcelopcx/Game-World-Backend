use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Genero {
    pub id_genero: i32,
    pub nombre: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Plataforma {
    pub id_plataforma: i32,
    pub nombre: String,
}
