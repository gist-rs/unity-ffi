//! Sprite Lifecycle Test - Rust-Only CRUD Verification
//!
//! This test verifies server sprite management without Unity:
//! - Connects to server via WebTransport
//! - Receives CREATE, UPDATE, DELETE messages
//! - Logs all operations to console
//! - Verifies statistics match expectations

use anyhow::Result;
use std::mem;
use std::ptr;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, timeout};
use tracing::{error, info, warn};
use wtransport::endpoint::Endpoint;
use wtransport::ClientConfig;
use wtransport::Connection;

// Import shared types from unity-network
use unity_network::{SpriteMessage, SpriteOp, SpriteType};

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

// Messages sent from receiver task to main task
#[derive(Debug)]
enum TestEvent {
    Create { id: uuid::Uuid, _sprite_type: SpriteType, x: i16, y: i16 },
    Update { id: uuid::Uuid, x: i16, y: i16 },
    Delete { id: uuid::Uuid },
    StateSnapshot { count: usize },
    OutOfBounds { id: uuid::Uuid, x: i16, y: i16 },
    _NonSpriteMessage(String),
    NonUtf8Data(usize),
}

// Test statistics
struct TestStats {
    created_count: u32,
    updated_count: u32,
    deleted_count: u32,
    active_sprites: Vec<uuid::Uuid>,
    out_of_bounds_count: u32,
    max_concurrent_sprites: u32,
}

impl TestStats {
    fn new() -> Self {
        Self {
            created_count: 0,
            updated_count: 0,
            deleted_count: 0,
            active_sprites: Vec::new(),
            out_of_bounds_count: 0,
            max_concurrent_sprites: 0,
        }
    }

