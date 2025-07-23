//! Illustrates spot lights.
//!
//! # Spot Lights: Focused Illumination
//!
//! Spot lights are like flashlights or stage lights - they emit light in a
//! cone shape, creating dramatic focused lighting effects. Think of:
//! - Theater spotlights following actors
//! - Car headlights cutting through darkness
//! - Flashlights exploring dark corridors
//! - Security lights with motion sensors
//!
//! ## Spot Light Properties:
//!
//! 1. **Inner/Outer Angles**: Define the cone shape
//!    - Inner angle: Full intensity cone
//!    - Outer angle: Falloff region (penumbra)
//! 2. **Direction**: Where the light points
//! 3. **Intensity**: How bright (in lumens)
//! 4. **Range**: How far the light reaches
//!
//! ## This Example Shows:
//!
//! - 16 animated spot lights in a 4x4 grid
//! - Dynamic angle adjustments (pulsing effect)
//! - Swaying motion (like stage lights)
//! - Movable scene with WASD controls
//! - Visual indicators showing light direction

// Rust: Glob import of f32 math constants
// Imports PI, FRAC_PI_2, FRAC_PI_4, etc.
use std::f32::consts::*;

// Rust: External crate imports
use bevy::{
    // Rust: Nested module imports for colors
    color::palettes::basic::{MAROON, RED},
    // Rust: Math operations module
    math::ops,
    // Rust: PBR component for shadow control
    pbr::NotShadowCaster,
    // Rust: Common Bevy types
    prelude::*,
    // Rust: HDR rendering component
    render::view::Hdr,
};
// Rust: External RNG (Random Number Generator) crates
use rand::{Rng, SeedableRng};  // Traits for random generation
use rand_chacha::ChaCha8Rng;   // Specific RNG algorithm

// Rust: String literal constant with raw string
// The \ at end of first line continues the string
const INSTRUCTIONS: &str = "\
Controls
--------
Horizontal Movement: WASD
Vertical Movement: Space and Shift
Rotate Camera: Left and Right Arrows";

// Rust: Program entry point
fn main() {
    // Rust: App builder with resource insertion
    App::new()
        // Rust: Set ambient light before plugin initialization
        .insert_resource(AmbientLight {
            brightness: 20.0,  // Low ambient for dramatic lighting
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // Rust: Multiple update systems as tuple
        .add_systems(Update, (light_sway, movement, rotation))
        .run();
}

#[derive(Component)]
struct Movable;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Movable,
    ));

    // cubes

    // We're seeding the PRNG here to make this example deterministic for testing purposes.
    // This isn't strictly required in practical use unless you need your app to be deterministic.
    // Rust: Create deterministic RNG with specific seed
    // ChaCha8Rng is a cryptographically secure PRNG
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467713);
    
    // Rust: Create shared mesh asset
    let cube_mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
    
    // Rust: Create material with u8 color values
    // srgb_u8 takes 0-255 values instead of 0.0-1.0
    let blue = materials.add(Color::srgb_u8(124, 144, 255));

    // Rust: Batch spawning for efficiency
    commands.spawn_batch(
        // Rust: Create infinite iterator with closure
        std::iter::repeat_with(move || {
            // Rust: Generate random values in range
            // Range syntax: start..end (exclusive end)
            let x = rng.gen_range(-5.0..5.0);
            let y = rng.gen_range(0.0..3.0);
            let z = rng.gen_range(-5.0..5.0);

            // Rust: Return tuple of components
            (
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(blue.clone()),
                Transform::from_xyz(x, y, z),
                Movable,  // Mark as movable
            )
        })
        // Rust: Take only first 40 items from infinite iterator
        .take(40),
    );

    let sphere_mesh = meshes.add(Sphere::new(0.05).mesh().uv(32, 18));
    let sphere_mesh_direction = meshes.add(Sphere::new(0.1).mesh().uv(32, 18));
    let red_emissive = materials.add(StandardMaterial {
        base_color: RED.into(),
        emissive: LinearRgba::new(1.0, 0.0, 0.0, 0.0),
        ..default()
    });
    let maroon_emissive = materials.add(StandardMaterial {
        base_color: MAROON.into(),
        emissive: LinearRgba::new(0.369, 0.0, 0.0, 0.0),
        ..default()
    });

    // Rust: Nested for loops for grid generation
    for x in 0..4 {
        for z in 0..4 {
            // Rust: Convert loop indices to world positions
            // Centers the 4x4 grid around origin
            let x = x as f32 - 2.0;  // Maps 0..4 to -2.0..2.0
            let z = z as f32 - 2.0;
            
            // red spot_light
            // Rust: Spawn parent entity with spot light
            commands
                .spawn((
                    SpotLight {
                        // Rust: Numeric literal with underscores
                        intensity: 40_000.0, // lumens (very bright)
                        color: Color::WHITE,
                        shadows_enabled: true,
                        // Rust: Mathematical expressions
                        // PI/4 = 45 degrees, multiplied by 0.85 for inner cone
                        inner_angle: PI / 4.0 * 0.85,
                        outer_angle: PI / 4.0,
                        ..default()
                    },
                    // Rust: Transform with calculated position
                    Transform::from_xyz(1.0 + x, 2.0, z)
                        // Rust: Point light downward at ground
                        .looking_at(Vec3::new(1.0 + x, 0.0, z), Vec3::X),
                ))
                // Rust: Add child entities with closure
                .with_children(|builder| {
                    // Rust: Spawn light source indicator (red sphere)
                    builder.spawn((
                        Mesh3d(sphere_mesh.clone()),
                        MeshMaterial3d(red_emissive.clone()),
                    ));
                    // Rust: Spawn direction indicator (maroon sphere)
                    builder.spawn((
                        Mesh3d(sphere_mesh_direction.clone()),
                        MeshMaterial3d(maroon_emissive.clone()),
                        // Rust: Vector multiplication for offset
                        Transform::from_translation(Vec3::Z * -0.1),
                        // Rust: Marker component prevents shadow
                        NotShadowCaster,
                    ));
                });
        }
    }

    // camera
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Transform::from_xyz(-4.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Text::new(INSTRUCTIONS),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// Rust: System for animating spot light motion and cone angles
fn light_sway(
    // Rust: Time resource for animation
    time: Res<Time>, 
    // Rust: Query for spot lights with transforms
    mut query: Query<(&mut Transform, &mut SpotLight)>
) {
    // Rust: Iterate over all spot lights
    for (mut transform, mut angles) in query.iter_mut() {
        // Rust: Create rotation from Euler angles
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,  // Rotation order
            // Rust: X rotation - base angle + oscillation
            // FRAC_PI_2 = PI/2 = 90 degrees (pointing down)
            -FRAC_PI_2 + ops::sin(time.elapsed_secs() * 0.67 * 3.0) * 0.5,
            // Rust: Y rotation - pure oscillation
            ops::sin(time.elapsed_secs() * 3.0) * 0.5,
            0.0,  // No Z rotation
        );
        
        // Rust: Calculate pulsing cone angle
        // sin() returns -1 to 1, adding 1 gives 0 to 2
        let angle = (ops::sin(time.elapsed_secs() * 1.2) + 1.0) * (FRAC_PI_4 - 0.1);
        
        // Rust: Update light cone angles
        angles.inner_angle = angle * 0.8;  // Inner cone is 80% of outer
        angles.outer_angle = angle;         // Outer cone pulses
    }
}

