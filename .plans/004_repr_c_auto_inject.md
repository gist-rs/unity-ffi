# Plan 004: Auto-inject `#[repr(C)]` via `#[unity]` / `#[unreal]` Attribute Macros

## Verdict: YES — Do It

`#[repr(C)]` is manual, forgettable, and invisible when missing (no compile error, just silent memory corruption at runtime).
Making `#[unity]` and `#[unreal]` the entry point that auto-injects `#[repr(C)]` + `#[derive(GameComponent)]` is the right call.

## Problem

1. **`#[repr(C)]` is easily forgotten** — no compile error, only silent UB when crossing FFI boundary
2. **Tests were missing it too** (fixed in both repos, but proves the pattern is fragile)
3. **Three annotations for one intent** — `#[repr(C)]` + `#[derive(GameComponent)]` + `#[unity(name = "...")]` is boilerplate
4. **Semantic disconnect** — `#[repr(C)]` is a Rust layout detail, not a business intent. The user's intent is "this type crosses to Unity"

## Solution: `#[unity]` and `#[unreal]` as Attribute Macros

```rust
// BEFORE (3 annotations, easy to forget #[repr(C)])
#[repr(C)]
#[derive(GameComponent)]
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos {
    pub x: f32,
    pub y: f32,
}

// AFTER (1 annotation — impossible to forget #[repr(C)])
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos {
    pub x: f32,
    pub y: f32,
}
```

### What each annotation means (after this plan)

| Annotation | `#[repr(C)]` | `#[derive(GameComponent)]` | Use case |
|---|---|---|---|
| `#[unity(...)]` | ✅ auto | ✅ auto | FFI types sent to Unity |
| `#[unreal(...)]` | ✅ auto | ✅ auto | FFI types sent to Unreal |
| `#[derive(GameComponent)]` alone | ❌ no | ✅ | DB-only, internal types (correct — no FFI boundary) |

### How it works internally

```
User writes:
  #[unity(name = "PlayerPosUnity")]
  struct PlayerPos { ... }

#[unity] attribute macro expands to:
  #[repr(C)]
  #[derive(GameComponent)]
  #[__game_ffi_unity(name = "PlayerPosUnity")]   ← internal helper, derive parses this
  struct PlayerPos { ... }

GameComponent derive sees:
  - #[repr(C)] already on struct ✅
  - #[__game_ffi_unity] helper attr → reads Unity name/config
  - Generates impl blocks as before
```

Key: `#[unity]` cannot re-emit `#[unity]` (infinite loop), so it passes config through
internal `#[__game_ffi_unity]` that the derive macro registers as a helper attribute.

### Unreal support — same pattern

```rust
#[unreal(class = "FCharacterUpdate", blueprint_type)]
pub struct CharacterUpdate {
    pub char_id: u64,
    pub x: u16,
    pub y: u16,
}

// Expands to:
// #[repr(C)]
// #[derive(GameComponent)]
// #[__game_ffi_unreal(class = "FCharacterUpdate", blueprint_type)]
// pub struct CharacterUpdate { ... }
```

### What about structs needing BOTH unity + unreal?

Rare in practice, but supported via manual derive (backward compat):

```rust
// Old way still works — unity/unreal are derive helper attrs
#[repr(C)]
#[derive(GameComponent)]
#[unity(name = "PlayerPosUnity")]
#[unreal(class = "FPlayerPos")]
pub struct DualStruct { ... }
```

Future: could add `#[game_ffi(unity = "...", unreal = "...")]` as a combined entry point.

## Backward Compatibility

**No breaking change.** Existing code continues to work:

```rust
// OLD — still works (unity is a derive helper attr)
#[repr(C)]
#[derive(GameComponent)]
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos { ... }

// NEW — preferred
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos { ... }
```

The derive macro continues to parse both `#[unity]` (helper) and `#[__game_ffi_unity]` (internal helper).

## Tasks

### Phase 1: `#[unity]` attribute macro

- [ ] **T1: Add `#[unity]` attribute macro in `lib.rs`**
  - Add `#[proc_macro_attribute]` for `unity`
  - Parse attribute args (`name = "..."`, `read_only`)
  - Parse input struct
  - Inject `#[repr(C)]`, `#[derive(GameComponent)]`, `#[__game_ffi_unity(...)]`
  - Return modified struct

