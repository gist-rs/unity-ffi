// GameFFITests.cs
//
// Unity Test Runner tests for generated GameFFI C# bindings
// These tests verify that the manually maintained GameFFI.cs matches
// Rust's memory layout exactly and works correctly in Unity runtime.
//
// See Issue 007: Unity Test Strategy for Generated C# Code
//
// Purpose:
// - Verify memory layout matches Rust (sizes, field offsets, padding)
// - Validate type UUIDs are v7 and unique
// - Test zero-copy serialization/deserialization
// - Verify validation and helper methods work correctly
//
// Run Tests:
// - Unity Editor: Window > General > Test Runner > EditMode > Run All
// - Command Line: Unity -runTests -batchmode -testPlatform EditMode -projectPath [path]

using NUnit.Framework;
using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;

namespace GameFFI
{
    /// <summary>
    /// Comprehensive tests for all GameFFI generated types
    /// </summary>
    public class GameFFITests
    {
        #region Helper Methods

        /// <summary>
        /// Get the memory offset of a field in a struct
        /// </summary>
        private static int GetFieldOffset<T>(string fieldName)
        {
            try
            {
                IntPtr offset = Marshal.OffsetOf<T>(fieldName);
                return offset.ToInt32();
            }
            catch (Exception ex)
            {
                throw new Exception($"Failed to get offset for field '{fieldName}' in type '{typeof(T).Name}': {ex.Message}");
            }
        }

        /// <summary>
        /// Get the total size of a struct
        /// </summary>
        private static int GetStructSize<T>()
        {
            return Marshal.SizeOf<T>();
        }

        /// <summary>
        /// Check if a UUID is valid v7
        /// </summary>
        private static bool IsUUIDv7(string uuidStr)
        {
            if (!Guid.TryParse(uuidStr, out Guid uuid))
            {
                return false;
            }

            // Version 7 UUIDs have a specific format
            // Version is in the high nibble of the 7th byte (index 6 in the string representation)
            byte[] bytes = uuid.ToByteArray();
            byte version = (byte)((bytes[6] >> 4) & 0x0F);

            // Variant is in the high nibble of the 9th byte (index 8 in the string representation)
            // For UUIDs, variant should be 0b10xx (2, 6, or 7)
            byte variant = (byte)((bytes[8] >> 6) & 0x03);

            return version == 7 && variant == 2;
        }

        #endregion

        #region PacketHeader Tests

        [TestFixture]
        public class PacketHeaderTests
        {
            [Test]
            public void PacketHeader_Size_MatchesRust()
            {
                int expected = 2; // Rust: mem::size_of::<PacketHeader>()
                int actual = GetStructSize<PacketHeader>();
                Assert.AreEqual(expected, actual,
                    $"PacketHeader size mismatch: expected {expected} bytes, got {actual} bytes");
            }

            [Test]
            public void PacketHeader_FieldOffsets_MatchRust()
            {
                // Rust offsets: packet_type @ 0, magic @ 1
                Assert.AreEqual(0, GetFieldOffset<PacketHeader>("packet_type"),
                    "packet_type offset must be 0");
                Assert.AreEqual(1, GetFieldOffset<PacketHeader>("magic"),
                    "magic offset must be 1");
            }

            [Test]
            public void PacketHeader_UUID_IsValidV7()
            {
                Assert.IsTrue(IsUUIDv7(PacketHeader.UUID.ToString()),
                    $"PacketHeader UUID {PacketHeader.UUID} must be v7");
            }

            [Test]
            public void PacketHeader_Magic_Constant()
            {
                Assert.AreEqual(0xCC, PacketHeader.MAGIC,
                    "PacketHeader magic constant must be 0xCC");
            }

