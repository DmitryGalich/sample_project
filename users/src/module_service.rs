use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::users_grpc::users_service_server::UsersService;
use crate::users_grpc::{AddUserRequest, AddUserResponse, GetUserRequest, GetUserResponse, User};

#[derive(Debug)]
pub struct MyUsersService {
    db_pool: PgPool,
}

impl MyUsersService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    fn row_to_user(row: PgRow) -> User {
        User {
            id: row.get::<Uuid, _>("id").to_string(),
            email: row.get("email"),
            display_name: row.get("display_name"),
            password_hash: row.get("password_hash"),

            first_name: row.get::<Option<String>, _>("first_name"),
            last_name: row.get::<Option<String>, _>("last_name"),
            avatar_url: row.get::<Option<String>, _>("avatar_url"),
            phone: row.get::<Option<String>, _>("phone"),
            bio: row.get::<Option<String>, _>("bio"),

            user_role: row.get("user_role"),
            is_active: row.get("is_active"),

            created_at: row.get::<DateTime<Utc>, _>("created_at").timestamp(),

            edited_at: row
                .get::<Option<DateTime<Utc>>, _>("edited_at")
                .map(|dt| dt.timestamp()),

            deleted_at: row
                .get::<Option<DateTime<Utc>>, _>("deleted_at")
                .map(|dt| dt.timestamp()),

            last_login_at: row
                .get::<Option<DateTime<Utc>>, _>("last_login_at")
                .map(|dt| dt.timestamp()),
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
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        match row_result {
            Some(row) => Ok(Response::new(GetUserResponse {
                user: Some(Self::row_to_user(row)),
            })),
            None => Err(Status::not_found("Пользователь не найден")),
        }
    }

    async fn add_user(
        &self,
        request: Request<AddUserRequest>,
    ) -> Result<Response<AddUserResponse>, Status> {
        let req = request.into_inner();

        // 1. Хэшируем пароль. DEFAULT_COST (12 раундов) — это баланс скорости и безопасности
        let password_hash = hash(&req.password, DEFAULT_COST)
            .map_err(|e| Status::internal(format!("Ошибка хэширования пароля: {}", e)))?;

        // 2. Валидируем и выставляем роль по умолчанию, если она пустая
        let user_role = if req.user_role.trim().is_empty() {
            "customer".to_string()
        } else {
            req.user_role
        };

        // 3. Выполняем INSERT в базу данных
        let row = sqlx::query(
            r#"
            INSERT INTO users (
                email, display_name, password_hash, 
                first_name, last_name, avatar_url, phone, bio, user_role
            ) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING 
                id, email, display_name, password_hash, 
                first_name, last_name, avatar_url, phone, bio, 
                user_role, is_active, created_at, edited_at, deleted_at, last_login_at
            "#,
        )
        .bind(&req.email)
        .bind(&req.display_name)
        .bind(password_hash)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.avatar_url)
        .bind(&req.phone)
        .bind(&req.bio)
        .bind(user_role)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| {
            // Обрабатываем ошибку уникальности (если такой email уже зарегистрирован)
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return Status::already_exists("Пользователь с таким email уже существует");
                }
            }
            Status::internal(e.to_string())
        })?;

        // 4. Возвращаем созданного пользователя, используя наш готовый row_to_user
        Ok(Response::new(AddUserResponse {
            user: Some(Self::row_to_user(row)),
        }))
    }
}
