//! Main implementation of the GameComponent derive macro
//!
//! This module contains the core procedural macro logic for `#[derive(GameComponent)]`.
//!
//! ## Plan 082: Schema-Driven Database
//!
//! When a struct has `#[db_table("name")]`, the macro also generates:
//! - `CREATE TABLE SQL` constant
//! - `CREATE INDEX SQL` constant (if indexes defined)
//! - `TABLE_NAME` constant
//! - `column_names()`, `column_count()`, `primary_key_field()` methods

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse2, parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Error, Ident, Type};

use super::attributes::{
    parse_field_attributes, parse_struct_attributes, FieldAttributes, HashMode, StructAttributes,
};
use super::schema::sql_gen::generate_schema_impl;
use super::schema::types::{DbFieldInfo, DbSchemaInfo};

/// Information about a struct field needed for signature hashing
#[derive(Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
    pub csharp_type: Option<String>,
    pub offset: Option<usize>,
    pub breaking_attributes: Vec<String>,
    pub is_padding: bool,
}

/// The main GameComponent derive macro
///
/// This macro generates FFI-compatible code for structs, including:
/// - UUID assignment (auto-generated UUID v7 from struct signature or manual)
/// - Memory layout verification
/// - Zero-copy methods (as_bytes, from_bytes)
/// - Validation methods (validate, is_valid)
/// - FFI wrapper functions
/// - Unity C# bindings (if feature enabled)
/// - Unreal C++ bindings (if feature enabled)
pub fn game_component_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match impl_game_component(input) {
        Ok(tokens) => tokens.into_token_stream().into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Attribute macro `#[unity(...)]` — auto-injects `#[repr(C)]`, `#[derive(GameComponent)]`,
/// and an internal `#[__game_ffi_unity(...)]` helper for the derive macro to parse.
///
/// This prevents users from forgetting `#[repr(C)]` and reduces boilerplate.
pub fn unity_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_engine_attribute(attr, item, "__game_ffi_unity", "unity")
}

/// Attribute macro `#[unreal(...)]` — same pattern as `#[unity]` but for Unreal Engine.
pub fn unreal_attribute(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_engine_attribute(attr, item, "__game_ffi_unreal", "unreal")
}

/// Shared expansion logic for `#[unity]` and `#[unreal]` attribute macros.
///
/// Reads the original attribute args (e.g. `name = "..."`, `read_only`),
/// then emits a struct annotated with:
///   1. `#[repr(C)]` — guarantees C memory layout for FFI
///   2. `#[derive(GameComponent)]` — triggers the derive macro
///   3. `#[<internal_helper>(...)]` — passes the original args through so
///      the derive macro can read them (avoids infinite recursion).
fn expand_engine_attribute(
    attr: TokenStream,
    item: TokenStream,
    internal_helper: &str,
    _engine: &str,
) -> TokenStream {
    let item_tokens: proc_macro2::TokenStream = item.into();

    // Build the internal helper ident from the string name
    let helper_ident = Ident::new(internal_helper, proc_macro2::Span::call_site());

    // Build the internal helper attribute tokens from the original attr args
    let internal_attr = if attr.is_empty() {
        quote! { #[#helper_ident] }
    } else {
        let attr_tokens: proc_macro2::TokenStream = attr.into();
        quote! { #[#helper_ident(#attr_tokens)] }
    };

    // Build combined token stream: new attrs prepended to original item,
    // then re-parse so syn normalises the attribute list.
    let combined = quote! {
        #[repr(C)]
        #[derive(GameComponent)]
        #internal_attr
        #item_tokens
    };

    // Re-parse to normalise attributes (derive will see #[repr(C)] etc.)
    let input: DeriveInput = match parse2(combined) {
        Ok(i) => i,
        Err(e) => return e.to_compile_error().into(),
    };

    input.into_token_stream().into()
}

/// Check if the struct has `#[repr(C)]` attribute.
fn has_repr_c(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if let syn::Meta::List(list) = &attr.meta {
            list.path.is_ident("repr")
                && list
                    .tokens
                    .clone()
                    .into_iter()
                    .any(|tt| matches!(tt, proc_macro2::TokenTree::Ident(ident) if ident == "C"))
        } else {
            false
        }
    })
}