    fn log_summary(&self) {
        info!("");
        info!("=== Sprite Lifecycle Test Summary ===");
        info!("✅ Created: {} sprites", self.created_count);
        info!("✅ Updated: {} positions", self.updated_count);
        info!("✅ Deleted: {} sprites", self.deleted_count);
        info!("✅ Active sprites: {}", self.active_sprites.len());
        info!("✅ Out of bounds violations: {}", self.out_of_bounds_count);
        info!("✅ Max concurrent sprites: {}", self.max_concurrent_sprites);
        info!("====================================");

        // Verify expectations (3s spawn * 60s test = ~20 sprites, ~3-4 concurrent)
        let expected_spawns = self.created_count;
        let expected_deletes = expected_spawns; // All should die after 10s

        if self.created_count >= 18 && self.created_count <= 22 {
            info!("✅ CREATE count: {} (expected ~20) ✅", self.created_count);
        } else {
            warn!("⚠️  CREATE count: {} (expected ~20)", self.created_count);
        }

        if self.deleted_count == expected_deletes {
            info!("✅ DELETE count: {} (matches creates) ✅", self.deleted_count);
        } else {
            warn!("⚠️  DELETE count: {} (expected {})", self.deleted_count, expected_deletes);
        }

        if self.max_concurrent_sprites >= 3 && self.max_concurrent_sprites <= 5 {
            info!("✅ Max concurrent sprites: {} (expected 3-4) ✅", self.max_concurrent_sprites);
        } else {
            warn!("⚠️  Max concurrent sprites: {} (expected 3-4)", self.max_concurrent_sprites);
        }

        if self.updated_count > 1500 {
            info!("✅ UPDATE count: {} (expected ~2000+) ✅", self.updated_count);
        } else {
            warn!("⚠️  UPDATE count: {} (expected ~2000+)", self.updated_count);
        }

        if self.active_sprites.is_empty() {
            info!("✅ Active sprites: {} (expected 0) ✅", self.active_sprites.len());
        } else {
            warn!("⚠️  Active sprites: {} (expected 0) - sprites remain!", self.active_sprites.len());
        }

        if self.out_of_bounds_count == 0 {
            info!("✅ No out-of-bounds violations ✅");
        } else {
            error!("❌ Out-of-bounds violations: {} ❌", self.out_of_bounds_count);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,test_sprite_lifecycle=debug,wtransport=warn")
        .init();

    info!("=== Sprite Lifecycle Test ===");
    info!("Verifying server sprite management: CREATE, READ, UPDATE, DELETE");
    info!("Test duration: 60 seconds");
    info!("");

    // Build client configuration
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
    info!("📊 Waiting for sprite messages...");

    // Channel for communicating events from receiver to main
    let (event_tx, mut event_rx) = mpsc::channel::<TestEvent>(256);

    // Spawn receiver task
    let connection_recv = connection.clone();

    tokio::spawn(async move {
        let mut packets_received = 0u64;
        info!("📥 Receiver task started");

        loop {
            // Use timeout to detect if receive is blocking
            match timeout(Duration::from_secs(1), connection_recv.receive_datagram()).await {
                Ok(Ok(data)) => {
                    packets_received += 1;

                    // Try to parse as SpriteMessage struct
                    if let Ok(sprite_msg) = read_struct_unaligned::<SpriteMessage>(&data) {
                        if sprite_msg.validate().is_err() {
                            warn!("⚠️  Received invalid SpriteMessage: magic=0x{:02x}, packet_type={}",
                                  sprite_msg.magic, sprite_msg.packet_type);
                        } else {
                            match sprite_msg.get_operation() {
                                Some(SpriteOp::Create) => {
                                    let id = sprite_msg.get_id();
                                    let sprite_type = sprite_msg.get_sprite_type()
                                        .expect("Sprite type should be present in Create messages");
                                    if event_tx
                                        .send(TestEvent::Create {
                                            id,
                                            _sprite_type: sprite_type,
                                            x: sprite_msg.x,
                                            y: sprite_msg.y,
                                        })
                                        .await
                                        .is_err()
                                    {
                                        error!("Failed to send Create event to main task");
                                        break;
                                    }
                                }

                                Some(SpriteOp::Update) => {
                                    let id = sprite_msg.get_id();
                                    let x = sprite_msg.x;
                                    let y = sprite_msg.y;

                                    // Check bounds (0-127 for 128x128 map)
                                    if !(0..=127).contains(&x) || !(0..=127).contains(&y) {
                                        if event_tx
                                            .send(TestEvent::OutOfBounds { id, x, y })
                                            .await
                                            .is_err()
                                        {
                                            error!("Failed to send OutOfBounds event to main task");
                                            break;
                                        }
                                    } else if event_tx
                                        .send(TestEvent::Update { id, x, y })
                                        .await
                                        .is_err()
                                    {
                                        error!("Failed to send Update event to main task");
                                        break;
                                    }
                                }

                                Some(SpriteOp::Delete) => {
                                    let id = sprite_msg.get_id();
                                    if event_tx.send(TestEvent::Delete { id }).await.is_err() {
                                        error!("Failed to send Delete event to main task");
                                        break;
                                    }
                                }

                                Some(SpriteOp::Snapshot) => {
                                    // Snapshot messages don't contain detailed data in this implementation
                                    if event_tx
                                        .send(TestEvent::StateSnapshot { count: 0 })
                                        .await
                                        .is_err()
                                    {
                                        error!("Failed to send Snapshot event to main task");
                                        break;
                                    }
                                }

                                None => {
                                    warn!("⚠️  Unknown sprite operation: {}", sprite_msg.operation);
                                }
                            }
                        }
                    } else {
                        warn!("⚠️  Failed to parse SpriteMessage: {} bytes", data.len());
                        if event_tx
                            .send(TestEvent::NonUtf8Data(data.len()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    error!("❌ Receive error: {:?}", e);
                    break;
                }
                Err(_) => {
                    // Timeout - no data for 1 second, continue silently
                }
            }
        }

        info!("📥 Receiver ended, total packets received: {}", packets_received);
    });

    // Test for 60 seconds
    let start_time = tokio::time::Instant::now();
    let mut stats = TestStats::new();

    info!("Starting 60 second test window...");

    // Periodic statistics logging
    let mut log_interval = interval(Duration::from_secs(10));
    log_interval.tick().await; // Skip first tick

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(60)) => {
            info!("⏱️ Test timeout reached");
        }
        _ = async {
            loop {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        match event {
                            TestEvent::Create { id, _sprite_type, x, y } => {
                                stats.created_count += 1;
                                stats.active_sprites.push(id);
                                let concurrent = stats.active_sprites.len() as u32;
                                if concurrent > stats.max_concurrent_sprites {
                                    stats.max_concurrent_sprites = concurrent;
                                }
                                info!("✅ [CREATE] serrif_{:?} at ({}, {})",
                                      id, x, y);
                            }

                            TestEvent::Update { id, x, y } => {
                                stats.updated_count += 1;
                                info!("✅ [UPDATE] {} moved to ({}, {})", id, x, y);
                            }

                            TestEvent::Delete { id } => {
                                stats.deleted_count += 1;
                                stats.active_sprites.retain(|s| s != &id);
                                info!("✅ [DELETE] {}", id);
                            }

                            TestEvent::StateSnapshot { count } => {
                                info!("📊 [SNAPSHOT] Server has {} sprites", count);
                            }

                            TestEvent::OutOfBounds { id, x, y } => {
                                stats.out_of_bounds_count += 1;
                                error!("❌ [UPDATE] {} out of bounds at ({}, {})", id, x, y);
                            }

                            TestEvent::_NonSpriteMessage(msg) => {
                                warn!("⚠️  Non-sprite message: {}", msg);
                            }

                            TestEvent::NonUtf8Data(len) => {
                                warn!("⚠️  Non-UTF8 data: {} bytes", len);
                            }
                        }
                    }

                    _ = log_interval.tick() => {
                        let elapsed = start_time.elapsed().as_secs();
                        info!("📊 Progress: {}s | Created: {} | Updated: {} | Deleted: {} | Active: {}",
                              elapsed, stats.created_count, stats.updated_count,
                              stats.deleted_count, stats.active_sprites.len());
                    }
                }
            }
        } => {}
    }

    // Log final summary
    stats.log_summary();

    info!("✅ Test complete!");

    Ok(())
}
