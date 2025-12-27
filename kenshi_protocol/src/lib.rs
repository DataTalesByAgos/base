use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Packet {
    // Auth
    LoginRequest { username: String, password_hash: String },
    LoginResponse { success: bool, session_token: String, reason: Option<String> },

    // Game State
    ClientStateUpdate(PlayerState),
    FullWorldUpdate { players: Vec<PlayerState>, world: WorldState },

    // Chat
    ChatMessage { sender: String, content: String, is_system: bool },
    
    // Connection
    Ping(u64),
    Pong(u64),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerState {
    pub id: u32,
    pub username: String,
    pub position: Vector3,
    pub rotation: Vector4,
    pub health: f32,
    pub max_health: f32,
    pub is_in_combat: bool,
    pub faction_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldState {
    pub game_time: f32,
    pub weather_intensity: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
