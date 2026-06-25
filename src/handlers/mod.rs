pub mod auth;
pub mod catalogo;
pub mod health;
pub mod igdb;
pub mod imagen;
pub mod videojuego;

pub use auth::{
    crear_usuario, delete_me, delete_usuario, get_me, get_me_opiniones, get_usuario, listar_usuarios,
    login, patch_me, patch_usuario, register,
};
pub use catalogo::{listar_generos, listar_plataformas};
pub use health::health_check;
pub use igdb::{buscar_igdb, detalle_igdb, sincronizar_igdb};
pub use imagen::{subir_avatar_usuario, subir_imagen_videojuego};
pub use videojuego::{
    actualizar_opinion, actualizar_videojuego, crear_opinion_videojuego, crear_videojuego,
    eliminar_opinion, eliminar_videojuego, listar_opiniones_videojuego, listar_videojuegos,
    obtener_opinion, obtener_videojuego, resumen_opiniones_videojuego,
};
