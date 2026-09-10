#[derive(Debug, serde::Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: String,
    preferred_username: String,
    email: Option<String>,
    exp: u64,
}

#[derive(Debug, serde::Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

struct AppState {
    jwks: Jwks,
}

#[tokio::main]
async fn main() {
    let jwks_url: &str =
        "http://keycloak:8080/realms/sample_project_realm/protocol/openid-connect/certs";
    println!("Loading public keys from {}", jwks_url);

    let jwks: Jwks = reqwest::get(jwks_url)
        .await
        .expect("Connecting error")
        .json()
        .await
        .expect("Parsing error JWKS");
    println!("Keys downloaded: {}", jwks.keys.len());

    let shared_state = std::sync::Arc::new(AppState { jwks });

    let app = axum::Router::new()
        .route("/health", axum::routing::get(protected_handler))
        .route("/protected", axum::routing::get(protected_handler))
        .with_state(shared_state);

    let url: &str = "0.0.0.0:8000";

    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    println!("Started on {}", url);

    axum::serve(listener, app).await.unwrap();
}

async fn health_handler(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<String, axum::http::StatusCode> {
        Ok(format!(
        "Backend health"
    ))
}

async fn protected_handler(
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<String, axum::http::StatusCode> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header["Bearer ".len()..];

    // Шаг A: Декодируем Header токена без проверки подписи, чтобы узнать `kid`
    let header =
        jsonwebtoken::decode_header(token).map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;
    let token_kid = header.kid.ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Шаг B: Ищем ключ с таким же `kid` в нашем кэше (AppState)
    let target_jwk = state
        .jwks
        .keys
        .iter()
        .find(|jwk| jwk.kid == token_kid)
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    // Шаг C: Создаем криптографический ключ из компонентов RSA (n и e)
    let decoding_key = jsonwebtoken::DecodingKey::from_rsa_components(&target_jwk.n, &target_jwk.e)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    // Шаг D: Настраиваем правила валидации.
    // Keycloak по умолчанию использует алгоритм RS256.
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    // Для простоты на данном этапе отключаем строгую проверку Audience (её разберем на Этапе 12)
    validation.validate_aud = false;

    // Шаг E: Производим математическую верификацию подписи токена!
    let token_data =
        jsonwebtoken::decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
            println!("Validation error: {:?}", e);
            axum::http::StatusCode::UNAUTHORIZED
        })?;

    println!("{} (ID: {})", token_data.claims.preferred_username, token_data.claims.sub);

    // Если мы дошли сюда — подпись верна, токен не изменен, время жизни проверено!
    Ok(format!(
        "Welcome, {} (ID: {})",
        token_data.claims.preferred_username, token_data.claims.sub
    ))
}
