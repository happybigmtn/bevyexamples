//! Showcases how to change the material of a `Scene` spawned from a Gltf
//!
//! # What This Example Demonstrates
//!
//! When you load a 3D model from a GLTF file, it comes with its own materials.
//! Sometimes you want to modify these materials - maybe to create different variants
//! of the same model (red team vs blue team), apply damage effects, or match a
//! specific art style.
//!
//! This example shows how to:
//! 1. Load a GLTF model (Flight Helmet)
//! 2. Spawn multiple instances of it
//! 3. Override the materials on specific instances
//!
//! # Key Concepts
//!
//! - **GLTF**: A standard 3D model format that includes meshes, materials, and animations
//! - **Scene**: A collection of entities loaded from a GLTF file
//! - **Observer**: A Bevy pattern for reacting to events (like scene loading completion)
//! - **Material Override**: Changing materials after they're loaded from the file
//!
//! # Graphics Theory: Material Systems in Modern Engines
//!
//! ## PBR Material Model
//!
//! Modern engines use Physically Based Rendering (PBR) materials:
//! ```text
//! Final Color = Diffuse + Specular + Emission
//! Where:
//!   Diffuse = BaseColor × (1 - Metallic) × Lighting
//!   Specular = Fresnel × Lighting × Roughness
//!   Emission = EmissiveColor × EmissiveStrength
//! ```
//!
//! ## Material Pipeline Architecture
//!
//! ```text
//! GLTF File → Material Loader → GPU Buffers
//!     ↓             ↓              ↓
//! JSON Data    Textures      Shader Parameters
//!     ↓             ↓              ↓
//! Material Asset → Material Instance → Draw Call
//! ```
//!
//! ## Memory Management
//!
//! Material instancing strategies:
//! 1. **Shared Materials**: Multiple meshes → One material (most efficient)
//! 2. **Material Instances**: Base material + per-instance parameters
//! 3. **Unique Materials**: Full copy per mesh (least efficient)
//!
//! This example uses strategy #3 - fine for demos, but consider #2 for production.
//!
//! # Game Design Context: Dynamic Material Systems
//!
//! ## Common Use Cases
//!
//! 1. **Team Colors**: RTS/MOBA games colorize units
//! 2. **Damage States**: Progressive wear, rust, battle damage
//! 3. **Power-ups**: Glowing, pulsing, special effects
//! 4. **Environmental Effects**: Wet surfaces, snow accumulation
//! 5. **Player Customization**: Skins, dyes, cosmetics
//!
//! ## Visual Feedback Patterns
//!
//! Materials communicate game state:
//! - **Health**: Gradual reddening, cracks appearing
//! - **Status Effects**: Frozen = blue tint, Poisoned = green
//! - **Interaction**: Highlight on hover, pulse when activatable
//! - **Rarity**: Material quality indicates item value
//!
//! ## Performance Budgets
//!
//! Typical material counts in games:
//! - **Mobile**: 20-50 unique materials
//! - **Console**: 100-500 unique materials  
//! - **PC High-end**: 500+ unique materials
//!
//! # Performance Deep Dive: Material Optimization
//!
//! ## GPU State Changes
//!
//! Material switches cause pipeline state changes:
//! ```text
//! Cost Hierarchy (cheapest to most expensive):
//! 1. Uniform updates: ~10 cycles
//! 2. Texture binding: ~100 cycles
//! 3. Shader switch: ~1000 cycles
//! 4. Render state: ~10000 cycles
//! ```
//!
//! ## Batching Strategies
//!
//! 1. **Material Sorting**: Group draws by material
//! 2. **Texture Atlasing**: Multiple textures → one bind
//! 3. **Uber Shaders**: One shader with feature flags
//! 4. **GPU Instancing**: Many objects, one draw call
//!
//! ## Memory Bandwidth
//!
//! Material data access patterns:
//! ```text
//! Per-Frame: View/Projection matrices (64 bytes)
//! Per-Material: Textures + parameters (~1KB)
//! Per-Instance: Transform + custom data (64-256 bytes)
//! ```
//!
//! # Real-World Applications: Industry Practices
//!
//! ## AAA Game Examples
//!
//! ### Overwatch
//! - Base hero model + skin system
//! - Materials use masked regions for team colors
//! - Separate materials for weapons, abilities
//!
//! ### Fortnite
//! - Procedural wear/damage on materials
//! - Building materials change based on health
//! - Rarity indicated by material effects
//!
//! ### Destiny 2
//! - Shader system: algorithm + color palette
//! - Materials respond to game lighting dynamically
//! - Exotic weapons have unique material behaviors
//!
//! ## Engine Implementations
//!
//! ### Unreal Engine 5
//! - Material Editor: Node-based system
//! - Material Instances: Cheap parameter overrides
//! - Material Functions: Reusable material logic
//!
//! ### Unity HDRP
//! - Shader Graph: Visual material authoring
//! - Material Property Blocks: Per-renderer overrides
//! - Shader variants: Compile-time feature toggles
//!
//! # Advanced Techniques: Beyond Basic Overrides
//!
//! ## Runtime Material Generation
//!
//! ```rust
//! // Procedural material creation
//! fn create_damage_material(
//!     base_material: &StandardMaterial,
//!     damage_percent: f32,
//! ) -> StandardMaterial {
//!     let mut material = base_material.clone();
//!     
//!     // Darken base color
//!     material.base_color *= 1.0 - (damage_percent * 0.5);
//!     
//!     // Increase roughness (worn surfaces)
//!     material.roughness = f32::min(1.0, material.roughness + damage_percent * 0.3);
//!     
//!     // Add emissive for "hot" damage
//!     if damage_percent > 0.7 {
//!         material.emissive = Color::srgb(1.0, 0.3, 0.1) * (damage_percent - 0.7);
//!     }
//!     
//!     material
//! }
//! ```
//!
//! ## Material Animation
//!
//! Common animated properties:
//! 1. **UV Offset**: Scrolling textures, conveyor belts
//! 2. **Color Lerp**: Smooth transitions, breathing effects
//! 3. **Parameter Curves**: Pulsing emissive, varying roughness
//!
//! ## Shader Parameter Packing
//!
//! Optimize uniform buffer usage:
//! ```glsl
//! // Pack 4 floats into vec4 for alignment
//! uniform MaterialParams {
//!     vec4 tint_and_metallic;      // rgb = tint, a = metallic
//!     vec4 rough_emissive_normal;  // r = roughness, g = emissive, ba = normal scale
//! };
//! ```
//!
//! # Debugging and Profiling: Material Diagnostics
//!
//! ## Debug Visualization Modes
//!
//! 1. **Material ID**: Unique color per material
//! 2. **Batch ID**: Shows draw call batching
//! 3. **Overdraw**: Visualize pixel shader cost
//! 4. **Texture MIPs**: Color by mipmap level
//!
//! ## Performance Profiling
//!
//! Key metrics:
//! - **Draw Calls**: Minimize material switches
//! - **State Changes**: Track pipeline reconfiguration
//! - **Memory Usage**: Monitor texture/buffer allocation
//! - **Shader Complexity**: Instructions per pixel
//!
//! ## Common Issues
//!
//! 1. **Material Leaks**
//!    - Solution: Proper cleanup, reference counting
//!    - Use weak references for caches
//!
//! 2. **Texture Thrashing**
//!    - Solution: Texture atlasing, smaller formats
//!    - Compress textures appropriately
//!
//! 3. **Shader Explosion**
//!    - Solution: Uber shaders, dynamic branching
//!    - Limit permutation count
//!
//! ## Rust-Specific Patterns
//!
//! Leverage Rust's type system for material safety:
//! ```rust
//! // Type-safe material variants
//! enum MaterialVariant {
//!     Standard(Handle<StandardMaterial>),
//!     Damaged { base: Handle<StandardMaterial>, damage: f32 },
//!     TeamColored { base: Handle<StandardMaterial>, team: TeamColor },
//! }
//! ```

