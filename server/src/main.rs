mod cli;

use futures_util::{StreamExt, stream::TryStreamExt, future};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async};
use std::net::{IpAddr, SocketAddr};
use cli::{ServerArgs};
use tokio_tungstenite::tungstenite::Message;
use std::fmt::Debug;

fn to_message<T>(message: &'static str, e: T) -> String
    where T: Debug {
    format!("{}: {:?}", message, e)
}

fn parse_message(message: Message) -> Result<serde_json::Value, String> {
    message
        .to_text()
        .map_err(|e| to_message("Could not extract text from message", e))
        .and_then(|text| serde_json::from_str(text)
            .map_err(|e| to_message("Could not parse json", e)))
}

async fn accept_connection(tcp_stream: TcpStream) {
    let peer_address = tcp_stream.peer_addr()
        .expect("Connected streams should have a peer address");

    let ws_stream = accept_async(tcp_stream).await
        .expect("Error during the websocket handshake occurred");

    println!("Connected: {}", peer_address);

    let (_, reader) = ws_stream.split();

    reader
        .try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .try_for_each(|msg| {
            let value = parse_message(msg);
            println!("{:?}", value);
            future::ok(())
        })
        .await
        .expect("Could not read messages");

    println!("Disconnected: {}", peer_address);
}

#[tokio::main]
async fn main() {
    let args = ServerArgs::parse_program_args();
    let socket_address = SocketAddr::new(IpAddr::V4(args.host), args.port);
    let tcp_listener = TcpListener::bind(socket_address).await
        .expect("Listening TCP failed.");

    println!("Listening on: {}", socket_address);

    while let Ok((tcp_stream, _)) = tcp_listener.accept().await {
        tokio::spawn(accept_connection(tcp_stream));
    }
}
