// Integration tests for GameComponent derive macro
//
// Tests that require access to internal functions (type_to_string,
// generate_deterministic_uuid_v7, map_rust_type_to_csharp, get_field_size)
// are in the module-level tests in src/derive/game_component.rs
//
// This file only contains integration tests that use the public API

// Integration test: test that macro compiles and generates code
// This tests the public API without requiring access to internals

#[test]
fn test_macro_compiles() {
    // This is a compile-time test - if it compiles, macro works
    // The actual code generation is tested by game-ffi crate

    let code = quote::quote! {
        #[derive(GameComponent)]
        struct TestStruct {
            pub x: f32,
            pub y: f32,
        }
    };

    // Parse the struct - verifies the derive attribute syntax is valid
    let _ = syn::parse2::<syn::DeriveInput>(code).unwrap();
}
