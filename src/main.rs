use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use backend::config::AppConfig;
use backend::{db, routes, services};
use std::net::UdpSocket;

fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env();
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("No se pudo conectar a la base de datos");

    // El catálogo IGDB puede tardar minutos; no bloquear el arranque del servidor
    // (Render exige que /health responda antes del timeout del deploy).
    {
        let pool_bg = pool.clone();
        let igdb = config.igdb.clone();
        let min_juegos = config.catalogo_min_juegos;
        actix_web::rt::spawn(async move {
            if let Err(e) = services::igdb::poblar_catalogo(&pool_bg, &igdb, min_juegos).await {
                eprintln!("ADVERTENCIA: no se pudo completar el catálogo de videojuegos: {e}");
            }
        });
    }

    let host = config.host.clone();
    let port = config.port;

    let printed_host = if host == "0.0.0.0" {
        local_ip().unwrap_or_else(|| host.clone())
    } else {
        host.clone()
    };

    let server = HttpServer::new(move || {
        // Abierto para navegador, Ionic/Capacitor y cualquier origen móvil
        // (capacitor://localhost, https://localhost, etc.).
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?;

    println!("Servidor listo en http://{}:{}", printed_host, port);

    server.run().await
}
