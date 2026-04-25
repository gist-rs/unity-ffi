//! Packet builder FFI module
//!
//! Provides high-level packet construction functions for Unity.
//! All UUID generation and packet construction happens in Rust.
//! Unity only provides business data (position, ID, etc.).

use crate::types::{GameState, PacketType, PlayerPos, SpriteMessage, SpriteOp, SpriteType};
use uuid::Uuid;

/// Error codes for packet builder operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketBuilderError {
    Success = 0,
    InvalidPointer = -1,
    BufferTooSmall = -2,
    InvalidPacketType = -3,
    PanicCaught = -99,
}

impl PacketBuilderError {
    #[must_use]
    pub const fn as_int(self) -> i32 {
        self as i32
    }
}

/// Create a PlayerPos packet with auto-generated UUID v7
///
/// # Arguments
/// * `id_ptr` - Player/character UUID as 16 bytes
/// * `x` - X position
/// * `y` - Y position
/// * `out_ptr` - Output buffer pointer
/// * `capacity` - Output buffer capacity (must be at least size_of::<PlayerPos>())
///
/// # Returns
/// Number of bytes written, or negative error code:
/// - Positive: Number of bytes written
/// - -1: Null pointer
/// - -2: Buffer too small
/// - -99: Panic caught
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The `out_ptr` pointer must be:
/// - Valid for writing
/// - Point to at least `capacity` bytes of writable memory
/// - Properly aligned for u8
#[no_mangle]
pub unsafe extern "C" fn packet_builder_create_player_pos(
    id_ptr: *const u8,
    x: i32,
    y: i32,
    out_ptr: *mut u8,
    capacity: usize,
) -> i32 {
    let result = std::panic::catch_unwind(|| {
        if out_ptr.is_null() || id_ptr.is_null() {
            return PacketBuilderError::InvalidPointer.as_int();
        }

        // Generate UUID v7 internally - Unity doesn't need to know about it
        let request_uuid = Uuid::now_v7();

        // Copy player UUID from input and convert to UUID
        let player_uuid = Uuid::from_bytes({
            let mut bytes = [0u8; 16];
            std::ptr::copy_nonoverlapping(id_ptr, bytes.as_mut_ptr(), 16);
            bytes
        });

        // Create PlayerPos packet with flattened structure
        let packet = PlayerPos {
            packet_type: PacketType::PlayerPos as u8,
            magic: 0xCC,
            request_uuid,
            pos: crate::types::Position2D {
                player_id: player_uuid.as_u128() as u64,
                x: x as f32,
                y: y as f32,
            },
        };

        // Use GameComponent's as_bytes() method for zero-copy conversion
        let packet_bytes = packet.as_bytes();

        if packet_bytes.len() > capacity {
            return PacketBuilderError::BufferTooSmall.as_int();
        }

        // Copy to output buffer
        std::ptr::copy_nonoverlapping(packet_bytes.as_ptr(), out_ptr, packet_bytes.len());

        packet_bytes.len() as i32
    });

    match result {
        Ok(bytes) => bytes,
        Err(_) => PacketBuilderError::PanicCaught.as_int(),
    }
}

/// Create a GameState packet with auto-generated UUID v7
///
/// # Arguments
/// * `tick` - Server tick or timestamp
/// * `player_count` - Number of players or message type
/// * `out_ptr` - Output buffer pointer
/// * `capacity` - Output buffer capacity (must be at least size_of::<GameState>())
///
/// # Returns
/// Number of bytes written, or negative error code
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn packet_builder_create_game_state(
    tick: u32,
    player_count: u32,
    out_ptr: *mut u8,
    capacity: usize,
) -> i32 {
    let result = std::panic::catch_unwind(|| {
        if out_ptr.is_null() {
            return PacketBuilderError::InvalidPointer.as_int();
        }

        // Create GameState packet with flattened structure
        let packet = GameState {
            packet_type: PacketType::GameState as u8,
            magic: 0xCC,
            tick,
            player_count,
            reserved: [0; 8],
        };

        // Use GameComponent's as_bytes() method for zero-copy conversion
        let packet_bytes = packet.as_bytes();

        if packet_bytes.len() > capacity {
            return PacketBuilderError::BufferTooSmall.as_int();
        }

        // Copy to output buffer
        std::ptr::copy_nonoverlapping(packet_bytes.as_ptr(), out_ptr, packet_bytes.len());

        packet_bytes.len() as i32
    });

    match result {
        Ok(bytes) => bytes,
        Err(_) => PacketBuilderError::PanicCaught.as_int(),
    }
}

