using UnityEngine;
using Unity.Profiler;
using System;

namespace Unity.Profiler.Examples
{
    /// <summary>
    /// Comprehensive usage example for Network Profiler integration
    /// Demonstrates various profiling scenarios in a MMORPG context
    /// </summary>
    [AddComponentMenu("Examples/Network Profiler Example")]
    public class NetworkProfilerExample : MonoBehaviour
    {
        #region Settings

        [Header("Example Settings")]
        [Tooltip("Enable automatic request simulation")]
        [SerializeField]
        private bool autoSimulateRequests = true;

        [Tooltip("Request simulation interval (seconds)")]
        [SerializeField]
        private float simulationInterval = 1.0f;

        [Tooltip("Number of requests to simulate per interval")]
        [SerializeField]
        private int requestsPerInterval = 3;

        [Tooltip("Show debug information in console")]
        [SerializeField]
        private bool showDebugInfo = true;

        #endregion

        #region Fields

        private float lastSimulationTime;
        private int totalSimulatedRequests;

        #endregion

        #region Unity Lifecycle

        private void Start()
        {
            LogInfo("Network Profiler Example Started");
            LogInfo($"Profiler ready: {NetworkProfilerBehaviour.Instance?.IsProfilerReady ?? false}");

            // Example 1: Simple request tracking
            Example1_SimpleRequestTracking();

            // Example 2: Request with manual stage management
            Example2_ManualStageManagement();

            // Example 3: Request with error handling
            Example3_ErrorHandling();
        }

        private void Update()
        {
            if (autoSimulateRequests)
            {
                SimulateRequests();
            }
        }

        private void OnDestroy()
        {
            LogInfo($"Total simulated requests: {totalSimulatedRequests}");
        }

        #endregion

        #region Automatic Simulation

        /// <summary>
        /// Automatically simulate various network requests
        /// </summary>
        private void SimulateRequests()
        {
            if (Time.realtimeSinceStartup - lastSimulationTime < simulationInterval)
            {
                return;
            }

            lastSimulationTime = Time.realtimeSinceStartup;

            for (int i = 0; i < requestsPerInterval; i++)
            {
                SimulateRandomRequest();
            }
        }

        /// <summary>
        /// Simulate a random network request
        /// </summary>
        private void SimulateRandomRequest()
        {
            RequestType[] requestTypes = {
                RequestType.MoveCommand,
                RequestType.ShopAction,
                RequestType.ChatMessage,
                RequestType.CharacterUpdate,
                RequestType.InventoryAction
            };

            RequestType randomType = requestTypes[UnityEngine.Random.Range(0, requestTypes.Length)];

            NetworkProfilerBehaviour.Track(randomType, requestId =>
            {
                // Simulate request processing time
                float processingTime = UnityEngine.Random.Range(0.01f, 0.05f);
                System.Threading.Thread.Sleep((int)(processingTime * 1000));

                // Simulate occasional failures
                if (UnityEngine.Random.value < 0.1f)
                {
                    throw new InvalidOperationException("Simulated network failure");
                }
            });

            totalSimulatedRequests++;
        }

        #endregion

        #region Example 1: Simple Request Tracking

        /// <summary>
        /// Example 1: Simple request tracking with automatic stage management
        /// Uses the high-level TrackRequest method for easy profiling
        /// </summary>
        private void Example1_SimpleRequestTracking()
        {
            LogInfo("=== Example 1: Simple Request Tracking ===");

            try
            {
                // Track a movement command request
                Guid requestId = NetworkProfilerBehaviour.Track(RequestType.MoveCommand, id =>
                {
                    // Your network request code here
                    LogInfo($"Processing movement request: {id}");

                    // Simulate network call
                    SimulateNetworkCall("Movement API");
                });

                LogInfo($"Movement request completed: {requestId}");

                // Track a shop action
                requestId = NetworkProfilerBehaviour.Track(RequestType.ShopAction, id =>
                {
                    LogInfo($"Processing shop action: {id}");
                    SimulateNetworkCall("Shop API");
                });

                LogInfo($"Shop action completed: {requestId}");
            }
            catch (Exception ex)
            {
                LogError($"Example 1 failed: {ex.Message}");
            }
        }

