//! Complex Patterns Example for game-ffi crate
//!
//! This example demonstrates advanced FFI patterns in the Game FFI system:
//! 1. Nested Components - Structs containing other GameComponent structs
//! 2. Enum Components - Enum types with #[repr(C)]
//! 3. Array Components - Structs with array fields
//! 4. Field Skipping - Fields with #[field(skip)] attribute
//! 5. Engine-Specific Config - Unity and Unreal field-level attributes

use game_ffi::GameComponent;

// ============================================================================
// Pattern 1: Nested Components
// ============================================================================
//
// GameComponent structs can contain other GameComponent structs.
// This is useful for composing complex data structures from simpler ones.

#[repr(C)]
#[derive(GameComponent)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(GameComponent)]
pub struct Rotation3D {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[repr(C)]
#[derive(GameComponent)]
pub struct Transform {
    pub position: Position3D,
    pub rotation: Rotation3D,
}

// ============================================================================
// Pattern 2: Enum Components
// ============================================================================
//
// Enums with #[repr(C)] can be used as FFI types. This is useful for
// representing state machines, animation states, or other discrete values.

#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy, PartialEq)]
pub enum MovementState {
    Idle = 0,
    Walking = 1,
    Running = 2,
    Jumping = 3,
    Falling = 4,
}

#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    None = 0,
    Playing = 1,
    Paused = 2,
    Stopped = 3,
}

// ============================================================================
// Pattern 3: Array Components
// ============================================================================
//
// Structs can contain fixed-size arrays. This is useful for inventory systems,
// entity batches, or other collections of fixed-size data.

#[repr(C)]
#[derive(GameComponent)]
pub struct Inventory {
    pub capacity: u32,
    pub item_count: u32,
    pub item_ids: [u64; 32],
    pub item_quantities: [u32; 32],
}

#[repr(C)]
#[derive(GameComponent)]
pub struct EntityBatch {
    pub tick: u64,
    pub entity_count: u32,
    pub entity_ids: [u64; 128],
    pub positions: [Position3D; 128],
}

// ============================================================================
// Pattern 4: Field Skipping
// ============================================================================
//
// The #[field(skip)] attribute excludes fields from the public FFI API while
// keeping them in memory. This is useful for server-internal state that
// shouldn't be exposed to clients.

#[repr(C)]
#[derive(GameComponent)]
pub struct PlayerState {
    pub health: f32,
    pub mana: f32,
    pub stamina: f32,

    #[field(skip)]
    pub last_server_tick: u64,

    #[field(skip)]
    pub pending_actions_count: u32,

    #[field(skip)]
    pub internal_flags: u32,
}

// ============================================================================
// Pattern 5: Engine-Specific Config
// ============================================================================
//
// Field-level attributes can customize how fields appear in Unity or Unreal.
// This enables engine-specific behavior while maintaining a single source of truth.

#[repr(C)]
#[derive(GameComponent)]
pub struct PlayerEquipment {
    pub weapon_id: u64,

    #[unity(header_field)]
    #[unreal(replicated)]
    pub armor_id: u64,

    #[unity(read_only)]
    pub helmet_id: u64,

    #[unreal(edit_mode = "instance_only")]
    pub shield_id: u64,
}

#[repr(C)]
#[derive(GameComponent)]
#[unity(name = "CharacterInfo")]
#[unreal(class = "FCharacterData", blueprint_type)]
pub struct CharacterData {
    #[unity(name = "CharacterId")]
    pub character_id: u64,

    #[unity(name = "DisplayName")]
    #[unreal(replicated)]
    pub display_name: [u8; 64],

    #[unity(name = "Level")]
    pub level: u8,

    #[unity(name = "Experience")]
    pub experience: u32,
}

// ============================================================================
// Demonstration Functions
// ============================================================================

fn demo_nested_components() {
    println!("\n=== Nested Components Demo ===");

    let transform = Transform {
        position: Position3D {
            x: 10.0,
            y: 20.0,
            z: 30.0,
        },
        rotation: Rotation3D {
            pitch: 0.5,
            yaw: 1.5,
            roll: 2.5,
        },
    };

    println!("Transform UUID: {}", Transform::UUID);
    println!("Transform size: {} bytes", Transform::size());
    println!(
        "Position: ({}, {}, {})",
        transform.position.x, transform.position.y, transform.position.z
    );
    println!(
        "Rotation: ({}, {}, {})",
        transform.rotation.pitch, transform.rotation.yaw, transform.rotation.roll
    );

    // Zero-copy serialization
    let bytes = transform.as_bytes();
    let recovered = unsafe { Transform::from_bytes(bytes) };
    println!(
        "Recovered position: ({}, {}, {})",
        recovered.position.x, recovered.position.y, recovered.position.z
    );
}

fn demo_enum_components() {
    println!("\n=== Enum Components Demo ===");

    let state = MovementState::Running;
    let animation = AnimationState::Playing;

    println!("MovementState UUID: {}", MovementState::UUID);
    println!("AnimationState UUID: {}", AnimationState::UUID);
    println!("MovementState::Running as i32: {}", state as i32);
    println!("AnimationState::Playing as i32: {}", animation as i32);

    // Zero-copy for enums
    let bytes = state.as_bytes();
    println!("Enum bytes length: {}", bytes.len());

    let recovered = unsafe { MovementState::from_bytes(bytes) };
    println!("Recovered enum: {:?}", recovered);
}