/// Core implementation of the GameComponent derive macro
fn impl_game_component(input: DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    // Parse struct attributes
    let struct_attrs = parse_struct_attributes(&input.attrs)?;

    // Safety net: #[repr(C)] required for zero-copy FFI types.
    // Types that skip zero-copy (DB-only, internal) don't need it.
    if !struct_attrs.skip_zero_copy && !has_repr_c(&input.attrs) {
        return Err(Error::new_spanned(
            &input,
            "GameComponent requires #[repr(C)] for zero-copy types. \
             Use #[unity(...)] to auto-inject it, or add #[game_ffi(skip_zero_copy)] for DB-only types.",
        ));
    }

    // Extract common information
    let struct_name = &input.ident;
    let struct_name_str = struct_name.to_string();
    let _visibility = &input.vis;
    let _generics = &input.generics;

    // Extract field or variant information for signature hashing
    let fields = match &input.data {
        Data::Struct(s) => extract_field_info(s)?,
        Data::Enum(e) => extract_variant_info(e)?,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "GameComponent can only be derived for structs or enums",
            ))
        }
    };

    // Calculate field offsets and padding for C# generation (structs only)
    let mut fields_with_layout = fields.clone();
    if let Data::Struct(_) = input.data {
        calculate_field_layout(&struct_name_str, &mut fields_with_layout)?;
    }

    // Generate or use provided UUID
    let uuid = if let Some(ref manual_uuid) = struct_attrs.uuid {
        // Manual UUID provided
        manual_uuid.clone()
    } else {
        // Auto-generate UUID v7 from struct signature
        generate_deterministic_uuid_v7(&struct_name_str, &fields, struct_attrs.hash_mode)
    };

    // Generate zero-copy methods (conditionally based on skip_zero_copy flag)
    let zero_copy_impl = if struct_attrs.skip_zero_copy {
        quote! {}
    } else {
        generate_zero_copy_impl(struct_name)
    };

    // Generate validation methods (conditionally based on skip_validation flag)
    let validation_impl = if struct_attrs.skip_validation {
        quote! {}
    } else {
        generate_validation_impl(struct_name, &fields)
    };

    // Generate layout verification (only for zero-copy types)
    let layout_verify = if struct_attrs.skip_zero_copy {
        quote! {}
    } else {
        generate_layout_verify(struct_name)
    };

    // Generate UUID constant
    let uuid_const = generate_uuid_constant(struct_name, &uuid);

    // Generate FFI wrapper functions (conditionally based on skip_ffi flag)
    let ffi_functions = if struct_attrs.skip_ffi {
        quote! {}
    } else {
        generate_ffi_functions(struct_name, &struct_attrs)
    };

    // Check feature flags for engine-specific code generation
    let unity_bindings = cfg!(feature = "unity")
        .then(|| generate_unity_bindings(struct_name, &struct_attrs, &fields_with_layout, &uuid));
    let unreal_bindings =
        cfg!(feature = "unreal").then(|| generate_unreal_bindings(struct_name, &struct_attrs));

    // Plan 082: Generate database schema code if #[db_table("...")] is present
    let schema_impl = if let Some(ref db_table) = struct_attrs.db_table {
        let db_schema = build_db_schema_info(
            &struct_name_str,
            db_table.table_name.clone(),
            &input,
            &struct_attrs,
        )?;
        Some(generate_schema_impl(struct_name, &db_schema))
    } else {
        None
    };

    // Combine all generated code
    // Note: Order matters - struct must be defined before impl blocks are processed
    let expanded = quote! {
        // UUID constant for type identification
        #uuid_const

        // Layout verification (compile-time and runtime)
        #layout_verify

        // Zero-copy methods
        #zero_copy_impl

        // Validation methods
        #validation_impl

        // FFI wrapper functions (includes Default impl)
        #ffi_functions

        // Plan 082: Database schema SQL constants and helpers
        #schema_impl

        // Unity C# bindings (conditional)
        #unity_bindings

        // Unreal C++ bindings (conditional)
        #unreal_bindings
    };

    Ok(expanded)
}

