using UnityEngine;
using UnityNetwork;
using System.Collections.Generic;
using System;

namespace UnityNetwork
{
    /// <summary>
    /// High-level Unity MonoBehaviour for networked player.
    /// Demonstrates sending player position updates to server via FFI.
    /// </summary>
    public class NetworkPlayer : MonoBehaviour
    {
        #region Configuration

        [Header("Server Configuration")]
        [SerializeField]
        private string serverUrl = "https://127.0.0.1:4433";

        [SerializeField]
        [Tooltip("Certificate hash (not needed in development with validation bypassed)")]
        private string certificateHash = null;

        [Header("Player Configuration")]
        [SerializeField]
        private uint playerId = 1;

        [SerializeField]
        private float updateInterval = 0.05f; // 20Hz updates

        [SerializeField]
        private bool logPackets = true;

        [Header("Debug")]
        [SerializeField]
        private bool showDebugInfo = true;

        #endregion

        #region Fields

        private NativeNetworkClient client;
        private float updateTimer;
        private bool isConnected;
        private uint lastTick;
        private uint connectedPlayerCount;
        private bool useLocalCircleFallback = true;
        private float localCircleAngle = 0f;
        private const float CIRCLE_RADIUS_LOCAL = 5f;
        private const float CIRCLE_SPEED_LOCAL = 2f;
        private const uint CIRCLE_PLAYER_ID = 999;

        // Sprite management
        private Dictionary<string, GameObject> activeSprites = new Dictionary<string, GameObject>();
        private uint spriteCreatedCount = 0;
        private uint spriteUpdatedCount = 0;
        private uint spriteDeletedCount = 0;

        #endregion

        #region Unity Lifecycle

        private void Awake()
        {
            try
            {
                Debug.Log("[NetworkPlayer] Awake - initializing native client...");
                client = new NativeNetworkClient();
                client.Initialize();
                Debug.Log("[NetworkPlayer] Awake - native client initialized successfully");
            }
            catch (System.Exception e)
            {
                Debug.LogError($"[NetworkPlayer] Awake - FATAL ERROR: {e.Message}");
                Debug.LogError($"[NetworkPlayer] Stack trace: {e.StackTrace}");
                // Don't rethrow - Unity will crash if we do
            }
        }

        private void Start()
        {
            try
            {
                Debug.Log("[NetworkPlayer] Start - connecting to server...");
                ConnectToServer();

                // Send hello world message after connection
                if (isConnected)
                {
                    Debug.Log("[NetworkPlayer] Start - connected, sending hello...");
                    SendHelloWorld();
                }
                else
                {
                    Debug.LogWarning("[NetworkPlayer] Start - not connected, will retry");
                }
            }
            catch (System.Exception e)
            {
                Debug.LogError($"[NetworkPlayer] Start - ERROR: {e.Message}");
                Debug.LogError($"[NetworkPlayer] Stack trace: {e.StackTrace}");
            }
        }

        private void Update()
        {
            if (!isConnected)
            {
                return;
            }

            // Send position updates at fixed interval
            updateTimer += Time.deltaTime;
            if (updateTimer >= updateInterval)
            {
                updateTimer = 0f;
                SendPositionUpdate();
            }

            // Poll for incoming data every frame
            PollIncomingData();

            // Fallback: If we haven't received server circle packets, move locally
            if (useLocalCircleFallback)
            {
                UpdateLocalCircleMotion();
            }
        }

        private void OnDestroy()
        {
            DisconnectFromServer();
        }

        private void OnGUI()
        {
            if (!showDebugInfo)
            {
                return;
            }

            // Draw debug info on screen
            GUI.Box(new Rect(10, 10, 250, 120), "Network Player Debug");

            GUI.Label(new Rect(20, 35, 230, 20), $"Status: {(isConnected ? "Connected" : "Disconnected")}");
            GUI.Label(new Rect(20, 55, 230, 20), $"Player ID: {playerId}");
            GUI.Label(new Rect(20, 75, 230, 20), $"Position: ({transform.position.x:F2}, {transform.position.y:F2})");
            GUI.Label(new Rect(20, 95, 230, 20), $"Server Tick: {lastTick}");
            GUI.Label(new Rect(20, 115, 230, 20), $"Players: {connectedPlayerCount}");

            // Sprite statistics
            GUI.Label(new Rect(20, 140, 230, 20), "--- Sprite Stats ---");
            GUI.Label(new Rect(20, 160, 230, 20), $"Created: {spriteCreatedCount}");
            GUI.Label(new Rect(20, 180, 230, 20), $"Updated: {spriteUpdatedCount}");
            GUI.Label(new Rect(20, 200, 230, 20), $"Deleted: {spriteDeletedCount}");
            GUI.Label(new Rect(20, 220, 230, 20), $"Active: {activeSprites.Count}");
        }