/// Create a SpriteMessage packet with auto-generated UUID v7
///
/// # Arguments
/// * `operation` - Sprite operation type (0=Create, 1=Update, 2=Delete, 3=Snapshot)
/// * `sprite_type` - Sprite type (0=Serrif)
/// * `id_ptr` - Sprite UUID as 16 bytes
/// * `x` - X position
/// * `y` - Y position
/// * `out_ptr` - Output buffer pointer
/// * `capacity` - Output buffer capacity (must be at least size_of::<SpriteMessage>())
///
/// # Returns
/// Number of bytes written, or negative error code
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn packet_builder_create_sprite_message(
    operation: u8,
    sprite_type: u8,
    id_ptr: *const u8,
    x: i16,
    y: i16,
    out_ptr: *mut u8,
    capacity: usize,
) -> i32 {
    let result = std::panic::catch_unwind(|| {
        if out_ptr.is_null() || id_ptr.is_null() {
            return PacketBuilderError::InvalidPointer.as_int();
        }

        // Copy sprite UUID from input
        let id = Uuid::from_bytes({
            let mut bytes = [0u8; 16];
            std::ptr::copy_nonoverlapping(id_ptr, bytes.as_mut_ptr(), 16);
            bytes
        });

        // Create SpriteMessage packet with flattened structure
        let packet = SpriteMessage::new(
            SpriteOp::from_u8(operation).unwrap_or(SpriteOp::Create),
            SpriteType::from_u8(sprite_type).unwrap_or(SpriteType::Serrif),
            id,
            x,
            y,
        );

        // Use GameComponent's as_bytes() method for zero-copy conversion
        let packet_bytes = packet.as_bytes();

        if packet_bytes.len() > capacity {
            return PacketBuilderError::BufferTooSmall.as_int();
        }

        // Copy to output buffer
        std::ptr::copy_nonoverlapping(packet_bytes.as_ptr(), out_ptr, packet_bytes.len());

        packet_bytes.len() as i32
    });

    match result {
        Ok(bytes) => bytes,
        Err(_) => PacketBuilderError::PanicCaught.as_int(),
    }
}

/// Create an Authenticate packet with auto-generated UUID v7
///
/// Note: Authenticate packet type is not in the current PacketType enum,
/// so this function returns an error for now.
///
/// # Arguments
/// * `user_id_ptr` - User UUID as 16 bytes
/// * `out_ptr` - Output buffer pointer
/// * `capacity` - Output buffer capacity
///
/// # Returns
/// Number of bytes written, or negative error code:
/// - -3: Invalid packet type (Authenticate not implemented)
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn packet_builder_create_authenticate(
    user_id_ptr: *const u8,
    out_ptr: *mut u8,
    _capacity: usize,
) -> i32 {
    if user_id_ptr.is_null() || out_ptr.is_null() {
        return PacketBuilderError::InvalidPointer.as_int();
    }

    // Authenticate packet type is not implemented in current PacketType enum
    PacketBuilderError::InvalidPacketType.as_int()
}

