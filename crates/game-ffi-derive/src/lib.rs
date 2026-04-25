//! # Game FFI Derive - Procedural Macros
//!
//! This crate provides the `#[derive(GameComponent)]` procedural macro for FFI types.
//!
//! Supported attributes:
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
        game_ffi,
        field,
        db_table,
        db_column,
        db_default,
        db_index,
        db_foreign_key,
        db_unique_constraint,
        primary_key
    )
)]
pub fn game_component_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive::game_component_macro(input)
}
