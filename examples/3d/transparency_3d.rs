//! Demonstrates how to use transparency in 3D.
//! Shows the effects of different blend modes.
//! The `fade_transparency` system smoothly changes the transparency over time.
//!
//! # Transparency: See-Through Materials
//!
//! Transparency in 3D graphics is surprisingly complex! Different techniques
//! work better for different situations:
//!
//! ## Alpha Blend Modes:
//!
//! 1. **Opaque**: No transparency (default)
//! 2. **Mask**: Binary on/off transparency (like cutouts)
//! 3. **Blend**: Smooth transparency (like glass)
//! 4. **AlphaToCoverage**: Step-based transparency (for foliage/hair)
//!
//! ## Common Use Cases:
//!
//! - **Windows/Glass**: Use Blend mode
//! - **Foliage/Hair**: Use AlphaToCoverage for better anti-aliasing
//! - **Decals/Cutouts**: Use Mask mode with texture alpha
//! - **Particle Effects**: Use Blend with additive blending
//!
//! ## Performance Considerations:
//!
//! - Opaque is fastest (no sorting needed)
//! - Mask is fast (early fragment discard)
//! - Blend is slower (requires depth sorting)
//! - AlphaToCoverage is medium (MSAA-based)

// Rust: Selective import from bevy modules
use bevy::{
    // Rust: Import ops module for mathematical operations
    math::ops,  // Contains sin, cos, etc.
    // Rust: Glob import of common types
    prelude::*,
};

// Rust: Program entry point
fn main() {
    // Rust: Builder pattern for app configuration
    App::new()
        // Rust: Add default plugins (renderer, window, etc.)
        .add_plugins(DefaultPlugins)
        // Rust: Register setup system for Startup schedule
        .add_systems(Startup, setup)
        // Rust: Register animation system for Update schedule
        .add_systems(Update, fade_transparency)
        // Rust: Consume app and start game loop
        .run();
}

