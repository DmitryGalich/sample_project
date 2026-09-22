use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
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

// === НАШ НОВЫЙ MIDDLEWARE ===
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut request: Request<axum::body::Body>, // Принимаем исходный HTTP-запрос
    next: Next,                            // Ссылка на следующий шаг в цепочке
) -> Result<Response, StatusCode> {
    
    // 1. Извлекаем заголовок
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = &auth_header["Bearer ".len()..];

    // 2. Декодируем Header токена, узнаем kid
    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token_kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;

    // 3. Ищем ключ в кэше
    let mut found_jwk = {
        let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.data.keys.iter().find(|jwk| jwk.kid == token_kid).cloned()
    };

    // 4. Ротация при промахе кэша
    if found_jwk.is_none() {
        {
            let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if jwks_guard.last_updated.elapsed() < state.config.jwks_min_refresh_interval {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
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

    // 5. Криптографическая проверка подписи
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // НАСТОЯЩАЯ МАГИЯ RUST: Внедряем Claims внутрь запроса как расширение (Extension).
    // Теперь любой обработчик, идущий далее, сможет вытащить эти данные.
    request.extensions_mut().insert(token_data.claims);

    // Передаем управление дальше по цепочке к самому обработчику
    Ok(next.run(request).await)
}

pub async fn run_server(host_port: &str, config: ServerConfig) {
    tracing::info!("Initial loading keys from Keycloak...");
    let mut initial_jwks = None;
    let mut attempts = 0;
    let max_attempts = 15; // Сделаем 15 попыток

    while attempts < max_attempts {
        match fetch_jwks(&config.jwks_url).await {
            Ok(jwks) => {
                initial_jwks = Some(jwks);
                break;
            }
            Err(_) => {
                attempts += 1;
                tracing::warn!(
                    "⚠️ Keycloak еще не готов (Попытка {}/{}). Ждем 3 секунды...", 
                    attempts, max_attempts
                );
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }

    // Если после всех попыток Keycloak так и не ответил — только тогда паникуем
    let initial_jwks = initial_jwks.expect("❌ Критическая ошибка: Не удалось подключиться к Keycloak после серии попыток.");
    tracing::info!("✅ Успешно загружено ключей: {}", initial_jwks.keys.len());

    
    let shared_state = Arc::new(AppState {
        config,
        jwks: RwLock::new(CachedJwks {
            data: initial_jwks,
            last_updated: Instant::now(),
        }),
    });

    // 1. Создаем защищенную группу роутов
    let protected_routes = Router::new()
        .route("/protected", get(protected_handler))
        .route("/admin/dashboard", get(admin_handler))
        // Обертываем всю эту группу в наш Middleware авторизации!
        .route_layer(axum::middleware::from_fn_with_state(
            shared_state.clone(),
            auth_middleware,
        ));

    // 2. Создаем общую таблицу маршрутов (включая публичные)
    let app = Router::new()
        // Публичный эндпоинт для проверки здоровья (без Middleware)
        .route("/health", get(health_handler)) 
        // Мержим защищенные роуты
        .merge(protected_routes)
        // Раздаем глобальный стейт
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(host_port).await.unwrap();
    tracing::info!("Started on {}", host_port);
    axum::serve(listener, app).await.unwrap();
}

// --- ТЕПЕРЬ НАШИ ОБРАБОТЧИКИ КРИСТАЛЬНО ЧИСТЫЕ ---

// Публичный обработчик (доступен всем без токенов)
async fn health_handler() -> String {
    "OK".to_string()
}

// Защищенный обработчик
async fn protected_handler(
    // Axum автоматически достает Claims, которые положил Middleware
    axum::Extension(claims): axum::Extension<Claims>,
) -> String {
    tracing::info!("Access granted for: {}", claims.preferred_username);
    format!("Access granted! Hello from Middleware, {}", claims.preferred_username)
}

// Административный обработчик
async fn admin_handler(
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<String, StatusCode> {
    // Проверяем роль админа
    let has_admin_role = claims.realm_access
        .as_ref()
        .map(|access| access.roles.contains(&"admin".to_string()))
        .unwrap_or(false);

    if !has_admin_role {
        tracing::warn!("Denied: User {} has no right 'admin'", claims.preferred_username);
        return Err(StatusCode::FORBIDDEN);
    }

    tracing::info!("Access granted: {}", claims.preferred_username);
    Ok(format!("Welcome, admin {}!", claims.preferred_username))
}
