use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex}
};

use tokio_tungstenite::{accept_async, tungstenite::Message};
use std::{collections::HashMap, sync::Arc, net::SocketAddr, error::Error};
use::futures::{
    stream::StreamExt,
    SinkExt
};

type Sender = mpsc::UnboundedSender<Message>;

#[derive(Clone)]
struct Connection {
    sender: Sender,
    id: i32,
}

struct ChannelManager {
    channels: Arc<Mutex<HashMap<String,Vec<Connection>>>>,
    id_count: Arc<Mutex<i32>>
}

impl ChannelManager {
    fn new() -> Self {
        ChannelManager { 
            channels: Arc::new(Mutex::new(HashMap::new())), 
            id_count: Arc::new(Mutex::new(0)) 
        }
    }

    async fn get_or_create_channel(&self, channel_name: String) {
        let mut channels = self.channels.lock().await;
        if !channels.contains_key(&channel_name) {
            channels.insert(channel_name.clone(), Vec::new());
        }
    }

    async fn add_sender_to_channel(&self, channel_name: String, connection: Connection) {
        let mut channels = self.channels.lock().await;
        if let Some(connections) = channels.get_mut(&channel_name) {
            connections.push(connection);
        }
    }

    async fn remove_sender_from_channel(&self, channel_name: String, connection: Connection) {
        let mut channels = self.channels.lock().await;
        if let Some(connections) = channels.get_mut(&channel_name) {
            connections.retain(|c| c.id != connection.id);
        }
    }

    async fn broadcast(&self, channel_name: String, message: Message, who_sent: Connection) {
        let mut channels = self.channels.lock().await;
        if let Some(connections) = channels.get_mut(&channel_name) {
            for connection in connections {
                if connection.id == who_sent.id {
                    continue;
                }
                
                connection.sender.send(message.clone()).expect("Failed to send msg");
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    channel_manager: Arc<ChannelManager>
) {
    let socket = accept_async(stream).await.expect("Error during web socket handshake");
    println!("New Web Socket connected at addr: {}", addr);

    let (mut write, mut read) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    let channel_manager = channel_manager.clone();
    
    let connection = {
        let mut id_count = channel_manager.id_count.lock().await;
        let connection = Connection{id: *id_count, sender: tx};
        *id_count += 1;
        connection
    };
    
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            write.send(msg).await.expect("Failed to send message");
        }
    });

    let mut current_channel = String::new();
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if text.starts_with("JOIN_ROOM:") {
                    let room_name = &text[10..];
                    channel_manager.get_or_create_channel(room_name.to_string()).await;
                    println!("line check: {}", connection.id);
                    channel_manager.add_sender_to_channel(room_name.to_string(), connection.clone()).await;
                    current_channel = room_name.to_string();
                } 
                else if text.starts_with("LEAVE_ROOM:") {
                    let room_name = &text[11..];
                    channel_manager.remove_sender_from_channel(room_name.to_string(), connection.clone()).await;
                    current_channel.clear();
                } 
                else if text.starts_with("ROOM_MSG:") {
                    let parts: Vec<&str> = text[9..].splitn(3, ':').collect();
                    if parts.len() == 3 {
                        let room_name = parts[0];
                        let username = parts[1];
                        let message = parts[2];
                        channel_manager.broadcast(room_name.to_string(), Message::Text(format!("{}: {}", username, message)), connection.clone()).await;
                    }
                }
            }
            Ok(_) => {
                // handle other msg types
            }
            Err(e) => eprint!("Error reading command: {}", e)
        }
    }

    println!("Web socket connection closed (Addr: {})", addr);
}

pub async fn start_server() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Listening on port 8080");

    let channel_manager = Arc::new(ChannelManager::new());

    while let Ok((stream, addr)) = listener.accept().await {
        let channel_manager = channel_manager.clone();
        tokio::spawn(async move {
            handle_connection(stream, addr, channel_manager).await
        });
    }
    Ok(())
}