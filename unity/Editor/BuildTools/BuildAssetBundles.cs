// Unity FFI Build Tool for Asset Bundles

using UnityEngine;
using UnityEditor;
using UnityEditor.Build.Reporting;
using System;
using System.IO;
using System.Linq;

namespace Unity.BuildTools
{
    /// <summary>
    /// Unity Editor script for building asset bundles from command line.
    ///
    /// Usage:
    /// unity-editor -quit -batchmode -nographics -projectPath . \
    ///   -executeMethod Unity.BuildTools.BuildAssetBundles.Execute \
    ///   -logFile - \
    ///   -buildTarget StandaloneLinux64 \
    ///   -outputDir build/asset_bundles
    ///
    /// Or with additional options:
    /// unity-editor -quit -batchmode -nographics -projectPath . \
    ///   -executeMethod Unity.BuildTools.BuildAssetBundles.Execute \
    ///   -buildTarget StandaloneLinux64 \
    ///   -outputDir build/asset_bundles \
    ///   -assetBundleVariant default \
    ///   -compression ChunkBasedCompression \
    ///   -clean true
    /// </summary>
    public static class BuildAssetBundles
    {
        private const string DEFAULT_OUTPUT_DIR = "build/asset_bundles";
        private const string DEFAULT_VARIANT = "";
        private const int SUCCESS_EXIT_CODE = 0;
        private const int ERROR_EXIT_CODE = 1;

        /// <summary>
        /// Main entry point for building asset bundles from command line.
        /// Called by Unity's -executeMethod flag.
        /// </summary>
        public static void Execute()
        {
            try
            {
                Debug.Log("=== Unity Build Tool: Starting Asset Bundle Build ===");
                Debug.Log($"Build started at: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");

                // Parse command line arguments
                string outputDir = GetCommandLineArg("-outputDir", DEFAULT_OUTPUT_DIR);
                string buildTargetStr = GetCommandLineArg("-buildTarget", EditorUserBuildSettings.activeBuildTarget.ToString());
                string variant = GetCommandLineArg("-assetBundleVariant", DEFAULT_VARIANT);
                string compressionStr = GetCommandLineArg("-compression", "ChunkBasedCompression");
                bool clean = GetCommandLineBoolArg("-clean", false);

                Debug.Log($"Configuration:");
                Debug.Log($"  Output Directory: {outputDir}");
                Debug.Log($"  Build Target: {buildTargetStr}");
                Debug.Log($"  Variant: {variant ?? "default"}");
                Debug.Log($"  Compression: {compressionStr}");
                Debug.Log($"  Clean Build: {clean}");

                // Parse build target
                BuildTarget buildTarget = ParseBuildTarget(buildTargetStr);

                // Parse compression option
                BuildAssetBundleOptions compressionOption = ParseCompressionOption(compressionStr);

                // Clean output directory if requested
                if (clean)
                {
                    CleanOutputDirectory(outputDir);
                }

                // Ensure output directory exists
                Directory.CreateDirectory(outputDir);

                // Build asset bundles
                BuildReport report = BuildBundles(outputDir, buildTarget, compressionOption);

                // Log results
                LogBuildResults(report);

                // Save build metadata
                SaveBuildMetadata(outputDir, buildTarget, variant, compressionStr, report);

                Debug.Log("=== Asset Bundle Build Completed Successfully ===");
                Debug.Log($"Build finished at: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");

                // Exit with success code
                EditorApplication.Exit(SUCCESS_EXIT_CODE);
            }
            catch (Exception ex)
            {
                Debug.LogError($"=== Asset Bundle Build Failed ===");
                Debug.LogError($"Error: {ex.Message}");
                Debug.LogError($"Stack Trace: {ex.StackTrace}");
                EditorApplication.Exit(ERROR_EXIT_CODE);
            }
        }

        /// <summary>
        /// Parse build target from string.
        /// </summary>
        private static BuildTarget ParseBuildTarget(string targetStr)
        {
            if (Enum.TryParse<BuildTarget>(targetStr, out BuildTarget target))
            {
                return target;
            }

            Debug.LogWarning($"Unknown build target '{targetStr}', using current active target");
            return EditorUserBuildSettings.activeBuildTarget;
        }

        /// <summary>
        /// Parse compression option from string.
        /// </summary>
        private static BuildAssetBundleOptions ParseCompressionOption(string compressionStr)
        {
            BuildAssetBundleOptions options = BuildAssetBundleOptions.ChunkBasedCompression |
                                             BuildAssetBundleOptions.DeterministicAssetBundle;

            switch (compressionStr?.ToLower())
            {
                case "none":
                    options = BuildAssetBundleOptions.UncompressedAssetBundle |
                              BuildAssetBundleOptions.DeterministicAssetBundle;
                    break;
                case "lzma":
                    options = BuildAssetBundleOptions.LZMACompression |
                              BuildAssetBundleOptions.DeterministicAssetBundle;
                    break;
                case "lz4":
                case "standard":
                    options = BuildAssetBundleOptions.UncompressedAssetBundle |
                              BuildAssetBundleOptions.DeterministicAssetBundle;
                    break;
                case "chunkbased":
                case "chunkbasedcompression":
                default:
                    options = BuildAssetBundleOptions.ChunkBasedCompression |
                              BuildAssetBundleOptions.DeterministicAssetBundle;
                    break;
            }

            return options;
        }

