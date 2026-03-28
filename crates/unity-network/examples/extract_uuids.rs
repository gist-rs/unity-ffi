//! Extract actual UUIDs from GameComponent structs
//!
//! This program prints the auto-generated UUID v7 values from the
//! GameComponent derive macro, along with struct sizes and validation info.
//!
//! Run with: cargo run --package unity-network --example extract_uuids

use std::mem;
use unity_network::{GameState, PacketHeader, PlayerPos, SpriteMessage};

fn main() {
    println!("=== Extracting UUIDs from GameComponent Structs ===\n");

    // PacketHeader
    println!("1. PacketHeader:");
    println!("   UUID: {}", PacketHeader::UUID);
    println!("   Size: {} bytes", mem::size_of::<PacketHeader>());
    println!("   Alignment: {} bytes", mem::align_of::<PacketHeader>());
    println!();

    // PlayerPos
    println!("2. PlayerPos:");
    println!("   UUID: {}", PlayerPos::UUID);
    println!("   Size: {} bytes", mem::size_of::<PlayerPos>());
    println!("   Alignment: {} bytes", mem::align_of::<PlayerPos>());
    println!();

    // GameState
    println!("3. GameState:");
    println!("   UUID: {}", GameState::UUID);
    println!("   Size: {} bytes", mem::size_of::<GameState>());
    println!("   Alignment: {} bytes", mem::align_of::<GameState>());
    println!();

    // SpriteMessage
    println!("4. SpriteMessage:");
    println!("   UUID: {}", SpriteMessage::UUID);
    println!("   Size: {} bytes", mem::size_of::<SpriteMessage>());
    println!("   Alignment: {} bytes", mem::align_of::<SpriteMessage>());
    println!();

    println!("=== Copy these UUIDs to unity/Generated/GameFFI.cs ===\n");

    // Verify UUID versions
    println!("=== UUID Verification ===\n");
    verify_uuid("PacketHeader", PacketHeader::UUID);
    verify_uuid("PlayerPos", PlayerPos::UUID);
    verify_uuid("GameState", GameState::UUID);
    verify_uuid("SpriteMessage", SpriteMessage::UUID);
}

fn verify_uuid(struct_name: &str, uuid_str: &str) {
    match uuid::Uuid::parse_str(uuid_str) {
        Ok(uuid) => {
            println!("{struct_name}:");
            println!("  ✓ Valid UUID");
            println!("  Version: {:?}", uuid.get_version());
            println!("  Variant: {:?}", uuid.get_variant());
        }
        Err(e) => {
            println!("{struct_name}: ✗ Invalid UUID: {e}");
        }
    }
    println!();
}
