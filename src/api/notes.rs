use aws_sdk_s3::primitives::ByteStream;
use axum::{
    Extension, Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::post,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    dto::notes::{CreateNoteData, CreateNotePayload, NoteWithDownloadUrl},
    entities::notes::Note,
    middlewares::auth::auth_middleware,
};
use crate::{error::AppError, services::notes::NoteService};
use crate::{services::s3::S3Service, state::AppState};

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

pub fn notes_router() -> Router<AppState> {
    Router::new()
        .route("/upload", post(upload_note))
        .route("/", axum::routing::get(list_user_notes))
        .layer(DefaultBodyLimit::max(MAX_FILE_SIZE))
        .layer(middleware::from_fn(auth_middleware))
}

async fn list_user_notes(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let notes = NoteService::list_by_user(&state.pool, user_id).await?;
    let mut result = Vec::with_capacity(notes.len());

    for note in notes {
        let download_url = S3Service::get_download_url(&state.s3_client, &note.s3_key).await?;

        result.push(NoteWithDownloadUrl { note, download_url });
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Notes retrieved successfully",
            "notes": result,
        })),
    ))
}

async fn upload_note(
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>, // from your JWT middleware
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let bucket_name = std::env::var("AWS_BUCKET_NAME").expect("AWS_BUCKET_NAME must be set");

    let mut payload = CreateNotePayload {
        title: String::new(),
        description: None,
    };

    let mut file_bytes: Option<Bytes> = None;
    let mut file_name: String = "unknown.file".to_string();
    let mut content_type: String = "application/octet-stream".to_string();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    {
        match field.name() {
            Some("title") => {
                payload.title = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
            }
            Some("description") => {
                payload.description = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
                );
            }
            Some("file") => {
                file_name = field
                    .file_name()
                    .map(|n| n.to_string())
                    .unwrap_or("uploaded.file".to_string());

                content_type = field
                    .content_type()
                    .map(|c| c.to_string())
                    .unwrap_or("application/octet-stream".to_string());

                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
                );
            }
            _ => {}
        }
    }

    let bytes = file_bytes.ok_or(AppError::BadRequest("Missing file".into()))?;

    if payload.title.is_empty() {
        return Err(AppError::BadRequest("Title is required".into()));
    }

    // Generate S3 key with extension
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    let s3_key = format!("notes/{}{}", Uuid::new_v4(), extension);

    // Upload to S3 (private bucket)
    let body = ByteStream::from(bytes.to_vec());
    state
        .s3_client
        .put_object()
        .bucket(&bucket_name)
        .key(&s3_key)
        .content_type(content_type.clone())
        .content_length(bytes.len() as i64)
        .body(body)
        .send()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let note: Note = NoteService::create(
        &state.pool,
        CreateNoteData {
            title: payload.title,
            description: payload.description,
            file_name: file_name,
            content_type: content_type,
            file_size: bytes.len() as i64,
            uploaded_by: user_id,
            s3_key: s3_key,
        },
    )
    .await?;

    let download_url = S3Service::get_download_url(&state.s3_client, &note.s3_key).await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Note created successfully",
            "note": note,
            "download_url": download_url
        })),
    ))
}
