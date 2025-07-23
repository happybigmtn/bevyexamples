//! Demonstrates how to prevent meshes from casting/receiving shadows in a 3d scene.
//!
//! # Shadow Control: Fine-Tuning What Casts and Receives Shadows
//!
//! Not everything in your scene needs to participate in shadows! Sometimes you want:
//! - Floating UI elements that don't cast shadows
//! - Transparent objects that receive but don't cast shadows
//! - Glowing effects that shouldn't be shadowed
//! - Performance optimization by disabling shadows on distant objects
//!
//! ## Shadow Components in Bevy:
//!
//! - **Default behavior**: Objects both cast AND receive shadows
//! - **NotShadowCaster**: Object doesn't cast shadows (others can't see its shadow)
//! - **NotShadowReceiver**: Object doesn't receive shadows (appears fully lit)
//!
//! ## This Example Demonstrates:
//!
//! - Red sphere: Casts and receives shadows (default)
//! - Blue sphere: Doesn't cast shadows (NotShadowCaster)
//! - Green plane: Neither casts nor receives (both components)
//! - White ground: Only receives shadows
//! - Interactive toggling of shadow properties

// Rust: Import mathematical constant from standard library
// PI ≈ 3.14159265... for angle calculations
use std::f32::consts::PI;

// Rust: Structured imports from external crate
use bevy::{
    // Rust: Multiple imports from color palettes
    // Basic color constants for easy use
    color::palettes::basic::{BLUE, LIME, RED},
    // Rust: PBR (Physically Based Rendering) components
    pbr::{
        CascadeShadowConfigBuilder, // Builder for shadow cascade configuration
        NotShadowCaster,           // Marker: doesn't cast shadows
        NotShadowReceiver,         // Marker: doesn't receive shadows
    },
    // Rust: Common Bevy types
    prelude::*,
};

// Rust: Program entry point
fn main() {
    // Rust: Multi-line string literal
    // println! macro for console output
    println!(
        "Controls:
    C      - toggle shadow casters (i.e. casters become not, and not casters become casters)
    R      - toggle shadow receivers (i.e. receivers become not, and not receivers become receivers)
    L      - switch between directional and point lights"
    );
    
    // Rust: App configuration with builder pattern
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // Rust: Tuple of systems for Update schedule
        .add_systems(Update, (toggle_light, toggle_shadows))
        .run();
}

/// set up a 3D scene to test shadow biases and perspective projections
// Rust: System function with mutable resource access
fn setup(
    // Rust: Mutable Commands for entity operations
    mut commands: Commands,
    // Rust: Mutable asset storage access
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Rust: Local variable bindings
    // f32 suffix explicitly specifies type
    let spawn_plane_depth = 500.0f32;  // Scene depth for light range
    let spawn_height = 2.0;             // Height for floating objects
    let sphere_radius = 0.25;           // Sphere size

    // Rust: Creating shared material asset
    let white_handle = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        // Rust: f32 literal for material property
        perceptual_roughness: 1.0,  // Completely diffuse surface
        // Rust: Struct update syntax
        ..default()
    });
    // Rust: Creating shared mesh asset
    let sphere_handle = meshes.add(Sphere::new(sphere_radius));

    // sphere - initially a caster
    // Rust: Tuple of components for entity
    commands.spawn((
        // Rust: Clone Handle (cheap - reference counted)
        Mesh3d(sphere_handle.clone()),
        // Rust: Color conversion and material creation
        // Color::from() converts color constant to Color enum
        MeshMaterial3d(materials.add(Color::from(RED))),
        // Rust: Transform positioning
        Transform::from_xyz(-1.0, spawn_height, 0.0),
        // NOTE: No NotShadowCaster component = casts shadows by default
    ));

    // sphere - initially not a caster
    commands.spawn((
        // Rust: Move sphere_handle (no clone this time)
        Mesh3d(sphere_handle),
        MeshMaterial3d(materials.add(Color::from(BLUE))),
        Transform::from_xyz(1.0, spawn_height, 0.0),
        // Rust: Marker component prevents shadow casting
        NotShadowCaster,
    ));

    // floating plane - initially not a shadow receiver and not a caster
    commands.spawn((
        // Rust: Inline mesh creation with method chaining
        Mesh3d(meshes.add(
            Plane3d::default()     // Default plane primitive
                .mesh()            // Convert to mesh builder
                .size(20.0, 20.0)  // Set plane dimensions
        )),
        MeshMaterial3d(materials.add(Color::from(LIME))),
        Transform::from_xyz(0.0, 1.0, -10.0),
        // Rust: Multiple marker components
        // This plane neither casts nor receives shadows
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // lower ground plane - initially a shadow receiver
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        // Rust: Use previously created white material
        MeshMaterial3d(white_handle),
        // NOTE: No shadow components = both casts and receives by default
    ));

    // Rust: Console output for current state
    println!("Using DirectionalLight");

    // Rust: Spawn point light (initially disabled)
    commands.spawn((
        PointLight {
            // Rust: Start with zero intensity (disabled)
            intensity: 0.0,
            // Rust: Use scene depth for light range
            range: spawn_plane_depth,
            color: Color::WHITE,
            // Rust: bool literal for shadow casting
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(5.0, 5.0, 0.0),
    ));

    // Rust: Spawn directional light (initially enabled)
    commands.spawn((
        DirectionalLight {
            // Rust: Module path to lighting constants
            // light_consts::lux provides realistic lighting values
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        // Rust: Complex rotation with Euler angles
        Transform::from_rotation(
            Quat::from_euler(
                EulerRot::ZYX,  // Rotation order
                0.0,            // Z rotation
                PI / 2.,        // Y rotation (90 degrees)
                -PI / 4.        // X rotation (-45 degrees)
            )
        ),
        // Rust: Builder pattern for shadow cascades
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 7.0,  // Near cascade distance
            maximum_distance: 25.0,        // Far cascade distance
            ..default()
        }
        // Rust: Consume builder to create final config
        .build(),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        // Rust: Camera positioning with method chaining
        Transform::from_xyz(-5.0, 5.0, 5.0)
            // Rust: Look at red sphere position
            .looking_at(Vec3::new(-1.0, 1.0, 0.0), Vec3::Y),
    ));
}

