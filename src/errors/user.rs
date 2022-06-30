use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use thiserror::Error;

pub type UserServiceResult = Result<HttpResponse, UserServiceError>;

#[derive(Error, Debug)]
pub enum UserServiceError {
    #[error("No user found for given ID: {0}")]
    InvalidId(String),
    #[error("Access denied")]
    Forbidden,
    #[error("Unknown internal server error")]
    UnknownInternal,
}

impl ResponseError for UserServiceError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::InvalidId(_) => StatusCode::NOT_FOUND,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::UnknownInternal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        HttpResponse::build(status_code).body(self.to_string())
    }
}