fn movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Movable>>,
) {
    // Calculate translation to move the cubes and ground plane
    let mut translation = Vec3::ZERO;

    // Horizontal forward and backward movement
    if input.pressed(KeyCode::KeyW) {
        translation.z += 1.0;
    } else if input.pressed(KeyCode::KeyS) {
        translation.z -= 1.0;
    }

    // Horizontal left and right movement
    if input.pressed(KeyCode::KeyA) {
        translation.x += 1.0;
    } else if input.pressed(KeyCode::KeyD) {
        translation.x -= 1.0;
    }

    // Vertical movement
    if input.pressed(KeyCode::ShiftLeft) {
        translation.y += 1.0;
    } else if input.pressed(KeyCode::Space) {
        translation.y -= 1.0;
    }

    translation *= 2.0 * time.delta_secs();

    // Apply translation
    for mut transform in &mut query {
        transform.translation += translation;
    }
}

// Rust: System for camera rotation
fn rotation(
    // Rust: Single query - expects exactly one result
    // More efficient than Query when you know there's only one
    mut transform: Single<&mut Transform, With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // Rust: Cache delta time
    let delta = time.delta_secs();

    // Rust: Handle rotation input
    if input.pressed(KeyCode::ArrowLeft) {
        // Rust: Rotate camera around world origin
        transform.rotate_around(
            Vec3::ZERO,                          // Pivot point
            Quat::from_rotation_y(delta)         // Positive = counter-clockwise
        );
    } else if input.pressed(KeyCode::ArrowRight) {
        transform.rotate_around(
            Vec3::ZERO, 
            Quat::from_rotation_y(-delta)        // Negative = clockwise
        );
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Random Number Generation**:
//    - `SeedableRng` trait for deterministic RNG
//    - `gen_range()` for bounded random values
//    - Move closure captures RNG by value
//
// 2. **Iterator Combinators**:
//    - `repeat_with()` creates infinite iterator
//    - `take()` limits to finite number
//    - `spawn_batch()` for efficient entity creation
//
// 3. **Single Query**:
//    - More efficient than Query for single entities
//    - Panics if not exactly one match
//    - Good for unique components like main camera
//
// 4. **Mathematical Constants**:
//    - `FRAC_PI_2` = π/2 (90 degrees)
//    - `FRAC_PI_4` = π/4 (45 degrees)
//    - Clearer than writing 1.5708...
//
// 5. **Color Creation Methods**:
//    - `srgb_u8()` for 0-255 values
//    - `LinearRgba::new()` for linear color space
//    - Different color spaces for different uses
//
// 6. **Emissive Materials**:
//    - Make objects glow/emit light
//    - Visual indicators for light sources
//    - Combine with NotShadowCaster for UI elements
//
// 7. **Spot Light Properties**:
//    - Inner/outer angles create soft edges
//    - Intensity in lumens (physical units)
//    - Direction from Transform orientation
