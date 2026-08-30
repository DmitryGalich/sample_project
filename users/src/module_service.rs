use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::users_grpc::users_service_server::UsersService;
use crate::users_grpc::{GetUserRequest, GetUserResponse, User};

#[derive(Debug)]
pub struct MyUsersService {
    db_pool: PgPool,
}

impl MyUsersService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    // Вспомогательная функция для конвертации времени из Chrono в Protobuf Timestamp
    fn to_proto_timestamp(dt: DateTime<Utc>) -> prost_types::Timestamp {
        prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }

    fn row_to_user(row: PgRow) -> User {
        User {
            id: row.get::<Uuid, _>("id").to_string(),
            email: row.get("email"),
            display_name: row.get("display_name"),
            password_hash: row.get("password_hash"),
            
            // Поля со свойством optional в proto файле ожидают Option в Rust
            first_name: row.get::<Option<String>, _>("first_name"),
            last_name: row.get::<Option<String>, _>("last_name"),
            avatar_url: row.get::<Option<String>, _>("avatar_url"),
            phone: row.get::<Option<String>, _>("phone"),
            bio: row.get::<Option<String>, _>("bio"),
            
            user_role: row.get("user_role"),
            is_active: row.get("is_active"),

            // Обязательные даты заворачиваем в Some(), опциональные маппим через .map
            created_at: Some(Self::to_proto_timestamp(row.get::<DateTime<Utc>, _>("created_at"))),
            edited_at: row.get::<Option<DateTime<Utc>>, _>("edited_at").map(Self::to_proto_timestamp),
            deleted_at: row.get::<Option<DateTime<Utc>>, _>("deleted_at").map(Self::to_proto_timestamp),
            last_login_at: row.get::<Option<DateTime<Utc>>, _>("last_login_at").map(Self::to_proto_timestamp),
        }
    }
}

#[tonic::async_trait]
impl UsersService for MyUsersService {
    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        // Парсим UUID из строки запроса. Если формат неверный, возвращаем клиенту ошибку
        let user_id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Неверный формат UUID"))?;

        let row_result = sqlx::query(
            r#"
            SELECT 
                id, email, display_name, password_hash, 
                first_name, last_name, avatar_url, phone, bio, 
                user_role, is_active, created_at, edited_at, deleted_at, last_login_at
            FROM users 
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool) // используем fetch_optional, чтобы мягко обработать отсутствие записи
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        match row_result {
            Some(row) => Ok(Response::new(GetUserResponse {
                user: Some(Self::row_to_user(row)),
            })),
            None => Err(Status::not_found("Пользователь не найден")),
        }
    }
}
