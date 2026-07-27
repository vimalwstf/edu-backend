use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    routing::{get, post},
};
use serde_json::{Value, json};
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::quiz::{CreateQuestionPayload, CreateQuizPayload, SubmitQuizPayload},
    entities::user::UserRole,
    error::AppError,
    middlewares::auth::auth_middleware,
    services::{quiz::QuizService, user::UserService},
    state::AppState,
};

pub fn quiz_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_quizzes).post(create_quiz))
        .route("/attempts", get(list_attempts))
        .route("/attempts/:attempt_id/submit", post(submit_attempt))
        .route("/:quiz_id", get(get_quiz))
        .route("/:quiz_id/questions", post(add_question))
        .route("/:quiz_id/attempts", post(start_attempt))
        .route("/:quiz_id/analytics", get(get_analytics))
        .layer(middleware::from_fn(auth_middleware))
}

async fn list_quizzes(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Value>, AppError> {
    let is_admin = is_admin(&state, user_id).await?;
    let quizzes = QuizService::list(&state.pool, is_admin).await?;
    Ok(Json(json!({ "data": quizzes })))
}

async fn create_quiz(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Json(payload): Json<CreateQuizPayload>,
) -> Result<Json<Value>, AppError> {
    require_admin(&state, user_id).await?;
    payload.validate().map_err(AppError::Validation)?;
    let quiz = QuizService::create(&state.pool, payload, user_id).await?;
    Ok(Json(json!({ "data": quiz })))
}

async fn get_quiz(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(quiz_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let quiz = QuizService::get(&state.pool, quiz_id, is_admin(&state, user_id).await?).await?;
    let questions = QuizService::questions(&state.pool, quiz_id).await?;
    let questions: Vec<Value> = questions
        .into_iter()
        .map(|(question, options)| json!({ "question": question, "options": options }))
        .collect();
    Ok(Json(
        json!({ "data": { "quiz": quiz, "questions": questions } }),
    ))
}

async fn add_question(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(quiz_id): Path<Uuid>,
    Json(payload): Json<CreateQuestionPayload>,
) -> Result<Json<Value>, AppError> {
    require_admin(&state, user_id).await?;
    QuizService::get(&state.pool, quiz_id, true).await?;
    let (question, options) = QuizService::add_question(&state.pool, quiz_id, payload).await?;
    Ok(Json(
        json!({ "data": { "question": question, "options": options } }),
    ))
}

async fn start_attempt(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(quiz_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    require_student(&state, user_id).await?;
    let attempt = QuizService::start_attempt(&state.pool, quiz_id, user_id).await?;
    Ok(Json(json!({ "data": attempt })))
}

async fn submit_attempt(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(attempt_id): Path<Uuid>,
    Json(payload): Json<SubmitQuizPayload>,
) -> Result<Json<Value>, AppError> {
    require_student(&state, user_id).await?;
    let attempt =
        QuizService::submit_attempt(&state.pool, attempt_id, user_id, payload.answers).await?;
    Ok(Json(json!({ "data": attempt })))
}

async fn list_attempts(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<Value>, AppError> {
    require_student(&state, user_id).await?;
    let attempts = QuizService::history(&state.pool, user_id).await?;
    Ok(Json(json!({ "data": attempts })))
}

async fn get_analytics(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
    Path(quiz_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    require_admin(&state, user_id).await?;
    QuizService::get(&state.pool, quiz_id, true).await?;
    let scores = QuizService::analytics(&state.pool, quiz_id).await?;
    Ok(Json(json!({ "data": scores })))
}

async fn is_admin(state: &AppState, user_id: Uuid) -> Result<bool, AppError> {
    Ok(UserService::get_user_by_id(&state.pool, user_id)
        .await?
        .role
        == UserRole::Admin)
}

async fn require_admin(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    if !is_admin(state, user_id).await? {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn require_student(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    if UserService::get_user_by_id(&state.pool, user_id)
        .await?
        .role
        != UserRole::Student
    {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
