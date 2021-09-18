use std::env;
use futures_util::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async};

async fn accept_connection(tcp_stream: TcpStream) {
    let peer_address = tcp_stream.peer_addr()
        .expect("Connected streams should have a peer address");

    let ws_stream = accept_async(tcp_stream).await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", peer_address);

    let (write, read) = ws_stream.split();

    read
        .try_filter(|msg| future::ready(msg.is_text()))
        .forward(write)
        .await
        .expect("Failed to forward messages")
}

#[tokio::main]
async fn main() {
    let server_address = env::args().nth(1).unwrap_or_else(|| "0.0.0.0:9001".to_string());

    let tcp_listener = TcpListener::bind(&server_address).await
        .expect("Listening TCP failed.");

    println!("Listening on: {}", server_address);

    while let Ok((tcp_stream, _)) = tcp_listener.accept().await {
        tokio::spawn(accept_connection(tcp_stream));
    }
}
