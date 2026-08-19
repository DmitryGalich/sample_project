use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};
use sqlx::PgPool;

// Импортируем типы из нашего модуля прото
use crate::proto::messenger_core_service_server::MessengerCoreService;
use crate::proto::{
    GetHistoryRequest, GetHistoryResponse, Message, 
    SendMessageRequest, SendMessageResponse, StreamRequest,
};

#[derive(Debug)]
pub struct MyMessenger {
    db_pool: PgPool,
}

impl MyMessenger {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }
}

#[tonic::async_trait]
impl MessengerCoreService for MyMessenger {
    type StreamMessagesStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send + 'static>>;

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let req = request.into_inner();
        
        let message_id = uuid::Uuid::new_v4();
        let chat_uuid = uuid::Uuid::parse_str(&req.chat_id)
            .map_err(|_| Status::invalid_argument("Некорректный UUID чата"))?;
        let sender_uuid = uuid::Uuid::parse_str(&req.sender_id)
            .map_err(|_| Status::invalid_argument("Некорректный UUID отправителя"))?;

        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO messages (id, chat_id, sender_id, text, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(message_id)
        .bind(chat_uuid)
        .bind(sender_uuid)
        .bind(req.text.clone()) // <-- ИСПРАВЛЕНО: Клонируем строку для БД
        .bind(now)
        .execute(&self.db_pool)
        .await
        .map_err(|e| Status::internal(format!("Ошибка保存 в БД: {}", e)))?;

        // Теперь на этой строке req.text по-прежнему валидна и доступна!
        let msg = Message {
            id: message_id.to_string(),
            chat_id: req.chat_id,
            sender_id: req.sender_id,
            text: req.text, // <-- Владение строкой уходит в gRPC ответ
            created_at: now.timestamp(),
        };


        Ok(Response::new(SendMessageResponse { message: Some(msg) }))
    }

    async fn get_history(
        &self,
        _request: Request<GetHistoryRequest>,
    ) -> Result<Response<GetHistoryResponse>, Status> {
        // Заглушка истории (допишем при интеграции с БД)
        Ok(Response::new(GetHistoryResponse { messages: vec![] }))
    }

    async fn stream_messages(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamMessagesStream>, Status> {
        let (tx, rx) = mpsc::channel(128);

        tokio::spawn(async move {
            let sample_msg = Message {
                id: "1".into(),
                chat_id: "00000000-0000-0000-0000-000000000001".into(),
                sender_id: "00000000-0000-0000-0000-000000000002".into(),
                text: "Добро пожаловать в чистую архитектуру чата!".into(),
                created_at: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(Ok(sample_msg)).await;
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::StreamMessagesStream))
    }
}