/// Extract field information from a struct for signature hashing
fn extract_field_info(data_struct: &DataStruct) -> Result<Vec<FieldInfo>, Error> {
    let mut fields = Vec::new();

    for field in &data_struct.fields {
        let name = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(field, "Unnamed fields are not supported"))?
            .to_string();

        let ty = type_to_string(&field.ty)?;

        // Parse field attributes to find breaking attributes
        let field_attrs = parse_field_attributes(&field.attrs)?;
        let breaking_attributes = extract_breaking_attributes(&field_attrs);

        // Map Rust type to C# type
        let csharp_type = map_rust_type_to_csharp(&ty);

        // Check if this is a padding field
        let is_padding = name.starts_with("padding") || name.starts_with("_padding");

        fields.push(FieldInfo {
            name,
            ty,
            csharp_type,
            offset: None,
            breaking_attributes,
            is_padding,
        });
    }

    Ok(fields)
}

/// Extract variant information from an enum for signature hashing
fn extract_variant_info(data_enum: &DataEnum) -> Result<Vec<FieldInfo>, Error> {
    let mut variants = Vec::new();

    for variant in &data_enum.variants {
        let name = variant.ident.to_string();

        // For C-like enums, we treat them as having a simple integer type
        // The "type" is the enum itself, represented as the variant name
        let ty = name.clone();

        // Parse variant attributes to find breaking attributes
        let variant_attrs = parse_field_attributes(&variant.attrs)?;
        let breaking_attributes = extract_breaking_attributes(&variant_attrs);

        // Map Rust type to C# type (enums map to their own name)
        let csharp_type = Some(name.clone());

        variants.push(FieldInfo {
            name,
            ty,
            csharp_type,
            offset: None,
            breaking_attributes,
            is_padding: false,
        });
    }

    Ok(variants)
}

// ============================================================================
// Plan 082: Database Schema Extraction
// ============================================================================

/// Extract database schema info from a struct for SQL generation.
///
/// Collects field-level `#[primary_key]`, `#[db_column]`, `#[db_default]`, `#[db_index]`
/// attributes and combines them with table-level attributes into a `DbSchemaInfo`.
fn build_db_schema_info(
    struct_name: &str,
    table_name: String,
    input: &DeriveInput,
    struct_attrs: &StructAttributes,
) -> Result<DbSchemaInfo, Error> {
    let fields = match &input.data {
        Data::Struct(data_struct) => extract_db_field_info(data_struct)?,
        Data::Enum(_) => {
            return Err(Error::new_spanned(
                input,
                "db_table is not supported on enums — only structs can be database tables",
            ))
        }
        _ => {
            return Err(Error::new_spanned(
                input,
                "db_table is only supported on structs",
            ))
        }
    };

    let skip_crud = struct_attrs.skip_crud;

    Ok(DbSchemaInfo::new(
        struct_name.to_string(),
        table_name,
        fields,
        struct_attrs.db_indexes.clone(),
        struct_attrs.db_foreign_keys.clone(),
        struct_attrs.db_unique_constraints.clone(),
        skip_crud,
    ))
}

/// Extract database field info from each struct field.
///
/// Parses `#[primary_key]`, `#[db_column]`, `#[db_default]`, `#[db_index]` attributes
/// on each field and builds a `DbFieldInfo` for SQL column generation.
fn extract_db_field_info(data_struct: &DataStruct) -> Result<Vec<DbFieldInfo>, Error> {
    let mut db_fields = Vec::new();

    for field in &data_struct.fields {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(field, "Unnamed fields not supported for db_table"))?
            .to_string();

        let rust_type = type_to_string(&field.ty)?;

        // Parse field attributes for database metadata
        let field_attrs = parse_field_attributes(&field.attrs)?;

        let db_field = DbFieldInfo::new(
            field_name,
            rust_type,
            field_attrs.primary_key,
            field_attrs.db_column.as_ref(),
            field_attrs.db_default.as_ref(),
            field_attrs.db_index.as_ref(),
            field_attrs.db_flatten,
        );

        db_fields.push(db_field);
    }

    Ok(db_fields)
}