        /// <summary>
        /// Clean the output directory.
        /// </summary>
        private static void CleanOutputDirectory(string outputDir)
        {
            Debug.Log($"Cleaning output directory: {outputDir}");

            if (Directory.Exists(outputDir))
            {
                Directory.Delete(outputDir, true);
                Debug.Log("Output directory cleaned");
            }

            // Recreate directory
            Directory.CreateDirectory(outputDir);
        }

        /// <summary>
        /// Build asset bundles for the specified target.
        /// </summary>
        private static BuildReport BuildBundles(
            string outputDir,
            BuildTarget buildTarget,
            BuildAssetBundleOptions options)
        {
            Debug.Log($"Building asset bundles for target: {buildTarget}");
            Debug.Log($"Output directory: {outputDir}");
            Debug.Log($"Options: {options}");

            // Build the asset bundles
            BuildReport report = BuildPipeline.BuildAssetBundles(
                outputDir,
                options,
                buildTarget
            );

            if (report == null)
            {
                throw new InvalidOperationException("BuildPipeline.BuildAssetBundles returned null");
            }

            return report;
        }

        /// <summary>
        /// Log build results to console.
        /// </summary>
        private static void LogBuildResults(BuildReport report)
        {
            Debug.Log("=== Build Results ===");

            if (report.summary.result == BuildResult.Succeeded)
            {
                Debug.Log($"Status: Success");
                Debug.Log($"Total Size: {report.summary.totalSize / (1024.0 * 1024.0):F2} MB");
                Debug.Log($"Total Time: {report.summary.totalTime.TotalSeconds:F2} seconds");
                Debug.Log($"Warnings: {report.summary.totalWarnings}");
                Debug.Log($"Errors: {report.summary.totalErrors}");

                if (report.packedAssets != null)
                {
                    Debug.Log($"Asset Bundles Created: {report.packedAssets.Count()}");

                    foreach (var packedAsset in report.packedAssets)
                    {
                        Debug.Log($"  - {packedAsset.assetBundleName}");
                        Debug.Log($"    Size: {packedAsset.size / 1024.0:F2} KB");
                        Debug.Log($"    Roles: {string.Join(", ", packedAsset.roles)}");
                    }
                }
            }
            else
            {
                throw new InvalidOperationException($"Build failed with result: {report.summary.result}");
            }
        }

        /// <summary>
        /// Save build metadata to JSON file.
        /// </summary>
        private static void SaveBuildMetadata(
            string outputDir,
            BuildTarget buildTarget,
            string variant,
            string compression,
            BuildReport report)
        {
            var metadata = new BuildMetadata
            {
                buildTime = DateTime.UtcNow.ToString("o"),
                buildTarget = buildTarget.ToString(),
                variant = variant,
                compression = compression,
                totalSize = report.summary.totalSize,
                buildTimeSeconds = report.summary.totalTime.TotalSeconds,
                bundles = report.packedAssets?.Select(pa => new BundleInfo
                {
                    name = pa.assetBundleName,
                    size = pa.size,
                    roles = pa.roles?.ToList() ?? new System.Collections.Generic.List<string>()
                }).ToList() ?? new System.Collections.Generic.List<BundleInfo>()
            };

            string metadataPath = Path.Combine(outputDir, "build_metadata.json");
            string metadataJson = JsonUtility.ToJson(metadata, true);
            File.WriteAllText(metadataPath, metadataJson);

            Debug.Log($"Build metadata saved to: {metadataPath}");
        }

        /// <summary>
        /// Get command line argument value.
        /// </summary>
        private static string GetCommandLineArg(string argName, string defaultValue = null)
        {
            string[] args = Environment.GetCommandLineArgs();
            for (int i = 0; i < args.Length; i++)
            {
                if (args[i] == argName && i + 1 < args.Length)
                {
                    return args[i + 1];
                }
            }
            return defaultValue;
        }

        /// <summary>
        /// Get boolean command line argument.
        /// </summary>
        private static bool GetCommandLineBoolArg(string argName, bool defaultValue)
        {
            string[] args = Environment.GetCommandLineArgs();
            for (int i = 0; i < args.Length; i++)
            {
                if (args[i] == argName)
                {
                    return true;
                }
                if (args[i] == argName && i + 1 < args.Length)
                {
                    return bool.TryParse(args[i + 1], out bool result) ? result : defaultValue;
                }
            }
            return defaultValue;
        }

        /// <summary>
        /// Build metadata structure.
        /// </summary>
        [Serializable]
        private class BuildMetadata
        {
            public string buildTime;
            public string buildTarget;
            public string variant;
            public string compression;
            public long totalSize;
            public double buildTimeSeconds;
            public System.Collections.Generic.List<BundleInfo> bundles;
        }

        /// <summary>
        /// Bundle info structure.
        /// </summary>
        [Serializable]
        private class BundleInfo
        {
            public string name;
            public long size;
            public System.Collections.Generic.List<string> roles;
        }
    }
}
