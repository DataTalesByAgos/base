use std::net::TcpStream;
use std::io::{Read, Write};
use crate::game_state;
use std::thread;
use std::time::Duration;

pub fn connect_and_run() {
    println!("Connecting to server...");
    
    // Retry logic
    let mut stream = loop {
        match TcpStream::connect("127.0.0.1:8080") {
            Ok(s) => {
                println!("Connected to server!");
                break s;
            }
            Err(_) => {
                println!("Failed to connect to server (retrying in 5s)...");
                thread::sleep(Duration::from_secs(5));
            }
        }
    };
    
    // Set timeouts
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

    // Initial handshake or loop start
    let mut buffer = [0u8; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let response = String::from_utf8_lossy(&buffer[..size]);
                // println!("Server: {}", response); // Debug log
                
                unsafe {
                    game_state::set_data(&response);
                    let data_to_send = game_state::get_data();
                    if !data_to_send.is_empty() {
                         if let Err(e) = stream.write_all(data_to_send.as_bytes()) {
                             println!("Failed to send data: {}", e);
                             break;
                         }
                    }
                }
            }
            Ok(_) => {
                println!("Server disconnected.");
                break;
            }
            Err(e) => {
                 println!("Connection error: {}", e);
                 break;
            }
        }
    }
}
