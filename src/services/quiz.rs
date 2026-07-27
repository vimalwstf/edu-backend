use std::collections::HashSet;

use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use validator::Validate;

use crate::{
    dto::quiz::{CreateQuestionPayload, CreateQuizPayload, SubmitAnswerPayload},
    entities::quiz::{
        Question, QuestionOption, Quiz, QuizAttempt, QuizAttemptHistory, QuizSummary,
        StudentQuizScore,
    },
    error::AppError,
};

pub struct QuizService;

impl QuizService {
    pub async fn create(
        pool: &PgPool,
        payload: CreateQuizPayload,
        admin_id: Uuid,
    ) -> Result<Quiz, AppError> {
        Ok(sqlx::query_as::<_, Quiz>(
            "INSERT INTO quizzes (id, title, description, created_by)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(payload.title)
        .bind(payload.description)
        .bind(admin_id)
        .fetch_one(pool)
        .await?)
    }

    pub async fn list(pool: &PgPool, include_inactive: bool) -> Result<Vec<QuizSummary>, AppError> {
        let query = if include_inactive {
            "SELECT q.id, q.title, q.description, q.created_by, q.is_active,
                    COUNT(question.id)::BIGINT AS question_count, q.created_at, q.updated_at
             FROM quizzes q LEFT JOIN questions question ON question.quiz_id = q.id
             GROUP BY q.id ORDER BY q.created_at DESC"
        } else {
            "SELECT q.id, q.title, q.description, q.created_by, q.is_active,
                    COUNT(question.id)::BIGINT AS question_count, q.created_at, q.updated_at
             FROM quizzes q LEFT JOIN questions question ON question.quiz_id = q.id
             WHERE q.is_active = TRUE
             GROUP BY q.id ORDER BY q.created_at DESC"
        };

        Ok(sqlx::query_as::<_, QuizSummary>(query)
            .fetch_all(pool)
            .await?)
    }

    pub async fn get(pool: &PgPool, id: Uuid, include_inactive: bool) -> Result<Quiz, AppError> {
        let quiz = sqlx::query_as::<_, Quiz>(
            "SELECT * FROM quizzes WHERE id = $1 AND ($2 OR is_active = TRUE)",
        )
        .bind(id)
        .bind(include_inactive)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
        Ok(quiz)
    }

    pub async fn questions(
        pool: &PgPool,
        quiz_id: Uuid,
    ) -> Result<Vec<(Question, Vec<QuestionOption>)>, AppError> {
        let questions = sqlx::query_as::<_, Question>(
            "SELECT id, quiz_id, question_text, question_type::TEXT, points, order_num
             FROM questions WHERE quiz_id = $1 ORDER BY order_num, created_at",
        )
        .bind(quiz_id)
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(questions.len());
        for question in questions {
            let options = sqlx::query_as::<_, QuestionOption>(
                "SELECT id, question_id, option_text FROM question_options
                 WHERE question_id = $1 ORDER BY id",
            )
            .bind(question.id)
            .fetch_all(pool)
            .await?;
            result.push((question, options));
        }
        Ok(result)
    }

    pub async fn add_question(
        pool: &PgPool,
        quiz_id: Uuid,
        payload: CreateQuestionPayload,
    ) -> Result<(Question, Vec<QuestionOption>), AppError> {
        validate_question(&payload)?;
        let mut tx = pool.begin().await?;

        let question = sqlx::query_as::<_, Question>(
            "INSERT INTO questions
                (id, quiz_id, question_text, question_type, points, order_num)
             VALUES ($1, $2, $3, $4::question_type, $5, $6)
             RETURNING id, quiz_id, question_text, question_type::TEXT, points, order_num",
        )
        .bind(Uuid::new_v4())
        .bind(quiz_id)
        .bind(payload.question_text)
        .bind(&payload.question_type)
        .bind(payload.points)
        .bind(payload.order_num.unwrap_or(0))
        .fetch_one(&mut *tx)
        .await?;

        let options = insert_options(&mut tx, question.id, payload.options).await?;
        tx.commit().await?;
        Ok((question, options))
    }

    pub async fn start_attempt(
        pool: &PgPool,
        quiz_id: Uuid,
        user_id: Uuid,
    ) -> Result<QuizAttempt, AppError> {
        let max_score: i32 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(points), 0)::INT FROM questions
             WHERE quiz_id = $1 AND EXISTS (SELECT 1 FROM quizzes WHERE id = $1 AND is_active)",
        )
        .bind(quiz_id)
        .fetch_one(pool)
        .await?;

