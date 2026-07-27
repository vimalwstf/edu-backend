use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Quiz {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct QuizSummary {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub is_active: bool,
    pub question_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Question {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub question_text: String,
    pub question_type: String,
    pub points: i32,
    pub order_num: i32,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct QuestionOption {
    pub id: Uuid,
    pub question_id: Uuid,
    pub option_text: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct QuizAttempt {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quiz_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub score: i32,
    pub max_score: i32,
    pub status: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct QuizAttemptHistory {
    pub id: Uuid,
    pub quiz_id: Uuid,
    pub quiz_title: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub score: i32,
    pub max_score: i32,
    pub status: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct StudentQuizScore {
    pub user_id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
    pub attempt_count: i64,
    pub score: Option<i32>,
    pub max_score: Option<i32>,
    pub status: Option<String>,
}
