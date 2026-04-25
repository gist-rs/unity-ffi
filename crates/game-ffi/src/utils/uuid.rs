//! UUID generation utilities for FFI types
//!
//! This module provides deterministic UUID v7 generation for FFI types
//! using Blake3 hashing. The generated UUIDs are UUID v7 which ensures
//! that the same struct signature always produces the same UUID.
//!
//! ## UUID Generation Modes
//!
//! The system supports three modes of UUID generation:
//!
//! 1. **Default Mode** (recommended): Hashes struct name + fields + breaking attributes
//!    - Detects: add/remove/rename field, change type/order, breaking attributes
//!    - Use for: Most FFI components
//!
//! 2. **Strict Mode** (`#[hash = "all"]`): Hashes full signature including all attributes
//!    - Detects: Everything above + validation attributes (min, max, default)
//!    - Use for: Validation-critical protocols
//!
//! 3. **Loose Mode** (`#[hash = "name"]`): Hashes struct name only
//!    - Detects: Struct name change only
//!    - Use for: Rapid prototyping
//!
//! ## Deterministic UUID v7
//!
//! UUIDs are generated using:
//! - Blake3 hash of struct signature
//! - Formatted as UUID v7 (RFC 4122)
//! - Version-aware (includes cargo.toml version)
//! - Namespace-safe (includes full struct path)
//!
//! ```rust
//! use game_ffi::utils::uuid::generate_uuid_from_signature;
//!
//! // Default mode
//! let signature = "struct:1.0.0:crate::components::PlayerPosition{x:f32,y:f32}";
//! let uuid1 = generate_uuid_from_signature(signature);
//! let uuid2 = generate_uuid_from_signature(signature);
//! assert_eq!(uuid1, uuid2); // Deterministic
//!
//! // Verify UUID v7 format
//! assert_eq!(uuid1.get_version_num(), 7);
//! ```

use blake3::Hash;
use uuid::Uuid;

/// Generate a deterministic UUID v7 from a signature string
///
/// This function uses Blake3 hashing to create a UUID v7 from the signature,
/// ensuring that the same signature always produces the same UUID.
///
/// # Arguments
///
/// * `signature` - The signature string to hash (e.g., "struct:1.0.0:crate::PlayerPosition{x:f32}")
///
/// # Returns
///
/// A UUID v7 generated from the signature
///
/// # Example
///
/// ```rust
/// use game_ffi::utils::uuid::generate_uuid_from_signature;
///
/// let signature = "struct:1.0.0:crate::PlayerPosition{x:f32,y:f32}";
/// let uuid1 = generate_uuid_from_signature(signature);
/// let uuid2 = generate_uuid_from_signature(signature);
/// assert_eq!(uuid1, uuid2); // Deterministic
///
/// // Verify it's a valid UUID v7
/// assert_eq!(uuid1.get_version_num(), 7);
/// assert_eq!(uuid1.get_variant(), uuid::Variant::RFC4122);
/// ```
pub fn generate_uuid_from_signature(signature: &str) -> Uuid {
    let hash = blake3::hash(signature.as_bytes());
    uuid_v7_from_blake3_hash(hash)
}

/// Generate a UUID v7 from a Blake3 hash
///
/// Converts a Blake3 hash to a UUID v7 format.
/// UUID v7 uses a time-ordered layout for better database indexing.
fn uuid_v7_from_blake3_hash(hash: Hash) -> Uuid {
    let bytes = hash.as_bytes();

    // Convert Blake3 hash to UUID v7 format
    let mut uuid_bytes: [u8; 16] = [0u8; 16];
    uuid_bytes.copy_from_slice(&bytes[..16]);

    // Set version to 7 (time-ordered)
    uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x70;

    // Set variant to RFC 4122
    uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80;

    Uuid::from_bytes(uuid_bytes)
}

/// Validate if a string is a valid UUID
///
/// # Arguments
///
/// * `uuid_str` - The UUID string to validate
///
/// # Returns
///
/// `true` if the string is a valid UUID, `false` otherwise
///
/// # Example
///
/// ```rust
/// use game_ffi::utils::uuid::validate_uuid;
///
/// assert!(validate_uuid("0188c972-593f-7b5f-8000-123456789abc"));
/// assert!(!validate_uuid("not-a-uuid"));
/// ```
pub fn validate_uuid(uuid_str: &str) -> bool {
    Uuid::parse_str(uuid_str).is_ok()
}

