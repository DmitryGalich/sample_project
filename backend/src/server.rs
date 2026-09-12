use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    routing::get,
    Router,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::{Instant, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RealmAccess {
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub exp: u64,
    pub iss: String,
    pub realm_access: Option<RealmAccess>, 
}

#[derive(Debug, Deserialize, Clone)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Jwks {
    keys: Vec<Jwk>,
}

pub struct ServerConfig {
    pub jwks_url: String,
    pub allowed_issuers: Vec<String>,
    pub allowed_audiences: Vec<String>,
    pub jwks_min_refresh_interval: Duration,
}

struct CachedJwks {
    data: Jwks,
    last_updated: Instant,
}

pub struct AppState {
    config: ServerConfig,
    jwks: RwLock<CachedJwks>,
}

async fn fetch_jwks(url: &str) -> Result<Jwks, StatusCode> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    client.get(url)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<Jwks>()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn run_server(host_port: &str, config: ServerConfig) {
    tracing::info!("Initial loading keys from Keycloak...");
    let initial_jwks = fetch_jwks(&config.jwks_url)
        .await
        .expect("Keycloak is not available");
    
    tracing::info!("Keys downloaded: {}", initial_jwks.keys.len());

    let shared_state = Arc::new(AppState {
        config,
        jwks: RwLock::new(CachedJwks {
            data: initial_jwks,
            last_updated: Instant::now(),
        }),
    });

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .route("/admin/dashboard", get(admin_handler))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(host_port).await.unwrap();
    tracing::info!("Started on {}", host_port);
    axum::serve(listener, app).await.unwrap();
}

async fn protected_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<String, StatusCode> {
    tracing::debug!("New request /protected");

    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Denied: No `Authorization` header");
            StatusCode::UNAUTHORIZED
        })?;

    if !auth_header.starts_with("Bearer ") {
        tracing::warn!("Denied: No `Bearer` in Header begin");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = &auth_header["Bearer ".len()..];

    let header = decode_header(token).map_err(|e| {
        tracing::error!("Denied: Can't decode token header: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;
    
    let token_kid = header.kid.ok_or_else(|| {
        tracing::warn!("Denied: No 'kid' in token");
        StatusCode::UNAUTHORIZED
    })?;

    let mut found_jwk = {
        let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.data.keys.iter().find(|jwk| jwk.kid == token_kid).cloned()
    };

    if found_jwk.is_none() {
        {
            let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let elapsed = jwks_guard.last_updated.elapsed();
            
            if elapsed < state.config.jwks_min_refresh_interval {
                tracing::warn!(
                    "Flood control denied request. kid='{}' after {:?}. Min interval: {:?}", 
                    token_kid, elapsed, state.config.jwks_min_refresh_interval
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
        }

        tracing::info!("Key with kid='{}' not found in cache. Updating JWKS...", token_kid);
        let fresh_jwks = fetch_jwks(&state.config.jwks_url).await?;
        
        let mut jwks_guard = state.jwks.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.data = fresh_jwks;
        jwks_guard.last_updated = Instant::now();
        
        found_jwk = jwks_guard.data.keys.iter().find(|jwk| jwk.kid == token_kid).cloned();
        tracing::info!("JWKS updated");
    }

    let target_jwk = found_jwk.ok_or_else(|| {
        tracing::warn!("Denied: Key with kid='{}' not found on Keycloak even after update", token_kid);
        StatusCode::UNAUTHORIZED
    })?;

    let decoding_key = DecodingKey::from_rsa_components(&target_jwk.n, &target_jwk.e)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&state.config.allowed_issuers);
    validation.set_audience(&state.config.allowed_audiences); 

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| {
            tracing::error!("Crypto error or old token: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    tracing::info!("Access granted for: {}", token_data.claims.preferred_username);
    
    Ok(format!(
        "Access granted! Hello, {}",
        token_data.claims.preferred_username
    ))
}

async fn admin_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<String, StatusCode> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = &auth_header["Bearer ".len()..];

    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token_kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;

    let mut found_jwk = {
        let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.data.keys.iter().find(|jwk| jwk.kid == token_kid).cloned()
    };

    if found_jwk.is_none() {
        let fresh_jwks = fetch_jwks(&state.config.jwks_url).await?;
        let mut jwks_guard = state.jwks.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.data = fresh_jwks;
        jwks_guard.last_updated = Instant::now();
        found_jwk = jwks_guard.data.keys.iter().find(|jwk| jwk.kid == token_kid).cloned();
    }

    let target_jwk = found_jwk.ok_or(StatusCode::UNAUTHORIZED)?;
    let decoding_key = DecodingKey::from_rsa_components(&target_jwk.n, &target_jwk.e)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&state.config.allowed_issuers);
    validation.set_audience(&state.config.allowed_audiences); 

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let has_admin_role = token_data.claims.realm_access
        .map(|access| access.roles.contains(&"admin".to_string()))
        .unwrap_or(false);

    if !has_admin_role {
        tracing::warn!("Denied: User {} has no right 'admin'", token_data.claims.preferred_username);
        return Err(StatusCode::FORBIDDEN); 
    }

    tracing::info!("Access granted: {}", token_data.claims.preferred_username);
    Ok(format!("Welcome, admin {}!", token_data.claims.preferred_username))
}
