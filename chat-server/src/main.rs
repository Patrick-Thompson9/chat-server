use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <server or client>");
        return;
    }

    match args[1].as_str() {
        "server" => {
            if let Err(e) = server::start_server().await {
                eprintln!("Error starting server: {}", e)
            }
        }
        "client" => {
            if let Err(e) = client::start_client().await {
                eprintln!("Error starting client: {}", e)
            }
        }
        _ => {
            eprintln!("Invalid argument");
        }
    }
}

mod server;
mod client;