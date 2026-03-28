//! Test that mimics exact Unity FFI threading architecture
//!
//! This reproduces the threading pattern used in Unity:
//! - Main thread: Calls network_connect, sends packets, polls for receive
//! - Background thread: Runs tokio runtime with WebTransport tasks
//! - MPSC channels: Bridge between threads

use anyhow::Result;
use std::mem;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, warn};
use unity_network::{PacketHeader, PacketType, PlayerPos};
use wtransport::endpoint::Endpoint;
use wtransport::ClientConfig;
use wtransport::Connection;

/// Simulates the Unity main thread behavior
struct UnityMainThread {
    outbound_tx: Option<Sender<Vec<u8>>>,
    inbound_rx: Option<Receiver<Vec<u8>>>,
    is_connected: bool,
}

impl UnityMainThread {
    fn connect(server_url: &str) -> Result<Self> {
        info!("🎮 [Unity Main Thread] Calling connect()...");

        // Create channels (same as FFI)
        // std::sync::mpsc for FFI interface (blocking)
        let (outbound_tx, outbound_rx) = mpsc::channel::<Vec<u8>>();
        let (inbound_tx, inbound_rx) = mpsc::channel::<Vec<u8>>();

        // tokio::sync::mpsc for async tasks (non-blocking)
        let (async_outbound_tx, async_outbound_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);
        let (async_inbound_tx, mut async_inbound_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);

        // Spawn bridge thread: std::sync::mpsc → tokio::sync::mpsc
        let _bridge_tx_handle = thread::spawn(move || {
            info!("🌉 [Bridge Thread] Started: std::sync::mpsc → tokio::sync::mpsc");
            loop {
                match outbound_rx.recv() {
                    Ok(data) => {
                        if async_outbound_tx.blocking_send(data).is_err() {
                            error!("🌉 [Bridge Thread] Failed to send to async channel");
                            break;
                        }
                    }
                    Err(_) => {
                        info!("🌉 [Bridge Thread] std::sync::mpsc channel closed, exiting");
                        break;
                    }
                }
            }
            info!("🌉 [Bridge Thread] Ended");
        });

        // Spawn bridge thread: tokio::sync::mpsc → std::sync::mpsc
        let _bridge_rx_handle = thread::spawn(move || {
            info!("🌉 [Bridge Thread] Started: tokio::sync::mpsc → std::sync::mpsc");
            let rt = tokio::runtime::Runtime::new().expect("Bridge: Failed to create runtime");
            rt.block_on(async move {
                while let Some(data) = async_inbound_rx.recv().await {
                    if inbound_tx.send(data).is_err() {
                        error!("🌉 [Bridge Thread] Failed to send to std channel");
                        break;
                    }
                }
                info!("🌉 [Bridge Thread] tokio::sync::mpsc channel closed, exiting");
            });
            info!("🌉 [Bridge Thread] Ended");
        });

        // Create oneshot to wait for connection ready
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();

        // Spawn background thread (exactly like FFI)
        let url = server_url.to_string();
        let _handle = thread::spawn(move || {
            info!("🔧 [Background Thread] Starting tokio runtime...");
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

            let result = rt.block_on(async move {
                background_task(url, async_outbound_rx, async_inbound_tx, ready_tx).await
            });

            if let Err(e) = result {
                error!("🔧 [Background Thread] Error: {:?}", e);
            }
        });

        // Wait for connection to be ready (blocking, exactly like FFI)
        info!("🎮 [Unity Main Thread] Waiting for connection ready signal...");
        ready_rx.recv()?;
        info!("🎮 [Unity Main Thread] Connection ready signal received!");
        info!("🎮 [Unity Main Thread] Bridge threads active: std::sync::mpsc ↔ tokio::sync::mpsc");

        // Simulate context structure returned by FFI
        Ok(Self {
            outbound_tx: Some(outbound_tx),
            inbound_rx: Some(inbound_rx),
            is_connected: true,
        })
    }

    fn send_packet(&self, data: Vec<u8>) -> Result<()> {
        if !self.is_connected {
            warn!("🎮 [Unity Main Thread] Not connected, can't send");
            return Ok(());
        }

        if let Some(tx) = &self.outbound_tx {
            info!(
                "🎮 [Unity Main Thread] Sending {} bytes to channel",
                data.len()
            );
            tx.send(data)?;
            info!("🎮 [Unity Main Thread] Packet sent to channel");
        }

        Ok(())
    }

