//! PacketBuilder.cs
//!
//! High-level packet builder wrapper for Unity.
//! All UUID generation and packet construction happens in Rust.
//! Unity only provides business data (position, ID, etc.).
//!
//! Usage:
//!   byte[] packet = PacketBuilder.CreatePlayerPos(playerUuid, x, y);
//!   network.Send(packet);

using System;
using System.Runtime.InteropServices;

namespace Unity.Network
{
    /// <summary>
    /// High-level packet builder wrapper.
    /// All UUID generation and packet construction happens in Rust.
    /// Unity only provides business data (position, ID, etc.).
    /// </summary>
    /// <remarks>
    /// This replaces the old approach where Unity manually constructed packets.
    /// Now Unity just calls these high-level methods and Rust handles everything.
    /// </remarks>
    public unsafe static class PacketBuilder
    {
        private const string DLL_NAME = "unity_network";

        #region FFI Declarations

        /// <summary>
        /// Create a PlayerPos packet with auto-generated UUID v7 in Rust.
        /// </summary>
        /// <param name="idPtr">Player UUID as 16 bytes (little-endian)</param>
        /// <param name="x">X position</param>
        /// <param name="y">Y position</param>
        /// <param name="outPtr">Output buffer pointer</param>
        /// <param name="capacity">Output buffer capacity</param>
        /// <returns>Number of bytes written, or negative error code</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int packet_builder_create_player_pos(
            IntPtr idPtr,
            int x,
            int y,
            IntPtr outPtr,
            ulong capacity
        );

