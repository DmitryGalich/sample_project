use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{transport::Server, Request, Response, Status};

// Импортируем сгенерированный из proto код
pub mod proto {
    tonic::include_proto!("messenger");
}

use proto::messenger_core_service_server::{MessengerCoreService, MessengerCoreServiceServer};
use proto::{GetHistoryRequest, GetHistoryResponse, Message, SendMessageRequest, SendMessageResponse, StreamRequest};

#[derive(Debug, Default)]
pub struct MyMessenger {}

#[tonic::async_trait]
impl MessengerCoreService for MyMessenger {
    type StreamMessagesStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send + 'static>>;

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let req = request.into_inner();
        
        // В будущем: логика сохранения в PostgreSQL и публикация в Redis Pub/Sub
        let msg = Message {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: req.chat_id,
            sender_id: req.sender_id,
            text: req.text,
            created_at: chrono::Utc::now().timestamp(),
        };

        Ok(Response::new(SendMessageResponse { message: Some(msg) }))
    }

    async fn get_history(
        &self,
        request: Request<GetHistoryRequest>,
    ) -> Result<Response<GetHistoryResponse>, Status> {
        // Заглушка истории
        Ok(Response::new(GetHistoryResponse { messages: vec![] }))
    }

    async fn stream_messages(
        &self,
        _request: Request<StreamRequest>,
    ) -> Result<Response<Self::StreamMessagesStream>, Status> {
        // Создаем канал для real-time отправки сообщений клиенту
        let (tx, rx) = mpsc::channel(128);

        // В реальной жизни здесь будет подписка на Redis Pub/Sub и пересылка в tx
        tokio::spawn(async move {
            let sample_msg = Message {
                id: "1".into(),
                chat_id: "chat_1".into(),
                sender_id: "system".into(),
                text: "Добро пожаловать в чат мастеров!".into(),
                created_at: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(Ok(sample_msg)).await;
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::StreamMessagesStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    let messenger_service = MyMessenger::default();

    println!("Rust gRPC Messenger запущен на {}", addr);

    Server::builder()
        .add_service(MessengerCoreServiceServer::new(messenger_service))
        .serve(addr)
        .await?;

    Ok(())
}
