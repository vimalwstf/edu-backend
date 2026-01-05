mod auth;
mod notes;

use axum::{Router, http::StatusCode, response::IntoResponse};

use crate::{
    api::{auth::auth_router, notes::notes_router},
    state::AppState,
};

pub fn api_router() -> Router<AppState> {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/auth", auth_router())
                .nest("/notes", notes_router()),
        )
        .fallback(handler_404)
}

async fn handler_404() -> impl IntoResponse {
    let html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>404 Not Found</title>
            <style>
                body {
                    font-family: Arial, sans-serif;
                    background-color: #f4f4f4;
                    color: #333;
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    min-height: 100vh;
                    margin: 0;
                    text-align: center;
                }
                .container {
                    background-color: #fff;
                    padding: 40px;
                    border-radius: 8px;
                    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
                }
                h1 {
                    font-size: 4em;
                    margin-bottom: 0.5em;
                    color: #dc3545;
                }
                p {
                    font-size: 1.2em;
                }
                a {
                    color: #007bff;
                    text-decoration: none;
                }
                a:hover {
                    text-decoration: underline;
                }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>404</h1>
                <p>Oops! The page you're looking for could not be found.</p>
                <p>It seems like this page has wandered off into the digital wilderness.</p>
                <p><a href="/">Go back to the homepage</a></p>
            </div>
        </body>
        </html>
    "#;

    (StatusCode::NOT_FOUND, axum::response::Html(html))
}
