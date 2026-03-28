//! Phase 5: Complex FFI Patterns Tests
//!
//! This module tests the implementation of complex patterns in the Game FFI system:
//! 1. Nested Components - Structs containing other GameComponent structs
//! 2. Enum Components - Enum types with #[repr(C)]
//! 3. Array Components - Structs with array fields
//! 4. Field Skipping - Fields with #[field(skip)] attribute
//! 5. Engine-Specific Config - Unity and Unreal field-level attributes

use game_ffi::GameComponent;
use uuid::Uuid;

// ============================================================================
// Pattern 1: Nested Components
// ============================================================================

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

#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy)]
pub enum MovementState {
    Idle = 0,
    Walking = 1,
    Running = 2,
    Jumping = 3,
    Falling = 4,
}

// ============================================================================
// Pattern 3: Array Components
// ============================================================================

#[repr(C)]
#[derive(GameComponent)]
pub struct Inventory {
    pub capacity: u32,
    pub item_count: u32,
    pub item_ids: [u64; 32], // Using u64 instead of Uuid for FFI simplicity
    pub item_quantities: [u32; 32],
}

// ============================================================================
// Pattern 4: Field Skipping
// ============================================================================

#[repr(C)]
#[derive(GameComponent)]
pub struct PlayerState {
    pub health: f32,
    pub mana: f32,

    #[field(skip)]
    pub last_server_tick: u64,

    #[field(skip)]
    pub pending_actions_count: u32,
}

// ============================================================================
// Pattern 5: Engine-Specific Config
// ============================================================================

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

// ============================================================================
// Tests
// ============================================================================

mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Nested Components Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_nested_components_uuids() {
        // All UUIDs should be auto-generated and valid UUID v7
        let _ = uuid::Uuid::parse_str(Transform::UUID).unwrap();
        let _ = uuid::Uuid::parse_str(Position3D::UUID).unwrap();
        let _ = uuid::Uuid::parse_str(Rotation3D::UUID).unwrap();

        // Verify they're all v7
        assert_eq!(
            uuid::Uuid::parse_str(Transform::UUID)
                .unwrap()
                .get_version_num(),
            7
        );
        assert_eq!(
            uuid::Uuid::parse_str(Position3D::UUID)
                .unwrap()
                .get_version_num(),
            7
        );
        assert_eq!(
            uuid::Uuid::parse_str(Rotation3D::UUID)
                .unwrap()
                .get_version_num(),
            7
        );
    }

    #[test]
    fn test_nested_components_layout() {
        // Position3D: 3 * f32 = 12 bytes
        assert_eq!(Position3D::size(), 12);
        assert_eq!(Position3D::alignment(), 4);

        // Rotation3D: 3 * f32 = 12 bytes
        assert_eq!(Rotation3D::size(), 12);
        assert_eq!(Rotation3D::alignment(), 4);

        // Transform: 2 * 12 = 24 bytes
        assert_eq!(Transform::size(), 24);
        assert_eq!(Transform::alignment(), 4);
    }

    #[test]
    fn test_nested_components_zero_copy() {
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

        let bytes = transform.as_bytes();
        assert_eq!(bytes.len(), 24);

        let recovered = unsafe { Transform::from_bytes(bytes) };
        assert_eq!(recovered.position.x, 10.0);
        assert_eq!(recovered.position.y, 20.0);
        assert_eq!(recovered.position.z, 30.0);
        assert_eq!(recovered.rotation.pitch, 0.5);
        assert_eq!(recovered.rotation.yaw, 1.5);
        assert_eq!(recovered.rotation.roll, 2.5);
    }

    #[test]
    fn test_nested_components_validation() {
        let transform = Transform {
            position: Position3D {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            rotation: Rotation3D {
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
            },
        };

        assert!(transform.is_valid());
        assert!(transform.validate().is_ok());
    }

    // ------------------------------------------------------------------------
    // Enum Components Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_enum_component_uuid() {
        // Should be auto-generated UUID v7
        let uuid = uuid::Uuid::parse_str(MovementState::UUID).unwrap();
        assert_eq!(uuid.get_version_num(), 7);
    }

    #[test]
    fn test_enum_component_size() {
        // Enum with #[repr(C)] should be 4 bytes (size of largest variant + discriminant)
        // Actually, since all variants are unit-like (no fields), it's just the discriminant
        // which is typically i32 (4 bytes)
        let size = std::mem::size_of::<MovementState>();
        assert_eq!(MovementState::size(), size);
    }

    #[test]
    fn test_enum_component_values() {
        assert_eq!(MovementState::Idle as i32, 0);
        assert_eq!(MovementState::Walking as i32, 1);
        assert_eq!(MovementState::Running as i32, 2);
        assert_eq!(MovementState::Jumping as i32, 3);
        assert_eq!(MovementState::Falling as i32, 4);
    }

    #[test]
    fn test_enum_component_zero_copy() {
        let state = MovementState::Running;

        let bytes = state.as_bytes();
        assert!(!bytes.is_empty());

        let recovered = unsafe { MovementState::from_bytes(bytes) };
        assert_eq!(*recovered as i32, MovementState::Running as i32);
    }

    // ------------------------------------------------------------------------
    // Array Components Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_array_component_uuid() {
        // Should be auto-generated UUID v7
        let uuid = uuid::Uuid::parse_str(Inventory::UUID).unwrap();
        assert_eq!(uuid.get_version_num(), 7);
    }

    #[test]
    fn test_array_component_layout() {
        // capacity: u32 (4) + item_count: u32 (4) = 8
        // item_ids: [u64; 32] = 256
        // item_quantities: [u32; 32] = 128
        // Total: 8 + 256 + 128 = 392 bytes
        let expected_size = 4 + 4 + (8 * 32) + (4 * 32);
        assert_eq!(Inventory::size(), expected_size);

        // Alignment should be 8 (u64 alignment)
        assert_eq!(Inventory::alignment(), 8);
    }

    #[test]
    fn test_array_component_zero_copy() {
        let mut item_ids = [0u64; 32];
        item_ids[0] = 100;
        item_ids[1] = 200;

        let mut item_quantities = [0u32; 32];
        item_quantities[0] = 5;
        item_quantities[1] = 10;

        let inventory = Inventory {
            capacity: 32,
            item_count: 2,
            item_ids,
            item_quantities,
        };

        let bytes = inventory.as_bytes();
        assert_eq!(bytes.len(), 392);

        let recovered = unsafe { Inventory::from_bytes(bytes) };
        assert_eq!(recovered.capacity, 32);
        assert_eq!(recovered.item_count, 2);
        assert_eq!(recovered.item_ids[0], 100);
        assert_eq!(recovered.item_ids[1], 200);
        assert_eq!(recovered.item_quantities[0], 5);
        assert_eq!(recovered.item_quantities[1], 10);
    }

    #[test]
    fn test_array_component_validation() {
        let inventory = Inventory {
            capacity: 32,
            item_count: 0,
            item_ids: [0u64; 32],
            item_quantities: [0u32; 32],
        };

        assert!(inventory.is_valid());
        assert!(inventory.validate().is_ok());
    }

    // ------------------------------------------------------------------------
    // Field Skipping Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_field_skip_uuid() {
        // Should be auto-generated UUID v7
        let uuid = uuid::Uuid::parse_str(PlayerState::UUID).unwrap();
        assert_eq!(uuid.get_version_num(), 7);
    }

    #[test]
    fn test_field_skip_layout() {
        // health: f32 (4) + mana: f32 (4) + last_server_tick: u64 (8) + pending_actions_count: u32 (4) = 24 bytes
        // Note: Skipped fields still affect memory layout in Rust (FFI compatibility)
        // They are excluded from public API but remain in memory
        let expected_size = 24;
        assert_eq!(PlayerState::size(), expected_size);

        // Alignment should be 8 (u64 alignment)
        assert_eq!(PlayerState::alignment(), 8);
    }

    #[test]
    fn test_field_skip_zero_copy() {
        let state = PlayerState {
            health: 100.0,
            mana: 50.0,
            last_server_tick: 12345,
            pending_actions_count: 3,
        };

        let bytes = state.as_bytes();
        assert_eq!(bytes.len(), 24);

        let recovered = unsafe { PlayerState::from_bytes(bytes) };
        // All fields should be recovered (skip doesn't affect zero-copy)
        assert_eq!(recovered.health, 100.0);
        assert_eq!(recovered.mana, 50.0);
        assert_eq!(recovered.last_server_tick, 12345);
        assert_eq!(recovered.pending_actions_count, 3);
    }

    #[test]
    fn test_field_skip_validation() {
        let state = PlayerState {
            health: 100.0,
            mana: 50.0,
            last_server_tick: 12345,
            pending_actions_count: 3,
        };

        assert!(state.is_valid());
        assert!(state.validate().is_ok());
    }

    // ------------------------------------------------------------------------
    // Engine-Specific Config Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_engine_config_uuid() {
        // Should be auto-generated UUID v7
        let uuid = uuid::Uuid::parse_str(PlayerEquipment::UUID).unwrap();
        assert_eq!(uuid.get_version_num(), 7);
    }

    #[test]
    fn test_engine_config_layout() {
        // weapon_id: u64 (8) + armor_id: u64 (8) + helmet_id: u64 (8) + shield_id: u64 (8) = 32 bytes
        let expected_size = 8 * 4;
        assert_eq!(PlayerEquipment::size(), expected_size);

        // Alignment should be 8 (u64 alignment)
        assert_eq!(PlayerEquipment::alignment(), 8);
    }

    #[test]
    fn test_engine_config_zero_copy() {
        let equipment = PlayerEquipment {
            weapon_id: 1000,
            armor_id: 2000,
            helmet_id: 3000,
            shield_id: 4000,
        };

        let bytes = equipment.as_bytes();
        assert_eq!(bytes.len(), 32);

        let recovered = unsafe { PlayerEquipment::from_bytes(bytes) };
        assert_eq!(recovered.weapon_id, 1000);
        assert_eq!(recovered.armor_id, 2000);
        assert_eq!(recovered.helmet_id, 3000);
        assert_eq!(recovered.shield_id, 4000);
    }

    #[test]
    fn test_engine_config_validation() {
        let equipment = PlayerEquipment {
            weapon_id: 1,
            armor_id: 2,
            helmet_id: 3,
            shield_id: 4,
        };

        assert!(equipment.is_valid());
        assert!(equipment.validate().is_ok());
    }

    #[cfg(feature = "unity")]
    #[test]
    fn test_unity_bindings_generated() {
        // Unity bindings should contain engine-specific attributes
        let bindings = PlayerEquipment::generate_unity_cs();
        assert!(bindings.contains("StructLayout"));
    }

    #[cfg(feature = "unreal")]
    #[test]
    fn test_unreal_bindings_generated() {
        // Unreal bindings should contain engine-specific attributes
        let bindings = PlayerEquipment::UNREAL_HPP;
        assert!(bindings.contains("USTRUCT"));
    }

    // ------------------------------------------------------------------------
    // Integration Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_all_patterns_generate_valid_uuids() {
        // All auto-generated UUIDs should be valid v7
        let uuids: Vec<&str> = vec![
            Transform::UUID,
            MovementState::UUID,
            Inventory::UUID,
            PlayerState::UUID,
            PlayerEquipment::UUID,
        ];

        for uuid_str in &uuids {
            let uuid = Uuid::parse_str(uuid_str).unwrap();
            assert_eq!(uuid.get_version_num(), 7);
        }

        // All UUIDs should be unique
        let uuid_set: std::collections::HashSet<_> = uuids.into_iter().collect();
        assert_eq!(uuid_set.len(), 5);
    }

    #[test]
    fn test_all_patterns_have_zero_copy() {
        // All patterns should support zero-copy operations
        let transform = Transform {
            position: Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: Rotation3D {
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
            },
        };
        assert!(!transform.as_bytes().is_empty());

        let movement = MovementState::Idle;
        assert!(!movement.as_bytes().is_empty());

        let inventory = Inventory {
            capacity: 0,
            item_count: 0,
            item_ids: [0; 32],
            item_quantities: [0; 32],
        };
        assert!(!inventory.as_bytes().is_empty());

        let state = PlayerState {
            health: 0.0,
            mana: 0.0,
            last_server_tick: 0,
            pending_actions_count: 0,
        };
        assert!(!state.as_bytes().is_empty());

        let equipment = PlayerEquipment {
            weapon_id: 0,
            armor_id: 0,
            helmet_id: 0,
            shield_id: 0,
        };
        assert!(!equipment.as_bytes().is_empty());
    }

    #[test]
    fn test_all_patterns_have_validation() {
        // All patterns should support validation
        let transform = Transform {
            position: Position3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            rotation: Rotation3D {
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
            },
        };
        assert!(transform.is_valid());

        let movement = MovementState::Idle;
        assert!(movement.is_valid());

        let inventory = Inventory {
            capacity: 0,
            item_count: 0,
            item_ids: [0; 32],
            item_quantities: [0; 32],
        };
        assert!(inventory.is_valid());

        let state = PlayerState {
            health: 0.0,
            mana: 0.0,
            last_server_tick: 0,
            pending_actions_count: 0,
        };
        assert!(state.is_valid());

        let equipment = PlayerEquipment {
            weapon_id: 0,
            armor_id: 0,
            helmet_id: 0,
            shield_id: 0,
        };
        assert!(equipment.is_valid());
    }
}
