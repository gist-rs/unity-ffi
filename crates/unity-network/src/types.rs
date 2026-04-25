// Shared types between Rust and C# Unity
// Must match exactly in both languages (same bit layout)

use game_ffi::GameComponent;

/// Discriminator for packet types
/// Every packet MUST start with one of these
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum PacketType {
    KeepAlive = 0,
    PlayerPos = 1,
    GameState = 2,
    SpriteMessage = 3,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PacketType::KeepAlive),
            1 => Some(PacketType::PlayerPos),
            2 => Some(PacketType::GameState),
            3 => Some(PacketType::SpriteMessage),
            _ => None,
        }
    }
}

/// Common header for all packets
/// Matches C#: [StructLayout(LayoutKind.Sequential, Pack = 1)]
#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy)]
pub struct PacketHeader {
    pub packet_type: u8,
    pub magic: u8, // 0xCC for sanity check
}

impl PacketHeader {
    pub const MAGIC: u8 = 0xCC;

    pub fn new(packet_type: PacketType) -> Self {
        Self {
            packet_type: packet_type as u8,
            magic: Self::MAGIC,
        }
    }
}

/// Player position update packet
/// Matches C#: [StructLayout(LayoutKind.Sequential, Pack = 1)]
/// Auto-generates: PlayerPos::UUID, PlayerPos::size(), PlayerPos::as_bytes(), PlayerPos::from_bytes()
#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy)]
pub struct PlayerPos {
    pub packet_type: u8,
    pub magic: u8,
    pub request_uuid: uuid::Uuid,
    pub player_id: u64,
    pub x: f32,
    pub y: f32,
}

impl PlayerPos {
    pub fn new(request_uuid: uuid::Uuid, player_id: u64, x: f32, y: f32) -> Self {
        Self {
            packet_type: PacketType::PlayerPos as u8,
            magic: PacketHeader::MAGIC,
            request_uuid,
            player_id,
            x,
            y,
        }
    }
}

/// Database row for recording player positions (auto schema demo).
///
/// Uses `#[db_table]` to auto-generate SQL DDL via `GameComponent` derive.
/// The generated `CREATE_TABLE_SQL` and `TABLE_NAME` constants are used
/// by the `schema_turso` example to create and query a turso SQLite database.
///
/// Related FFI struct: [`PlayerPos`] (network packet → this DB row for persistence)
///
/// ```rust
/// use unity_network::PlayerPositionRecord;
///
/// // Auto-generated constants from #[db_table("player_positions")]
/// assert_eq!(PlayerPositionRecord::TABLE_NAME, "player_positions");
/// assert!(PlayerPositionRecord::CREATE_TABLE_SQL.contains("player_positions"));
/// assert!(PlayerPositionRecord::CREATE_TABLE_SQL.contains("player_id"));
/// assert!(PlayerPositionRecord::CREATE_TABLE_SQL.contains("x"));
/// assert!(PlayerPositionRecord::CREATE_TABLE_SQL.contains("y"));
/// ```
#[derive(Debug, Clone, GameComponent)]
#[game_ffi(skip_zero_copy, skip_ffi, skip_crud)]
#[db_table("player_positions")]
pub struct PlayerPositionRecord {
    /// Auto-incrementing row ID (SQLite INTEGER PRIMARY KEY)
    #[primary_key]
    pub id: i64,
    /// Player ID from the network packet, indexed for fast lookup
    #[db_index(name = "idx_player_positions_player_id", on = "player_id")]
    pub player_id: u64,
    /// X coordinate from `PlayerPos::x`
    pub x: f32,
    /// Y coordinate from `PlayerPos::y`
    pub y: f32,
    /// Server tick when position was recorded
    pub tick: u32,
    /// Unix timestamp (seconds) when position was recorded
    pub created_at: i64,
}

impl PlayerPositionRecord {
    /// Create a new record from a `PlayerPos` network packet.
    pub fn from_player_pos(pos: &PlayerPos, tick: u32) -> Self {
        Self {
            id: 0, // SQLite auto-assigns for INTEGER PRIMARY KEY
            player_id: pos.player_id,
            x: pos.x,
            y: pos.y,
            tick,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
        }
    }
}