        #endregion

        #region Example 2: Manual Stage Management

        /// <summary>
        /// Example 2: Manual stage management for fine-grained control
        /// Useful when you need to track specific stages of a request
        /// </summary>
        private void Example2_ManualStageManagement()
        {
            LogInfo("=== Example 2: Manual Stage Management ===");

            try
            {
                // Start a new request
                Guid requestId = NetworkProfilerBehaviour.StartRequest(RequestType.CharacterUpdate);
                LogInfo($"Started character update: {requestId}");

                // Record stages manually
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.UserInput);
                // ... User input processing ...
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.UserInput);

                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.UnityProcess);
                // ... Unity processing ...
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.UnityProcess);

                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.RustFFIOutbound);
                // ... FFI call to Rust ...
                SimulateNetworkCall("Character Update API");
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.RustFFIOutbound);

                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.RustFFIInbound);
                // ... FFI response from Rust ...
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.RustFFIInbound);

                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.UnityRender);
                // ... Unity rendering ...
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.UnityRender);

                // Complete the request
                NetworkProfilerBehaviour.CompleteRequest(requestId, RequestStatus.Completed);
                LogInfo($"Character update completed: {requestId}");
            }
            catch (Exception ex)
            {
                LogError($"Example 2 failed: {ex.Message}");
            }
        }

        #endregion

        #region Example 3: Error Handling

        /// <summary>
        /// Example 3: Demonstrating error handling and status tracking
        /// Shows how to track failed and timed-out requests
        /// </summary>
        private void Example3_ErrorHandling()
        {
            LogInfo("=== Example 3: Error Handling ===");

            // Example of a failed request
            try
            {
                Guid requestId = NetworkProfilerBehaviour.Track(RequestType.Authentication, id =>
                {
                    LogInfo($"Attempting authentication: {id}");

                    // Simulate authentication failure
                    throw new UnauthorizedAccessException("Invalid credentials");
                });

                // This won't be reached because the exception is thrown
                LogInfo($"Authentication request completed: {requestId}");
            }
            catch (Exception ex)
            {
                // The profiler automatically marks the request as Failed
                LogInfo($"Authentication request failed (expected): {ex.Message}");
            }

            // Example of a timed-out request (manual)
            try
            {
                Guid requestId = NetworkProfilerBehaviour.StartRequest(RequestType.InventoryAction);
                LogInfo($"Started inventory action (will timeout): {requestId}");

                // Simulate a timeout by not completing the request
                // In a real scenario, you'd have a timeout mechanism
                // Here we'll just mark it as timed out
                NetworkProfilerBehaviour.CompleteRequest(requestId, RequestStatus.TimedOut);
                LogInfo($"Inventory action timed out: {requestId}");
            }
            catch (Exception ex)
            {
                LogError($"Example 3 failed: {ex.Message}");
            }
        }

        #endregion

        #region Example 4: Context Filtering

        /// <summary>
        /// Example 4: Retrieving waterfall data with context filtering
        /// Demonstrates how to get and analyze profiling data
        /// </summary>
        public void Example4_RetrieveWaterfallData()
        {
            LogInfo("=== Example 4: Retrieve Waterfall Data ===");

            try
            {
                // Get all completed requests (Total context)
                WaterfallEntry[] allRequests = NetworkProfilerBehaviour.Instance.GetWaterfall(ProfilerContext.Total);
                LogInfo($"Total requests: {allRequests.Length}");

                // Get Unity-only requests
                WaterfallEntry[] unityRequests = NetworkProfilerBehaviour.Instance.GetWaterfall(ProfilerContext.Unity);
                LogInfo($"Unity requests: {unityRequests.Length}");

                // Get Rust-only requests
                WaterfallEntry[] rustRequests = NetworkProfilerBehaviour.Instance.GetWaterfall(ProfilerContext.Rust);
                LogInfo($"Rust requests: {rustRequests.Length}");

                // Analyze the data
                AnalyzeWaterfallData(allRequests);
            }
            catch (Exception ex)
            {
                LogError($"Example 4 failed: {ex.Message}");
            }
        }

        /// <summary>
        /// Analyze waterfall data and extract statistics
        /// </summary>
        private void AnalyzeWaterfallData(WaterfallEntry[] entries)
        {
            if (entries.Length == 0)
            {
                LogInfo("No requests to analyze");
                return;
            }

            // Calculate statistics
            int completedCount = 0;
            int failedCount = 0;
            float totalDuration = 0;
            float maxDuration = 0;
            float minDuration = float.MaxValue;

            foreach (var entry in entries)
            {
                if (entry.status == RequestStatus.Completed)
                {
                    completedCount++;
                    totalDuration += entry.total_duration_ms;
                    maxDuration = Mathf.Max(maxDuration, entry.total_duration_ms);
                    minDuration = Mathf.Min(minDuration, entry.total_duration_ms);
                }
                else if (entry.status == RequestStatus.Failed)
                {
                    failedCount++;
                }
            }

            float avgDuration = completedCount > 0 ? totalDuration / completedCount : 0;

            LogInfo($"Statistics:");
            LogInfo($"  Completed: {completedCount}");
            LogInfo($"  Failed: {failedCount}");
            LogInfo($"  Avg Duration: {avgDuration:F2} ms");
            LogInfo($"  Max Duration: {maxDuration:F2} ms");
            LogInfo($"  Min Duration: {(minDuration < float.MaxValue ? minDuration.ToString("F2") : "N/A")} ms");
        }

        #endregion

        #region Example 5: Integration with Existing Network Code

        /// <summary>
        /// Example 5: Shows how to integrate profiling into existing network code
        /// This demonstrates a pattern you can use throughout your codebase
        /// </summary>
        public void Example5_NetworkIntegration()
        {
            LogInfo("=== Example 5: Network Integration ===");

            // Example: Profile a movement command
            SendMovementCommand(new Vector3(10, 0, 20));

            // Example: Profile a shop purchase
            PurchaseItem("sword", 100);
        }

        /// <summary>
        /// Send movement command with profiling
        /// </summary>
        private void SendMovementCommand(Vector3 targetPosition)
        {
            NetworkProfilerBehaviour.Track(RequestType.MoveCommand, requestId =>
            {
                // Your existing movement code here
                LogInfo($"Sending movement command to {targetPosition}");

                // Convert to your network protocol and send
                byte[] packet = CreateMovementPacket(targetPosition);
                SendToServer(packet);

                LogInfo($"Movement command sent: {requestId}");
            });
        }

        /// <summary>
        /// Purchase item with profiling
        /// </summary>
        private void PurchaseItem(string itemId, int price)
        {
            NetworkProfilerBehaviour.Track(RequestType.ShopAction, requestId =>
            {
                LogInfo($"Purchasing item {itemId} for {price} gold");

                // Your existing shop code here
                byte[] packet = CreatePurchasePacket(itemId, price);
                SendToServer(packet);

                LogInfo($"Purchase request sent: {requestId}");
            });
        }

        #endregion

        #region Example 6: Advanced Stage Tracking

        /// <summary>
        /// Example 6: Advanced stage tracking for complex requests
        /// Demonstrates tracking all stages of a request lifecycle
        /// </summary>
        public void Example6_AdvancedStageTracking()
        {
            LogInfo("=== Example 6: Advanced Stage Tracking ===");

            Guid requestId = NetworkProfilerBehaviour.StartRequest(RequestType.ChatMessage);
            LogInfo($"Started chat message: {requestId}");

            try
            {
                // Stage 1: User Input
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.UserInput);
                string message = GetUserInput();
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.UserInput);

                // Stage 2: Unity Processing
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.UnityProcess);
                ProcessMessageInUnity(message);
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.UnityProcess);

                // Stage 3: FFI Outbound
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.RustFFIOutbound);
                SendToServerViaFFI(message);
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.RustFFIOutbound);

                // Stage 4: Server (simulated)
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.Server);
                // Server processes the message
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.Server);

                // Stage 5: FFI Inbound
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.RustFFIInbound);
                ReceiveFromServerViaFFI();
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.RustFFIInbound);

                // Stage 6: Unity Render
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.UnityRender);
                RenderChatMessage(message);
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.UnityRender);

                // Stage 7: Output
                NetworkProfilerBehaviour.Instance.RecordStageStart(requestId, RequestStage.Output);
                DisplayMessageToUser(message);
                NetworkProfilerBehaviour.Instance.RecordStageEnd(requestId, RequestStage.Output);

                // Complete the request
                NetworkProfilerBehaviour.CompleteRequest(requestId, RequestStatus.Completed);
                LogInfo($"Chat message completed: {requestId}");
            }
            catch (Exception ex)
            {
                NetworkProfilerBehaviour.CompleteRequest(requestId, RequestStatus.Failed);
                LogError($"Chat message failed: {ex.Message}");
            }
        }

        #endregion

        #region Helper Methods

        /// <summary>
        /// Simulate a network call
        /// </summary>
        private void SimulateNetworkCall(string apiName)
        {
            LogInfo($"Calling {apiName}...");
            // Simulate network latency
            System.Threading.Thread.Sleep(UnityEngine.Random.Range(10, 50));
        }

        /// <summary>
        /// Create a movement packet
        /// </summary>
        private byte[] CreateMovementPacket(Vector3 position)
        {
            // Your packet creation logic here
            return new byte[] { 0x01, 0x02, 0x03 };
        }

        /// <summary>
        /// Create a purchase packet
        /// </summary>
        private byte[] CreatePurchasePacket(string itemId, int price)
        {
            // Your packet creation logic here
            return new byte[] { 0x04, 0x05, 0x06 };
        }

        /// <summary>
        /// Send data to server
        /// </summary>
        private void SendToServer(byte[] packet)
        {
            // Your network sending logic here
            LogInfo($"Sending {packet.Length} bytes to server");
        }

        /// <summary>
        /// Get user input
        /// </summary>
        private string GetUserInput()
        {
            // Simulate getting user input
            return "Hello, World!";
        }

        /// <summary>
        /// Process message in Unity
        /// </summary>
        private void ProcessMessageInUnity(string message)
        {
            // Your message processing logic here
            LogInfo($"Processing: {message}");
        }

        /// <summary>
        /// Send to server via FFI
        /// </summary>
        private void SendToServerViaFFI(string message)
        {
            // Your FFI call logic here
            LogInfo($"Sending via FFI: {message}");
        }

        /// <summary>
        /// Receive from server via FFI
        /// </summary>
        private void ReceiveFromServerViaFFI()
        {
            // Your FFI receive logic here
            LogInfo("Receiving via FFI");
        }

        /// <summary>
        /// Render chat message
        /// </summary>
        private void RenderChatMessage(string message)
        {
            // Your rendering logic here
            LogInfo($"Rendering: {message}");
        }

        /// <summary>
        /// Display message to user
        /// </summary>
        private void DisplayMessageToUser(string message)
        {
            // Your display logic here
            LogInfo($"Displaying: {message}");
        }

        /// <summary>
        /// Log info message
        /// </summary>
        private void LogInfo(string message)
        {
            if (showDebugInfo)
            {
                Debug.Log($"[NetworkProfilerExample] {message}");
            }
        }

        /// <summary>
        /// Log error message
        /// </summary>
        private void LogError(string message)
        {
            if (showDebugInfo)
            {
                Debug.LogError($"[NetworkProfilerExample] {message}");
            }
        }

        #endregion

        #region Static Access Methods

        /// <summary>
        /// Static helper to track a movement command
        /// </summary>
        public static void ProfileMovementCommand(Vector3 targetPosition, Action action)
        {
            NetworkProfilerBehaviour.Track(RequestType.MoveCommand, requestId =>
            {
                action();
            });
        }

        /// <summary>
        /// Static helper to track a shop action
        /// </summary>
        public static void ProfileShopAction(Action action)
        {
            NetworkProfilerBehaviour.Track(RequestType.ShopAction, requestId =>
            {
                action();
            });
        }

        /// <summary>
        /// Static helper to track a chat message
        /// </summary>
        public static void ProfileChatMessage(Action action)
        {
            NetworkProfilerBehaviour.Track(RequestType.ChatMessage, requestId =>
            {
                action();
            });
        }

        #endregion
    }
}
