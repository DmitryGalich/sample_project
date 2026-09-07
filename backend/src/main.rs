use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Extension, Router,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, env, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use url::Url;

#[derive(Clone)]
struct AppState {
    auth: Arc<KeycloakAuth>,
}

struct KeycloakAuth {
    client: Client,
    issuer: Url,
    audience: String,
    jwks_uri: RwLock<Option<Url>>,
    keys: RwLock<HashMap<String, DecodingKey>>,
}

#[derive(Debug, Deserialize)]
struct OidcConfiguration {
    issuer: String,
    jwks_uri: String,
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
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
struct Claims {
    sub: String,
}

#[derive(Debug, thiserror::Error)]
enum AuthError {
    #[error("missing bearer token")]
    MissingToken,
    #[error("invalid bearer token")]
    InvalidToken,
    #[error("keycloak request failed")]
    Keycloak(#[from] reqwest::Error),
    #[error("invalid keycloak configuration")]
    Configuration,
}

impl KeycloakAuth {
    async fn from_env() -> Result<Self, AuthError> {
        let issuer =
            Url::parse(&env::var("KEYCLOAK_ISSUER").map_err(|_| AuthError::Configuration)?)
                .map_err(|_| AuthError::Configuration)?;
        let audience = env::var("KEYCLOAK_AUDIENCE").map_err(|_| AuthError::Configuration)?;
        let auth = Self {
            client: Client::builder().timeout(Duration::from_secs(5)).build()?,
            issuer,
            audience,
            jwks_uri: RwLock::new(None),
            keys: RwLock::new(HashMap::new()),
        };
        auth.refresh_keys().await?;
        Ok(auth)
    }

    async fn refresh_keys(&self) -> Result<(), AuthError> {
        let configuration_url = Url::parse(&format!(
            "{}/.well-known/openid-configuration",
            self.issuer.as_str().trim_end_matches('/')
        ))
        .map_err(|_| AuthError::Configuration)?;
    
        let configuration: OidcConfiguration = self
            .client
            .get(configuration_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if configuration.issuer.trim_end_matches('/') != self.issuer.as_str().trim_end_matches('/')
        {
            return Err(AuthError::Configuration);
        }
        let jwks_uri = Url::parse(&configuration.jwks_uri).map_err(|_| AuthError::Configuration)?;
        let jwks: Jwks = self
            .client
            .get(jwks_uri.clone())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let keys = jwks
            .keys
            .into_iter()
            .filter(|key| key.kty == "RSA")
            .map(|key| {
                DecodingKey::from_rsa_components(&key.n, &key.e)
                    .map(|decoding_key| (key.kid, decoding_key))
                    .map_err(|_| AuthError::Configuration)
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        *self.jwks_uri.write().await = Some(jwks_uri);
        *self.keys.write().await = keys;
        Ok(())
    }

    async fn validate(&self, token: &str) -> Result<Claims, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        if header.alg != Algorithm::RS256 {
            return Err(AuthError::InvalidToken);
        }
        let kid = header.kid.ok_or(AuthError::InvalidToken)?;
        let mut key = self.keys.read().await.get(&kid).cloned();
        if key.is_none() {
            self.refresh_keys().await?;
            key = self.keys.read().await.get(&kid).cloned();
        }
        let key = key.ok_or(AuthError::InvalidToken)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_audience(&[self.audience.as_str()]);
        decode::<Claims>(token, &key, &validation)
            .map(|token| token.claims)
            .map_err(|_| AuthError::InvalidToken)
    }
}

async fn require_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or(AuthError::MissingToken)?;
    let claims = state.auth.validate(token).await?;
    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
    }
}

#[tokio::main]
async fn main() {
    let exposed_addr: String = env::var("EXPOSED_ADDR").expect("EXPOSED_ADDR must be set");
    let auth = KeycloakAuth::from_env()
        .await
        .expect("Keycloak configuration must be valid and reachable");
    let state = AppState {
        auth: Arc::new(auth),
    };

    let protected = Router::new()
        .route("/me", get(me))
        .layer(middleware::from_fn_with_state(state.clone(), require_auth));
    let app = Router::new()
        .route("/health", get(health))
        .merge(protected)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(exposed_addr).await.unwrap();
    println!("Started...");

    axum::serve(listener, app).await.unwrap();

    println!("Stopped");
}

async fn health() -> &'static str {
    "Backend health ok"
}

async fn me(Extension(claims): Extension<Claims>) -> (StatusCode, axum::Json<Claims>) {
    (StatusCode::OK, axum::Json(claims))
}
