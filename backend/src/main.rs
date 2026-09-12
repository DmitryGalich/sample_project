use axum::{
    http::{header::AUTHORIZATION, StatusCode},
    routing::get,
    Router,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    preferred_username: String,
    email: Option<String>,
    exp: u64,
    iss: String, 
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

// 1. Теперь JWKS внутри состояния обернут в RwLock (аналог std::shared_mutex в C++)
struct AppState {
    jwks_url: &'static str,
    jwks: RwLock<Jwks>,
}

// Вспомогательная функция для скачивания ключей с Keycloak
async fn fetch_jwks(url: &str) -> Result<Jwks, StatusCode> {
    reqwest::get(url)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<Jwks>()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[tokio::main]
async fn main() {
    let jwks_url = "http://keycloak:8080/realms/sample_project_realm/protocol/openid-connect/certs";
    
    println!("⏳ Стартовая загрузка публичных ключей из Keycloak...");
    let initial_jwks = fetch_jwks(jwks_url).await.expect("Критическая ошибка: Keycloak недоступен при старте.");
    println!("✅ Успешно загружено ключей: {}", initial_jwks.keys.len());

    // Создаем общее состояние, доступное всем потокам веб-сервера
    let shared_state = Arc::new(AppState {
        jwks_url,
        jwks: RwLock::new(initial_jwks),
    });

    let app = Router::new()
        .route("/protected", get(protected_handler))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("🚀 Rust-бэкенд готов к ротации ключей на порту 8000");
    axum::serve(listener, app).await.unwrap();
}

async fn protected_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
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

    // Шаг А: Быстро декодируем Header токена, чтобы узнать `kid`
    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let token_kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;

    // Шаг B: Пытаемся найти ключ в кэше в режиме READ (параллельный shared_lock)
    let mut found_jwk = {
        let jwks_guard = state.jwks.read().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        jwks_guard.keys.iter().find(|jwk| jwk.kid == token_kid).cloned()
    };

    // Шаг C: ИНТЕНСИВНАЯ РОТАЦИЯ (Cache Miss)
    // Если ключа в памяти нет, значит в Keycloak могла пройти ротация!
    if found_jwk.is_none() {
        println!("⚠️ Ключ с kid='{}' не найден в кэше. Пробую обновить JWKS...", token_kid);
        
        // Скачиваем свежие ключи из Keycloak "вживую"
        let fresh_jwks = fetch_jwks(state.jwks_url).await?;
        
        // Блокируем кэш на ЗАПИСЬ (эксклюзивный unique_lock), чтобы обновить данные
        let mut jwks_guard = state.jwks.write().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        *jwks_guard = fresh_jwks; // Перезаписываем старый кэш новым списком
        
        // Ищем ключ в уже обновленном списке
        found_jwk = jwks_guard.keys.iter().find(|jwk| jwk.kid == token_kid).cloned();
        println!("🔄 Кэш ключей успешно обновлен.");
    }

    // Если ключа нет даже после обновления — токен точно фейковый или от другого реалма
    let target_jwk = found_jwk.ok_or(StatusCode::UNAUTHORIZED)?;

    // Шаг D: Математическая валидация подписи (как на Прошлом Этапе)
    let decoding_key = DecodingKey::from_rsa_components(&target_jwk.n, &target_jwk.e)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&["frontend", "account"]); // Указываем, какие client_id имеют право обращаться к бэкенду
    validation.set_issuer(&["http://localhost/realms/sample_project_realm"]);
    

    // Печатаем ТОЧНЫЙ issuer, который прилетел в токене
    // validation.set_issuer(&[
    //     "http://localhost:8080/realms/sample_project_realm",
    //     "http://localhost/auth/realms/sample_project_realm",
    //     "http://keycloak:8080/realms/sample_project_realm"
    // ]);


    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| {
            println!("❌ Ошибка валидации подписи: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    Ok(format!(
        "🔒 Доступ разрешен по протоколу ротации! Привет, {}",
        token_data.claims.preferred_username
    ))
}
