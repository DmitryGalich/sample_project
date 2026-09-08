use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,                // ID пользователя в Keycloak
    preferred_username: String, // Логин пользователя
    email: Option<String>,
    exp: usize,                 // Время протухания токена
    iss: String,                // Кто выпустил токен
}

// Структура для хранения публичного ключа (или JWKS) в состоянии приложения
#[derive(Clone)]
struct AppState {
    // В реальном проде здесь может быть полноценный JWKS-кэш.
    // Для старта мы сохраняем URL или готовый декодирующий ключ.
    keycloak_jwks_url: String,
}

#[tokio::main]
async fn main() {
    let exposed_addr = env::var("EXPOSED_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());
    
    // URL для получения публичных ключей вашего Realm в Keycloak внутри Docker-сети
    let keycloak_jwks_url = "http://keycloak:8080/realms/my-production-realm".to_string();
    
    let state = Arc::new(AppState { keycloak_jwks_url });

    // Публичные маршруты
    let public_routes = Router::new().route("/health", get(health));

    // Защищенные маршруты (прокидываем State в middleware)
    let protected_routes = Router::new()
        .route("/protected-data", get(get_protected_data))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&exposed_addr).await.unwrap();
    println!("Started production-ready server on {}...", exposed_addr);
    axum::serve(listener, app).await.unwrap();
}

// Безопасный Middleware с валидацией подписи Keycloak
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Извлекаем заголовок Authorization
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // 2. Читаем заголовок токена, чтобы узнать kid (Key ID) и алгоритм
    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // В проде Keycloak использует RS256
    if header.alg != Algorithm::RS256 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // 3. ПРОД-РЕШЕНИЕ: Динамически запрашиваем публичный ключ у Keycloak (JWKS)
    // В реальном высоконагруженном проекте этот шаг нужно кэшировать в памяти на пару часов!
    let jwks_endpoint = format!("{}/protocol/openid-connect/certs", state.keycloak_jwks_url);
    let response = reqwest::get(&jwks_endpoint)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let jwks: serde_json::Value = response.json().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Ищем нужный ключ по kid из заголовка JWT
    let target_kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;
    let keys = jwks["keys"].as_array().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let key_data = keys.iter().find(|k| k["kid"].as_str() == Some(&target_kid))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Строим DecodingKey из RSA-компонентов (n и e), переданных Keycloak
    let n = key_data["n"].as_str().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let e = key_data["e"].as_str().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let decoding_key = DecodingKey::from_rsa_components(n, e)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 4. Полноценная безопасная валидация токена
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true; // Проверка протухания
    // На проде обязательно проверяйте эмитент (iss):
    validation.set_issuer(&[state.keycloak_jwks_url.clone()]);
    // Если настраивали Audience в Keycloak, раскомментируйте:
    // validation.set_audience(&["backend-client"]); 

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => {
            // Токен валиден, подпись проверена через асимметричный ключ Keycloak!
            req.extensions_mut().insert(token_data.claims);
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

async fn health() -> &'static str {
    "Backend health ok"
}

async fn get_protected_data() -> &'static str {
    "This is verified production data from Rust backend!"
}
