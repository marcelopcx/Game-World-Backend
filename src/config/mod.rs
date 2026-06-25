use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub host: String,
    pub port: u16,
    pub cloudinary: CloudinaryConfig,
    pub igdb: IgdbConfig,
    /// Mínimo de videojuegos en BD al arrancar (se completan desde IGDB si faltan).
    pub catalogo_min_juegos: i64,
    /// URL del avatar asignado al registrar o crear usuarios sin imagen propia.
    pub default_avatar_url: String,
}

#[derive(Clone)]
pub struct IgdbConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Clone)]
pub struct CloudinaryConfig {
    pub cloud_name: String,
    pub upload_preset: String,
    pub folder: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL")
                .expect("falta DATABASE_URL en .env"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("falta JWT_SECRET en .env"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .expect("falta JWT_EXPIRATION_HOURS en .env")
                .parse::<i64>()
                .expect("JWT_EXPIRATION_HOURS debe ser un número (ej: 24)"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .expect("falta PORT en .env")
                .parse::<u16>()
                .expect("PORT debe ser un número (ej: 8080)"),
            cloudinary: CloudinaryConfig {
                cloud_name: env::var("CLOUDINARY_CLOUD_NAME")
                    .unwrap_or_else(|_| "mpc-uru".to_string()),
                upload_preset: env::var("CLOUDINARY_UPLOAD_PRESET")
                    .unwrap_or_else(|_| "n3n6sbhv".to_string()),
                folder: env::var("CLOUDINARY_FOLDER")
                    .unwrap_or_else(|_| "game-world".to_string()),
            },
            igdb: IgdbConfig {
                client_id: env::var("IGDB_CLIENT_ID").unwrap_or_default(),
                client_secret: env::var("IGDB_CLIENT_SECRET").unwrap_or_default(),
            },
            catalogo_min_juegos: env::var("CATALOGO_MIN_JUEGOS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse::<i64>()
                .unwrap_or(1000),
            default_avatar_url: env::var("DEFAULT_AVATAR_URL").unwrap_or_else(|_| {
                let cloud = env::var("CLOUDINARY_CLOUD_NAME")
                    .unwrap_or_else(|_| "mpc-uru".to_string());
                let folder = env::var("CLOUDINARY_FOLDER")
                    .unwrap_or_else(|_| "game-world".to_string());
                format!(
                    "https://res.cloudinary.com/{cloud}/image/upload/{folder}/avatars/default.png"
                )
            }),
        }
    }
}
