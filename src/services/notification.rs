use crate::dto::notification::CreateNotificationPayload;
use crate::entities::notification::Notification;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

pub struct NotificationService;

impl NotificationService {
    pub async fn create(
        pool: &PgPool,
        payload: CreateNotificationPayload,
    ) -> Result<Notification, AppError> {
        let id = Uuid::new_v4();

        let notification = sqlx::query_as::<_, Notification>(
            r#"
            INSERT INTO notifications (id, title, description)
            VALUES ($1, $2, $3)
            RETURNING id, title, description, created_at
            "#,
        )
        .bind(id)
        .bind(payload.title)
        .bind(payload.description)
        .fetch_one(pool)
        .await?;

        Ok(notification)
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<Notification>, AppError> {
        let notifications = sqlx::query_as::<_, Notification>(
            "SELECT id, title, description, created_at FROM notifications ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;

        Ok(notifications)
    }
}