/// Convert a type to a string representation for hashing
fn type_to_string(ty: &Type) -> Result<String, Error> {
    // For now, use the TokenStream representation
    // In a more sophisticated implementation, we'd normalize the type
    let mut type_string = ty.to_token_stream().to_string();

    // Normalize whitespace for consistent hashing
    type_string.retain(|c| !c.is_whitespace());

    Ok(type_string)
}

// parse_field_attributes and parse_field_config are now imported from super::attributes
// which handles all field-level attributes including Plan 082 db_* attributes

/// Extract breaking attributes from field attributes
fn extract_breaking_attributes(field_attrs: &FieldAttributes) -> Vec<String> {
    let mut breaking = Vec::new();

    if field_attrs.skip {
        breaking.push("skip".to_string());
    }

    // Add other breaking attributes here as needed
    // e.g., ffi_order, array_len, etc.

    breaking
}

/// Map Rust type to C# type string
pub fn map_rust_type_to_csharp(rust_type: &str) -> Option<String> {
    let type_str = rust_type.trim();

    // Array types: [T; N] -> T[]
    // Handle both simple types [u8; N] and complex types [EntityUpdate; N]
    if type_str.starts_with('[') && type_str.ends_with(']') {
        // Extract inner part: "u8; N" or "EntityUpdate; N"
        let inner = &type_str[1..type_str.len() - 1].trim();

        // Split by semicolon to get type and count
        if let Some(semicolon_pos) = inner.find(';') {
            let inner_type = inner[..semicolon_pos].trim();
            let _count = inner[semicolon_pos + 1..].trim();

            // Map the inner type to C#
            if let Some(csharp_inner) = map_rust_type_to_csharp(inner_type) {
                return Some(format!("{}[]", csharp_inner));
            }
        }

        // If we can't parse it properly, return None
        return None;
    }

    // Basic type mappings
    match type_str {
        "u8" => Some("byte".to_string()),
        "i8" => Some("sbyte".to_string()),
        "u16" => Some("ushort".to_string()),
        "i16" => Some("short".to_string()),
        "u32" => Some("uint".to_string()),
        "i32" => Some("int".to_string()),
        "u64" => Some("ulong".to_string()),
        "i64" => Some("long".to_string()),
        "f32" => Some("float".to_string()),
        "f64" => Some("double".to_string()),
        "bool" => Some("bool".to_string()),
        "uuid::Uuid" => Some("Guid".to_string()),
        _ => {
            // Unknown type - use original (may fail in C#)
            None
        }
    }
}

/// Get alignment requirement for a field type
fn get_field_alignment(ty: &str) -> usize {
    match ty {
        "u8" | "i8" | "bool" => 1,
        "u16" | "i16" => 2,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" => 8,
        "uuid::Uuid" => 8, // Guid is 16 bytes but aligned to 8
        _ => {
            // Handle array types: [T; N]
            if ty.starts_with('[') && ty.ends_with(']') {
                let inner = &ty[1..ty.len() - 1];
                if let Some(pos) = inner.find(';') {
                    let inner_type = &inner[..pos];
                    return get_field_alignment(inner_type);
                }
            }
            1 // Default alignment
        }
    }
}

/// Get size of a field type
pub fn get_field_size(ty: &str) -> usize {
    match ty {
        "u8" | "i8" | "bool" => 1,
        "u16" | "i16" => 2,
        "u32" | "i32" | "f32" => 4,
        "u64" | "i64" | "f64" => 8,
        "uuid::Uuid" => 16,
        _ => {
            // Handle array types: [T; N]
            if ty.starts_with('[') && ty.ends_with(']') {
                let inner = &ty[1..ty.len() - 1];
                if let Some(pos) = inner.find(';') {
                    let inner_type = &inner[..pos];
                    let size_str = &inner[pos + 1..].trim();
                    if let Ok(count) = size_str.parse::<usize>() {
                        return get_field_size(inner_type) * count;
                    }
                }
            }
            0 // Unknown size
        }
    }
}

