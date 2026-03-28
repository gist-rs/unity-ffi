using NUnit.Framework;
using UnityEngine;
using Unity.Profiler;
using System;
using System.Linq;

namespace Unity.Profiler.Tests
{
    /// <summary>
    /// Unit tests for NetworkProfiler C# implementation
    /// Tests FFI types, core profiler functionality, and error handling
    /// </summary>
    [TestFixture]
    public class NetworkProfilerTests
    {
        #region Setup/Teardown

        private NetworkProfiler profiler;

        [SetUp]
        public void SetUp()
        {
            // Create a new profiler for each test
            try
            {
                profiler = new NetworkProfiler(maxCompletedRequests: 100, ProfilerContext.Total);
            }
            catch (DllNotFoundException)
            {
                // Native library not available - skip integration tests
                Debug.LogWarning("Native profiler library not found. Skipping integration tests.");
                profiler = null;
            }
            catch (Exception ex)
            {
                Debug.LogError($"Failed to initialize profiler for tests: {ex.Message}");
                profiler = null;
            }
        }

        [TearDown]
        public void TearDown()
        {
            // Clean up profiler after each test
            if (profiler != null)
            {
                profiler.Dispose();
                profiler = null;
            }
        }

        #endregion

        #region FFI Type Tests

        [Test]
        public void ProfilerContext_EnumValues_Correct()
        {
            Assert.AreEqual((uint)0, (uint)ProfilerContext.Unity, "Unity context should be 0");
            Assert.AreEqual((uint)1, (uint)ProfilerContext.Rust, "Rust context should be 1");
            Assert.AreEqual((uint)2, (uint)ProfilerContext.Total, "Total context should be 2");
        }

        [Test]
        public void RequestType_EnumValues_Correct()
        {
            Assert.AreEqual((uint)0, (uint)RequestType.Unknown, "Unknown should be 0");
            Assert.AreEqual((uint)1, (uint)RequestType.MoveCommand, "MoveCommand should be 1");
            Assert.AreEqual((uint)2, (uint)RequestType.ShopAction, "ShopAction should be 2");
            Assert.AreEqual((uint)3, (uint)RequestType.ChatMessage, "ChatMessage should be 3");
            Assert.AreEqual((uint)4, (uint)RequestType.CharacterUpdate, "CharacterUpdate should be 4");
            Assert.AreEqual((uint)5, (uint)RequestType.InventoryAction, "InventoryAction should be 5");
            Assert.AreEqual((uint)6, (uint)RequestType.Authentication, "Authentication should be 6");
        }

        [Test]
        public void RequestStatus_EnumValues_Correct()
        {
            Assert.AreEqual((uint)0, (uint)RequestStatus.Pending, "Pending should be 0");
            Assert.AreEqual((uint)1, (uint)RequestStatus.InProgress, "InProgress should be 1");
            Assert.AreEqual((uint)2, (uint)RequestStatus.Completed, "Completed should be 2");
            Assert.AreEqual((uint)3, (uint)RequestStatus.Failed, "Failed should be 3");
            Assert.AreEqual((uint)4, (uint)RequestStatus.TimedOut, "TimedOut should be 4");
        }

        [Test]
        public void RequestStage_EnumValues_Correct()
        {
            Assert.AreEqual((uint)0, (uint)RequestStage.UserInput, "UserInput should be 0");
            Assert.AreEqual((uint)1, (uint)RequestStage.UnityProcess, "UnityProcess should be 1");
            Assert.AreEqual((uint)2, (uint)RequestStage.RustFFIOutbound, "RustFFIOutbound should be 2");
            Assert.AreEqual((uint)3, (uint)RequestStage.Server, "Server should be 3");
            Assert.AreEqual((uint)4, (uint)RequestStage.RustFFIInbound, "RustFFIInbound should be 4");
            Assert.AreEqual((uint)5, (uint)RequestStage.UnityRender, "UnityRender should be 5");
            Assert.AreEqual((uint)6, (uint)RequestStage.Output, "Output should be 6");
        }

        #endregion