- [ ] **T2: Register `__game_ffi_unity` as derive helper**
  - Add `__game_ffi_unity` to `#[proc_macro_derive(GameComponent, attributes(...))]`
  - Derive macro parses `#[__game_ffi_unity]` same as current `#[unity]` parsing

- [ ] **T3: Update derive attribute parsing**
  - `parse_struct_attributes`: also check for `#[__game_ffi_unity]` → parse same as `#[unity]`
  - Keep `#[unity]` parsing for backward compat

- [ ] **T4: Update test structs in `derive_tests.rs`**
  - `PlayerPos` → use `#[unity(name = "PlayerPosUnity")]` alone (no `#[repr(C)]`, no `#[derive]`)
  - Other test structs → keep `#[derive(GameComponent)]` + `#[repr(C)]` (no unity attr)
  - Verify all 37 tests still pass

- [ ] **T5: Update `unity-ffi` production types**
  - `types.rs`: `PacketHeader`, `PlayerPos`, `GameState`, `SpriteMessage` → use `#[unity]` attribute macro
  - Remove manual `#[repr(C)]` and `#[derive(GameComponent)]` from those structs
  - Keep `#[derive(GameComponent)]` on DB-only types (`PlayerPositionRecord`, `Position2D`)

### Phase 2: `#[unreal]` attribute macro

- [ ] **T6: Add `#[unreal]` attribute macro in `lib.rs`**
  - Same pattern as `#[unity]`
  - Inject `#[repr(C)]`, `#[derive(GameComponent)]`, `#[__game_ffi_unreal(...)]`

- [ ] **T7: Register `__game_ffi_unreal` as derive helper**
  - Add `__game_ffi_unreal` to derive attributes list
  - Parse same as current `#[unreal]` parsing

- [ ] **T8: Update test struct `CharacterUpdate`**
  - Use `#[unreal(class = "FCharacterUpdate", blueprint_type)]` alone
  - Verify tests pass

### Phase 3: Sync to mu-maxage-shop

- [ ] **T9: Port changes to `mu-maxage-shop/mmorpg/crates/game-ffi-derive`**
  - Same T1-T8 changes
  - Update mmorpg test structs
  - Verify all tests pass

### Phase 4: Safety net

- [ ] **T10: Add compile-time check in derive macro**
  - When `skip_zero_copy` is NOT set and struct has NO `#[repr(C)]`
  - Emit compile error: "GameComponent requires #[repr(C)] for zero-copy types. Use #[unity(...)] to auto-inject it."
  - This catches anyone using bare `#[derive(GameComponent)]` on FFI types

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| `#[unity]` / `#[unreal]` are attribute macros, not derive | Attribute macros can modify struct (inject `#[repr(C)]`), derive cannot |
| Internal `#[__game_ffi_unity]` helper | Prevents infinite loop (attribute macro can't re-emit itself) |
| Keep backward compat with `#[unity]` as derive helper | No migration needed — old code works, new code is simpler |
| `#[derive(GameComponent)]` alone = no `#[repr(C)]` | Correct for DB-only / internal types that never cross FFI |
| Compile-time check for missing `#[repr(C)]` | Safety net — catches mistakes before they become silent UB |

## File Change Summary

| File | Change |
|------|--------|
| `crates/game-ffi-derive/src/lib.rs` | Add `#[proc_macro_attribute]` for `unity` and `unreal`, register internal helpers |
| `crates/game-ffi-derive/src/derive/attributes.rs` | Parse `#[__game_ffi_unity]` and `#[__game_ffi_unreal]` as aliases |
| `crates/game-ffi-derive/src/derive/game_component.rs` | Add compile-time `#[repr(C)]` check |
| `crates/game-ffi/tests/derive_tests.rs` | Update `PlayerPos` to use `#[unity]` attribute macro |
| `crates/unity-network/src/types.rs` | Update FFI structs to use `#[unity]` attribute macro |
| `mu-maxage-shop/...` (same files) | Port all changes |

## Migration Guide (for users)

```rust
// Before:
#[repr(C)]
#[derive(GameComponent)]
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos { ... }

// After:
#[unity(name = "PlayerPosUnity")]
pub struct PlayerPos { ... }

// DB-only types — no change needed:
#[derive(GameComponent)]
#[game_ffi(skip_zero_copy, skip_ffi)]
#[db_table("records")]
pub struct SomeRecord { ... }
```
