using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;
using Unity.Profiler;
using System.Collections;

namespace Unity.Profiler.Tests
{
    /// <summary>
    /// Unit tests for FPS/RAM Profiler functionality
    /// Tests core profiler operations, FFI bridge, and Unity integration
    /// </summary>
    public class FpsRamProfilerTests
    {
        #region Setup/Teardown

        private FpsRamProfiler profiler;

        /// <summary>
        /// Setup before each test
        /// </summary>
        [SetUp]
        public void SetUp()
        {
            profiler = new FpsRamProfiler();
        }

        /// <summary>
        /// Cleanup after each test
        /// </summary>
        [TearDown]
        public void TearDown()
        {
            if (profiler != null && !profiler.IsDisposed)
            {
                profiler.Dispose();
                profiler = null;
            }
        }

        #endregion

        #region Initialization Tests

        [Test]
        public void Test_Profiler_Initialization_Success()
        {
            // Arrange & Act
            var testProfiler = new FpsRamProfiler();

            // Assert
            Assert.IsNotNull(testProfiler, "Profiler should be initialized");
            Assert.IsTrue(testProfiler.IsInitialized, "Profiler should be marked as initialized");
            Assert.IsFalse(testProfiler.IsDisposed, "Profiler should not be disposed after initialization");

            // Cleanup
            testProfiler.Dispose();
        }

        [Test]
        public void Test_Profiler_Dispose_Success()
        {
            // Arrange
            Assert.IsTrue(profiler.IsInitialized, "Profiler should be initialized");

            // Act
            profiler.Dispose();

            // Assert
            Assert.IsTrue(profiler.IsDisposed, "Profiler should be marked as disposed");
            Assert.IsFalse(profiler.IsInitialized, "Profiler should not be initialized after dispose");
        }

        [Test]
        public void Test_Provider_DoubleDispose_NoError()
        {
            // Arrange
            Assert.IsNotNull(profiler, "Profiler should be initialized");

            // Act & Assert - Should not throw
            Assert.DoesNotThrow(() => profiler.Dispose());
            Assert.DoesNotThrow(() => profiler.Dispose());

            Assert.IsTrue(profiler.IsDisposed, "Profiler should be disposed");
        }

        #endregion

        #region Frame Recording Tests

        [Test]
        public void Test_RecordFrame_SingleFrame_Success()
        {
            // Arrange
            float deltaTimeMs = 16.67f; // 60 FPS

            // Act
            bool result = profiler.RecordFrame(deltaTimeMs);

            // Assert
            Assert.IsTrue(result, "Frame recording should succeed");
        }

        [Test]
        public void Test_RecordFrame_MultipleFrames_Success()
        {
            // Arrange
            int frameCount = 100;

            // Act
            bool allSuccess = true;
            for (int i = 0; i < frameCount; i++)
            {
                float deltaTimeMs = 16.67f + (i % 10) * 0.1f; // Varying frame times
                if (!profiler.RecordFrame(deltaTimeMs))
                {
                    allSuccess = false;
                    break;
                }
            }

            // Assert
            Assert.IsTrue(allSuccess, "All frame recordings should succeed");
        }

        [Test]
        public void Test_RecordFrame_DisposedProfiler_ReturnsFalse()
        {
            // Arrange
            profiler.Dispose();

            // Act
            bool result = profiler.RecordFrame(16.67f);

            // Assert
            Assert.IsFalse(result, "Frame recording should fail after dispose");
        }

        [Test]
        public void Test_RecordFrame_ZeroFrameTime_Success()
        {
            // Arrange
            float deltaTimeMs = 0.0f;

            // Act
            bool result = profiler.RecordFrame(deltaTimeMs);

            // Assert
            Assert.IsTrue(result, "Zero frame time should be accepted");
        }