        #endregion

        #region Connection Management

        private void ConnectToServer()
        {
            try
            {
                Debug.Log($"[NetworkPlayer] ConnectToServer - starting connection to {serverUrl}");
                Debug.Log($"[NetworkPlayer] ConnectToServer - certificate hash: {(string.IsNullOrEmpty(certificateHash) ? "(none)" : certificateHash)}");

                client.Connect(serverUrl, certificateHash);
                isConnected = true;
                Debug.Log("[NetworkPlayer] ConnectToServer - connection successful!");
            }
            catch (System.DllNotFoundException e)
            {
                Debug.LogError("[NetworkPlayer] ConnectToServer - FATAL: Native library not found!");
                Debug.LogError("[NetworkPlayer] Make sure libunity_network.dylib is in Assets/Plugins/macOS/");
                Debug.LogError($"[NetworkPlayer] Error: {e.Message}");
                isConnected = false;
            }
            catch (System.BadImageFormatException e)
            {
                Debug.LogError("[NetworkPlayer] ConnectToServer - FATAL: Invalid native library format!");
                Debug.LogError("[NetworkPlayer] The library may be for wrong platform or architecture");
                Debug.LogError($"[NetworkPlayer] Error: {e.Message}");
                isConnected = false;
            }
            catch (System.Exception e)
            {
                Debug.LogError($"[NetworkPlayer] ConnectToServer - ERROR: {e.Message}");
                Debug.LogError($"[NetworkPlayer] Error type: {e.GetType().Name}");
                Debug.LogError($"[NetworkPlayer] Stack trace: {e.StackTrace}");
                isConnected = false;
            }
        }

        private void DisconnectFromServer()
        {
            if (isConnected)
            {
                try
                {
                    client.Disconnect();
                    isConnected = false;
                    Debug.Log("Disconnected from server");
                }
                catch (System.Exception e)
                {
                    Debug.LogError($"Error during disconnect: {e.Message}");
                }
            }
        }

        #endregion

        #region Sending

        private void SendPositionUpdate()
        {
            try
            {
                // Create PlayerPos struct with current position
                var pos = new PlayerPos(
                    playerId,
                    transform.position.x,
                    transform.position.y
                );

                // Send struct directly (zero-copy, no GC)
                client.SendStruct(pos);

                if (logPackets)
                {
                    Debug.Log($"[Sent] PlayerPos: id={pos.id}, x={pos.x:F2}, y={pos.y:F2}");
                }
            }
            catch (System.Exception e)
            {
                Debug.LogError($"Failed to send position update: {e.Message}");
                isConnected = false;
            }
        }

        #endregion

        #region Receiving

        private void PollIncomingData()
        {
            try
            {
                // Poll for incoming data
                int bytesReceived = client.Poll();

                if (bytesReceived == 0)
                {
                    return; // No data available - this is normal most frames
                }

                if (bytesReceived < 0)
                {
                    // Error occurred
                    var error = (FfiError)bytesReceived;
                    Debug.LogError($"Poll error: {error}");
                    isConnected = false;
                    return;
                }

                // Get packet type from header
                var packetType = client.GetPacketType(bytesReceived);
                if (!packetType.HasValue)
                {
                    Debug.LogWarning("Received invalid packet (no valid header)");
                    return;
                }

                // Handle packet based on type
                switch (packetType.Value)
                {
                    case PacketType.KeepAlive:
                        HandleKeepAlive(bytesReceived);
                        break;

                    case PacketType.PlayerPos:
                        HandlePlayerPos(bytesReceived);
                        break;

                    case PacketType.GameState:
                        HandleGameState(bytesReceived);
                        break;

                    case PacketType.SpriteMessage:
                        HandleSpriteMessage(bytesReceived);
                        break;

                    default:
                        Debug.LogWarning($"Unknown packet type: {packetType.Value}");
                        break;
                }
            }
            catch (System.Exception e)
            {
                Debug.LogError($"Error polling data: {e.Message}");
            }
        }

        private void HandleKeepAlive(int length)
        {
            if (logPackets)
            {
                Debug.Log("[Received] KeepAlive");
            }
            // KeepAlive packets don't have additional data beyond header
        }

