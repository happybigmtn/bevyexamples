//! This example shows various ways to configure texture materials in 3D.
//!
//! # Textures: Bringing Surfaces to Life
//!
//! Textures are images applied to 3D surfaces, like wrapping paper around a box.
//! They're the difference between a gray cube and a wooden crate, between a
//! white sphere and Earth!
//!
//! ## Key Texture Concepts:
//!
//! 1. **Base Color Texture**: The main color/pattern of the surface
//! 2. **Alpha Blending**: Transparency for see-through effects
//! 3. **Color Modulation**: Tinting textures with colors
//! 4. **UV Mapping**: How 2D images wrap onto 3D surfaces
//!
//! ## This Example Demonstrates:
//!
//! - Loading image files as textures
//! - Applying textures to rectangular meshes
//! - Color modulation (tinting textures red/blue)
//! - Alpha transparency for overlapping effects
//! - Unlit materials (not affected by scene lighting)

// Rust: Import mathematical constant from standard library
use std::f32::consts::PI;

// Rust: Glob import brings all common Bevy types into scope
use bevy::prelude::*;

// Rust: Program entry point - required for all Rust executables
fn main() {
    // Rust: App builder pattern for configuration
    App::new()
        // Rust: Add default Bevy plugins (renderer, window, input, etc.)
        .add_plugins(DefaultPlugins)
        // Rust: Register setup system to run once at startup
        .add_systems(Startup, setup)
        // Rust: Consume App and start the game loop
        .run();
}

/// sets up a scene with textured entities
// Rust: System function with ECS resource parameters
fn setup(
    // Rust: Mutable Commands for spawning entities
    mut commands: Commands,
    // Rust: Read-only resource for loading assets from disk
    asset_server: Res<AssetServer>,
    // Rust: Mutable access to mesh asset storage
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to material asset storage
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // load a texture and retrieve its aspect ratio
    // Rust: load() returns Handle<Image> - a smart pointer to GPU texture
    // The actual loading happens asynchronously in the background
    let texture_handle = asset_server.load("branding/bevy_logo_dark_big.png");
    // Rust: f64 literal (0.25) automatically converts to f32
    let aspect = 0.25;  // Aspect ratio for rectangular mesh

    // create a new quad mesh. this is what we will apply the texture to
    // Rust: f32 literal with decimal point
    let quad_width = 8.0;
    // Rust: Method chaining to create and store mesh
    // Rectangle::new() creates a 2D primitive, add() stores it and returns a Handle
    let quad_handle = meshes.add(Rectangle::new(quad_width, quad_width * aspect));

    // this material renders the texture normally
    // Rust: Create material and store in asset system
    let material_handle = materials.add(StandardMaterial {
        // Rust: Option<Handle<Image>> with Some variant
        // clone() duplicates the Handle (cheap - reference counted)
        base_color_texture: Some(texture_handle.clone()),
        // Rust: Enum variant for transparency mode
        // Blend mode enables alpha transparency
        alpha_mode: AlphaMode::Blend,
        // Rust: bool literal - unlit ignores scene lighting
        unlit: true,
        // Rust: Struct update syntax fills remaining fields with defaults
        ..default()
    });

    // this material modulates the texture to make it red (and slightly transparent)
    // Rust: Another material with color tinting
    let red_material_handle = materials.add(StandardMaterial {
        // Rust: srgba() takes RGBA values in 0.0-1.0 range
        // Alpha 0.5 = 50% transparent
        base_color: Color::srgba(1.0, 0.0, 0.0, 0.5),  // Red tint
        // Rust: Texture multiplied by base_color
        base_color_texture: Some(texture_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // and lets make this one blue! (and also slightly transparent)
    let blue_material_handle = materials.add(StandardMaterial {
        // Rust: Color components as f32 literals
        base_color: Color::srgba(0.0, 0.0, 1.0, 0.5),  // Blue tint
        // Rust: Move texture_handle (no clone - last use)
        base_color_texture: Some(texture_handle),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // textured quad - normal
    // Rust: Spawn entity with tuple of components
    commands.spawn((
        // Rust: Clone mesh handle for reuse
        Mesh3d(quad_handle.clone()),
        // Rust: Normal material (no color tint)
        MeshMaterial3d(material_handle),
        // Rust: Transform with method chaining
        // from_xyz() creates position, with_rotation() adds rotation
        Transform::from_xyz(0.0, 0.0, 1.5)  // Back position
            .with_rotation(Quat::from_rotation_x(-PI / 5.0)),  // Tilt forward
    ));
    
    // textured quad - modulated
    commands.spawn((
        Mesh3d(quad_handle.clone()),
        // Rust: Red-tinted material
        MeshMaterial3d(red_material_handle),
        // Rust: Transform with only rotation (default position at origin)
        Transform::from_rotation(Quat::from_rotation_x(-PI / 5.0)),
    ));
    
    // textured quad - modulated
    commands.spawn((
        // Rust: Move quad_handle (last use - no clone needed)
        Mesh3d(quad_handle),
        // Rust: Blue-tinted material
        MeshMaterial3d(blue_material_handle),
        // Rust: Transform with position and rotation
        Transform::from_xyz(0.0, 0.0, -1.5)  // Front position
            .with_rotation(Quat::from_rotation_x(-PI / 5.0)),
    ));
    
    // camera
    commands.spawn((
        // Rust: Default camera configuration
        Camera3d::default(),
        // Rust: Camera positioned to view all three quads
        Transform::from_xyz(3.0, 5.0, 8.0)  // Offset position
            .looking_at(Vec3::ZERO, Vec3::Y),  // Look at origin
    ));
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Handle Types**:
//    - `Handle<Image>` - Reference to texture asset
//    - `Handle<Mesh>` - Reference to mesh asset
//    - Clone is cheap (reference counted)
//    - Move when possible (last use)
//
// 2. **Option Type**:
//    - `Some(value)` - Has a value
//    - `None` - No value (not used here)
//    - Used for optional material properties
//
// 3. **Color Creation**:
//    - `srgba()` - sRGB color space with alpha
//    - Values from 0.0 to 1.0
//    - Alpha controls transparency
//
// 4. **Material Properties**:
//    - `base_color` - Tints the entire material
//    - `base_color_texture` - Image texture
//    - Color and texture are multiplied together
//
// 5. **Transform Building**:
//    - `from_xyz()` - Create with position
//    - `from_rotation()` - Create with rotation only
//    - `with_rotation()` - Add rotation to existing
//
// 6. **Asset Management**:
//    - `load()` - Start async loading from disk
//    - `add()` - Store in asset collection
//    - Returns Handle for efficient sharing
//
// 7. **Struct Update Syntax**:
//    - `..default()` fills unspecified fields
//    - Reduces boilerplate code
//    - Common pattern in Bevy
