use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
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
    time::{Duration, Instant},
};

use tokio::sync::RwLock;


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,

    #[serde(default)]
    preferred_username: Option<String>,

    #[serde(default)]
    email: Option<String>,

    exp: usize,

    iss: String,

    #[serde(default)]
    aud: Vec<String>,
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

    #[serde(default)]
    alg: Option<String>,

    #[serde(default)]
    use_: Option<String>,
}


struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    loaded_at: Instant,
}


struct AppState {
    issuer: String,
    audience: String,
    jwks_url: String,

    jwks: RwLock<JwksCache>,
}


type SharedState = Arc<AppState>;


#[tokio::main]
async fn main() {
    let exposed_addr =
        env::var("EXPOSED_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:50051".to_string());

    let issuer =
        env::var("KEYCLOAK_ISSUER")
            .expect("KEYCLOAK_ISSUER must be configured");

    let jwks_url =
        env::var("KEYCLOAK_JWKS_URL")
            .expect("KEYCLOAK_JWKS_URL must be configured");

    let audience =
        env::var("JWT_AUDIENCE")
            .expect("JWT_AUDIENCE must be configured");

    println!("Loading Keycloak JWKS...");

    let keys = load_jwks(&jwks_url)
        .await
        .expect("Failed to load Keycloak JWKS");

    let state = Arc::new(AppState {
        issuer,
        audience,
        jwks_url,

        jwks: RwLock::new(JwksCache {
            keys,
            loaded_at: Instant::now(),
        }),
    });

    let public_routes =
        Router::new()
            .route("/health", get(health));

    let protected_routes =
        Router::new()
            .route(
                "/protected-data",
                get(get_protected_data),
            )
            .route_layer(
                middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ),
            );

    let app =
        Router::new()
            .merge(public_routes)
            .merge(protected_routes)
            .with_state(state);

    let listener =
        tokio::net::TcpListener::bind(&exposed_addr)
            .await
            .expect("Failed to bind backend");

    println!(
        "Backend listening on {}",
        exposed_addr
    );

    axum::serve(listener, app)
        .await
        .expect("Backend server failed");
}


async fn load_jwks(
    url: &str,
) -> Result<HashMap<String, DecodingKey>, Box<dyn std::error::Error + Send + Sync>> {
    let client =
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

    let response =
        client
            .get(url)
            .send()
            .await?
            .error_for_status()?;

    let jwks: Jwks =
        response.json().await?;

    let mut keys = HashMap::new();

    for jwk in jwks.keys {
        // We only accept RSA signing keys.
        if jwk.kty != "RSA" {
            continue;
        }

        if let Some(alg) = &jwk.alg {
            if alg != "RS256" {
                continue;
            }
        }

        let key =
            DecodingKey::from_rsa_components(
                &jwk.n,
                &jwk.e,
            )?;

        keys.insert(jwk.kid, key);
    }

    if keys.is_empty() {
        return Err("JWKS contains no usable RSA keys".into());
    }

    Ok(keys)
}


async fn refresh_jwks(
    state: &SharedState,
) -> Result<(), StatusCode> {
    let keys =
        load_jwks(&state.jwks_url)
            .await
            .map_err(|err| {
                eprintln!(
                    "Failed to refresh JWKS: {}",
                    err
                );

                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    let mut cache =
        state.jwks.write().await;

    cache.keys = keys;
    cache.loaded_at = Instant::now();

    Ok(())
}


async fn auth_middleware(
    State(state): State<SharedState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    let auth_header =
        req.headers()
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

    let token =
        match auth_header {
            Some(value)
                if value.starts_with("Bearer ") =>
            {
                value
                    .strip_prefix("Bearer ")
                    .unwrap_or("")
                    .trim()
            }

            _ => {
                return Err(StatusCode::UNAUTHORIZED);
            }
        };

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // ------------------------------------------------------------
    // Read JWT header.
    // ------------------------------------------------------------

    let header =
        decode_header(token)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Never allow algorithm supplied by attacker.
    if header.alg != Algorithm::RS256 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let kid =
        header.kid
            .ok_or(StatusCode::UNAUTHORIZED)?;

    // ------------------------------------------------------------
    // Find key by kid.
    // ------------------------------------------------------------

    let decoding_key = {
        let cache =
            state.jwks.read().await;

        cache.keys.get(&kid).cloned()
    };

    let decoding_key =
        match decoding_key {
            Some(key) => key,

            None => {
                // Possible Keycloak key rotation.
                //
                // Refresh JWKS and try once more.
                refresh_jwks(&state).await?;

                let cache =
                    state.jwks.read().await;

                cache.keys
                    .get(&kid)
                    .cloned()
                    .ok_or(StatusCode::UNAUTHORIZED)?
            }
        };

    // ------------------------------------------------------------
    // JWT validation.
    // ------------------------------------------------------------

    let mut validation =
        Validation::new(Algorithm::RS256);

    validation.validate_exp = true;

    // Important: issuer is canonical public URL.
    validation.set_issuer(&[
        state.issuer.as_str()
    ]);

    // Important: validate audience.
    validation.set_audience(&[
        state.audience.as_str()
    ]);

    let token_data =
        decode::<Claims>(
            token,
            &decoding_key,
            &validation,
        )
        .map_err(|err| {
            eprintln!(
                "JWT validation failed: {}",
                err
            );

            StatusCode::UNAUTHORIZED
        })?;

    // ------------------------------------------------------------
    // Put verified claims into request extensions.
    // ------------------------------------------------------------

    req.extensions_mut()
        .insert(token_data.claims);

    Ok(next.run(req).await)
}


async fn health() -> &'static str {
    "Backend health ok"
}


async fn get_protected_data() -> &'static str {
    "This is verified production data from Rust backend!"
}