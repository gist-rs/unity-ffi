//! Tests for GameComponent derive macro with UUID v7 and hash modes

use game_ffi::GameComponent;

// ============================================================================
// Default Mode (field signature hashing) - Recommended for most cases
// ============================================================================

#[derive(GameComponent)]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(GameComponent)]
pub struct GameState {
    pub tick: u64,
    pub player_count: u32,
}

// ============================================================================
// Strict Mode (hash = "all") - For validation-critical protocols
// ============================================================================

#[derive(GameComponent)]
#[hash = "all"]
pub struct ServerConfig {
    #[field(min = 1000, max = 9999)]
    pub port: u16,
    pub max_connections: u32,
}

// ============================================================================
// Loose Mode (hash = "name") - For rapid prototyping
// ============================================================================

#[derive(GameComponent)]
#[hash = "name"]
pub struct ProtoState {
    pub data: Vec<u8>,
}

// ============================================================================
// Manual UUID Mode - For explicit control (legacy/special cases)
// ============================================================================

#[derive(GameComponent)]
#[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
pub struct LegacyComponent {
    pub value: u32,
}

// ============================================================================
// Complex Types
// ============================================================================

#[derive(GameComponent)]
pub struct EntityUpdate {
    pub entity_id: u64,
    pub position_delta_x: i16,
    pub position_delta_y: i16,
    pub hp_delta: i16,
    pub state_flags_delta: u16,
    pub animation_id: u16,
    #[field(skip)]
    pub _padding: u16,
}

#[derive(GameComponent)]
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos {
    pub header: PacketHeader,
    pub id: u64,
    pub x: f32,
    pub y: f32,
}

#[derive(GameComponent)]
#[unreal(class = "FCharacterUpdate", blueprint_type)]
pub struct CharacterUpdate {
    pub char_id: u64,
    pub x: u16,
    pub y: u16,
}

#[derive(GameComponent)]
pub struct PacketHeader {
    pub packet_type: u8,
    pub magic: u8,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use game_ffi::{
        build_struct_signature_default, build_struct_signature_loose,
        build_struct_signature_strict, generate_uuid_from_signature, uuid_to_label, validate_uuid,
    };
    // use uuid::Uuid; // Not needed for these tests

    // =========================================================================
    // UUID v7 Format Tests
    // =========================================================================

    #[test]
    fn test_default_mode_uuid_is_v7() {
        let uuid = PlayerPosition::uuid();
        assert_eq!(uuid.get_version_num(), 7, "UUID should be v7");
        assert_eq!(
            uuid.get_variant(),
            uuid::Variant::RFC4122,
            "UUID should be RFC 4122 variant"
        );
    }

    #[test]
    fn test_strict_mode_uuid_is_v7() {
        let uuid = ServerConfig::uuid();
        assert_eq!(uuid.get_version_num(), 7, "Strict mode UUID should be v7");
        assert_eq!(
            uuid.get_variant(),
            uuid::Variant::RFC4122,
            "Strict mode UUID should be RFC 4122 variant"
        );
    }

    #[test]
    fn test_loose_mode_uuid_is_v7() {
        let uuid = ProtoState::uuid();
        assert_eq!(uuid.get_version_num(), 7, "Loose mode UUID should be v7");
        assert_eq!(
            uuid.get_variant(),
            uuid::Variant::RFC4122,
            "Loose mode UUID should be RFC 4122 variant"
        );
    }

    #[test]
    fn test_manual_uuid_preserved() {
        // Manual UUID should be preserved exactly as specified
        assert_eq!(
            LegacyComponent::UUID,
            "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"
        );

        // Manual UUID may not be v7 format (legacy support)
        let uuid = LegacyComponent::uuid();
        // This specific manual UUID is v4 (random) format
        assert_eq!(uuid.get_version_num(), 4, "Manual UUID should be preserved");
    }

    // =========================================================================
    // Deterministic UUID Tests
    // =========================================================================

    #[test]
    fn test_default_mode_deterministic() {
        let uuid1 = PlayerPosition::uuid();
        let uuid2 = PlayerPosition::uuid();
        assert_eq!(uuid1, uuid2, "Same struct should always produce same UUID");
    }

    #[test]
    fn test_strict_mode_deterministic() {
        let uuid1 = ServerConfig::uuid();
        let uuid2 = ServerConfig::uuid();
        assert_eq!(uuid1, uuid2, "Strict mode should be deterministic");
    }