        private void HandlePlayerPos(int length)
        {
            if (!client.TryParseStruct<PlayerPos>(length, out var pos))
            {
                Debug.LogWarning("Failed to parse PlayerPos packet");
                return;
            }

            if (!pos.Validate())
            {
                Debug.LogWarning("Received invalid PlayerPos packet");
                return;
            }

            if (logPackets)
            {
                Debug.Log($"[Received] PlayerPos: id={pos.id}, x={pos.x:F2}, y={pos.y:F2}");
            }

            // Handle circle player position from server (ID 999)
            if (pos.id == CIRCLE_PLAYER_ID)
            {
                Debug.Log($"<color=yellow>Received Circle Motion from server at ({pos.x:F2}, {pos.y:F2})</color>");
                // Move this sprite in a circle based on server position
                transform.position = new Vector3(pos.x, pos.y, 0);
                // We received from server, disable local fallback
                useLocalCircleFallback = false;
                Debug.Log("<color=green>✓ Server packets working, local fallback disabled</color>");
            }
            // If this is not our player, we could update their position
            else if (pos.id != playerId)
            {
                // TODO: Update other players' positions
                // For now, just log it
            }
        }

        private void HandleGameState(int length)
        {
            if (!client.TryParseStruct<GameState>(length, out var state))
            {
                Debug.LogWarning("Failed to parse GameState packet");
                return;
            }

            if (!state.Validate())
            {
                Debug.LogWarning("Received invalid GameState packet");
                return;
            }

            // Check message type and handle accordingly
            if (state.IsHello())
            {
                if (logPackets)
                {
                    Debug.Log($"[Received] Hello from client: tick={state.tick}");
                }
            }
            else if (state.IsEcho())
            {
                // Echo response from server - hello world round-trip complete!
                Debug.Log($"<color=green>Hello World Round-Trip Complete!</color>");
                Debug.Log($"Server echoed our hello request at tick {state.tick}");
            }
            else
            {
                // Regular game state update
                lastTick = state.tick;
                connectedPlayerCount = state.playerCount;

                if (logPackets)
                {
                    Debug.Log($"[Received] GameState: tick={state.tick}, players={state.playerCount}");
                }
            }
        }

        #endregion

        #region Hello World

        private void SendHelloWorld()
        {
            try
            {
                // Create a hello message using GameState packet
                var hello = GameStateExtensions.CreateHello();

                // Send struct directly (zero-copy, no GC)
                client.SendStruct(hello);

                if (logPackets)
                {
                    Debug.Log($"[Sent] Hello message at tick {hello.tick}");
                }
            }
            catch (System.Exception e)
            {
                Debug.LogError($"Failed to send hello: {e.Message}");
            }
        }

        #endregion

        #region Local Circle Fallback

        private void UpdateLocalCircleMotion()
        {
            // Update angle based on time
            localCircleAngle += CIRCLE_SPEED_LOCAL * Time.deltaTime;

            // Calculate circle position
            float x = CIRCLE_RADIUS_LOCAL * Mathf.Cos(localCircleAngle);
            float y = CIRCLE_RADIUS_LOCAL * Mathf.Sin(localCircleAngle);

            // Update position
            transform.position = new Vector3(x, y, 0);

            // Log occasionally
            if (Time.frameCount % 60 == 0)
            {
                Debug.Log($"<color=cyan>[Local Fallback] Moving in circle: angle={localCircleAngle:F2}, pos=({x:F2}, {y:F2})</color>");
            }
        }

        #endregion

        #region Sprite Management

        /// <summary>
        /// Handle sprite messages (JSON format)
        /// </summary>
        /// <summary>
        /// Handle sprite message (struct-based, zero-copy)
        /// </summary>
        private void HandleSpriteMessage(int length)
        {
            if (!client.TryParseStruct<SpriteMessage>(length, out var spriteMsg))
            {
                Debug.LogWarning("Failed to parse SpriteMessage packet");
                return;
            }

            if (!spriteMsg.Validate())
            {
                Debug.LogWarning("Received invalid SpriteMessage packet");
                return;
            }

            switch (spriteMsg.GetOperation())
            {
                case SpriteOp.Create:
                    HandleSpriteCreate(spriteMsg);
                    break;

                case SpriteOp.Update:
                    HandleSpriteUpdate(spriteMsg);
                    break;

                case SpriteOp.Delete:
                    HandleSpriteDelete(spriteMsg);
                    break;

                case SpriteOp.Snapshot:
                    HandleSpriteSnapshot(spriteMsg);
                    break;

                default:
                    Debug.LogWarning($"Unknown sprite operation: {spriteMsg.GetOperation()}");
                    break;
            }
        }

