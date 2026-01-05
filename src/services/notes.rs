use crate::{dto::notes::CreateNoteData, entities::notes::Note, error::AppError};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct NoteService;

impl NoteService {
    pub async fn create(pool: &PgPool, payload: CreateNoteData) -> Result<Note, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let note: Note = sqlx::query_as(
            r#"
            INSERT INTO notes (
                id, title, description, file_name, s3_key,
                content_type, file_size, uploaded_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.file_name)
        .bind(&payload.s3_key)
        .bind(&payload.content_type)
        .bind(payload.file_size)
        .bind(&payload.uploaded_by)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(note)
    }

    pub async fn list_by_user(pool: &PgPool, user_id: Uuid) -> Result<Vec<Note>, AppError> {
        let notes = sqlx::query_as::<_, Note>(
            r#"
                SELECT *
                FROM notes
                WHERE uploaded_by = $1
                ORDER BY created_at DESC
                LIMIT 100
                "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(notes)
    }
}
