use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PacketHeader {
    pub packet_type: u8,
    pub magic: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PlayerPos {
    pub header: PacketHeader,
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct GameState {
    pub header: PacketHeader,
    pub tick: u32,
    pub player_count: u32,
    pub reserved: [u8; 8],
}

fn main() {
    println!("PacketHeader size: {}", mem::size_of::<PacketHeader>());
    println!("PlayerPos size: {}", mem::size_of::<PlayerPos>());
    println!("GameState size: {}", mem::size_of::<GameState>());
}
