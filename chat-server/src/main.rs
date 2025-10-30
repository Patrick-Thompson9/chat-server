use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  cargo run -- server");
        eprintln!("  cargo run -- client [server_ip]");
        return;
    }

    match args[1].as_str() {
        "server" => {
            if let Err(e) = server::start_server().await {
                eprintln!("Error starting server: {}", e)
            }
        }
        "client" => {
            let server_ip = if args.len() > 2 {
                Some(args[2].as_str())
            } else {
                None
            };
            
            if let Err(e) = client::start_client(server_ip).await {
                eprintln!("Error starting client: {}", e)
            }
        }
        _ => {
            eprintln!("Invalid argument. Use 'server' or 'client'");
        }
    }
}

mod server;
mod client;