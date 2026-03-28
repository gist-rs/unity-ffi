//! Simple Rust Test Client for WebTransport
//!
//! Tests bidirectional communication with the unity-ffi-server
//! without Unity FFI involvement.

use anyhow::Result;
use std::mem;
use std::time::Duration;
use tokio::time::{interval, timeout};
use tracing::{error, info, warn};
use uuid::Uuid;
use wtransport::endpoint::Endpoint;
use wtransport::ClientConfig;
use wtransport::Connection;
use unity_network::{GameState, PacketHeader, PacketType, PlayerPos};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,test_client=debug,wtransport=warn")
        .init();

    info!("=== WebTransport Test Client ===");
    info!("This test will verify bidirectional WebTransport communication");

    // Build client configuration (same as FFI)
    let client_config = ClientConfig::builder()
        .with_bind_default()
        .with_no_cert_validation()
        .build();

    info!("Creating endpoint...");
    let endpoint = Endpoint::client(client_config)?;

    info!("Connecting to wtransport://127.0.0.1:4433...");
    let connection: Connection = endpoint
        .connect("https://127.0.0.1:4433")
        .await
        .expect("Failed to connect to server");

    info!("✅ Connected to server!");
    info!("📊 Performing initial receive to complete QUIC handshake...");

    // Receive first to complete QUIC handshake
    let initial_result = timeout(Duration::from_secs(2), connection.receive_datagram()).await;
    match initial_result {
        Ok(Ok(data)) => {
            info!("✅ Initial receive completed: {} bytes", data.len());
            info!("📊 Datagram stream ready!");
        }
        Ok(Err(e)) => {
            error!("❌ Initial receive error: {:?}", e);
        }
        Err(_) => {
            info!("⏱️ Initial receive timeout (no initial packet), proceeding...");
        }
    }

    // Spawn receiver task
    let connection_recv = connection.clone();
    let recv_handle = tokio::spawn(async move {
        let mut packets_received = 0u64;
        let mut no_data_count: u32 = 0;
        info!("📥 Receiver task started");

        loop {
            // Add timeout to detect if receive is blocking
            match timeout(Duration::from_secs(1), connection_recv.receive_datagram()).await {
                Ok(Ok(data)) => {
                    no_data_count = 0;
                    packets_received += 1;

                    // Parse packet header
                    if data.len() >= mem::size_of::<PacketHeader>() {
                        let header = unsafe {
                            *(data.as_ptr() as *const PacketHeader)
                        };

                        if header.is_valid() {
                            match PacketType::from_u8(header.packet_type) {
                                Some(PacketType::PlayerPos) => {
                                    if data.len() >= mem::size_of::<PlayerPos>() {
                                        let pos = unsafe {
                                            *(data.as_ptr() as *const PlayerPos)
                                        };
                                        info!("📥 [RECV] PlayerPos: player_id={}, x={:.2}, y={:.2}",
                                              pos.player_id, pos.x, pos.y);

                                        if pos.player_id == 999 {
                                            info!("🎯 This is the circle motion from server!");
                                        }
                                    }
                                }
                                Some(PacketType::GameState) => {
                                    if data.len() >= mem::size_of::<GameState>() {
                                        let state = unsafe {
                                            *(data.as_ptr() as *const GameState)
                                        };
                                        info!("📥 [RECV] GameState: tick={}, player_count={}",
                                              state.tick, state.player_count);
                                    }
                                }
                                Some(PacketType::KeepAlive) => {
                                    info!("📥 [RECV] KeepAlive");
                                }
                                Some(PacketType::SpriteMessage) => {
                                    if data.len() >= mem::size_of::<unity_network::SpriteMessage>() {
                                        let sprite_msg = unsafe {
                                            *(data.as_ptr() as *const unity_network::SpriteMessage)
                                        };
                                        info!("📥 [RECV] SpriteMessage: op={:?}, id={:?}, x={}, y={}",
                                              sprite_msg.get_operation(), sprite_msg.get_id(), sprite_msg.x, sprite_msg.y);
                                    }
                                }
                                None => {
                                    warn!("📥 [RECV] Unknown packet type: {}", header.packet_type);
                                }
                            }
                        } else {
                            warn!("📥 [RECV] Invalid magic byte!");
                        }
                    } else {
                        warn!("📥 [RECV] Packet too small: {} bytes", data.len());
                    }
                }
                Ok(Err(e)) => {
                    error!("❌ Receive error: {:?}", e);
                    break;
                }
                Err(_) => {
                    // Timeout - no data for 1 second
                    no_data_count += 1;
                    if no_data_count == 1 {
                        warn!("⏱️ Receive timed out (1s) - no data received yet");
                    }
                    if no_data_count.is_multiple_of(10) {
                        warn!("⏱️ Still no data after {} seconds", no_data_count);
                    }
                    if no_data_count > 30 {
                        error!("❌ No data for 30 seconds, closing receiver");
                        break;
                    }
                    // Continue to try again
                }
            }
        }

        info!("📥 Receiver ended, total packets received: {}", packets_received);
    });

    // Send test packets every 100ms
    info!("Starting to send packets every 100ms...");
    let mut send_interval = interval(Duration::from_millis(100));
    let mut packets_sent = 0u64;
    let mut player_id: u32 = 1;

    // Test for 30 seconds
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            info!("⏱️ Test timeout reached");
        }
        _ = async {
            loop {
                send_interval.tick().await;

                // Create PlayerPos packet
                let pos = PlayerPos::new(Uuid::now_v7(), player_id as u64, 1.0, 2.0);
                let bytes = unsafe {
                    std::slice::from_raw_parts(
                        &pos as *const PlayerPos as *const u8,
                        mem::size_of::<PlayerPos>(),
                    )
                    .to_vec()
                };

                match connection.send_datagram(bytes.clone()) {
                    Ok(_) => {
                        packets_sent += 1;
                        info!("📤 [SEND] PlayerPos #{}: id={}, x={:.2}, y={:.2}, size={} bytes",
                              packets_sent, player_id, pos.x, pos.y, bytes.len());
                        if packets_sent.is_multiple_of(50) {
                            info!("📊 Send progress: {} packets sent", packets_sent);
                        }
                    }
                    Err(e) => {
                        error!("❌ Send error: {:?}", e);
                        error!("❌ Packet size: {} bytes", bytes.len());
                        break;
                    }
                }

                // Alternate player IDs for variety
                player_id = if player_id == 1 { 2 } else { 1 };
            }
        } => {}
    }

    // Cleanup
    info!("=== Test Summary ===");
    info!("📤 Total packets sent: {}", packets_sent);

    recv_handle.abort();

    info!("✅ Test complete!");

    Ok(())
}
