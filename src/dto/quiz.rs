// use serde::{Deserialize, Serialize};
// use uuid::Uuid;
// use validator::Validate;

// #[derive(Debug, Deserialize, Validate)]
// pub struct CreateQuizPayload {
//     pub title: String,
//     #[validate(length(min = 1))]
//     pub description: Option<String>,
// }

// #[derive(Debug, Deserialize)]
// pub struct CreateQuestionPayload {
//     pub question_text: String,
//     pub question_type: String, // "single_choice", etc.
//     pub points: i32,
//     pub options: Vec<CreateOptionPayload>,
// }

// #[derive(Debug, Deserialize)]
// pub struct CreateOptionPayload {
//     pub option_text: String,
//     pub is_correct: bool,
// }

// #[derive(Serialize)]
// pub struct QuizResponse {
//     pub id: Uuid,
//     pub title: String,
//     pub description: Option<String>,
//     pub created_by: Uuid,
//     pub is_active: bool,
//     pub question_count: i64, // optional: count for frontend
// }

// #[derive(Serialize)]
// pub struct QuestionWithOptions {
//     pub id: Uuid,
//     pub question_text: String,
//     pub question_type: String,
//     pub points: i32,
//     pub options: Vec<OptionResponse>,
// }

// #[derive(Serialize)]
// pub struct OptionResponse {
//     pub id: Uuid,
//     pub option_text: String,
//     // do NOT send is_correct to students!
// }

// #[derive(Deserialize)]
// pub struct SubmitAnswerPayload {
//     pub question_id: Uuid,
//     pub selected_option_ids: Option<Vec<Uuid>>, // for multiple choice
//     pub short_answer: Option<String>,
// }

// #[derive(Deserialize)]
// pub struct SubmitQuizPayload {
//     pub attempt_id: Uuid,
//     pub answers: Vec<SubmitAnswerPayload>,
// }

// #[derive(Serialize)]
// pub struct QuizAttemptResponse {
//     pub id: Uuid,
//     pub quiz_id: Uuid,
//     pub score: i32,
//     pub max_score: i32,
//     pub status: String,
//     pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
// }