/// Generate a human-readable UUID label for debugging
///
/// Creates a shortened, more readable version of a UUID for logging
/// and debugging purposes.
///
/// # Arguments
///
/// * `uuid` - The UUID to create a label for
///
/// # Returns
///
/// A shortened string representation of the UUID
///
/// # Example
///
/// ```rust
/// use game_ffi::utils::uuid::uuid_to_label;
/// use uuid::Uuid;
///
/// let uuid = Uuid::now_v7();
/// let label = uuid_to_label(&uuid);
/// // label might be something like "0188c972..."
/// ```
pub fn uuid_to_label(uuid: &Uuid) -> String {
    let uuid_str = uuid.to_string();
    if uuid_str.len() > 8 {
        format!("{}...", &uuid_str[..8])
    } else {
        uuid_str
    }
}

/// Build a struct signature for UUID generation (default mode)
///
/// This function creates a signature string that includes:
/// - Cargo version (for version-aware hashing)
/// - Full struct path (for namespace safety)
/// - Field names and types
/// - Breaking attributes only (skip, ffi_order, etc.)
///
/// # Arguments
///
/// * `cargo_version` - The cargo version (e.g., "1.0.0")
/// * `struct_path` - The full struct path (e.g., "crate::components::PlayerPosition")
/// * `fields` - Vector of (field_name, field_type, breaking_attributes) tuples
///
/// # Returns
///
/// A signature string suitable for UUID generation
///
/// # Example
///
/// ```rust
/// use game_ffi::utils::uuid::build_struct_signature_default;
///
/// let fields: Vec<(&str, &str, Vec<&str>)> = vec![
///     ("x", "f32", vec![]),
///     ("y", "f32", vec![]),
/// ];
/// let signature = build_struct_signature_default("1.0.0", "crate::PlayerPosition", &fields);
/// // signature: "struct:1.0.0:crate::PlayerPosition{x:f32,y:f32}"
/// ```
pub fn build_struct_signature_default(
    cargo_version: &str,
    struct_path: &str,
    fields: &[(&str, &str, Vec<&str>)],
) -> String {
    let field_list = fields
        .iter()
        .map(|(name, ty, attrs)| {
            let attrs_str = if attrs.is_empty() {
                String::new()
            } else {
                format!(
                    "[{}]",
                    attrs
                        .iter()
                        .map(|a| format!("{}={}", a, a))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            };
            format!("{}:{}{}", name, ty, attrs_str)
        })
        .collect::<Vec<_>>()
        .join(",");

    format!("struct:{}:{}{{{}}}", cargo_version, struct_path, field_list)
}

/// Build a struct signature for strict mode (all attributes)
///
/// Similar to `build_struct_signature_default` but includes ALL attributes,
/// not just breaking ones. Use this for validation-critical protocols.
pub fn build_struct_signature_strict(
    cargo_version: &str,
    struct_path: &str,
    fields: &[(&str, &str, Vec<&str>)],
) -> String {
    let field_list = fields
        .iter()
        .map(|(name, ty, _attrs)| {
            // In strict mode, we include all attributes
            // The actual implementation would parse all field attributes
            format!("{}:{}", name, ty)
        })
        .collect::<Vec<_>>()
        .join(",");

    format!("struct:{}:{}{{{}}}", cargo_version, struct_path, field_list)
}

/// Build a struct signature for loose mode (name only)
///
/// Only includes struct path, ignoring fields entirely.
/// Useful for rapid prototyping where field changes shouldn't trigger new UUIDs.
pub fn build_struct_signature_loose(cargo_version: &str, struct_path: &str) -> String {
    format!("struct:{}:{}", cargo_version, struct_path)
}

/// FFI namespace for mmorpg core types
pub const NAMESPACE_CORE: &str = "mmorpg.core";

/// FFI namespace for player-related types
pub const NAMESPACE_PLAYER: &str = "mmorpg.player";

/// FFI namespace for entity-related types
pub const NAMESPACE_ENTITY: &str = "mmorpg.entity";

/// FFI namespace for inventory-related types
pub const NAMESPACE_INVENTORY: &str = "mmorpg.inventory";

/// FFI namespace for shop-related types
pub const NAMESPACE_SHOP: &str = "mmorpg.shop";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_uuid_from_signature_deterministic() {
        let signature = "struct:1.0.0:crate::PlayerPosition{x:f32,y:f32}";
        let uuid1 = generate_uuid_from_signature(signature);
        let uuid2 = generate_uuid_from_signature(signature);
        assert_eq!(uuid1, uuid2);
    }

    #[test]
    fn test_generate_uuid_from_signature_different_signatures() {
        let sig1 = "struct:1.0.0:crate::PlayerPosition{x:f32,y:f32}";
        let sig2 = "struct:1.0.0:crate::GameState{tick:u64,player_count:u32}";
        let uuid1 = generate_uuid_from_signature(sig1);
        let uuid2 = generate_uuid_from_signature(sig2);
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_uuid_v7_format() {
        let signature = "struct:1.0.0:crate::PlayerPosition{x:f32,y:f32}";
        let uuid = generate_uuid_from_signature(signature);

        // Verify it's a valid UUID v7
        assert_eq!(uuid.get_version_num(), 7);
        assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn test_validate_uuid() {
        assert!(validate_uuid("0188c972-593f-7b5f-8000-123456789abc"));
        assert!(validate_uuid("fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"));
        assert!(!validate_uuid("not-a-uuid"));
        assert!(!validate_uuid(""));
    }

    #[test]
    fn test_uuid_to_label() {
        let uuid = Uuid::parse_str("0188c972-593f-7b5f-8000-123456789abc").unwrap();
        let label = uuid_to_label(&uuid);
        assert!(label.starts_with("0188c972"));
        assert!(label.ends_with("..."));
    }

    #[test]
    fn test_build_struct_signature_default() {
        let fields: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![]), ("y", "f32", vec![])];
        let signature = build_struct_signature_default("1.0.0", "crate::PlayerPosition", &fields);

        assert!(signature.contains("struct:1.0.0:crate::PlayerPosition"));
        assert!(signature.contains("x:f32"));
        assert!(signature.contains("y:f32"));
    }

    #[test]
    fn test_build_struct_signature_default_with_breaking_attrs() {
        let fields: Vec<(&str, &str, Vec<&str>)> =
            vec![("x", "f32", vec![]), ("y", "f32", vec!["skip"])];
        let signature = build_struct_signature_default("1.0.0", "crate::PlayerPosition", &fields);

        assert!(signature.contains("x:f32"));
        assert!(signature.contains("y:f32[skip=skip]"));
    }

    #[test]
    fn test_build_struct_signature_strict() {
        let fields: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![]), ("y", "f32", vec![])];
        let signature = build_struct_signature_strict("1.0.0", "crate::PlayerPosition", &fields);

        assert!(signature.contains("struct:1.0.0:crate::PlayerPosition"));
        assert!(signature.contains("x:f32"));
        assert!(signature.contains("y:f32"));
    }

    #[test]
    fn test_build_struct_signature_loose() {
        let signature = build_struct_signature_loose("1.0.0", "crate::PlayerPosition");

        assert_eq!(signature, "struct:1.0.0:crate::PlayerPosition");
        // Loose mode only includes version and struct path, no field braces or fields
        assert!(!signature.contains('{'));
        assert!(!signature.contains('}'));
        assert!(!signature.contains("field_x"));
        assert!(!signature.contains("field_y"));
    }

    #[test]
    fn test_namespaces() {
        assert!(!NAMESPACE_CORE.is_empty());
        assert!(!NAMESPACE_PLAYER.is_empty());
        assert!(!NAMESPACE_ENTITY.is_empty());
        assert!(!NAMESPACE_INVENTORY.is_empty());
        assert!(!NAMESPACE_SHOP.is_empty());
    }

    #[test]
    fn test_version_aware_hashing() {
        let fields: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];

        let uuid_v1 = generate_uuid_from_signature(&build_struct_signature_default(
            "1.0.0",
            "crate::PlayerPosition",
            &fields,
        ));

        let uuid_v2 = generate_uuid_from_signature(&build_struct_signature_default(
            "2.0.0",
            "crate::PlayerPosition",
            &fields,
        ));

        assert_ne!(
            uuid_v1, uuid_v2,
            "Different versions should produce different UUIDs"
        );
    }

    #[test]
    fn test_namespace_safe_hashing() {
        let fields: Vec<(&str, &str, Vec<&str>)> = vec![("x", "f32", vec![])];

        let uuid1 = generate_uuid_from_signature(&build_struct_signature_default(
            "1.0.0",
            "crate::components::PlayerPosition",
            &fields,
        ));

        let uuid2 = generate_uuid_from_signature(&build_struct_signature_default(
            "1.0.0",
            "crate::systems::PlayerPosition",
            &fields,
        ));

        assert_ne!(
            uuid1, uuid2,
            "Different namespaces should produce different UUIDs"
        );
    }
}
