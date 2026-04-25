//! Unity-Network FFI Bridge
//!
//! Provides a safe Rust FFI interface for Unity to connect to a WebTransport server.
//! Uses the "Caller-Allocated" pattern to avoid heap corruption between C# and Rust allocators.

pub mod packet_builder;
pub mod sprite_manager;
pub mod types;
pub mod webtransport;

// Re-export types used in examples and tests
pub use types::{PlayerPositionRecord, Position2D};

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::panic::catch_unwind;
use std::ptr;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::sync::mpsc as tokio_mpsc;
use tracing::{error, info, warn};
use wtransport::{ClientConfig, Connection, Endpoint};

pub use types::game_state;
pub use types::{FfiError, GameState, PacketHeader, PacketType, PlayerPos};

// Sprite management types
pub use sprite_manager::SpriteManager;
pub use types::{SpriteData, SpriteMessage, SpriteOp, SpriteType};

// Maximum buffer sizes
const PROTOCOL_VERSION: u32 = 1;

/// Log callback type from Unity
type LogCallback = unsafe extern "C" fn(level: *const c_char, message: *const c_char);

/// Opaque client context
/// Never exposed to C# directly, always passed as *mut c_void
#[allow(dead_code)]
struct ClientContext {
    runtime: Option<Runtime>,
    outbound_tx: Option<Sender<Vec<u8>>>,
    inbound_rx: Option<Receiver<Vec<u8>>>,
    is_connected: bool,
}

/// Global log callback (unsafe, but this is FFI)
static mut LOG_CALLBACK: Option<LogCallback> = None;

/// Set up logging callback from Unity
///
/// # Safety
///
/// This function is unsafe because it accepts a function pointer from C#.
/// The `log_callback` must:
/// - Be a valid function pointer that can be called from Rust
/// - Accept two C string pointers (level and message)
/// - Not be null
/// - Remain valid for the lifetime of the FFI library
#[no_mangle]
pub unsafe extern "C" fn network_init(log_callback: LogCallback) -> c_int {
    let result = catch_unwind(|| {
        // First, store the log callback so we can log errors
        LOG_CALLBACK = Some(log_callback);

        log_to_unity("INFO", "Initializing Unity network FFI...");

        // Initialize tracing to stdout as fallback
        log_to_unity("INFO", "Setting up tracing subscriber...");
        tracing_subscriber::fmt()
            .with_env_filter("info,unity_network=debug,wtransport=warn")
            .try_init()
            .map_err(|e| {
                log_to_unity("ERROR", &format!("Failed to initialize tracing: {:?}", e));
                e
            })
            .ok();

        log_to_unity("INFO", "Unity network FFI initialized successfully");

        FfiError::Success as c_int
    });

    match result {
        Ok(code) => code,
        Err(_) => {
            eprintln!("Panic in network_init");
            // Try to log even if we're in panic state
            log_to_unity(
                "ERROR",
                "Panic caught during initialization - check native logs for details",
            );
            FfiError::PanicCaught as c_int
        }
    }
}

