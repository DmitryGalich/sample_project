pub mod proto;
pub mod service;

use proto::messenger_core_service_server::MessengerCoreServiceServer;
use service::MyMessenger;

use sqlx::PgPool;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin@database:5432/main_db".to_string());

    println!("Подключение к PostgreSQL...");
    let db_pool = PgPool::connect(&database_url).await?;

    println!("Автоматический запуск миграций базы данных...");
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    println!("Подключение к Redis...");
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://cache:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;

    // Передаем и пул БД, и клиент Redis
    let messenger_service = MyMessenger::new(db_pool, redis_client);
    println!("Rust gRPC Messenger запущен на {}", addr);

    Server::builder()
        .add_service(MessengerCoreServiceServer::new(messenger_service))
        .serve(addr)
        .await?;

    Ok(())
}
