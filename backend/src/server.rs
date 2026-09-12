use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    routing::get,
    Router,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

// Ожидаемый Payload (Claims) токена
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub preferred_username: String,
    pub email: Option<String>,
    pub exp: u64,
    pub iss: String,
}

// Структуры для JWKS
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

// Конфигурация, которая передается в состояние сервера
pub struct ServerConfig {
    pub jwks_url: String,
    pub allowed_issuers: Vec<String>,
    pub allowed_audiences: Vec<String>,
}

// Внутреннее состояние приложения (кэш ключей + копия конфига)
struct AppState {
    config: ServerConfig,
    jwks: RwLock<Jwks>,
}

// Вспомогательная асинхронная функция скачивания ключей
async fn fetch_jwks(url: &str) -> Result<Jwks, StatusCode> {
    reqwest::get(url)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<Jwks>()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// Главная точка входа для запуска сервера, вызываемая из main.rs
pub async fn run_server(host_port: &str, config: ServerConfig) {
    println!("⏳ Стартовая загрузка публичных ключей из Keycloak...");
    let initial_jwks = fetch_jwks(&config.jwks_url)
        .await
        .expect("Критическая ошибка: Keycloak недоступен при старте.");

    println!("✅ Успешно загружено ключей: {}", initial_jwks.keys.len());

    let shared_state = Arc::new(AppState {
        config,
        jwks: RwLock::new(initial_jwks),
    });

    // Маршрутизация (согласованная с Nginx)
    let app = Router::new()
        .route("/protected", get(protected_handler))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(host_port).await.unwrap();
    println!("🚀 Rust-бэкенд успешно запущен на {}", host_port);
    axum::serve(listener, app).await.unwrap();
}

// Обработчик защищенного эндпоинта
async fn protected_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<String, StatusCode> {
    // 1. Извлекаем токен из заголовка
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let token = &auth_header["Bearer ".len()..];

    // 2. Декодируем Header токена, чтобы узнать `kid`
    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token_kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;

    // 3. Поиск ключа в кэше (режим чтения)
    let mut found_jwk = {
        let jwks_guard = state
            .jwks
            .read()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard
            .keys
            .iter()
            .find(|jwk| jwk.kid == token_kid)
            .cloned()
    };

    // 4. Ротация ключей при промахе кэша (Cache Miss)
    if found_jwk.is_none() {
        println!("⚠️ Ключ с kid='{}' не найден. Обновляю JWKS...", token_kid);
        let fresh_jwks = fetch_jwks(&state.config.jwks_url).await?;

        let mut jwks_guard = state
            .jwks
            .write()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        *jwks_guard = fresh_jwks;

        found_jwk = jwks_guard
            .keys
            .iter()
            .find(|jwk| jwk.kid == token_kid)
            .cloned();
        println!("🔄 Кэш ключей успешно обновлен.");
    }

    let target_jwk = found_jwk.ok_or(StatusCode::UNAUTHORIZED)?;

    // 5. Криптографическая валидация
    let decoding_key = DecodingKey::from_rsa_components(&target_jwk.n, &target_jwk.e)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut validation = Validation::new(Algorithm::RS256);

    // Передаем динамические настройки из нашего конфига
    validation.set_issuer(&state.config.allowed_issuers);
    validation.set_audience(&state.config.allowed_audiences);

    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        println!("❌ Ошибка валидации подписи или клейм: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    Ok(format!(
        "🔒 Доступ разрешен! Все проверки пройдены. Привет, {}",
        token_data.claims.preferred_username
    ))
}
