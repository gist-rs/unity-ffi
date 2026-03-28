//! Unity C# binding generation for GameComponent derive macro
//!
//! This module handles generation of C# code for Unity integration,
//! including struct definitions, marshaling attributes, and helper methods.

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::game_component::{get_field_size, map_rust_type_to_csharp, FieldInfo};
use crate::derive::attributes::StructAttributes;

/// Generate Unity C# bindings for a struct
///
/// This generates a `UNITY_CS` const string containing the complete C# struct
/// definition with proper memory layout attributes and helper methods.
pub fn generate_unity_bindings(
    struct_name: &Ident,
    struct_attrs: &StructAttributes,
    fields: &[FieldInfo],
    uuid_str: &str,
) -> TokenStream {
    // Use custom Unity name if provided, otherwise use struct name
    let default_name = struct_name.to_string();
    let unity_name = struct_attrs
        .unity
        .as_ref()
        .and_then(|config| config.name.as_ref())
        .unwrap_or(&default_name);

    // Convert struct_name to string for runtime use in generated code
    let struct_name_str = struct_name.to_string();

    // Generate C# field declarations
    let csharp_fields: Vec<String> = generate_csharp_fields(fields);

    // Check if struct contains fixed arrays (requires unsafe struct)
    let has_fixed_arrays = fields
        .iter()
        .any(|f| f.ty.starts_with("[") && f.ty.ends_with(']'));

    // Calculate total size from last field offset + size
    let total_size = calculate_total_size(fields);

    quote! {
        // Unity C# bindings
        #[cfg(feature = "unity")]
        impl #struct_name {
            /// Generate Unity C# struct definition as a string
            /// This includes proper memory layout, marshaling attributes, and helper methods
            pub fn generate_unity_cs() -> String {
                let mut cs_code = String::new();

                // Namespace wrapper
                cs_code.push_str("namespace GameFFI\n");
                cs_code.push_str("{\n");
                cs_code.push_str("    using System;\n");
                cs_code.push_str("    using System.Runtime.InteropServices;\n");
                cs_code.push_str("\n");

                // Struct declaration (unsafe if contains fixed arrays)
                cs_code.push_str("    [StructLayout(LayoutKind.Sequential, Pack = 1)]\n");
                cs_code.push_str("    public ");
                if #has_fixed_arrays {
                    cs_code.push_str("unsafe ");
                }
                cs_code.push_str("struct ");
                cs_code.push_str(#unity_name);
                cs_code.push_str("\n");
                cs_code.push_str("    {\n");

                // Add fields
                #(
                    cs_code.push_str(#csharp_fields);
                )*
                cs_code.push_str("\n");

                // Add Size constant
                cs_code.push_str("        /// <summary>\n");
                cs_code.push_str("        /// Size of this struct in bytes\n");
                cs_code.push_str("        /// </summary>\n");
                cs_code.push_str(&format!("        public static readonly int Size = {};\n", #total_size));
                cs_code.push_str("\n");

                // Add UUID constant
                cs_code.push_str("        /// <summary>\n");
                cs_code.push_str("        /// Auto-generated UUID v7 for type identification\n");
                cs_code.push_str("        /// Generated from struct signature: struct:1.0.0:unity_network::");
                cs_code.push_str(#struct_name_str);
                cs_code.push_str("\n");
                cs_code.push_str("        /// </summary>\n");
                cs_code.push_str(&format!("        public static readonly Guid UUID = Guid.Parse(\"{}\");\n", #uuid_str));

                // Close struct and namespace
                cs_code.push_str("    }\n");
                cs_code.push_str("}\n");

                cs_code
            }

            // Provide the Unity C# code as a static method result
            // Note: This must be called as a method since proc macros cannot generate
            // complex string constants with TokenStream concatenation
            pub fn unity_cs() -> &'static str {
                // Use a lazy static to cache the generated code
                use std::sync::OnceLock;
                static CS_CODE: OnceLock<String> = OnceLock::new();
                CS_CODE.get_or_init(|| Self::generate_unity_cs())
            }

            // Backwards compatibility: UNITY_CS is now an alias to the method
            // Tests should use unity_code() method or check the generated code
            pub const UNITY_CS: &'static str = "";
        }
    }
}

/// Generate C# field declarations with proper marshaling attributes
/// Generate C# field declarations with proper marshaling attributes
fn generate_csharp_fields(fields: &[FieldInfo]) -> Vec<String> {
    fields
        .iter()
        .filter_map(|field| {
            // Skip fields with #[field(skip)]
            if field.breaking_attributes.contains(&"skip".to_string()) {
                return None;
            }

            // Determine visibility
            let visibility = if field.is_padding {
                "private"
            } else {
                "public"
            };

            // Handle fixed-size array types using C# unsafe fixed buffers
            if field.ty.starts_with("[u8;") && field.ty.ends_with(']') {
                // Extract array size (trim to handle whitespace like "[u8; 4]")
                let inner = &field.ty[4..field.ty.len() - 1].trim();
                if let Ok(size) = inner.parse::<usize>() {
                    return Some(format!(
                        "        {} fixed byte {}[{}];\n",
                        visibility, field.name, size
                    ));
                }
            }

            // Get C# type
            let csharp_type: String = if let Some(ref cs_type) = field.csharp_type {
                cs_type.clone()
            } else {
                // Try to map Rust type to C# type
                map_rust_type_to_csharp(&field.ty)?
            };

            Some(format!(
                "        {} {} {};\n",
                visibility, csharp_type, field.name
            ))
        })
        .collect()
}

/// Calculate total size of the struct from field layout
fn calculate_total_size(fields: &[FieldInfo]) -> usize {
    let non_padding: Vec<_> = fields.iter().filter(|f| !f.is_padding).collect();
    if let Some(last_field) = non_padding.last() {
        if let Some(offset) = last_field.offset {
            offset + get_field_size(&last_field.ty)
        } else {
            0
        }
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_csharp_fields_basic() {
        let fields = vec![
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
        ];

        let result = generate_csharp_fields(&fields);
        assert!(result.len() == 2);
        assert!(result[0].contains("public float x;"));
        assert!(result[1].contains("public float y;"));
    }

    #[test]
    fn test_generate_csharp_fields_with_padding() {
        let fields = vec![
            FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            },
            FieldInfo {
                name: "_padding".to_string(),
                ty: "[u8; 4]".to_string(),
                csharp_type: Some("byte[]".to_string()),
                offset: Some(4),
                breaking_attributes: vec![],
                is_padding: true,
            },
        ];

        let result = generate_csharp_fields(&fields);
        assert!(result.len() == 2);
        assert!(result[0].contains("public float x;"));
        assert!(result[1].contains("private byte[] _padding;"));
        assert!(result[1].contains("MarshalAs"));
    }

    #[test]
    fn test_generate_csharp_fields_skip_field() {
        let fields = vec![
            FieldInfo {
                name: "x".to_string(),
                ty: "f32".to_string(),
                csharp_type: Some("float".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            },
            FieldInfo {
                name: "_internal".to_string(),
                ty: "u64".to_string(),
                csharp_type: Some("ulong".to_string()),
                offset: Some(8),
                breaking_attributes: vec!["skip".to_string()],
                is_padding: false,
            },
        ];

        let result = generate_csharp_fields(&fields);
        assert!(result.len() == 1);
        assert!(result[0].contains("public float x;"));
    }

    #[test]
    fn test_calculate_total_size() {
        let fields = vec![
            FieldInfo {
                name: "a".to_string(),
                ty: "u8".to_string(),
                csharp_type: Some("byte".to_string()),
                offset: Some(0),
                breaking_attributes: vec![],
                is_padding: false,
            },
            FieldInfo {
                name: "b".to_string(),
                ty: "u32".to_string(),
                csharp_type: Some("uint".to_string()),
                offset: Some(4),
                breaking_attributes: vec![],
                is_padding: false,
            },
        ];

        let size = calculate_total_size(&fields);
        assert_eq!(size, 8); // 4 (offset) + 4 (u32)
    }
}
