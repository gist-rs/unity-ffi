# unity-network

Safe Rust FFI bridge for Unity, providing WebTransport networking with zero-copy packet handling and auto-schema database persistence.

## Architecture

```
┌─────────┐     FFI (cdylib)     ┌───────────────────┐     WebTransport     ┌──────────────┐
│  Unity  │ ◄──────────────────► │  unity-network    │ ◄──────────────────► │ Game Server  │
│  (C#)   │   network_*() calls  │  (Rust cdylib)    │   QUIC/HTTP3         │ (wtransport) │
└─────────┘                      └───────────────────┘                      └──────────────┘
                                          │
                                          ▼
                                   ┌──────────────┐
                                   │ turso SQLite │
                                   │ (persistence)│
                                   └──────────────┘
```

**Unity is VIEW-ONLY** — no business logic, no state, no networking. Rust handles everything.

## Auto Schema Flow

The `schema_turso` example demonstrates recording player positions from network packets into a turso SQLite database using auto-generated DDL from `#[derive(GameComponent)]`.

```mermaid
stateDiagram-v2
    [*] --> InspectSchema : cargo run --example schema_turso

    InspectSchema --> CreateTable : Auto-generated constants
    note right of InspectSchema
        #[derive(GameComponent)]
        #[db_table("player_positions")]
        #[game_ffi(skip_crud)]
        
        Generates:
        - TABLE_NAME
        - CREATE_TABLE_SQL
        - CREATE_INDEXES_SQL
    end note

    CreateTable --> ReadyDB : conn.execute(CREATE_TABLE_SQL)
    note right of CreateTable
        CREATE TABLE IF NOT EXISTS player_positions (
            id BIGINT PRIMARY KEY,
            player_id BIGINT NOT NULL,
            x REAL NOT NULL,
            y REAL NOT NULL,
            tick BIGINT NOT NULL,
            created_at BIGINT NOT NULL
        )
    end note

    ReadyDB --> ReceivePacket : WebTransport / simulated
    note right of ReadyDB
        CREATE INDEX IF NOT EXISTS
        idx_player_positions_player_id
        ON player_positions(player_id)
    end note

    ReceivePacket --> ConvertToRecord : PlayerPos (FFI packet)
    note right of ReceivePacket
        PlayerPos {
            request_uuid: Uuid,
            player_id: u64,
            x: f32, y: f32
        }
    end note

    ConvertToRecord --> InsertRow : PlayerPositionRecord::from_player_pos()
    note right of ConvertToRecord
        PlayerPositionRecord {
            id: 0, -- auto-assigned
            player_id, x, y, tick,
            created_at: now()
        }
    end note

    InsertRow --> ReceivePacket : More packets?
    note right of InsertRow
        INSERT INTO player_positions
        (player_id, x, y, tick, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
    end note

    InsertRow --> QueryAll : No more packets

    QueryAll --> QueryByPlayer : SELECT * ORDER BY id
    QueryByPlayer --> [*] : WHERE player_id = ?1

    state ReadyDB {
        [*] --> MemoryDB : ":memory:"
        MemoryDB --> DiskDB : "positions.db"
    }
```

## Data Flow Sequence

The sequence below shows the full lifecycle: Unity sends a `PlayerPos` packet, Rust converts it to a `PlayerPositionRecord`, persists to turso SQLite, then queries it back.

```mermaid
sequenceDiagram
    participant U as Unity (C#)
    participant FFI as FFI Bridge<br/>network_poll()
    participant R as Rust Handler
    participant T as turso SQLite

    Note over U,T: ── Bootstrap (auto-generated DDL) ──

    R->>T: conn.execute(PlayerPositionRecord::CREATE_TABLE_SQL)
    T-->>R: ok
    R->>T: conn.execute(PlayerPositionRecord::CREATE_INDEXES_SQL)
    T-->>R: ok

    Note over U,T: ── Per-packet cycle (repeats each tick) ──

    U->>FFI: network_send(&PlayerPos bytes)
    Note right of U: PlayerPos {<br/>player_id: 1,<br/>x: 10.0, y: 20.0<br/>}

    FFI->>R: inbound_rx.recv() → Vec<u8>
    R->>R: PlayerPos::from_bytes(&bytes)
    R->>R: PlayerPositionRecord::from_player_pos(&packet, tick)

    R->>T: INSERT INTO player_positions<br/>(player_id, x, y, tick, created_at)<br/>VALUES (?1, ?2, ?3, ?4, ?5)
    T-->>R: ok

    R->>FFI: outbound_tx.send(GameState)
    FFI->>U: network_poll() → GameState bytes

    Note over U,T: ── Query phase (on demand) ──

    R->>T: SELECT x, y, tick<br/>FROM player_positions<br/>WHERE player_id = ?1<br/>ORDER BY tick
    T-->>R: Rows

    loop rows.next() → Some(row)
        R->>R: x = get_value(0), y = get_value(1), tick = get_value(2)
    end
    Note right of R: Player 1 trail:<br/>tick 0: (10.0, 20.0)<br/>tick 2: (11.0, 21.0)<br/>tick 5: (12.0, 22.5)

    Note over T: Persistence:<br/>":memory:" → ephemeral<br/>"positions.db" → disk
```