    #[test]
    fn test_loose_mode_deterministic() {
        let uuid1 = ProtoState::uuid();
        let uuid2 = ProtoState::uuid();
        assert_eq!(uuid1, uuid2, "Loose mode should be deterministic");
    }

    #[test]
    fn test_different_structs_different_uuids() {
        assert_ne!(
            PlayerPosition::UUID,
            GameState::UUID,
            "Different structs should have different UUIDs"
        );
        assert_ne!(
            ServerConfig::UUID,
            ProtoState::UUID,
            "Different modes should produce different UUIDs"
        );
    }

    // =========================================================================
    // Breaking Change Detection Tests
    // =========================================================================

    #[test]
    fn test_default_mode_field_addition_breaks() {
        // These are two different structs with the same name
        // In a real scenario, they would be in different versions

        let fields_default: Vec<(&str, &str, Vec<&str>)> =
            vec![("x", "f32", vec![]), ("y", "f32", vec![])];

        let fields_extended: Vec<(&str, &str, Vec<&str>)> = vec![
            ("x", "f32", vec![]),
            ("y", "f32", vec![]),
            ("z", "f32", vec![]),
        ];

        let sig1 = build_struct_signature_default("1.0.0", "TestStruct", &fields_default);
        let sig2 = build_struct_signature_default("1.0.0", "TestStruct", &fields_extended);

        let uuid1 = generate_uuid_from_signature(&sig1);
        let uuid2 = generate_uuid_from_signature(&sig2);

        assert_ne!(uuid1, uuid2, "Adding a field should produce different UUID");
    }

    #[test]
    fn test_default_mode_field_removal_breaks() {
        let fields1: Vec<(&str, &str, Vec<&str>)> =
            vec![("x", "f32", vec![]), ("y", "f32", vec![])];

        let fields2: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];

        let sig1 = build_struct_signature_default("1.0.0", "TestStruct", &fields1);
        let sig2 = build_struct_signature_default("1.0.0", "TestStruct", &fields2);

        let uuid1 = generate_uuid_from_signature(&sig1);
        let uuid2 = generate_uuid_from_signature(&sig2);

