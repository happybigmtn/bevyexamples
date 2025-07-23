//! This example shows how to configure Physically Based Rendering (PBR) parameters.
//!
//! 🎨 The Science of Digital Materials: Understanding PBR
//!
//! Have you ever wondered why a gold ring looks different from a rubber ball,
//! even when they're the same color? It's all about how materials interact with
//! light! Physically Based Rendering (PBR) is like having a recipe book for
//! creating any material imaginable - from shiny metals to rough concrete.
//! Instead of guessing, we use real physics to make materials look right under
//! any lighting condition. It's digital alchemy!
//!
//! 🎯 What You'll See:
//! - A grid of 55 spheres showing all combinations of:
//!   - Metallic values (vertical: 0% to 100%)
//!   - Roughness values (horizontal: 0% to 100%)
//! - Environment lighting creating realistic reflections
//! - One unlit sphere for comparison
//! - Interactive labels showing the parameter gradients
//!
//! 🔑 Key Concepts:
//! - Metallic: Is it a metal (reflective) or not (diffuse)?
//! - Roughness: Is the surface smooth (mirror-like) or rough (matte)?
//! - Base Color: The material's inherent color
//! - Environment Maps: Real-world lighting for realistic reflections
//! - Energy Conservation: Materials can't reflect more light than they receive!
//!
//! 💡 The PBR Magic:
//! Top-left sphere: Smooth dielectric (like plastic)
//! Top-right sphere: Rough dielectric (like clay)
//! Bottom-left sphere: Smooth metal (like polished gold)
//! Bottom-right sphere: Rough metal (like brushed bronze)

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, environment_map_load_finish)
        .run();
}

// 🏗️ Scene Setup: Building Our Material Gallery
/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 🌐 Create our test sphere mesh
    let sphere_mesh = meshes.add(Sphere::new(0.45));
    
    // 🎨 The Material Grid: A Complete PBR Parameter Space
    // We create a 5x11 grid of spheres to visualize all combinations
    for y in -2..=2 {
        for x in -5..=5 {
            // 📊 Normalize coordinates to 0-1 range
            let x01 = (x + 5) as f32 / 10.0;  // Roughness: 0.0 (left) to 1.0 (right)
            let y01 = (y + 2) as f32 / 4.0;   // Metallic: 0.0 (top) to 1.0 (bottom)
            
            // 🌟 Create sphere with specific PBR parameters
            commands.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    // 🎨 Base color: Warm gold tone
                    // This is the "albedo" - the material's inherent color
                    base_color: Srgba::hex("#ffd891").unwrap().into(),
                    
                    // 🔧 The Two Key PBR Parameters:
                    metallic: y01,              // 0 = dielectric, 1 = metal
                    perceptual_roughness: x01,  // 0 = mirror, 1 = diffuse
                    
                    ..default()
                })),
                Transform::from_xyz(x as f32, y as f32 + 0.5, 0.0),
            ));
        }
    }
    // 💡 Reference Sphere: Unlit Material
    // This shows the base color without any lighting calculations
    // Useful for comparing how PBR lighting affects the appearance
    commands.spawn((
        Mesh3d(sphere_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("#ffd891").unwrap().into(),
            unlit: true,  // Skip all lighting calculations
            ..default()
        })),
        Transform::from_xyz(-5.0, -2.5, 0.0),
    ));

    // ☀️ Directional Light: Simulating Sunlight
    // Provides basic illumination for our materials
    commands.spawn((
        DirectionalLight {
            illuminance: 1_500.,  // Lux - relatively dim to let environment map shine
            ..default()
        },
        Transform::from_xyz(50.0, 50.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 📊 UI Labels: Parameter Indicators
    
    // 🔄 Roughness Label (Horizontal Axis)
    commands.spawn((
        Text::new("Perceptual Roughness"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(100.0),
            ..default()
        },
    ));

    // 🔧 Metallic Label (Vertical Axis)
    // Rotated 90 degrees to align with the vertical axis
    commands.spawn((
        Text::new("Metallic"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(130.0),
            right: Val::ZERO,
            ..default()
        },
        Transform {
            rotation: Quat::from_rotation_z(std::f32::consts::PI / 2.0),  // Rotate 90°
            ..default()
        },
    ));

    // 🌍 Environment Map Loading Indicator
    commands.spawn((
        Text::new("Loading Environment Map..."),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            right: Val::Px(20.0),
            ..default()
        },
        EnvironmentMapLabel,  // Tag for removal when loaded
    ));

    // 📷 Camera Setup: Orthographic for Perfect Grid View
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::default(), Vec3::Y),
        // 📐 Orthographic projection for no perspective distortion
        Projection::from(OrthographicProjection {
            scale: 0.01,
            scaling_mode: ScalingMode::WindowSize,
            ..OrthographicProjection::default_3d()
        }),
        // 🌍 Environment Map Light: The Secret Sauce!
        // This provides realistic reflections from a real-world environment
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 900.0,
            ..default()
        },
    ));
}

