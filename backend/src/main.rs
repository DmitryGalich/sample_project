use axum::{routing::get, Router};
use std::env;

#[tokio::main]
async fn main() {
    let exposed_addr: String = env::var("EXPOSED_ADDR").expect("EXPOSED_ADDR must be set");
    println!("EXPOSED_ADDR: {}", &exposed_addr);

    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind(exposed_addr).await.unwrap();
    println!("Started...");

    axum::serve(listener, app).await.unwrap();

    println!("Stopped");
}

async fn health() -> &'static str {
    "Backend health ok"
}
