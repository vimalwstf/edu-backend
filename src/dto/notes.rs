use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::notes::Note;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNoteData {
    pub title: String,
    pub description: Option<String>,
    pub file_name: String,
    pub s3_key: String,
    pub content_type: String,
    pub file_size: i64,
    pub uploaded_by: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNotePayload {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct NoteWithDownloadUrl {
    pub note: Note,
    pub download_url: String,
}
