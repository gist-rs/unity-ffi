unity-editor \
  -quit \
  -batchmode \
  -nographics \
  -projectPath /path/to/unity/project \
  -executeMethod Unity.BuildTools.BuildAssetBundles.Execute \
  -logFile build.log \
  -buildTarget StandaloneLinux64 \
  -outputDir build/asset_bundles
```

### Unity Editor Menu

From Unity Editor, you can manually trigger builds by opening the script and running the `Execute()` method, or by creating a custom menu item.

## Command Line Arguments

| Argument | Description | Default | Required |
|----------|-------------|---------|----------|
| `-buildTarget` | Target platform (StandaloneLinux64, StandaloneWindows64, StandaloneOSX, WebGL, etc.) | Current active target | No |
| `-outputDir` | Output directory for asset bundles | `build/asset_bundles` | No |
| `-assetBundleVariant` | Asset bundle variant | empty string | No |
| `-compression` | Compression method (none, lzma, lz4, chunkbasedcompression) | `ChunkBasedCompression` | No |
| `-clean` | Clean output directory before building | false | No |
| `-logFile` | Path to build log file | - | No |

## Compression Options

- `none` - Uncompressed bundles
- `lzma` - LZMA compression (highest compression, slower)
- `lz4` - LZ4 compression (fast compression)
- `chunkbasedcompression` - Chunk-based compression (balanced, default)

## Build Targets

Supported build targets:

- `StandaloneLinux64` - Linux 64-bit
- `StandaloneWindows64` - Windows 64-bit
- `StandaloneOSX` - macOS
- `WebGL` - WebGL
- `iOS` - iOS
- `Android` - Android

## Output Structure

```
build/asset_bundles/
├── [bundle_name]/
│   ├── [bundle_name]
│   └── [bundle_name].manifest
└── build_metadata.json
```

### Build Metadata

The `build_metadata.json` file contains:

```json
{
  "buildTime": "2025-02-19T10:30:00.000Z",
  "buildTarget": "StandaloneLinux64",
  "variant": "",
  "compression": "ChunkBasedCompression",
  "totalSize": 1048576,
  "buildTimeSeconds": 12.5,
  "bundles": [
    {
      "name": "example_bundle",
      "size": 262144,
      "roles": []
    }
  ]
}
```

## Examples

### Basic Build for Linux

```bash
unity-editor \
  -quit \
  -batchmode \
  -nographics \
  -projectPath . \
  -executeMethod Unity.BuildTools.BuildAssetBundles.Execute \
  -logFile - \
  -buildTarget StandaloneLinux64
```

### Clean Build with Custom Output

```bash
unity-editor \
  -quit \
  -batchmode \
  -nographics \
  -projectPath . \
  -executeMethod Unity.BuildTools.BuildAssetBundles.Execute \
  -logFile build.log \
  -buildTarget StandaloneWindows64 \
  -outputDir build/asset_bundles/windows \
  -compression lzma \
  -clean true
```

### Build with Variant

```bash
unity-editor \
  -quit \
  -batchmode \
  -nographics \
  -projectPath . \
  -executeMethod Unity.BuildTools.BuildAssetBundles.Execute \
  -buildTarget WebGL \
  -assetBundleVariant hd
```

## CI/CD Integration

The build script is designed to work seamlessly with CI/CD pipelines.

### GitHub Actions Example

```yaml
- name: Build asset bundles
  run: |
    unity-editor \
      -quit \
      -batchmode \
      -nographics \
      -projectPath . \
      -executeMethod Unity.BuildTools.BuildAssetBundles.Execute \
      -logFile - \
      -buildTarget StandaloneLinux64 \
      -outputDir build/asset_bundles
```

## Error Handling

The build script will:

- Exit with code 0 on success
- Exit with code 1 on failure
- Log all errors to console and log file
- Provide detailed error messages and stack traces

### Common Issues

1. **Unity License Not Found**
   - Ensure Unity license is activated for CI/CD
   - Use `UNITY_LICENSE` environment variable

2. **Build Target Not Supported**
   - Verify the build target is valid for your Unity version
   - Check that required modules are installed

3. **Output Directory Permission Denied**
   - Ensure the output directory is writable
   - Check CI/CD runner permissions

4. **No Assets with AssetBundle Labels**
   - Ensure assets in your project have asset bundle labels set
   - Check AssetBundleBrowser for configuration

## License

See project LICENSE file.

## Support

For issues or questions, please refer to the main project documentation or create an issue in the repository.