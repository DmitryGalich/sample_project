use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::projects_grpc::projects_service_server::ProjectsService;
use crate::projects_grpc::{
    CreateProjectRequest, CreateProjectResponse, GetProjectRequest, GetProjectResponse, Project,
    UpdateProjectRequest, UpdateProjectResponse,
};

#[derive(Debug)]
pub struct MyProjectsService {
    db_pool: PgPool,
}

impl MyProjectsService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    fn row_to_project(row: &PgRow) -> Project {
        let member_uuids: Vec<Uuid> = row.try_get("team_members").unwrap_or_default();
        let team_members = member_uuids.into_iter().map(|id| id.to_string()).collect();

        Project {
            id: row.get::<Uuid, _>("id").to_string(),
            owner_id: row.get::<Uuid, _>("owner_id").to_string(),
            title: row.get("title"),
            description: row.get::<Option<String>, _>("description"),

            deadline: row
                .get::<Option<DateTime<Utc>>, _>("deadline")
                .map(|dt| dt.timestamp()),

            created_at: row.get::<DateTime<Utc>, _>("created_at").timestamp(),

            edited_at: row
                .get::<Option<DateTime<Utc>>, _>("edited_at")
                .map(|dt| dt.timestamp()),

            deleted_at: row
                .get::<Option<DateTime<Utc>>, _>("deleted_at")
                .map(|dt| dt.timestamp()),

            status: row.get("status"),

            team_members,
        }
    }
}