    fn poll_packet(&self) -> Result<Option<Vec<u8>>> {
        if !self.is_connected {
            return Ok(None);
        }

        if let Some(rx) = &self.inbound_rx {
            match rx.try_recv() {
                Ok(data) => {
                    info!("🎮 [Unity Main Thread] Poll received {} bytes", data.len());
                    Ok(Some(data))
                }
                Err(TryRecvError::Empty) => Ok(None),
                Err(TryRecvError::Disconnected) => {
                    warn!("🎮 [Unity Main Thread] Inbound channel disconnected");
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

/// Background thread async task (exactly like connect_async in FFI)
async fn background_task(
    url: String,
    outbound_rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    inbound_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ready_tx: std::sync::mpsc::Sender<()>,
) -> Result<()> {
    info!("🔧 [Background Thread] Async task started");

    // Build client config
    let client_config = ClientConfig::builder()
        .with_bind_default()
        .with_no_cert_validation()
        .build();

    info!("🔧 [Background Thread] Creating endpoint...");
    let endpoint = Endpoint::client(client_config)?;

    info!("🔧 [Background Thread] Connecting to {}...", url);
    let connection: Connection = endpoint.connect(url).await?;
    info!("🔧 [Background Thread] Connected successfully!");

    // Signal that connection is ready
    info!("🔧 [Background Thread] Sending ready signal...");
    let _ = ready_tx.send(());
    info!("🔧 [Background Thread] Ready signal sent!");

    // Spawn outbound task
    let outbound_connection = connection.clone();
    let outbound_handle = tokio::spawn(async move {
        info!("🔧 [Outbound Task] Spawned!");
        let mut packets_sent = 0u64;
        let mut outbound_rx = outbound_rx;

        while let Some(data) = outbound_rx.recv().await {
            info!(
                "🔧 [Outbound Task] Received {} bytes from channel",
                data.len()
            );

            if let Err(e) = outbound_connection.send_datagram(data) {
                error!("🔧 [Outbound Task] Send error: {:?}", e);
                break;
            }

            packets_sent += 1;
            info!(
                "🔧 [Outbound Task] Sent packet #{} via WebTransport",
                packets_sent
            );
        }

        info!("🔧 [Outbound Task] Ended, total sent: {}", packets_sent);
    });

    // Spawn inbound task
    let inbound_handle = tokio::spawn(async move {
        info!("🔧 [Inbound Task] Spawned!");
        let mut packets_received = 0u64;

        loop {
            match connection.receive_datagram().await {
                Ok(data) => {
                    info!(
                        "🔧 [Inbound Task] Received {} bytes from WebTransport",
                        data.len()
                    );
                    packets_received += 1;

                    if inbound_tx.send(data.to_vec()).await.is_err() {
                        error!("🔧 [Inbound Task] Channel closed");
                        break;
                    }
                    info!(
                        "🔧 [Inbound Task] Queued packet #{} for main thread",
                        packets_received
                    );
                }
                Err(e) => {
                    error!("🔧 [Inbound Task] Receive error: {:?}", e);
                    break;
                }
            }
        }

        info!(
            "🔧 [Inbound Task] Ended, total received: {}",
            packets_received
        );
    });

    // Keep async task alive (exactly like FFI)
    info!("🔧 [Background Thread] Tasks spawned, keeping alive...");
    sleep(Duration::from_secs(u64::MAX)).await;

    // Wait for tasks to complete
    outbound_handle.await?;
    inbound_handle.await?;

    info!("🔧 [Background Thread] Terminating");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,test_ffi_arch=debug")
        .init();

    info!("=== Unity FFI Architecture Test ===");
    info!("This test mimics exact threading pattern used in Unity FFI");
    info!("");
    info!("Architecture:");
    info!("  - Main thread: Simulates C# (sends packets, polls)");
    info!("  - Background thread: Runs tokio runtime");
    info!("  - MPSC channels: Bridge between threads");
    info!("  - Tokio tasks: Handle WebTransport I/O");
    info!("");

    // Connect to server
    info!("Step 1: Connecting to server...");
    let main_thread = UnityMainThread::connect("https://127.0.0.1:4433")?;
    info!("✅ Step 1: Connected!");
    info!("");

    // Test bidirectional communication
    info!("Step 2: Testing bidirectional communication for 30 seconds...");
    let start_time = Instant::now();
    let mut packets_sent = 0u64;
    let mut packets_received = 0u64;
    let mut last_circle_update = None;

    while start_time.elapsed() < Duration::from_secs(30) {
        // Send packets every 100ms (simulating Unity Update)
        if start_time.elapsed().as_millis() % 100 < 10 {
            let pos = PlayerPos::new(uuid::Uuid::now_v7(), 1, 10.0, 20.0);
            let bytes = unsafe {
                std::slice::from_raw_parts(
                    &pos as *const PlayerPos as *const u8,
                    mem::size_of::<PlayerPos>(),
                )
                .to_vec()
            };

            if let Err(e) = main_thread.send_packet(bytes) {
                warn!("Send error: {:?}", e);
            } else {
                packets_sent += 1;
            }
        }

        // Poll for received packets (simulating Unity Poll)
        if let Ok(Some(data)) = main_thread.poll_packet() {
            packets_received += 1;

            // Parse packet
            if data.len() >= mem::size_of::<PacketHeader>() {
                let header = unsafe { *(data.as_ptr() as *const PacketHeader) };

                if header.is_valid() {
                    if let Some(PacketType::PlayerPos) = PacketType::from_u8(header.packet_type) {
                        if data.len() >= mem::size_of::<PlayerPos>() {
                            let pos = unsafe { *(data.as_ptr() as *const PlayerPos) };

                            if pos.player_id == 999 {
                                if last_circle_update.is_none() {
                                    info!("🎯 First circle motion received from server!");
                                }
                                last_circle_update = Some((pos.x, pos.y));
                                info!(
                                    "📥 [MAIN] Circle motion: player_id={}, x={:.2}, y={:.2}",
                                    pos.player_id, pos.x, pos.y
                                );
                            } else {
                                info!(
                                    "📥 [MAIN] PlayerPos: player_id={}, x={:.2}, y={:.2}",
                                    pos.player_id, pos.x, pos.y
                                );
                            }
                        }
                    }
                }
            }
        }

        // Small sleep to avoid 100% CPU
        sleep(Duration::from_millis(10)).await;
    }

    info!("");
    info!("=== Test Complete ===");
    info!("📤 Packets sent: {}", packets_sent);
    info!("📥 Packets received: {}", packets_received);

    if last_circle_update.is_some() {
        info!("✅ Circle motion packets received from server!");
    } else {
        info!("❌ No circle motion packets received!");
        info!("❌ This reproduces the Unity FFI bug!");
    }

    if packets_sent > 0 && packets_received == 0 {
        info!("❌ BUG REPRODUCED: Packets sent but none received!");
        info!("❌ This matches Unity behavior!");
    }

    Ok(())
}
