// entities/quiz.rs

use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

#[derive(sqlx::FromRow)]
pub struct Quiz {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// similar for Question, QuestionOption, QuizAttempt, AttemptAnswer
