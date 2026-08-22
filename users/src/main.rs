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
    let exposed_addr = env::var("EXPOSED_ADDR")?.parse()?;
    println!("gRPC Users Service running on {}...", exposed_addr);

    Server::builder()
        .add_service(UsersServiceServer::new(MyUsersService::default()))
        .serve(exposed_addr)
        .await?;

    Ok(())
}
