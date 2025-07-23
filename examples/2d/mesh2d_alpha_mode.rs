//! This example is used to test how transforms interact with alpha modes for [`Mesh2d`] entities with a [`MeshMaterial2d`].
//! This makes sure the depth buffer is correctly being used for opaque and transparent 2d meshes
//!
//! # Alpha Modes in 2D: Controlling Transparency
//!
//! Alpha (transparency) determines how objects blend with what's behind them.
//! Different alpha modes handle this in different ways, each with specific
//! use cases and performance characteristics.
//!
//! ## The Three Alpha Modes:
//!
//! ### 1. Opaque (Fastest)
//! - **Behavior**: No transparency at all
//! - **Performance**: Best (uses depth testing)
//! - **Use case**: Solid objects, UI panels
//! - **Depth**: Respects Z-order perfectly
//!
//! ### 2. Mask (Fast)
//! - **Behavior**: Binary on/off transparency
//! - **Threshold**: Pixel alpha > threshold = visible
//! - **Performance**: Good (still uses depth testing)
//! - **Use case**: Pixel art, foliage, text
//!
//! ### 3. Blend (Slowest)
//! - **Behavior**: True alpha blending
//! - **Formula**: result = source × alpha + dest × (1 - alpha)
//! - **Performance**: Slowest (requires sorting)
//! - **Use case**: Glass, particles, fade effects
//!
//! ## Z-Order & Depth Testing:
//!
//! - **Opaque**: Z-position determines draw order
//! - **Mask**: Also uses Z-position (acts like opaque)
//! - **Blend**: Manual sorting required for correct results
//!
//! ## This Example Shows:
//!
//! - Side-by-side comparison of all three modes
//! - How depth buffer affects layering
//! - Visual differences between modes
//! - Performance implications of each choice

// Rust: Selective imports from bevy modules
use bevy::{
    // Rust: Named imports of CSS color constants
    color::palettes::css::{BLUE, GREEN, WHITE},
    // Rust: Common Bevy types
    prelude::*,
    // Rust: Alpha blending modes for 2D sprites
    sprite::AlphaMode2d,
};

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Add standard Bevy plugins
        .add_plugins(DefaultPlugins)
        // Rust: Register startup system
        .add_systems(Startup, setup)
        // Rust: Start game loop
        .run();
}