fn demo_array_components() {
    println!("\n=== Array Components Demo ===");

    let mut item_ids = [0u64; 32];
    item_ids[0] = 100;
    item_ids[1] = 200;
    item_ids[2] = 300;

    let mut item_quantities = [0u32; 32];
    item_quantities[0] = 5;
    item_quantities[1] = 10;
    item_quantities[2] = 3;

    let inventory = Inventory {
        capacity: 32,
        item_count: 3,
        item_ids,
        item_quantities,
    };

    println!("Inventory UUID: {}", Inventory::UUID);
    println!("Inventory size: {} bytes", Inventory::size());
    println!("Inventory capacity: {}", inventory.capacity);
    println!("Inventory item count: {}", inventory.item_count);
    println!("First 3 items:");
    for i in 0..3 {
        println!(
            "  Item {}: id={}, quantity={}",
            i, inventory.item_ids[i], inventory.item_quantities[i]
        );
    }
}

fn demo_field_skipping() {
    println!("\n=== Field Skipping Demo ===");

    let player_state = PlayerState {
        health: 100.0,
        mana: 50.0,
        stamina: 75.0,
        last_server_tick: 12345,
        pending_actions_count: 3,
        internal_flags: 0xFF,
    };

    println!("PlayerState UUID: {}", PlayerState::UUID);
    println!("PlayerState size: {} bytes", PlayerState::size());
    println!("Public fields (health, mana, stamina):");
    println!("  Health: {}", player_state.health);
    println!("  Mana: {}", player_state.mana);
    println!("  Stamina: {}", player_state.stamina);

    // Note: Skipped fields are still in memory and accessible in Rust,
    // but they're excluded from the public FFI API
    println!("Internal fields (should not be exposed to FFI):");
    println!("  Last server tick: {}", player_state.last_server_tick);
    println!("  Pending actions: {}", player_state.pending_actions_count);
    println!("  Internal flags: 0x{:X}", player_state.internal_flags);

    // Zero-copy includes all fields (skipped fields are still in memory)
    let bytes = player_state.as_bytes();
    let recovered = unsafe { PlayerState::from_bytes(bytes) };
    println!("Recovered internal tick: {}", recovered.last_server_tick);
}

fn demo_engine_specific_config() {
    println!("\n=== Engine-Specific Config Demo ===");

    let equipment = PlayerEquipment {
        weapon_id: 1000,
        armor_id: 2000,
        helmet_id: 3000,
        shield_id: 4000,
    };

    let character_data = CharacterData {
        character_id: 12345,
        display_name: {
            let mut name = [0u8; 64];
            name[.."Hero123".len()].copy_from_slice("Hero123".as_bytes());
            name
        },
        level: 42,
        experience: 50000,
    };

    println!("PlayerEquipment UUID: {}", PlayerEquipment::UUID);
    println!("CharacterData UUID: {}", CharacterData::UUID);

    println!("Equipment:");
    println!("  Weapon ID: {}", equipment.weapon_id);
    println!(
        "  Armor ID (header_field, replicated): {}",
        equipment.armor_id
    );
    println!("  Helmet ID (read_only): {}", equipment.helmet_id);
    println!("  Shield ID (instance_only): {}", equipment.shield_id);

    println!("Character Data:");
    println!("  Character ID: {}", character_data.character_id);
    let name = std::str::from_utf8(&character_data.display_name)
        .unwrap_or("")
        .trim_end_matches('\0');
    println!("  Display Name: {}", name);
    println!("  Level: {}", character_data.level);
    println!("  Experience: {}", character_data.experience);

    // Display engine-specific bindings (if features are enabled)
    #[cfg(feature = "unity")]
    {
        println!("\n=== Unity C# Bindings ===");
        println!("{}", PlayerEquipment::UNITY_CS);
    }

    #[cfg(feature = "unreal")]
    {
        println!("\n=== Unreal C++ Bindings ===");
        println!("{}", PlayerEquipment::UNREAL_HPP);
    }
}

fn main() {
    println!("=== Game FFI Complex Patterns Example ===");

    demo_nested_components();
    demo_enum_components();
    demo_array_components();
    demo_field_skipping();
    demo_engine_specific_config();

    println!("\n=== Example Complete ===");

    // Summary of all patterns
    println!("\n=== Summary ===");
    println!("All patterns support:");
    println!("  ✓ UUID generation (auto-generated v7)");
    println!("  ✓ Memory layout verification");
    println!("  ✓ Zero-copy operations");
    println!("  ✓ Validation methods");
    println!("  ✓ Engine-specific bindings (Unity/Unreal)");

    // Verify all UUIDs are valid v7
    let uuids = vec![
        Transform::UUID,
        MovementState::UUID,
        AnimationState::UUID,
        Inventory::UUID,
        EntityBatch::UUID,
        PlayerState::UUID,
        PlayerEquipment::UUID,
        CharacterData::UUID,
    ];

    println!("\n=== UUID Validation ===");
    for uuid_str in &uuids {
        let uuid = uuid::Uuid::parse_str(uuid_str).unwrap();
        assert_eq!(uuid.get_version_num(), 7);
        println!("  ✓ {} (v7)", uuid_str);
    }
}
