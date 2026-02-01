// use crate::{
//     dto::quiz::*,
//     entities::{quiz::Quiz, *},
//     error::AppError,
// };
// use sqlx::{PgPool, Row};
// use uuid::Uuid;

// pub struct QuizService;

// impl QuizService {
//     pub async fn create_quiz(
//         pool: &PgPool,
//         data: CreateQuizPayload,
//         admin_id: Uuid,
//     ) -> Result<Quiz, AppError> {
//         let quiz = sqlx::query_as!(
//             Quiz,
//             r#"
//             INSERT INTO quizzes (id, title, description, created_by)
//             VALUES ($1, $2, $3, $4)
//             RETURNING *
//             "#,
//             Uuid::new_v4(),
//             data.title,
//             data.description,
//             admin_id
//         )
//         .fetch_one(pool)
//         .await?;
//         Ok(quiz)
//     }

//     // Add create_question (with options), list_quizzes (filter active, admin sees all), get_quiz_with_questions (hide correct answers for students), start_attempt, submit_attempt (compute score), etc.

//     pub async fn start_attempt(
//         pool: &PgPool,
//         user_id: Uuid,
//         quiz_id: Uuid,
//     ) -> Result<Uuid, AppError> {
//         let max_score: i64 = sqlx::query_scalar!(
//             "SELECT COALESCE(SUM(points), 0) FROM questions WHERE quiz_id = $1",
//             quiz_id
//         )
//         .fetch_one(pool)
//         .await?;

//         let attempt_id = Uuid::new_v4();

//         sqlx::query!(
//             r#"
//             INSERT INTO quiz_attempts (id, user_id, quiz_id, max_score)
//             VALUES ($1, $2, $3, $4)
//             "#,
//             attempt_id,
//             user_id,
//             quiz_id,
//             max_score as i32
//         )
//         .execute(pool)
//         .await?;

//         Ok(attempt_id)
//     }

//     pub async fn submit_attempt(
//         pool: &PgPool,
//         attempt_id: Uuid,
//         user_id: Uuid,
//         answers: Vec<SubmitAnswerPayload>,
//     ) -> Result<i32, AppError> {
//         // 1. Verify attempt belongs to user and is in_progress
//         // 2. For each answer: check correctness, compute points_earned, update attempt_answers
//         // 3. Sum points_earned -> update quiz_attempts score & status='completed', completed_at=now()
//         // Return final score
//         // Implement carefully with transactions
//         todo!();
//     }

//     // More: get_attempt_results (for student), get_quiz_analytics (for admin: avg score, completion rate, etc.)
// }