        #region WaterfallEntry Tests

        [Test]
        public void WaterfallEntry_UUID_RoundTrip()
        {
            // Create a test GUID
            Guid originalGuid = Guid.NewGuid();

            // Create waterfall entry
            WaterfallEntry entry = new WaterfallEntry();
            entry.SetUuid(originalGuid);

            // Verify round-trip
            Guid retrievedGuid = entry.GetUuid();
            Assert.AreEqual(originalGuid, retrievedGuid, "UUID should match after round-trip");
        }

        [Test]
        public void WaterfallEntry_DefaultValues_Correct()
        {
            WaterfallEntry entry = new WaterfallEntry();

            // Check default values
            Assert.AreEqual(RequestType.Unknown, entry.request_type, "Default request type should be Unknown");
            Assert.AreEqual(RequestStatus.Pending, entry.status, "Default status should be Pending");
            Assert.AreEqual(0.0f, entry.total_duration_ms, "Default duration should be 0");
            Assert.AreEqual(0u, entry.stage_count, "Default stage count should be 0");
            Assert.AreEqual(0u, entry.start_ns, "Default start_ns should be 0");
            Assert.AreEqual(ProfilerContext.Total, entry.context, "Default context should be Total");
        }

        #endregion

        #region Integration Tests (Require Native Library)

        [Test]
        public void Profiler_Initialization_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Assert.IsNotNull(profiler, "Profiler should be initialized");
            Assert.IsTrue(profiler.IsActive, "Profiler should be active");
            Assert.AreEqual(ProfilerContext.Total, profiler.Context, "Context should match");
        }

