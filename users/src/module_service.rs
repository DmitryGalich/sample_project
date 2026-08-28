use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::Row;
use sqlx::postgres::PgRow;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::users_grpc::users_service_server::UsersService;
use crate::users_grpc::{
    AddUserRequest, AddUserResponse, GetAllUsersRequest, GetAllUsersResponse, GetUserRequest,
    GetUserResponse, User,
};

#[derive(Debug)]
pub struct MyUsersService {
    db_pool: PgPool,
}

impl MyUsersService {
    // Исправили имя
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    fn row_to_user(row: PgRow) -> User {
        User {
            id: row.get::<Uuid, _>("id").to_string(),
            email: row.get("email"),
            display_name: row.get("display_name"),
            created_at: row.get::<DateTime<Utc>, _>("created_at").to_rfc3339(),
        }
    }
}

#[tonic::async_trait]
impl UsersService for MyUsersService {
    async fn add_user(
        &self,
        request: Request<AddUserRequest>,
    ) -> Result<Response<AddUserResponse>, Status> {
        let req = request.into_inner();

        let row = sqlx::query(
            r#"
            INSERT INTO users (email, display_name) 
            VALUES ($1, $2)
            RETURNING id, email, display_name, created_at
            "#,
        )
        .bind(&req.email)
        .bind(&req.display_name)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AddUserResponse {
            user: Some(Self::row_to_user(row)),
        }))
    }


}
