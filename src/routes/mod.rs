use actix_web::web;

use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::health_check)
        .service(handlers::register)
        .service(handlers::login)
        .service(handlers::get_me)
        .service(handlers::patch_me)
        .service(handlers::delete_me)
        .service(handlers::get_me_opiniones)
        .service(handlers::listar_usuarios)
        .service(handlers::crear_usuario)
        .service(handlers::get_usuario)
        .service(handlers::patch_usuario)
        .service(handlers::delete_usuario)
        .service(handlers::listar_generos)
        .service(handlers::listar_plataformas)
        .service(handlers::listar_videojuegos)
        .service(handlers::obtener_videojuego)
        .service(handlers::crear_videojuego)
        .service(handlers::actualizar_videojuego)
        .service(handlers::eliminar_videojuego)
        .service(handlers::listar_opiniones_videojuego)
        .service(handlers::resumen_opiniones_videojuego)
        .service(handlers::crear_opinion_videojuego)
        .service(handlers::obtener_opinion)
        .service(handlers::actualizar_opinion)
        .service(handlers::eliminar_opinion)
        .service(handlers::subir_avatar_usuario)
        .service(handlers::subir_imagen_videojuego)
        .service(handlers::buscar_igdb)
        .service(handlers::detalle_igdb)
        .service(handlers::sincronizar_igdb);
}
