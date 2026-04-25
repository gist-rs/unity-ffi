# Plan 001: `#[db_flatten]` — Single Source of Truth for Shared Fields

## Problem

`PlayerPos` (FFI wire packet) and `PlayerPositionRecord` (DB row) share position fields (`player_id`, `x`, `y`).  
In a real MMORPG this grows to 20+ fields (`z`, `rotation`, `velocity`, `health`, `mana`, etc.).  
Maintaining two copies of every field is fragile and violates DRY.

## Solution: Composition + `#[db_flatten]`

Extract shared fields into `Position2D`, embed it in both structs, and teach `#[db_table]` to expand embedded struct columns.

```rust
// Single source of truth
#[derive(Debug, Clone, Copy, GameComponent)]
#[db_table("position_2d")]
pub struct Position2D {
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
}

// FFI packet = header + shared payload
#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy)]
pub struct PlayerPos {
    pub packet_type: u8,
    pub magic: u8,
    pub request_uuid: uuid::Uuid,
    pub pos: Position2D,
}

// DB record = metadata + shared payload (flattened into columns)
#[derive(Debug, Clone, GameComponent)]
#[game_ffi(skip_zero_copy, skip_ffi, skip_crud)]
#[db_table("player_positions")]
pub struct PlayerPositionRecord {
    #[primary_key]
    pub id: i64,
    #[db_flatten]  // expands Position2D's columns into this table
    pub pos: Position2D,
    pub tick: u32,
    pub created_at: i64,
}
```

## Proc Macro Constraint

Proc macros **cannot** introspect other types at derive time.  
Solution: each `#[db_table]` struct gets a new `COLUMN_DEFS_SQL` const.  
Parent composes `CREATE_TABLE_SQL` at runtime via `format!()`.

## Tasks

- [x] **T1: Add `#[db_flatten]` field attribute parsing** — `attributes.rs`
  - Add `db_flatten: bool` to `FieldAttributes`
  - Parse `#[db_flatten]` in `parse_field_attributes`

- [x] **T2: Add `flatten` flag to `DbFieldInfo`** — `types.rs` (derive)
  - Add `flatten: bool` field
  - Update `DbFieldInfo::new()` signature
  - Skip flattened fields from direct column SQL generation

- [x] **T3: Generate `COLUMN_DEFS_SQL` on all `#[db_table]` structs** — `sql_gen.rs`
  - New const: just column definitions without `CREATE TABLE (...)` wrapper
  - Example: `"player_id BIGINT NOT NULL,\n    x REAL NOT NULL"`

- [x] **T4: Handle `#[db_flatten]` in SQL generation** — `sql_gen.rs`
  - When any field has `flatten: true`:
    - `TABLE_NAME` stays `const`
    - `CREATE_TABLE_SQL` becomes `fn create_table_sql() -> String` (runtime composition)
    - `COLUMN_DEFS_SQL` still generated as `const` (own columns only, no flattened)
    - `CREATE_INDEXES_SQL` stays `const`
  - Generate `fn create_table_sql()` that calls `<FlattenedType>::COLUMN_DEFS_SQL`

- [x] **T5: Pass `flatten` through `extract_db_field_info`** — `game_component.rs`
  - Read `field_attrs.db_flatten` and pass to `DbFieldInfo::new()`

- [x] **T6: Refactor `types.rs` (unity-network)** — composition
  - Add `Position2D` struct with `#[db_table("position_2d")]`
  - Refactor `PlayerPos` to embed `Position2D`
  - Refactor `PlayerPositionRecord` to embed `Position2D` with `#[db_flatten]`
  - Simplify `from_player_pos()` to `pos: packet.pos`

- [x] **T7: Update `schema_turso.rs` example**
  - Use `create_table_sql()` instead of `CREATE_TABLE_SQL` (API change for flatten)
  - Access fields via `record.pos.x` instead of `record.x`

- [x] **T8: Add derive macro tests** — `sql_gen.rs` tests
  - Test `COLUMN_DEFS_SQL` generation
  - Test `#[db_flatten]` skips field from own columns
  - Test `create_table_sql()` composition

- [x] **T9: Run tests + clippy + fix diagnostics**

- [x] **T10: Update README.md** — add `#[db_flatten]` to attribute reference

## Key Decisions

| Decision | Rationale |
|----------|-----------|
| `COLUMN_DEFS_SQL` const on every `#[db_table]` | Enables parent to compose SQL without type introspection |
| `fn create_table_sql() -> String` for flatten | Can't concat const strings at compile time; `OnceLock` not worth complexity |
| `CREATE_TABLE_SQL` stays const for non-flatten structs | No API break for existing code |
| Flattened type must also have `#[db_table]` | So it has `COLUMN_DEFS_SQL` available |

## File Change Summary

| File | Change |
|------|--------|
| `crates/game-ffi-derive/src/derive/attributes.rs` | Add `db_flatten: bool` to `FieldAttributes` |
| `crates/game-ffi-derive/src/derive/schema/types.rs` | Add `flatten: bool` to `DbFieldInfo` |
| `crates/game-ffi-derive/src/derive/schema/sql_gen.rs` | Generate `COLUMN_DEFS_SQL`, handle flatten in `generate_schema_impl` |
| `crates/game-ffi-derive/src/derive/game_component.rs` | Pass `db_flatten` through `extract_db_field_info` |
| `crates/unity-network/src/types.rs` | Add `Position2D`, refactor `PlayerPos`, `PlayerPositionRecord` |
| `crates/unity-network/examples/schema_turso.rs` | Update for new API |
| `crates/unity-network/README.md` | Add `#[db_flatten]` docs |