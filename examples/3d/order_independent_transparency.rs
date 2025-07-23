//! A simple 3D scene showing how alpha blending can break and how order independent transparency (OIT) can fix it.
//!
//! See [`OrderIndependentTransparencyPlugin`] for the trade-offs of using OIT.
//!
//! [`OrderIndependentTransparencyPlugin`]: bevy::render::pipeline::OrderIndependentTransparencyPlugin
//!
//! # The Transparency Problem
//!
//! When rendering transparent objects, the order matters! Traditional alpha
//! blending has a major limitation: it requires sorting objects back-to-front.
//! But what happens when objects intersect or overlap in complex ways?
//!
//! ## Why Order Matters
//!
//! With normal alpha blending:
//! 1. Render object A with 50% opacity
//! 2. Render object B behind it with 50% opacity
//! Result: B shows through A correctly
//!
//! But if rendered in wrong order:
//! 1. Render object B with 50% opacity
//! 2. Render object A in front with 50% opacity
//! Result: A appears behind B - wrong!
//!
//! ## Order Independent Transparency (OIT)
//!
//! OIT solves this by accumulating color and alpha values in a special way
//! that produces correct results regardless of rendering order. This uses
//! more memory and GPU power but gives correct results.
//!
//! ## When to Use OIT
//!
//! - Complex transparent geometry (hair, foliage, particles)
//! - Intersecting transparent objects
//! - When sorting is expensive or impossible
//! - UI elements with overlapping transparency
//!
//! Press T to toggle OIT and see the difference!
//! Press C to cycle through different test scenes!
use bevy::{
    color::palettes::css::{BLUE, GREEN, RED}, // Primary colors for visualization
    core_pipeline::oit::OrderIndependentTransparencySettings, // The OIT component
    prelude::*,
    render::view::RenderLayers, // For organizing what renders where
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // Interactive systems for toggling OIT and switching scenes
        .add_systems(Update, (toggle_oit, cycle_scenes))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        // Add this component to this camera to render transparent meshes using OIT
        // This enables the special rendering technique for this camera
        OrderIndependentTransparencySettings::default(),
        // Use render layers to ensure everything renders to the same camera
        RenderLayers::layer(1),
        // MSAA (Multi-Sample Anti-Aliasing) currently doesn't work with OIT
        // This is a technical limitation - OIT uses special buffers incompatible with MSAA
        Msaa::Off,
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: false, // Shadows add complexity, keep it simple
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        RenderLayers::layer(1), // Same layer as camera
    ));

    // spawn help text
    commands
        .spawn((
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            RenderLayers::layer(1),
        ))
        .with_children(|p| {
            p.spawn(TextSpan::new("Press T to toggle OIT\n"));
            p.spawn(TextSpan::new("OIT Enabled")); // This will be updated dynamically
            p.spawn(TextSpan::new("\nPress C to cycle test scenes"));
        });

    // spawn default scene - overlapping transparent spheres
    spawn_spheres(&mut commands, &mut meshes, &mut materials);
}

/// Toggle Order Independent Transparency on/off
fn toggle_oit(
    mut commands: Commands,
    text: Single<Entity, With<Text>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Query for camera entity and whether it has OIT enabled
    q: Single<(Entity, Has<OrderIndependentTransparencySettings>), With<Camera3d>>,
    mut text_writer: TextUiWriter,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        let (e, has_oit) = *q;
        // Update the status text (index 2 is the second TextSpan)
        *text_writer.text(*text, 2) = if has_oit {
            // Removing the component will completely disable OIT for this camera
            // The scene will fall back to traditional alpha blending
            commands
                .entity(e)
                .remove::<OrderIndependentTransparencySettings>();
            "OIT disabled".to_string()
        } else {
            // Adding the component to the camera will render any transparent meshes
            // with OIT instead of alpha blending - fixing sorting issues!
            commands
                .entity(e)
                .insert(OrderIndependentTransparencySettings::default());
            "OIT enabled".to_string()
        };
    }
}

/// Cycle through different test scenes to demonstrate OIT in various scenarios
fn cycle_scenes(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q: Query<Entity, With<Mesh3d>>,
    mut scene_id: Local<usize>, // Local<T> maintains state between system runs
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        // despawn current scene - remove all mesh entities
        for e in &q {
            commands.entity(e).despawn();
        }
        // increment scene_id with wrapping
        *scene_id = (*scene_id + 1) % 2;
        // spawn next scene based on ID
        match *scene_id {
            0 => spawn_spheres(&mut commands, &mut meshes, &mut materials),
            1 => spawn_occlusion_test(&mut commands, &mut meshes, &mut materials),
            _ => unreachable!(),
        }
    }
}

