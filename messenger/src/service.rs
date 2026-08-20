use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};
use sqlx::PgPool;
use redis::AsyncCommands;

use crate::proto::messenger_core_service_server::MessengerCoreService;
use crate::proto::{
    GetHistoryRequest, GetHistoryResponse, Message, 
    SendMessageRequest, SendMessageResponse, StreamRequest,
};

#[derive(Debug)]
pub struct MyMessenger {
    db_pool: PgPool,
    redis_client: redis::Client, 
}

impl MyMessenger {
    pub fn new(db_pool: PgPool, redis_client: redis::Client) -> Self {
        Self { db_pool, redis_client }
    }
}

#[tonic::async_trait]
impl MessengerCoreService for MyMessenger {
    type StreamMessagesStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send + 'static>>;

    // async fn send_message(
    //     &self,
    //     request: Request<SendMessageRequest>,
    // ) -> Result<Response<SendMessageResponse>, Status> {
    //     let req = request.into_inner();
        
    //     let message_id = uuid::Uuid::new_v4();
    //     let chat_uuid = uuid::Uuid::parse_str(&req.chat_id)
    //         .map_err(|_| Status::invalid_argument("Некорректный UUID чата"))?;
    //     let sender_uuid = uuid::Uuid::parse_str(&req.sender_id)
    //         .map_err(|_| Status::invalid_argument("Некорректный UUID отправителя"))?;

    //     let now = chrono::Utc::now();

    //     // Запись в Postgres
    //     sqlx::query(
    //         r#"
    //         INSERT INTO messages (id, chat_id, sender_id, text, created_at)
    //         VALUES ($1, $2, $3, $4, $5)
    //         "#
    //     )
    //     .bind(message_id)
    //     .bind(chat_uuid)
    //     .bind(sender_uuid)
    //     .bind(req.text.clone())
    //     .bind(now)
    //     .execute(&self.db_pool)
    //     .await
    //     .map_err(|e| Status::internal(format!("Ошибка сохранения в БД: {}", e)))?;

    //     let msg = Message {
    //         id: message_id.to_string(),
    //         chat_id: req.chat_id.clone(),
    //         sender_id: req.sender_id,
    //         text: req.text,
    //         created_at: now.timestamp(),
    //     };

    //     let payload = format!("{}:{}", msg.sender_id, msg.text);

    //     // Публикуем в Redis Pub/Sub без блокировки основного метода
    //     if let Ok(mut redis_conn) = self.redis_client.get_multiplexed_tokio_connection().await {
    //         let _: Result<i64, _> = redis_conn.publish(&req.chat_id, payload).await;
    //     }

    //     Ok(Response::new(SendMessageResponse { message: Some(msg) }))
    // }


    // // 2. ПОЛУЧЕНИЕ ИСТОРИИ
    // async fn get_history(
    //     &self,
    //     request: Request<GetHistoryRequest>,
    // ) -> Result<Response<GetHistoryResponse>, Status> {
    //     let req = request.into_inner();

    //     let chat_uuid = uuid::Uuid::parse_str(&req.chat_id)
    //         .map_err(|_| Status::invalid_argument("Некорректный UUID чата"))?;

    //     let limit = if req.limit <= 0 { 50 } else { req.limit } as i64;

    //     let rows = sqlx::query!(
    //         r#"
    //         SELECT id, chat_id, sender_id, text, created_at 
    //         FROM messages 
    //         WHERE chat_id = $1 
    //         ORDER BY created_at DESC 
    //         LIMIT $2
    //         "#,
    //         chat_uuid,
    //         limit
    //     )
    //     .fetch_all(&self.db_pool)
    //     .await
    //     .map_err(|e| Status::internal(format!("Ошибка чтения из БД: {}", e)))?;

    //     let messages = rows
    //         .into_iter()
    //         .map(|row| Message {
    //             id: row.id.to_string(),
    //             // chat_id — это Option<Uuid>, тут unwrap нужен:
    //             chat_id: row.chat_id.unwrap_or_default().to_string(), 
                
    //             // ИСПРАВЛЕНО: sender_id — это чистый Uuid, unwrap НЕ НУЖЕН, сразу в строку:
    //             sender_id: row.sender_id.to_string(), 
                
    //             text: row.text,
    //             created_at: row.created_at.timestamp(),
    //         })
    //         .collect();

    //     Ok(Response::new(GetHistoryResponse { messages }))
    // }

    // // 3. REAL-TIME СТРИМИНГ
    // async fn stream_messages(
    //     &self,
    //     request: Request<StreamRequest>,
    // ) -> Result<Response<Self::StreamMessagesStream>, Status> {
    //     let req = request.into_inner();
        
    //     // ИСПРАВЛЕНО: Используем user_id, так как в StreamRequest у нас именно он
    //     let listen_channel = req.user_id; 

    //     let pubsub_client = self.redis_client.get_async_pubsub().await
    //         .map_err(|e| Status::internal(format!("Ошибка Redis Pub/Sub: {}", e)))?;
        
    //     let (tx, rx) = mpsc::channel(128);

    //     tokio::spawn(async move {
    //         let mut pubsub = pubsub_client;
            
    //         if pubsub.subscribe(&listen_channel).await.is_err() {
    //             return;
    //         }

    //         let mut pubsub_stream = pubsub.on_message();

    //         while let Some(msg) = pubsub_stream.next().await {
    //             if let Ok(payload_str) = msg.get_payload::<String>() {
    //                 // Разделяем нашу простую строку обратно на автора и текст
    //                 let parts: Vec<&str> = payload_str.splitn(2, ':').collect();
    //                 if parts.len() == 2 {
    //                     let grpc_msg = Message {
    //                         id: uuid::Uuid::new_v4().to_string(),
    //                         chat_id: listen_channel.clone(),
    //                         sender_id: parts[0].to_string(),
    //                         text: parts[1].to_string(),
    //                         created_at: chrono::Utc::now().timestamp(),
    //                     };

    //                     if tx.send(Ok(grpc_msg)).await.is_err() {
    //                         break; 
    //                     }
    //                 }
    //             }
    //         }
    //     });

    //     let output_stream = ReceiverStream::new(rx);
    //     Ok(Response::new(Box::pin(output_stream) as Self::StreamMessagesStream))
    // }
}
