//! Shows how to render to a texture. Useful for mirrors, UI, or exporting images.
//!
//! # Render-to-Texture: Creating Images on the GPU
//!
//! Imagine you have a security camera in a game that shows what's happening
//! in another room. Or a rear-view mirror in a racing game. Or you want to
//! create a minimap. All of these need "render-to-texture" - the ability to
//! render a scene not to the screen, but to an image in memory!
//!
//! ## How Render-to-Texture Works
//!
//! 1. **Create a texture** that can be rendered to (with special flags)
//! 2. **Set up a camera** that renders to this texture instead of screen
//! 3. **Use render layers** to control what each camera sees
//! 4. **Apply the texture** to objects in your main scene
//!
//! ## This Example Creates
//!
//! - An inner cube that rotates (only visible to texture camera)
//! - A texture camera that captures the inner cube
//! - An outer cube that displays the captured texture
//! - A main camera that sees the textured outer cube
//!
//! It's like having a TV screen (outer cube) showing a live feed
//! from a security camera (texture camera) watching a spinning cube!

// Rust: Importing from standard library
// PI constant for rotation calculations (3.14159...)
use std::f32::consts::PI;

// Rust: External crate imports with nested modules
use bevy::{
    // Rust: Glob import - brings all public items into scope
    prelude::*,
    // Rust: Selective imports from render module
    render::{
        // Rust: Deep module path for render assets
        // RenderAssetUsages controls how assets are used on GPU
        render_asset::RenderAssetUsages,
        // Rust: Multiple imports from same module
        render_resource::{
            Extent3d,           // 3D size specification
            TextureDimension,   // 2D vs 3D textures
            TextureFormat,      // Pixel format (RGBA, etc)
            TextureUsages      // How texture can be used
        },
        // Rust: Single import from view module
        // RenderLayers control visibility per camera
        view::RenderLayers,
    },
};

// Rust: Program entry point
fn main() {
    // Rust: Method chaining for builder pattern
    App::new()
        // Rust: Standard Bevy plugins
        .add_plugins(DefaultPlugins)
        // Rust: System registration for startup
        .add_systems(Startup, setup)
        // Rust: Tuple of systems for Update schedule
        // Both run every frame
        .add_systems(Update, (cube_rotator_system, rotator_system))
        // Rust: Consumes App and runs forever
        .run();
}

// Rust: Marker components - empty structs used as tags
// These help us identify specific entities in queries

// Rust: Derive macro generates trait implementations
// #[derive(Component)] implements the Component trait
// This allows the struct to be used in ECS
#[derive(Component)]
// Rust: Unit struct - no fields, zero size at runtime
// Used purely for type-based identification
struct FirstPassCube;  // The cube we render TO the texture

// Rust: Another marker component
#[derive(Component)]
struct MainPassCube;   // The cube that DISPLAYS the texture