/// Game state snapshot packet
/// Matches C#: [StructLayout(LayoutKind.Sequential, Pack = 1)]
/// This is a flexible packet type that can represent different state messages
/// Use the `GameStateType` constants to interpret fields appropriately
/// Auto-generates: GameState::UUID, GameState::size(), GameState::as_bytes(), GameState::from_bytes()
#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy)]
pub struct GameState {
    pub packet_type: u8,
    pub magic: u8,
    pub tick: u32,         // Server tick or timestamp
    pub player_count: u32, // Number of players or message type
    pub reserved: [u8; 8], // Additional data for extensibility
}

/// Constants for interpreting GameState packet fields
/// These allow the same GameState struct to serve multiple purposes
pub mod game_state {
    /// Special player_count values indicating the message type
    pub const MSG_TYPE_HELLO: u32 = 0xFFFF0000; // Hello world / connection test
    pub const MSG_TYPE_ECHO: u32 = 0xFFFF0001; // Echo response
    pub const MSG_TYPE_STATE: u32 = 0xFFFF0002; // Standard game state

    /// Reserved field indices for additional data
    pub const RESERVED_SIZE: usize = 8;

    /// Helper to check if player_count is a message type
    pub fn is_message_type(value: u32) -> bool {
        (value & 0xFFFF0000) != 0
    }
}

impl GameState {
    pub fn new(tick: u32, player_count: u32) -> Self {
        Self {
            packet_type: PacketType::GameState as u8,
            magic: PacketHeader::MAGIC,
            tick,
            player_count,
            reserved: [0; 8],
        }
    }

    /// Create a hello message for round-trip testing
    pub fn hello() -> Self {
        Self {
            packet_type: PacketType::GameState as u8,
            magic: PacketHeader::MAGIC,
            tick: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            player_count: game_state::MSG_TYPE_HELLO,
            reserved: [0; 8],
        }
    }

    /// Create an echo response from a received hello
    pub fn echo_response(_original_tick: u32) -> Self {
        Self {
            packet_type: PacketType::GameState as u8,
            magic: PacketHeader::MAGIC,
            tick: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32,
            player_count: game_state::MSG_TYPE_ECHO,
            reserved: [0; 8],
        }
    }

    /// Create a standard game state update
    pub fn state_update(tick: u32, player_count: u32) -> Self {
        Self {
            packet_type: PacketType::GameState as u8,
            magic: PacketHeader::MAGIC,
            tick,
            player_count,
            reserved: [0; 8],
        }
    }

    /// Check if this is a hello message
    pub fn is_hello(&self) -> bool {
        self.player_count == game_state::MSG_TYPE_HELLO
    }

    /// Check if this is an echo response
    pub fn is_echo(&self) -> bool {
        self.player_count == game_state::MSG_TYPE_ECHO
    }

    /// Get message type description for debugging
    pub fn get_type_description(&self) -> &'static str {
        match self.player_count {
            game_state::MSG_TYPE_HELLO => "Hello",
            game_state::MSG_TYPE_ECHO => "EchoResponse",
            game_state::MSG_TYPE_STATE => "StateUpdate",
            _ if game_state::is_message_type(self.player_count) => "UnknownMessage",
            _ => "PlayerCount",
        }
    }
}

/// Position within 128x128 pixel map (0-127 range)
pub type SpritePosition = (i16, i16);

/// Individual sprite data (server-side only, not sent over network)
#[derive(Clone, Debug)]
pub struct SpriteData {
    pub id: uuid::Uuid,
    pub sprite_type: SpriteType,
    pub position: SpritePosition,
    pub spawn_time: std::time::Instant,
    pub lifetime: std::time::Duration,
}

/// Sprite operation types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SpriteOp {
    Create = 0,
    Update = 1,
    Delete = 2,
    Snapshot = 3,
}

impl SpriteOp {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SpriteOp::Create),
            1 => Some(SpriteOp::Update),
            2 => Some(SpriteOp::Delete),
            3 => Some(SpriteOp::Snapshot),
            _ => None,
        }
    }
}

/// Sprite type enum (extensible for future types)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SpriteType {
    Serrif = 0,
}

impl SpriteType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(SpriteType::Serrif),
            _ => None,
        }
    }
}

