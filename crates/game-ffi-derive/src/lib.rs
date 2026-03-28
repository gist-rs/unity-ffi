//! # Game FFI Derive - Procedural Macros
//!
//! This crate provides the `#[derive(GameComponent)]` procedural macro for FFI types.

// Internal module for macro implementation
mod derive;

// Export the main derive macro
#[proc_macro_derive(GameComponent, attributes(uuid, hash, unity, unreal, game_ffi, field))]
pub fn game_component_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive::game_component_macro(input)
}