/// Connect to WebTransport server
///
/// Returns opaque handle to ClientContext
///
/// # Safety
///
/// This function is unsafe because it deals with raw pointers from C#.
/// The `url` pointer must:
/// - Be a valid pointer to a null-terminated C string
/// - Point to valid memory for the entire string
/// - Be encoded as UTF-8
///
/// The `cert_hash` pointer may be null, or if not null must:
/// - Be a valid pointer to a null-terminated C string
/// - Contain a hex-encoded certificate hash
#[no_mangle]
pub unsafe extern "C" fn network_connect(
    url: *const c_char,
    cert_hash: *const c_char,
    protocol_version: u32,
) -> *mut c_void {
    let result = catch_unwind(|| {
        // Validate inputs
        if url.is_null() {
            log_to_unity("ERROR", "URL is null");
            return ptr::null_mut();
        }

        // Validate that URL is null-terminated
        let mut len = 0;
        while *url.add(len) != 0 {
            len += 1;
            if len > 4096 {
                log_to_unity("ERROR", "URL not null-terminated or too long");
                return ptr::null_mut();
            }
        }

        // Log URL bytes for debugging (first 64 bytes)
        let url_bytes = std::slice::from_raw_parts(url as *const u8, len.min(64));
        info!("URL bytes (hex): {:02x?}", url_bytes);
        info!(
            "URL bytes (str): {:?}",
            std::str::from_utf8_unchecked(url_bytes)
        );

        if protocol_version != PROTOCOL_VERSION {
            log_to_unity(
                "ERROR",
                &format!(
                    "Protocol version mismatch: expected {}, got {}",
                    PROTOCOL_VERSION, protocol_version
                ),
            );
            return ptr::null_mut();
        }

        // Convert C strings to Rust strings
        let url_str = match CStr::from_ptr(url).to_str() {
            Ok(s) => s.to_string(),
            Err(e) => {
                log_to_unity("ERROR", &format!("Invalid URL: {:?}", e));
                return ptr::null_mut();
            }
        };

        let cert_hash_str = if cert_hash.is_null() {
            None
        } else {
            match CStr::from_ptr(cert_hash).to_str() {
                Ok(s) => Some(s.to_string()),
                Err(_) => None,
            }
        };

        log_to_unity("INFO", &format!("Connecting to: {}", url_str));

        // Create channels for thread-safe communication
        // These are std::sync::mpsc channels for FFI interface (blocking)
        let (outbound_tx, outbound_rx) = mpsc::channel::<Vec<u8>>();
        let (inbound_tx, inbound_rx) = mpsc::channel::<Vec<u8>>();

        // Create async channels for tokio tasks (non-blocking)
        let (async_outbound_tx, async_outbound_rx) = tokio_mpsc::channel::<Vec<u8>>(128);
        let (async_inbound_tx, mut async_inbound_rx) = tokio_mpsc::channel::<Vec<u8>>(128);

        // Spawn bridge thread: std::sync::mpsc → tokio::sync::mpsc
        // This thread blocks on recv(), but that's OK because it's a dedicated thread
        let _bridge_tx_handle = thread::spawn(move || {
            info!("Bridge thread started: std::sync::mpsc → tokio::sync::mpsc");
            loop {
                match outbound_rx.recv() {
                    Ok(data) => {
                        if async_outbound_tx.blocking_send(data).is_err() {
                            error!("Bridge: Failed to send to async channel");
                            break;
                        }
                    }
                    Err(_) => {
                        info!("Bridge: std::sync::mpsc channel closed, exiting");
                        break;
                    }
                }
            }
            info!("Bridge thread ended");
        });

        // Spawn bridge thread: tokio::sync::mpsc → std::sync::mpsc
        let _bridge_rx_handle = thread::spawn(move || {
            info!("Bridge thread started: tokio::sync::mpsc → std::sync::mpsc");
            let rt = tokio::runtime::Runtime::new().expect("Bridge: Failed to create runtime");
            rt.block_on(async move {
                while let Some(data) = async_inbound_rx.recv().await {
                    if inbound_tx.send(data).is_err() {
                        error!("Bridge: Failed to send to std channel");
                        break;
                    }
                }
                info!("Bridge: tokio::sync::mpsc channel closed, exiting");
            });
            info!("Bridge thread ended");
        });

        // Create oneshot channel to signal when connection is ready
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();

        // Spawn Tokio runtime on dedicated thread
        let url_clone = url_str.clone();
        let _thread_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

            rt.block_on(async move {
                connect_async(
                    url_clone,
                    cert_hash_str,
                    async_outbound_rx,
                    async_inbound_tx,
                    ready_tx,
                )
                .await
            })
        });

        // Wait for connection to be established (blocking call)
        // This ensures we don't return until WebTransport is ready to send/receive
        match ready_rx.blocking_recv() {
            Ok(_) => {
                log_to_unity("INFO", "WebTransport connection fully established");
            }
            Err(_) => {
                log_to_unity("ERROR", "Failed to establish connection");
                return ptr::null_mut();
            }
        }

        // Create context
        let ctx = Box::new(ClientContext {
            runtime: None, // Runtime is owned by the thread
            outbound_tx: Some(outbound_tx),
            inbound_rx: Some(inbound_rx),
            is_connected: true,
        });

        log_to_unity("INFO", "Connection ready for use");
        log_to_unity(
            "DEBUG",
            "Bridge threads active: std::sync::mpsc ↔ tokio::sync::mpsc",
        );

        Box::into_raw(ctx) as *mut c_void
    });

    match result {
        Ok(ptr) => ptr,
        Err(_) => {
            eprintln!("Panic in network_connect");
            ptr::null_mut()
        }
    }
}

