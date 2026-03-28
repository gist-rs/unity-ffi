//! Unreal Engine C++ binding generation for GameComponent derive macro
//!
//! This module handles generation of C++ code for Unreal Engine integration,
//! including struct definitions, USTRUCT specifiers, and helper methods.

use proc_macro2::Ident;
use quote::quote;

use crate::derive::attributes::StructAttributes;

/// Generate Unreal Engine C++ bindings for a struct
///
/// This generates a `UNREAL_HPP` const string containing the complete C++ struct
/// definition with USTRUCT specifiers and proper memory layout.
#[allow(dead_code)] // Part of public API, used by derive macro
pub fn generate_unreal_bindings(
    struct_name: &Ident,
    struct_attrs: &StructAttributes,
) -> proc_macro2::TokenStream {
    // Use custom Unreal class name if provided, otherwise use struct name
    let default_name = struct_name.to_string();
    let unreal_class = struct_attrs
        .unreal
        .as_ref()
        .and_then(|config| config.class.as_ref())
        .unwrap_or(&default_name);

    // Determine specifiers
    let blueprint_type = if struct_attrs
        .unreal
        .as_ref()
        .map(|c| c.blueprint_type)
        .unwrap_or(false)
    {
        ", BlueprintType"
    } else {
        ""
    };

    quote! {
        // Unreal Engine C++ bindings
        #[cfg(feature = "unreal")]
        impl #struct_name {
            pub const UNREAL_HPP: &'static str = concat!(
                "USTRUCT(BlueprintType)\n",
                "struct ", #unreal_class, #blueprint_type, "\n",
                "{\n",
                "    GENERATED_BODY()\n",
                "\n",
                "    // Auto-generated from Rust struct: ", #struct_name, "\n",
                "    // Regenerate with: cargo run --package unity-network --example generate_unreal_cpp\n",
                "};\n"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_str;

    #[test]
    fn test_generate_unreal_bindings_basic() {
        let struct_name = parse_str::<Ident>("TestStruct").unwrap();
        let struct_attrs = StructAttributes::default();

        let result = generate_unreal_bindings(&struct_name, &struct_attrs);
        let result_str = result.to_string();

        assert!(result_str.contains("TestStruct"));
        assert!(result_str.contains("USTRUCT"));
        assert!(result_str.contains("GENERATED_BODY"));
    }

    #[test]
    fn test_generate_unreal_bindings_with_custom_class() {
        let struct_name = parse_str::<Ident>("TestStruct").unwrap();
        let struct_attrs = StructAttributes {
            unreal: Some(crate::derive::attributes::UnrealConfig {
                class: Some("ACustomStruct".to_string()),
                blueprint_type: true,
            }),
            ..Default::default()
        };

        let result = generate_unreal_bindings(&struct_name, &struct_attrs);
        let result_str = result.to_string();

        assert!(result_str.contains("ACustomStruct"));
        assert!(result_str.contains("BlueprintType"));
    }

    #[test]
    fn test_generate_unreal_bindings_blueprint_type() {
        let struct_name = parse_str::<Ident>("TestStruct").unwrap();
        let struct_attrs = StructAttributes {
            unreal: Some(crate::derive::attributes::UnrealConfig {
                class: None,
                blueprint_type: true,
            }),
            ..Default::default()
        };

        let result = generate_unreal_bindings(&struct_name, &struct_attrs);
        let result_str = result.to_string();

        assert!(result_str.contains("BlueprintType"));
    }
}
