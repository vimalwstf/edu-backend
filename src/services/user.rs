use crate::dto::user::{LoginPayload, RegisterPayload};
use crate::entities::user::User;
use crate::error::AppError;
use crate::utils::{
    crypto::{hash_password, verify_password},
    jwt::create_token,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserService;

impl UserService {
    pub async fn register(pool: &PgPool, mut payload: RegisterPayload) -> Result<User, AppError> {
        payload.email = payload.email.to_lowercase();

        let password_hash = hash_password(&payload.password)?;

        let id = Uuid::new_v4();
        let now = Utc::now();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                id, first_name, last_name, email, password_hash,
                created_at, updated_at, role
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'student'::user_role)
            RETURNING id, first_name, last_name, email, role, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.email)
        .bind(password_hash)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("users_email_key") {
                    return AppError::EmailTaken;
                }
            }
            e.into()
        })?;

        Ok(user)
    }

    pub async fn login(
        pool: &PgPool,
        mut payload: LoginPayload,
    ) -> Result<(User, String), AppError> {
        payload.email = payload.email.to_lowercase();

        let password_hash: Option<String> =
            sqlx::query_scalar("SELECT password_hash FROM users WHERE email = $1")
                .bind(&payload.email)
                .fetch_optional(pool)
                .await?;

        let hash = match password_hash {
            Some(h) => h,
            None => return Err(AppError::InvalidCredentials),
        };

        if !verify_password(&hash, &payload.password) {
            return Err(AppError::InvalidCredentials);
        }

        let user = sqlx::query_as::<_, User>(
            "SELECT id, first_name, last_name, email, role, created_at, updated_at
             FROM users WHERE email = $1",
        )
        .bind(&payload.email)
        .fetch_one(pool)
        .await?;

        let token = create_token(user.id);
        Ok((user, token))
    }
}
