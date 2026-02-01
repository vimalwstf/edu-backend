use axum::{
    Extension, Json, Router,
    extract::State,
    middleware,
    routing::{get, post},
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::notification::CreateNotificationPayload,
    entities::user::UserRole,
    error::AppError,
    middlewares::auth::auth_middleware,
    services::{notification::NotificationService, user::UserService},
    state::AppState,
};

pub fn notification_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create_notification_handler)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route("/", get(list_notifications_handler))
}

async fn create_notification_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateNotificationPayload>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;

    // Admin check
    let user = UserService::get_user_by_id(&state.pool, user_id).await?;
    if user.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let notification = NotificationService::create(&state.pool, payload).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "data": notification
    })))
}

async fn list_notifications_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let notifications = NotificationService::list(&state.pool).await?;

    Ok(Json(serde_json::json!({
        "status": "success",
        "data": notifications
    })))
}