        /// <summary>
        /// Create a GameState packet with auto-generated UUID v7 in Rust.
        /// </summary>
        /// <param name="tick">Server tick or timestamp</param>
        /// <param name="playerCount">Number of players or message type</param>
        /// <param name="outPtr">Output buffer pointer</param>
        /// <param name="capacity">Output buffer capacity</param>
        /// <returns>Number of bytes written, or negative error code</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int packet_builder_create_game_state(
            uint tick,
            uint playerCount,
            IntPtr outPtr,
            ulong capacity
        );

        /// <summary>
        /// Create a SpriteMessage packet with auto-generated UUID v7 in Rust.
        /// </summary>
        /// <param name="operation">Sprite operation type (0=Create, 1=Update, 2=Delete, 3=Snapshot)</param>
        /// <param name="spriteType">Sprite type (0=Serrif)</param>
        /// <param name="idPtr">Sprite UUID as 16 bytes (little-endian)</param>
        /// <param name="x">X position</param>
        /// <param name="y">Y position</param>
        /// <param name="outPtr">Output buffer pointer</param>
        /// <param name="capacity">Output buffer capacity</param>
        /// <returns>Number of bytes written, or negative error code</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int packet_builder_create_sprite_message(
            byte operation,
            byte spriteType,
            IntPtr idPtr,
            short x,
            short y,
            IntPtr outPtr,
            ulong capacity
        );

        /// <summary>
        /// Create an Authenticate packet with auto-generated UUID v7 in Rust.
        /// </summary>
        /// <param name="userIdPtr">User UUID as 16 bytes (little-endian)</param>
        /// <param name="outPtr">Output buffer pointer</param>
        /// <param name="capacity">Output buffer capacity</param>
        /// <returns>Number of bytes written, or negative error code</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int packet_builder_create_authenticate(
            IntPtr userIdPtr,
            IntPtr outPtr,
            ulong capacity
        );

        /// <summary>
        /// Create a KeepAlive packet with auto-generated UUID v7 in Rust.
        /// </summary>
        /// <param name="outPtr">Output buffer pointer</param>
        /// <param name="capacity">Output buffer capacity</param>
        /// <returns>Number of bytes written, or negative error code</returns>
        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int packet_builder_create_keep_alive(
            IntPtr outPtr,
            ulong capacity
        );

        #endregion

        #region Public Methods

        /// <summary>
        /// Create a PlayerPos packet with auto-generated UUID v7.
        /// Rust handles UUID generation and packet construction.
        /// </summary>
        /// <param name="playerUuid">Player UUID</param>
        /// <param name="x">X position</param>
        /// <param name="y">Y position</param>
        /// <returns>Complete packet as byte array ready to send</returns>
        /// <exception cref="InvalidOperationException">Failed to create packet</exception>
        public static byte[] CreatePlayerPos(Guid playerUuid, int x, int y)
        {
            byte[] buffer = new byte[128]; // Sufficient for any packet

            fixed (byte* idPtr = playerUuid.ToByteArray())
            fixed (byte* outPtr = buffer)
            {
                int length = packet_builder_create_player_pos(
                    (IntPtr)idPtr,
                    x,
                    y,
                    (IntPtr)outPtr,
                    (ulong)buffer.Length
                );

                if (length < 0)
                {
                    throw new InvalidOperationException($"Failed to create PlayerPos packet: {GetErrorString(length)}");
                }

                byte[] result = new byte[length];
                Array.Copy(buffer, 0, result, 0, length);
                return result;
            }
        }

        /// <summary>
        /// Create a GameState packet with auto-generated UUID v7.
        /// Rust handles UUID generation and packet construction.
        /// </summary>
        /// <param name="tick">Server tick or timestamp</param>
        /// <param name="playerCount">Number of players or message type</param>
        /// <returns>Complete packet as byte array ready to send</returns>
        /// <exception cref="InvalidOperationException">Failed to create packet</exception>
        public static byte[] CreateGameState(uint tick, uint playerCount)
        {
            byte[] buffer = new byte[128];

            fixed (byte* outPtr = buffer)
            {
                int length = packet_builder_create_game_state(tick, playerCount, (IntPtr)outPtr, (ulong)buffer.Length);

                if (length < 0)
                {
                    throw new InvalidOperationException($"Failed to create GameState packet: {GetErrorString(length)}");
                }

                byte[] result = new byte[length];
                Array.Copy(buffer, 0, result, 0, length);
                return result;
            }
        }

        /// <summary>
        /// Create a SpriteMessage packet with auto-generated UUID v7.
        /// Rust handles UUID generation and packet construction.
        /// </summary>
        /// <param name="operation">Sprite operation type</param>
        /// <param name="spriteType">Sprite type</param>
        /// <param name="spriteUuid">Sprite UUID</param>
        /// <param name="x">X position</param>
        /// <param name="y">Y position</param>
        /// <returns>Complete packet as byte array ready to send</returns>
        /// <exception cref="InvalidOperationException">Failed to create packet</exception>
        public static byte[] CreateSpriteMessage(SpriteOperation operation, SpriteType spriteType, Guid spriteUuid, short x, short y)
        {
            byte[] buffer = new byte[128];

            fixed (byte* idPtr = spriteUuid.ToByteArray())
            fixed (byte* outPtr = buffer)
            {
                int length = packet_builder_create_sprite_message(
                    (byte)operation,
                    (byte)spriteType,
                    (IntPtr)idPtr,
                    x,
                    y,
                    (IntPtr)outPtr,
                    (ulong)buffer.Length
                );

                if (length < 0)
                {
                    throw new InvalidOperationException($"Failed to create SpriteMessage packet: {GetErrorString(length)}");
                }

                byte[] result = new byte[length];
                Array.Copy(buffer, 0, result, 0, length);
                return result;
            }
        }

        /// <summary>
        /// Create an Authenticate packet with auto-generated UUID v7.
        /// Rust handles UUID generation and packet construction.
        /// </summary>
        /// <param name="userUuid">User UUID</param>
        /// <returns>Complete packet as byte array ready to send</returns>
        /// <exception cref="InvalidOperationException">Failed to create packet</exception>
        public static byte[] CreateAuthenticate(Guid userUuid)
        {
            byte[] buffer = new byte[128];

            fixed (byte* userIdPtr = userUuid.ToByteArray())
            fixed (byte* outPtr = buffer)
            {
                int length = packet_builder_create_authenticate((IntPtr)userIdPtr, (IntPtr)outPtr, (ulong)buffer.Length);

                if (length < 0)
                {
                    throw new InvalidOperationException($"Failed to create Authenticate packet: {GetErrorString(length)}");
                }

                byte[] result = new byte[length];
                Array.Copy(buffer, 0, result, 0, length);
                return result;
            }
        }

        /// <summary>
        /// Create a KeepAlive packet with auto-generated UUID v7.
        /// Rust handles UUID generation and packet construction.
        /// </summary>
        /// <returns>Complete packet as byte array ready to send</returns>
        /// <exception cref="InvalidOperationException">Failed to create packet</exception>
        public static byte[] CreateKeepAlive()
        {
            byte[] buffer = new byte[128];

            fixed (byte* outPtr = buffer)
            {
                int length = packet_builder_create_keep_alive((IntPtr)outPtr, (ulong)buffer.Length);

                if (length < 0)
                {
                    throw new InvalidOperationException($"Failed to create KeepAlive packet: {GetErrorString(length)}");
                }

                byte[] result = new byte[length];
                Array.Copy(buffer, 0, result, 0, length);
                return result;
            }
        }

        #endregion

        #region Helper Methods

        /// <summary>
        /// Get error description from error code.
        /// </summary>
        private static string GetErrorString(int errorCode)
        {
            return errorCode switch
            {
                -1 => "Invalid pointer (null pointer passed to FFI)",
                -2 => "Buffer too small (internal buffer insufficient for packet)",
                -3 => "Invalid packet type",
                -99 => "Panic caught in Rust code (internal error)",
                _ => $"Unknown error code: {errorCode}"
            };
        }

        #endregion

        #region Enums

        /// <summary>
        /// Sprite operation types.
        /// </summary>
        public enum SpriteOperation
        {
            /// <summary>Create new sprite</summary>
            Create = 0,
            /// <summary>Update sprite position</summary>
            Update = 1,
            /// <summary>Delete sprite</summary>
            Delete = 2,
            /// <summary>Snapshot of all sprites</summary>
            Snapshot = 3
        }

        /// <summary>
        /// Sprite type enum.
        /// </summary>
        public enum SpriteType
        {
            /// <summary>Serrif sprite</summary>
            Serrif = 0
        }

        #endregion
    }
}
