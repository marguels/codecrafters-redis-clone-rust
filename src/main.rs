mod redis_commands;
mod resp_parser;
mod storage;

use anyhow::{anyhow, Result};
use redis_commands::Command;
use resp_parser::{RESPParser, RESPType};
use std::{sync::Arc, time::Duration};
use storage::Storage;
use tokio::{
    io::{AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use clap::{arg, Parser};

type ThreadSafeStorage = Arc<RwLock<Storage>>;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "6379")]
    port: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Args::parse();
	let address = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(&address).await?;
    let storage = Arc::new(RwLock::new(storage::Storage::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        let mut storage = storage.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, &mut storage).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(
    stream: TcpStream,
    storage: &mut ThreadSafeStorage,
) -> Result<(), anyhow::Error> {
    let (rx, tx) = stream.into_split();

    let reader = BufReader::new(rx);
    let mut writer = BufWriter::new(tx);

	let mut parser = RESPParser::new(reader);

	loop { 
		let resp_type = parser.parse().await?;
		if let Some(resp_type) = &resp_type {
			println!("Received: {:?}", resp_type.serialize());
		} else {
			return Err(anyhow!("Invalid RESPType"));
		}
	
		let command = match resp_type {
			Some(resp_type) => Command::from_resp(resp_type.clone()),
			None => return Err(anyhow!("Invalid RESPType")),
		};
		match command {
			Command::Ping => {
				let response = RESPType::SimpleString("PONG".to_string()).serialize();
				println!("Sending: {:?}", response);
				writer.write(response.as_bytes()).await?;
				writer.flush().await?;
			}
			Command::Echo(message) => {
				let response = RESPType::BulkString(message).serialize();
				println!("Sending: {:?}", response);
				writer.write(response.as_bytes()).await?;
				writer.flush().await?;
			}
			Command::Get(key) => {
				let storage = storage.read().await;
				let value = storage.get(key);
				match value {
					Some(value) => {
						let response = RESPType::BulkString(value.to_string()).serialize();
						println!("Sending: {:?}", response);
						writer.write(response.as_bytes()).await?;
						writer.flush().await?;
					}
					None => {
						let response = &RESPType::NullBulkString.serialize();
						writer
							.write(response.as_bytes())
							.await?;
						println!("Sending: {:?}", response);
						writer.flush().await?;
					}
				}
			}
			Command::Set(key, value, expiry) => {
				let mut storage = storage.write().await;
				storage.set(key, value, expiry.map(Duration::from_millis));
				writer.write_all(b"+OK\r\n").await?;
				writer.flush().await?;
			}
			Command::Info => {
				let response = RESPType::BulkString("role:master".to_string()).serialize();
				println!("Sending: {:?}", response);
				writer.write(response.as_bytes()).await?;
				writer.flush().await?;
			}
			Command::Unknown => {}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use tokio::io::{AsyncReadExt, AsyncWriteExt};
	use tokio::net::{TcpListener, TcpStream};

	async fn simulate_client(addr: std::net::SocketAddr, message: &[u8] ) -> anyhow::Result<()> {
		let mut stream = TcpStream::connect(addr).await?;
	
		stream.write_all(message).await?;
		let mut buffer = [0; 1024];
		let n = stream.read(&mut buffer).await?;
	
		assert!(n > 0, "No response received from the server.");
	
		Ok(())
	}

	#[tokio::test]
	async fn test_handle_multiple_connections() -> anyhow::Result<()> {
		let listener = TcpListener::bind("127.0.0.1:0").await?;
		let addr = listener.local_addr()?;

		let storage = Arc::new(RwLock::new(Storage::new()));

		let server_task = tokio::spawn(async move {
			while let Ok((stream, _)) = listener.accept().await {
				let mut storage_clone = storage.clone();
				tokio::spawn(async move {
					if let Err(e) = handle_client(stream, &mut storage_clone).await {
						eprintln!("Error handling client: {}", e);
					}
				});
			}
		});

		let client_task1 = tokio::spawn(simulate_client(addr, b"*1\r\n$4\r\nPING\r\n"));
		let client_task2 = tokio::spawn(simulate_client(addr, b"*1\r\n$4\r\nPING\r\n"));

		let _ = tokio::try_join!(client_task1, client_task2)?;

		server_task.abort();

		Ok(())
	}
		
}