        if !sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (SELECT 1 FROM quizzes WHERE id = $1 AND is_active)",
        )
        .bind(quiz_id)
        .fetch_one(pool)
        .await?
        {
            return Err(AppError::NotFound);
        }

        Ok(sqlx::query_as::<_, QuizAttempt>(
            "INSERT INTO quiz_attempts (id, user_id, quiz_id, max_score)
             VALUES ($1, $2, $3, $4) RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(quiz_id)
        .bind(max_score)
        .fetch_one(pool)
        .await?)
    }

    pub async fn submit_attempt(
        pool: &PgPool,
        attempt_id: Uuid,
        user_id: Uuid,
        answers: Vec<SubmitAnswerPayload>,
    ) -> Result<QuizAttempt, AppError> {
        let mut tx = pool.begin().await?;
        let attempt = sqlx::query_as::<_, QuizAttempt>(
            "SELECT * FROM quiz_attempts WHERE id = $1 AND user_id = $2 FOR UPDATE",
        )
        .bind(attempt_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound)?;

        if attempt.status != "in_progress" {
            return Err(AppError::BadRequest(
                "Attempt has already been submitted".into(),
            ));
        }

        let mut score = 0;
        let mut answered_questions = HashSet::new();
        for answer in answers {
            if !answered_questions.insert(answer.question_id) {
                return Err(AppError::BadRequest(
                    "A question can only be answered once".into(),
                ));
            }
            let question = sqlx::query_as::<_, (String, i32)>(
                "SELECT question_type::TEXT, points FROM questions
                 WHERE id = $1 AND quiz_id = $2",
            )
            .bind(answer.question_id)
            .bind(attempt.quiz_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::BadRequest("Question does not belong to this quiz".into()))?;

            let correct_ids: Vec<Uuid> = sqlx::query_scalar(
                "SELECT id FROM question_options WHERE question_id = $1 AND is_correct ORDER BY id",
            )
            .bind(answer.question_id)
            .fetch_all(&mut *tx)
            .await?;

            let is_correct = score_question(
                &question.0,
                &correct_ids,
                &answer.selected_option_ids,
                answer.short_answer.as_deref(),
            );
            let points = if is_correct { question.1 } else { 0 };
            score += points;

            sqlx::query(
                "INSERT INTO attempt_answers
                 (id, attempt_id, question_id, selected_option_ids, short_answer, is_correct, points_earned)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(Uuid::new_v4())
            .bind(attempt_id)
            .bind(answer.question_id)
            .bind(&answer.selected_option_ids)
            .bind(answer.short_answer)
            .bind(is_correct)
            .bind(points)
            .execute(&mut *tx)
            .await?;
        }

        let completed = sqlx::query_as::<_, QuizAttempt>(
            "UPDATE quiz_attempts SET score = $1, status = 'completed', completed_at = $2
             WHERE id = $3 RETURNING *",
        )
        .bind(score)
        .bind(Utc::now())
        .bind(attempt_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(completed)
    }

    pub async fn history(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<QuizAttemptHistory>, AppError> {
        Ok(sqlx::query_as::<_, QuizAttemptHistory>(
            "SELECT a.id, a.quiz_id, q.title AS quiz_title, a.started_at,
                    a.completed_at, a.score, a.max_score, a.status
             FROM quiz_attempts a JOIN quizzes q ON q.id = a.quiz_id
             WHERE a.user_id = $1 ORDER BY a.started_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?)
    }

    pub async fn analytics(
        pool: &PgPool,
        quiz_id: Uuid,
    ) -> Result<Vec<StudentQuizScore>, AppError> {
        Ok(sqlx::query_as::<_, StudentQuizScore>(
            "SELECT u.id AS user_id, u.first_name, u.last_name, u.email,
                    (SELECT COUNT(*) FROM quiz_attempts a
                     WHERE a.quiz_id = $1 AND a.user_id = u.id)::BIGINT AS attempt_count,
                    latest.score, latest.max_score, latest.status
             FROM users u
             LEFT JOIN LATERAL (
                 SELECT a.score, a.max_score, a.status
                 FROM quiz_attempts a
                 WHERE a.quiz_id = $1 AND a.user_id = u.id
                 ORDER BY a.completed_at DESC NULLS LAST, a.started_at DESC
                 LIMIT 1
             ) latest ON TRUE
             WHERE u.role = 'student'::user_role
             ORDER BY u.last_name NULLS LAST, u.first_name NULLS LAST, u.email",
        )
        .bind(quiz_id)
        .fetch_all(pool)
        .await?)
    }
}

fn validate_question(payload: &CreateQuestionPayload) -> Result<(), AppError> {
    payload.validate().map_err(AppError::Validation)?;
    let valid_types = ["single_choice", "multiple_choice", "true_false"];
    if !valid_types.contains(&payload.question_type.as_str()) {
        return Err(AppError::BadRequest(
            "question_type must be single_choice, multiple_choice, or true_false".into(),
        ));
    }
    if payload.options.len() < 2 || !payload.options.iter().any(|option| option.is_correct) {
        return Err(AppError::BadRequest(
            "A question needs at least two options and one correct option".into(),
        ));
    }
    if payload.question_type != "multiple_choice"
        && payload
            .options
            .iter()
            .filter(|option| option.is_correct)
            .count()
            != 1
    {
        return Err(AppError::BadRequest(
            "This question type must have exactly one correct option".into(),
        ));
    }
    Ok(())
}

async fn insert_options(
    tx: &mut Transaction<'_, Postgres>,
    question_id: Uuid,
    options: Vec<crate::dto::quiz::CreateOptionPayload>,
) -> Result<Vec<QuestionOption>, AppError> {
    let mut result = Vec::with_capacity(options.len());
    for option in options {
        result.push(
            sqlx::query_as::<_, QuestionOption>(
                "INSERT INTO question_options (id, question_id, option_text, is_correct)
                 VALUES ($1, $2, $3, $4) RETURNING id, question_id, option_text",
            )
            .bind(Uuid::new_v4())
            .bind(question_id)
            .bind(option.option_text)
            .bind(option.is_correct)
            .fetch_one(&mut **tx)
            .await?,
        );
    }
    Ok(result)
}

pub fn score_question(
    question_type: &str,
    correct_ids: &[Uuid],
    selected_ids: &[Uuid],
    _short_answer: Option<&str>,
) -> bool {
    if question_type == "short_answer" {
        return false;
    }
    let correct: HashSet<Uuid> = correct_ids.iter().copied().collect();
    let selected: HashSet<Uuid> = selected_ids.iter().copied().collect();
    !correct.is_empty() && correct == selected
}

#[cfg(test)]
mod tests {
    use super::{score_question, validate_question};
    use crate::dto::quiz::{CreateOptionPayload, CreateQuestionPayload};
    use uuid::Uuid;

    #[test]
    fn scores_single_choice_only_when_the_answer_matches() {
        let correct = Uuid::new_v4();
        let wrong = Uuid::new_v4();
        assert!(score_question(
            "single_choice",
            &[correct],
            &[correct],
            None
        ));
        assert!(!score_question("single_choice", &[correct], &[wrong], None));
        assert!(!score_question(
            "single_choice",
            &[correct],
            &[correct, wrong],
            None
        ));
    }

    #[test]
    fn scores_multiple_choice_as_a_set() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        assert!(score_question(
            "multiple_choice",
            &[first, second],
            &[second, first],
            None
        ));
        assert!(!score_question(
            "multiple_choice",
            &[first, second],
            &[first],
            None
        ));
    }

    #[test]
    fn rejects_single_choice_questions_with_multiple_correct_options() {
        let payload = CreateQuestionPayload {
            question_text: "Pick one".into(),
            question_type: "single_choice".into(),
            points: 1,
            order_num: None,
            options: vec![
                CreateOptionPayload {
                    option_text: "A".into(),
                    is_correct: true,
                },
                CreateOptionPayload {
                    option_text: "B".into(),
                    is_correct: true,
                },
            ],
        };
        assert!(validate_question(&payload).is_err());
    }
}
