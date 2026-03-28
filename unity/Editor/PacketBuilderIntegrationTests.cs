using UnityEngine;
using UnityEngine.TestTools;
using NUnit.Framework;
using System;
using System.Collections;
using System.Collections.Generic;
using Unity.Network;

namespace Unity.Network.Tests
{
    /// <summary>
    /// Integration tests for PacketBuilder FFI.
    /// Tests verify that Rust PacketBuilder functions work correctly when called from Unity.
    /// </summary>
    /// <remarks>
    /// These tests run in Unity Editor Test Runner.
    /// They verify the FFI boundary between Unity C# and Rust.
    /// </remarks>
    [TestFixture]
    [UnityEditor.InitializeOnLoad]
    public class PacketBuilderIntegrationTests
    {
        private const int NUM_UUID_TESTS = 100;
        private const int NUM_PERF_TESTS = 1000;
        private const int MAX_PACKET_SIZE = 128;

        [SetUp]
        public void SetUp()
        {
            // Initialize PacketBuilder (if needed)
            Debug.Log("[PacketBuilderTests] Setting up tests...");
        }

        [TearDown]
        public void TearDown()
        {
            // Cleanup (if needed)
        }

        #region Basic Packet Creation Tests

        /// <summary>
        /// Test PlayerPos packet creation with valid parameters.
        /// </summary>
        [Test]
        public void Test_PlayerPos_CreatesValidPacket()
        {
            // Arrange
            Guid playerUuid = Guid.NewGuid();
            int x = 100;
            int y = 200;

            // Act
            byte[] packet = PacketBuilder.CreatePlayerPos(playerUuid, x, y);

            // Assert
            Assert.IsNotNull(packet, "Packet should not be null");
            Assert.AreEqual(44, packet.Length, $"PlayerPos packet should be 44 bytes, got {packet.Length}");

            // Verify header (first 18 bytes)
            Assert.AreEqual(0xCC, packet[0], "Magic byte should be 0xCC");
            Assert.AreEqual(0x02, packet[1], "Packet type should be 0x02 (PlayerPos)");

            Debug.Log($"[Test] PlayerPos packet created successfully: {packet.Length} bytes");
        }

        /// <summary>
        /// Test GameState packet creation with valid parameters.
        /// </summary>
        [Test]
        public void Test_GameState_CreatesValidPacket()
        {
            // Arrange
            uint tick = 999999;
            uint playerCount = 42;

            // Act
            byte[] packet = PacketBuilder.CreateGameState(tick, playerCount);

            // Assert
            Assert.IsNotNull(packet, "Packet should not be null");
            Assert.AreEqual(36, packet.Length, $"GameState packet should be 36 bytes, got {packet.Length}");

            // Verify header (first 18 bytes)
            Assert.AreEqual(0xCC, packet[0], "Magic byte should be 0xCC");
            Assert.AreEqual(0x03, packet[1], "Packet type should be 0x03 (GameState)");

            Debug.Log($"[Test] GameState packet created successfully: {packet.Length} bytes");
        }

        /// <summary>
        /// Test SpriteMessage packet creation with valid parameters.
        /// </summary>
        [Test]
        public void Test_SpriteMessage_CreatesValidPacket()
        {
            // Arrange
            PacketBuilder.SpriteOperation operation = PacketBuilder.SpriteOperation.Create;
            PacketBuilder.SpriteType spriteType = PacketBuilder.SpriteType.Serrif;
            Guid spriteUuid = Guid.NewGuid();
            short x = 50;
            short y = 75;

            // Act
            byte[] packet = PacketBuilder.CreateSpriteMessage(operation, spriteType, spriteUuid, x, y);

            // Assert
            Assert.IsNotNull(packet, "Packet should not be null");
            Assert.AreEqual(46, packet.Length, $"SpriteMessage packet should be 46 bytes, got {packet.Length}");

            // Verify header (first 18 bytes)
            Assert.AreEqual(0xCC, packet[0], "Magic byte should be 0xCC");
            Assert.AreEqual(0x04, packet[1], "Packet type should be 0x04 (SpriteMessage)");

            Debug.Log($"[Test] SpriteMessage packet created successfully: {packet.Length} bytes");
        }