            [Test]
            public void PacketHeader_Validate_Works()
            {
                var valid = new PacketHeader
                {
                    packet_type = (byte)PacketType.PlayerPos,
                    magic = PacketHeader.MAGIC
                };
                Assert.IsTrue(valid.validate(), "Valid packet header should pass validation");

                var invalidMagic = new PacketHeader
                {
                    packet_type = (byte)PacketType.PlayerPos,
                    magic = 0x00
                };
                Assert.IsFalse(invalidMagic.validate(), "Invalid magic should fail validation");
            }

            [Test]
            public void PacketHeader_ZeroCopy_Roundtrip_PreservesData()
            {
                var original = new PacketHeader
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC
                };

                byte[] bytes = original.AsBytes();
                var recovered = PacketHeader.FromBytes(bytes);

                Assert.AreEqual(original.packet_type, recovered.packet_type,
                    "packet_type should be preserved after roundtrip");
                Assert.AreEqual(original.magic, recovered.magic,
                    "magic should be preserved after roundtrip");
            }

            [Test]
            public void PacketHeader_GetPacketType_Works()
            {
                var header = new PacketHeader
                {
                    packet_type = (byte)PacketType.SpriteMessage,
                    magic = PacketHeader.MAGIC
                };

                PacketType type = header.GetPacketType();
                Assert.AreEqual(PacketType.SpriteMessage, type,
                    "GetPacketType should return correct enum value");
            }
        }

        #endregion

        #region PlayerPos Tests

        [TestFixture]
        public class PlayerPosTests
        {
            [Test]
            public void PlayerPos_Size_MatchesRust()
            {
                // Rust reports 40 bytes (34 data + 6 padding for alignment)
                int expected = 40;
                int actual = GetStructSize<PlayerPos>();
                Assert.AreEqual(expected, actual,
                    $"PlayerPos size mismatch: expected {expected} bytes, got {actual} bytes");
            }

            [Test]
            public void PlayerPos_FieldOffsets_MatchRust()
            {
                // Rust offsets:
                // packet_type:u8 @ 0
                // magic:u8 @ 1
                // request_uuid:Uuid @ 2 (16 bytes)
                // player_id:u64 @ 18
                // x:f32 @ 26
                // y:f32 @ 30
                // padding:6 bytes @ 34-39

                Assert.AreEqual(0, GetFieldOffset<PlayerPos>("packet_type"),
                    "packet_type offset must be 0");
                Assert.AreEqual(1, GetFieldOffset<PlayerPos>("magic"),
                    "magic offset must be 1");
                Assert.AreEqual(2, GetFieldOffset<PlayerPos>("request_uuid"),
                    "request_uuid offset must be 2");
                Assert.AreEqual(18, GetFieldOffset<PlayerPos>("player_id"),
                    "player_id offset must be 18");
                Assert.AreEqual(26, GetFieldOffset<PlayerPos>("x"),
                    "x offset must be 26");
                Assert.AreEqual(30, GetFieldOffset<PlayerPos>("y"),
                    "y offset must be 30");
            }

            [Test]
            public void PlayerPos_UUID_IsValidV7()
            {
                Assert.IsTrue(IsUUIDv7(PlayerPos.UUID.ToString()),
                    $"PlayerPos UUID {PlayerPos.UUID} must be v7");
            }

            [Test]
            public void PlayerPos_Size_Constant_MatchesActual()
            {
                int expectedSize = GetStructSize<PlayerPos>();
                Assert.AreEqual(expectedSize, PlayerPos.Size,
                    $"PlayerPos.Size constant {PlayerPos.Size} must match actual size {expectedSize}");
            }