// 🔄 Loading System: Hide the Loading Text When Ready
fn environment_map_load_finish(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    environment_map: Single<&EnvironmentMapLight>,
    label_entity: Option<Single<Entity, With<EnvironmentMapLabel>>>,
) {
    // 🌍 Check if both environment maps are loaded
    if asset_server
        .load_state(&environment_map.diffuse_map)
        .is_loaded()
        && asset_server
            .load_state(&environment_map.specular_map)
            .is_loaded()
    {
        // ✅ Remove the loading text once maps are ready
        // This prevents trying to remove an already-removed entity
        if let Some(label_entity) = label_entity {
            commands.entity(*label_entity).despawn();
        }
    }
}

// 🏷️ Marker component for the loading text
#[derive(Component)]
struct EnvironmentMapLabel;

// 🎓 Deep Dive: The Physics of PBR
//
// **What Makes PBR Special?**
// Traditional shading uses artistic tricks. PBR uses actual physics!
// This means materials look correct under ANY lighting condition.
//
// **The Core Principle: Energy Conservation**
// A surface can't reflect more light than it receives. This constraint
// makes materials behave realistically.
//
// **The Metallic Workflow**:
// 
// 1. **Metallic = 0 (Dielectrics)**:
//    - Non-metals like plastic, wood, fabric, stone
//    - Reflect ~4% of light as specular (F0 = 0.04)
//    - Rest is absorbed and re-emitted as diffuse color
//    - Base color = actual material color
//
// 2. **Metallic = 1 (Metals)**:
//    - Conductors like gold, silver, copper, iron
//    - No diffuse reflection (absorbed light becomes heat)
//    - 100% specular reflection
//    - Base color = specular tint (that's why gold looks gold!)
//
// 3. **Roughness**:
//    - Controls micro-surface detail
//    - Rough = scattered reflections (blurry)
//    - Smooth = coherent reflections (mirror-like)
//    - Affects both metals and dielectrics
//
// **The Rendering Equation** (simplified):
// Color = Diffuse × (1 - Metallic) + Specular × FresnelEffect
// Where:
// - Diffuse uses base color for dielectrics
// - Specular uses base color for metals
// - Fresnel makes things more reflective at grazing angles

// 💡 Practical Material Examples:
//
// **Polished Gold**: Metallic = 1.0, Roughness = 0.0
// **Brushed Aluminum**: Metallic = 1.0, Roughness = 0.3
// **Shiny Plastic**: Metallic = 0.0, Roughness = 0.1
// **Rubber**: Metallic = 0.0, Roughness = 0.8
// **Painted Metal**: Metallic = 0.0 (paint is dielectric!)
// **Rusted Iron**: Mix of metallic (iron) and non-metallic (rust)
//
// **Environment Maps**:
// - Provide realistic reflections from real-world lighting
// - Diffuse map: For rough reflections (irradiance)
// - Specular map: For sharp reflections (radiance)
// - Without these, metals would look dull and unrealistic!

// 🎨 Artist Tips:
//
// 1. **Avoid 0.5 Metallic**: Materials are either metal or not
// 2. **Start with Roughness**: It has the most visual impact
// 3. **Reference Real Materials**: Use photo references
// 4. **Test Under Different Lighting**: Good PBR works everywhere
// 5. **Mind the Base Color**:
//    - Dielectrics: Use actual colors
//    - Metals: Use measured reflectance values
//    - Too dark = unrealistic (nothing is pure black)
//    - Too bright = unrealistic (nothing is pure white)
