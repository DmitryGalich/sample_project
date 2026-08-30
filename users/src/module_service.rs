use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::QueryBuilder;
use sqlx::{PgPool, Row};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::users_grpc::users_service_server::UsersService;
use crate::users_grpc::{
    AddUserRequest, AddUserResponse, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    GetUserResponse, UpdateUserRequest, UpdateUserResponse, User,
};

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

        let password_hash = hash(&req.password, DEFAULT_COST)
            .map_err(|e| Status::internal(format!("Ошибка хэширования пароля: {}", e)))?;

        let user_role = if req.user_role.trim().is_empty() {
            "customer".to_string()
        } else {
            req.user_role
        };

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
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return Status::already_exists("Пользователь с таким email уже существует");
                }
            }
            Status::internal(e.to_string())
        })?;

        Ok(Response::new(AddUserResponse {
            user: Some(Self::row_to_user(row)),
        }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();

        // 1. Валидируем UUID пользователя
        let user_id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Неверный формат UUID"))?;

        // Сначала проверяем, прислал ли клиент хоть что-нибудь
        if req.email.is_none()
            && req.display_name.is_none()
            && req.password.is_none()
            && req.is_active.is_none()
            && req.first_name.is_none()
            && req.last_name.is_none()
            && req.avatar_url.is_none()
            && req.phone.is_none()
            && req.bio.is_none()
            && req.user_role.is_none()
        {
            return Err(Status::invalid_argument("Не указаны поля для обновления"));
        }

        // 2. Создаем чистый QueryBuilder без автоматических разделителей
        let mut query_builder = QueryBuilder::new("UPDATE users SET ");
        let mut need_comma = false;

        // 3. Динамически добавляем только те поля, которые пришли
        if let Some(ref email) = req.email {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("email = ").push_bind(email);
            need_comma = true;
        }
        if let Some(ref display_name) = req.display_name {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder
                .push("display_name = ")
                .push_bind(display_name);
            need_comma = true;
        }
        if let Some(ref password) = req.password {
            let hash = hash(password, DEFAULT_COST)
                .map_err(|e| Status::internal(format!("Ошибка хэширования пароля: {}", e)))?;
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("password_hash = ").push_bind(hash);
            need_comma = true;
        }
        if let Some(is_active) = req.is_active {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("is_active = ").push_bind(is_active);
            need_comma = true;
        }
        if let Some(ref first_name) = req.first_name {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("first_name = ").push_bind(first_name);
            need_comma = true;
        }
        if let Some(ref last_name) = req.last_name {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("last_name = ").push_bind(last_name);
            need_comma = true;
        }
        if let Some(ref avatar_url) = req.avatar_url {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("avatar_url = ").push_bind(avatar_url);
            need_comma = true;
        }
        if let Some(ref phone) = req.phone {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("phone = ").push_bind(phone);
            need_comma = true;
        }
        if let Some(ref bio) = req.bio {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("bio = ").push_bind(bio);
            need_comma = true;
        }
        if let Some(ref user_role) = req.user_role {
            if need_comma {
                query_builder.push(", ");
            }
            query_builder.push("user_role = ").push_bind(user_role);
            need_comma = true;
        }

        // Всегда обновляем дату редактирования профиля
        if need_comma {
            query_builder.push(", ");
        }
        query_builder.push("edited_at = NOW()");

        // 4. Завершаем запрос условием WHERE и блоком RETURNING
        query_builder.push(" WHERE id = ").push_bind(user_id);
        query_builder.push(
            r#"
        RETURNING 
            id, email, display_name, password_hash, 
            first_name, last_name, avatar_url, phone, bio, 
            user_role, is_active, created_at, edited_at, deleted_at, last_login_at
        "#,
        );

        // 5. Выполняем собранный запрос в БД
        let row_result = query_builder
            .build()
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| {
                if let Some(db_err) = e.as_database_error() {
                    if db_err.is_unique_violation() {
                        return Status::already_exists("Этот email уже занят другим пользователем");
                    }
                }
                Status::internal(e.to_string())
            })?;

        // 6. Возвращаем результат или ошибку 404
        match row_result {
            Some(row) => Ok(Response::new(UpdateUserResponse {
                user: Some(Self::row_to_user(row)),
            })),
            None => Err(Status::not_found("Пользователь не найден")),
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();

        // 1. Валидируем UUID пользователя
        let user_id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Неверный формат UUID"))?;

        // 2. Выполняем Soft Delete запрос в базу данных
        // Выставляем флаг активности в false и фиксируем время удаления NOW()
        // (Опциональное поле reason вы можете сохранить в отдельную таблицу логов, если потребуется)
        let row_result = sqlx::query(
            r#"
            UPDATE users 
            SET 
                is_active = false,
                deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, deleted_at
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        // 3. Формируем ответ клиенту
        match row_result {
            Some(row) => {
                let id: Uuid = row.get("id");
                // Достаем записанную дату и сразу конвертируем в Unix Timestamp (секунды)
                let deleted_at_dt: DateTime<Utc> = row.get("deleted_at");
                let deleted_at_ts = deleted_at_dt.timestamp();

                Ok(Response::new(DeleteUserResponse {
                    id: id.to_string(),
                    success: true,
                    deleted_at: deleted_at_ts,
                }))
            }
            // Если пользователь не найден или ОН УЖЕ был удален ранее
            None => Err(Status::not_found("Пользователь не найден или уже удален")),
        }
    }
}