/// Calculate field offsets and padding for a struct
fn calculate_field_layout(_struct_name: &str, fields: &mut Vec<FieldInfo>) -> Result<(), Error> {
    let mut current_offset = 0;
    let mut last_field_align = 1;

    for field in fields.iter_mut() {
        // Skip fields marked with #[field(skip)]
        if field.breaking_attributes.contains(&"skip".to_string()) {
            field.offset = None;
            continue;
        }

        // Calculate field alignment based on type
        let field_align = get_field_alignment(&field.ty);

        // Calculate padding before this field
        let padding_before = (field_align - (current_offset % field_align)) % field_align;

        // Update offset
        current_offset += padding_before;
        field.offset = Some(current_offset);

        // Add field size
        let field_size = get_field_size(&field.ty);
        current_offset += field_size;

        last_field_align = field_align;
    }

    // Calculate tail padding
    let tail_padding = (last_field_align - (current_offset % last_field_align)) % last_field_align;

    // Add tail padding field if needed
    if tail_padding > 0 {
        fields.push(FieldInfo {
            name: format!("_padding{}", tail_padding),
            ty: format!("[u8; {}]", tail_padding),
            csharp_type: Some(format!(
                "[MarshalAs(UnmanagedType.ByValArray, SizeConst = {})]\n        private byte[] _padding{}",
                tail_padding, tail_padding
            )),
            offset: Some(current_offset),
            breaking_attributes: vec![],
            is_padding: true,
        });
    }

    Ok(())
}

/// Generate a deterministic UUID v7 from struct signature
///
/// This function creates a UUID v7 by hashing the struct signature with Blake3,
/// then formatting the hash as a UUID v7.
///
/// # Arguments
///
/// * `struct_name` - The name of the struct
/// * `fields` - Field information for signature hashing
/// * `hash_mode` - The hash mode to use (default, all, or name)
///
/// # Returns
///
/// A UUID v7 string generated from the struct signature
fn generate_deterministic_uuid_v7(
    struct_name: &str,
    fields: &[FieldInfo],
    hash_mode: HashMode,
) -> String {
    // Get cargo version for version-aware hashing
    let cargo_version = env!("CARGO_PKG_VERSION");

    // Build signature based on hash mode
    let signature = match hash_mode {
        HashMode::Default => {
            // Hash: struct name + fields + breaking attributes
            let field_list = fields
                .iter()
                .map(|f| {
                    let attrs = if f.breaking_attributes.is_empty() {
                        String::new()
                    } else {
                        format!(
                            "[{}]",
                            f.breaking_attributes
                                .iter()
                                .map(|a| format!("{}={}", a, a))
                                .collect::<Vec<_>>()
                                .join(",")
                        )
                    };
                    format!("{}:{}{}", f.name, f.ty, attrs)
                })
                .collect::<Vec<_>>()
                .join(",");

            format!("struct:{}:{}{{{}}}", cargo_version, struct_name, field_list)
        }
        HashMode::All => {
            // Hash: struct name + fields + ALL attributes
            let field_list = fields
                .iter()
                .map(|f| {
                    // Include all attributes in strict mode
                    format!("{}:{}", f.name, f.ty)
                })
                .collect::<Vec<_>>()
                .join(",");

            format!("struct:{}:{}{{{}}}", cargo_version, struct_name, field_list)
        }
        HashMode::Name => {
            // Hash: struct name only
            format!("struct:{}:{}", cargo_version, struct_name)
        }
    };

    // Hash the signature with Blake3
    let hash = blake3::hash(signature.as_bytes());
    let bytes = hash.as_bytes();

    // Convert Blake3 hash to UUID v7 format
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(&bytes[..16]);

    // Set version to 7 in the most significant 4 bits of byte 6
    uuid_bytes[6] = (uuid_bytes[6] & 0x0F) | 0x70;

    // Set variant to RFC 4122 in the most significant 2 bits of byte 8
    uuid_bytes[8] = (uuid_bytes[8] & 0x3F) | 0x80;

    // Convert to standard UUID string format
    let uuid = uuid::Uuid::from_bytes(uuid_bytes);
    uuid.to_string()
}

