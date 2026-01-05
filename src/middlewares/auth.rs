use axum::{body::Body, http::Request, middleware::Next, response::Response};

use crate::{error::AppError, utils::jwt::verify_token};

pub async fn auth_middleware(mut req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            header.trim_start_matches("Bearer ").to_string()
        }
        _ => return Err(AppError::Unauthorized),
    };

    let claims = verify_token(&token).map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
