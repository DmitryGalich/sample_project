#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let state = AppState {
        issuer,
        audience,
    };

    let app = Router::new()
        .route("/health", get(health))
        .with_state(state);

    let listener =
        tokio::net::TcpListener::bind("0.0.0.0:8080")
            .await
            .expect("failed to bind");

    info!("listening on 0.0.0.0:8080");

    axum::serve(listener, app)
        .await
        .expect("server failed");
}

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok"
    }))
}
