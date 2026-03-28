//! WebTransport Server for Unity FFI POC
//!
//! Demonstrates receiving structs from Unity via WebTransport FFI.
//! Uses self-signed certificates for development.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::mem;
use std::ptr;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;
use wtransport::Endpoint;
use wtransport::ServerConfig;
use wtransport::Identity;
use wtransport::Connection;

// Import shared types from unity-network
// In production, these would be in a shared crate
use unity_network::{GameState, PacketHeader, PacketType, PlayerPos, SpriteMessage, sprite_manager::SpriteManager};
use unity_network::SpriteOp;

/// Safely read a struct from a potentially unaligned byte buffer
/// This prevents panics when the buffer address is not properly aligned
fn read_struct_unaligned<T: Copy>(data: &[u8]) -> Result<T> {
    if data.len() < mem::size_of::<T>() {
        anyhow::bail!("Buffer too small: got {}, need {}", data.len(), mem::size_of::<T>());
    }

    unsafe {
        Ok(ptr::read_unaligned(data.as_ptr() as *const T))
    }
}

type ConnectionMap = Arc<RwLock<HashMap<Uuid, Connection>>>;

pub struct GameServer {
    connections: ConnectionMap,
    sprite_manager: Arc<RwLock<SpriteManager>>,
}

/// Circle motion parameters
const CIRCLE_RADIUS: f32 = 5.0;
const CIRCLE_SPEED: f32 = 2.0; // radians per second
const BROADCAST_INTERVAL_MS: u64 = 50; // 20Hz (50ms)

impl Default for GameServer {
    fn default() -> Self {
        Self::new()
    }
}

