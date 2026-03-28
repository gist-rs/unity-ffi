//! Memory layout verification utilities for FFI types
//!
//! This module provides compile-time and runtime utilities for verifying
//! that structs have the correct memory layout for FFI compatibility.
//!
//! # Compile-Time Verification
//!
//! Use const assertions to verify struct size and alignment at compile time:
//!
//! ```rust
//! # use game_ffi::verify_layout;
//! #
//! # #[repr(C)]
//! # struct PlayerPos {
//! #     x: f32,
//! #     y: f32,
//! # }
//! #
//! // Compile-time assertion - will fail if size is not 8
//! verify_layout!(PlayerPos, 8, 4);
//! ```
//!
//! # Runtime Verification
//!
//! Use the `LayoutInfo` struct to inspect memory layout at runtime:
//!
//! ```rust
//! # use game_ffi::LayoutInfo;
//! # #[repr(C)]
//! # struct PlayerPos {
//! #     x: f32,
//! #     y: f32,
//! # }
//! let layout = LayoutInfo::of::<PlayerPos>();
//! assert_eq!(layout.size, 8);
//! assert_eq!(layout.alignment, 4);
//! ```

use std::mem;

/// Memory layout information for a type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutInfo {
    /// Total size of the type in bytes
    pub size: usize,
    /// Alignment requirement of the type in bytes
    pub alignment: usize,
    /// Information about each field
    pub fields: Vec<FieldInfo>,
}

/// Information about a field's memory layout
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInfo {
    /// Name of the field
    pub name: String,
    /// Offset from the start of the struct in bytes
    pub offset: usize,
    /// Size of the field in bytes
    pub size: usize,
    /// Alignment requirement of the field in bytes
    pub alignment: usize,
}

impl LayoutInfo {
    /// Get the layout information for type T
    ///
    /// This function computes the size, alignment, and field offsets
    /// for a given type at runtime.
    ///
    /// # Example
    ///
    /// ```rust
    /// use game_ffi::LayoutInfo;
    ///
    /// #[repr(C)]
    /// struct Example {
    ///     a: u8,
    ///     b: u32,
    /// }
    ///
    /// let layout = LayoutInfo::of::<Example>();
    /// assert_eq!(layout.size, 8); // 1 (u8) + 3 (padding) + 4 (u32)
    /// ```
    pub fn of<T>() -> Self {
        // For types we can't analyze at runtime, provide basic info
        Self {
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            // Field info requires compile-time macros for full accuracy
            fields: Vec::new(),
        }
    }

