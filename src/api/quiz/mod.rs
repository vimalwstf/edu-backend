// use axum::{Router, middleware};

// use crate::state::AppState;

// pub fn quiz_router() -> Router<AppState> {
//     Router::new()
//         .route("/", post(create_quiz).get(list_quizzes))
//         .route("/:id", get(get_quiz))
//         .route("/:id/questions", post(add_question)) // admin only
//         .route("/attempts/start", post(start_attempt))
//         .route("/attempts/submit", post(submit_quiz))
//         .route("/attempts/:id", get(get_attempt_results))
//         .route("/analytics/:id", get(get_quiz_analytics)) // admin only
//         .layer(middleware::from_fn(auth_middleware))
// }
