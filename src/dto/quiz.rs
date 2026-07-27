use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQuizPayload {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateQuestionPayload {
    #[validate(length(min = 1))]
    pub question_text: String,
    pub question_type: String,
    #[validate(range(min = 1))]
    pub points: i32,
    pub order_num: Option<i32>,
    pub options: Vec<CreateOptionPayload>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOptionPayload {
    #[validate(length(min = 1))]
    pub option_text: String,
    pub is_correct: bool,
}

#[derive(Debug, Deserialize)]
pub struct SubmitAnswerPayload {
    pub question_id: Uuid,
    #[serde(default)]
    pub selected_option_ids: Vec<Uuid>,
    pub short_answer: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitQuizPayload {
    pub answers: Vec<SubmitAnswerPayload>,
}
