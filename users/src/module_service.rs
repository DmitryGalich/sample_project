use tonic::{Request, Response, Status};

use crate::users_grpc::users_service_server::UsersService;
use crate::users_grpc::{
    AddUserRequest, AddUserResponse, 
    GetAllUsersRequest, GetAllUsersResponse, // 🌟 Теперь это будет работать!
    GetUserRequest, GetUserResponse, User
};

#[derive(Debug, Default)]
pub struct MyUsersService {}

#[tonic::async_trait]
impl UsersService for MyUsersService {
    // 1. Добавление пользователя
    async fn add_user(&self, request: Request<AddUserRequest>) -> Result<Response<AddUserResponse>, Status> {
        let req = request.into_inner();
        
        let new_user = User {
            id: "generated-uuid-123".to_string(),
            email: req.email,
            display_name: req.display_name,
            created_at: "2026-08-22T23:00:00Z".to_string(),
        };

        Ok(Response::new(AddUserResponse { user: Some(new_user) }))
    }

    // 2. Получение пользователя по ID
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = User {
            id: req.id,
            email: "test@example.com".to_string(),
            display_name: "John Doe".to_string(),
            created_at: "2026-08-22T23:00:00Z".to_string(),
        };

        Ok(Response::new(GetUserResponse { user: Some(user) }))
    }

    // 3. Получение всех пользователей с учетом лимита
    async fn get_all_users(&self, request: Request<GetAllUsersRequest>) -> Result<Response<GetAllUsersResponse>, Status> {
        // Извлекаем пришедший лимит из запроса
        let req = request.into_inner();
        let limit = req.limit as usize;

        // Наш фейковый список пользователей
        let mut all_users = vec![
            User {
                id: "1".to_string(),
                email: "alice@example.com".to_string(),
                display_name: "Alice".to_string(),
                created_at: "2026-08-22T23:00:00Z".to_string(),
            },
            User {
                id: "2".to_string(),
                email: "bob@example.com".to_string(),
                display_name: "Bob".to_string(),
                created_at: "2026-08-22T23:00:00Z".to_string(),
            },
            User {
                id: "3".to_string(),
                email: "charlie@example.com".to_string(),
                display_name: "Charlie".to_string(),
                created_at: "2026-08-22T23:00:00Z".to_string(),
            },
        ];

        // Если limit передан (больше 0), обрезаем массив до нужного количества
        if limit > 0 {
            all_users.truncate(limit);
        }

        Ok(Response::new(GetAllUsersResponse {
            users: all_users,
        }))
    }
}
