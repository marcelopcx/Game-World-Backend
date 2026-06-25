//! Endpoints para subir imágenes a Cloudinary.
//!
//! El cliente envía `multipart/form-data` con un campo `file`.
//! El servidor reenvía los bytes a Cloudinary y devuelve la `secure_url`.
//! Para cambiar el avatar: subir aquí y luego persistir con `PATCH /auth/me` (`url_avatar`).
//! Al registrarse, el usuario recibe un avatar predeterminado hasta que lo actualice.

use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse};
use futures_util::StreamExt;
use serde::Serialize;

use crate::auth::AuthenticatedUser;
use crate::config::AppConfig;
use crate::error::ApiError;
use crate::services::cloudinary;

const MAX_BYTES: usize = 10 * 1024 * 1024;

#[derive(Serialize)]
struct ImagenSubidaResponse {
    secure_url: String,
}

async fn leer_archivo_multipart(payload: &mut Multipart) -> Result<(Vec<u8>, String), ApiError> {
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            ApiError::SolicitudInvalida(format!("multipart inválido: {e}"))
        })?;

        if field.name() != Some("file") {
            continue;
        }

        let filename = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .map(|name| name.to_string())
            .unwrap_or_else(|| "imagen.jpg".to_string());

        let mut bytes: Vec<u8> = Vec::new();
        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|e| {
                ApiError::ErrorDelServidor(format!("error leyendo archivo: {e}"))
            })?;

            if bytes.len() + data.len() > MAX_BYTES {
                return Err(ApiError::SolicitudInvalida(
                    "la imagen supera el tamaño máximo permitido (10 MB)".into(),
                ));
            }

            bytes.extend_from_slice(&data);
        }

        if bytes.is_empty() {
            return Err(ApiError::SolicitudInvalida(
                "el archivo está vacío".into(),
            ));
        }

        return Ok((bytes, filename));
    }

    Err(ApiError::SolicitudInvalida(
        "no se envió ningún archivo en el campo `file`".into(),
    ))
}

#[post("/auth/me/avatar")]
pub async fn subir_avatar_usuario(
    config: web::Data<AppConfig>,
    _user: AuthenticatedUser,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let (bytes, filename) = leer_archivo_multipart(&mut payload).await?;
    let folder = format!("{}/avatars", config.cloudinary.folder);
    let secure_url =
        cloudinary::subir_imagen(&config.cloudinary, bytes, filename, Some(&folder)).await?;

    Ok(HttpResponse::Ok().json(ImagenSubidaResponse { secure_url }))
}

#[post("/videojuegos/imagen")]
pub async fn subir_imagen_videojuego(
    config: web::Data<AppConfig>,
    _user: AuthenticatedUser,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    let (bytes, filename) = leer_archivo_multipart(&mut payload).await?;
    let folder = format!("{}/caratulas", config.cloudinary.folder);
    let secure_url =
        cloudinary::subir_imagen(&config.cloudinary, bytes, filename, Some(&folder)).await?;

    Ok(HttpResponse::Ok().json(ImagenSubidaResponse { secure_url }))
}