        /// <summary>
        /// Handle sprite creation message
        /// </summary>
        private void HandleSpriteCreate(SpriteMessage spriteMsg)
        {
            try
            {
                Guid id = spriteMsg.GetId();
                string spriteName = $"serrif_{id}";

                Debug.Log($"<color=green>[CREATE] {spriteName} at ({spriteMsg.x}, {spriteMsg.y})</color>");

                // Create a simple sprite GameObject
                GameObject spriteObj = new GameObject(spriteName);
                spriteObj.transform.position = new Vector3(spriteMsg.x, spriteMsg.y, 0);

                // Add a SpriteRenderer component
                SpriteRenderer renderer = spriteObj.AddComponent<SpriteRenderer>();

                // Load sirref sprite from Resources
                Sprite sirrefSprite = Resources.Load<Sprite>("Sprites/sirref");
                if (sirrefSprite != null)
                {
                    renderer.sprite = sirrefSprite;
                }
                else
                {
                    Debug.LogWarning("sirref sprite not found in Resources/Sprites/, falling back to square");
                    renderer.sprite = CreateSquareSprite();
                }

                // Store in active sprites
                activeSprites[id.ToString()] = spriteObj;
                spriteCreatedCount++;

                // Debug: Show position relative to player and camera
                if (logPackets)
                {
                    Vector3 playerPos = transform != null ? transform.position : Vector3.zero;
                    Vector3 cameraPos = Camera.main != null ? Camera.main.transform.position : Vector3.zero;
                    float distanceToPlayer = Vector3.Distance(spriteObj.transform.position, playerPos);
                    float distanceToCamera = Vector3.Distance(spriteObj.transform.position, cameraPos);

                    Debug.Log($"[SPRITE DEBUG] Sprite at ({spriteMsg.x}, {spriteMsg.y}) | Player at ({playerPos.x:F1}, {playerPos.y:F1}) | Camera at ({cameraPos.x:F1}, {cameraPos.y:F1}) | Distance to player: {distanceToPlayer:F1} | Distance to camera: {distanceToCamera:F1}");
                }
            }
            catch (System.Exception e)
            {
                Debug.LogError($"Failed to handle sprite create: {e.Message}");
            }
        }

        /// <summary>
        /// Handle sprite update message
        /// </summary>
        private void HandleSpriteUpdate(SpriteMessage spriteMsg)
        {
            try
            {
                Guid id = spriteMsg.GetId();
                if (activeSprites.TryGetValue(id.ToString(), out GameObject sprite))
                {
                    sprite.transform.position = new Vector3(spriteMsg.x, spriteMsg.y, 0);
                    spriteUpdatedCount++;

                    Debug.Log($"<color=yellow>[UPDATE] {sprite.name} moved to ({spriteMsg.x}, {spriteMsg.y})</color>");
                }
                else
                {
                    Debug.LogWarning($"[UPDATE] Sprite {id} not found (may have been deleted)");
                }
            }
            catch (System.Exception e)
            {
                Debug.LogError($"Failed to handle sprite update: {e.Message}");
            }
        }

        /// <summary>
        /// Handle sprite deletion message
        /// </summary>
        private void HandleSpriteDelete(SpriteMessage spriteMsg)
        {
            try
            {
                Guid id = spriteMsg.GetId();
                if (activeSprites.TryGetValue(id.ToString(), out GameObject sprite))
                {
                    Debug.Log($"<color=red>[DELETE] {sprite.name}</color>");
                    Destroy(sprite);
                    activeSprites.Remove(id.ToString());
                    spriteDeletedCount++;
                }
                else
                {
                    // Sprite already deleted - DELETE is idempotent, duplicate packets are normal
                }
            }
            catch (System.Exception e)
            {
                Debug.LogError($"Failed to handle sprite delete: {e.Message}");
            }
        }

        /// <summary>
        /// Handle sprite snapshot message
        /// </summary>
        private void HandleSpriteSnapshot(SpriteMessage spriteMsg)
        {
            try
            {
                Debug.Log($"<color=cyan>[SNAPSHOT] Server snapshot received</color>");
                // Snapshot messages don't contain detailed data in this simple implementation
                // Server just sends periodic snapshots to indicate it's alive
            }
            catch (System.Exception e)
            {
                Debug.LogError($"Failed to handle sprite snapshot: {e.Message}");
            }
        }

        /// <summary>
        /// Create a simple square sprite for rendering (fallback)
        /// </summary>
        private Sprite CreateSquareSprite()
        {
            Texture2D texture = new Texture2D(1, 1);
            texture.SetPixel(0, 0, Color.white);
            texture.Apply();

            return Sprite.Create(texture, new Rect(0, 0, 1, 1), new Vector2(0.5f, 0.5f), 0.1f); // 10x larger for visibility
        }

        #endregion



        #region Public API

        /// <summary>
        /// Reconnect to the server with current configuration.
        /// </summary>
        public void Reconnect()
        {
            if (isConnected)
            {
                DisconnectFromServer();
            }

            ConnectToServer();
        }

        /// <summary>
        /// Set player ID.
        /// </summary>
        public void SetPlayerId(uint id)
        {
            playerId = id;
            Debug.Log($"Player ID set to: {playerId}");
        }

        /// <summary>
        /// Check if connected to server.
        /// </summary>
        public bool IsConnected()
        {
            return isConnected && client.IsConnected;
        }

        #endregion
    }
}
