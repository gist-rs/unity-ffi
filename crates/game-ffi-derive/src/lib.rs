//! # Game FFI Derive - Procedural Macros
//!
//! This crate provides the `#[derive(GameComponent)]` procedural macro for FFI types,
//! plus `#[unity]` and `#[unreal]` attribute macros that auto-inject `#[repr(C)]`
//! and `#[derive(GameComponent)]`.
//!
//! ## Attribute Macros (preferred entry point)
//!
//! - `#[unity(name = "...", read_only)]` — Auto-injects `#[repr(C)]`, `#[derive(GameComponent)]`
//! - `#[unreal(class = "...", blueprint_type)]` — Same pattern for Unreal Engine
//!
//! ## Derive Helper Attributes (backward compat)
//!
//! - `uuid`, `hash` - Type identification
//! - `unity`, `unreal` - Engine-specific bindings
//! - `game_ffi`, `field` - FFI configuration
//! - `db_table` - Database table name (Plan 082)
//! - `db_column` - SQL column type and constraints (Plan 082)
//! - `db_default` - Database default value (Plan 082)
//! - `db_index` - Database index definition (Plan 082)
//! - `db_foreign_key` - Foreign key constraint (Plan 082)
//! - `db_unique_constraint` - Unique constraint (Plan 082)
//! - `primary_key` - Primary key field marker (Plan 082)
//! - `db_flatten` - Flatten embedded struct columns into parent table (Plan 001)

// Internal module for macro implementation
mod derive;

// Export the main derive macro
#[proc_macro_derive(
    GameComponent,
    attributes(
        uuid,
        hash,
        unity,
        unreal,
        __game_ffi_unity,
        __game_ffi_unreal,
        game_ffi,
        field,
        db_table,
        db_column,
        db_default,
        db_index,
        db_foreign_key,
        db_unique_constraint,
        primary_key,
        db_flatten
    )
)]
pub fn game_component_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive::game_component_macro(input)
}

/// Attribute macro for Unity FFI types.
///
/// Auto-injects `#[repr(C)]`, `#[derive(GameComponent)]`, and an internal
/// `#[__game_ffi_unity(...)]` helper attribute that the derive macro parses.
///
/// # Example
///
/// ```rust,ignore
/// #[unity(name = "PlayerPosUnity")]
/// pub struct PlayerPos {
///     pub x: f32,
///     pub y: f32,
/// }
/// ```
#[proc_macro_attribute]
pub fn unity(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    derive::unity_attribute(attr, item)
}

/// Attribute macro for Unreal FFI types.
///
/// Auto-injects `#[repr(C)]`, `#[derive(GameComponent)]`, and an internal
/// `#[__game_ffi_unreal(...)]` helper attribute that the derive macro parses.
///
/// # Example
///
/// ```rust,ignore
/// #[unreal(class = "FCharacterUpdate", blueprint_type)]
/// pub struct CharacterUpdate {
///     pub char_id: u64,
///     pub x: u16,
///     pub y: u16,
/// }
/// ```
#[proc_macro_attribute]
pub fn unreal(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    derive::unreal_attribute(attr, item)
}
