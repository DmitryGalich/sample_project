// src/main.rs
use tonic::transport::Server; // Восстановили импорт gRPC сервера
use sqlx::PgPool;             // Восстановили импорт пула Postgres

// Регистрируем наши подмодули в дереве проекта
pub mod proto;
pub mod service;

// Явно вытаскиваем сгенерированный сервер из модуля proto
use proto::messenger_core_service_server::MessengerCoreServiceServer;
// Явно вытаскиваем нашу структуру из модуля service
use service::MyMessenger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin@database:5432/main_db".to_string());

    println!("Подключение к PostgreSQL...");
    let db_pool = PgPool::connect(&database_url).await?;

    println!("Автоматический запуск миграций базы данных...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;

    let messenger_service = MyMessenger::new(db_pool);

    println!("Rust gRPC Messenger запущен на {}", addr);

    Server::builder()
        .add_service(MessengerCoreServiceServer::new(messenger_service))
        .serve(addr)
        .await?;

    Ok(())
}
