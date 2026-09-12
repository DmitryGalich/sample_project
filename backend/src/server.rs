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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub exp: u64,
    pub iss: String,
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

// Конфигурация сервера
pub struct ServerConfig {
    pub jwks_url: String,
    pub allowed_issuers: Vec<String>,
    pub allowed_audiences: Vec<String>,
    pub jwks_min_refresh_interval: Duration, // <-- Добавили в структуру
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
    tracing::info!("⏳ Стартовая загрузка публичных ключей из Keycloak...");
    let initial_jwks = fetch_jwks(&config.jwks_url)
        .await
        .expect("Критическая ошибка: Keycloak недоступен при старте.");
    
    tracing::info!("✅ Успешно загружено ключей: {}", initial_jwks.keys.len());

    let shared_state = Arc::new(AppState {
        config,
        jwks: RwLock::new(CachedJwks {
            data: initial_jwks,
            last_updated: Instant::now(),
        }),
    });

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(host_port).await.unwrap();
    tracing::info!("🚀 Боевой Rust-бэкенд успешно запущен на {}", host_port);
    axum::serve(listener, app).await.unwrap();
}

async fn protected_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<String, StatusCode> {
    tracing::debug!("Получен новый запрос на эндпоинт /protected");

    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Отклонено: Отсутствует заголовок Authorization");
            StatusCode::UNAUTHORIZED
        })?;

    if !auth_header.starts_with("Bearer ") {
        tracing::warn!("Отклонено: Заголовок не начинается с 'Bearer '");
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = &auth_header["Bearer ".len()..];

    let header = decode_header(token).map_err(|e| {
        tracing::error!("Отклонено: Не удалось декодировать Header токена: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;
    
    let token_kid = header.kid.ok_or_else(|| {
        tracing::warn!("Отклонено: В токене отсутствует поле 'kid'");
        StatusCode::UNAUTHORIZED
    })?;

    // Шаг B: Поиск ключа в кэше
    let mut found_jwk = {
        let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.data.keys.iter().find(|jwk| jwk.kid == token_kid).cloned()
    };

    // Шаг C: Ротация ключей с динамическим флуд-контролем
    if found_jwk.is_none() {
        {
            let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let elapsed = jwks_guard.last_updated.elapsed();
            
            // Используем динамический интервал времени из конфигурации!
            if elapsed < state.config.jwks_min_refresh_interval {
                tracing::warn!(
                    "🛑 Флуд-контроль заблокировал запрос. Попытка перезапросить фейковый kid='{}' спустя всего {:?}. Минимальный интервал: {:?}", 
                    token_kid, elapsed, state.config.jwks_min_refresh_interval
                );
                return Err(StatusCode::UNAUTHORIZED);
            }
        }

        tracing::info!("⚠️ Ключ с kid='{}' не найден в кэше. Обновляю JWKS...", token_kid);
        let fresh_jwks = fetch_jwks(&state.config.jwks_url).await?;
        
        let mut jwks_guard = state.jwks.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.data = fresh_jwks;
        jwks_guard.last_updated = Instant::now();
        
        found_jwk = jwks_guard.data.keys.iter().find(|jwk| jwk.kid == token_kid).cloned();
        tracing::info!("🔄 Кэш ключей успешно обновлен.");
    }

    let target_jwk = found_jwk.ok_or_else(|| {
        tracing::warn!("Отклонено: Ключ с kid='{}' отсутствует на сервере Keycloak даже после обновления.", token_kid);
        StatusCode::UNAUTHORIZED
    })?;

    // Шаг D: Валидация подписи
    let decoding_key = DecodingKey::from_rsa_components(&target_jwk.n, &target_jwk.e)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&state.config.allowed_issuers);
    validation.set_audience(&state.config.allowed_audiences); 

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| {
            tracing::error!("❌ Криптографическая ошибка или просроченный токен: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    tracing::info!("🔒 Доступ разрешен для пользователя: {}", token_data.claims.preferred_username);
    
    Ok(format!(
        "🔒 Доступ разрешен! Привет, {}",
        token_data.claims.preferred_username
    ))
}
