use std::net::{TcpListener, TcpStream};
use std::io::Write;

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

fn handle_client(_stream: &mut TcpStream) {
    _stream.write(b"+PONG\r\n").unwrap();
}