## Packet Types

All packets use `#[repr(C)]` with `#[derive(GameComponent)]` for guaranteed memory layout matching between Rust and C#.

| Packet | Purpose | Fields |
|--------|---------|--------|
| `PacketHeader` | Common header | `packet_type`, `magic` (0xCC) |
| `PlayerPos` | Player position update | `request_uuid`, `player_id`, `x`, `y` |
| `GameState` | Server state snapshot | `tick`, `player_count`, `reserved` |
| `SpriteMessage` | Sprite CRUD operations | `operation`, `sprite_type`, `id`, `x`, `y` |

## Database Types

| Struct | Table | Purpose |
|--------|-------|---------|
| `PlayerPositionRecord` | `player_positions` | Persist player positions from `PlayerPos` packets |

Annotated with `#[db_table]` + `#[db_index]` for auto DDL generation. Uses `#[game_ffi(skip_crud)]` since CRUD queries are written manually for turso (generated CRUD targets sqlx/PostgreSQL).

## Examples

| Example | Description | Run |
|---------|-------------|-----|
| `schema_turso` | Auto-schema DDL + turso SQLite persistence | `cargo run --package unity-network --example schema_turso` |
| `extract_bindings` | Print generated C# bindings for each struct | `cargo run --package unity-network --example extract_bindings` |
| `extract_layout` | Show memory layout (offsets, sizes, padding) | `cargo run --package unity-network --example extract_layout` |
| `extract_uuids` | Print auto-generated UUID v7 values | `cargo run --package unity-network --example extract_uuids` |
| `generate_unity_cs` | Generate complete `GameFFI.cs` file | `cargo run --package unity-network --example generate_unity_cs` |

## Quick Start

### Define a DB-backed struct

```rust
use game_ffi::GameComponent;

#[derive(Debug, Clone, GameComponent)]
#[game_ffi(skip_zero_copy, skip_ffi, skip_crud)]
#[db_table("player_positions")]
pub struct PlayerPositionRecord {
    #[primary_key]
    pub id: i64,
    #[db_index(name = "idx_player_positions_player_id", on = "player_id")]
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
    pub tick: u32,
    pub created_at: i64,
}
```

### Create table and insert records

```rust
let db = turso::Builder::new_local(":memory:").build().await?;
let conn = db.connect()?;

// Auto-generated DDL — no hand-written SQL for schema
conn.execute(PlayerPositionRecord::CREATE_TABLE_SQL, ()).await?;
conn.execute(PlayerPositionRecord::CREATE_INDEXES_SQL, ()).await?;

// Insert with parameterized query
conn.execute(
    "INSERT INTO player_positions (player_id, x, y, tick, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    [turso::Value::Integer(player_id as i64), turso::Value::Real(x as f64),
     turso::Value::Real(y as f64), turso::Value::Integer(tick as i64),
     turso::Value::Integer(created_at)],
).await?;

// Query by indexed column
let mut rows = conn.query(
    "SELECT x, y, tick FROM player_positions WHERE player_id = ?1 ORDER BY tick",
    [turso::Value::Integer(target_player as i64)],
).await?;
while let Some(row) = rows.next().await? {
    let x = row.get_value(0)?.as_real().copied().unwrap_or(0.0);
    let y = row.get_value(1)?.as_real().copied().unwrap_or(0.0);
    let tick = row.get_value(2)?.as_integer().copied().unwrap_or(0);
    println!("tick {tick}: ({x:.1}, {y:.1})");
}
```

## Attribute Reference

### Struct-level

| Attribute | Purpose |
|-----------|---------|
| `#[db_table("name")]` | Auto-generate SQL DDL constants |
| `#[game_ffi(skip_crud)]` | Skip sqlx CRUD generation (use with turso) |
| `#[game_ffi(skip_zero_copy)]` | Skip `as_bytes()`/`from_bytes()` generation |
| `#[game_ffi(skip_ffi)]` | Skip `extern "C"` FFI function generation |
| `#[hash = "all"`] | Strict UUID mode (all attributes) |
| `#[hash = "name"]` | Loose UUID mode (name only) |

### Field-level

| Attribute | Purpose |
|-----------|---------|
| `#[primary_key]` | Mark as primary key column |
| `#[db_index(name = "...", on = "...")]` | Generate CREATE INDEX |
| `#[db_default("value")]` | SQL DEFAULT value |
| `#[db_column(TYPE, CONSTRAINTS)]` | Override SQL column type |

## Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `unity` | yes | Generate Unity C# bindings |
| `unreal` | yes | Generate Unreal C++ bindings |
