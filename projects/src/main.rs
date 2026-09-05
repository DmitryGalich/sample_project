use anyhow::Context;
use sqlx::PgPool;
use std::env;
use tonic::transport::Server;

pub mod projects_grpc {
    include!("generated/projects.rs");
}

mod module_service;

use module_service::MyProjectsService;
use projects_grpc::projects_service_server::ProjectsServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exposed_addr = env::var("EXPOSED_ADDR")
        .context("Not found env var EXPOSED_ADDR")?
        .parse()
        .context("Not parsed env var EXPOSED_ADDR")?;
    println!("EXPOSED_ADDR: {}", exposed_addr);

    let database_url: String = env::var("DATABASE_URL")
        .context("Not found env var DATABASE_URL")?
        .parse()
        .context("Not parsed env var DATABASE_URL")?;
    println!("Database configured");

    let db_pool = PgPool::connect(&database_url)
        .await
        .context("Not connected to database")?;
    
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .context("Error while database migration")?;

    println!("Run...");

    Server::builder()
        .add_service(ProjectsServiceServer::new(MyProjectsService::new(db_pool)))
        .serve(exposed_addr)
        .await?;

    Ok(())
}