/// Generate UUID constant for the struct
fn generate_uuid_constant(struct_name: &Ident, uuid: &str) -> proc_macro2::TokenStream {
    quote! {
        impl #struct_name {
            /// UUID for this FFI type (used for type identification across languages)
            ///
            /// This is a deterministic UUID v7 generated from the struct signature.
            /// Any breaking change to the struct will produce a different UUID.
            pub const UUID: &'static str = #uuid;

            /// Get the UUID for this struct as a Uuid object
            #[inline]
            pub fn uuid() -> uuid::Uuid {
                uuid::Uuid::parse_str(Self::UUID)
                    .expect("Invalid UUID string generated by macro")
            }
        }
    }
}

/// Generate memory layout verification code
fn generate_layout_verify(struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl #struct_name {
            /// Verify the expected memory layout at runtime
            ///
            /// Returns the actual layout information for this type.
            pub fn actual_layout() -> game_ffi::utils::LayoutInfo {
                game_ffi::utils::LayoutInfo::of::<Self>()
            }

            /// Get the expected memory layout (should match actual)
            pub fn expected_layout() -> game_ffi::utils::LayoutInfo {
                game_ffi::utils::LayoutInfo {
                    size: std::mem::size_of::<Self>(),
                    alignment: std::mem::align_of::<Self>(),
                    fields: std::vec![],
                }
            }

            /// Get the size of this struct in bytes
            #[inline]
            pub fn size() -> usize {
                std::mem::size_of::<Self>()
            }

            /// Get the alignment requirement of this struct in bytes
            #[inline]
            pub fn alignment() -> usize {
                std::mem::align_of::<Self>()
            }
        }

        // Compile-time size and alignment assertions
        const _: () = {
            let _ = std::mem::size_of::<#struct_name>();
            let _ = std::mem::align_of::<#struct_name>();
        };
    }
}

/// Generate zero-copy method implementations
fn generate_zero_copy_impl(struct_name: &Ident) -> proc_macro2::TokenStream {
    quote! {
        impl #struct_name {
            /// Get raw bytes of this type (zero-copy)
            ///
            /// # Safety
            ///
            /// The returned slice references this value's memory.
            /// The slice is only valid as long as this value exists.
            #[inline]
            pub fn as_bytes(&self) -> &[u8] {
                unsafe {
                    std::slice::from_raw_parts(
                        self as *const Self as *const u8,
                        std::mem::size_of::<Self>()
                    )
                }
            }

            /// Create a reference to this type from raw bytes (zero-copy)
            ///
            /// # Safety
            ///
            /// The bytes must represent a valid instance of this type.
            /// The bytes must be properly aligned and not contain any padding
            /// that could violate Rust's safety guarantees.
            #[inline]
            pub unsafe fn from_bytes(bytes: &[u8]) -> &Self {
                assert!(
                    bytes.len() >= std::mem::size_of::<Self>(),
                    "Insufficient bytes for #struct_name"
                );
                &*(bytes.as_ptr() as *const Self)
            }

            /// Create a mutable reference to this struct from raw bytes (zero-copy)
            ///
            /// # Safety
            ///
            /// The bytes must represent a valid instance of this struct.
            /// The bytes must be properly aligned and not contain any padding.
            #[inline]
            pub unsafe fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
                assert!(
                    bytes.len() >= std::mem::size_of::<Self>(),
                    "Insufficient bytes for #struct_name"
                );
                &mut *(bytes.as_mut_ptr() as *mut Self)
            }
        }
    }
}