            [Test]
            public void PlayerPos_Validate_Works()
            {
                var valid = new PlayerPos
                {
                    packet_type = (byte)PacketType.PlayerPos,
                    magic = PacketHeader.MAGIC,
                    request_uuid = Guid.NewGuid(),
                    player_id = 12345,
                    x = 10.5f,
                    y = 20.7f
                };
                Assert.IsTrue(valid.validate(), "Valid PlayerPos should pass validation");

                var invalidType = new PlayerPos
                {
                    packet_type = (byte)PacketType.KeepAlive, // Wrong type
                    magic = PacketHeader.MAGIC,
                    request_uuid = Guid.NewGuid(),
                    player_id = 12345,
                    x = 10.5f,
                    y = 20.7f
                };
                Assert.IsFalse(invalidType.validate(), "Invalid packet_type should fail validation");

                var invalidMagic = new PlayerPos
                {
                    packet_type = (byte)PacketType.PlayerPos,
                    magic = 0x00, // Wrong magic
                    request_uuid = Guid.NewGuid(),
                    player_id = 12345,
                    x = 10.5f,
                    y = 20.7f
                };
                Assert.IsFalse(invalidMagic.validate(), "Invalid magic should fail validation");
            }

            [Test]
            public void PlayerPos_ZeroCopy_Roundtrip_PreservesData()
            {
                var uuid = Guid.NewGuid();
                var original = new PlayerPos
                {
                    packet_type = (byte)PacketType.PlayerPos,
                    magic = PacketHeader.MAGIC,
                    request_uuid = uuid,
                    player_id = 999887766,
                    x = 123.456f,
                    y = 789.012f
                };

                byte[] bytes = original.AsBytes();
                var recovered = PlayerPos.FromBytes(bytes);

                Assert.AreEqual(original.packet_type, recovered.packet_type);
                Assert.AreEqual(original.magic, recovered.magic);
                Assert.AreEqual(original.request_uuid, recovered.request_uuid);
                Assert.AreEqual(original.player_id, recovered.player_id);
                Assert.AreEqual(original.x, recovered.x, 0.001f);
                Assert.AreEqual(original.y, recovered.y, 0.001f);
            }

            [Test]
            public void PlayerPos_ToString_Works()
            {
                var uuid = Guid.NewGuid();
                var packet = new PlayerPos
                {
                    packet_type = (byte)PacketType.PlayerPos,
                    magic = PacketHeader.MAGIC,
                    request_uuid = uuid,
                    player_id = 42,
                    x = 1.0f,
                    y = 2.0f
                };

                string str = packet.ToString();
                Assert.IsTrue(str.Contains("player_id=42"),
                    $"ToString should contain player_id: {str}");
                Assert.IsTrue(str.Contains($"uuid={uuid}"),
                    $"ToString should contain uuid: {str}");
            }

            [Test]
            public void PlayerPos_FromBytes_ThrowsOnTooSmall()
            {
                byte[] tooSmall = new byte[10];
                Assert.Throws<ArgumentException>(() => PlayerPos.FromBytes(tooSmall),
                    "FromBytes should throw ArgumentException for data smaller than struct size");
            }
        }

        #endregion

        #region GameState Tests

        [TestFixture]
        public class GameStateTests
        {
            [Test]
            public void GameState_Size_MatchesRust()
            {
                // Rust reports 20 bytes (18 data + 2 padding for alignment)
                int expected = 20;
                int actual = GetStructSize<GameState>();
                Assert.AreEqual(expected, actual,
                    $"GameState size mismatch: expected {expected} bytes, got {actual} bytes");
            }

            [Test]
            public void GameState_FieldOffsets_MatchRust()
            {
                // Rust offsets:
                // packet_type:u8 @ 0
                // magic:u8 @ 1
                // tick:u32 @ 2
                // player_count:u32 @ 6
                // reserved:[u8;8] @ 10
                // padding:2 bytes @ 18-19

                Assert.AreEqual(0, GetFieldOffset<GameState>("packet_type"),
                    "packet_type offset must be 0");
                Assert.AreEqual(1, GetFieldOffset<GameState>("magic"),
                    "magic offset must be 1");
                Assert.AreEqual(2, GetFieldOffset<GameState>("tick"),
                    "tick offset must be 2");
                Assert.AreEqual(6, GetFieldOffset<GameState>("player_count"),
                    "player_count offset must be 6");
                Assert.AreEqual(10, GetFieldOffset<GameState>("reserved"),
                    "reserved offset must be 10");
            }

