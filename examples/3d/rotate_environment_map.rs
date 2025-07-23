//! Demonstrates how to rotate the skybox and the environment map simultaneously.
//!
//! # Dynamic Environment Lighting
//!
//! In real life, when you turn your head, the world doesn't rotate - you do!
//! But in games, sometimes we want to rotate the entire environment for artistic
//! effects. This example shows how to spin both the skybox (what you see in the
//! background) and the environment lighting (reflections on objects) together.
//!
//! ## Why Rotate Environment Maps?
//!
//! - **Time of day**: Rotate the sun position for sunrise/sunset
//! - **Dramatic effect**: Spinning environments for dream sequences
//! - **Finding the best angle**: Artists tweaking lighting direction
//! - **Dynamic weather**: Rotating storm clouds
//!
//! ## Key Concepts:
//!
//! - **Skybox**: The background image that surrounds your scene
//! - **Environment Map**: Provides realistic reflections and lighting
//! - **Synchronization**: Both must rotate together or it looks wrong!

// Rust: Import from standard library
// std::f32::consts contains mathematical constants
use std::f32::consts::PI;

// Rust: Complex nested imports from external crate
use bevy::{
    // Rust: Deep module path with multiple items
    // color::palettes::css provides CSS color constants
    color::palettes::css::{GOLD, WHITE},
    // Rust: Importing specific enum variant directly
    // This lets us use `AcesFitted` instead of `Tonemapping::AcesFitted`
    core_pipeline::{tonemapping::Tonemapping::AcesFitted, Skybox},
    // Rust: Single import from deep module
    image::ImageLoaderSettings,
    // Rust: Glob import for common types
    prelude::*,
    // Rust: Another deep module import
    render::view::Hdr,
};

// Rust: Doc comment with ///
/// Entry point.
// Rust: pub makes function visible outside module
// main() is special - program entry point
pub fn main() {
    // Rust: Method chaining (builder pattern)
    App::new()
        // Rust: Each method returns Self for chaining
        .add_plugins(DefaultPlugins)
        // Rust: Function pointers as arguments
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_skybox_and_environment_map)
        // Rust: Consumes App, runs forever
        .run();
}

// Rust: Doc comment for function
/// Initializes the scene.
// Rust: Function with multiple parameters
fn setup(
    // Rust: Mutable binding for entity commands
    mut commands: Commands,
    // Rust: ResMut<T> for mutable resource access
    // Assets<T> is a typed asset storage
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Rust: Res<T> for immutable resource access
    asset_server: Res<AssetServer>,
) {
    // Rust: Function call with mutable reference
    // &mut creates a mutable borrow
    let sphere_mesh = create_sphere_mesh(&mut meshes);
    
    // Rust: Multiple function calls with references
    // & for immutable references, &mut for mutable
    spawn_sphere(&mut commands, &mut materials, &asset_server, &sphere_mesh);
    spawn_light(&mut commands);
    spawn_camera(&mut commands, &asset_server);
}

/// Rotate the skybox and the environment map per frame.
// Rust: System function that runs each frame
fn rotate_skybox_and_environment_map(
    // Rust: Query with tuple of mutable components
    // Finds all entities with BOTH Skybox AND EnvironmentMapLight
    mut environments: Query<(&mut Skybox, &mut EnvironmentMapLight)>,
    // Rust: Time resource for frame timing
    time: Res<Time>,
) {
    // Rust: Method call on smart pointer
    // Res<T> auto-derefs to &T
    let now = time.elapsed_secs();  // Total seconds since startup
    
    // Rust: Quaternion creation from axis rotation
    // 0.2 * now creates smooth rotation over time
    let rotation = Quat::from_rotation_y(0.2 * now);
    
    // Rust: Iterating over query results
    // iter_mut() returns mutable references
    for (mut skybox, mut environment_map) in environments.iter_mut() {
        // Rust: Field assignment through mutable reference
        // Both get the same rotation to stay synchronized
        skybox.rotation = rotation;
        environment_map.rotation = rotation;
    }
}

/// Generates a sphere.
// Rust: Function with mutable reference parameter and return type
fn create_sphere_mesh(meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    // We're going to use normal maps, so make sure we've generated tangents, or
    // else the normal maps won't show up.

    // Rust: let mut for mutable local variable
    // Method chaining: new() -> mesh() -> build()
    let mut sphere_mesh = Sphere::new(1.0)  // 1.0 radius
        .mesh()     // Convert to mesh builder
        .build();   // Build the actual mesh
    
    // Rust: Method that returns Result<T, E>
    sphere_mesh
        .generate_tangents()
        // Rust: expect() unwraps Result, panics with message on Err
        // Use expect() when failure is unrecoverable
        .expect("Failed to generate tangents");
    
    // Rust: Method call returns Handle<Mesh>
    // Ownership of sphere_mesh moves into meshes.add()
    meshes.add(sphere_mesh)
}