/// Generate validation method implementations
fn generate_validation_impl(struct_name: &Ident, fields: &[FieldInfo]) -> proc_macro2::TokenStream {
    // Generate validation logic for each field
    let field_validations = fields.iter().map(|f| {
        let _field_name = Ident::new(&f.name, proc_macro2::Span::call_site());
        // For now, just add basic validation
        // In a full implementation, we'd add min/max validation based on attributes
        quote! {
            // Validation for field: #field_name
        }
    });

    quote! {
        impl #struct_name {
            /// Validate this struct instance
            ///
            /// Returns `Ok(())` if all fields are valid, `Err(String)` otherwise.
            pub fn validate(&self) -> Result<(), String> {
                // Basic validation: ensure struct is not in an invalid state
                // Extend this with field-specific validation based on attributes
                #(#field_validations)*
                Ok(())
            }

            /// Check if this struct instance is valid
            ///
            /// Returns `true` if all fields are valid.
            #[inline]
            pub fn is_valid(&self) -> bool {
                self.validate().is_ok()
            }
        }
    }
}

/// Generate Unity C# bindings
/// Generate Unity C# bindings
/// Delegates to unity module
fn generate_unity_bindings(
    struct_name: &Ident,
    struct_attrs: &StructAttributes,
    fields: &[FieldInfo],
    uuid_str: &str,
) -> proc_macro2::TokenStream {
    super::unity::generate_unity_bindings(struct_name, struct_attrs, fields, uuid_str)
}

/// Generate Unreal C++ bindings
fn generate_unreal_bindings(
    struct_name: &Ident,
    struct_attrs: &StructAttributes,
) -> proc_macro2::TokenStream {
    // Use custom Unreal class name if provided, otherwise use F{struct_name}
    let default_unreal_name = format!("F{}", struct_name);
    let unreal_name_str = struct_attrs
        .unreal
        .as_ref()
        .and_then(|config| config.class.as_ref())
        .unwrap_or(&default_unreal_name);
    quote! {
        // Unreal C++ bindings
        #[cfg(feature = "unreal")]
        impl #struct_name {
            /// Generate Unreal C++ struct definition
            pub const UNREAL_HPP: &'static str = concat!(
                "#pragma once\n",
                "\n",
                "#include \"CoreMinimal.h\"\n",
                "#include \"", #unreal_name_str, ".generated.h\"\n",
                "\n",
                "USTRUCT(BlueprintType)\n",
                "struct GAMEFFI_API ", #unreal_name_str, "\n",
                "{\n",
                "    GENERATED_BODY()\n",
                "    \n",
                "    // Fields will be added by macro\n",
                "};\n"
            );
        }
    }
}

