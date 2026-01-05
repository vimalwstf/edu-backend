use edu_backend::{api, db, state};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let pool = db::create_pool().await;

    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    let app_state = state::AppState { pool, s3_client };

    let app = api::api_router().with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}
