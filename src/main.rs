mod chain;
use chain::run;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

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
    // 使用tokio-tungstenite库将TCP流升级为WebSocket流
    let mut ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    // let (mut write, read) = ws_stream.split();
    run(&mut ws_stream).await.unwrap();
}
