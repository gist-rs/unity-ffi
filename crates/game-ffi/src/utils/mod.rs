//! Utility functions and macros for FFI types
//!
//! Utility functions and macros for FFI types
//!
//! This module provides memory layout verification utilities and helper functions
//! for working with FFI-compatible types.

pub mod uuid;
pub mod validation;

// Re-export commonly used items from uuid
pub use uuid::generate_uuid_from_signature;

// Re-export commonly used types and functions
pub use validation::{is_zero_copy_safe, FieldInfo, LayoutInfo};
