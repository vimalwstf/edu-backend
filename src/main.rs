use edu_backend::{api, db, state};
use tokio::net::TcpListener;
use tower_http::{
    LatencyUnit,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

#[tokio::main]
async fn main() {
    let pool = db::create_pool().await;

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO) // или Level::DEBUG
        .with_target(false)
        .init();

    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    let app_state = state::AppState { pool, s3_client };

    let app = api::api_router().with_state(app_state).layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO)) // ← request start
            .on_request(DefaultOnRequest::new().level(Level::INFO)) // optional: customize
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros), // or Millis
            )
            // optional: log failures (4xx/5xx) at warn/error level
            .on_failure(tower_http::trace::DefaultOnFailure::new().level(Level::WARN)),
    );

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}
