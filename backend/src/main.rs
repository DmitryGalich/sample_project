use std::time::Duration;

mod server;

#[tokio::main]
async fn main() {
    // 1. Инициализируем систему логирования tracing.
    // Она будет читать уровень логов из переменной окружения RUST_LOG.
    // Если переменная не задана, по умолчанию ставим уровень INFO.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_backend=info,axum=info".into()),
        )
        .init();

    tracing::info!("🚀 Инициализация конфигурации приложения...");

    // === ВСЕ ПЕРЕМЕННЫЕ КОНФИГУРАЦИИ ТЕПЕРЬ ТУТ ===
    let listen_address = "0.0.0.0:8000";
    let jwks_url = "http://keycloak:8080/realms/sample_project_realm/protocol/openid-connect/certs".to_string();
    
    let allowed_issuers = vec![
        "http://localhost/realms/sample_project_realm".to_string()
    ];
    
    let allowed_audiences = vec![
        "frontend".to_string(), 
        "account".to_string()
    ];

    // Динамический лимит времени флуд-контроля для ротации ключей
    let jwks_min_refresh_interval = Duration::from_secs(60);
    // ===============================================

    let config = server::ServerConfig {
        jwks_url,
        allowed_issuers,
        allowed_audiences,
        jwks_min_refresh_interval, // Передаем лимит времени в конфиг
    };

    server::run_server(listen_address, config).await;
}