use bevy::{
    app::{App, PluginGroup, Startup},
    asset::{AssetServer, Assets},
    audio::AudioPlugin,
    // Color types and predefined color palettes (like CSS colors)
    color::{palettes, Color},
    // GltfAssetLabel helps load specific parts of a GLTF file
    gltf::GltfAssetLabel,
    // Math types for 3D positioning
    math::{Dir3, Vec3},
    // PBR (Physically Based Rendering) components
    pbr::{DirectionalLight, MeshMaterial3d, StandardMaterial},
    prelude::{Camera3d, Children, Commands, Component, Query, Res, ResMut, Transform, Trigger},
    // Scene-related types for loading GLTF models
    scene::{
        SceneInstanceReady, // Event fired when a scene finishes spawning
        SceneRoot,          // Component that marks the root of a loaded scene
    },
    DefaultPlugins,
};

fn main() {
    App::new()
        // Use DefaultPlugins but disable audio (not needed for this example)
        // The .build() method allows customization of the plugin group
        .add_plugins(DefaultPlugins.build().disable::<AudioPlugin>())
        // Setup system runs once at startup
        .add_systems(Startup, setup_scene)
        // Observer listens for SceneInstanceReady events
        // This is triggered when a GLTF scene finishes loading and spawning
        .add_observer(change_material)
        .run();
}

