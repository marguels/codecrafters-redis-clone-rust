use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read};

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                println!("accepted new connection");
                handle_client(&mut _stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(stream: &mut TcpStream) {
    let mut command = [0u8; 1024];
    while match stream.read(&mut command) {
        Ok(_) => {
            match stream.write("+PONG\r\n".as_bytes()) {
                Ok(_) => {
                    println!("Response sent");
                    true
                }
                Err(e) => {
                    println!("Failed to send response: {}", e);
                    false
                }
            }
        }
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            false
        }
    } {}
}
