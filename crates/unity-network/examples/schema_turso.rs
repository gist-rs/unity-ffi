//! # Auto Schema with Turso SQLite Example
//!
//! Demonstrates how `#[db_table]` + `#[derive(GameComponent)]` auto-generates
//! SQL DDL that works directly with turso (libSQL) SQLite databases.
//!
//! This example simulates receiving `PlayerPos` network packets (like from
//! the `helloworld-ffi` Unity example) and records them to a turso in-memory
//! database using the auto-generated schema from `PlayerPositionRecord`.
//!
//! ## What it shows:
//! 1. Auto-generated `TABLE_NAME`, `CREATE_TABLE_SQL`, `CREATE_INDEXES_SQL`
//! 2. Creating a turso SQLite database with auto-generated DDL
//! 3. Recording player positions: `PlayerPos` (FFI packet) → `PlayerPositionRecord` (DB row)
//! 4. Querying by player_id (uses auto-generated index)
//!
//! Run with: `cargo run --package unity-network --example schema_turso`

use unity_network::{PlayerPos, PlayerPositionRecord};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Auto Schema + Turso SQLite Demo ===\n");

    // ── Step 1: Inspect auto-generated schema ───────────────────────────
    println!("Auto-generated from #[db_table(\"player_positions\", skip_crud)]:\n");
    println!("  TABLE_NAME : {}", PlayerPositionRecord::TABLE_NAME);
    println!("  Columns    : {:?}", PlayerPositionRecord::column_names());
    println!(
        "  Primary key: {:?}\n",
        PlayerPositionRecord::primary_key_field()
    );
    println!("  CREATE TABLE SQL:");
    for line in PlayerPositionRecord::CREATE_TABLE_SQL.lines() {
        println!("    {line}");
    }
    println!();
    println!("  CREATE INDEX SQL:");
    for line in PlayerPositionRecord::CREATE_INDEXES_SQL.lines() {
        println!("    {line}");
    }
    println!();

    // ── Step 2: Create turso in-memory database ─────────────────────────
    let db = turso::Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;
    println!("[ok] Turso in-memory database created");

    // ── Step 3: Execute auto-generated DDL ──────────────────────────────
    conn.execute(PlayerPositionRecord::CREATE_TABLE_SQL, ())
        .await?;
    conn.execute(PlayerPositionRecord::CREATE_INDEXES_SQL, ())
        .await?;
    println!("[ok] Table + index created using auto-generated DDL\n");

    // ── Step 4: Simulate PlayerPos packets → DB records ─────────────────
    // These represent positions arriving from Unity via WebTransport,
    // just like the helloworld-ffi example sends PlayerPos packets.
    let simulated_packets: [(u64, f32, f32); 6] = [
        (1, 10.0, 20.0), // Player 1 spawns at (10, 20)
        (2, 30.5, 40.5), // Player 2 spawns at (30.5, 40.5)
        (1, 11.0, 21.0), // Player 1 moves
        (3, 50.0, 60.0), // Player 3 spawns
        (2, 31.5, 42.0), // Player 2 moves
        (1, 12.0, 22.5), // Player 1 moves again
    ];

    for (tick, &(player_id, x, y)) in simulated_packets.iter().enumerate() {
        // Create a PlayerPos FFI packet (same struct helloworld-ffi uses)
        let request_uuid = uuid::Uuid::now_v7();
        let packet = PlayerPos::new(request_uuid, player_id, x, y);

        // Convert FFI packet → DB record using the auto-annotated struct
        let record = PlayerPositionRecord::from_player_pos(&packet, tick as u32);

        // Insert with SQLite parameterized query (?1..?5 style)
        conn.execute(
            "INSERT INTO player_positions (player_id, x, y, tick, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                turso::Value::Integer(record.player_id as i64),
                turso::Value::Real(record.x as f64),
                turso::Value::Real(record.y as f64),
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

    // ── Step 5: Query all positions ─────────────────────────────────────
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

    // ── Step 6: Query by player_id (uses auto-generated index) ──────────
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

    // ── Step 7: Summary ─────────────────────────────────────────────────
    println!("\n=== Summary ===");
    println!("[ok] #[db_table] -> auto-generated DDL works with turso SQLite");
    println!("[ok] PlayerPos (FFI packet) -> PlayerPositionRecord (DB row)");
    println!("[ok] #[db_index] -> auto-generated index for fast player lookups");
    println!("[ok] Parameterized queries (?1, ?2, ...) prevent SQL injection");
    println!();
    println!("Tip: Try persisting to disk: change \":memory:\" to \"positions.db\"");

    Ok(())
}