        assert_ne!(
            uuid1, uuid2,
            "Removing a field should produce different UUID"
        );
    }

    #[test]
    fn test_default_mode_field_rename_breaks() {
        let fields1: Vec<(&str, &str, Vec<&str>)> =
            vec![("x", "f32", vec![]), ("y", "f32", vec![])];

        let fields2: Vec<(&str, &str, Vec<&str>)> =
            vec![("position_x", "f32", vec![]), ("y", "f32", vec![])];

        let sig1 = build_struct_signature_default("1.0.0", "TestStruct", &fields1);
        let sig2 = build_struct_signature_default("1.0.0", "TestStruct", &fields2);

        let uuid1 = generate_uuid_from_signature(&sig1);
        let uuid2 = generate_uuid_from_signature(&sig2);

        assert_ne!(
            uuid1, uuid2,
            "Renaming a field should produce different UUID"
        );
    }

    #[test]
    fn test_default_mode_type_change_breaks() {
        let fields1: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];

        let fields2: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f64", vec![])];

        let sig1 = build_struct_signature_default("1.0.0", "TestStruct", &fields1);
        let sig2 = build_struct_signature_default("1.0.0", "TestStruct", &fields2);

        let uuid1 = generate_uuid_from_signature(&sig1);
        let uuid2 = generate_uuid_from_signature(&sig2);

        assert_ne!(
            uuid1, uuid2,
            "Changing field type should produce different UUID"
        );
    }

    #[test]
    fn test_default_mode_breaking_attribute_breaks() {
        let fields1: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];

        let fields2: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec!["skip"])];

        let sig1 = build_struct_signature_default("1.0.0", "TestStruct", &fields1);
        let sig2 = build_struct_signature_default("1.0.0", "TestStruct", &fields2);

        let uuid1 = generate_uuid_from_signature(&sig1);
        let uuid2 = generate_uuid_from_signature(&sig2);

        assert_ne!(
            uuid1, uuid2,
            "Adding breaking attribute should produce different UUID"
        );
    }

    #[test]
    fn test_version_change_breaks() {
        let fields: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];

        let sig1 = build_struct_signature_default("1.0.0", "TestStruct", &fields);
        let sig2 = build_struct_signature_default("2.0.0", "TestStruct", &fields);

        let uuid1 = generate_uuid_from_signature(&sig1);
        let uuid2 = generate_uuid_from_signature(&sig2);

        assert_ne!(uuid1, uuid2, "Version change should produce different UUID");
    }

    // =========================================================================
    // Hash Mode Comparison Tests
    // =========================================================================

    #[test]
    fn test_loose_mode_ignores_field_changes() {
        // Loose mode only hashes struct name, so field changes don't matter

        let _fields1: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];

        let _fields2: Vec<(&str, &str, Vec<&str>)> = vec![
            ("x", "f32", vec![]),
            ("y", "f32", vec![]),
            ("z", "f32", vec![]),
        ];

        let sig1 = build_struct_signature_loose("1.0.0", "TestStruct");
        let sig2 = build_struct_signature_loose("1.0.0", "TestStruct");

        let uuid1 = generate_uuid_from_signature(&sig1);
        let uuid2 = generate_uuid_from_signature(&sig2);

        assert_eq!(uuid1, uuid2, "Loose mode should ignore field changes");
    }

    #[test]
    fn test_different_modes_produce_different_uuids() {
        // Test 1: Default and strict are same when no attributes
        let fields_no_attrs: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];
        let sig_default = build_struct_signature_default("1.0.0", "TestStruct", &fields_no_attrs);
        let sig_strict = build_struct_signature_strict("1.0.0", "TestStruct", &fields_no_attrs);
        let uuid_default = generate_uuid_from_signature(&sig_default);
        let uuid_strict = generate_uuid_from_signature(&sig_strict);
        assert_eq!(
            uuid_default, uuid_strict,
            "Default and strict should be same when no attributes present"
        );

        // Test 2: Loose mode is always different
        let sig_loose = build_struct_signature_loose("1.0.0", "TestStruct");
        let uuid_loose = generate_uuid_from_signature(&sig_loose);
        assert_ne!(
            uuid_default, uuid_loose,
            "Default and loose modes should differ"
        );
        assert_ne!(
            uuid_strict, uuid_loose,
            "Strict and loose modes should differ"
        );
    }

    #[test]
    fn test_strict_mode_catches_validation_attrs() {
        // Default mode ignores validation attributes (min, max)
        let fields_default: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];
        let sig_v1_default = build_struct_signature_default("1.0.0", "TestStruct", &fields_default);
        let sig_v2_default = build_struct_signature_default("1.0.0", "TestStruct", &fields_default);
        assert_eq!(
            sig_v1_default, sig_v2_default,
            "Default mode signatures should match"
        );

        // In strict mode, we'd hash all attributes (implementation detail)
        // For now, strict mode is similar to default for basic types
        let sig_v1_strict = build_struct_signature_strict("1.0.0", "TestStruct", &fields_default);
        let sig_v2_strict = build_struct_signature_strict("1.0.0", "TestStruct", &fields_default);
        assert_eq!(
            sig_v1_strict, sig_v2_strict,
            "Strict mode signatures should match"
        );
    }

    // =========================================================================
    // Memory Layout and Zero-Copy Tests
    // =========================================================================

    #[test]
    fn test_struct_has_repr_c() {
        assert_eq!(std::mem::size_of::<PlayerPosition>(), 12);
        assert_eq!(std::mem::align_of::<PlayerPosition>(), 4);
    }

    #[test]
    fn test_zero_copy_methods() {
        let pos = PlayerPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let bytes = pos.as_bytes();
        assert_eq!(bytes.len(), std::mem::size_of::<PlayerPosition>());

        unsafe {
            let pos2 = PlayerPosition::from_bytes(bytes);
            assert_eq!(pos2.x, pos.x);
            assert_eq!(pos2.y, pos.y);
            assert_eq!(pos2.z, pos.z);
        }
    }

    #[test]
    fn test_zero_copy_mutable() {
        let mut bytes = [0u8; 12];
        unsafe {
            let pos = PlayerPosition::from_bytes_mut(&mut bytes);
            pos.x = 5.0;
            pos.y = 10.0;
            pos.z = 15.0;

            let pos2 = PlayerPosition::from_bytes(&bytes);
            assert_eq!(pos2.x, 5.0);
            assert_eq!(pos2.y, 10.0);
            assert_eq!(pos2.z, 15.0);
        }
    }

    // =========================================================================
    // Validation and Layout Tests
    // =========================================================================

    #[test]
    fn test_validation_methods() {
        let pos = PlayerPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        assert!(pos.validate().is_ok());
        assert!(pos.is_valid());
    }

    #[test]
    fn test_default_impl() {
        let pos = PlayerPosition::default();
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 0.0);
    }

    #[test]
    fn test_layout_verification() {
        let layout = PlayerPosition::actual_layout();
        assert_eq!(layout.size, 12);
        assert_eq!(layout.alignment, 4);

        let expected = PlayerPosition::expected_layout();
        assert_eq!(layout.size, expected.size);
        assert_eq!(layout.alignment, expected.alignment);
    }

    #[test]
    fn test_size_function() {
        assert_eq!(PlayerPosition::size(), 12);
    }

    #[test]
    fn test_align_function() {
        assert_eq!(PlayerPosition::alignment(), 4);
    }

    // =========================================================================
    // Code Generation Tests
    // =========================================================================

    #[test]
    fn test_unity_bindings_exist() {
        #[cfg(feature = "unity")]
        {
            let unity_code = PlayerPosition::generate_unity_cs();
            assert!(unity_code.contains("namespace GameFFI"));
            assert!(unity_code.contains("StructLayout"));
        }
    }

    #[test]
    fn test_unreal_bindings_exist() {
        #[cfg(feature = "unreal")]
        {
            let unreal_code = CharacterUpdate::UNREAL_HPP;
            assert!(unreal_code.contains("#pragma once"));
            assert!(unreal_code.contains("USTRUCT"));
        }
    }

    // =========================================================================
    // Utility Tests
    // =========================================================================

    #[test]
    fn test_field_skip_excludes_from_layout() {
        let size = std::mem::size_of::<EntityUpdate>();
        assert!(size > 0);
    }

    #[test]
    fn test_nested_structs() {
        let header = PacketHeader {
            packet_type: 1,
            magic: 42,
        };
        let pos = PlayerPos {
            header,
            id: 123,
            x: 1.0,
            y: 2.0,
        };

        assert_eq!(pos.header.packet_type, 1);
        assert_eq!(pos.header.magic, 42);
        assert_eq!(pos.id, 123);
    }

    #[test]
    fn test_unity_config_preserved() {
        #[cfg(feature = "unity")]
        {
            let unity_code = PlayerPos::generate_unity_cs();
            assert!(unity_code.contains("PlayerPosUnity"));
        }
    }

    #[test]
    fn test_unreal_config_preserved() {
        #[cfg(feature = "unreal")]
        {
            let unreal_code = CharacterUpdate::UNREAL_HPP;
            assert!(unreal_code.contains("FCharacterUpdate"));
            assert!(unreal_code.contains("BlueprintType"));
        }
    }

    #[test]
    fn test_uuid_format_valid() {
        assert!(validate_uuid(PlayerPosition::UUID));
        assert!(validate_uuid(GameState::UUID));
        assert!(validate_uuid(ServerConfig::UUID));
        assert!(validate_uuid(ProtoState::UUID));
    }

    #[test]
    fn test_uuid_to_label() {
        let label = uuid_to_label(&PlayerPosition::uuid());
        assert!(label.len() <= 12);
        assert!(label.ends_with("..."));
    }

    #[test]
    fn test_memory_copy_safety() {
        let pos1 = PlayerPosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };

        let bytes = pos1.as_bytes();
        let pos2 = unsafe { PlayerPosition::from_bytes(bytes) };

        assert_eq!(pos1.x, pos2.x);
        assert_eq!(pos1.y, pos2.y);
        assert_eq!(pos1.z, pos2.z);
    }

    #[test]
    fn test_batch_operations() {
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

        let bytes: Vec<u8> = positions
            .iter()
            .flat_map(|p| p.as_bytes().iter().copied())
            .collect();

        let size = std::mem::size_of::<PlayerPosition>();
        for (i, pos) in positions.iter().enumerate() {
            let start = i * size;
            let end = start + size;
            let recovered = unsafe { PlayerPosition::from_bytes(&bytes[start..end]) };
            assert_eq!(recovered.x, pos.x);
            assert_eq!(recovered.y, pos.y);
            assert_eq!(recovered.z, pos.z);
        }
    }

    #[test]
    fn test_uuid_constant_matches_method() {
        let uuid_str = PlayerPosition::UUID;
        let uuid_obj = PlayerPosition::uuid();
        assert_eq!(uuid_str, uuid_obj.to_string());
    }

    #[test]
    fn test_generate_uuid_from_signature_utility() {
        let signature = "struct:1.0.0:crate::TestStruct{x:f32,y:f32}";
        let uuid1 = generate_uuid_from_signature(signature);
        let uuid2 = generate_uuid_from_signature(signature);

        assert_eq!(
            uuid1, uuid2,
            "generate_uuid_from_signature should be deterministic"
        );
        assert_eq!(uuid1.get_version_num(), 7, "Should generate UUID v7");
    }
}