impl GameServer {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            sprite_manager: Arc::new(RwLock::new(SpriteManager::new())),
        }
    }

    /// Start broadcasting player position in circular motion to all clients
    pub fn start_circle_motion_broadcast(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut angle = 0.0f32;
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(BROADCAST_INTERVAL_MS));
            let mut tick_count = 0u64;

            loop {
                interval.tick().await;
                tick_count += 1;

                // Calculate circle position
                let x = CIRCLE_RADIUS * angle.cos();
                let y = CIRCLE_RADIUS * angle.sin();

                // Update angle
                angle += CIRCLE_SPEED * (BROADCAST_INTERVAL_MS as f32 / 1000.0);
                if angle > std::f32::consts::PI * 2.0 {
                    angle -= std::f32::consts::PI * 2.0;
                }

                // Send to all connected clients
                let connections = self.connections.read().await;
                let client_count = connections.len();

                if client_count > 0 {
                    if tick_count.is_multiple_of(20) {
                        info!("[Circle Broadcast] Sending to {} clients: id=999, x={:.2}, y={:.2}, angle={:.2}",
                              client_count, x, y, angle);
                    }

                    for (client_id, conn) in connections.iter() {
                        let player_pos = PlayerPos::new(
                            uuid::Uuid::now_v7(),
                            999, // Use special ID 999 for the circle player
                            x,
                            y,
                        );

                        let bytes = unsafe {
                            std::slice::from_raw_parts(
                                &player_pos as *const PlayerPos as *const u8,
                                std::mem::size_of::<PlayerPos>(),
                            )
                            .to_vec()
                        };

                        if let Err(e) = conn.send_datagram(bytes.clone()) {
                            error!("Failed to send circle position to {}: {}", client_id, e);
                        } else if tick_count.is_multiple_of(20) {
                            info!("[DEBUG] Successfully sent circle packet to {} ({} bytes)", client_id, bytes.len());
                        }
                    }
                }
            }
        });
    }

    /// Start sprite management tasks
    pub fn start_sprite_management(self: Arc<Self>) {
        let sprite_manager = self.sprite_manager.clone();
        let connections = self.connections.clone();

        // Task 1: Spawn sprites every 3 seconds
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));

            loop {
                interval.tick().await;

                let mut manager = sprite_manager.write().await;
                if manager.should_spawn() {
                    let create_msg = manager.spawn_sprite();

                    // Broadcast to all clients as struct (zero-copy)
                    let connections = connections.read().await;
                    let bytes = unsafe {
                        std::slice::from_raw_parts(
                            &create_msg as *const SpriteMessage as *const u8,
                            std::mem::size_of::<SpriteMessage>(),
                        )
                        .to_vec()
                    };

                    info!("[Sprite Broadcast] Sending CREATE to {} clients ({} bytes)", connections.len(), bytes.len());

                    for (client_id, conn) in connections.iter() {
                        if let Err(e) = conn.send_datagram(bytes.clone()) {
                            error!("Failed to send sprite CREATE to {}: {}", client_id, e);
                        }
                    }
                }
            }
        });

        // Task 2: Update sprites every 100ms
        let sprite_manager = self.sprite_manager.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

            loop {
                interval.tick().await;

                let mut manager = sprite_manager.write().await;
                let updates = manager.update_sprites();

                if !updates.is_empty() {
                    let connections = connections.read().await;

                    for update_msg in updates {
                        let bytes = unsafe {
                            std::slice::from_raw_parts(
                                &update_msg as *const SpriteMessage as *const u8,
                                std::mem::size_of::<SpriteMessage>(),
                            )
                            .to_vec()
                        };

                        for (client_id, conn) in connections.iter() {
                            if let Err(e) = conn.send_datagram(bytes.clone()) {
                                error!("Failed to send sprite UPDATE to {}: {}", client_id, e);
                            }
                        }
                    }
                }
            }
        });

        // Task 3: Cleanup expired sprites every 1 second
        let sprite_manager = self.sprite_manager.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

            loop {
                interval.tick().await;

                let mut manager = sprite_manager.write().await;
                let deletes = manager.cleanup_expired_sprites();

                if !deletes.is_empty() {
                    let connections = connections.read().await;

                    for delete_msg in deletes {
                        let bytes = unsafe {
                            std::slice::from_raw_parts(
                                &delete_msg as *const SpriteMessage as *const u8,
                                std::mem::size_of::<SpriteMessage>(),
                            )
                            .to_vec()
                        };

                        for (client_id, conn) in connections.iter() {
                            if let Err(e) = conn.send_datagram(bytes.clone()) {
                                error!("Failed to send sprite DELETE to {}: {}", client_id, e);
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn handle_connection(&self, conn: Connection) {
        let client_id = Uuid::now_v7();
        info!("New connection from client: {}", client_id);

        // Register connection
        {
            let mut map = self.connections.write().await;
            map.insert(client_id, conn.clone());
        }

        let connection_map = self.connections.clone();

        // Main receive loop
        loop {
            match conn.receive_datagram().await {
                Ok(data) => {
                    if let Err(e) = self.handle_packet(&conn, &data, client_id).await {
                        warn!("Failed to handle packet from {}: {}", client_id, e);
                    }
                }
                Err(_) => {
                    info!("Client {} disconnected", client_id);
                    break;
                }
            }
        }

        // Cleanup
        {
            let mut map = connection_map.write().await;
            map.remove(&client_id);
        }
    }

    async fn handle_packet(&self, conn: &Connection, data: &[u8], client_id: Uuid) -> Result<()> {
        info!("[DEBUG] Received packet from client {}: {} bytes, raw data: {:02x?}",
              client_id, data.len(), &data[..data.len().min(32)]);

        // Validate minimum packet size
        if data.len() < std::mem::size_of::<PacketHeader>() {
            warn!("Received packet too small: {} bytes", data.len());
            return Ok(());
        }

        // Parse header
        let header_ptr = data.as_ptr() as *const PacketHeader;
        let header = unsafe { &*header_ptr };

        // Validate magic byte
        if !header.is_valid() {
            warn!("Invalid magic byte in packet");
            return Ok(());
        }

        // Match packet type
        match PacketType::from_u8(header.packet_type) {
            Some(PacketType::KeepAlive) => {
                info!("Received KeepAlive from client {}", client_id);
            }
            Some(PacketType::PlayerPos) => {
                // Parse PlayerPos struct using safe unaligned read
                let pos = match read_struct_unaligned::<PlayerPos>(data) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to parse PlayerPos: {}", e);
                        warn!("Raw data: {:02x?}", &data[..data.len().min(32)]);
                        return Ok(());
                    }
                };

                if pos.validate().is_err() {
                    warn!("Invalid PlayerPos packet: magic=0x{:02x}, packet_type={}",
                          pos.magic, pos.packet_type);
                    warn!("Raw data: {:02x?}", &data[..data.len().min(32)]);
                    return Ok(());
                }

                info!(
                    "PlayerPos received: client={}, player_id={}, x={:.2}, y={:.2}, packet_size={}",
                    client_id, pos.player_id, pos.x, pos.y, data.len()
                );

                // Echo back to client (optional, for testing)
                // self.send_game_state(conn, 0, 1).await?;
            }
            Some(PacketType::GameState) => {
                // Parse GameState struct using safe unaligned read
                let state = match read_struct_unaligned::<GameState>(data) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Failed to parse GameState: {}", e);
                        warn!("Raw data: {:02x?}", &data[..data.len().min(32)]);
                        return Ok(());
                    }
                };

                if state.validate().is_err() {
                    warn!("Invalid GameState packet: magic=0x{:02x}, packet_type={}",
                          state.magic, state.packet_type);
                    warn!("Raw data: {:02x?}", &data[..data.len().min(32)]);
                    return Ok(());
                }

                info!(
                    "GameState received: client={}, type={}, tick={}, player_count={:08x}, packet_size={}",
                    client_id, state.get_type_description(), state.tick, state.player_count, data.len()
                );

                // Handle different GameState message types
                if state.is_hello() {
                    // Respond to hello message with echo
                    info!("  -> Hello from client, sending echo response");
                    self.send_game_state(conn, 0, unity_network::game_state::MSG_TYPE_ECHO).await?;
                } else if state.is_echo() {
                    // This shouldn't happen on server side, but log it
                    info!("  -> Received echo response (unexpected)");
                } else {
                    // Regular game state, log and optionally respond
                    info!("  -> Player count: {}", state.player_count);
                    // Echo back current server state
                    self.send_game_state(conn, state.tick, state.player_count).await?;
                }
            }
            Some(PacketType::SpriteMessage) => {
                // Parse SpriteMessage struct using safe unaligned read
                let sprite_msg = match read_struct_unaligned::<SpriteMessage>(data) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Failed to parse SpriteMessage: {}", e);
                        warn!("Raw data: {:02x?}", &data[..data.len().min(32)]);
                        return Ok(());
                    }
                };

                if sprite_msg.validate().is_err() {
                    warn!("Invalid SpriteMessage packet: magic=0x{:02x}, packet_type={}",
                          sprite_msg.magic, sprite_msg.packet_type);
                    warn!("Raw data: {:02x?}", &data[..data.len().min(32)]);
                    return Ok(());
                }

                match sprite_msg.get_operation() {
                    Some(SpriteOp::Create) => {
                        info!("Sprite CREATE received from client: id={:?}, x={}, y={}",
                              sprite_msg.get_id(), sprite_msg.x, sprite_msg.y);
                    }
                    Some(SpriteOp::Update) => {
                        info!("Sprite UPDATE received from client: id={:?}, x={}, y={}",
                              sprite_msg.get_id(), sprite_msg.x, sprite_msg.y);
                    }
                    Some(SpriteOp::Delete) => {
                        info!("Sprite DELETE received from client: id={:?}", sprite_msg.get_id());
                    }
                    Some(SpriteOp::Snapshot) => {
                        info!("Sprite SNAPSHOT received from client");
                    }
                    None => {
                        warn!("Unknown sprite operation: {}", sprite_msg.operation);
                    }
                }
            }
            None => {
                warn!("Unknown packet type: {}", header.packet_type);
            }
        }

        Ok(())
    }

    async fn send_game_state(&self, conn: &Connection, tick: u32, player_count: u32) -> Result<()> {
        let state = GameState::new(tick, player_count);
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &state as *const GameState as *const u8,
                std::mem::size_of::<GameState>(),
            )
            .to_vec()
        };

        let bytes_len = bytes.len();

        info!(
            "Sending GameState: type={}, tick={}, player_count={:08x}, size={}",
            state.get_type_description(), state.tick, state.player_count, bytes_len
        );

        conn.send_datagram(bytes)?;
        info!("[DEBUG] Server sent GameState: type={}, tick={}, size={}", state.get_type_description(), state.tick, bytes_len);
        Ok(())
    }




}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,unity_network=debug,wtransport=warn")
        .init();

    info!("Starting Unity FFI WebTransport server...");

    // Generate self-signed certificate for development
    let identity = Identity::self_signed(["localhost"])?;

    // Note: For development with self-signed certificates, Unity client
    // should use no certificate validation or accept any certificate
    info!("Using self-signed certificate for development");
    info!("Unity client can connect without certificate hash for testing");

    // Build server configuration
    let config = ServerConfig::builder()
        .with_bind_default(4433)
        .with_identity(identity)
        .build();

    let endpoint = Endpoint::server(config)?;

    let server = Arc::new(GameServer::new());

    // Start circle motion broadcaster
    info!("Starting circle motion broadcast (radius={:.1}, speed={:.1} rad/s)", CIRCLE_RADIUS, CIRCLE_SPEED);
    server.clone().start_circle_motion_broadcast();

    // Start sprite management
    info!("Starting sprite management (spawn every 3s, update every 100ms, cleanup every 1s)");
    server.clone().start_sprite_management();

    info!("Server listening on wtransport://127.0.0.1:4433");
    info!("Waiting for Unity connections...");

    // Accept loop
    loop {
        let incoming = endpoint.accept().await;
        let connection_request = incoming.await?;
        let conn = connection_request.accept().await?;

        let server_clone = server.clone();
        tokio::spawn(async move {
            server_clone.handle_connection(conn).await;
        });
    }
}