// Rust: System function for scene setup
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Mutable access to mesh asset storage
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to material asset storage
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Opaque plane, uses `alpha_mode: Opaque` by default
    // Rust: Spawn ground plane entity
    commands.spawn((
        // Rust: Create and store plane mesh
        // Method chain: default() -> mesh() -> size()
        Mesh3d(meshes.add(Plane3d::default().mesh().size(6.0, 6.0))),
        // Rust: Implicit material conversion from Color
        // When Color is used directly, alpha_mode defaults to Opaque
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),  // Green
    ));

    // Transparent sphere, uses `alpha_mode: Mask(f32)`
    commands.spawn((
        // Rust: Create icosphere mesh with subdivision level 3
        // unwrap() because ico() returns Result (can fail if level too high)
        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(3).unwrap())),
        // Rust: Explicit StandardMaterial construction
        MeshMaterial3d(materials.add(StandardMaterial {
            // Alpha channel of the color controls transparency.
            // We set it to 0.0 here, because it will be changed over time in the
            // `fade_transparency` function.
            // Note that the transparency has no effect on the objects shadow.
            // Rust: srgba() includes alpha channel (4th parameter)
            base_color: Color::srgba(0.2, 0.7, 0.1, 0.0),  // Green, fully transparent
            // Mask sets a cutoff for transparency. Alpha values below are fully transparent,
            // alpha values above are fully opaque.
            // Rust: Enum variant with associated f32 value
            alpha_mode: AlphaMode::Mask(0.5),  // Cutoff at 50% alpha
            // Rust: Struct update syntax for remaining fields
            ..default()
        })),
        // Rust: Position in 3D space
        Transform::from_xyz(1.0, 0.5, -1.5),  // Right side
    ));

    // Transparent unlit sphere, uses `alpha_mode: Mask(f32)`
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(3).unwrap())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.2, 0.7, 0.1, 0.0),
            alpha_mode: AlphaMode::Mask(0.5),
            // Rust: bool literal - unlit materials ignore lighting
            unlit: true,  // No shading, constant color
            ..default()
        })),
        // Rust: Left side position
        Transform::from_xyz(-1.0, 0.5, -1.5),
    ));

    // Transparent cube, uses `alpha_mode: Blend`
    commands.spawn((
        // Rust: Default Cuboid is 1x1x1
        Mesh3d(meshes.add(Cuboid::default())),
        // Notice how there is no need to set the `alpha_mode` explicitly here.
        // When converting a color to a material using `into()`, the alpha mode is
        // automatically set to `Blend` if the alpha channel is anything lower than 1.0.
        // Rust: Implicit conversion from Color to StandardMaterial
        // The From trait implementation detects alpha < 1.0 and sets Blend mode
        MeshMaterial3d(materials.add(Color::srgba(0.5, 0.5, 1.0, 0.0))),  // Blue
        // Rust: Center position
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Transparent cube, uses `alpha_mode: AlphaToCoverage`
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            // Rust: Light green color with 0 alpha
            base_color: Color::srgba(0.5, 1.0, 0.5, 0.0),
            // Rust: AlphaToCoverage uses MSAA samples for transparency
            // Creates dithered transparency effect, good for foliage
            alpha_mode: AlphaMode::AlphaToCoverage,
            ..default()
        })),
        // Rust: Left of center
        Transform::from_xyz(-1.5, 0.5, 0.0),
    ));

    // Opaque sphere
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(3).unwrap())),
        // Rust: Opaque material (no alpha specified)
        // When alpha is 1.0 or not specified, mode is Opaque
        MeshMaterial3d(materials.add(Color::srgb(0.7, 0.2, 0.1))),  // Red-orange
        Transform::from_xyz(0.0, 0.5, -1.5),
    ));

    // Light
    commands.spawn((
        // Rust: PointLight component with shadows
        PointLight {
            // Rust: Named field syntax
            shadows_enabled: true,  // Enable shadow casting
            // Rust: Use defaults for other fields
            ..default()
        },
        // Rust: Light position above scene
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Camera
    commands.spawn((
        // Rust: Default camera settings
        Camera3d::default(),
        // Rust: Camera positioned to view all objects
        Transform::from_xyz(-2.0, 3.0, 5.0)  // Position
            .looking_at(Vec3::ZERO, Vec3::Y),  // Look at origin
    ));
}

/// Fades the alpha channel of all materials between 0 and 1 over time.
/// Each blend mode responds differently to this:
/// - [`Opaque`](AlphaMode::Opaque): Ignores alpha channel altogether, these materials stay completely opaque.
/// - [`Mask(f32)`](AlphaMode::Mask): Object appears when the alpha value goes above the mask's threshold, disappears
///   when the alpha value goes back below the threshold.
/// - [`Blend`](AlphaMode::Blend): Object fades in and out smoothly.
/// - [`AlphaToCoverage`](AlphaMode::AlphaToCoverage): Object fades in and out
///   in steps corresponding to the number of multisample antialiasing (MSAA)
///   samples in use. For example, assuming 8xMSAA, the object will be
///   completely opaque, then will be 7/8 opaque (1/8 transparent), then will be
///   6/8 opaque, then 5/8, etc.
// Rust: pub keyword makes function public (accessible from other modules)
pub fn fade_transparency(
    // Rust: Read-only Time resource
    time: Res<Time>, 
    // Rust: Mutable access to material assets
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // Rust: Calculate alpha value using sine wave
    // sin() returns -1 to 1, so we transform to 0 to 1 range
    let alpha = (ops::sin(time.elapsed_secs()) / 2.0) + 0.5;
    
    // Rust: Iterate over all materials in the asset storage
    // iter_mut() returns iterator of (&Handle, &mut StandardMaterial) tuples
    for (_, material) in materials.iter_mut() {
        // Rust: _ placeholder ignores the handle (first tuple element)
        // We only need the material (second element)
        
        // Rust: Mutate the alpha channel of the base color
        // set_alpha() modifies the existing color's alpha value
        material.base_color.set_alpha(alpha);
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Alpha Modes**:
//    - `Opaque` - No transparency (fastest)
//    - `Mask(threshold)` - Binary cutoff
//    - `Blend` - Full transparency (requires sorting)
//    - `AlphaToCoverage` - MSAA-based dithering
//
// 2. **Implicit Conversions**:
//    - `Color` -> `StandardMaterial` via From trait
//    - Automatically sets alpha_mode based on alpha value
//    - Convenient for simple materials
//
// 3. **Result Handling**:
//    - `.unwrap()` extracts value from Result
//    - Panics if Result is Err
//    - Safe here because ico(3) won't fail
//
// 4. **Iterator Patterns**:
//    - `iter_mut()` for mutable iteration
//    - Returns tuples of (key, value)
//    - `_` placeholder ignores unwanted values
//
// 5. **Mathematical Transformations**:
//    - `sin(x)` returns -1 to 1
//    - `/2.0` scales to -0.5 to 0.5
//    - `+0.5` shifts to 0.0 to 1.0
//
// 6. **Visibility Modifiers**:
//    - `pub` makes items public
//    - Default is private to module
//    - Functions need pub to be called externally
//
// 7. **Color Spaces**:
//    - `srgb()` - No alpha (opaque)
//    - `srgba()` - With alpha channel
//    - Alpha affects transparency mode selection
