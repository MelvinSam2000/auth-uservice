use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use serde_json::json;
use thiserror::Error;

pub type UserServiceResult<T> = Result<Json<T>, UserServiceError>;

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("No user found for given ID: {0}")]
    NoUserForId(String),
    #[error("Invalid ID: {0}")]
    InvalidId(String),
    #[error("Username taken")]
    UsernameTaken,
    #[error("User fields invalid: {0}")]
    InvalidUserFields(#[from] validator::ValidationErrors),
    #[error("Access denied")]
    Forbidden,
    #[error("Unknown internal server error")]
    UnknownInternal,
}

impl ResponseError for UserServiceError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NoUserForId(_) => StatusCode::NOT_FOUND,
            Self::InvalidId(_) | Self::UsernameTaken | Self::InvalidUserFields(_) => {
                StatusCode::BAD_REQUEST
            }
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::UnknownInternal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        log::error!(
            "Sending error HTTP response: {} {}",
            status_code,
            self.to_string()
        );
        HttpResponse::build(status_code).json(json!({
            "error": self.to_string()
        }))
    }
}

pub fn log_err(any_err: impl Into<anyhow::Error>) -> anyhow::Error {
    let err = any_err.into();
    log::error!("Internal Error: {:?}", err);
    err
}
