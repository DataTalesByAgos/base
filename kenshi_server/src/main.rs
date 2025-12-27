use kenshi_protocol::Packet;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;
use dashmap::DashMap;
use tracing::{info, error, warn};

type PlayerDb = Arc<DashMap<String, kenshi_protocol::PlayerState>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let addr = "0.0.0.0:5555";
    let listener = TcpListener::bind(addr).await?;
    info!("Kenshi Online Server listening on {}", addr);

    let (tx, _rx) = broadcast::channel(100);
    let players = Arc::new(DashMap::new());

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("Client connected: {}", addr);
        
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let players = players.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, tx, rx, players).await {
                warn!("Connection error: {}", e);
            }
        });
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    tx: broadcast::Sender<Packet>,
    mut rx: broadcast::Receiver<Packet>,
    players: PlayerDb
) -> anyhow::Result<()> {
    let mut username: Option<String> = None;
    let mut buf = [0u8; 4096];

    loop {
        tokio::select! {
            // Read from socket
            n = socket.read(&mut buf) => {
                let n = n?;
                if n == 0 { return Ok(()); }
                
                // Extremely simplified deserialization for demo
                // In production, use length-prefixed framing (tokio-util::codec)
                if let Ok(packet) = serde_json::from_slice::<Packet>(&buf[..n]) {
                    match packet {
                        Packet::LoginRequest { username: u, .. } => {
                            username = Some(u.clone());
                            let response = Packet::LoginResponse { 
                                success: true, 
                                session_token: "test_token".to_string(),
                                reason: None 
                            };
                            let resp_bytes = serde_json::to_vec(&response)?;
                            socket.write_all(&resp_bytes).await?;
                        }
                        Packet::ClientStateUpdate(player_state) => {
                            if let Some(ref u) = username {
                                players.insert(u.clone(), player_state.clone());
                                // Broadcast to others
                                let _ = tx.send(Packet::ClientStateUpdate(player_state));
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Write broadcast to socket
            msg = rx.recv() => {
                if let Ok(packet) = msg {
                     // Don't echo back to sender (optimization omitted for brevity)
                     let bytes = serde_json::to_vec(&packet)?;
                     socket.write_all(&bytes).await?;
                }
            }
        }
    }
}
