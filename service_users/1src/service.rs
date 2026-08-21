use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};
use sqlx::PgPool;
use redis::AsyncCommands;

use crate::proto::service_users_server::ServiceUsersServer;
use crate::proto::{
    AddUserRequest, AddUserResponse, User, 
};

#[derive(Debug)]
pub struct ServiceUsersObject {
    db_pool: PgPool,
}

impl ServiceUsersObject {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[tonic::async_trait]
impl ServiceUsersObjectImpl for ServiceUsersObject {

}