        [Test]
        public void Test_RecordFrame_LargeFrameTime_Success()
        {
            // Arrange
            float deltaTimeMs = 1000.0f; // 1 second frame (spike)

            // Act
            bool result = profiler.RecordFrame(deltaTimeMs);

            // Assert
            Assert.IsTrue(result, "Large frame time should be accepted");
        }

        #endregion

        #region Memory Submission Tests

        [Test]
        public void Test_SubmitUnityMemory_SingleSubmission_Success()
        {
            // Arrange
            float allocatedMb = 512.0f;
            float reservedMb = 1024.0f;
            float monoMb = 256.0f;

            // Act
            bool result = profiler.SubmitUnityMemory(allocatedMb, reservedMb, monoMb);

            // Assert
            Assert.IsTrue(result, "Memory submission should succeed");
        }

        [Test]
        public void Test_SubmitUnityMemory_MultipleSubmissions_Success()
        {
            // Arrange
            int submissionCount = 10;

            // Act
            bool allSuccess = true;
            for (int i = 0; i < submissionCount; i++)
            {
                float allocatedMb = 512.0f + i * 10.0f;
                float reservedMb = 1024.0f + i * 20.0f;
                float monoMb = 256.0f + i * 5.0f;
                if (!profiler.SubmitUnityMemory(allocatedMb, reservedMb, monoMb))
                {
                    allSuccess = false;
                    break;
                }
            }

            // Assert
            Assert.IsTrue(allSuccess, "All memory submissions should succeed");
        }

        [Test]
        public void Test_SubmitUnityMemory_DisposedProfiler_ReturnsFalse()
        {
            // Arrange
            profiler.Dispose();

            // Act
            bool result = profiler.SubmitUnityMemory(512.0f, 1024.0f, 256.0f);

            // Assert
            Assert.IsFalse(result, "Memory submission should fail after dispose");
        }

        [Test]
        public void Test_SubmitUnityMemory_ZeroValues_Success()
        {
            // Arrange
            float allocatedMb = 0.0f;
            float reservedMb = 0.0f;
            float monoMb = 0.0f;

            // Act
            bool result = profiler.SubmitUnityMemory(allocatedMb, reservedMb, monoMb);

            // Assert
            Assert.IsTrue(result, "Zero memory values should be accepted");
        }

        #endregion

        #region FPS Metrics Tests

        [Test]
        public void Test_GetFpsMetrics_NoFrames_ReturnsZero()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;

            // Act
            bool result = profiler.GetFpsMetrics(context, out FpsMetricsGraphy metrics);

            // Assert
            Assert.IsTrue(result, "GetFpsMetrics should succeed");
            Assert.AreEqual(0.0f, metrics.current_fps, "Current FPS should be zero");
            Assert.AreEqual(0.0f, metrics.avg_fps, "Average FPS should be zero");
        }

        [Test]
        public void Test_GetFpsMetrics_AfterRecording_ReturnsValidMetrics()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;
            int frameCount = 60;
            float deltaTimeMs = 16.67f; // 60 FPS

            for (int i = 0; i < frameCount; i++)
            {
                profiler.RecordFrame(deltaTimeMs);
            }

            // Act
            bool result = profiler.GetFpsMetrics(context, out FpsMetricsGraphy metrics);

