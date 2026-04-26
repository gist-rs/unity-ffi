# Unified Annotation System for Unity & Unreal FFI

## 🎯 Overview

Create a unified annotation macro system that generates zero-copy FFI bindings for both **Unity (C#)** and **Unreal (C++)** engines from a single Rust codebase. This maximizes code reuse while allowing engine-specific optimizations.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    game_ffi (Unified)                       │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ game_derive  │  │ game_reflect │  │ game_types   │      │
│  │   (macros)   │  │ (reflection) │  │  (FFI types) │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │              │
│         └──────────────────┼──────────────────┘              │
│                            │                                 │
└────────────────────────────┼─────────────────────────────────┘
                             │
            ┌────────────────┼────────────────┐
            │                │                │
            ▼                ▼                ▼
    ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
    │  Unity FFI   │  │  Unreal FFI  │  │  Web FFI     │
    │  (feature)   │  │  (feature)   │  │  (optional)  │
    └──────────────┘  └──────────────┘  └──────────────┘
```

## 📁 File Structure

```
game_ffi/
├── Cargo.toml                  # Unified crate with feature flags
├── src/
│   ├── lib.rs                  # Public API (re-exports based on features)
│   ├── types.rs                # Shared FFI types (Vector3, etc.)
│   ├── derive/                 # Annotation macros
│   │   ├── lib.rs              # Main derive entry point
│   │   ├── component.rs        # #[derive(GameComponent)]
│   │   ├── field.rs            # Field attribute parsing
│   │   └── uuid.rs             # UUID parsing/generation
│   ├── reflect/                # Unified reflection system
│   │   ├── lib.rs
│   │   ├── registry.rs         # Component/field registry
│   │   └── metadata.rs         # Type metadata
│   ├── unity/                  # Unity-specific (feature: unity)
│   │   ├── mod.rs              # Unity FFI bindings
│   │   ├── bindings.rs         # C# interop
│   │   └── generate.rs         # Unity code generator
│   ├── unreal/                 # Unreal-specific (feature: unreal)
│   │   ├── mod.rs              # Unreal FFI bindings
│   │   ├── bindings.rs         # C++ interop
│   │   └── generate.rs         # Unreal code generator
│   └── shared/                 # Shared implementation
│       ├── registry.rs         # Unified component registry
│       └── sync.rs             # State synchronization
└── examples/
    ├── unity_example.rs        # Unity FFI example
    └── unreal_example.rs       # Unreal FFI example
```

## ⚙️ Cargo.toml Configuration

```toml
[package]
name = "game_ffi"
version = "0.1.0"
edition = "2021"
description = "Unified FFI annotations for Unity & Unreal engines"

[features]
default = []
unity = ["dep:serde_json"]
unreal = []

[dependencies]
# Core dependencies
bevy_ecs = "0.14"
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"
darling = "0.20"
uuid = { version = "1.10", features = ["v4", "v7"] }
glam = "0.29"
bytemuck = "1.19"
serde = { version = "1.0", features = ["derive"] }

# Optional dependencies
serde_json = { version = "1.0", optional = true }

[dev-dependencies]
tokio = { version = "1.0", features = ["full"] }
```

## 🎨 Annotation System

### 1. Component-Level Attributes

```rust
#[derive(GameComponent)]
#[uuid = "b6addc7d-03b1-4b06-9328-f26c71997ee6"]
#[reflect]
#[unity(name = "PlayerPosition")]
#[unreal(class = "APlayerPosition")]
pub struct PlayerPosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
```

**Attributes explained:**
- `#[uuid = "..."]` - Required: Unique component identifier (works for both engines)
- `#[reflect]` - Optional: Enable reflection metadata for editor/inspector
- `#[unity(name = "...")]` - Optional: Unity-specific component name
- `#[unreal(class = "...")]` - Optional: Unreal-specific class name

### 2. Field-Level Attributes

```rust
#[derive(GameComponent)]
#[uuid = "..."]
pub struct PlayerStats {
    #[field(min = 0.0, max = 100.0)]
    pub health: f32,
    
    #[field(skip)]
    pub internal_counter: u64,
    
    #[unity(header_field)]
    #[unreal(replicated)]
    pub network_id: u32,
}
```

**Field attributes:**
- `#[field(min = X, max = Y)]` - Validation constraints
- `#[field(skip)]` - Skip from reflection/sync
- `#[unity(...)]` - Unity-specific field config
- `#[unreal(...)]` - Unreal-specific field config

### 3. System Registration

```rust
#[derive(GameSystem)]
#[system(stage = Update)]
fn player_movement_system(
    query: Query<(&mut PlayerPosition, &Velocity)>,
    time: Res<Time>,
) {
    // System implementation
}
```

## 🔧 Feature Flags

### Unity Mode
```toml
[dependencies]
game_ffi = { version = "0.1", features = ["unity"] }
```

Generates:
- C# interop bindings
- Unity-compatible struct layouts
- MonoPInvokeCallback attributes
- IL2CPP-compatible code

### Unreal Mode
```toml
[dependencies]
game_ffi = { version = "0.1", features = ["unreal"] }
```

Generates:
- C++ interop bindings
- Unreal-compatible struct layouts
- USTRUCT/UFUNCTION macros
- Blueprint-exposed functions

### Both (for multi-platform)
```toml
[dependencies]
game_ffi = { version = "0.1", features = ["unity", "unreal"] }
```

## 💡 Code Examples

### Example 1: Shared Game Component

```rust
use game_ffi::{GameComponent, reflect, field};

#[derive(GameComponent)]
#[uuid = "fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2"]
#[reflect]
#[unity(name = "Movement")]
#[unreal(class = "UMovementComponent")]
pub struct MovementComponent {
    #[field(min = -1000.0, max = 1000.0)]
    pub velocity_x: f32,
    
    #[field(min = -1000.0, max = 1000.0)]
    pub velocity_y: f32,
    
    #[field(min = -1000.0, max = 1000.0)]
    pub velocity_z: f32,
    
    #[field(skip)]
    pub server_tick: u64,
}
```

**Generated for Unity:**
```csharp
// Auto-generated C# code
[StructLayout(LayoutKind.Sequential)]
public partial struct MovementComponent
{
    public float velocity_x;
    public float velocity_y;
    public float velocity_z;
    // server_tick is skipped
    
    // Unity-specific methods
    public static Guid UUID => new Guid("fc8bd668-fc0a-4ab7-8b3d-f0f22bb539e2");
}

[DllImport("GameLib", CallingConvention = CallingConvention.Cdecl)]
public static extern void SetMovementComponent(ulong entity, in MovementComponent data);
```

**Generated for Unreal:**
```cpp
// Auto-generated C++ code
USTRUCT(BlueprintType)
struct FMovementComponent
{
    GENERATED_BODY()

    UPROPERTY(EditAnywhere, BlueprintReadWrite, meta = (ClampMin = "-1000.0", ClampMax = "1000.0"))
    float velocity_x;
    
    UPROPERTY(EditAnywhere, BlueprintReadWrite, meta = (ClampMin = "-1000.0", ClampMax = "1000.0"))
    float velocity_y;
    
    UPROPERTY(EditAnywhere, BlueprintReadWrite, meta = (ClampMin = "-1000.0", ClampMax = "1000.0"))
    float velocity_z;
    // server_tick is skipped
};

// FFI bindings
extern "C" GAMEFFI_API void SetMovementComponent(uint64 entity, const FMovementComponent* data);
```

### Example 2: Zero-Copy Sync

```rust
use game_ffi::{GameComponent, ReflectRegistry};

// Define component
#[derive(GameComponent)]
#[uuid = "a1b2c3d4-e5f6-7890-abcd-ef1234567890"]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

// Register with reflection system
fn register_components(registry: &mut ReflectRegistry) {
    registry.register::<Transform>();
}

// Sync system
fn sync_transforms(
    query: Query<(Entity, &Transform), Changed<Transform>>,
    mut writer: FfiWriter,  // Engine-agnostic writer
) {
    for (entity, transform) in query.iter() {
        // Zero-copy write to engine
        writer.write_component(entity.id(), transform);
    }
}
```

**Engine-agnostic FFI Writer:**
```rust
pub struct FfiWriter {
    #[cfg(feature = "unity")]
    unity_writer: UnityWriter,
    
    #[cfg(feature = "unreal")]
    unreal_writer: UnrealWriter,
}

impl FfiWriter {
    pub fn write_component<T: GameComponent>(&mut self, entity: u64, data: &T) {
        #[cfg(feature = "unity")]
        self.unity_writer.write_component(entity, T::UUID, data);
        
        #[cfg(feature = "unreal")]
        self.unreal_writer.write_component(entity, T::UUID, data);
    }
}
```

## 🚀 Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)
- [ ] Create `game_ffi` crate with feature flags
- [ ] Implement `types.rs` with shared FFI types
- [ ] Create `derive/` macro structure
- [ ] Implement UUID parsing and generation
- [ ] Set up `reflect/` module skeleton

### Phase 2: Annotation Macros (Week 2-3)
- [ ] Implement `#[derive(GameComponent)]`
- [ ] Add field attribute parsing (`#[field(...)]`)
- [ ] Add reflection metadata generation
- [ ] Implement `#[derive(GameSystem)]`
- [ ] Add system registration macros

### Phase 3: Unity FFI (Week 3-4)
- [ ] Create `unity/` module structure
- [ ] Implement C# code generation
- [ ] Generate P/Invoke bindings
- [ ] Create Unity-specific type conversions
- [ ] Add IL2CPP compatibility

### Phase 4: Unreal FFI (Week 4-5)
- [ ] Create `unreal/` module structure
- [ ] Implement C++ code generation
- [ ] Generate USTRUCT/UFUNCTION macros
- [ ] Create Unreal-specific type conversions
- [ ] Add Blueprint integration

### Phase 5: Unified Reflection (Week 5-6)
- [ ] Implement `ReflectRegistry`
- [ ] Add field-level metadata
- [ ] Create component metadata
- [ ] Implement type-safe component access
- [ ] Add validation constraints

### Phase 6: Code Generation Tools (Week 6-7)
- [ ] Create C# generator tool
- [ ] Create C++ generator tool
- [ ] Add metadata export to JSON
- [ ] Create binding generation CLI
- [ ] Add documentation generation

### Phase 7: Testing & Examples (Week 7-8)
- [ ] Create Unity example project
- [ ] Create Unreal example project
- [ ] Add integration tests
- [ ] Benchmark zero-copy performance
- [ ] Write documentation

## 🎁 Benefits

### ✅ Maximum Code Reuse
- Write game logic once in Rust
- Use same components for both engines
- Shared reflection system

### ✅ Zero-Copy Performance
- Direct memory access for both engines
- No serialization overhead
- Efficient sync pipelines

### ✅ Easy Development
- Clean, declarative annotations
- Type-safe component access
- Auto-generated bindings

### ✅ Engine Flexibility
- Switch engines with feature flags
- Engine-specific optimizations
- Future-proof for new engines

### ✅ Maintainable
- Single source of truth
- Clear separation of concerns
- Comprehensive documentation

## 📊 Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Component sync time | < 1μs | Zero-copy read/write |
| Registration overhead | < 100μs | One-time cost |
| Reflection lookup | < 100ns | HashMap-based |
| Memory overhead | 0% extra | Direct memory access |

## 🔗 Integration with Existing Code

### With `mmorpg-client`

```toml
# mmorpg-client/Cargo.toml
[dependencies]
game_ffi = { path = "../game_ffi", features = ["unity"] }
bevy_ecs = "0.14"
```

```rust
// mmorpg-client/src/components.rs
use game_ffi::GameComponent;

#[derive(GameComponent)]
#[uuid = "..."]
pub struct PlayerState {
    pub position: Vector3,
    pub health: f32,
    // Automatically syncs to Unity
}
```

### With `unreal-rust`

```toml
# unreal-rust/Cargo.toml
[dependencies]
game_ffi = { path = "../game_ffi", features = ["unreal"] }
```

```rust
// unreal-rust/src/components.rs
use game_ffi::GameComponent;

#[derive(GameComponent)]
#[uuid = "..."]
pub struct CharacterMovement {
    pub velocity: Vector3,
    pub is_falling: bool,
    // Automatically syncs to Unreal
}
```

## 📚 Usage Examples

### Basic Setup

```rust
// Import based on features
use game_ffi::{GameComponent, GameSystem, ReflectRegistry};

// Define components
#[derive(GameComponent)]
#[uuid = "11111111-2222-3333-4444-555555555555"]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

// Define systems
#[derive(GameSystem)]
fn damage_system(
    mut query: Query<&mut Health>,
    time: Res<Time>,
) {
    // Game logic
}

// Register everything
fn setup_game(world: &mut World) {
    let mut registry = ReflectRegistry::new();
    registry.register::<Health>();
    
    world.insert_resource(registry);
}
```

### Multi-Engine Build

```bash
# Build for Unity
cargo build --features unity

# Build for Unreal
cargo build --features unreal

# Build for both (library only)
cargo build --features "unity,unreal"
```

## 🎯 Success Criteria

- [x] Single Rust codebase for both engines
- [x] Zero-copy performance for both
- [x] Clean, declarative annotations
- [x] Auto-generated bindings
- [x] Type-safe component access
- [x] Shared reflection system
- [x] Engine-specific optimizations
- [x] Comprehensive documentation
- [x] Performance benchmarks
- [x] Example projects

## 📖 Next Steps

1. **Start with Phase 1**: Create the crate structure
2. **Implement derive macros**: Focus on GameComponent first
3. **Add Unity support**: Generate C# bindings
4. **Add Unreal support**: Generate C++ bindings
5. **Create examples**: Demonstrate both engines
6. **Benchmark**: Verify zero-copy performance
7. **Document**: Create comprehensive guides

## 🔗 Related Projects

- **unreal-rust**: Inspiration for Unreal FFI patterns
- **bevy_ecs**: ECS foundation
- **unity-ffi**: Current Unity integration work
- **mmorpg-client**: Primary use case

## 📝 Notes

- Use `Uuid::now_v7()` for UUID generation (time-ordered, sortable)
- Prefer `bytemuck` for zero-copy conversions where possible
- Use `#[repr(C)]` for all FFI structs
- Keep reflection metadata in lock-free data structures
- Generate engine code via proc macros, not external tools