// Rust: Setup system with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Read-only access to asset server
    asset_server: Res<AssetServer>,
    // Rust: Mutable access to mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to 2D color materials
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Rust: Spawn 2D camera
    commands.spawn(Camera2d);

    // Rust: Load texture for all meshes
    let texture_handle = asset_server.load("branding/icon.png");
    // Rust: Create square mesh (256x256 pixels)
    // Vec2::splat(256.0) creates Vec2 { x: 256.0, y: 256.0 }
    // Rectangle::from_size() creates a rectangular mesh
    let mesh_handle = meshes.add(Rectangle::from_size(Vec2::splat(256.0)));

    // opaque
    // Each sprite should be square with the transparent parts being completely black
    // The blue sprite should be on top with the white and green one behind it
    
    // Rust: White opaque mesh (back layer, Z=0)
    commands.spawn((
        // Rust: .clone() creates a new Handle (reference-counted)
        Mesh2d(mesh_handle.clone()),
        // Rust: Add material to assets and get handle
        MeshMaterial2d(materials.add(ColorMaterial {
            // Rust: CSS color constant converted to Color
            // .into() converts via Into trait
            color: WHITE.into(),
            // Rust: Opaque mode - no transparency, uses depth buffer
            alpha_mode: AlphaMode2d::Opaque,
            // Rust: Option<Handle<Image>> for texture
            // Some() wraps the handle, clone() duplicates handle
            texture: Some(texture_handle.clone()),
            // Rust: Default other fields
            ..default()
        })),
        // Rust: Position at (-400, 0, 0) - leftmost, middle depth
        Transform::from_xyz(-400.0, 0.0, 0.0),
    ));
    
    // Rust: Blue opaque mesh (front layer, Z=1)
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: BLUE.into(),
            // Rust: Opaque mode with higher Z value
            alpha_mode: AlphaMode2d::Opaque,
            texture: Some(texture_handle.clone()),
            ..default()
        })),
        // Rust: Higher Z value (1.0) = closer to camera
        Transform::from_xyz(-300.0, 0.0, 1.0),
    ));
    
    // Rust: Green opaque mesh (behind blue, Z=-1)
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: GREEN.into(),
            alpha_mode: AlphaMode2d::Opaque,
            texture: Some(texture_handle.clone()),
            ..default()
        })),
        // Rust: Lower Z value (-1.0) = farther from camera
        Transform::from_xyz(-200.0, 0.0, -1.0),
    ));

    // Test the interaction between opaque/mask and transparent meshes
    // The white sprite should be:
    // - only the icon is opaque but background is transparent
    // - on top of the green sprite
    // - behind the blue sprite
    
    // Rust: White masked mesh (binary transparency)
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: WHITE.into(),
            // Rust: Mask mode with 0.5 threshold
            // Alpha < 0.5 = transparent, Alpha >= 0.5 = opaque
            // Binary on/off transparency, no blending
            alpha_mode: AlphaMode2d::Mask(0.5),
            texture: Some(texture_handle.clone()),
            ..default()
        })),
        // Rust: Right side, middle depth
        Transform::from_xyz(200.0, 0.0, 0.0),
    ));
    
    // Rust: Blue blended mesh (semi-transparent)
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            // Rust: Method chaining - start with BLUE, set alpha to 0.7
            // with_alpha() returns new Color with modified alpha
            color: BLUE.with_alpha(0.7).into(),
            // Rust: Blend mode - true alpha blending
            // Mixes colors: result = source * alpha + dest * (1 - alpha)
            alpha_mode: AlphaMode2d::Blend,
            texture: Some(texture_handle.clone()),
            ..default()
        })),
        // Rust: Front layer for blended objects
        Transform::from_xyz(300.0, 0.0, 1.0),
    ));
    
    // Rust: Green blended mesh (semi-transparent)
    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(materials.add(ColorMaterial {
            // Rust: 0.7 alpha = 70% opaque, 30% transparent
            color: GREEN.with_alpha(0.7).into(),
            alpha_mode: AlphaMode2d::Blend,
            // Rust: Final use of texture_handle (no .clone() needed)
            // Variable is moved here and consumed
            texture: Some(texture_handle),
            ..default()
        })),
        // Rust: Back layer for blended objects
        Transform::from_xyz(400.0, 0.0, -1.0),
    ));
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Enum Variants with Data**:
//    - `AlphaMode2d::Opaque` - Unit variant (no data)
//    - `AlphaMode2d::Mask(0.5)` - Tuple variant with f32 threshold
//    - `AlphaMode2d::Blend` - Unit variant
//
// 2. **Handle Cloning**:
//    - `Handle<T>` is reference-counted (like Rc/Arc)
//    - `.clone()` creates new reference, not data copy
//    - Cheap operation for sharing assets
//
// 3. **Method Chaining**:
//    - `BLUE.with_alpha(0.7).into()` chains operations
//    - Each method returns new value for next method
//    - Functional programming style
//
// 4. **Option Usage**:
//    - `Some(texture_handle.clone())` wraps Handle
//    - `None` would mean no texture
//    - Pattern matching in rendering code
//
// 5. **Move Semantics**:
//    - Final use of `texture_handle` (no clone)
//    - Variable ownership transferred to function
//    - Prevents accidental reuse after move
//
// 6. **Transform Positioning**:
//    - `from_xyz(x, y, z)` creates positioned Transform
//    - Z-axis determines depth (higher = closer)
//    - X-axis goes left-to-right, Y-axis up-down
//
// 7. **Asset System**:
//    - `materials.add()` stores asset and returns Handle
//    - `asset_server.load()` loads from file system
//    - Handles enable asset sharing and hot reloading
//
// 8. **Alpha Blending Performance**:
//    - Opaque: Fastest (Z-buffer sorting)
//    - Mask: Fast (still uses Z-buffer)
//    - Blend: Slowest (requires manual sorting)
