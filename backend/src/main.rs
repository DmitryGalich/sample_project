use axum::{
    http::{header::AUTHORIZATION, StatusCode},
    routing::get,
    Router,
};


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/protected", get(protected_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Started on 8000");
    axum::serve(listener, app).await.unwrap();
}

async fn protected_handler(
    headers: axum::http::HeaderMap,
) -> Result<String, StatusCode> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header["Bearer ".len()..];
    
    println!("Token: {}", token);

    Ok(format!("Token received but untested"))
}