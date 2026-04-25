//! # Auto Schema + Turso SQLite with Position2D Composition
//!
//! Demonstrates `#[db_flatten]` — the single source of truth pattern where
//! `Position2D` is shared between the FFI packet (`PlayerPos`) and the DB row
//! (`PlayerPositionRecord`) with zero field duplication.
//!
//! ## What it shows:
//! 1. `Position2D` — shared payload with own `#[db_table]` schema
//! 2. `PlayerPos` — FFI packet embedding `Position2D` (flat memory via nested `#[repr(C)]`)
//! 3. `PlayerPositionRecord` — DB row with `#[db_flatten]` expanding `Position2D` columns
//! 4. `create_table_sql()` — runtime SQL composition (const can't concat flattened columns)
//!
//! Run with: `cargo run --package unity-network --example schema_turso`

use unity_network::{PlayerPos, PlayerPositionRecord, Position2D};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Auto Schema + Turso SQLite with Position2D Composition ===\n");

    // ── Step 1: Inspect Position2D (shared payload) ────────────────────
    println!("Shared payload (single source of truth):");
    println!("  TABLE_NAME  : {}", Position2D::TABLE_NAME);
    println!("  Columns     : {:?}", Position2D::column_names());
    println!("  CREATE TABLE:");
    for line in Position2D::CREATE_TABLE_SQL.lines() {
        println!("    {line}");
    }
    println!();

    // ── Step 2: Inspect PlayerPositionRecord (DB row with flatten) ─────
    println!("DB record (with #[db_flatten] on pos: Position2D):");
    println!("  TABLE_NAME  : {}", PlayerPositionRecord::TABLE_NAME);
    println!("  Own columns : {:?}", PlayerPositionRecord::column_names());
    println!(
        "  Primary key : {:?}",
        PlayerPositionRecord::primary_key_field()
    );
    println!();

    println!("  Composed CREATE TABLE (runtime via create_table_sql()):");
    let create_sql = PlayerPositionRecord::create_table_sql();
    for line in create_sql.lines() {
        println!("    {line}");
    }
    println!();

    if !PlayerPositionRecord::CREATE_INDEXES_SQL.is_empty() {
        println!("  CREATE INDEX:");
        for line in PlayerPositionRecord::CREATE_INDEXES_SQL.lines() {
            println!("    {line}");
        }
        println!();
    }

    // ── Step 3: Create turso in-memory database ─────────────────────────
    let db = turso::Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    println!("[ok] Turso in-memory database created");

    // ── Step 4: Execute composed DDL ────────────────────────────────────
    conn.execute(PlayerPositionRecord::create_table_sql(), ())
        .await?;
    conn.execute(PlayerPositionRecord::CREATE_INDEXES_SQL, ())
        .await?;
    println!("[ok] Table + index created using composed DDL\n");

    // ── Step 5: Simulate PlayerPos packets → DB records ─────────────────
    // Each packet embeds Position2D — converting to DB row is just `packet.pos`
    let simulated_packets: [(u64, f32, f32); 6] = [
        (1, 10.0, 20.0), // Player 1 spawns
        (2, 30.5, 40.5), // Player 2 spawns
        (1, 11.0, 21.0), // Player 1 moves
        (3, 50.0, 60.0), // Player 3 spawns
        (2, 31.5, 42.0), // Player 2 moves
        (1, 12.0, 22.5), // Player 1 moves again
    ];

    for (tick, &(player_id, x, y)) in simulated_packets.iter().enumerate() {
        // FFI packet: header + Position2D payload
        let request_uuid = uuid::Uuid::now_v7();
        let packet = PlayerPos::new(request_uuid, player_id, x, y);

        // DB record: metadata + same Position2D payload (just `packet.pos`)
        let record = PlayerPositionRecord::from_player_pos(&packet, tick as u32);

        // INSERT uses flattened column names (player_id, x, y from Position2D)
        conn.execute(
            "INSERT INTO player_positions (player_id, x, y, tick, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                turso::Value::Integer(record.pos.player_id as i64),
                turso::Value::Real(record.pos.x as f64),
                turso::Value::Real(record.pos.y as f64),
                turso::Value::Integer(record.tick as i64),
                turso::Value::Integer(record.created_at),
            ],
        )
        .await?;
    }
    println!(
        "[ok] Inserted {} player position records\n",
        simulated_packets.len()
    );

    // ── Step 6: Query all positions ─────────────────────────────────────
    println!("All Player Positions (ordered by id):");
    println!(
        "  {:<5} {:<10} {:<10} {:<10} {:<8} {:<12}",
        "id", "player_id", "x", "y", "tick", "created_at"
    );
    println!("  {}", "-".repeat(60));

    let mut rows = conn
        .query(
            "SELECT id, player_id, x, y, tick, created_at \
             FROM player_positions ORDER BY id",
            (),
        )
        .await?;

    let mut count = 0u32;
    while let Some(row) = rows.next().await? {
        let id = row.get_value(0)?.as_integer().copied().unwrap_or(0);
        let player_id = row.get_value(1)?.as_integer().copied().unwrap_or(0);
        let x = row.get_value(2)?.as_real().copied().unwrap_or(0.0);
        let y = row.get_value(3)?.as_real().copied().unwrap_or(0.0);
        let tick = row.get_value(4)?.as_integer().copied().unwrap_or(0);
        let created_at = row.get_value(5)?.as_integer().copied().unwrap_or(0);
        println!(
            "  {:<5} {:<10} {:<10.1} {:<10.1} {:<8} {:<12}",
            id, player_id, x, y, tick, created_at
        );
        count += 1;
    }
    println!("  Total: {count} records\n");

    // ── Step 7: Query by player_id (uses auto-generated index) ──────────
    let target_player: u64 = 1;
    println!("Player {target_player} trail (uses idx_player_positions_player_id):");

    let mut rows = conn
        .query(
            "SELECT x, y, tick FROM player_positions \
             WHERE player_id = ?1 ORDER BY tick",
            [turso::Value::Integer(target_player as i64)],
        )
        .await?;

    while let Some(row) = rows.next().await? {
        let x = row.get_value(0)?.as_real().copied().unwrap_or(0.0);
        let y = row.get_value(1)?.as_real().copied().unwrap_or(0.0);
        let tick = row.get_value(2)?.as_integer().copied().unwrap_or(0);
        println!("  tick {tick}: ({x:.1}, {y:.1})");
    }

    // ── Step 8: Summary ─────────────────────────────────────────────────
    println!("\n=== Summary ===");
    println!("[ok] Position2D — single source of truth for player_id, x, y");
    println!("[ok] PlayerPos — FFI packet with embedded Position2D (flat memory)");
    println!("[ok] PlayerPositionRecord — DB row with #[db_flatten] expanding Position2D");
    println!("[ok] create_table_sql() — runtime composition with Position2D::COLUMN_DEFS_SQL");
    println!("[ok] Adding z/rotation/velocity to Position2D auto-propagates everywhere");
    println!();
    println!("Tip: Try persisting to disk: change \":memory:\" to \"positions.db\"");

    Ok(())
}
