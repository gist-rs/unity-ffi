//! Extract detailed memory layout information for GameComponent structs
//!
//! This program shows the byte-by-byte layout of each struct including
//! field offsets, sizes, and any padding added for alignment.
//!
//! Run with: cargo run --package unity-network --example extract_layout

use std::mem;

/// Print detailed memory layout for a struct
fn print_struct_layout<T: ?Sized>(name: &str, _example: &T, total_size: usize, alignment: usize) {
    println!("{}:", name);
    println!("  Total Size: {} bytes", total_size);
    println!("  Alignment: {} bytes", alignment);
    println!();
}

fn main() {
    println!("=== Detailed Memory Layout of GameComponent Structs ===\n");

    // PacketHeader
    let header = unity_network::PacketHeader::new(unity_network::PacketType::KeepAlive);
    print_struct_layout(
        "PacketHeader",
        &header,
        mem::size_of::<unity_network::PacketHeader>(),
        mem::align_of::<unity_network::PacketHeader>(),
    );
    println!("  Layout: packet_type:u8(1) + magic:u8(1)");
    println!("  Fields:");
    println!("    packet_type (u8): 1 byte @ offset 0");
    println!("    magic      (u8): 1 byte @ offset 1");
    println!();

    // PlayerPos
    let request_uuid = uuid::Uuid::now_v7();
    let player_pos = unity_network::PlayerPos::new(request_uuid, 42, 10.5, 20.3);
    print_struct_layout(
        "PlayerPos",
        &player_pos,
        mem::size_of::<unity_network::PlayerPos>(),
        mem::align_of::<unity_network::PlayerPos>(),
    );
    println!("  Layout: packet_type:u8(1) + magic:u8(1) + request_uuid:Uuid(16) + player_id:u64(8) + x:f32(4) + y:f32(4)");
    println!("  Fields:");
    println!("    packet_type   (u8) : 1 byte  @ offset 0");
    println!("    magic         (u8) : 1 byte  @ offset 1");
    println!("    request_uuid  (Uuid): 16 bytes @ offset 2");
    println!("    player_id     (u64) : 8 bytes @ offset 18");
    println!("    x             (f32) : 4 bytes @ offset 26");
    println!("    y             (f32) : 4 bytes @ offset 30");
    println!("  ---");
    println!("  Total: 34 bytes");
    println!(
        "  Note: Rust's mem::size_of reports {} - checking for alignment...",
        mem::size_of::<unity_network::PlayerPos>()
    );
    println!();

    // GameState
    let game_state = unity_network::GameState::new(12345, 5);
    print_struct_layout(
        "GameState",
        &game_state,
        mem::size_of::<unity_network::GameState>(),
        mem::align_of::<unity_network::GameState>(),
    );
    println!("  Layout: packet_type:u8(1) + magic:u8(1) + tick:u32(4) + player_count:u32(4) + reserved:[u8;8](8)");
    println!("  Fields:");
    println!("    packet_type  (u8)        : 1 byte  @ offset 0");
    println!("    magic        (u8)        : 1 byte  @ offset 1");
    println!("    tick         (u32)       : 4 bytes @ offset 2");
    println!("    player_count (u32)       : 4 bytes @ offset 6");
    println!("    reserved     ([u8; 8])   : 8 bytes @ offset 10");
    println!("  ---");
    println!("  Total: 18 bytes");
    println!(
        "  Note: Rust's mem::size_of reports {} - checking for alignment...",
        mem::size_of::<unity_network::GameState>()
    );
    println!();

    // SpriteMessage
    let sprite_uuid = uuid::Uuid::now_v7();
    let sprite_msg = unity_network::SpriteMessage::create(
        unity_network::SpriteType::Serrif,
        sprite_uuid,
        100,
        200,
    );
    print_struct_layout(
        "SpriteMessage",
        &sprite_msg,
        mem::size_of::<unity_network::SpriteMessage>(),
        mem::align_of::<unity_network::SpriteMessage>(),
    );
    println!("  Layout: packet_type:u8(1) + magic:u8(1) + operation:u8(1) + padding1:u8(1) + sprite_type:u8(1) + padding2:[u8;3](3) + id:[u8;16](16) + x:i16(2) + y:i16(2) + padding3:[u8;2](2)");
    println!("  Fields:");
    println!("    packet_type  (u8)        : 1 byte  @ offset 0");
    println!("    magic        (u8)        : 1 byte  @ offset 1");
    println!("    operation    (u8)        : 1 byte  @ offset 2");
    println!("    padding1     (u8)        : 1 byte  @ offset 3");
    println!("    sprite_type  (u8)        : 1 byte  @ offset 4");
    println!("    padding2     ([u8; 3])   : 3 bytes @ offset 5");
    println!("    id           ([u8; 16])  : 16 bytes @ offset 8");
    println!("    x            (i16)       : 2 bytes @ offset 24");
    println!("    y            (i16)       : 2 bytes @ offset 26");
    println!("    padding3     ([u8; 2])   : 2 bytes @ offset 28");
    println!("  ---");
    println!("  Total: 30 bytes");
    println!(
        "  Note: Rust's mem::size_of reports {} - checking for alignment...",
        mem::size_of::<unity_network::SpriteMessage>()
    );
    println!();

    println!("=== Summary ===\n");
    println!(
        "PacketHeader:  {} bytes (expected: 2)",
        mem::size_of::<unity_network::PacketHeader>()
    );
    println!(
        "PlayerPos:     {} bytes (expected: 34)",
        mem::size_of::<unity_network::PlayerPos>()
    );
    println!(
        "GameState:     {} bytes (expected: 18)",
        mem::size_of::<unity_network::GameState>()
    );
    println!(
        "SpriteMessage: {} bytes (expected: 30)",
        mem::size_of::<unity_network::SpriteMessage>()
    );
    println!();
    println!("⚠️  Note: Any differences between expected and actual sizes are due to");
    println!("   Rust's alignment requirements. Use [repr(C)] for explicit control, but");
    println!("   Rust may still add padding for alignment. The actual C# bindings must");
    println!("   match these exact byte offsets for zero-copy to work correctly.");
}