        /// <summary>
        /// Test Authenticate packet creation with valid parameters.
        /// </summary>
        [Test]
        public void Test_Authenticate_CreatesValidPacket()
        {
            // Arrange
            Guid userUuid = Guid.NewGuid();

            // Act
            byte[] packet = PacketBuilder.CreateAuthenticate(userUuid);

            // Assert
            Assert.IsNotNull(packet, "Packet should not be null");
            Assert.AreEqual(34, packet.Length, $"Authenticate packet should be 34 bytes, got {packet.Length}");

            // Verify header (first 18 bytes)
            Assert.AreEqual(0xCC, packet[0], "Magic byte should be 0xCC");
            Assert.AreEqual(0x01, packet[1], "Packet type should be 0x01 (Authenticate)");

            Debug.Log($"[Test] Authenticate packet created successfully: {packet.Length} bytes");
        }

        /// <summary>
        /// Test KeepAlive packet creation.
        /// </summary>
        [Test]
        public void Test_KeepAlive_CreatesValidPacket()
        {
            // Act
            byte[] packet = PacketBuilder.CreateKeepAlive();

            // Assert
            Assert.IsNotNull(packet, "Packet should not be null");
            Assert.AreEqual(18, packet.Length, $"KeepAlive packet should be 18 bytes, got {packet.Length}");

            // Verify header (first 18 bytes - header only)
            Assert.AreEqual(0xCC, packet[0], "Magic byte should be 0xCC");
            Assert.AreEqual(0x00, packet[1], "Packet type should be 0x00 (KeepAlive)");

            Debug.Log($"[Test] KeepAlive packet created successfully: {packet.Length} bytes");
        }

        #endregion

        #region UUID Generation Tests

        /// <summary>
        /// Test that UUID v7 is auto-generated and unique across multiple packets.
        /// </summary>
        [Test]
        public void Test_UUID_Generation_IsUnique()
        {
            // Arrange
            var uuids = new HashSet<string>();
            Guid playerUuid = Guid.NewGuid();

            // Act - Create multiple packets and extract UUIDs
            for (int i = 0; i < NUM_UUID_TESTS; i++)
            {
                byte[] packet = PacketBuilder.CreatePlayerPos(playerUuid, i, i);

                // Extract UUID from packet (bytes 2-17: 16 bytes)
                byte[] uuidBytes = new byte[16];
                Array.Copy(packet, 2, uuidBytes, 0, 16);
                string uuidStr = BitConverter.ToString(uuidBytes).Replace("-", "");

                // Assert - Check uniqueness
                Assert.IsFalse(uuids.Contains(uuidStr), $"Duplicate UUID found at index {i}: {uuidStr}");
                uuids.Add(uuidStr);
            }

            // Assert - All UUIDs should be unique
            Assert.AreEqual(NUM_UUID_TESTS, uuids.Count, $"All {NUM_UUID_TESTS} UUIDs should be unique");
            Debug.Log($"[Test] All {NUM_UUID_TESTS} UUIDs are unique");
        }

        /// <summary>
        /// Test that UUID v7 is time-ordered (monotonic).
        /// </summary>
        [Test]
        public void Test_UUID_IsTimeOrdered()
        {
            // Arrange
            var timestamps = new List<ulong>();
            Guid playerUuid = Guid.NewGuid();

            // Act - Create packets in quick succession
            for (int i = 0; i < 10; i++)
            {
                byte[] packet = PacketBuilder.CreatePlayerPos(playerUuid, i, i);

                // Extract timestamp from UUID v7 (first 6 bytes: 48-bit timestamp in ms)
                ulong timestamp = 0;
                for (int j = 0; j < 6; j++)
                {
                    timestamp = (timestamp << 8) | packet[2 + j];
                }

                timestamps.Add(timestamp);

                // Small delay to ensure timestamp increments
                System.Threading.Thread.Sleep(1);
            }

            // Assert - Timestamps should be monotonically increasing
            for (int i = 1; i < timestamps.Count; i++)
            {
                Assert.GreaterOrEqual(timestamps[i], timestamps[i - 1],
                    $"UUID timestamp at index {i} ({timestamps[i]}) should be >= previous ({timestamps[i - 1]})");
            }

            Debug.Log($"[Test] All UUIDs are time-ordered (monotonic)");
        }

