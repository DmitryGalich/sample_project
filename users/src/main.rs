use std::env;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exposed_addr = env::var("EXPOSED_ADDR")?;
 
    let listener = TcpListener::bind(exposed_addr.clone()).await?;
    println!("Raw TCP/HTTP Server running on {exposed_addr}...");

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            
            if let Ok(n) = socket.read(&mut buffer).await {
                if n == 0 { return; }

                let response = "HTTP/1.1 200 OK\r\nContent-Length: 22\r\nContent-Type: text/plain\r\n\r\nHello from raw Tokio1!";
                
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });
    }
}
