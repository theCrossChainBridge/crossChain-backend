mod chain;
use chain::run;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    // 监听端口9000
    let listener = TcpListener::bind("127.0.0.1:9000").await.unwrap();
    println!("WebSocket server running on ws://127.0.0.1:9000");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: TcpStream) {
    // upgrade TCP to WebSocket
    let mut ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    println!("{}", msg);
                    run(&mut ws_stream, msg.to_string()).await.unwrap();
                }
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
}
