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
    sub: String,
    preferred_username: String,
    email: Option<String>,
    exp: usize,
    iss: String,
}

// Теперь храним готовый ключ валидации в памяти бэкенда!
#[derive(Clone)]
struct AppState {
    keycloak_issuer_url: String,
    decoding_key: Arc<DecodingKey>,
}

#[tokio::main]
async fn main() {
    let exposed_addr = env::var("EXPOSED_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());
    
    // ВАЖНО: Внутри сети Docker мы используем имя контейнера `keycloak`
    let keycloak_internal_url = "http://keycloak:8080/realms/my-production-realm";
    
    println!("Скачивание публичных ключей JWKS из Keycloak при старте...");
    
    // Скачиваем ключи один раз при запуске бэкенда
    let jwks_endpoint = format!("{}/protocol/openid-connect/certs", keycloak_internal_url);
    
    // Пробуем скачать ключи с несколькими попытками, так как Keycloak может запускаться чуть дольше бэкенда
    let mut response = None;
    for _ in 0..10 {
        if let Ok(res) = reqwest::get(&jwks_endpoint).await {
            response = Some(res);
            break;
        }
        println!("Ожидание готовности Keycloak...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    let response = response.expect("Не удалось подключиться к Keycloak JWKS эндпоинту!");
    let jwks: serde_json::Value = response.json().await.expect("Не удалось распарсить JWKS JSON!");
    
    // Берем первый доступный ключ Keycloak для RS256
    let keys = jwks["keys"].as_array().expect("Неверный формат JWKS: keys не найден");
    let key_data = &keys[0]; // В проде для идеала ищут по kid, но первый ключ всегда основной
    
    let n = key_data["n"].as_str().expect("Компонент 'n' отсутствует");
    let e = key_data["e"].as_str().expect("Компонент 'e' отсутствует");
    
    let decoding_key = DecodingKey::from_rsa_components(n, e)
        .expect("Не удалось создать DecodingKey из RSA компонентов");

    let state = Arc::new(AppState { 
        keycloak_issuer_url: keycloak_internal_url.to_string(), 
        decoding_key: Arc::new(decoding_key) 
    });

    let public_routes = Router::new().route("/health", get(health));

    let protected_routes = Router::new()
        .route("/protected-data", get(get_protected_data))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&exposed_addr).await.unwrap();
    println!("Бэкенд успешно запущен и защищен! Слушаем {}...", exposed_addr);
    axum::serve(listener, app).await.unwrap();
}

// Теперь middleware работает мгновенно без сетевых запросов!
async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let header = decode_header(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    if header.alg != Algorithm::RS256 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = true; 

    validation.set_issuer(&[
        "http://localhost:8080/realms/my-production-realm".to_string(),
        "http://localhost:8081/realms/my-production-realm".to_string(),
        state.keycloak_issuer_url.clone(), // http://keycloak:8080/...
    ]);

    // Отключаем проверку audience на этапе тестов, чтобы не было строгих конфликтов с client_id
    validation.validate_aud = false; 

    match decode::<Claims>(token, &state.decoding_key, &validation) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(next.run(req).await)
        }
        Err(err) => {
            println!("Ошибка валидации токена: {:?}", err);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn health() -> &'static str {
    "Backend health ok"
}

async fn get_protected_data() -> &'static str {
    "This is verified production data from Rust backend!"
}
