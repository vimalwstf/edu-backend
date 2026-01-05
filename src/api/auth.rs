use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::json;

use crate::{
    dto::user::{LoginPayload, RegisterPayload},
    error::AppError,
    services::user::UserService,
    state::AppState,
};
use validator::Validate;

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let (user, token) = UserService::login(&state.pool, payload).await?;

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Login successful",
            "user": user,
            "token": token
        })),
    ))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate().map_err(AppError::Validation)?;

    let user = UserService::register(&state.pool, payload).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Registration successful",
            "user": user,
            "token": "token"
        })),
    ))
}
