//! Renders two 3d passes to the same window from different perspectives.
//!
//! # Multi-Pass Rendering: Multiple Views in One Frame
//!
//! Sometimes you need to render the same scene from multiple viewpoints:
//! - **Split-screen games**: Each player has their own camera
//! - **Picture-in-picture**: Mini-map or rear-view mirror
//! - **Security monitors**: Multiple camera feeds
//! - **Artistic effects**: Kaleidoscope or mirror effects
//!
//! ## How Multi-Pass Works:
//!
//! 1. **Render Order**: Cameras render in order (0, 1, 2...)
//! 2. **Clear Behavior**: First camera clears, others overlay
//! 3. **Same Scene**: All cameras see the same world
//! 4. **Performance**: Each camera is a full render pass
//!
//! ## This Example Shows:
//!
//! - Two cameras rendering the same scene
//! - Different perspectives (normal view + bird's eye)
//! - Second camera overlays on first (no clear)
//! - Creates a picture-in-picture effect

// Rust: Import all common Bevy types
use bevy::prelude::*;

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Add standard plugins
        .add_plugins(DefaultPlugins)
        // Rust: Register setup system
        .add_systems(Startup, setup)
        // Rust: Start the game loop
        .run();
}

/// Set up a simple 3D scene
// Rust: System function with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Mutable access to mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to material assets
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    // Rust: Spawn ground plane entity
    commands.spawn((
        // Rust: Create 5x5 plane mesh
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        // Rust: Green material for grass-like ground
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Cube
    commands.spawn((
        // Rust: Default cube is 1x1x1
        Mesh3d(meshes.add(Cuboid::default())),
        // Rust: Beige/tan colored material
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        // Rust: Position cube above ground
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Light
    commands.spawn((
        // Rust: PointLight with custom settings
        PointLight {
            // Rust: Enable shadow casting
            shadows_enabled: true,
            // Rust: Use defaults for other fields
            ..default()
        },
        // Rust: Position light above and to the side
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera - First pass (main view)
    // Rust: Primary camera with default settings
    commands.spawn((
        // Rust: Default Camera3d configuration
        Camera3d::default(),
        // Rust: Position for main view angle
        Transform::from_xyz(-2.0, 2.5, 5.0)  // Slightly elevated
            .looking_at(Vec3::ZERO, Vec3::Y),  // Look at origin
    ));

    // Camera - Second pass (overlay)
    // Rust: Secondary camera for picture-in-picture effect
    commands.spawn((
        Camera3d::default(),
        // Rust: Custom Camera settings for overlay
        Camera {
            // renders after / on top of the main camera
            // Rust: order field controls render sequence
            // Higher numbers render later (on top)
            order: 1,  // Default is 0, so this renders second
            
            // Rust: ClearColorConfig enum controls frame buffer clearing
            // None = don't clear (overlay on previous render)
            clear_color: ClearColorConfig::None,
            
            // Rust: Other fields use defaults
            ..default()
        },
        // Rust: Bird's eye view position
        // Note: 10. is shorthand for 10.0 (f32 literal)
        Transform::from_xyz(10.0, 10., -5.0)  // High above scene
            .looking_at(Vec3::ZERO, Vec3::Y),   // Look down at origin
    ));
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Camera Render Order**:
//    - `order: 0` (default) renders first
//    - `order: 1` renders second (on top)
//    - Can have many cameras with different orders
//
// 2. **ClearColorConfig Enum**:
//    - `Default` - Use global clear color
//    - `Custom(Color)` - Use specific color
//    - `None` - Don't clear (overlay mode)
//
// 3. **Struct Update Syntax**:
//    - Override specific fields
//    - `..default()` fills remaining fields
//    - Common pattern for configuration
//
// 4. **Float Literals**:
//    - `10.0` and `10.` are equivalent
//    - Both create f32 values
//    - Trailing dot is valid Rust syntax
//
// 5. **Multiple Cameras**:
//    - Each camera is a full render pass
//    - Same entities rendered multiple times
//    - Different transforms = different views
//
// 6. **Transform Chaining**:
//    - `from_xyz()` creates with position
//    - `.looking_at()` adds rotation
//    - Fluent API for readability
//
// 7. **Resource Reuse**:
//    - Same meshes/materials for all cameras
//    - Only camera perspective changes
//    - Efficient multi-view rendering