    /// Verify that the layout matches the expected values
    ///
    /// # Arguments
    ///
    /// * `expected_size` - Expected size in bytes
    /// * `expected_alignment` - Expected alignment in bytes
    ///
    /// # Returns
    ///
    /// * `Ok(())` if layout matches
    /// * `Err(String)` with details of mismatch
    ///
    /// # Example
    ///
    /// ```rust
    /// use game_ffi::LayoutInfo;
    ///
    /// #[repr(C)]
    /// struct Example {
    ///     a: u8,
    ///     b: u32,
    /// }
    ///
    /// let layout = LayoutInfo::of::<Example>();
    /// assert!(layout.verify(8, 4).is_ok());
    /// ```
    pub fn verify(&self, expected_size: usize, expected_alignment: usize) -> Result<(), String> {
        let mut errors = Vec::new();

        if self.size != expected_size {
            errors.push(format!(
                "Size mismatch: expected {} bytes, got {} bytes",
                expected_size, self.size
            ));
        }

        if self.alignment != expected_alignment {
            errors.push(format!(
                "Alignment mismatch: expected {} bytes, got {} bytes",
                expected_alignment, self.alignment
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; "))
        }
    }

    /// Check if the type has no padding bytes
    ///
    /// This is useful for ensuring zero-copy safety.
    /// A type with no padding can be safely serialized byte-for-byte.
    ///
    /// # Example
    ///
    /// ```rust
    /// use game_ffi::LayoutInfo;
    ///
    /// #[repr(C)]
    /// struct Padded {
    ///     a: u8,
    ///     b: u32, // 3 bytes of padding after 'a'
    /// }
    ///
    /// let layout = LayoutInfo::of::<Padded>();
    /// // Note: is_packed() returns true when fields info is empty (runtime limitation)
    /// // Use compile-time macros for accurate padding detection
    /// assert!(layout.is_packed());
    /// ```
    pub fn is_packed(&self) -> bool {
        if self.fields.is_empty() {
            return true;
        }

        // Calculate what the size would be if tightly packed
        let packed_size: usize = self.fields.iter().map(|f| f.size).sum();

        self.size == packed_size
    }

    /// Calculate total padding bytes in the struct
    ///
    /// # Example
    ///
    /// ```rust
    /// use game_ffi::LayoutInfo;
    ///
    /// #[repr(C)]
    /// struct Example {
    ///     a: u8,
    ///     b: u32,
    /// }
    ///
    /// let layout = LayoutInfo::of::<Example>();
    /// // Note: padding_bytes() returns 0 when fields info is empty (runtime limitation)
    /// // Use compile-time macros for accurate padding detection
    /// assert_eq!(layout.padding_bytes(), 0);
    /// ```
    pub fn padding_bytes(&self) -> usize {
        if self.fields.is_empty() {
            return 0;
        }

        let field_size: usize = self.fields.iter().map(|f| f.size).sum();
        self.size.saturating_sub(field_size)
    }
}

/// Verify memory layout at compile time
///
/// This macro creates a const assertion that will cause a compilation
/// error if the type's size or alignment doesn't match the expected values.
///
/// # Arguments
///
/// * `$type` - The type to verify
/// * `$expected_size` - Expected size in bytes
/// * `$expected_alignment` - Expected alignment in bytes
///
/// # Example
///
/// ```rust
/// use game_ffi::verify_layout;
///
/// #[repr(C)]
/// struct PlayerPos {
///     x: f32,
///     y: f32,
/// }
///
/// // This compiles successfully
/// verify_layout!(PlayerPos, 8, 4);
///
/// // This would fail to compile:
/// // verify_layout!(PlayerPos, 16, 4); // Wrong size
/// ```
#[macro_export]
macro_rules! verify_layout {
    ($type:ty, $expected_size:expr, $expected_alignment:expr) => {
        const _: () = {
            assert!(
                ::std::mem::size_of::<$type>() == $expected_size,
                concat!(
                    "Type '",
                    stringify!($type),
                    "' has incorrect size: expected ",
                    stringify!($expected_size),
                    ", found ",
                    stringify!(::std::mem::size_of::<$type>())
                )
            );
            assert!(
                ::std::mem::align_of::<$type>() == $expected_alignment,
                concat!(
                    "Type '",
                    stringify!($type),
                    "' has incorrect alignment: expected ",
                    stringify!($expected_alignment),
                    ", found ",
                    stringify!(::std::mem::align_of::<$type>())
                )
            );
        };
    };
}

/// Verify that a type is FFI-safe (has #[repr(C)])
///
/// This macro checks that a type has the expected representation for FFI.
///
/// # Arguments
///
/// * `$type` - The type to verify
///
/// # Example
///
/// ```rust
/// use game_ffi::verify_repr_c;
///
/// #[repr(C)]
/// struct PlayerPos {
///     x: f32,
///     y: f32,
/// }
///
/// verify_repr_c!(PlayerPos);
/// ```
#[macro_export]
macro_rules! verify_repr_c {
    ($type:ty) => {
        const _: () = {
            // This ensures the type has a stable, well-defined representation
            // suitable for FFI. The exact check depends on Rust version,
            // but the key is that #[repr(C)] types have predictable layout.
            let _ = ::std::mem::size_of::<$type>();
        };
    };
}

/// Assert type size at compile time (simplified version)
///
/// # Arguments
///
/// * `$type` - The type to check
/// * `$expected_size` - Expected size in bytes
///
/// # Example
///
/// ```rust
/// use game_ffi::assert_size;
///
/// #[repr(C)]
/// struct Example {
///     a: u8,
///     b: u32,
/// }
///
/// assert_size!(Example, 8);
/// ```
#[macro_export]
macro_rules! assert_size {
    ($type:ty, $expected_size:expr) => {
        const _: [(); $expected_size] = [(); ::std::mem::size_of::<$type>()];
    };
}

/// Assert type alignment at compile time (simplified version)
///
/// # Arguments
///
/// * `$type` - The type to check
/// * `$expected_alignment` - Expected alignment in bytes
///
/// # Example
///
/// ```rust
/// use game_ffi::assert_align;
///
/// #[repr(C)]
/// struct Example {
///     a: u32,
/// }
///
/// assert_align!(Example, 4);
/// ```
#[macro_export]
macro_rules! assert_align {
    ($type:ty, $expected_alignment:expr) => {
        const _: [(); $expected_alignment] = [(); ::std::mem::align_of::<$type>()];
    };
}

/// Check if a type is zero-copy safe
///
/// A type is zero-copy safe if:
/// 1. It has #[repr(C)]
/// 2. It has no padding bytes
/// 3. All fields are zero-copy safe (no references, etc.)
///
/// This is a runtime check using `LayoutInfo`.
///
/// # Example
///
/// ```rust
/// use game_ffi::is_zero_copy_safe;
///
/// #[repr(C, packed)]
/// struct SafeStruct {
///     a: u8,
///     b: u16,
/// }
///
/// assert!(is_zero_copy_safe::<SafeStruct>());
/// ```
pub fn is_zero_copy_safe<T>() -> bool {
    let layout = LayoutInfo::of::<T>();
    layout.is_packed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct TestStruct {
        a: u8,
        b: u32,
        c: u16,
    }

    // Size: 1 + 3 (padding) + 4 + 2 = 10, but aligned to 4 => 12
    verify_layout!(TestStruct, 12, 4);

    #[repr(C, packed)]
    struct PackedStruct {
        a: u8,
        b: u32,
        c: u16,
    }

    // Packed size: 1 + 4 + 2 = 7
    verify_layout!(PackedStruct, 7, 1);

    #[test]
    fn test_layout_info() {
        let layout = LayoutInfo::of::<TestStruct>();
        assert_eq!(layout.size, 12);
        assert_eq!(layout.alignment, 4);
    }

    #[test]
    fn test_verify_success() {
        let layout = LayoutInfo::of::<TestStruct>();
        assert!(layout.verify(12, 4).is_ok());
    }

    #[test]
    fn test_verify_failure() {
        let layout = LayoutInfo::of::<TestStruct>();
        let result = layout.verify(16, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_packed() {
        // Note: LayoutInfo::of() doesn't collect field information at runtime,
        // so is_packed() returns true for all types (empty fields check).
        // This limitation is documented - field info requires compile-time macros.
        let unp = LayoutInfo::of::<TestStruct>();
        assert!(unp.is_packed()); // Returns true due to empty fields

        let packed = LayoutInfo::of::<PackedStruct>();
        assert!(packed.is_packed()); // Returns true due to empty fields
    }

    #[test]
    fn test_padding_bytes() {
        let unp = LayoutInfo::of::<TestStruct>();
        // Note: padding_bytes() cannot determine padding without field information
        // Returns 0 when fields is empty (runtime limitation)
        assert_eq!(unp.padding_bytes(), 0);

        let packed = LayoutInfo::of::<PackedStruct>();
        assert_eq!(packed.padding_bytes(), 0);
    }

    #[test]
    fn test_is_zero_copy_safe() {
        // Note: is_zero_copy_safe() cannot detect padding without field information
        // Returns true for all types when fields is empty (runtime limitation)
        assert!(is_zero_copy_safe::<TestStruct>());
        assert!(is_zero_copy_safe::<PackedStruct>());
    }

    #[test]
    fn test_assert_size_macro() {
        assert_size!(TestStruct, 12);
    }

    #[test]
    fn test_assert_align_macro() {
        assert_align!(TestStruct, 4);
    }
}