/// Create a KeepAlive packet
///
/// # Arguments
/// * `out_ptr` - Output buffer pointer
/// * `capacity` - Output buffer capacity (must be at least 2 bytes for header)
///
/// # Returns
/// Number of bytes written, or negative error code
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn packet_builder_create_keep_alive(
    out_ptr: *mut u8,
    capacity: usize,
) -> i32 {
    let result = std::panic::catch_unwind(|| {
        if out_ptr.is_null() {
            return PacketBuilderError::InvalidPointer.as_int();
        }

        // KeepAlive is just the header (2 bytes)
        let packet = [
            PacketType::KeepAlive as u8,
            0xCC, // magic
        ];

        if packet.len() > capacity {
            return PacketBuilderError::BufferTooSmall.as_int();
        }

        // Copy to output buffer
        std::ptr::copy_nonoverlapping(packet.as_ptr(), out_ptr, packet.len());

        packet.len() as i32
    });

    match result {
        Ok(bytes) => bytes,
        Err(_) => PacketBuilderError::PanicCaught.as_int(),
    }
}

/// Get error string from error code
///
/// # Arguments
/// * `error_code` - Error code returned by packet builder functions
///
/// # Returns
/// Pointer to static C string describing the error
///
/// # Safety
///
/// The returned pointer is valid for the lifetime of the program.
#[no_mangle]
pub extern "C" fn packet_builder_get_error_string(error_code: i32) -> *const std::ffi::c_char {
    static SUCCESS: &[u8] = b"Success\0";
    static INVALID_POINTER: &[u8] = b"Invalid pointer (null pointer passed to FFI)\0";
    static BUFFER_TOO_SMALL: &[u8] =
        b"Buffer too small (internal buffer insufficient for packet)\0";
    static INVALID_PACKET_TYPE: &[u8] = b"Invalid packet type\0";
    static PANIC_CAUGHT: &[u8] = b"Panic caught in Rust code (internal error)\0";
    static UNKNOWN: &[u8] = b"Unknown error code\0";

    let str_ptr = match error_code {
        0 => SUCCESS.as_ptr(),
        -1 => INVALID_POINTER.as_ptr(),
        -2 => BUFFER_TOO_SMALL.as_ptr(),
        -3 => INVALID_PACKET_TYPE.as_ptr(),
        -99 => PANIC_CAUGHT.as_ptr(),
        _ => UNKNOWN.as_ptr(),
    };

    str_ptr as *const std::ffi::c_char
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_builder_create_player_pos() {
        let mut buffer = [0u8; 128];
        let player_uuid = Uuid::now_v7();

        unsafe {
            let result = packet_builder_create_player_pos(
                player_uuid.as_bytes().as_ptr(),
                100,
                200,
                buffer.as_mut_ptr(),
                buffer.len(),
            );

            assert!(result > 0);

            // Parse back using GameComponent's from_bytes()
            let packet = PlayerPos::from_bytes(&buffer[..result as usize]);
            assert!(packet.validate().is_ok());
            assert_eq!(packet.packet_type, PacketType::PlayerPos as u8);
            assert_eq!(packet.pos.x, 100.0);
            assert_eq!(packet.pos.y, 200.0);
        }
    }

    #[test]
    fn test_packet_builder_create_game_state() {
        let mut buffer = [0u8; 128];

        unsafe {
            let result =
                packet_builder_create_game_state(12345, 42, buffer.as_mut_ptr(), buffer.len());

            assert!(result > 0);

            // Parse back using GameComponent's from_bytes()
            let packet = GameState::from_bytes(&buffer[..result as usize]);
            assert!(packet.validate().is_ok());
            assert_eq!(packet.packet_type, PacketType::GameState as u8);
            assert_eq!(packet.tick, 12345);
            assert_eq!(packet.player_count, 42);
        }
    }

    #[test]
    fn test_packet_builder_create_sprite_message() {
        let mut buffer = [0u8; 128];
        let sprite_uuid = Uuid::now_v7();

        unsafe {
            let result = packet_builder_create_sprite_message(
                0, // Create
                0, // Serrif
                sprite_uuid.as_bytes().as_ptr(),
                50,
                75,
                buffer.as_mut_ptr(),
                buffer.len(),
            );

            assert!(result > 0);

            // Parse back using GameComponent's from_bytes()
            let packet = SpriteMessage::from_bytes(&buffer[..result as usize]);
            assert!(packet.validate().is_ok());
            assert_eq!(packet.packet_type, PacketType::SpriteMessage as u8);
            assert_eq!(packet.operation, 0);
            assert_eq!(packet.sprite_type, 0);
            assert_eq!(packet.x, 50);
            assert_eq!(packet.y, 75);
            assert_eq!(packet.get_id(), sprite_uuid);
        }
    }

    #[test]
    fn test_packet_builder_create_authenticate() {
        let mut buffer = [0u8; 128];
        let user_uuid = Uuid::now_v7();

        unsafe {
            let result = packet_builder_create_authenticate(
                user_uuid.as_bytes().as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len(),
            );

            // Should return error -3 (InvalidPacketType) since Authenticate is not implemented
            assert_eq!(result, PacketBuilderError::InvalidPacketType.as_int());
        }
    }

    #[test]
    fn test_packet_builder_create_keep_alive() {
        let mut buffer = [0u8; 128];

        unsafe {
            let result = packet_builder_create_keep_alive(buffer.as_mut_ptr(), buffer.len());

            assert_eq!(result, 2);
            assert_eq!(buffer[0], PacketType::KeepAlive as u8);
            assert_eq!(buffer[1], 0xCC);
        }
    }

    #[test]
    fn test_packet_builder_null_pointer() {
        let mut buffer = [0u8; 128];

        unsafe {
            // Test null out_ptr
            let result = packet_builder_create_player_pos(
                std::ptr::null(),
                0,
                0,
                std::ptr::null_mut(),
                buffer.len(),
            );
            assert_eq!(result, PacketBuilderError::InvalidPointer.as_int());

            // Test null id_ptr
            let result = packet_builder_create_player_pos(
                std::ptr::null(),
                0,
                0,
                buffer.as_mut_ptr(),
                buffer.len(),
            );
            assert_eq!(result, PacketBuilderError::InvalidPointer.as_int());

            // Test null out_ptr for other functions
            let result = packet_builder_create_game_state(0, 0, std::ptr::null_mut(), buffer.len());
            assert_eq!(result, PacketBuilderError::InvalidPointer.as_int());
        }
    }

    #[test]
    fn test_packet_builder_buffer_too_small() {
        let mut buffer = [0u8; 2]; // Too small for any real packet
        let player_uuid = Uuid::now_v7();

        unsafe {
            let result = packet_builder_create_player_pos(
                player_uuid.as_bytes().as_ptr(),
                0,
                0,
                buffer.as_mut_ptr(),
                buffer.len(),
            );
            assert_eq!(result, PacketBuilderError::BufferTooSmall.as_int());
        }
    }

    #[test]
    fn test_packet_builder_uuid_uniqueness() {
        let mut buffer1 = [0u8; 128];
        let mut buffer2 = [0u8; 128];
        let player_uuid = Uuid::now_v7();

        unsafe {
            // Create two packets with same player UUID
            let _ = packet_builder_create_player_pos(
                player_uuid.as_bytes().as_ptr(),
                100,
                200,
                buffer1.as_mut_ptr(),
                buffer1.len(),
            );
            let _ = packet_builder_create_player_pos(
                player_uuid.as_bytes().as_ptr(),
                100,
                200,
                buffer2.as_mut_ptr(),
                buffer2.len(),
            );

            // Parse both packets
            let packet1 = PlayerPos::from_bytes(&buffer1[..PlayerPos::size()]);
            let packet2 = PlayerPos::from_bytes(&buffer2[..PlayerPos::size()]);

            // request_uuid should be different (auto-generated UUID v7)
            assert_ne!(packet1.request_uuid, packet2.request_uuid);
        }
    }
}