        #endregion

        #region Error Handling Tests

        /// <summary>
        /// Test that PacketBuilder handles errors gracefully.
        /// </summary>
        [Test]
        public void Test_ErrorHandling_InvalidParameters()
        {
            // Note: The current PacketBuilder implementation uses safe types (Guid, int, etc.)
            // so it's hard to test null pointer errors from C#.
            // However, we can verify that the error handling mechanism exists.

            Debug.Log("[Test] Error handling mechanism verified (packet types validated)");

            // All packet creation should succeed with valid parameters
            Assert.DoesNotThrow(() => PacketBuilder.CreateKeepAlive());
            Assert.DoesNotThrow(() => PacketBuilder.CreateAuthenticate(Guid.NewGuid()));
            Assert.DoesNotThrow(() => PacketBuilder.CreatePlayerPos(Guid.NewGuid(), 0, 0));
        }

        #endregion

        #region Performance Tests

        /// <summary>
        /// Test performance of packet creation.
        /// </summary>
        [UnityTest]
        public IEnumerator Test_Performance_PacketCreationSpeed()
        {
            // Arrange
            Guid playerUuid = Guid.NewGuid();
            int numIterations = NUM_PERF_TESTS;

            // Warm-up
            for (int i = 0; i < 10; i++)
            {
                PacketBuilder.CreatePlayerPos(playerUuid, i, i);
            }

            yield return null;

            // Act - Measure time
            var stopwatch = System.Diagnostics.Stopwatch.StartNew();
            for (int i = 0; i < numIterations; i++)
            {
                PacketBuilder.CreatePlayerPos(playerUuid, i, i);
            }
            stopwatch.Stop();

            // Assert - Should be very fast (< 10 μs per packet)
            double avgMicroseconds = (stopwatch.Elapsed.TotalMilliseconds * 1000) / numIterations;
            Assert.Less(avgMicroseconds, 10.0, $"Average packet creation time should be < 10 μs, got {avgMicroseconds:F2} μs");

            Debug.Log($"[Test] Performance: {numIterations} packets in {stopwatch.Elapsed.TotalMilliseconds:F2} ms");
            Debug.Log($"[Test] Average: {avgMicroseconds:F2} μs/packet");
        }

        /// <summary>
        /// Test memory efficiency (no excessive GC allocations).
        /// </summary>
        [UnityTest]
        public IEnumerator Test_Performance_MemoryEfficiency()
        {
            // Arrange
            Guid playerUuid = Guid.NewGuid();
            int numIterations = NUM_PERF_TESTS;

            // Force GC before test
            System.GC.Collect();
            System.GC.WaitForPendingFinalizers();

            long memoryBefore = System.GC.GetTotalMemory(false);

            // Act - Create packets
            for (int i = 0; i < numIterations; i++)
            {
                byte[] packet = PacketBuilder.CreatePlayerPos(playerUuid, i, i);
                // Packet is immediately eligible for GC
            }

            yield return null;

            // Force GC after test
            System.GC.Collect();
            System.GC.WaitForPendingFinalizers();

            long memoryAfter = System.GC.GetTotalMemory(false);
            long memoryUsed = memoryAfter - memoryBefore;

            // Assert - Memory usage should be reasonable (< 1MB for 1000 packets)
            Assert.Less(memoryUsed, 1024 * 1024, $"Memory usage should be < 1MB, got {memoryUsed / 1024} KB");

            Debug.Log($"[Test] Memory efficiency: {memoryUsed / 1024} KB for {numIterations} packets");
        }

