//! Basic usage example for the game-ffi crate
//!
//! This example demonstrates how to use the GameComponent derive macro
//! to create FFI-compatible types with automatic code generation.

use game_ffi::GameComponent;

/// Basic example: A simple position struct
///
/// The GameComponent derive macro automatically generates:
/// - UUID constant
/// - Zero-copy methods (as_bytes, from_bytes)
/// - Validation methods
/// - FFI wrapper functions
#[derive(GameComponent)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Example with manual UUID assignment
///
/// Use this for types that need stable UUIDs across versions.
#[derive(GameComponent)]
#[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
pub struct GameState {
    pub tick: u32,
    pub player_count: u32,
}

/// Example with field validation attributes
///
/// Field-level attributes like min/max are parsed by the macro.
/// These can be used for validation logic in generated code.
#[derive(GameComponent)]
pub struct EntityUpdate {
    pub entity_id: u64,

    /// Position delta with validation range
    #[field(min = -1000, max = 1000)]
    pub position_delta_x: i16,

    #[field(min = -1000, max = 1000)]
    pub position_delta_y: i16,

    pub hp_delta: i16,
    pub state_flags_delta: u16,
    pub animation_id: u8,

    /// Padding fields can be skipped from public API
    #[field(skip)]
    pub _padding: [u8; 3],
}

/// Example with Unity-specific configuration
///
/// The unity attribute allows customizing the generated C# code.
#[derive(GameComponent)]
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos {
    pub header: PacketHeader,
    pub id: u64,
    pub x: f32,
    pub y: f32,
}

/// Example with Unreal-specific configuration
///
/// The unreal attribute allows customizing the generated C++ code.
#[derive(GameComponent)]
#[unreal(class = "FCharacterUpdate", blueprint_type)]
pub struct CharacterUpdate {
    pub char_id: u64,
    pub x: u16,
    pub y: u16,
}

/// Nested struct example
///
/// Structs can reference other GameComponent types.
#[derive(GameComponent)]
pub struct PacketHeader {
    pub packet_type: u8,
    pub magic: u8,
}

fn main() {
    println!("=== Game FFI Basic Usage Example ===\n");

    // 1. Create a PlayerPosition
    let pos = PlayerPosition {
        x: 100.0,
        y: 200.0,
        z: 300.0,
    };
    println!("Created PlayerPosition: ({}, {}, {})", pos.x, pos.y, pos.z);

    // 2. Access the auto-generated UUID
    println!("\nPlayerPosition UUID: {}", PlayerPosition::UUID);

    // 3. Zero-copy serialization
    let bytes = pos.as_bytes();
    println!("\nZero-copy bytes length: {} bytes", bytes.len());

    // 4. Zero-copy deserialization
    let pos2 = unsafe { PlayerPosition::from_bytes(bytes) };
    println!("Recovered from bytes: ({}, {}, {})", pos2.x, pos2.y, pos2.z);

    // 5. Validation
    match pos.validate() {
        Ok(_) => println!("\n✓ PlayerPosition is valid"),
        Err(e) => println!("\n✗ PlayerPosition validation failed: {}", e),
    }

    // 6. Check if valid (convenience method)
    println!("is_valid(): {}", pos.is_valid());

    // 7. Layout verification
    let layout = PlayerPosition::actual_layout();
    println!("\nLayout info:");
    println!("  Size: {} bytes", layout.size);
    println!("  Alignment: {} bytes", layout.alignment);
    println!("  Packed: {}", layout.is_packed());

    // 8. Use FFI wrapper functions
    println!("\nFFI info:");
    println!("  size(): {}", PlayerPosition::size());
    println!("  alignment(): {}", PlayerPosition::alignment());

    // 9. Create GameState with manual UUID
    let _state = GameState {
        tick: 12345,
        player_count: 10,
    };
    println!("\nGameState UUID: {} (manually assigned)", GameState::UUID);

    // 10. Validate UUID format
    use game_ffi::{uuid_to_label, validate_uuid};
    println!("\nUUID validation:");
    println!(
        "  PlayerPosition UUID valid: {}",
        validate_uuid(PlayerPosition::UUID)
    );
    println!(
        "  GameState label: {}",
        uuid_to_label(&uuid::Uuid::parse_str(GameState::UUID).unwrap())
    );

    // 11. Generate deterministic UUID from signature
    use game_ffi::build_struct_signature_default;
    use game_ffi::generate_uuid_from_signature;

    let fields: [(&str, &str, Vec<&str>); 1] = [("x", "f32", vec![])];
    let signature = build_struct_signature_default("1.0.0", "CustomStruct", &fields);
    let custom_uuid = generate_uuid_from_signature(&signature);
    println!("\nGenerated UUID from signature: {}", custom_uuid);

    // 12. Use UUID with namespace (manual signature construction)
    let ns_signature = "struct:1.0.0:mmorpg.player:Position{{}}";
    let ns_uuid = generate_uuid_from_signature(ns_signature);
    println!("UUID with namespace 'mmorpg.player': {}", ns_uuid);

    // 13. Batch operations example
    println!("\n=== Batch Operations ===");
    let positions = [
        PlayerPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        PlayerPosition {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        PlayerPosition {
            x: 7.0,
            y: 8.0,
            z: 9.0,
        },
    ];

    // Convert to bytes (zero-copy)
    let bytes: Vec<u8> = positions
        .iter()
        .flat_map(|p| p.as_bytes().iter().copied())
        .collect();

    println!(
        "Batch serialized {} positions to {} bytes",
        positions.len(),
        bytes.len()
    );

    // Read back
    let size = std::mem::size_of::<PlayerPosition>();
    println!("Each position is {} bytes", size);

    for (i, _pos) in positions.iter().enumerate() {
        let start = i * size;
        let end = start + size;
        let recovered = unsafe { PlayerPosition::from_bytes(&bytes[start..end]) };
        println!(
            "  Position {}: ({}, {}, {})",
            i, recovered.x, recovered.y, recovered.z
        );
    }

    // 14. Unity bindings (conditional)
    #[cfg(feature = "unity")]
    {
        println!("\n=== Unity C# Bindings ===");
        println!("{}", PlayerPos::UNITY_CS);
    }

    // 15. Unreal bindings (conditional)
    #[cfg(feature = "unreal")]
    {
        println!("\n=== Unreal C++ Bindings ===");
        println!("{}", CharacterUpdate::UNREAL_HPP);
    }

    println!("\n=== Example Complete ===");
}