/// This is added to a [`SceneRoot`] and will cause the [`StandardMaterial::base_color`]
/// of all materials to be overwritten
/// 
/// This is a marker component pattern - we attach data (the color) to an entity
/// and use it later to modify behavior. The component holds the color we want
/// to apply to all materials in the scene.
///
/// ## Design Pattern: Marker Components
///
/// This demonstrates several Rust/Bevy patterns:
/// 1. **Newtype Pattern**: Wrapping `Color` gives type safety
/// 2. **Zero-Cost Abstraction**: No runtime overhead vs raw Color
/// 3. **ECS Data Locality**: Components stored in dense arrays
///
/// ## Alternative Designs
///
/// For more complex material modifications:
/// ```rust
/// enum MaterialModification {
///     Tint(Color),
///     Damage { amount: f32, burn_marks: bool },
///     TeamVariant { primary: Color, secondary: Color },
///     Enchantment { glow: Color, intensity: f32 },
/// }
/// ```
#[derive(Component)]
struct ColorOverride(Color);

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a camera positioned to view all three helmets
    commands.spawn((
        Camera3d::default(),
        // Position at (0, 1, 2.5) looking slightly above the origin
        // This gives us a good view of the three helmets side by side
        Transform::from_xyz(0., 1., 2.5).looking_at(Vec3::new(0., 0.25, 0.), Dir3::Y),
    ));

    // Add lighting to the scene
    commands.spawn((
        DirectionalLight::default(),
        // Position the light above and slightly forward, pointing down at the origin
        Transform::from_xyz(0., 1., 0.25).looking_at(Vec3::ZERO, Dir3::Y),
    ));

    // Load the Flight Helmet GLTF model
    // GltfAssetLabel::Scene(0) loads the first (index 0) scene from the GLTF file
    // GLTF files can contain multiple scenes, but most have just one
    //
    // ## GLTF Asset Loading Deep Dive
    //
    // The GLTF format stores:
    // - Scenes: Collections of nodes (objects)
    // - Nodes: Transform hierarchies
    // - Meshes: Geometry data (vertices, indices)
    // - Materials: PBR parameters and texture references
    // - Textures: Image references (usually external files)
    //
    // Loading process:
    // 1. Parse JSON: ~1ms for typical model
    // 2. Load buffers: Binary geometry data
    // 3. Load textures: Async from disk/network
    // 4. Create GPU resources: Upload to VRAM
    //
    // ## Asset Handle System
    //
    // The returned handle is a smart pointer that:
    // - Tracks asset lifetime (reference counting)
    // - Allows async loading (poll for readiness)
    // - Enables hot-reloading in development
    // - Provides weak references for caches
    let flight_helmet = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf"));
    
    // Spawn three instances of the same model:
    //
    // ## Instance Sharing Benefits
    //
    // All three helmets share:
    // - Same mesh data (vertices, indices) in GPU memory
    // - Same textures (only loaded once)
    // - Different materials (due to our color override)
    //
    // Memory usage:
    // - 1 helmet: ~50MB (textures) + 2MB (mesh)
    // - 3 helmets: Still ~52MB total (not 156MB!)
    // - Only materials are duplicated (~1KB each)
    
    // 1. Center helmet - keeps its original materials
    commands.spawn(SceneRoot(flight_helmet.clone()));
    
    // 2. Left helmet - will be tinted red
    commands.spawn((
        SceneRoot(flight_helmet.clone()),
        Transform::from_xyz(-1.25, 0., 0.), // Position to the left
        // Using Tailwind CSS color palette for a nice red shade
        ColorOverride(palettes::tailwind::RED_300.into()),
    ));
    
    // 3. Right helmet - will be tinted green  
    commands.spawn((
        SceneRoot(flight_helmet), // No need to clone for the last use
        Transform::from_xyz(1.25, 0., 0.), // Position to the right
        ColorOverride(palettes::tailwind::GREEN_300.into()),
    ));
}

