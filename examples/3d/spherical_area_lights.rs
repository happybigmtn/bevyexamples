//! Demonstrates how lighting is affected by different radius of point lights.
//!
//! # Spherical Area Lights: From Point to Sphere
//!
//! In real life, lights aren't infinitely small points - they have size!
//! A light bulb has a glass sphere, the sun is a massive ball of fire,
//! and even LEDs have a physical emitting surface.
//!
//! ## Why Light Radius Matters:
//!
//! 1. **Soft Shadows**: Larger lights create softer shadow edges
//! 2. **Specular Highlights**: Size affects the shape of reflections
//! 3. **Realism**: Point lights look artificial, area lights look natural
//! 4. **Energy Distribution**: Light spreads differently from surfaces vs points
//!
//! ## This Example Creates:
//!
//! A row of 6 lights with increasing radius from left to right:
//! - Left: Nearly point-like (radius ≈ 0.0)
//! - Right: Larger spherical light (radius = 0.4)
//!
//! Watch how the reflections on the ground change from sharp points
//! to broader, softer highlights as the radius increases!

// Rust: Import all common Bevy types
use bevy::prelude::*;

// Rust: Program entry point
fn main() {
    // Rust: Builder pattern for app configuration
    App::new()
        // Rust: Insert resource before plugins
        // This sets ambient light before rendering setup
        .insert_resource(AmbientLight {
            // Rust: f64 literal (60.0) works with f32 field
            brightness: 60.0,  // Subtle ambient to see surfaces
            // Rust: Default other fields
            ..default()
        })
        // Rust: Standard Bevy plugins
        .add_plugins(DefaultPlugins)
        // Rust: Single startup system
        .add_systems(Startup, setup)
        // Rust: Consume app and run
        .run();
}

// Rust: System function with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Mutable access to mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to material assets
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn((
        // Rust: Default camera configuration
        Camera3d::default(),
        // Rust: Transform with method chaining
        Transform::from_xyz(0.2, 1.5, 2.5)  // Slightly offset position
            .looking_at(Vec3::ZERO, Vec3::Y), // Look at origin
    ));

    // plane
    commands.spawn((
        // Rust: Create mesh inline with builder pattern
        Mesh3d(meshes.add(
            Plane3d::default()    // Default plane primitive
                .mesh()           // Convert to mesh builder
                .size(100.0, 100.0)  // Large ground plane
        )),
        // Rust: Create material inline
        MeshMaterial3d(materials.add(StandardMaterial {
            // Rust: Color with sRGB values
            base_color: Color::srgb(0.2, 0.2, 0.2),  // Dark gray
            // Rust: f32 literal for material property
            perceptual_roughness: 0.08,  // Very smooth (reflective)
            ..default()
        })),
    ));

    // Rust: const in function scope - compile-time constant
    const COUNT: usize = 6;  // Number of lights to create
    
    // Rust: Range literals with exclusive end
    let position_range = -2.0..2.0;  // X positions from -2 to 2
    let radius_range = 0.0..0.4;     // Light radius from 0 to 0.4
    
    // Rust: Calculate range lengths
    // Range.end and Range.start access bounds
    let pos_len = position_range.end - position_range.start;
    let radius_len = radius_range.end - radius_range.start;
    
    // Rust: Create shared sphere mesh with high tessellation
    let mesh = meshes.add(
        Sphere::new(1.0)    // Unit sphere
            .mesh()         // Convert to mesh builder
            .uv(120, 64)    // High resolution (120 horizontal, 64 vertical segments)
    );

    // Rust: for loop with range
    for i in 0..COUNT {
        // Rust: Type casting with 'as'
        // Convert usize to f32 for calculations
        let percent = i as f32 / COUNT as f32;
        
        // Rust: Linear interpolation calculation
        // Maps 0..1 percent to radius range
        let radius = radius_range.start + percent * radius_len;

        // sphere light
        // Rust: Spawn entity with child
        commands
            .spawn((
                // Rust: Clone handle (cheap - reference counted)
                Mesh3d(mesh.clone()),
                // Rust: Unlit material for glowing effect
                MeshMaterial3d(materials.add(StandardMaterial {
                    // Rust: Light blue color
                    base_color: Color::srgb(0.5, 0.5, 1.0),
                    // Rust: bool literal
                    unlit: true,  // Material emits light, not affected by lights
                    ..default()
                })),
                // Rust: Position and scale based on loop iteration
                Transform::from_xyz(
                    position_range.start + percent * pos_len,  // Spread along X
                    0.3,  // Slightly above ground
                    0.0   // Centered on Z
                )
                    // Rust: Scale sphere to match light radius
                    .with_scale(Vec3::splat(radius)),
            ))
            // Rust: Add child entity (light component)
            .with_child(PointLight {
                // Rust: Field shorthand - radius: radius
                radius,  // This is the key - variable light size!
                // Rust: Darker blue for the actual light
                color: Color::srgb(0.2, 0.2, 1.0),
                ..default()
            });
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Range Types**:
//    - `start..end` - Exclusive range (doesn't include end)
//    - Access with `.start` and `.end` fields
//    - Used for iteration and calculations
//
// 2. **Type Casting**:
//    - `as f32` converts integers to floats
//    - Required for mixed-type arithmetic
//    - Safe for numeric types
//
// 3. **Const vs Let**:
//    - `const` - Compile-time constant, can't use runtime values
//    - `let` - Runtime variable binding
//    - Use const for fixed values known at compile time
//
// 4. **Linear Interpolation Pattern**:
//    - `start + percent * length`
//    - Maps normalized value (0-1) to range
//    - Common pattern in graphics programming
//
// 5. **Field Shorthand**:
//    - `radius` instead of `radius: radius`
//    - When field name matches variable name
//    - Makes code more concise
//
// 6. **Entity Hierarchies**:
//    - `.with_child()` creates parent-child relationship
//    - Child transforms are relative to parent
//    - Light moves with its visual sphere
//
// 7. **Handle Cloning**:
//    - Mesh handles can be cloned efficiently
//    - Multiple entities share same GPU mesh data
//    - Only the handle is duplicated, not the mesh