// Rust: System for toggling between light types
fn toggle_light(
    input: Res<ButtonInput<KeyCode>>,
    // Rust: Separate queries for different light types
    mut point_lights: Query<&mut PointLight>,
    mut directional_lights: Query<&mut DirectionalLight>,
) {
    // Rust: Single-frame key press detection
    if input.just_pressed(KeyCode::KeyL) {
        // Rust: Iterate over all point lights
        for mut light in &mut point_lights {
            // Rust: Conditional assignment with if expression
            light.intensity = if light.intensity == 0.0 {
                println!("Using PointLight");
                // Rust: Numeric literal with underscores for readability
                1_000_000.0 // Mini-sun point light
            } else {
                0.0  // Turn off
            };
        }
        // Rust: Same pattern for directional lights
        for mut light in &mut directional_lights {
            light.illuminance = if light.illuminance == 0.0 {
                println!("Using DirectionalLight");
                // Rust: Use lighting constant
                light_consts::lux::OVERCAST_DAY
            } else {
                0.0
            };
        }
    }
}

// Rust: Complex system with ParamSet for multiple queries
fn toggle_shadows(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    // Rust: ParamSet allows multiple conflicting queries
    // Without ParamSet, these queries would conflict (mutable access)
    mut queries: ParamSet<(
        // Rust: Query 0: Entities that are meshes AND have NotShadowCaster
        Query<Entity, (With<Mesh3d>, With<NotShadowCaster>)>,
        // Rust: Query 1: Entities that are meshes AND have NotShadowReceiver
        Query<Entity, (With<Mesh3d>, With<NotShadowReceiver>)>,
        // Rust: Query 2: Entities that are meshes but DON'T have NotShadowCaster
        Query<Entity, (With<Mesh3d>, Without<NotShadowCaster>)>,
        // Rust: Query 3: Entities that are meshes but DON'T have NotShadowReceiver
        Query<Entity, (With<Mesh3d>, Without<NotShadowReceiver>)>,
    )>,
) {
    // Rust: Toggle shadow casting
    if input.just_pressed(KeyCode::KeyC) {
        println!("Toggling casters");
        // Rust: Access specific query from ParamSet with .p0()
        // Remove NotShadowCaster from entities that have it
        for entity in queries.p0().iter() {
            // Rust: Turbofish syntax ::<Type> for generic method
            commands.entity(entity).remove::<NotShadowCaster>();
        }
        // Rust: Add NotShadowCaster to entities that don't have it
        for entity in queries.p2().iter() {
            commands.entity(entity).insert(NotShadowCaster);
        }
    }
    
    // Rust: Toggle shadow receiving
    if input.just_pressed(KeyCode::KeyR) {
        println!("Toggling receivers");
        // Rust: Same pattern but for shadow receivers
        // .p1() and .p3() access different queries from the ParamSet
        for entity in queries.p1().iter() {
            commands.entity(entity).remove::<NotShadowReceiver>();
        }
        for entity in queries.p3().iter() {
            commands.entity(entity).insert(NotShadowReceiver);
        }
    }
}

// 🎯 Advanced Rust Concepts in This Example:
//
// 1. **ParamSet**:
//    - Allows multiple conflicting queries in one system
//    - .p0(), .p1(), .p2(), .p3() access individual queries
//    - Essential when queries would otherwise conflict
//
// 2. **Query Filters**:
//    - `With<T>` - entities that have component T
//    - `Without<T>` - entities that DON'T have component T
//    - Can combine multiple filters with tuples
//
// 3. **Turbofish Syntax**:
//    - `::<Type>` explicitly specifies generic parameters
//    - Needed when compiler can't infer the type
//    - Common with remove::<ComponentType>()
//
// 4. **Marker Components**:
//    - NotShadowCaster, NotShadowReceiver are zero-size
//    - Used purely for tagging/identification
//    - No runtime memory cost
//
// 5. **Entity Commands**:
//    - `.insert()` adds components to entities
//    - `.remove::<T>()` removes specific component types
//    - Operations are deferred until end of frame
//
// 6. **Numeric Literals**:
//    - `1_000_000.0` - underscores for readability
//    - Equivalent to `1000000.0`
//    - Common in Rust for large numbers
//
// 7. **Module Paths**:
//    - `light_consts::lux::OVERCAST_DAY`
//    - Deep nested module access
//    - Provides organized constant values