            // Assert
            Assert.IsTrue(result, "GetFpsMetrics should succeed");
            Assert.Greater(metrics.current_fps, 0.0f, "Current FPS should be greater than zero");
            Assert.Greater(metrics.avg_fps, 0.0f, "Average FPS should be greater than zero");
        }

        [Test]
        public void Test_GetFpsMetrics_AllContexts_ReturnsValidMetrics()
        {
            // Arrange
            int frameCount = 30;
            float deltaTimeMs = 16.67f;

            for (int i = 0; i < frameCount; i++)
            {
                profiler.RecordFrame(deltaTimeMs);
            }

            // Act & Assert
            foreach (ProfilerContext context in System.Enum.GetValues(typeof(ProfilerContext)))
            {
                bool result = profiler.GetFpsMetrics(context, out FpsMetricsGraphy metrics);

                Assert.IsTrue(result, $"GetFpsMetrics should succeed for context {context}");
                Assert.GreaterOrEqual(metrics.avg_fps, 0.0f, $"Average FPS should be valid for context {context}");
            }
        }

        [Test]
        public void Test_GetFpsMetrics_DisposedProfiler_ReturnsFalse()
        {
            // Arrange
            profiler.Dispose();

            // Act
            bool result = profiler.GetFpsMetrics(ProfilerContext.Total, out FpsMetricsGraphy metrics);

            // Assert
            Assert.IsFalse(result, "GetFpsMetrics should fail after dispose");
        }

        #endregion

        #region Memory Metrics Tests

        [Test]
        public void Test_GetMemoryMetrics_NoSubmissions_ReturnsZero()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;

            // Act
            bool result = profiler.GetMemoryMetrics(context, out MemoryMetricsGraphy metrics);

            // Assert
            Assert.IsTrue(result, "GetMemoryMetrics should succeed");
            Assert.AreEqual(0.0f, metrics.allocated_mb, "Allocated memory should be zero");
            Assert.AreEqual(0.0f, metrics.reserved_mb, "Reserved memory should be zero");
            Assert.AreEqual(0.0f, metrics.mono_mb, "Mono memory should be zero");
        }

        [Test]
        public void Test_GetMemoryMetrics_AfterSubmission_ReturnsValidMetrics()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;
            float allocatedMb = 512.0f;
            float reservedMb = 1024.0f;
            float monoMb = 256.0f;

            profiler.SubmitUnityMemory(allocatedMb, reservedMb, monoMb);

            // Act
            bool result = profiler.GetMemoryMetrics(context, out MemoryMetricsGraphy metrics);

            // Assert
            Assert.IsTrue(result, "GetMemoryMetrics should succeed");
            Assert.Greater(metrics.allocated_mb, 0.0f, "Allocated memory should be greater than zero");
            Assert.Greater(metrics.reserved_mb, 0.0f, "Reserved memory should be greater than zero");
            Assert.Greater(metrics.mono_mb, 0.0f, "Mono memory should be greater than zero");
        }

        [Test]
        public void Test_GetMemoryMetrics_AllContexts_ReturnsValidMetrics()
        {
            // Arrange
            profiler.SubmitUnityMemory(512.0f, 1024.0f, 256.0f);

            // Act & Assert
            foreach (ProfilerContext context in System.Enum.GetValues(typeof(ProfilerContext)))
            {
                bool result = profiler.GetMemoryMetrics(context, out MemoryMetricsGraphy metrics);

                Assert.IsTrue(result, $"GetMemoryMetrics should succeed for context {context}");
            }
        }

        [Test]
        public void Test_GetMemoryMetrics_DisposedProfiler_ReturnsFalse()
        {
            // Arrange
            profiler.Dispose();

            // Act
            bool result = profiler.GetMemoryMetrics(ProfilerContext.Total, out MemoryMetricsGraphy metrics);

            // Assert
            Assert.IsFalse(result, "GetMemoryMetrics should fail after dispose");
        }

        #endregion

        #region Graph Data Tests

        [Test]
        public void Test_GetGraphData_NoFrames_ReturnsZeroValues()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;
            int bufferSize = 512;
            float[] buffer = new float[bufferSize];

            ProfilerGraphData graphData = default;
            unsafe
            {
                fixed (float* ptr = buffer)
                {
                    graphData.values = ptr;
                    graphData.length = (uint)bufferSize;
                }
            }

            // Act
            bool result = profiler.GetGraphData(context, ref graphData);

            // Assert
            Assert.IsTrue(result, "GetGraphData should succeed");
            Assert.AreEqual(0.0f, graphData.average, "Average should be zero");
        }

        [Test]
        public void Test_GetGraphData_AfterRecording_ReturnsValidData()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;
            int bufferSize = 512;
            float[] buffer = new float[bufferSize];

            // Record some frames
            for (int i = 0; i < 100; i++)
            {
                profiler.RecordFrame(16.67f);
            }

            ProfilerGraphData graphData = default;
            unsafe
            {
                fixed (float* ptr = buffer)
                {
                    graphData.values = ptr;
                    graphData.length = (uint)bufferSize;
                }
            }

            // Act
            bool result = profiler.GetGraphData(context, ref graphData);

            // Assert
            Assert.IsTrue(result, "GetGraphData should succeed");
            Assert.Greater(graphData.average, 0.0f, "Average should be greater than zero");
            Assert.Greater(graphData.good_threshold, 0.0f, "Good threshold should be set");
            Assert.Greater(graphData.caution_threshold, 0.0f, "Caution threshold should be set");
        }

        [Test]
        public void Test_GetGraphData_NullBuffer_ReturnsFalse()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;
            ProfilerGraphData graphData = default;
            unsafe
            {
                graphData.values = null;
                graphData.length = 512;
            }

            // Act
            bool result = profiler.GetGraphData(context, ref graphData);

            // Assert
            Assert.IsFalse(result, "GetGraphData should fail with null buffer");
        }

        [Test]
        public void Test_GetGraphData_DisposedProfiler_ReturnsFalse()
        {
            // Arrange
            profiler.Dispose();
            ProfilerContext context = ProfilerContext.Total;
            int bufferSize = 512;
            float[] buffer = new float[bufferSize];

            ProfilerGraphData graphData = default;
            unsafe
            {
                fixed (float* ptr = buffer)
                {
                    graphData.values = ptr;
                    graphData.length = (uint)bufferSize;
                }
            }

            // Act
            bool result = profiler.GetGraphData(context, ref graphData);

            // Assert
            Assert.IsFalse(result, "GetGraphData should fail after dispose");
        }

        #endregion

        #region Visibility Control Tests

        [Test]
        public void Test_SetVisibility_True_Succeeds()
        {
            // Arrange
            bool visible = true;

            // Act
            bool result = profiler.SetVisibility(visible);

            // Assert
            Assert.IsTrue(result, "SetVisibility should succeed");
        }

        [Test]
        public void Test_SetVisibility_False_Succeeds()
        {
            // Arrange
            bool visible = false;

            // Act
            bool result = profiler.SetVisibility(visible);

            // Assert
            Assert.IsTrue(result, "SetVisibility should succeed");
        }

        [Test]
        public void Test_SetVisibility_MultipleToggles_Succeeds()
        {
            // Arrange & Act & Assert
            for (int i = 0; i < 10; i++)
            {
                bool visible = (i % 2) == 0;
                bool result = profiler.SetVisibility(visible);
                Assert.IsTrue(result, $"SetVisibility should succeed for toggle {i}");
            }
        }

        [Test]
        public void Test_SetVisibility_DisposedProfiler_ReturnsFalse()
        {
            // Arrange
            profiler.Dispose();

            // Act
            bool result = profiler.SetVisibility(true);

            // Assert
            Assert.IsFalse(result, "SetVisibility should fail after dispose");
        }

        #endregion

        #region Reset Tests

        [Test]
        public void Test_Reset_AfterRecording_Succeeds()
        {
            // Arrange
            for (int i = 0; i < 100; i++)
            {
                profiler.RecordFrame(16.67f);
            }

            // Act
            bool result = profiler.Reset();

            // Assert
            Assert.IsTrue(result, "Reset should succeed");
        }

        [Test]
        public void Test_Reset_DisposedProfiler_ReturnsFalse()
        {
            // Arrange
            profiler.Dispose();

            // Act
            bool result = profiler.Reset();

            // Assert
            Assert.IsFalse(result, "Reset should fail after dispose");
        }

        [Test]
        public void Test_Reset_GetFpsMetricsReturnsZero()
        {
            // Arrange
            for (int i = 0; i < 60; i++)
            {
                profiler.RecordFrame(16.67f);
            }

            // Get metrics before reset
            profiler.GetFpsMetrics(ProfilerContext.Total, out FpsMetricsGraphy beforeMetrics);
            Assert.Greater(beforeMetrics.avg_fps, 0.0f, "Average FPS should be > 0 before reset");

            // Act
            profiler.Reset();

            // Get metrics after reset
            profiler.GetFpsMetrics(ProfilerContext.Total, out FpsMetricsGraphy afterMetrics);

            // Assert
            Assert.AreEqual(0.0f, afterMetrics.avg_fps, "Average FPS should be zero after reset");
            Assert.AreEqual(0.0f, afterMetrics.current_fps, "Current FPS should be zero after reset");
        }

        #endregion

        #region Integration Tests

        [Test]
        public void Test_CompleteWorkflow_FrameAndMemory()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;
            int frameCount = 60;
            float deltaTimeMs = 16.67f;
            float allocatedMb = 512.0f;
            float reservedMb = 1024.0f;
            float monoMb = 256.0f;
            int bufferSize = 512;
            float[] buffer = new float[bufferSize];

            // Act - Record frames
            for (int i = 0; i < frameCount; i++)
            {
                profiler.RecordFrame(deltaTimeMs);
            }

            // Act - Submit memory
            profiler.SubmitUnityMemory(allocatedMb, reservedMb, monoMb);

            // Act - Get FPS metrics
            bool fpsResult = profiler.GetFpsMetrics(context, out FpsMetricsGraphy fpsMetrics);

            // Act - Get memory metrics
            bool memResult = profiler.GetMemoryMetrics(context, out MemoryMetricsGraphy memMetrics);

            // Act - Get graph data
            ProfilerGraphData graphData = default;
            unsafe
            {
                fixed (float* ptr = buffer)
                {
                    graphData.values = ptr;
                    graphData.length = (uint)bufferSize;
                }
            }
            bool graphResult = profiler.GetGraphData(context, ref graphData);

            // Assert
            Assert.IsTrue(fpsResult, "FPS metrics retrieval should succeed");
            Assert.IsTrue(memResult, "Memory metrics retrieval should succeed");
            Assert.IsTrue(graphResult, "Graph data retrieval should succeed");

            Assert.Greater(fpsMetrics.avg_fps, 0.0f, "Average FPS should be > 0");
            Assert.Greater(memMetrics.allocated_mb, 0.0f, "Allocated memory should be > 0");
            Assert.Greater(graphData.average, 0.0f, "Graph average should be > 0");
        }

        [Test]
        public void Test_VaryingFrameTimes_OnePercentLowCalculation()
        {
            // Arrange
            ProfilerContext context = ProfilerContext.Total;
            int normalFrames = 90;
            int spikeFrames = 10;
            float normalFrameTimeMs = 16.67f; // 60 FPS
            float spikeFrameTimeMs = 33.33f; // 30 FPS (spike)

            // Act - Record normal frames
            for (int i = 0; i < normalFrames; i++)
            {
                profiler.RecordFrame(normalFrameTimeMs);
            }

            // Record spike frames (10% of total)
            for (int i = 0; i < spikeFrames; i++)
            {
                profiler.RecordFrame(spikeFrameTimeMs);
            }

            // Get metrics
            profiler.GetFpsMetrics(context, out FpsMetricsGraphy metrics);

            // Assert
            Assert.Greater(metrics.one_percent_low, 0.0f, "1% low FPS should be calculated");
            Assert.Greater(metrics.zero1_percent_low, 0.0f, "0.1% low FPS should be calculated");
            Assert.Less(metrics.one_percent_low, metrics.avg_fps, "1% low should be less than average");
        }

        #endregion
    }
}