            [Test]
            public void GameState_UUID_IsValidV7()
            {
                Assert.IsTrue(IsUUIDv7(GameState.UUID.ToString()),
                    $"GameState UUID {GameState.UUID} must be v7");
            }

            [Test]
            public void GameState_Size_Constant_MatchesActual()
            {
                int expectedSize = GetStructSize<GameState>();
                Assert.AreEqual(expectedSize, GameState.Size,
                    $"GameState.Size constant {GameState.Size} must match actual size {expectedSize}");
            }

            [Test]
            public void GameState_Validate_Works()
            {
                var valid = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = 5,
                    reserved = new byte[8]
                };
                Assert.IsTrue(valid.validate(), "Valid GameState should pass validation");

                var invalidType = new GameState
                {
                    packet_type = (byte)PacketType.PlayerPos, // Wrong type
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = 5,
                    reserved = new byte[8]
                };
                Assert.IsFalse(invalidType.validate(), "Invalid packet_type should fail validation");
            }

            [Test]
            public void GameState_ZeroCopy_Roundtrip_PreservesData()
            {
                var original = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 99999,
                    player_count = 42,
                    reserved = new byte[] { 1, 2, 3, 4, 5, 6, 7, 8 }
                };

                byte[] bytes = original.AsBytes();
                var recovered = GameState.FromBytes(bytes);

