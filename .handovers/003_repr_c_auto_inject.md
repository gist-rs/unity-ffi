# Handover 003: Auto-inject `#[repr(C)]` via `#[unity]` / `#[unreal]` Attribute Macros

## What Happened

Implemented Plan 004 — `#[unity]` and `#[unreal]` attribute macros that auto-inject `#[repr(C)]` and `#[derive(GameComponent)]`, eliminating a common footgun where forgetting `#[repr(C)]` causes silent memory corruption at FFI boundaries.

All 10 tasks (T1–T10) completed for `unity-ffi`. Phase 3 (sync to `mu-maxage-shop`) is not part of this repo.

## Where Is the Plan/Code/Test

- **Plan**: `.plans/004_repr_c_auto_inject.md`
- **Code changed**:
  - `crates/game-ffi-derive/src/lib.rs` — Added `#[proc_macro_attribute]` entries for `unity` and `unreal`
  - `crates/game-ffi-derive/src/derive/mod.rs` — Re-exported `unity_attribute`, `unreal_attribute`
  - `crates/game-ffi-derive/src/derive/game_component.rs` — Added `expand_engine_attribute()`, `unity_attribute()`, `unreal_attribute()`, `has_repr_c()` safety net
  - `crates/game-ffi-derive/src/derive/attributes.rs` — `parse_struct_attributes` now also parses `#[__game_ffi_unity]` and `#[__game_ffi_unreal]` internal helpers
  - `crates/game-ffi/src/lib.rs` — Re-exported `unity` and `unreal` attribute macros; fixed doc test missing `#[repr(C)]`
  - `crates/game-ffi/tests/derive_tests.rs` — `PlayerPos` uses `#[unity]` alone; `CharacterUpdate` uses `#[unreal]` alone
  - `crates/unity-network/src/types.rs` — `PacketHeader`, `PlayerPos`, `GameState`, `SpriteMessage` now use `#[unity]` alone
  - `crates/game-ffi/examples/basic_usage.rs` — Added `#[repr(C)]` to all example structs (safety net catches missing ones)
- **Tests**: `cargo test -p game-ffi --test derive_tests` (37 tests pass), full workspace `cargo test --quiet` (194+ tests pass)

## Reflection — Struggling / Solved

- **`syn::Attribute` doesn't implement `Parse`**: Initial approach tried `syn::parse2::<syn::Attribute>(ts)` which fails. Solved by building a combined `quote!` token stream and re-parsing the entire `DeriveInput` — simpler and lets syn normalise the attribute list.
- **Safety net catches too much**: The `#[repr(C)]` check (T10) surfaced missing annotations in `examples/basic_usage.rs` and a doc test in `lib.rs`. Fixed all of them — these were pre-existing latent issues.
- **Backward compat verified**: Old-style `#[repr(C)] #[derive(GameComponent)] #[unity(name = "...")]` still works alongside new `#[unity(name = "...")]` alone.

## Remain Work

- **Phase 3 (T9)**: Port same changes to `mu-maxage-shop/mmorpg/crates/game-ffi-derive` — same pattern, separate repo.
- **Future enhancement**: Combined `#[game_ffi(unity = "...", unreal = "...")]` entry point for structs needing both engines (currently use manual derive with both attrs).

## Issues Ref

- None (no issues created)

## How to Dev/Test

```bash
# Build everything
cargo build --quiet

# Run all tests
cargo test --quiet

# Run derive-specific tests only
cargo test -p game-ffi --test derive_tests --quiet

# Run example
cargo run -p game-ffi --example basic_usage --quiet

# Clippy
cargo clippy --quiet --allow-dirty
```
