use server::ServerConfig;
use std::env;
use std::time::Duration;

mod server;

impl ServerConfig {
    fn from_env() -> Self {
        let jwks_url = env::var("JWKS_URL").expect("Critical: JWKS_URL should be in .env");

        let allowed_realms: Vec<String> = env::var("ALLOWED_REALMS")
            .expect("Critical: ALLOWED_REALMS should be in .env")
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let allowed_audiences: Vec<String> = env::var("ALLOWED_AUDIENCES")
            .expect("Critical: ALLOWED_AUDIENCES should be in .env")
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let refresh_secs_str =
            env::var("JWKS_MIN_REFRESH_INTERVAL_SECS").unwrap_or_else(|_| "60".to_string());

        let refresh_secs: u64 = refresh_secs_str
            .parse()
            .expect("Error: JWKS_MIN_REFRESH_INTERVAL_SECS should be number");

        let jwks_init_max_attempts_str =
            env::var("JWKS_INIT_MAX_ATTEMPTS").unwrap_or_else(|_| "15".to_string());

        let jwks_init_max_attempts: u64 = jwks_init_max_attempts_str
            .parse()
            .expect("Error: JWKS_INIT_MAX_ATTEMPTS should be number");

        let jwks_init_attempts_interval_sec_str =
            env::var("JWKS_INIT_ATTEMPTS_INTERVAL_SEC").unwrap_or_else(|_| "3".to_string());

        let jwks_init_interval_sec: u64 = jwks_init_attempts_interval_sec_str
            .parse()
            .expect("Error: JWKS_INIT_ATTEMPTS_INTERVAL_SEC should be number");

        ServerConfig {
            jwks_url,
            allowed_realms,
            allowed_audiences,
            jwks_min_refresh_interval: Duration::from_secs(refresh_secs),
            jwks_init_max_attempts,
            jwks_init_attempts_interval_sec: Duration::from_secs(jwks_init_interval_sec),
        }


    }
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "rust_backend=info,axum=info".into()),
        )
        .init();

    tracing::info!("Config loading...");

    let config = ServerConfig::from_env();

    tracing::info!("Config loaded");

    let listen_address = env::var("LISTEN_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8000".to_string());

    server::run_server(&listen_address, config).await;
}