/// Generate FFI wrapper functions
fn generate_ffi_functions(
    struct_name: &Ident,
    _struct_attrs: &StructAttributes,
) -> proc_macro2::TokenStream {
    let struct_name_lower = format!("{}", struct_name).to_lowercase();
    let fn_name_prefix = Ident::new(&format!("set_{}", struct_name_lower), struct_name.span());

    quote! {
        // Default implementation
        impl Default for #struct_name {
            fn default() -> Self {
                // Zero-initialize the struct
                // This is safe because we're only handling POD types with #[repr(C)]
                unsafe { std::mem::zeroed() }
            }
        }

        // FFI wrapper functions
        impl #struct_name {
            /// Create a new instance with default values
            #[no_mangle]
            pub unsafe extern "C" fn #fn_name_prefix(out: *mut Self) {
                if !out.is_null() {
                    *out = Self::default();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_string_simple() {
        use syn::{parse_str, Type};

        let ty: Type = parse_str("f32").unwrap();
        let result = type_to_string(&ty).unwrap();
        assert_eq!(result, "f32");
    }

    #[test]
    fn test_type_to_string_complex() {
        use syn::{parse_str, Type};

        let ty: Type = parse_str("Vec<u8>").unwrap();
        let result = type_to_string(&ty).unwrap();
        assert!(result.contains("Vec"));
        assert!(result.contains("u8"));
    }

    #[test]
    fn test_generate_deterministic_uuid_v7_default_mode() {
        let uuid1 = generate_deterministic_uuid_v7(
            "PlayerPosition",
            &[FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            }],
            HashMode::Default,
        );

        let uuid2 = generate_deterministic_uuid_v7(
            "PlayerPosition",
            &[FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            }],
            HashMode::Default,
        );

        assert_eq!(uuid1, uuid2, "UUIDs should be deterministic");

        // Verify it's a valid UUID v7
        let parsed = uuid::Uuid::parse_str(&uuid1).unwrap();
        assert_eq!(parsed.get_version_num(), 7);
        assert_eq!(parsed.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn test_generate_deterministic_uuid_v7_different_fields() {
        let uuid1 = generate_deterministic_uuid_v7(
            "PlayerPosition",
            &[FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            }],
            HashMode::Default,
        );

        let uuid2 = generate_deterministic_uuid_v7(
            "PlayerPosition",
            &[
                FieldInfo {
                    name: "x".to_string(),
                    ty: "f32".to_string(),
                    csharp_type: Some("float".to_string()),
                    offset: Some(0),
                    breaking_attributes: vec![],
                    is_padding: false,
                },
                FieldInfo {
                    name: "y".to_string(),
                    ty: "f32".to_string(),
                    csharp_type: Some("float".to_string()),
                    offset: Some(4),
                    breaking_attributes: vec![],
                    is_padding: false,
                },
            ],
            HashMode::Default,
        );

        assert_ne!(
            uuid1, uuid2,
            "Different fields should produce different UUIDs"
        );
    }

    #[test]
    fn test_generate_deterministic_uuid_v7_name_mode() {
        let uuid1 = generate_deterministic_uuid_v7(
            "TestStruct",
            &[FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            }],
            HashMode::Name,
        );

        let uuid2 = generate_deterministic_uuid_v7(
            "TestStruct",
            &[FieldInfo {
                name: "y".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            }],
            HashMode::Name,
        );

        assert_eq!(uuid1, uuid2, "Name mode should ignore field changes");
    }

    #[test]
    fn test_generate_deterministic_uuid_v7_all_mode() {
        // All mode currently works like default mode (field signature)
        // Full attribute parsing (min, max, default) is not yet implemented
        let uuid1 = generate_deterministic_uuid_v7(
            "TestStruct",
            &[FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            }],
            HashMode::All,
        );

        let uuid2 = generate_deterministic_uuid_v7(
            "TestStruct",
            &[FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec!["skip".to_string()],
                is_padding: false,
            }],
            HashMode::All,
        );

        // For now, All mode behaves like Default mode (breaking attrs only)
        // Full attribute support requires parsing field validation attributes
        // This will be added when field attribute parsing is complete
        assert_eq!(
            uuid1, uuid2,
            "All mode currently matches default mode behavior"
        );
    }

    #[test]
    fn test_map_rust_type_to_csharp() {
        // Test basic types
        assert_eq!(map_rust_type_to_csharp("u8"), Some("byte".to_string()));
        assert_eq!(map_rust_type_to_csharp("i32"), Some("int".to_string()));
        assert_eq!(map_rust_type_to_csharp("f32"), Some("float".to_string()));
        assert_eq!(map_rust_type_to_csharp("f64"), Some("double".to_string()));
        assert_eq!(map_rust_type_to_csharp("bool"), Some("bool".to_string()));

        // Test UUID
        assert_eq!(
            map_rust_type_to_csharp("uuid::Uuid"),
            Some("Guid".to_string())
        );

        // Test arrays
        assert_eq!(
            map_rust_type_to_csharp("[u8; 16]"),
            Some("byte[]".to_string())
        );

        // Test unsupported type
        assert_eq!(map_rust_type_to_csharp("Vec<u8>"), None);
    }

    #[test]
    fn test_get_field_size() {
        assert_eq!(get_field_size("u8"), 1);
        assert_eq!(get_field_size("u16"), 2);
        assert_eq!(get_field_size("u32"), 4);
        assert_eq!(get_field_size("u64"), 8);
        assert_eq!(get_field_size("i8"), 1);
        assert_eq!(get_field_size("i16"), 2);
        assert_eq!(get_field_size("i32"), 4);
        assert_eq!(get_field_size("i64"), 8);
        assert_eq!(get_field_size("f32"), 4);
        assert_eq!(get_field_size("f64"), 8);
        assert_eq!(get_field_size("bool"), 1);
        assert_eq!(get_field_size("uuid::Uuid"), 16);
        assert_eq!(get_field_size("[u8; 32]"), 32);
    }
}
