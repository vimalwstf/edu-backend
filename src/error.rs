use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::collections::HashMap;
use validator::ValidationErrors;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Email already taken")]
    EmailTaken,

    #[error("Not found")]
    NotFound,

    #[error("Forbidden")]
    Forbidden,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request")]
    BadRequest(String),

    #[error(transparent)]
    Validation(#[from] ValidationErrors),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error("Password hashing failed")]
    HashError,

    #[error("Internal server error")]
    Internal(#[source] anyhow::Error),
}

// Private helper for consistent JSON error bodies
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    fields: HashMap<String, Vec<String>>,
}

impl AppError {
    fn simple(error: impl Into<String>) -> ErrorResponse {
        ErrorResponse {
            error: error.into(),
            message: None,
            fields: HashMap::new(),
        }
    }

    fn with_message(error: impl Into<String>, message: impl Into<String>) -> ErrorResponse {
        ErrorResponse {
            error: error.into(),
            message: Some(message.into()),
            fields: HashMap::new(),
        }
    }

    fn validation_fields(errors: &ValidationErrors) -> HashMap<String, Vec<String>> {
        errors
            .field_errors()
            .iter()
            .map(|(field, errs)| {
                (
                    field.to_string(),
                    errs.iter()
                        .map(|e| {
                            e.message
                                .clone()
                                .map(|m| m.to_string())
                                .unwrap_or_else(|| "Invalid value".to_string())
                        })
                        .collect(),
                )
            })
            .collect()
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidCredentials | AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::EmailTaken => StatusCode::CONFLICT,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::BadRequest(_) | AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Sqlx(_) | AppError::HashError | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn payload(&self) -> ErrorResponse {
        match self {
            AppError::InvalidCredentials => Self::simple("Invalid email or password"),
            AppError::EmailTaken => Self::simple("Email already taken"),
            AppError::NotFound => Self::simple("Not found"),
            AppError::Forbidden => Self::simple("Forbidden"),
            AppError::Unauthorized => Self::simple("Unauthorized"),
            AppError::BadRequest(msg) => Self::with_message("Bad request", msg),
            AppError::Validation(errors) => ErrorResponse {
                error: "Validation failed".to_string(),
                message: None,
                fields: Self::validation_fields(errors),
            },
            AppError::Sqlx(_) | AppError::HashError | AppError::Internal(_) => {
                // Log internally but don't expose details
                // tracing::error!("Internal error: {:?}", self);
                println!("Internal error: {:?}", self);

                Self::simple("An unexpected error occurred")
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let payload = self.payload();
        (status, Json(json!(payload))).into_response()
    }
}

// // Convenience constructors
// impl AppError {
//     pub fn bad_request(msg: impl Into<String>) -> Self {
//         AppError::BadRequest(msg.into())
//     }
// }

// impl From<anyhow::Error> for AppError {
//     fn from(err: anyhow::Error) -> Self {
//         AppError::Internal(err)
//     }
// }