/// Send datagram to server
///
/// Returns FfiError code
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers from C#.
/// The `ctx` pointer must:
/// - Be a valid pointer returned by network_connect
/// - Point to memory that has not been freed (via network_destroy)
///
/// The `data_ptr` pointer must:
/// - Be a valid pointer to a byte array
/// - Point to at least `data_len` bytes of readable memory
#[no_mangle]
pub unsafe extern "C" fn network_send(
    ctx: *mut c_void,
    data_ptr: *const u8,
    data_len: usize,
) -> c_int {
    let result = catch_unwind(|| {
        if ctx.is_null() || data_ptr.is_null() {
            return FfiError::InvalidPointer as c_int;
        }

        let ctx = &mut *(ctx as *mut ClientContext);

        if !ctx.is_connected {
            return FfiError::Disconnected as c_int;
        }

        // Validate packet header if possible
        if data_len >= std::mem::size_of::<PacketHeader>() {
            let header_ptr = data_ptr as *const PacketHeader;
            let header = &*header_ptr;
            if !header.is_valid() {
                log_to_unity("ERROR", "Invalid magic byte in packet");
                return FfiError::InvalidMagic as c_int;
            }
        }

        // Copy data into vector
        let data = std::slice::from_raw_parts(data_ptr, data_len).to_vec();

        // Send to outbound channel
        match ctx.outbound_tx.as_ref() {
            Some(tx) => match tx.send(data) {
                Ok(_) => FfiError::Success as c_int,
                Err(_) => {
                    log_to_unity("ERROR", "Outbound channel disconnected");
                    ctx.is_connected = false;
                    FfiError::Disconnected as c_int
                }
            },
            None => FfiError::Disconnected as c_int,
        }
    });

    match result {
        Ok(code) => code,
        Err(_) => FfiError::PanicCaught as c_int,
    }
}

/// Poll for incoming data
///
/// Returns bytes written, 0 if no data, negative on error
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers from C#.
/// The `ctx` pointer must:
/// - Be a valid pointer returned by network_connect
/// - Point to memory that has not been freed (via network_destroy)
///
/// The `out_ptr` pointer must:
/// - Be a valid pointer to a writable byte buffer
/// - Point to at least `capacity` bytes of writable memory
#[no_mangle]
pub unsafe extern "C" fn network_poll(
    ctx: *mut c_void,
    out_ptr: *mut u8,
    capacity: usize,
) -> c_int {
    let result = catch_unwind(|| {
        if ctx.is_null() || out_ptr.is_null() {
            return FfiError::InvalidPointer as c_int;
        }

        let ctx = &mut *(ctx as *mut ClientContext);

        if !ctx.is_connected {
            return FfiError::Disconnected as c_int;
        }

        match ctx.inbound_rx.as_ref() {
            Some(rx) => {
                match rx.try_recv() {
                    Ok(data) => {
                        if data.len() > capacity {
                            log_to_unity(
                                "WARN",
                                &format!(
                                    "Buffer too small: need {}, have {}",
                                    data.len(),
                                    capacity
                                ),
                            );
                            return FfiError::BufferTooSmall as c_int;
                        }

                        // Copy data to caller-allocated buffer
                        let out_slice = std::slice::from_raw_parts_mut(out_ptr, capacity);
                        out_slice[..data.len()].copy_from_slice(&data);

                        data.len() as c_int
                    }
                    Err(TryRecvError::Empty) => 0, // No data available
                    Err(TryRecvError::Disconnected) => {
                        log_to_unity("ERROR", "Inbound channel disconnected");
                        ctx.is_connected = false;
                        FfiError::Disconnected as c_int
                    }
                }
            }
            None => FfiError::Disconnected as c_int,
        }
    });

    match result {
        Ok(bytes) => bytes,
        Err(_) => FfiError::PanicCaught as c_int,
    }
}

