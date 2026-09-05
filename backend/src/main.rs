use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use jsonwebtoken::{
    decode,
    decode_header,
    Algorithm,
    DecodingKey,
    Validation,
};

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    sync::Arc,
};

use tracing::info;

#[derive(Clone)]
struct AppState {
    issuer: String,
    audience: String,
    jwks: Arc<Jwks>,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
    alg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: Audience,
    exp: usize,

    #[serde(default)]
    preferred_username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Audience {
    One(String),
    Many(Vec<String>),
}

impl Audience {
    fn contains(&self, value: &str) -> bool {
        match self {
            Audience::One(aud) => aud == value,
            Audience::Many(auds) => auds.iter().any(|aud| aud == value),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let issuer =
        env::var("OIDC_ISSUER")
            .expect("OIDC_ISSUER is required");

    let audience =
        env::var("OIDC_AUDIENCE")
            .unwrap_or_else(|_| "backend".to_string());

    let jwks_url = format!(
        "{}/protocol/openid-connect/certs",
        issuer.trim_end_matches('/')
    );

    info!("OIDC issuer: {}", issuer);
    info!("JWKS URL: {}", jwks_url);

    let client = reqwest::Client::new();

    let jwks: Jwks = client
        .get(&jwks_url)
        .send()
        .await
        .expect("failed to download JWKS")
        .error_for_status()
        .expect("JWKS request failed")
        .json()
        .await
        .expect("invalid JWKS");

    let state = AppState {
        issuer,
        audience,
        jwks: Arc::new(jwks),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/public", get(public))
        .route("/api/private", get(private))
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


async fn public() -> impl IntoResponse {
    Json(serde_json::json!({
        "message": "public endpoint"
    }))
}


async fn private(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {

    let token = match extract_bearer(&headers) {
        Some(token) => token,

        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "missing bearer token"
                })),
            );
        }
    };

    match verify_token(&state, token).await {

        Ok(claims) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "authenticated": true,
                "subject": claims.sub,
                "username": claims.preferred_username
            })),
        ),

        Err(error) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": error
            })),
        ),
    }
}


fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?;

    let value = value.to_str().ok()?;

    value.strip_prefix("Bearer ")
}


async fn verify_token(
    state: &AppState,
    token: &str,
) -> Result<Claims, &'static str> {

    let header =
        decode_header(token)
            .map_err(|_| "invalid token header")?;

    if header.alg != Algorithm::RS256 {
        return Err("unsupported signing algorithm");
    }

    let kid =
        header.kid.ok_or("missing kid")?;

    let jwk =
        state
            .jwks
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or("signing key not found")?;

    if jwk.kty != "RSA" {
        return Err("invalid key type");
    }

    let decoding_key =
        DecodingKey::from_rsa_components(
            &jwk.n,
            &jwk.e,
        )
        .map_err(|_| "invalid RSA key")?;

    let mut validation =
        Validation::new(Algorithm::RS256);

    validation.set_issuer(&[&state.issuer]);
    validation.set_audience(&[&state.audience]);

    let token_data =
        decode::<Claims>(
            token,
            &decoding_key,
            &validation,
        )
        .map_err(|_| "invalid or expired token")?;

    if !token_data
        .claims
        .iss
        .eq(&state.issuer)
    {
        return Err("invalid issuer");
    }

    if !token_data
        .claims
        .aud
        .contains(&state.audience)
    {
        return Err("invalid audience");
    }

    Ok(token_data.claims)
}