// Rust: System function with multiple resource parameters
// Called once at startup to create our scene
fn setup(
    // Rust: Mutable access to spawn entities
    mut commands: Commands,
    // Rust: ResMut<Assets<T>> for mutable asset storage access
    // Assets are GPU resources like meshes, textures, materials
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Rust: Struct literal with named fields
    // Extent3d defines size in 3 dimensions
    let size = Extent3d {
        width: 512,   // Texture width in pixels
        height: 512,  // Texture height in pixels
        // Rust: Struct update syntax
        // Sets depth_or_array_layers to default (1)
        ..default()
    };

    // This is the texture that will be rendered to.
    // Rust: let mut for mutable binding
    let mut image = Image::new_fill(
        size,                          // Size we defined above
        TextureDimension::D2,          // 2D texture (not 3D or array)
        // Rust: Array reference &[T; N]
        // RGBA color: fully transparent black
        &[0, 0, 0, 0],                 
        // Rust: Enum variant for pixel format
        // BGRA with 8 bits per channel, sRGB color space
        TextureFormat::Bgra8UnormSrgb,
        // Rust: Default trait for standard usage
        RenderAssetUsages::default(),
    );
    
    // CRITICAL: Set texture usage flags for render target
    // Rust: Bitwise OR operator | combines flags
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING     // Can be used in shaders
        | TextureUsages::COPY_DST          // Can receive data
        | TextureUsages::RENDER_ATTACHMENT; // Can be rendered to

    // Rust: Method returns Handle<Image>
    // Handles are like smart pointers to GPU resources
    let image_handle = images.add(image);

    // Rust: Creating mesh and material assets
    let cube_handle = meshes.add(
        // Rust: Struct construction with new
        // Creates a 4x4x4 unit cube mesh
        Cuboid::new(4.0, 4.0, 4.0)
    );
    
    // Rust: Creating material with struct literal
    let cube_material_handle = materials.add(StandardMaterial {
        // Rust: Method on Color for sRGB values
        base_color: Color::srgb(0.8, 0.7, 0.6),  // Light brown
        reflectance: 0.02,  // Low reflectance (rough surface)
        unlit: false,       // Affected by lights
        // Rust: Fill remaining fields with defaults
        ..default()
    });

    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    // Rust: Associated function call
    // RenderLayers control what cameras can see what entities
    // Layer 0 is default, layer 1 is our texture render pass
    let first_pass_layer = RenderLayers::layer(1);

    // The cube that will be rendered to the texture.
    // Rust: Spawning entity with tuple of components
    commands.spawn((
        // Rust: Newtype pattern wrapping Handle<Mesh>
        Mesh3d(cube_handle),
        // Rust: Another newtype wrapping Handle<StandardMaterial>
        MeshMaterial3d(cube_material_handle),
        // Rust: Associated function creating Transform
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        // Rust: Marker component (zero-size type)
        FirstPassCube,
        // Rust: Clone trait creates a copy
        // RenderLayers is Copy but using clone() for clarity
        first_pass_layer.clone(),
    ));

    // Light
    // NOTE: we add the light to both layers so it affects both the rendered-to-texture cube, and the cube on which we display the texture
    // Setting the layer to RenderLayers::layer(0) would cause the main view to be lit, but the rendered-to-texture cube to be unlit.
    // Setting the layer to RenderLayers::layer(1) would cause the rendered-to-texture cube to be lit, but the main view to be unlit.
    commands.spawn((
        // Rust: Default trait provides standard values
        PointLight::default(),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        // Rust: Method chaining on RenderLayers
        // layer(0) creates layer 0, with(1) adds layer 1
        // Result: light visible on BOTH layers
        RenderLayers::layer(0).with(1),
    ));

    // First pass camera - renders TO the texture
    commands.spawn((
        Camera3d::default(),
        // Rust: Struct literal with custom render target
        Camera {
            // Rust: Method chaining with type conversion
            // clone() copies the Handle
            // into() converts Handle<Image> to RenderTarget
            target: image_handle.clone().into(),
            // Rust: ClearColor enum with conversion
            // Color::WHITE creates white color
            // into() converts to ClearColorConfig
            clear_color: Color::WHITE.into(),
            ..default()
        },
        // Rust: Transform with chained methods
        Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
            // Rust: Method that modifies self and returns it
            .looking_at(Vec3::ZERO, Vec3::Y),
        // Only renders entities on layer 1
        first_pass_layer,
    ));

    // Rust: Variable for reuse
    let cube_size = 4.0;
    // Rust: Variable shadowing - same name, new value
    // This is a different cube mesh for the outer cube
    let cube_handle = meshes.add(Cuboid::new(cube_size, cube_size, cube_size));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        // Rust: Option<T> enum - Some variant with value
        // This makes the render texture appear on the cube!
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // Main pass cube, with material containing the rendered first pass texture.
    commands.spawn((
        Mesh3d(cube_handle),
        MeshMaterial3d(material_handle),
        // Rust: Chained transform methods
        Transform::from_xyz(0.0, 0.0, 1.5)
            // Rust: Quaternion from axis rotation
            // -PI/5 radians = -36 degrees tilt
            .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
        MainPassCube,
        // NOTE: No RenderLayers means default layer 0
    ));

    // The main pass camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        // NOTE: No RenderLayers means it only sees layer 0 (default)
    ));
}

// Rust: Doc comment with ///
/// Rotates the inner cube (first pass)
// Rust: System function signature
fn rotator_system(
    // Rust: Immutable resource access
    time: Res<Time>, 
    // Rust: Query with mutable component access
    // With<T> filters to entities that have component T
    mut query: Query<&mut Transform, With<FirstPassCube>>
) {
    // Rust: Iterate over query results
    // &mut query gives mutable iterator
    for mut transform in &mut query {
        // Rust: Mutable method calls
        // time.delta_secs() returns f32 seconds since last frame
        transform.rotate_x(1.5 * time.delta_secs());  // Faster rotation
        transform.rotate_z(1.3 * time.delta_secs());  // Different speed for variety
    }
}

/// Rotates the outer cube (main pass)
// Rust: Another system with same pattern
fn cube_rotator_system(
    time: Res<Time>, 
    // Rust: Different query filter - With<MainPassCube>
    mut query: Query<&mut Transform, With<MainPassCube>>
) {
    for mut transform in &mut query {
        // Rust: Different rotation speeds and axes
        transform.rotate_x(1.0 * time.delta_secs());  // Slower than inner
        transform.rotate_y(0.7 * time.delta_secs());  // Y rotation for variety
    }
}

// 🎯 Rust Patterns in This Example:
//
// 1. **Smart Pointers and Handles**:
//    - `Handle<T>` is like `Arc<T>` but for GPU resources
//    - Automatic reference counting
//    - Cheap to clone
//
// 2. **Builder Pattern**:
//    - Transform::from_xyz().with_rotation()
//    - RenderLayers::layer(0).with(1)
//    - Method chaining for ergonomic APIs
//
// 3. **Type Conversions**:
//    - `.into()` for trait-based conversions
//    - Handle<Image> → RenderTarget
//    - Color → ClearColorConfig
//
// 4. **Option Type**:
//    - `Some(value)` for present values
//    - `None` for absent values (not shown)
//    - Rust's null safety
//
// 5. **Marker Types**:
//    - Zero-size types for compile-time tagging
//    - No runtime cost
//    - Type-safe entity identification
//
// 6. **Resource Management**:
//    - Assets stored in typed collections
//    - Handles prevent premature deallocation
//    - GPU resources managed automatically
