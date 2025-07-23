//! Update a scene from a glTF file, either by spawning the scene as a child of another entity,
//! or by accessing the entities of the scene.
//!
//! # GLTF Scene Manipulation: Working with 3D Models
//!
//! GLTF (GL Transmission Format) is the "JPEG of 3D" - a standard format for
//! 3D models that includes meshes, materials, animations, and more. This example
//! shows how to load and manipulate GLTF scenes at runtime.
//!
//! ## Key Concepts:
//!
//! 1. **Scene Loading**: GLTF files contain entire scenes with hierarchy
//! 2. **Scene Root**: Parent entity that contains all scene entities
//! 3. **Entity Access**: Iterate through scene descendants
//! 4. **Runtime Modification**: Change transforms, materials, etc.
//!
//! ## Common Use Cases:
//!
//! - Loading character models with animations
//! - Importing level geometry from 3D editors
//! - Dynamic scene assembly from multiple files
//! - Runtime customization of imported assets
//!
//! ## This Example Demonstrates:
//!
//! - Loading the same GLTF scene twice
//! - Accessing child entities within a scene
//! - Animating scene entities independently
//! - Using marker components to identify scenes

// Rust: Selective imports from bevy modules
use bevy::{
    // Rust: Import specific type from pbr module
    pbr::DirectionalLightShadowMap,  // Shadow map configuration
    // Rust: Glob import of common types
    prelude::*,
};

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Insert global resource for shadow quality
        // Struct literal with field name
        .insert_resource(DirectionalLightShadowMap { size: 4096 })  // 4K shadow map
        // Rust: Add default plugins
        .add_plugins(DefaultPlugins)
        // Rust: Register systems for different schedules
        .add_systems(Startup, setup)
        .add_systems(Update, move_scene_entities)
        // Rust: Start the game loop
        .run();
}

// Rust: Derive macro generates Component trait implementation
#[derive(Component)]
// Rust: Zero-size marker type (no fields = no memory)
struct MovedScene;  // Tags the scene we want to animate

// Rust: Setup system with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands, 
    // Rust: Asset server for loading files
    asset_server: Res<AssetServer>
) {
    // Rust: Spawn directional light (sun-like)
    commands.spawn((
        // Rust: Position and orientation for light
        Transform::from_xyz(4.0, 25.0, 8.0)  // High above scene
            .looking_at(Vec3::ZERO, Vec3::Y),  // Point at origin
        // Rust: DirectionalLight configuration
        DirectionalLight {
            // Rust: Enable shadow casting
            shadows_enabled: true,
            // Rust: Default other fields
            ..default()
        },
    ));
    
    // Rust: Spawn camera with environment lighting
    commands.spawn((
        // Rust: Default 3D camera
        Camera3d::default(),
        // Rust: Camera position and orientation
        Transform::from_xyz(-0.5, 0.9, 1.5)  // Slightly offset view
            .looking_at(Vec3::new(-0.5, 0.3, 0.0), Vec3::Y),  // Look at helmet
        // Rust: IBL (Image-Based Lighting) configuration
        EnvironmentMapLight {
            // Rust: Load HDR environment maps
            // These provide realistic lighting from all directions
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            // Rust: f32 literal for light intensity
            intensity: 150.0,  // Brightness multiplier
            ..default()
        },
    ));

    // Spawn the scene as a child of this entity at the given transform
    // Rust: First helmet - static position
    commands.spawn((
        // Rust: Position on the left side
        Transform::from_xyz(-1.0, 0.0, 0.0),
        // Rust: SceneRoot component loads and spawns GLTF scene
        SceneRoot(
            // Rust: Complex asset loading with labels
            asset_server
                // Rust: GltfAssetLabel specifies which part to load
                // Scene(0) = first scene in the GLTF file
                .load(GltfAssetLabel::Scene(0)
                    // Rust: from_asset() creates full asset path
                    .from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
    ));

    // Spawn a second scene, and add a tag component to be able to target it later
    // Rust: Second helmet - will be animated
    commands.spawn((
        // Rust: Same GLTF scene loaded again
        SceneRoot(
            asset_server
                .load(GltfAssetLabel::Scene(0)
                    .from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
        // Rust: Marker component to identify this scene
        MovedScene,  // This scene will be animated
    ));
}

// This system will move all entities that are descendants of MovedScene (which will be all entities spawned in the scene)
// Rust: System to animate scene entities
fn move_scene_entities(
    // Rust: Time resource for animation
    time: Res<Time>,
    // Rust: Query for root entities with MovedScene marker
    // Returns Entity IDs, filtered by With<MovedScene>
    moved_scene: Query<Entity, With<MovedScene>>,
    // Rust: Query for parent-child relationships
    children: Query<&Children>,
    // Rust: Mutable query for transforms
    mut transforms: Query<&mut Transform>,
) {
    // Rust: Iterate over all marked scene roots
    for moved_scene_entity in &moved_scene {
        // Rust: Mutable offset for staggered animation
        // f64 literal (0.) converts to f32
        let mut offset = 0.;
        
        // Rust: iter_descendants() recursively visits all children
        // This captures ALL entities in the GLTF scene hierarchy
        for entity in children.iter_descendants(moved_scene_entity) {
            // Rust: if-let pattern for safe component access
            // get_mut() returns Result<Mut<Transform>, QueryEntityError>
            if let Ok(mut transform) = transforms.get_mut(entity) {
                // Rust: Animate position with sine/cosine waves
                transform.translation = Vec3::new(
                    // Rust: X position - offset creates wave effect
                    offset * ops::sin(time.elapsed_secs()) / 20.,
                    // Rust: Y position - keep at zero
                    0.,
                    // Rust: Z position - circular motion
                    ops::cos(time.elapsed_secs()) / 20.,
                );
                // Rust: Increment offset for next entity
                // Creates cascading wave effect through scene
                offset += 0.5;
            }
        }
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **GLTF Asset Labels**:
//    - `Scene(index)` - Load specific scene
//    - `Mesh(index)` - Load specific mesh
//    - `Material(index)` - Load specific material
//    - `from_asset()` builds the full path
//
// 2. **Scene Hierarchies**:
//    - GLTF scenes spawn multiple entities
//    - Parent-child relationships preserved
//    - `iter_descendants()` traverses tree
//
// 3. **Query Filters**:
//    - `With<T>` - Only entities with component T
//    - Returns Entity ID for further operations
//    - Efficient scene identification
//
// 4. **Safe Component Access**:
//    - `get_mut()` returns Result type
//    - `if let Ok()` handles missing components
//    - Prevents panics on partial scenes
//
// 5. **Marker Components**:
//    - Zero-size types for tagging
//    - No runtime memory overhead
//    - Enable targeted queries
//
// 6. **Environment Maps**:
//    - HDR textures for realistic lighting
//    - Diffuse for soft reflections
//    - Specular for sharp reflections
//
// 7. **Animation Patterns**:
//    - Time-based with elapsed_secs()
//    - Sine/cosine for smooth motion
//    - Offset creates wave effects