/// Spawn a regular object with a clearcoat layer. This looks like car paint.
// Rust: Function taking multiple reference parameters
fn spawn_sphere(
    // Rust: Mutable reference to Commands
    commands: &mut Commands,
    // Rust: Mutable reference to asset storage
    materials: &mut Assets<StandardMaterial>,
    // Rust: Immutable reference (read-only)
    asset_server: &AssetServer,
    // Rust: Reference to Handle (smart pointer)
    sphere_mesh: &Handle<Mesh>,
) {
    // Rust: Spawning entity with component tuple
    commands.spawn((
        // Rust: Clone trait creates a copy of Handle
        // Handles are cheap to clone (reference counted)
        Mesh3d(sphere_mesh.clone()),
        // Rust: Complex nested expression
        MeshMaterial3d(materials.add(StandardMaterial {
            // Rust: f32 literals
            clearcoat: 1.0,                      // Full clearcoat effect
            clearcoat_perceptual_roughness: 0.3, // Semi-rough clearcoat
            // Rust: Option<T> with Some variant
            clearcoat_normal_texture: Some(
                // Rust: Method with closure parameter
                asset_server.load_with_settings(
                    "textures/ScratchedGold-Normal.png",
                    // Rust: Closure syntax |params| body
                    // &mut in closure parameter for mutable access
                    |settings: &mut ImageLoaderSettings| {
                        // Rust: Field assignment in closure
                        // Normal maps must NOT be sRGB
                        settings.is_srgb = false
                    },
                )
            ),
            metallic: 0.9,              // Highly metallic
            perceptual_roughness: 0.1,  // Very smooth base layer
            // Rust: Into trait for type conversion
            // GOLD (Color) -> LinearRgba
            base_color: GOLD.into(),
            // Rust: Struct update syntax
            ..default()
        })),
        // Rust: Builder pattern with method chaining
        Transform::from_xyz(0.0, 0.0, 0.0)  // Position at origin
            // Rust: Vec3::splat creates (x, x, x)
            .with_scale(Vec3::splat(1.25)), // Scale uniformly by 1.25
    ));
}

/// Spawns a light.
// Rust: Simple function with one parameter
fn spawn_light(commands: &mut Commands) {
    // Rust: Spawning entity with struct literal
    commands.spawn(PointLight {
        // Rust: Into conversion for Color type
        color: WHITE.into(),     // CSS white to LinearRgba
        // Rust: f32 with decimal point
        // 100,000 lumens (very bright!)
        intensity: 100000.0,
        // Rust: Default for remaining fields
        ..default()
    });
}

/// Spawns a camera with associated skybox and environment map.
// Rust: Function with multiple reference parameters
fn spawn_camera(commands: &mut Commands, asset_server: &AssetServer) {
    // Rust: Method chaining on commands
    commands
        // Rust: spawn returns an EntityCommands for chaining
        .spawn((
            // Rust: Default implementation
            Camera3d::default(),
            // Rust: Unit struct (marker component)
            Hdr,  // High Dynamic Range rendering
            // Rust: Enum with struct variant
            Projection::Perspective(PerspectiveProjection {
                // Rust: Mathematical expression
                // 27 degrees converted to radians
                fov: 27.0 / 180.0 * PI,
                ..default()
            }),
            // Rust: Transform creation
            Transform::from_xyz(0.0, 0.0, 10.0),
            // Rust: Imported enum variant used directly
            AcesFitted,  // Tonemapping algorithm
        ))
        // Rust: Chained method calls on EntityCommands
        .insert(Skybox {
            brightness: 5000.0,
            // Rust: Asset loading returns Handle<Image>
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            ..default()
        })
        // Rust: Another chained insert
        .insert(EnvironmentMapLight {
            // Rust: Multiple asset loads
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2000.0,
            ..default()
        });
}

// 🎯 Key Rust Patterns in This Example:
//
// 1. **Reference Types**:
//    - `&T` - Immutable borrow (read-only)
//    - `&mut T` - Mutable borrow (read-write)
//    - References prevent ownership transfer
//
// 2. **Smart Pointers**:
//    - `Res<T>` - Shared immutable access
//    - `ResMut<T>` - Exclusive mutable access
//    - `Handle<T>` - Reference-counted asset handle
//
// 3. **Closures**:
//    - `|param| expression` - Short form
//    - `|param| { statements }` - Block form
//    - Capture variables from enclosing scope
//
// 4. **Option Type**:
//    - `Some(value)` - Contains a value
//    - `None` - No value (not used here)
//    - Rust's null safety mechanism
//
// 5. **Result Type**:
//    - `expect()` - Unwrap or panic with message
//    - Used for unrecoverable errors
//    - Better than unwrap() for debugging
//
// 6. **Type Conversions**:
//    - `.into()` - Caller determines target type
//    - Requires From/Into trait implementation
//    - Common for color conversions
//
// 7. **Method Chaining**:
//    - Each method returns self or related type
//    - Enables fluent API design
//    - Common in builder patterns
