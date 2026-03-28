//! Derive macro implementation for GameComponent
//!
//! This module contains the procedural macro implementation for the
//! `#[derive(GameComponent)]` attribute.

pub mod attributes;
pub mod unity;
pub mod unreal;

mod game_component;

// Re-export the main derive function
pub use game_component::game_component_macro;
