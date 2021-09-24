use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use tokio::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::accept_async;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ClientId {
    pub uuid: Uuid,
}

impl ClientId {
    fn new() -> ClientId {
        ClientId {
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug)]
enum ServerNetworkingAction {
    ClientConnected(ClientId, mpsc::Sender<Value>),
    ReceivedMessage(ClientId, Value),
    ClientDisconnected(ClientId),
}

struct ServerNetworkingState {
    clients: HashMap<ClientId, mpsc::Sender<Value>>,
}

impl ServerNetworkingState {
    fn new() -> ServerNetworkingState {
        ServerNetworkingState {
            clients: HashMap::new(),
        }
    }
}

fn to_message<T>(message: &'static str, e: T) -> String
where
    T: Debug,
{
    format!("{}: {:?}", message, e)
}

fn parse_message(message: &Message) -> Result<serde_json::Value, String> {
    message
        .to_text()
        .map_err(|e| to_message("Could not extract text from message", e))
        .and_then(|text| {
            serde_json::from_str(text).map_err(|e| to_message("Could not parse json", e))
        })
}

async fn send_messages_to_client<S>(mut receiver: mpsc::Receiver<Value>, mut socket_sender: S)
where
    S: futures::SinkExt<Message> + Unpin,
    S::Error : Debug
{
    while let Some(value) = receiver.recv().await {
        socket_sender.send(Message::Text(value.to_string())).await.unwrap();
    }
}

async fn accept_connection(tcp_stream: TcpStream, sender: mpsc::Sender<ServerNetworkingAction>) {
    let ws_stream = accept_async(tcp_stream).await.unwrap();

    let client = ClientId::new();
    let (client_sender, client_receiver) = mpsc::channel(10);

    sender
        .send(ServerNetworkingAction::ClientConnected(
            client.clone(),
            client_sender.clone(),
        ))
        .await
        .unwrap();

    let (ws_sender, mut ws_receiver) = ws_stream.split();

    tokio::spawn(send_messages_to_client(client_receiver, ws_sender));

    while let Some(Ok(msg)) = ws_receiver.next().await {
        if msg.is_text() {
            let parse_result = parse_message(&msg);
            if parse_result.is_ok() {
                sender.send(ServerNetworkingAction::ReceivedMessage(
                    client.clone(),
                    parse_result.unwrap(),
                )).await.unwrap();
            }
        }
    }

    sender
        .send(ServerNetworkingAction::ClientDisconnected(client.clone()))
        .await
        .unwrap();
}

async fn handle_events(mut receiver: mpsc::Receiver<ServerNetworkingAction>) {
    let mut state = ServerNetworkingState::new();

    while let Some(action) = receiver.recv().await {
        match action {
            ServerNetworkingAction::ClientConnected(client, sender) => {
                println!("Connected: {:?}", client);
                state.clients.insert(client, sender);
            }
            ServerNetworkingAction::ClientDisconnected(client) => {
                println!("Disconnected: {:?}", client);
                state.clients.remove(&client);
            }
            ServerNetworkingAction::ReceivedMessage(client, message) => {
                println!("{}", message);
                let sender = state.clients.get_mut(&client).unwrap();
                sender.send(message).await.unwrap();
            }
        }
    }
}

pub async fn start<A>(socket_address: A)
where
    A: ToSocketAddrs + Display + Copy,
{
    let tcp_listener = TcpListener::bind(socket_address).await.unwrap();
    println!("Listening on: {}", socket_address);

    let (socket_sender, socket_receiver) = mpsc::channel(100);

    tokio::spawn(handle_events(socket_receiver));

    while let Ok((tcp_stream, _)) = tcp_listener.accept().await {
        tokio::spawn(accept_connection(tcp_stream, socket_sender.clone()));
    }
}
