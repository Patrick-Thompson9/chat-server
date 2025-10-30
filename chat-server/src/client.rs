use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{stream::StreamExt, SinkExt};
use std::{error::Error, io::{self, Write}, sync::Arc};
use colored::*;
use std::time::Instant;

pub async fn start_client(server_ip: Option<&str>) -> Result<(), Box<dyn Error>> {
    let server_address = server_ip.unwrap_or("127.0.0.1");
    let connection_string = format!("ws://{}:8080", server_address);
    
    let (stream, response) = connect_async(&connection_string).await?;
    println!("Connected to WebSocket server at {}", server_address);

    let (mut write, mut read) = stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let message = Arc::new(Mutex::new(Vec::new()));

    tokio::spawn({
        let message = message.clone();
        async move {
            while let Some(msg) = rx.recv().await {
                write.send(msg).await.expect("Failed to send message to server");
            }
        }
    });
    
    tokio::spawn({
        let message = message.clone();
        async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        let mut msgs = message.lock().await;

                        if !text.starts_with("ROOM_MSG") {
                            msgs.push(text.clone());
                            println!("{}", text);
                        }
                    }
                    Ok(_) => {
                        // handle other message types
                    }
                    Err(e) => eprintln!("Error reading message: {}", e)
                }
            }
        }
    });

    println!("Enter your username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();
    if username.to_lowercase().contains("neil") {
        println!("{}", "Real Neil: Nice try...".magenta().bold());
        println!("{}", "Real Neil: Get back to work!".magenta().bold());
        println!("Disconnected from server");
        return Ok(())
    }

    let mut current_room = String::new();
    let mut in_room = false;
    'out_room: loop {
        println!("Do you want to create/join a room with JOIN <room name>");
        println!("> ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.starts_with("JOIN ") {
            current_room = input[5..].to_string();
            tx.send(Message::Text(format!("JOIN_ROOM:{}", current_room))).expect("Failed to join room");
            println!("{}{}", "Joined room: ".green().bold(), current_room);
            
            let msgs = message.lock().await;
            println!("--- Previous Messages ---");
            for msg in msgs.iter() {
                println!("{}", msg);
            }
            println!("-----------------------------------------------");

            in_room = true;
        } else {
            println!("Invalid command. Please type 'JOIN <room name>'.")
        }

        if in_room {
            let chat_timer = Instant::now();
            println!("{}{}", "You can now chat in room: ".green().bold(), current_room);
            'in_room: loop {
                println!("{} > ", username);
                io::stdout().flush()?;
                let mut message = String::new();
                io::stdin().read_line(&mut message)?;
                let message = message.trim();
        
                if message == "/leave" {
                    tx.send(Message::Text(format!("LEAVE_ROOM: {}", current_room))).expect("Failed to send message");
                    println!("{}{}", "You have left room: ".red().bold(), current_room);
                    break 'in_room;
                }
        
                tx.send(Message::Text(format!("ROOM_MSG:{}:{}:{}", current_room, username, message))).
                expect("Failed to send message");
        
                if chat_timer.elapsed().as_secs() > 30 {
                    println!("{}", "Neil: Get back to work!".magenta().bold());
                    println!("Disconnected from chat room: {}", current_room);
                    break 'in_room;
                }
        }
    }
    if input == "QUIT" {
        println!("{}", "Quitting app".red().bold());
        break 'out_room;
    }
    
    }


    Ok(())
}


