use crate::server::ServerNetworkingAction::*;
use crate::server::{self, ClientId, ServerNetworkingAction};
use futures::StreamExt;
use serde_json::Value;
use std::fmt::{Debug, Display};
use tokio::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

async fn send_messages_to_client<S>(mut receiver: mpsc::Receiver<Value>, mut socket_sender: S)
where
    S: futures::SinkExt<Message> + Unpin,
    S::Error: Debug,
{
    while let Some(value) = receiver.recv().await {
        socket_sender
            .send(Message::Text(value.to_string()))
            .await
            .unwrap();
    }
}

async fn accept_connection(tcp_stream: TcpStream, sender: mpsc::Sender<ServerNetworkingAction>) {
    let ws_stream = accept_async(tcp_stream).await.unwrap();

    let client = ClientId::new();
    let (client_sender, client_receiver) = mpsc::channel(10);

    sender
        .send(ClientConnected(client, client_sender.clone()))
        .await
        .unwrap();

    let (ws_sender, mut ws_receiver) = ws_stream.split();

    tokio::spawn(send_messages_to_client(client_receiver, ws_sender));

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if msg.is_text() {
            let text = msg.to_text().unwrap();
            let value = serde_json::from_str(text).unwrap();
            sender.send(ReceivedMessage(client, value)).await.unwrap();
        }
    }

    sender.send(ClientDisconnected(client)).await.unwrap();
}

pub async fn start<A>(socket_address: A)
where
    A: ToSocketAddrs + Display + Copy,
{
    let tcp_listener = TcpListener::bind(socket_address).await.unwrap();
    println!("Listening on: {}", socket_address);

    let (socket_sender, socket_receiver) = mpsc::channel(100);

    tokio::spawn(server::handle_events(socket_receiver));

    while let Ok((tcp_stream, _)) = tcp_listener.accept().await {
        tokio::spawn(accept_connection(tcp_stream, socket_sender.clone()));
    }
}