/// Destroy connection and free resources
///
/// # Safety
///
/// This function is unsafe because it deallocates memory managed by Rust.
/// The `ctx` pointer must:
/// - Be a valid pointer returned by network_connect
/// - Not be null
/// - Not have been previously destroyed (double-free is undefined behavior)
/// - Not be used after this function returns
#[no_mangle]
pub unsafe extern "C" fn network_destroy(ctx: *mut c_void) -> c_int {
    let result = catch_unwind(|| {
        if ctx.is_null() {
            return FfiError::InvalidPointer as c_int;
        }

        let mut ctx = Box::from_raw(ctx as *mut ClientContext);

        log_to_unity("INFO", "Destroying connection");

        // Drop channels (closes them)
        ctx.outbound_tx = None;
        ctx.inbound_rx = None;
        ctx.is_connected = false;

        FfiError::Success as c_int
    });

    match result {
        Ok(code) => code,
        Err(_) => FfiError::PanicCaught as c_int,
    }
}

/// Async connection task
/// Runs in spawned Tokio runtime
async fn connect_async(
    url: String,
    cert_hash: Option<String>,
    outbound_rx: tokio_mpsc::Receiver<Vec<u8>>,
    inbound_tx: tokio_mpsc::Sender<Vec<u8>>,
    ready_tx: tokio::sync::oneshot::Sender<()>,
) {
    info!("Async connection task started for: {}", url);
    info!("Certificate hash provided: {}", cert_hash.is_some());

    // Parse certificate hash if provided
    let _cert_hash_bytes = cert_hash.and_then(|hash| hex::decode(hash).ok());
    info!("Connecting to WebTransport server...");

    // Build client configuration
    // For development/POC, we bypass certificate validation to allow self-signed certs
    let client_config = ClientConfig::builder()
        .with_bind_default()
        .with_no_cert_validation()
        .build();

    log_to_unity(
        "INFO",
        "Using client config with certificate validation bypassed (development mode)",
    );

    // Create endpoint for client connections
    let endpoint = match Endpoint::client(client_config) {
        Ok(ep) => ep,
        Err(e) => {
            error!("Failed to create endpoint: {:?}", e);
            return;
        }
    };

    // Connect to server
    let connection: Connection = match endpoint.connect(url).await {
        Ok(conn) => {
            info!("Connected to server successfully");
            log_to_unity("INFO", "WebTransport connection established");
            conn
        }
        Err(e) => {
            error!("Failed to connect: {:?}", e);
            log_to_unity("ERROR", &format!("Connection failed: {:?}", e));
            let _ = ready_tx.send(()); // Signal connection failed
            return;
        }
    };

    // Receive first to complete QUIC handshake
    // WebTransport datagrams may not work until both sides have received at least once
    info!("Performing initial receive to complete handshake...");
    tokio::select! {
        _ = connection.receive_datagram() => {
            info!("Initial receive completed, datagram stream ready");
        }
        _ = tokio::time::sleep(Duration::from_secs(1)) => {
            info!("Initial receive timeout (no initial packet), proceeding...");
        }
    }

    // Signal that connection is ready
    let _ = ready_tx.send(());
    info!("Connection ready signal sent to FFI");

    // Spawn outbound task
    let outbound_connection = connection.clone();
    tokio::spawn(async move {
        info!("Outbound task started");
        let mut outbound_rx = outbound_rx;
        let mut packets_sent = 0u64;

        loop {
            // Use await on tokio channel - this is non-blocking and allows proper async scheduling
            match outbound_rx.recv().await {
                Some(data) => {
                    if let Err(e) = outbound_connection.send_datagram(data) {
                        error!("Failed to send datagram: {:?}", e);
                        log_to_unity("ERROR", &format!("Send error: {:?}", e));
                        break;
                    }
                    packets_sent += 1;
                    if packets_sent.is_multiple_of(100) {
                        info!("Outbound task: {} packets sent", packets_sent);
                    }
                }
                None => {
                    info!("Outbound channel closed");
                    break;
                }
            }
        }

        info!("Outbound task ended, total packets sent: {}", packets_sent);
    });

    // Spawn inbound task
    tokio::spawn(async move {
        info!("Inbound task started");
        let connection = connection;
        let mut packets_received = 0u64;
        let mut no_data_count: u32 = 0;

        loop {
            // Add timeout to detect if receive_datagram is blocking
            match tokio::time::timeout(Duration::from_secs(1), connection.receive_datagram()).await
            {
                Ok(Ok(data)) => {
                    packets_received += 1;
                    no_data_count = 0; // Reset counter on success

                    if packets_received == 1 {
                        info!(
                            "Inbound task: First packet received, size: {} bytes",
                            data.len()
                        );
                        log_to_unity(
                            "INFO",
                            &format!("First packet received: {} bytes", data.len()),
                        );
                    }
                    if packets_received.is_multiple_of(100) {
                        info!("Inbound task: {} packets received", packets_received);
                    }

                    // Push to inbound queue
                    if inbound_tx.send(data.to_vec()).await.is_err() {
                        error!("Inbound channel closed");
                        log_to_unity("ERROR", "Inbound channel closed");
                        break;
                    }
                }
                Ok(Err(e)) => {
                    error!("Receive error: {:?}", e);
                    log_to_unity("ERROR", &format!("Receive error: {:?}", e));
                    break;
                }
                Err(_) => {
                    // Timeout - receive_datagram is blocking
                    no_data_count += 1;
                    if no_data_count == 1 {
                        warn!(
                            "Inbound task: receive_datagram timed out (1s) - no data received yet"
                        );
                    }
                    if no_data_count.is_multiple_of(10) {
                        warn!(
                            "Inbound task: Still no data after {} seconds",
                            no_data_count
                        );
                    }
                    if no_data_count > 30 {
                        error!("Inbound task: No data for 30 seconds, closing");
                        break;
                    }
                    // Continue loop to try again
                }
            }
        }

        info!(
            "Inbound task ended, total packets received: {}",
            packets_received
        );
    });

    // Keep async task alive
    // In production, you might want a graceful shutdown mechanism
    log_to_unity("INFO", "Connection tasks spawned, keeping connection alive");
    tokio::time::sleep(Duration::from_secs(u64::MAX)).await;

    // Cleanup
    info!("Async connection task terminating");
    log_to_unity("INFO", "Connection closed");
}

/// Helper to log to Unity
fn log_to_unity(level: &str, message: &str) {
    unsafe {
        if let Some(callback) = LOG_CALLBACK {
            let level_cstr = CString::new(level).unwrap();
            let msg_cstr = CString::new(message).unwrap();
            callback(level_cstr.as_ptr(), msg_cstr.as_ptr());
        } else {
            eprintln!("[{}] {}", level, message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_error_codes() {
        assert_eq!(FfiError::Success as i32, 0);
        assert_eq!(FfiError::InvalidPointer as i32, -1);
        assert_eq!(FfiError::PanicCaught as i32, -99);
    }
}