                Assert.AreEqual(original.packet_type, recovered.packet_type);
                Assert.AreEqual(original.magic, recovered.magic);
                Assert.AreEqual(original.tick, recovered.tick);
                Assert.AreEqual(original.player_count, recovered.player_count);
                CollectionAssert.AreEqual(original.reserved, recovered.reserved);
            }

            // NOTE: Static factory methods (hello, echo_response, state_update) are not
            // available in the manually maintained GameFFI.cs. These tests are commented
            // out until Issue 006 implements full C# auto-generation.

            /*
            [Test]
            public void GameState_Hello_CreatesValidMessage()
            {
                GameState hello = GameState.hello();

                Assert.AreEqual((byte)PacketType.GameState, hello.packet_type);
                Assert.AreEqual(PacketHeader.MAGIC, hello.magic);
                Assert.IsTrue(hello.validate(), "Hello message should be valid");
            }

            [Test]
            public void GameState_IsHello_DetectsHelloMessages()
            {
                GameState hello = GameState.hello();
                Assert.IsTrue(hello.IsHello(), "Hello message should be detected");

                GameState state = GameState.state_update(100, 5);
                Assert.IsFalse(state.IsHello(), "State update should not be detected as hello");
            }

            [Test]
            public void GameState_EchoResponse_CreatesValidResponse()
            {
                GameState response = GameState.echo_response(12345);

                Assert.AreEqual((byte)PacketType.GameState, response.packet_type);
                Assert.AreEqual(PacketHeader.MAGIC, response.magic);
                Assert.IsTrue(response.IsEcho(), "Should be detected as echo response");
            }

            [Test]
            public void GameState_GetTypeDescription_Works()
            {
                GameState hello = GameState.hello();
                Assert.AreEqual("Hello", hello.GetTypeDescription());

                GameState echo = GameState.echo_response(100);
                Assert.AreEqual("EchoResponse", echo.GetTypeDescription());

                GameState state = GameState.state_update(200, 10);
                Assert.AreEqual("PlayerCount", state.GetTypeDescription());
            }
            */

            // Tests for instance methods that exist in manual GameFFI.cs
            [Test]
            public void GameState_IsHello_DetectsHelloMessages()
            {
                var hello = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = GameState.MSG_TYPE_HELLO,
                    reserved = new byte[8]
                };
                Assert.IsTrue(hello.IsHello(), "Hello message should be detected");

                var state = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = 100,
                    reserved = new byte[8]
                };
                Assert.IsFalse(state.IsHello(), "State update should not be detected as hello");
            }

            [Test]
            public void GameState_IsEcho_DetectsEchoMessages()
            {
                var echo = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = GameState.MSG_TYPE_ECHO,
                    reserved = new byte[8]
                };
                Assert.IsTrue(echo.IsEcho(), "Echo message should be detected");

                var state = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = 100,
                    reserved = new byte[8]
                };
                Assert.IsFalse(state.IsEcho(), "State update should not be detected as echo");
            }

            [Test]
            public void GameState_GetTypeDescription_Works()
            {
                var hello = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = GameState.MSG_TYPE_HELLO,
                    reserved = new byte[8]
                };
                Assert.AreEqual("Hello", hello.GetTypeDescription());

                var echo = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 12345,
                    player_count = GameState.MSG_TYPE_ECHO,
                    reserved = new byte[8]
                };
                Assert.AreEqual("EchoResponse", echo.GetTypeDescription());

                var state = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 200,
                    player_count = 10,
                    reserved = new byte[8]
                };
                Assert.AreEqual("PlayerCount", state.GetTypeDescription());
            }
        }

        #endregion

        #region SpriteMessage Tests

        [TestFixture]
        public class SpriteMessageTests
        {
            [Test]
            public void SpriteMessage_Size_MatchesRust()
            {
                // Rust reports 30 bytes (exact)
                int expected = 30;
                int actual = GetStructSize<SpriteMessage>();
                Assert.AreEqual(expected, actual,
                    $"SpriteMessage size mismatch: expected {expected} bytes, got {actual} bytes");
            }

            [Test]
            public void SpriteMessage_FieldOffsets_MatchRust()
            {
                // Rust offsets:
                // packet_type:u8 @ 0
                // magic:u8 @ 1
                // operation:u8 @ 2
                // padding1:u8 @ 3
                // sprite_type:u8 @ 4
                // padding2:[u8;3] @ 5-7
                // id:[u8;16] @ 8-23
                // x:i16 @ 24
                // y:i16 @ 26
                // padding3:[u8;2] @ 28-29

                Assert.AreEqual(0, GetFieldOffset<SpriteMessage>("packet_type"),
                    "packet_type offset must be 0");
                Assert.AreEqual(1, GetFieldOffset<SpriteMessage>("magic"),
                    "magic offset must be 1");
                Assert.AreEqual(2, GetFieldOffset<SpriteMessage>("operation"),
                    "operation offset must be 2");
                Assert.AreEqual(4, GetFieldOffset<SpriteMessage>("sprite_type"),
                    "sprite_type offset must be 4");
                Assert.AreEqual(8, GetFieldOffset<SpriteMessage>("id"),
                    "id offset must be 8");
                Assert.AreEqual(24, GetFieldOffset<SpriteMessage>("x"),
                    "x offset must be 24");
                Assert.AreEqual(26, GetFieldOffset<SpriteMessage>("y"),
                    "y offset must be 26");
            }

            [Test]
            public void SpriteMessage_UUID_IsValidV7()
            {
                Assert.IsTrue(IsUUIDv7(SpriteMessage.UUID.ToString()),
                    $"SpriteMessage UUID {SpriteMessage.UUID} must be v7");
            }

            [Test]
            public void SpriteMessage_Size_Constant_MatchesActual()
            {
                int expectedSize = GetStructSize<SpriteMessage>();
                Assert.AreEqual(expectedSize, SpriteMessage.Size,
                    $"SpriteMessage.Size constant {SpriteMessage.Size} must match actual size {expectedSize}");
            }
            // NOTE: Static factory methods (create, update, delete, snapshot) are not
            // available in the manually maintained GameFFI.cs. These tests are commented
            // out until Issue 006 implements full C# auto-generation.

            /*
            [Test]
            public void SpriteMessage_Create_CreatesValidMessage()
            {
                Guid id = Guid.NewGuid();
                SpriteMessage msg = SpriteMessage.Create(SpriteType.Serrif, id, 50, 60);

                Assert.AreEqual((byte)PacketType.SpriteMessage, msg.packet_type);
                Assert.AreEqual(PacketHeader.MAGIC, msg.magic);
                Assert.AreEqual((byte)SpriteOp.Create, msg.operation);
                Assert.AreEqual((byte)SpriteType.Serrif, msg.sprite_type);
                Assert.AreEqual(id, msg.GetId());
                Assert.AreEqual(50, msg.x);
                Assert.AreEqual(60, msg.y);
                Assert.IsTrue(msg.validate(), "Create message should be valid");
            }

            [Test]
            public void SpriteMessage_Update_CreatesValidMessage()
            {
                Guid id = Guid.NewGuid();
                SpriteMessage msg = SpriteMessage.Update(id, 75, 85);

                Assert.AreEqual((byte)SpriteOp.Update, msg.operation);
                Assert.AreEqual(id, msg.GetId());
                Assert.AreEqual(75, msg.x);
                Assert.AreEqual(85, msg.y);
                Assert.IsTrue(msg.validate(), "Update message should be valid");
            }

            [Test]
            public void SpriteMessage_Delete_CreatesValidMessage()
            {
                Guid id = Guid.NewGuid();
                SpriteMessage msg = SpriteMessage.Delete(id);

                Assert.AreEqual((byte)SpriteOp.Delete, msg.operation);
                Assert.AreEqual(id, msg.GetId());
                Assert.IsTrue(msg.validate(), "Delete message should be valid");
            }

            [Test]
            public void SpriteMessage_Snapshot_CreatesValidMessage()
            {
                SpriteMessage msg = SpriteMessage.Snapshot();

                Assert.AreEqual((byte)SpriteOp.Snapshot, msg.operation);
                Assert.AreEqual(Guid.Empty, msg.GetId());
                Assert.IsTrue(msg.validate(), "Snapshot message should be valid");
            }

            [Test]
            public void SpriteMessage_GetId_ReturnsCorrectUUID()
            {
                Guid id = Guid.NewGuid();
                SpriteMessage msg = SpriteMessage.Create(SpriteType.Serrif, id, 10, 20);

                Assert.AreEqual(id, msg.GetId());
            }
            */

            [Test]
            public void SpriteMessage_ZeroCopy_Roundtrip_PreservesData()
            {
                Guid id = Guid.NewGuid();
                var original = new SpriteMessage
                {
                    packet_type = (byte)PacketType.SpriteMessage,
                    magic = PacketHeader.MAGIC,
                    operation = (byte)SpriteOp.Create,
                    padding1 = 0,
                    sprite_type = (byte)SpriteType.Serrif,
                    padding2 = new byte[3],
                    id = id.ToByteArray(),
                    x = 100,
                    y = 200,
                    padding3 = new byte[2]
                };

                byte[] bytes = original.AsBytes();
                var recovered = SpriteMessage.FromBytes(bytes);

                Assert.AreEqual(original.packet_type, recovered.packet_type);
                Assert.AreEqual(original.magic, recovered.magic);
                Assert.AreEqual(original.operation, recovered.operation);
                Assert.AreEqual(original.sprite_type, recovered.sprite_type);
                CollectionAssert.AreEqual(original.id, recovered.id);
                Assert.AreEqual(original.x, recovered.x);
                Assert.AreEqual(original.y, recovered.y);
            }
        }

        #endregion

        #region CrossComponent Tests

        [TestFixture]
        public class CrossComponentTests
        {
            [Test]
            public void AllTypeUUIDs_AreValidV7()
            {
                Assert.IsTrue(IsUUIDv7(PacketHeader.UUID.ToString()),
                    "PacketHeader UUID must be v7");
                Assert.IsTrue(IsUUIDv7(PlayerPos.UUID.ToString()),
                    "PlayerPos UUID must be v7");
                Assert.IsTrue(IsUUIDv7(GameState.UUID.ToString()),
                    "GameState UUID must be v7");
                Assert.IsTrue(IsUUIDv7(SpriteMessage.UUID.ToString()),
                    "SpriteMessage UUID must be v7");
            }

            [Test]
            public void AllTypeUUIDs_AreUnique()
            {
                var uuids = new HashSet<string>
                {
                    PacketHeader.UUID.ToString(),
                    PlayerPos.UUID.ToString(),
                    GameState.UUID.ToString(),
                    SpriteMessage.UUID.ToString()
                };

                Assert.AreEqual(4, uuids.Count,
                    "All type UUIDs must be unique (found {uuids.Count} unique UUIDs, expected 4)");
            }

            [Test]
            public void AllStructs_Compile_WithoutErrors()
            {
                // If this test compiles, all structs are valid
                var _ = new PacketHeader();
                var _ = new PlayerPos();
                var _ = new GameState();
                var _ = new SpriteMessage();

                Assert.Pass("All structs compile successfully");
            }

            [Test]
            public void AllPackets_HaveCorrectMagic()
            {
                var header = new PacketHeader { magic = PacketHeader.MAGIC };
                var playerPos = new PlayerPos { magic = PacketHeader.MAGIC };
                var gameState = new GameState { magic = PacketHeader.MAGIC };
                var spriteMsg = new SpriteMessage { magic = PacketHeader.MAGIC };

                Assert.AreEqual(0xCC, header.magic);
                Assert.AreEqual(0xCC, playerPos.magic);
                Assert.AreEqual(0xCC, gameState.magic);
                Assert.AreEqual(0xCC, spriteMsg.magic);
            }

            [Test]
            public void ZeroCrossType_Roundtrip_DoesNotCorruptData()
            {
                // Create packets of different types
                var playerPos = new PlayerPos
                {
                    packet_type = (byte)PacketType.PlayerPos,
                    magic = PacketHeader.MAGIC,
                    request_uuid = Guid.NewGuid(),
                    player_id = 1,
                    x = 10.0f,
                    y = 20.0f
                };

                var gameState = new GameState
                {
                    packet_type = (byte)PacketType.GameState,
                    magic = PacketHeader.MAGIC,
                    tick = 100,
                    player_count = 5,
                    reserved = new byte[8]
                };
                var spriteMsg = new SpriteMessage
                {
                    packet_type = (byte)PacketType.SpriteMessage,
                    magic = PacketHeader.MAGIC,
                    operation = (byte)SpriteOp.Create,
                    padding1 = 0,
                    sprite_type = (byte)SpriteType.Serrif,
                    padding2 = new byte[3],
                    id = Guid.NewGuid().ToByteArray(),
                    x = 50,
                    y = 60,
                    padding3 = new byte[2]
                };

                // Serialize/deserialize each
                var playerPosRecovered = PlayerPos.FromBytes(playerPos.AsBytes());
                var gameStateRecovered = GameState.FromBytes(gameState.AsBytes());
                var spriteMsgRecovered = SpriteMessage.FromBytes(spriteMsg.AsBytes());

                // Verify no corruption
                Assert.AreEqual(playerPos.player_id, playerPosRecovered.player_id);
                Assert.AreEqual(gameState.tick, gameStateRecovered.tick);
                Assert.AreEqual(spriteMsg.x, spriteMsgRecovered.x);
            }

            [Test]
            public void AllSizeConstants_MatchActual()
            {
                Assert.AreEqual(GetStructSize<PacketHeader>(), PacketHeader.Size,
                    "PacketHeader.Size must match actual size");
                Assert.AreEqual(GetStructSize<PlayerPos>(), PlayerPos.Size,
                    "PlayerPos.Size must match actual size");
                Assert.AreEqual(GetStructSize<GameState>(), GameState.Size,
                    "GameState.Size must match actual size");
                Assert.AreEqual(GetStructSize<SpriteMessage>(), SpriteMessage.Size,
                    "SpriteMessage.Size must match actual size");
            }
        }

        #endregion
    }
}
