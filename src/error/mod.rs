use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;

use crate::services::auth::AuthError;
use crate::services::catalogo::CatalogoError;
use crate::services::cloudinary::CloudinaryError;
use crate::services::igdb::IgdbError;
use crate::services::opinion::OpinionError;
use crate::services::videojuego::VideojuegoError;

#[derive(Debug)]
pub enum ApiError {
    LoginIncorrecto,
    NoAutorizado,
    Prohibido,
    NoEncontrado,
    UsuarioYaExiste,
    SolicitudInvalida(String),
    ErrorDelServidor(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mensaje())
    }
}

impl ApiError {
    fn mensaje(&self) -> String {
        match self {
            ApiError::LoginIncorrecto => "credenciales inválidas".into(),
            ApiError::NoAutorizado => "no autorizado".into(),
            ApiError::Prohibido => "no tienes permiso para esta acción".into(),
            ApiError::NoEncontrado => "recurso no encontrado".into(),
            ApiError::UsuarioYaExiste => "el usuario ya existe".into(),
            ApiError::SolicitudInvalida(detalle) => detalle.clone(),
            ApiError::ErrorDelServidor(detalle) => detalle.clone(),
        }
    }

    fn codigo_http(&self) -> StatusCode {
        match self {
            ApiError::LoginIncorrecto => StatusCode::UNAUTHORIZED,
            ApiError::NoAutorizado => StatusCode::UNAUTHORIZED,
            ApiError::Prohibido => StatusCode::FORBIDDEN,
            ApiError::NoEncontrado => StatusCode::NOT_FOUND,
            ApiError::UsuarioYaExiste => StatusCode::CONFLICT,
            ApiError::SolicitudInvalida(_) => StatusCode::BAD_REQUEST,
            ApiError::ErrorDelServidor(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<CloudinaryError> for ApiError {
    fn from(err: CloudinaryError) -> Self {
        ApiError::ErrorDelServidor(err.to_string())
    }
}

impl From<AuthError> for ApiError {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => ApiError::LoginIncorrecto,
            AuthError::Unauthorized => ApiError::NoAutorizado,
            AuthError::NotFound => ApiError::NoEncontrado,
            AuthError::Conflict => ApiError::UsuarioYaExiste,
            AuthError::Forbidden => ApiError::Prohibido,
            AuthError::Database(e) => ApiError::ErrorDelServidor(e.to_string()),
            AuthError::PasswordHash(e) => ApiError::ErrorDelServidor(e.to_string()),
            AuthError::Token(e) => ApiError::ErrorDelServidor(e.to_string()),
        }
    }
}

impl From<CatalogoError> for ApiError {
    fn from(err: CatalogoError) -> Self {
        match err {
            CatalogoError::Database(e) => ApiError::ErrorDelServidor(e.to_string()),
        }
    }
}

impl From<VideojuegoError> for ApiError {
    fn from(err: VideojuegoError) -> Self {
        match err {
            VideojuegoError::InvalidRequest(msg) => ApiError::SolicitudInvalida(msg),
            VideojuegoError::NotFound => ApiError::NoEncontrado,
            VideojuegoError::Conflict => {
                ApiError::SolicitudInvalida("id externa duplicada".into())
            }
            VideojuegoError::Database(e) => ApiError::ErrorDelServidor(e.to_string()),
        }
    }
}

impl From<OpinionError> for ApiError {
    fn from(err: OpinionError) -> Self {
        match err {
            OpinionError::InvalidRequest(msg) => ApiError::SolicitudInvalida(msg),
            OpinionError::Forbidden => ApiError::Prohibido,
            OpinionError::NotFound => ApiError::NoEncontrado,
            OpinionError::Conflict => ApiError::SolicitudInvalida(
                "ya existe una opinión para este videojuego".into(),
            ),
            OpinionError::Database(e) => ApiError::ErrorDelServidor(e.to_string()),
        }
    }
}

impl From<IgdbError> for ApiError {
    fn from(err: IgdbError) -> Self {
        match err {
            IgdbError::NotConfigured => {
                ApiError::ErrorDelServidor("credenciales IGDB no configuradas".into())
            }
            IgdbError::InvalidRequest(msg) => ApiError::SolicitudInvalida(msg),
            IgdbError::NotFound => ApiError::NoEncontrado,
            IgdbError::Http(msg) => ApiError::ErrorDelServidor(msg),
            IgdbError::Database(e) => ApiError::ErrorDelServidor(e.to_string()),
            IgdbError::Videojuego(e) => e.into(),
            IgdbError::Catalogo(e) => e.into(),
        }
    }
}

#[derive(Serialize)]
struct CuerpoError {
    error: String,
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.codigo_http()
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.codigo_http()).json(CuerpoError {
            error: self.mensaje(),
        })
    }
}
