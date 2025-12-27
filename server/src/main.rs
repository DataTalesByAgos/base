use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on port 8080");

    let (tx, _rx) = broadcast::channel(100);
    
    // Shared state for positions etc.
    // In C++, simpler logic: 
    // plr1, plr2, speed are global.
    // When client sends update, global is updated, and broadcast happens?
    // Actually C++ sends updates to clients inside a loop.
    let state = Arc::new(Mutex::new(ServerState::new()));

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Client connected: {}", addr);
        
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let state = state.clone();

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            let mut buf = [0; 1024];

            // Send initial state
            {
                let s = state.lock().unwrap();
                let initial_data = s.get_sync_string(0); // client ID handling simplified
                if let Err(_) = writer.write_all(initial_data.as_bytes()).await {
                    return;
                }
            }

            loop {
                tokio::select! {
                    result = reader.read(&mut buf) => {
                        match result {
                            Ok(0) => break,
                            Ok(n) => {
                                let msg = String::from_utf8_lossy(&buf[..n]);
                                // Parse and update state
                                match state.lock() {
                                    Ok(mut s) => {
                                        s.update(&msg);
                                        // Broadcast the new state to all (or just echo?)
                                        // C++ logic: "Respond with last message" or custom logic
                                        // "send_to_all_clients"
                                        let sync_msg = s.get_sync_string(1); // dummy id
                                        let _ = tx.send(sync_msg);
                                    }
                                    Err(_) => break,
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    Ok(msg) = rx.recv() => {
                        if let Err(_) = writer.write_all(msg.as_bytes()).await {
                            break;
                        }
                    }
                }
            }
            println!("Client disconnected: {}", addr);
        });
    }
}

pub struct ServerState {
    speed: f32,
    plr1_pos: String,
    plr2_pos: String,
    factions: Vec<&'static str>,
}

impl ServerState {
    fn new() -> Self {
        ServerState {
            speed: 1.0,
            plr1_pos: "-5139.11,158.019,345.631".to_string(),
            plr2_pos: "-5139.11,158.019,345.631".to_string(),
            factions: vec!["204-gamedata.base", "10-multiplayr.mod", "12-multiplayr.mod"],
        }
    }

    fn update(&mut self, msg: &str) {
        let mut key = String::new();
        // Naive parsing mirroring the C++ logic
        // Protocol: "key\nvalue\nkey\nvalue\n"
        for line in msg.lines() {
             if key.is_empty() {
                 key = line.to_string();
                 continue;
             }
             match key.as_str() {
                 "1" => {
                     // Speed
                     if let Ok(v) = line.parse::<f32>() {
                         self.speed = v;
                     }
                 }
                 "2" => {
                     // Player 1
                     if line != "0,0,0" { self.plr1_pos = line.to_string(); }
                 }
                 "3" => {
                     // Player 2
                     if line != "0,0,0" { self.plr2_pos = line.to_string(); }
                 }
                 _ => {}
            }
            key.clear();
        }
    }

    fn get_sync_string(&self, _client_id: usize) -> String {
        // Construct the payload to sync clients
        // Format: "key\nvalue\n..."
        // 0: Faction (logic needed per client ID, but omitting for simple broadcast)
        // 1: Speed
        // 2: P1
        // 3: P2
        format!("1\n{}\n2\n{}\n3\n{}\n", self.speed, self.plr1_pos, self.plr2_pos)
    }
}