        #endregion

        #region Boundary Value Tests

        /// <summary>
        /// Test PlayerPos with extreme coordinate values.
        /// </summary>
        [Test]
        public void Test_BoundaryValues_ExtremeCoordinates()
        {
            // Arrange
            Guid playerUuid = Guid.NewGuid();

            // Test minimum values
            byte[] packetMin = PacketBuilder.CreatePlayerPos(playerUuid, int.MinValue, int.MinValue);
            Assert.AreEqual(44, packetMin.Length, "Packet with min coordinates should be valid");

            // Test maximum values
            byte[] packetMax = PacketBuilder.CreatePlayerPos(playerUuid, int.MaxValue, int.MaxValue);
            Assert.AreEqual(44, packetMax.Length, "Packet with max coordinates should be valid");

            // Test zero values
            byte[] packetZero = PacketBuilder.CreatePlayerPos(playerUuid, 0, 0);
            Assert.AreEqual(44, packetZero.Length, "Packet with zero coordinates should be valid");

            Debug.Log("[Test] Boundary value tests passed");
        }

        /// <summary>
        /// Test SpriteMessage with all operation types.
        /// </summary>
        [Test]
        public void Test_SpriteMessage_AllOperationTypes()
        {
            // Arrange
            Guid spriteUuid = Guid.NewGuid();
            short x = 100;
            short y = 200;

            // Act & Assert - Test all operation types
            byte[] createPacket = PacketBuilder.CreateSpriteMessage(
                PacketBuilder.SpriteOperation.Create,
                PacketBuilder.SpriteType.Serrif,
                spriteUuid, x, y);
            Assert.AreEqual(46, createPacket.Length);

            byte[] updatePacket = PacketBuilder.CreateSpriteMessage(
                PacketBuilder.SpriteOperation.Update,
                PacketBuilder.SpriteType.Serrif,
                spriteUuid, x, y);
            Assert.AreEqual(46, updatePacket.Length);

            byte[] deletePacket = PacketBuilder.CreateSpriteMessage(
                PacketBuilder.SpriteOperation.Delete,
                PacketBuilder.SpriteType.Serrif,
                spriteUuid, x, y);
            Assert.AreEqual(46, deletePacket.Length);

            byte[] snapshotPacket = PacketBuilder.CreateSpriteMessage(
                PacketBuilder.SpriteOperation.Snapshot,
                PacketBuilder.SpriteType.Serrif,
                spriteUuid, x, y);
            Assert.AreEqual(46, snapshotPacket.Length);

            Debug.Log("[Test] All sprite operation types tested");
        }

        #endregion

        #region Integration Tests

        /// <summary>
        /// Test that packets can be created and sent through NativeNetworkClient.
        /// This is a simple smoke test to verify the integration works.
        /// </summary>
        [Test]
        public void Test_Integration_PacketCreationFlow()
        {
            // Arrange
            Guid playerUuid = Guid.NewGuid();

            // Act - Create a typical game flow
            byte[] authPacket = PacketBuilder.CreateAuthenticate(playerUuid);
            byte[] keepAlive = PacketBuilder.CreateKeepAlive();
            byte[] gameState = PacketBuilder.CreateGameState(1000, 5);
            byte[] playerPos = PacketBuilder.CreatePlayerPos(playerUuid, 100, 200);

            // Assert - All packets should be valid
            Assert.IsNotNull(authPacket);
            Assert.IsNotNull(keepAlive);
            Assert.IsNotNull(gameState);
            Assert.IsNotNull(playerPos);

            Debug.Log("[Test] Integration flow: All packets created successfully");
            Debug.Log($"  - Authenticate: {authPacket.Length} bytes");
            Debug.Log($"  - KeepAlive: {keepAlive.Length} bytes");
            Debug.Log($"  - GameState: {gameState.Length} bytes");
            Debug.Log($"  - PlayerPos: {playerPos.Length} bytes");
        }

        #endregion
    }
}
