use actix_web::{HttpResponse, ResponseError};
use cinematch_abi::domain::DomainError;
use cinematch_common::models::ErrorResponse;
use cinematch_db::DbError;
use utoipa::{IntoResponses, ToSchema};

#[derive(Debug, thiserror::Error, IntoResponses, ToSchema)]
pub enum ApiError {
    #[error("Bad Request: {0}")]
    #[response(status = 400, description = "Bad Request")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    #[response(status = 401, description = "Unauthorized")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    #[response(status = 403, description = "Forbidden")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    #[response(status = 404, description = "Not Found")]
    NotFound(String),

    #[error("Internal Server Error: {0}")]
    #[response(status = 500, description = "Internal Server Error")]
    InternalServerError(String),

    #[error("Conflict: {0}")]
    #[response(status = 409, description = "Conflict")]
    Conflict(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse::new(self.to_string());
        match self {
            ApiError::BadRequest(_) => HttpResponse::BadRequest().json(error_response),
            ApiError::Unauthorized(_) => HttpResponse::Unauthorized().json(error_response),
            ApiError::Forbidden(_) => HttpResponse::Forbidden().json(error_response),
            ApiError::NotFound(_) => HttpResponse::NotFound().json(error_response),
            ApiError::InternalServerError(_) => {
                HttpResponse::InternalServerError().json(error_response)
            }
            ApiError::Conflict(_) => HttpResponse::Conflict().json(error_response),
        }
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::BadRequest(s) => ApiError::BadRequest(s),
            DomainError::Unauthorized(s) => ApiError::Unauthorized(s),
            DomainError::Forbidden(s) => ApiError::Forbidden(s),
            DomainError::NotFound(s) => ApiError::NotFound(s),
            DomainError::Conflict(s) => ApiError::Conflict(s),
            DomainError::Internal(s) => ApiError::InternalServerError(s),
        }
    }
}

impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        DomainError::from(err).into()
    }
}

impl From<actix_web::Error> for ApiError {
    fn from(err: actix_web::Error) -> Self {
        let status = err.as_response_error().status_code();
        let message = err.to_string();
        match status.as_u16() {
            400 => ApiError::BadRequest(message),
            401 => ApiError::Unauthorized(message),
            403 => ApiError::Forbidden(message),
            404 => ApiError::NotFound(message),
            409 => ApiError::Conflict(message),
            _ => ApiError::InternalServerError(message),
        }
    }
}