/// Sprite message packet (zero-copy, repr(C))
/// Layout: packet_type(1) + magic(1) + operation(1) + padding1(1) + sprite_type(1) + padding2(3) + id(16) + x(2) + y(2) + padding3(2) = 30 bytes
/// Matches C#: [StructLayout(LayoutKind.Sequential, Pack = 1)]
/// Auto-generates: SpriteMessage::UUID, SpriteMessage::size(), SpriteMessage::as_bytes(), SpriteMessage::from_bytes()
#[repr(C)]
#[derive(GameComponent, Debug, Clone, Copy)]
pub struct SpriteMessage {
    pub packet_type: u8,
    pub magic: u8,
    pub operation: u8,
    padding1: u8,
    pub sprite_type: u8,
    padding2: [u8; 3],
    pub id: [u8; 16], // UUID as 16 bytes
    pub x: i16,
    pub y: i16,
    padding3: [u8; 2],
}

impl SpriteMessage {
    pub fn new(
        operation: SpriteOp,
        sprite_type: SpriteType,
        id: uuid::Uuid,
        x: i16,
        y: i16,
    ) -> Self {
        Self {
            packet_type: PacketType::SpriteMessage as u8,
            magic: PacketHeader::MAGIC,
            operation: operation as u8,
            padding1: 0,
            sprite_type: sprite_type as u8,
            padding2: [0; 3],
            id: *id.as_bytes(),
            x,
            y,
            padding3: [0; 2],
        }
    }

    pub fn create(sprite_type: SpriteType, id: uuid::Uuid, x: i16, y: i16) -> Self {
        Self::new(SpriteOp::Create, sprite_type, id, x, y)
    }

    pub fn update(id: uuid::Uuid, x: i16, y: i16) -> Self {
        Self::new(SpriteOp::Update, SpriteType::Serrif, id, x, y)
    }

    pub fn delete(id: uuid::Uuid) -> Self {
        Self::new(SpriteOp::Delete, SpriteType::Serrif, id, 0, 0)
    }

    pub fn snapshot() -> Self {
        Self::new(
            SpriteOp::Snapshot,
            SpriteType::Serrif,
            uuid::Uuid::nil(),
            0,
            0,
        )
    }

    pub fn get_id(&self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.id)
    }

    pub fn get_operation(&self) -> Option<SpriteOp> {
        SpriteOp::from_u8(self.operation)
    }

    pub fn get_sprite_type(&self) -> Option<SpriteType> {
        SpriteType::from_u8(self.sprite_type)
    }
}

/// Error codes returned to Unity
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiError {
    Success = 0,
    InvalidPointer = -1,
    InvalidMagic = -2,
    UnknownPacketType = -3,
    BufferTooSmall = -4,
    Disconnected = -5,
    AlreadyConnected = -6,
    InvalidUrl = -7,
    CertValidationFailed = -8,
    PanicCaught = -99,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_header_magic() {
        let header = PacketHeader::new(PacketType::PlayerPos);
        assert_eq!(header.magic, PacketHeader::MAGIC);
    }

    #[test]
    fn test_player_pos_validation() {
        let request_uuid = uuid::Uuid::now_v7();
        let pos = PlayerPos::new(request_uuid, 42, 10.5, 20.3);
        assert_eq!(pos.player_id, 42);
        assert_eq!(pos.request_uuid, request_uuid);
        assert_eq!(pos.x, 10.5);
        assert_eq!(pos.y, 20.3);
    }

    #[test]
    fn test_game_state_hello() {
        let hello = GameState::hello();
        assert!(hello.is_hello());
        assert!(!hello.is_echo());
        assert_eq!(hello.get_type_description(), "Hello");
    }

    #[test]
    fn test_game_state_echo() {
        let echo = GameState::echo_response(12345);
        assert!(echo.is_echo());
        assert!(!echo.is_hello());
        assert_eq!(echo.get_type_description(), "EchoResponse");
    }

    #[test]
    fn test_game_state_update() {
        let state = GameState::state_update(100, 5);
        assert!(!state.is_hello());
        assert!(!state.is_echo());
        assert_eq!(state.tick, 100);
        assert_eq!(state.player_count, 5);
        assert_eq!(state.get_type_description(), "PlayerCount");
    }
}