/// Spawns 3 overlapping spheres
/// Technically, when using `alpha_to_coverage` with MSAA this particular example wouldn't break,
/// but it breaks when disabling MSAA and is enough to show the difference between OIT enabled vs disabled.
///
/// This scene demonstrates the classic transparency sorting problem:
/// - 3 spheres overlap in a way that no single draw order is correct
/// - Red, green, and blue spheres each partially occlude the others
/// - Without OIT, you'll see incorrect blending where spheres overlap
/// - With OIT, all overlaps blend correctly regardless of draw order
fn spawn_spheres(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // Position spheres in a triangle formation
    let pos_a = Vec3::new(-1.0, 0.75, 0.0);  // Top-left (red)
    let pos_b = Vec3::new(0.0, -0.75, 0.0);  // Bottom (green)
    let pos_c = Vec3::new(1.0, 0.75, 0.0);   // Top-right (blue)

    let offset = Vec3::new(0.0, 0.0, 0.0);

    let sphere_handle = meshes.add(Sphere::new(2.0).mesh());

    // Low alpha makes overlapping areas more visible
    let alpha = 0.25;

    let render_layers = RenderLayers::layer(1);

    // Red sphere (top-left)
    commands.spawn((
        Mesh3d(sphere_handle.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.with_alpha(alpha).into(),
            // AlphaMode::Blend enables transparency
            // This is where sorting issues occur without OIT
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(pos_a + offset),
        render_layers.clone(),
    ));
    
    // Green sphere (bottom)
    commands.spawn((
        Mesh3d(sphere_handle.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GREEN.with_alpha(alpha).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(pos_b + offset),
        render_layers.clone(),
    ));
    
    // Blue sphere (top-right)
    commands.spawn((
        Mesh3d(sphere_handle.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: BLUE.with_alpha(alpha).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_translation(pos_c + offset),
        render_layers.clone(),
    ));
}

/// Spawn a combination of opaque cubes and transparent spheres.
/// This is useful to make sure transparent meshes drawn with OIT
/// are properly occluded by opaque meshes.
///
/// This scene tests a critical requirement:
/// - Transparent objects must still be hidden by opaque objects in front
/// - OIT must respect the depth buffer from opaque geometry
/// - We test three cases: sphere behind, intersecting, and in front of cube
fn spawn_occlusion_test(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let sphere_handle = meshes.add(Sphere::new(1.0).mesh());
    let cube_handle = meshes.add(Cuboid::from_size(Vec3::ONE).mesh());
    let cube_material = materials.add(Color::srgb(0.8, 0.7, 0.6)); // Beige color

    let render_layers = RenderLayers::layer(1);

    // Test 1: Transparent sphere behind opaque cube
    // The sphere should be completely hidden
    let x = -2.5;
    commands.spawn((
        Mesh3d(cube_handle.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(x, 0.0, 2.0), // Cube in front
        render_layers.clone(),
    ));
    commands.spawn((
        Mesh3d(sphere_handle.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.with_alpha(0.5).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(x, 0., 0.), // Sphere behind
        render_layers.clone(),
    ));

    // Test 2: Transparent sphere intersecting opaque cube
    // Half the sphere should be visible, half hidden
    commands.spawn((
        Mesh3d(cube_handle.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(x, 0.0, 1.0), // Cube partially overlaps
        render_layers.clone(),
    ));
    commands.spawn((
        Mesh3d(sphere_handle.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.with_alpha(0.5).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(0., 0., 0.), // Sphere at center
        render_layers.clone(),
    ));

    // Test 3: Transparent sphere in front of opaque cube
    // The entire sphere should be visible with cube showing through
    let x = 2.5;
    commands.spawn((
        Mesh3d(cube_handle.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(x, 0.0, -2.0), // Cube in back
        render_layers.clone(),
    ));
    commands.spawn((
        Mesh3d(sphere_handle.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.with_alpha(0.5).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(x, 0., 0.), // Sphere in front
        render_layers.clone(),
    ));
}