/// Observer function that runs when a scene finishes spawning
/// 
/// Observers are Bevy's way of reacting to events. This one listens for
/// SceneInstanceReady events, which fire when a GLTF scene has finished
/// loading and all its entities have been spawned.
fn change_material(
    // The trigger contains information about which scene just loaded
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    // Query to traverse the entity hierarchy
    children: Query<&Children>,
    // Query to check if an entity has a ColorOverride component
    color_override: Query<&ColorOverride>,
    // Query to get material handles from mesh entities
    mesh_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    // Access to the material assets so we can modify them
    mut asset_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Get the `ColorOverride` of the entity, if it does not have a color override, skip
    // trigger.target() returns the entity that triggered the event (the SceneRoot)
    let Ok(color_override) = color_override.get(trigger.target()) else {
        // Early return if this scene doesn't have a ColorOverride component
        // This is how the center helmet keeps its original materials
        return;
    };

    // Iterate over all children recursively
    // GLTF scenes have a hierarchy: SceneRoot -> Nodes -> Meshes
    // We need to find all mesh entities within the scene
    //
    // ## Scene Graph Traversal
    //
    // GLTF scene structure:
    // ```
    // SceneRoot
    // ├── Node (Transform)
    // │   ├── Mesh (Primitive 0) + Material 0
    // │   └── Mesh (Primitive 1) + Material 1
    // └── Node (Transform)
    //     └── Mesh + Material 2
    // ```
    //
    // The Flight Helmet model has multiple materials:
    // - Glass visor (transparent)
    // - Metal parts (high metallic)
    // - Leather padding (high roughness)
    // - Plastic components (medium roughness)
    for descendants in children.iter_descendants(trigger.target()) {
        // Try to get the material handle from this entity
        if let Some(material) = mesh_materials
            .get(descendants)
            .ok()
            // Convert the material handle to the actual material asset
            .and_then(|id| asset_materials.get_mut(id.id()))
        {
            // Create a copy of the material and override base color
            // Note: If you're creating many instances with the same tint,
            // it's more efficient to create the tinted material once and reuse it
            //
            // ## Performance Consideration
            //
            // This clones the entire material (including texture handles).
            // For production, consider:
            // 1. Material pooling - pre-create common variants
            // 2. Material instances - share base, override parameters
            // 3. Dynamic batching - group by material to reduce state changes
            //
            // ## GPU Memory Impact
            //
            // Each material clone:
            // - CPU: ~200 bytes (Material struct)
            // - GPU: ~256 bytes (Uniform buffer)
            // - Total with 10 materials: ~4.5KB
            //
            // Textures are NOT cloned, only referenced (handles).
            let mut new_material = material.clone();
            
            // Apply the color override (ColorOverride.0 accesses the Color inside)
            // 
            // ## Color Blending Math
            //
            // The base_color is multiplied with the texture in the shader:
            // ```glsl
            // vec4 final_color = texture(base_color_texture, uv) * base_color;
            // ```
            //
            // This creates a tinting effect, not a replacement.
            // For example:
            // - White base × Red tint = Red
            // - Gray base × Red tint = Dark red
            // - Black base × Any tint = Black
            new_material.base_color = color_override.0;

            // Replace the entity's material with our new tinted version
            //
            // ## ECS Pattern: Component Replacement
            //
            // This demonstrates Bevy's "insert overwrites" behavior:
            // 1. Entity already has MeshMaterial3d<StandardMaterial>
            // 2. We insert a new one
            // 3. Old component is removed, new one added
            // 4. Next frame, renderer uses new material
            //
            // The asset handle system ensures proper cleanup:
            // - Old material's reference count decreases
            // - If it reaches zero, material is freed
            commands
                .entity(descendants)
                .insert(MeshMaterial3d(asset_materials.add(new_material)));
        }
    }
}