#[tonic::async_trait]
impl ProjectsService for MyProjectsService {
    async fn get_project(
        &self,
        request: Request<GetProjectRequest>,
    ) -> Result<Response<GetProjectResponse>, Status> {
        let req = request.into_inner();

        // 1. Валидация входного UUID проекта
        let project_id = Uuid::from_str(&req.id)
            .map_err(|_| Status::invalid_argument("Неверный формат ID проекта (ожидался UUID)"))?;

        // 2. Выполняем запрос с агрегацией участников в массив
        let row_result = sqlx::query(
            r#"
            SELECT 
                p.id, 
                p.owner_id, 
                p.title, 
                p.description, 
                p.created_at, 
                p.deadline, 
                p.edited_at, 
                p.deleted_at, 
                p.status,
                -- Собираем все ID участников в один массив, убирая NULL, если участников нет
                COALESCE(
                    ARRAY_AGG(ptm.user_id) FILTER (WHERE ptm.user_id IS NOT NULL), 
                    '{}'
                ) as team_members
            FROM projects p
            LEFT JOIN project_team_members ptm ON p.id = ptm.project_id
            WHERE p.id = $1 AND p.deleted_at IS NULL -- Проверка на Soft Delete (не удален ли)
            GROUP BY p.id
            "#,
        )
        .bind(project_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Ошибка базы данных: {}", e)))?;

        // 3. Возвращаем результат или gRPC ошибку NOT_FOUND
        match row_result {
            Some(row) => Ok(Response::new(GetProjectResponse {
                project: Some(Self::row_to_project(&row)),
            })),
            None => Err(Status::not_found("Проект не найден или был удален")),
        }
    }

    async fn create_project(
        &self,
        request: Request<CreateProjectRequest>,
    ) -> Result<Response<CreateProjectResponse>, Status> {
        let req = request.into_inner();

        // 1. Валидация UUID владельца
        let owner_uuid = Uuid::from_str(&req.owner_id)
            .map_err(|_| Status::invalid_argument("Неверный формат owner_id (ожидался UUID)"))?;

        // Валидация UUID участников команды
        let mut team_member_uuids = Vec::new();
        for member_id in &req.team_members {
            let member_uuid = Uuid::from_str(member_id).map_err(|_| {
                Status::invalid_argument(format!("Неверный формат ID участника: {}", member_id))
            })?;
            team_member_uuids.push(member_uuid);
        }

        // Парсинг опционального дедлайна из i64 (Unix timestamp) в DateTime<Utc>
        let deadline_dt = req
            .deadline
            .and_then(|ts| DateTime::from_timestamp(ts, 0).map(|dt| dt.with_timezone(&Utc)));

        // Устанавливаем дефолтный статус, если клиент прислал пустую строку
        let status = if req.status.trim().is_empty() {
            "active".to_string()
        } else {
            req.status
        };

        // 2. Открываем транзакцию в Postgres
        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| Status::internal(format!("Ошибка базы данных: {}", e)))?;

        // 3. Вставляем проект в таблицу `projects`
        // СУБД сама сгенерирует id (через uuid_generate_v4) и created_at (через NOW())
        let project_row = sqlx::query(
            r#"
            INSERT INTO projects (owner_id, title, description, deadline, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, owner_id, title, description, created_at, deadline, edited_at, deleted_at, status
            "#,
        )
        .bind(owner_uuid)
        .bind(&req.title)
        .bind(&req.description) // Option<String> отлично биндится в nullable поле
        .bind(deadline_dt)      // Option<DateTime<Utc>> биндится в nullable timestamp
        .bind(&status)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| Status::internal(format!("Не удалось сохранить проект: {}", e)))?;

        let project_id = project_row.get::<Uuid, _>("id");

        // 4. Вставляем участников в таблицу `project_team_members`
        for member_uuid in team_member_uuids {
            sqlx::query(
                r#"
                INSERT INTO project_team_members (project_id, user_id)
                VALUES ($1, $2)
                "#,
            )
            .bind(project_id)
            .bind(member_uuid)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                Status::internal(format!("Не удалось добавить участника команды: {}", e))
            })?;
        }

        // Подтверждаем транзакцию
        tx.commit()
            .await
            .map_err(|e| Status::internal(format!("Не удалось подтвердить транзакцию: {}", e)))?;

        // 5. Формируем финальный PgRow для маппинга
        // Чтобы функция row_to_project сработала корректно, ей нужен PgRow с агрегированным полем team_members.
        // Делаем быстрый SELECT уже созданного проекта (это гарантирует актуальность всех дефолтных полей из БД)
        let final_row = sqlx::query(
            r#"
            SELECT 
                p.*,
                COALESCE(ARRAY_AGG(ptm.user_id) FILTER (WHERE ptm.user_id IS NOT NULL), '{}') as team_members
            FROM projects p
            LEFT JOIN project_team_members ptm ON p.id = ptm.project_id
            WHERE p.id = $1
            GROUP BY p.id
            "#,
        )
        .bind(project_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Ошибка получения созданного проекта: {}", e)))?;

        // 6. Возвращаем gRPC ответ
        Ok(Response::new(CreateProjectResponse {
            project: Some(Self::row_to_project(&final_row)),
        }))
    }

    async fn update_project(
        &self,
        request: Request<UpdateProjectRequest>,
    ) -> Result<Response<UpdateProjectResponse>, Status> {
        let req = request.into_inner();

        // 1. Валидация основного ID проекта
        let project_id = Uuid::from_str(&req.id)
            .map_err(|_| Status::invalid_argument("Неверный формат ID проекта (ожидался UUID)"))?;

        // 2. Валидация опционального owner_id (если передан)
        let owner_uuid = if let Some(ref oid) = req.owner_id {
            Some(Uuid::from_str(oid).map_err(|_| {
                Status::invalid_argument("Неверный формат owner_id (ожидался UUID)")
            })?)
        } else {
            None
        };

        // 3. Парсинг опционального дедлайна из i64 (Unix timestamp) в DateTime<Utc>
        let deadline_dt = req
            .deadline
            .and_then(|ts| DateTime::from_timestamp(ts, 0).map(|dt| dt.with_timezone(&Utc)));

        // 4. Открываем транзакцию
        let mut tx = self
            .db_pool
            .begin()
            .await
            .map_err(|e| Status::internal(format!("Ошибка базы данных: {}", e)))?;

        // 5. Динамическое обновление проекта с помощью COALESCE
        // Обновляем edited_at на текущее время (NOW()) только если проект существует и не удален
        let update_result = sqlx::query(
            r#"
            UPDATE projects
            SET 
                owner_id = COALESCE($1, owner_id),
                title = COALESCE($2, title),
                description = COALESCE($3, description),
                deadline = COALESCE($4, deadline),
                status = COALESCE($5, status),
                edited_at = NOW()
            WHERE id = $6 AND deleted_at IS NULL
            "#,
        )
        .bind(owner_uuid)
        .bind(&req.title)
        .bind(&req.description)
        .bind(deadline_dt)
        .bind(&req.status)
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| Status::internal(format!("Не удалось обновить проект: {}", e)))?;

        // Если ни одна строка не была затронута, значит проект не найден или мягко удален
        if update_result.rows_affected() == 0 {
            return Err(Status::not_found("Проект не найден или был удален"));
        }

        // Подтверждаем транзакцию
        tx.commit()
            .await
            .map_err(|e| Status::internal(format!("Не удалось подтвердить транзакцию: {}", e)))?;

        // 6. Получаем финальное состояние проекта с агрегацией участников команды
        let final_row = sqlx::query(
            r#"
            SELECT 
                p.id, 
                p.owner_id, 
                p.title, 
                p.description, 
                p.created_at, 
                p.deadline, 
                p.edited_at, 
                p.deleted_at, 
                p.status,
                COALESCE(
                    ARRAY_AGG(ptm.user_id) FILTER (WHERE ptm.user_id IS NOT NULL), 
                    '{}'
                ) as team_members
            FROM projects p
            LEFT JOIN project_team_members ptm ON p.id = ptm.project_id
            WHERE p.id = $1
            GROUP BY p.id
            "#,
        )
        .bind(project_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Ошибка получения обновленного проекта: {}", e)))?;

        // 7. Возвращаем gRPC ответ
        Ok(Response::new(UpdateProjectResponse {
            project: Some(Self::row_to_project(&final_row)),
        }))
    }
}
