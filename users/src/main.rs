use anyhow::Context;
use std::env;
use tonic::transport::Server;

pub mod users_grpc {
    tonic::include_proto!("users");
}

mod module_service;

use module_service::MyUsersService;
use users_grpc::users_service_server::UsersServiceServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exposed_addr = env::var("EXPOSED_ADDR")
        .context("Not found env var EXPOSED_ADDR")?
        .parse()
        .context("Not parsed env var EXPOSED_ADDR")?;
    let database_url: String = env::var("DATABASE_URL1")
        .context("Not found env var EXPOSED_ADDR")?
        .parse()
        .context("Not parsed env var DATABASE_URL1")?;

    println!("Address: {}", exposed_addr);
    println!("Db url: {}", database_url);
    println!("Run...");

    Server::builder()
        .add_service(UsersServiceServer::new(MyUsersService::default()))
        .serve(exposed_addr)
        .await?;

    Ok(())
}