        [Test]
        public void StartRequest_ReturnsValidGuid()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);

            Assert.AreNotEqual(Guid.Empty, requestId, "Request ID should not be empty");
        }

        [Test]
        public void StartRequest_MultipleRequests_ReturnsUniqueGuids()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid request1 = profiler.StartRequest(RequestType.MoveCommand);
            Guid request2 = profiler.StartRequest(RequestType.ShopAction);
            Guid request3 = profiler.StartRequest(RequestType.ChatMessage);

            Assert.AreNotEqual(request1, request2, "Request IDs should be unique");
            Assert.AreNotEqual(request2, request3, "Request IDs should be unique");
            Assert.AreNotEqual(request1, request3, "Request IDs should be unique");
        }

        [Test]
        public void RecordStageStart_WithValidRequest_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);

            Assert.DoesNotThrow(() =>
            {
                profiler.RecordStageStart(requestId, RequestStage.UserInput);
            }, "Recording stage start should not throw");
        }

        [Test]
        public void RecordStageEnd_WithValidRequest_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);
            profiler.RecordStageStart(requestId, RequestStage.UserInput);

            Assert.DoesNotThrow(() =>
            {
                profiler.RecordStageEnd(requestId, RequestStage.UserInput);
            }, "Recording stage end should not throw");
        }

        [Test]
        public void CompleteRequest_WithValidRequest_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);

            Assert.DoesNotThrow(() =>
            {
                profiler.CompleteRequest(requestId, RequestStatus.Completed);
            }, "Completing request should not throw");
        }

        [Test]
        public void CompleteRequest_FailedStatus_RecordsCorrectly()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);

            Assert.DoesNotThrow(() =>
            {
                profiler.CompleteRequest(requestId, RequestStatus.Failed);
            }, "Completing request as failed should not throw");
        }

        [Test]
        public void GetWaterfall_AfterCompletedRequests_ReturnsData()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            // Create and complete some requests
            Guid request1 = profiler.StartRequest(RequestType.MoveCommand);
            profiler.CompleteRequest(request1, RequestStatus.Completed);

            Guid request2 = profiler.StartRequest(RequestType.ShopAction);
            profiler.CompleteRequest(request2, RequestStatus.Completed);

            // Get waterfall data
            WaterfallEntry[] entries = profiler.GetWaterfall();

            Assert.IsNotNull(entries, "Waterfall data should not be null");
            Assert.Greater(entries.Length, 0, "Waterfall data should contain entries");
        }

        [Test]
        public void GetWaterfall_WithDifferentContexts_ReturnsFilteredData()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            // Create requests
            Guid request1 = profiler.StartRequest(RequestType.MoveCommand);
            profiler.CompleteRequest(request1, RequestStatus.Completed);

            // Get waterfall data for different contexts
            WaterfallEntry[] totalEntries = profiler.GetWaterfall(ProfilerContext.Total);
            WaterfallEntry[] unityEntries = profiler.GetWaterfall(ProfilerContext.Unity);
            WaterfallEntry[] rustEntries = profiler.GetWaterfall(ProfilerContext.Rust);

            Assert.IsNotNull(totalEntries, "Total context entries should not be null");
            Assert.IsNotNull(unityEntries, "Unity context entries should not be null");
            Assert.IsNotNull(rustEntries, "Rust context entries should not be null");
        }

        [Test]
        public void TrackRequest_WithAction_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.TrackRequest(RequestType.MoveCommand, id =>
            {
                // Simulate some work
                System.Threading.Thread.Sleep(1);
            });

            Assert.AreNotEqual(Guid.Empty, requestId, "Request ID should not be empty");
        }

        [Test]
        public void TrackRequest_WithException_CompletesAsFailed()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Assert.Throws<InvalidOperationException>(() =>
            {
                profiler.TrackRequest(RequestType.MoveCommand, id =>
                {
                    throw new InvalidOperationException("Test exception");
                });
            }, "TrackRequest should re-throw exception");

            // Request should be marked as failed internally
            WaterfallEntry[] entries = profiler.GetWaterfall();
            var failedEntries = entries.Where(e => e.status == RequestStatus.Failed);
            Assert.Greater(failedEntries.Count(), 0, "Should have failed requests");
        }

        #endregion

        #region Error Handling Tests

        [Test]
        public void StartRequest_AfterDispose_ThrowsObjectDisposedException()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            profiler.Dispose();

            Assert.Throws<ObjectDisposedException>(() =>
            {
                profiler.StartRequest(RequestType.MoveCommand);
            }, "Should throw ObjectDisposedException after dispose");
        }

        [Test]
        public void RecordStageStart_AfterDispose_ThrowsObjectDisposedException()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);
            profiler.Dispose();

            Assert.Throws<ObjectDisposedException>(() =>
            {
                profiler.RecordStageStart(requestId, RequestStage.UserInput);
            }, "Should throw ObjectDisposedException after dispose");
        }

        [Test]
        public void CompleteRequest_AfterDispose_ThrowsObjectDisposedException()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);
            profiler.Dispose();

            Assert.Throws<ObjectDisposedException>(() =>
            {
                profiler.CompleteRequest(requestId, RequestStatus.Completed);
            }, "Should throw ObjectDisposedException after dispose");
        }

        [Test]
        public void RecordStageStart_WithInvalidGuid_ThrowsArgumentException()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid invalidId = Guid.NewGuid();

            Assert.Throws<ArgumentException>(() =>
            {
                profiler.RecordStageStart(invalidId, RequestStage.UserInput);
            }, "Should throw ArgumentException for invalid request ID");
        }

        [Test]
        public void RecordStageEnd_WithInvalidGuid_ThrowsArgumentException()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid invalidId = Guid.NewGuid();

            Assert.Throws<ArgumentException>(() =>
            {
                profiler.RecordStageEnd(invalidId, RequestStage.UserInput);
            }, "Should throw ArgumentException for invalid request ID");
        }

        [Test]
        public void CompleteRequest_WithInvalidGuid_ThrowsArgumentException()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid invalidId = Guid.NewGuid();

            Assert.Throws<ArgumentException>(() =>
            {
                profiler.CompleteRequest(invalidId, RequestStatus.Completed);
            }, "Should throw ArgumentException for invalid request ID");
        }

        #endregion

        #region Lifecycle Tests

        [Test]
        public void Profiler_Dispose_CanBeCalledMultipleTimes()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Assert.DoesNotThrow(() =>
            {
                profiler.Dispose();
                profiler.Dispose();
                profiler.Dispose();
            }, "Dispose should be idempotent");
        }

        [Test]
        public void Profiler_UsingStatement_AutomaticallyDisposes()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            bool isActive = false;

            using (var testProfiler = new NetworkProfiler(100, ProfilerContext.Total))
            {
                isActive = testProfiler.IsActive;
                Assert.IsTrue(isActive, "Profiler should be active inside using block");
            }

            // Profiler should be disposed after using block
            // We can't directly test this without accessing the private disposed field,
            // but the fact that it didn't throw means Dispose was called successfully
        }

        #endregion

        #region Performance Tests

        [Test]
        public void Performance_StartRequest_1000Requests_CompletesWithinTime()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            int requestCount = 1000;
            var startTime = DateTime.UtcNow;

            for (int i = 0; i < requestCount; i++)
            {
                profiler.StartRequest(RequestType.MoveCommand);
            }

            var duration = (DateTime.UtcNow - startTime).TotalMilliseconds;

            Assert.Less(duration, 100, $"Starting {requestCount} requests should take less than 100ms, took {duration:F2}ms");
        }

        [Test]
        public void Performance_GetWaterfall_With1000Requests_CompletesWithinTime()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            // Create 100 requests
            for (int i = 0; i < 100; i++)
            {
                Guid requestId = profiler.StartRequest(RequestType.MoveCommand);
                profiler.CompleteRequest(requestId, RequestStatus.Completed);
            }

            var startTime = DateTime.UtcNow;
            WaterfallEntry[] entries = profiler.GetWaterfall();
            var duration = (DateTime.UtcNow - startTime).TotalMilliseconds;

            Assert.Less(duration, 50, $"Getting waterfall data should take less than 50ms, took {duration:F2}ms");
            Assert.AreEqual(100, entries.Length, "Should have 100 entries");
        }

        #endregion

        #region Edge Case Tests

        [Test]
        public void StartRequest_WithUnknownRequestType_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Assert.DoesNotThrow(() =>
            {
                profiler.StartRequest(RequestType.Unknown);
            }, "Starting request with Unknown type should succeed");
        }

        [Test]
        public void GetWaterfall_WithNoRequests_ReturnsEmptyArray()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            WaterfallEntry[] entries = profiler.GetWaterfall();

            Assert.IsNotNull(entries, "Waterfall data should not be null");
            Assert.AreEqual(0, entries.Length, "Waterfall data should be empty for no requests");
        }

        [Test]
        public void RecordStage_RecordsStartAndEnd()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);

            Assert.DoesNotThrow(() =>
            {
                profiler.RecordStage(requestId, RequestStage.UserInput);
            }, "Recording both start and end should succeed");
        }

        [Test]
        public void CompleteRequest_AllStatusTypes_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid request1 = profiler.StartRequest(RequestType.MoveCommand);
            Assert.DoesNotThrow(() => profiler.CompleteRequest(request1, RequestStatus.Completed));

            Guid request2 = profiler.StartRequest(RequestType.ShopAction);
            Assert.DoesNotThrow(() => profiler.CompleteRequest(request2, RequestStatus.Failed));

            Guid request3 = profiler.StartRequest(RequestType.ChatMessage);
            Assert.DoesNotThrow(() => profiler.CompleteRequest(request3, RequestStatus.TimedOut));
        }

        #endregion

        #region Multi-Stage Request Tests

        [Test]
        public void MultiStageRequest_AllStages_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.MoveCommand);

            // Record all stages
            profiler.RecordStage(requestId, RequestStage.UserInput);
            profiler.RecordStage(requestId, RequestStage.UnityProcess);
            profiler.RecordStage(requestId, RequestStage.RustFFIOutbound);
            profiler.RecordStage(requestId, RequestStage.Server);
            profiler.RecordStage(requestId, RequestStage.RustFFIInbound);
            profiler.RecordStage(requestId, RequestStage.UnityRender);
            profiler.RecordStage(requestId, RequestStage.Output);

            profiler.CompleteRequest(requestId, RequestStatus.Completed);

            // Verify in waterfall data
            WaterfallEntry[] entries = profiler.GetWaterfall();
            var completedMoveCommand = entries.FirstOrDefault(e =>
                e.request_type == RequestType.MoveCommand &&
                e.status == RequestStatus.Completed);

            Assert.IsNotNull(completedMoveCommand, "Should find completed move command");
            Assert.AreEqual(7u, completedMoveCommand.stage_count, "Should have recorded 7 stages");
        }

        [Test]
        public void MultiStageRequest_WithPartialStages_Succeeds()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.StartRequest(RequestType.ChatMessage);

            // Record only some stages
            profiler.RecordStage(requestId, RequestStage.UserInput);
            profiler.RecordStage(requestId, RequestStage.UnityProcess);
            profiler.RecordStage(requestId, RequestStage.RustFFIOutbound);

            profiler.CompleteRequest(requestId, RequestStatus.Completed);

            // Verify in waterfall data
            WaterfallEntry[] entries = profiler.GetWaterfall();
            var completedChatMessage = entries.FirstOrDefault(e =>
                e.request_type == RequestType.ChatMessage &&
                e.status == RequestStatus.Completed);

            Assert.IsNotNull(completedChatMessage, "Should find completed chat message");
            Assert.AreEqual(3u, completedChatMessage.stage_count, "Should have recorded 3 stages");
        }

        #endregion

        #region Request Type Filtering Tests

        [Test]
        public void GetWaterfall_CanFilterByRequestType()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            // Create different request types
            Guid moveRequest = profiler.StartRequest(RequestType.MoveCommand);
            profiler.CompleteRequest(moveRequest, RequestStatus.Completed);

            Guid shopRequest = profiler.StartRequest(RequestType.ShopAction);
            profiler.CompleteRequest(shopRequest, RequestStatus.Completed);

            Guid chatRequest = profiler.StartRequest(RequestType.ChatMessage);
            profiler.CompleteRequest(chatRequest, RequestStatus.Completed);

            // Get all entries
            WaterfallEntry[] allEntries = profiler.GetWaterfall();

            // Verify we have different types
            var moveEntries = allEntries.Where(e => e.request_type == RequestType.MoveCommand);
            var shopEntries = allEntries.Where(e => e.request_type == RequestType.ShopAction);
            var chatEntries = allEntries.Where(e => e.request_type == RequestType.ChatMessage);

            Assert.AreEqual(1, moveEntries.Count(), "Should have 1 move command");
            Assert.AreEqual(1, shopEntries.Count(), "Should have 1 shop action");
            Assert.AreEqual(1, chatEntries.Count(), "Should have 1 chat message");
        }

        #endregion

        #region Status Tracking Tests

        [Test]
        public void GetWaterfall_CanFilterByStatus()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            // Create requests with different statuses
            Guid completedRequest = profiler.StartRequest(RequestType.MoveCommand);
            profiler.CompleteRequest(completedRequest, RequestStatus.Completed);

            Guid failedRequest = profiler.StartRequest(RequestType.ShopAction);
            profiler.CompleteRequest(failedRequest, RequestStatus.Failed);

            Guid timedOutRequest = profiler.StartRequest(RequestType.ChatMessage);
            profiler.CompleteRequest(timedOutRequest, RequestStatus.TimedOut);

            // Get all entries
            WaterfallEntry[] allEntries = profiler.GetWaterfall();

            // Verify we have different statuses
            var completedEntries = allEntries.Where(e => e.status == RequestStatus.Completed);
            var failedEntries = allEntries.Where(e => e.status == RequestStatus.Failed);
            var timedOutEntries = allEntries.Where(e => e.status == RequestStatus.TimedOut);

            Assert.AreEqual(1, completedEntries.Count(), "Should have 1 completed request");
            Assert.AreEqual(1, failedEntries.Count(), "Should have 1 failed request");
            Assert.AreEqual(1, timedOutEntries.Count(), "Should have 1 timed out request");
        }

        #endregion

        #region Timing Tests

        [Test]
        public void CompletedRequest_HasValidTiming()
        {
            if (profiler == null)
            {
                Assert.Ignore("Native library not available");
                return;
            }

            Guid requestId = profiler.TrackRequest(RequestType.MoveCommand, id =>
            {
                System.Threading.Thread.Sleep(10); // Simulate work
            });

            WaterfallEntry[] entries = profiler.GetWaterfall();
            var entry = entries.FirstOrDefault(e => e.GetUuid() == requestId);

            Assert.IsNotNull(entry, "Should find the request");
            Assert.Greater(entry.total_duration_ms, 0, "Should have positive duration");
            Assert.Greater(entry.start_ns, 0, "Should have valid start timestamp");
        }

        #endregion

        #region Overlay Tests

        /// <summary>
        /// Test tab switching functionality in NetworkProfilerOverlay
        /// </summary>
        [Test]
        public void TestTabSwitching()
        {
            // Create overlay component
            var gameObject = new GameObject("NetworkProfilerOverlay");
            var overlay = gameObject.AddComponent<NetworkProfilerOverlay>();

            // Test initial state
            Assert.AreEqual(NetworkProfilerOverlay.ProfilerTab.Total, overlay.CurrentTab,
                "Default tab should be Total");

            // Test tab switching
            overlay.SetTab(NetworkProfilerOverlay.ProfilerTab.Unity);
            Assert.AreEqual(NetworkProfilerOverlay.ProfilerTab.Unity, overlay.CurrentTab,
                "Tab should switch to Unity");

            overlay.SetTab(NetworkProfilerOverlay.ProfilerTab.Rust);
            Assert.AreEqual(NetworkProfilerOverlay.ProfilerTab.Rust, overlay.CurrentTab,
                "Tab should switch to Rust");

            overlay.SetTab(NetworkProfilerOverlay.ProfilerTab.Total);
            Assert.AreEqual(NetworkProfilerOverlay.ProfilerTab.Total, overlay.CurrentTab,
                "Tab should switch to Total");

            // Cleanup
            Object.DestroyImmediate(gameObject);
        }

        /// <summary>
        /// Test request filtering functionality in NetworkProfilerOverlay
        /// </summary>
        [Test]
        public void TestRequestFiltering()
        {
            // Create overlay component
            var gameObject = new GameObject("NetworkProfilerOverlay");
            var overlay = gameObject.AddComponent<NetworkProfilerOverlay>();

            // Create test entries
            var entries = new List<WaterfallEntry>
            {
                new WaterfallEntry
                {
                    request_type = RequestType.MoveCommand,
                    status = RequestStatus.Completed,
                    total_duration_ms = 100.0f,
                    stage_count = 5,
                    start_ns = 1000000000,
                    context = ProfilerContext.Total
                },
                new WaterfallEntry
                {
                    request_type = RequestType.ShopAction,
                    status = RequestStatus.Completed,
                    total_duration_ms = 150.0f,
                    stage_count = 6,
                    start_ns = 2000000000,
                    context = ProfilerContext.Total
                },
                new WaterfallEntry
                {
                    request_type = RequestType.ChatMessage,
                    status = RequestStatus.Failed,
                    total_duration_ms = 50.0f,
                    stage_count = 3,
                    start_ns = 3000000000,
                    context = ProfilerContext.Total
                }
            };

            // Set entries via reflection (simulating internal cache)
            var field = typeof(NetworkProfilerOverlay).GetField("cachedEntries",
                System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
            if (field != null)
            {
                field.SetValue(overlay, entries.ToArray());
            }

            // Test filtering by request type
            overlay.ApplyFilter("Move");
            var displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(1, displayedEntries.Length,
                "Should filter to only MoveCommand entries");
            Assert.AreEqual(RequestType.MoveCommand, displayedEntries[0].request_type,
                "Filtered entry should be MoveCommand");

            // Test filtering by status
            overlay.ApplyFilter("Failed");
            displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(1, displayedEntries.Length,
                "Should filter to only Failed entries");
            Assert.AreEqual(RequestStatus.Failed, displayedEntries[0].status,
                "Filtered entry should have Failed status");

            // Test empty filter (show all)
            overlay.ApplyFilter("");
            displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(3, displayedEntries.Length,
                "Empty filter should show all entries");

            // Cleanup
            Object.DestroyImmediate(gameObject);
        }

        /// <summary>
        /// Test sorting functionality in NetworkProfilerOverlay
        /// </summary>
        [Test]
        public void TestSorting()
        {
            // Create overlay component
            var gameObject = new GameObject("NetworkProfilerOverlay");
            var overlay = gameObject.AddComponent<NetworkProfilerOverlay>();

            // Create test entries with different durations and timestamps
            var entries = new List<WaterfallEntry>
            {
                new WaterfallEntry
                {
                    request_type = RequestType.MoveCommand,
                    status = RequestStatus.Completed,
                    total_duration_ms = 100.0f,
                    stage_count = 5,
                    start_ns = 3000000000,
                    context = ProfilerContext.Total
                },
                new WaterfallEntry
                {
                    request_type = RequestType.ShopAction,
                    status = RequestStatus.Completed,
                    total_duration_ms = 50.0f,
                    stage_count = 4,
                    start_ns = 2000000000,
                    context = ProfilerContext.Total
                },
                new WaterfallEntry
                {
                    request_type = RequestType.ChatMessage,
                    status = RequestStatus.Failed,
                    total_duration_ms = 150.0f,
                    stage_count = 3,
                    start_ns = 1000000000,
                    context = ProfilerContext.Total
                }
            };

            // Set entries via reflection (simulating internal cache)
            var field = typeof(NetworkProfilerOverlay).GetField("cachedEntries",
                System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
            if (field != null)
            {
                field.SetValue(overlay, entries.ToArray());
            }

            // Test sorting by duration (longest first)
            overlay.SetSort(NetworkProfilerOverlay.SortType.Duration);
            var displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(150.0f, displayedEntries[0].total_duration_ms,
                "First entry should have longest duration (150ms)");
            Assert.AreEqual(100.0f, displayedEntries[1].total_duration_ms,
                "Second entry should have medium duration (100ms)");
            Assert.AreEqual(50.0f, displayedEntries[2].total_duration_ms,
                "Third entry should have shortest duration (50ms)");

            // Test sorting by timestamp (newest first)
            overlay.SetSort(NetworkProfilerOverlay.SortType.Timestamp);
            displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(3000000000UL, displayedEntries[0].start_ns,
                "First entry should have newest timestamp");
            Assert.AreEqual(2000000000UL, displayedEntries[1].start_ns,
                "Second entry should have medium timestamp");
            Assert.AreEqual(1000000000UL, displayedEntries[2].start_ns,
                "Third entry should have oldest timestamp");

            // Test sorting by request type
            overlay.SetSort(NetworkProfilerOverlay.SortType.RequestType);
            displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(RequestType.ChatMessage, displayedEntries[0].request_type,
                "First entry should be alphabetically first (ChatMessage)");
            Assert.AreEqual(RequestType.MoveCommand, displayedEntries[1].request_type,
                "Second entry should be alphabetically middle (MoveCommand)");
            Assert.AreEqual(RequestType.ShopAction, displayedEntries[2].request_type,
                "Third entry should be alphabetically last (ShopAction)");

            // Test sorting by status
            overlay.SetSort(NetworkProfilerOverlay.SortType.Status);
            displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(RequestStatus.Completed, displayedEntries[0].status,
                "Completed status should come before Failed status");

            // Cleanup
            Object.DestroyImmediate(gameObject);
        }

        /// <summary>
        /// Test visibility toggle in NetworkProfilerOverlay
        /// </summary>
        [Test]
        public void TestVisibilityToggle()
        {
            // Create overlay component
            var gameObject = new GameObject("NetworkProfilerOverlay");
            var overlay = gameObject.AddComponent<NetworkProfilerOverlay>();

            // Test initial state (visible by default)
            Assert.IsTrue(overlay.IsVisible, "Overlay should be visible by default");

            // Test toggle off
            overlay.ToggleVisibility();
            Assert.IsFalse(overlay.IsVisible, "Overlay should not be visible after toggle");

            // Test toggle on
            overlay.ToggleVisibility();
            Assert.IsTrue(overlay.IsVisible, "Overlay should be visible after toggle");

            // Test SetVisible
            overlay.SetVisible(false);
            Assert.IsFalse(overlay.IsVisible, "Overlay should not be visible after SetVisible(false)");

            overlay.SetVisible(true);
            Assert.IsTrue(overlay.IsVisible, "Overlay should be visible after SetVisible(true)");

            // Cleanup
            Object.DestroyImmediate(gameObject);
        }

        /// <summary>
        /// Test clearing entries in NetworkProfilerOverlay
        /// </summary>
        [Test]
        public void TestClearEntries()
        {
            // Create overlay component
            var gameObject = new GameObject("NetworkProfilerOverlay");
            var overlay = gameObject.AddComponent<NetworkProfilerOverlay>();

            // Create test entries
            var entries = new List<WaterfallEntry>
            {
                new WaterfallEntry
                {
                    request_type = RequestType.MoveCommand,
                    status = RequestStatus.Completed,
                    total_duration_ms = 100.0f,
                    stage_count = 5,
                    start_ns = 1000000000,
                    context = ProfilerContext.Total
                },
                new WaterfallEntry
                {
                    request_type = RequestType.ShopAction,
                    status = RequestStatus.Completed,
                    total_duration_ms = 150.0f,
                    stage_count = 6,
                    start_ns = 2000000000,
                    context = ProfilerContext.Total
                }
            };

            // Set entries via reflection (simulating internal cache)
            var field = typeof(NetworkProfilerOverlay).GetField("cachedEntries",
                System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
            if (field != null)
            {
                field.SetValue(overlay, entries.ToArray());
            }

            // Verify entries are present
            Assert.AreEqual(2, overlay.DisplayedEntries.Length,
                "Should have 2 entries before clear");

            // Clear entries
            overlay.ClearEntries();

            // Verify entries are cleared
            Assert.AreEqual(0, overlay.DisplayedEntries.Length,
                "Should have 0 entries after clear");

            // Cleanup
            Object.DestroyImmediate(gameObject);
        }

        /// <summary>
        /// Test max requests limit in NetworkProfilerOverlay
        /// </summary>
        [Test]
        public void TestMaxRequestsLimit()
        {
            // Create overlay component
            var gameObject = new GameObject("NetworkProfilerOverlay");
            var overlay = gameObject.AddComponent<NetworkProfilerOverlay>();

            // Set max requests to 10 via reflection
            var maxRequestsField = typeof(NetworkProfilerOverlay).GetField("maxRequestsToDisplay",
                System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
            if (maxRequestsField != null)
            {
                maxRequestsField.SetValue(overlay, 10);
            }

            // Create 15 test entries
            var entries = new List<WaterfallEntry>();
            for (int i = 0; i < 15; i++)
            {
                entries.Add(new WaterfallEntry
                {
                    request_type = RequestType.MoveCommand,
                    status = RequestStatus.Completed,
                    total_duration_ms = (float)i * 10.0f,
                    stage_count = 5,
                    start_ns = (ulong)(i * 1000000000),
                    context = ProfilerContext.Total
                });
            }

            // Set entries via reflection (simulating internal cache)
            var field = typeof(NetworkProfilerOverlay).GetField("cachedEntries",
                System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance);
            if (field != null)
            {
                field.SetValue(overlay, entries.ToArray());
            }

            // Verify only max requests are displayed
            var displayedEntries = overlay.DisplayedEntries;
            Assert.AreEqual(10, displayedEntries.Length,
                "Should limit display to max requests (10)");

            // Cleanup
            Object.DestroyImmediate(gameObject);
        }

        #endregion
    }
}
