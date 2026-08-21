pub mod proto;
pub mod service;

#[tokio::main]
async fn main() {

    let addr = "0.0.0.0:50051".parse()?;

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://admin:admin@database:5432/main_db".to_string());

    let db_pool = PgPool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://cache:6379".to_string());
    let redis_client = redis::Client::open(redis_url)?;

    let messenger_service = MyMessenger::new(db_pool, redis_client);

    Server::builder()
        .add_service(MessengerCoreServiceServer::new(messenger_service))
        .serve(addr)
        .await?;

    Ok(())
}
