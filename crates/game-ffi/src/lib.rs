//! # Game FFI - Unified Annotation System
//!
//! This crate provides a declarative macro-based system for FFI (Foreign Function Interface)
//! types used in game engine integration (Unity C#, Unreal C++, etc.).
//!
//! ## Features
//!
//! - **Declarative annotations**: Use `#[derive(GameComponent)]` to auto-generate FFI code
//! - **Field-level attributes**: Add validation, skip fields, engine-specific config
//! - **Auto UUID generation**: Deterministic UUID v7 from struct signatures
//! - **Three hash modes**: Default (field signature), Strict (all attributes), Loose (name only)
//! - **Memory layout verification**: Compile-time checks for struct size and alignment
//! - **Zero-copy patterns**: Direct memory access for performance
//! - **Code generation**: Automatic Unity C# and Unreal C++ bindings
//!
//! ## Quick Start
//!
//! ```rust
//! #![allow(dead_code)]
//! use game_ffi::GameComponent;
//!
//! #[repr(C)]
//! #[derive(GameComponent)]
//! pub struct PlayerPosition {
//!     pub x: f32,
//!     pub y: f32,
//!     pub z: f32,
//! }
//! ```
//!
//! ## UUID Generation Modes
//!
//! The system supports three simple modes for deterministic UUID v7 generation:
//!
//! ### Default Mode (Recommended)
//!
//! Hashes struct name + fields + breaking attributes. Detects:
//! - Add/remove/rename field
//! - Change field type or order
//! - Add breaking attributes (skip, ffi_order, etc.)
//!
//! ```rust
//! #![allow(dead_code)]
//! use game_ffi::GameComponent;
//! #[repr(C)]
//! #[derive(GameComponent)]
//! pub struct PlayerPosition {
//!     pub x: f32,
//!     pub y: f32,
//! }
//! // Signature: "struct:1.0.0:crate::PlayerPosition{x:f32,y:f32}"
//! ```
//!
//! ### Strict Mode (`#[hash = "all"]`)
//!
//! Hashes full signature including ALL attributes. Detects everything above plus:
//! - Validation attribute changes (min, max, default)
//!
//! ```rust
//! #![allow(dead_code)]
//! use game_ffi::GameComponent;
//! #[repr(C)]
//! #[derive(GameComponent)]
//! #[hash = "all"]
//! pub struct ServerConfig {
//!     #[field(min = 1000, max = 9999)]  // Changing these = new UUID
//!     pub port: u16,
//! }
//! ```
//!
//! ### Loose Mode (`#[hash = "name"]`)
//!
//! Hashes struct name only. Fields can change freely.
//!
//! ```rust
//! #![allow(dead_code)]
//! use game_ffi::GameComponent;
//! #[repr(C)]
//! #[derive(GameComponent)]
//! #[hash = "name"]
//! pub struct ProtoState {
//!     pub data: Vec<u8>,  // Can add/remove fields freely
//! }
//! ```
//!
//! ### Manual Mode (Legacy)
//!
//! Explicit UUID assignment for special cases.
//!
//! ```rust
//! #![allow(dead_code)]
//! use game_ffi::GameComponent;
//! #[repr(C)]
//! #[derive(GameComponent)]
//! #[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
//! pub struct LegacyComponent {
//!     pub x: f32,
//! }
//! ```
//!
//! ## Generated Code
//!
//! The `GameComponent` macro automatically generates:
//!
//! - **UUID constant**: `PlayerPosition::UUID` - Type identification string
//! - **UUID method**: `PlayerPosition::uuid()` - Returns `Uuid` object
//! - **Layout info**: `size()`, `alignment()`, `actual_layout()`
//! - **Zero-copy**: `as_bytes()`, `from_bytes()`, `from_bytes_mut()`
//! - **Validation**: `validate()`, `is_valid()`
//! - **FFI wrappers**: `set_player_position()`, etc.
//! - **Unity C#**: With `[StructLayout(LayoutKind.Sequential)]`
//! - **Unreal C++**: With USTRUCT macros
//!
//! ## Field Attributes
//!
//! - `#[field(skip)]` - Exclude from public API (breaking change)
//! - `#[field(min = X)]` - Minimum value validation
//! - `#[field(max = Y)]` - Maximum value validation
//! - `#[unity(name = "CustomName")]` - Custom field name in Unity
//! - `#[unreal(replicated)]` - Mark for network replication in Unreal
//!
//! ## Top-Level Attributes
//!
//! - `#[hash = "all"]` - Hash all attributes (strict mode)
//! - `#[hash = "name"]` - Hash struct name only (loose mode)
//! - `#[uuid = "..."]` - Manual UUID assignment (rare)
//! - `#[unity(name = "...")]` - Custom struct name in Unity
//! - `#[unreal(class = "...")]` - Custom class name in Unreal
//!
//! ## Memory Safety
//!
//! All types use `#[repr(C)]` and enforce compile-time size/alignment checks:
//!
//! ```rust
//! use game_ffi::GameComponent;
//!
//! #[repr(C)]
//! #[derive(GameComponent)]
//! struct PlayerPosition {
//!     pub x: f32,
//!     pub y: f32,
//!     pub z: f32,
//! }
//!
//! // Access layout information
//! let size = PlayerPosition::size(); // 12 bytes
//! let alignment = PlayerPosition::alignment(); // 4 bytes
//! ```
//!
//! ## Zero-Copy Operations
//!
//! Direct memory access without copying:
//!
//! See examples/basic_usage.rs for a working zero-copy example.
//!
//!
//! ## Deterministic UUID v7
//!
//! UUIDs are generated deterministically using Blake3 hashing:
//!
//! ```rust
//! use game_ffi::GameComponent;
//!
//! #[repr(C)]
//! #[derive(GameComponent)]
//! pub struct PlayerPosition {
//!     pub x: f32,
//!     pub y: f32,
//! }
//!
//! // UUID is deterministic
//! let uuid = PlayerPosition::uuid();
//! assert_eq!(PlayerPosition::UUID, uuid.to_string());
//!
//! // UUID is v7 format
//! assert_eq!(uuid.get_version_num(), 7);
//! assert_eq!(uuid.get_variant(), uuid::Variant::RFC4122);
//!
//! // Same struct = same UUID
//! let uuid2 = PlayerPosition::uuid();
//! assert_eq!(uuid, uuid2);
//! ```
//!
//! ## Breaking Change Detection
//!
//! Changes that trigger new UUIDs:
//!
//! ```rust
//! #![allow(dead_code)]
//! use game_ffi::GameComponent;
//! // Version 1.0.0
//! #[repr(C)]
//! #[derive(GameComponent)]
//! pub struct PlayerStateV1 {
//!     pub health: f32,
//!     pub mana: f32,
//! }
//! // UUID: abc123...
//!
//! // Version 1.1.0 - Breaking change detected
//! #[repr(C)]
//! #[derive(GameComponent)]
//! pub struct PlayerStateV2 {
//!     pub health: f32,
//!     pub mana: f32,
//!     pub stamina: f32,  // New field = new UUID
//! }
//! // UUID: def456...
//!
//! // Clients can detect mismatch:
//! # let client_uuid = "abc123";
//! # let server_uuid = "def456";
//! if client_uuid != server_uuid {
//!     println!("Protocol version mismatch!");
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `unity` (default): Generate Unity C# bindings
//! - `unreal` (default): Generate Unreal C++ bindings
//!
//! Disable unwanted features to reduce compile time:
//! ```toml
//! [dependencies.game-ffi]
//! version = "0.1"
//! default-features = false
//! features = ["unity"]
//! ```
//!
//! ## Decision Guide
//!
//! Need strict validation? → `#[hash = "all"]`
//!
//! Standard FFI protocol? → `#[derive(GameComponent)]` (default)
//!
//! Fast iteration/prototyping? → `#[hash = "name"]`

// Re-export the main derive macro
pub use game_ffi_derive::GameComponent;

// Re-export attribute macros that auto-inject #[repr(C)] + #[derive(GameComponent)]
pub use game_ffi_derive::{unity, unreal};

// Public utilities
pub mod utils;

// Re-export commonly used types from utils
pub use utils::{is_zero_copy_safe, LayoutInfo};

// Re-export UUID utilities
pub use utils::uuid::{
    build_struct_signature_default, build_struct_signature_loose, build_struct_signature_strict,
    generate_uuid_from_signature, uuid_to_label, validate_uuid, NAMESPACE_CORE, NAMESPACE_ENTITY,
    NAMESPACE_INVENTORY, NAMESPACE_PLAYER, NAMESPACE_SHOP,
};

// Error types
pub use thiserror::Error;

#[doc(hidden)]
pub use quote::{quote, ToTokens};
