use tokio::{
	io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
	net::{TcpListener, TcpStream},
	net::tcp::OwnedReadHalf,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let listener = TcpListener::bind("127.0.0.1:6379").await?;

	loop {
		let (stream, _) = listener.accept().await?;
		tokio::spawn(async move {
			if let Err(e) = handle_client(stream).await {
				eprintln!("Error handling client: {}", e);
			}
		});
	}

}

async fn handle_client(stream: TcpStream) -> Result<(), std::io::Error> {
		let (rx, tx) = stream.into_split();

		let mut reader = BufReader::new(rx);
		let mut writer = BufWriter::new(tx);

		loop {
			let command = read_line(&mut reader).await?;
			if command.is_empty() {
				break;
			}

			let command = command.trim();
			
			match command {
				"ping" => {
					writer.write(b"+PONG\r\n").await?;
					writer.flush().await?;					
				}
				"echo" => {
					let prefix_size = read_line(&mut reader).await?;
					let message = read_line(&mut reader).await?;
					let response = format!("{}\r\n{}", prefix_size.trim(), message);
					writer.write(response.as_bytes()).await?;
					writer.flush().await?;
				}
				_ => println!("Unrecognized command: {}", command)
			}
		}

		Ok(())
	}

	async fn read_line(stream: &mut BufReader<OwnedReadHalf>) -> Result<String, std::io::Error> {
		let mut line = String::new();
		match stream.read_line(&mut line).await {
			Ok(_) => Ok(line),
			Err(e) => {
				eprintln!("Error reading line: {}", e);
				Err(e)
			}
		}
	}
