//! Meshlet rendering for dense high-poly scenes (experimental).
//!
//! # What are Meshlets?
//!
//! Meshlets are a modern GPU rendering technique that breaks large meshes into
//! small clusters of triangles (typically 64-128 triangles each). Think of it
//! like cutting a large 3D model into tiny puzzle pieces.
//!
//! # Why Meshlets?
//!
//! Traditional rendering processes entire meshes at once. With meshlets:
//! - **GPU-driven culling**: The GPU can skip invisible meshlets entirely
//! - **Better LOD**: Level-of-detail per meshlet instead of per object
//! - **Efficient memory**: Better GPU cache usage with smaller chunks
//! - **Massive scenes**: Render millions of triangles efficiently
//!
//! # When to Use Meshlets
//!
//! Meshlets excel with:
//! - **High-poly meshes**: Models with 100k+ triangles
//! - **Large scenes**: Many instances of complex geometry
//! - **Nanite-style rendering**: Like Unreal Engine 5's virtualized geometry
//!
//! Note: This example showcases the meshlet API, but is not the type of scene that would benefit from using meshlets.
//! The bunny model is relatively low-poly. In production, you'd use meshlets for much denser geometry.

// Include helper module for camera controls
#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

use bevy::{
    pbr::{
        experimental::meshlet::{
            MeshletMesh3d,  // Component for meshlet-based meshes
            MeshletPlugin,  // Plugin that enables meshlet rendering
        },
        CascadeShadowConfigBuilder, DirectionalLightShadowMap,
    },
    prelude::*,
    render::render_resource::AsBindGroup, // For custom materials
};
use camera_controller::{CameraController, CameraControllerPlugin};
use std::{
    f32::consts::PI,
    path::Path,         // For checking if asset exists
    process::ExitCode,  // For returning error codes
};

// URL to download the pre-processed meshlet asset
// The .meshlet_mesh format contains a mesh that's already been
// converted into meshlets, saving processing time at runtime
const ASSET_URL: &str =
    "https://raw.githubusercontent.com/JMS55/bevy_meshlet_asset/7a7c14138021f63904b584d5f7b73b695c7f4bbf/bunny.meshlet_mesh";

fn main() -> ExitCode {
    // Check if the required asset exists
    // Meshlet assets must be pre-processed, so we can't generate them at runtime
    if !Path::new("./assets/external/models/bunny.meshlet_mesh").exists() {
        eprintln!("ERROR: Asset at path <bevy>/assets/external/models/bunny.meshlet_mesh is missing. Please download it from {ASSET_URL}");
        return ExitCode::FAILURE;
    }

    App::new()
        // High-res shadow map for better quality
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((
            DefaultPlugins,
            MeshletPlugin {
                // Number of cluster slots in GPU buffer
                // Higher = more meshlets can be rendered
                // Each meshlet is a cluster of ~64-128 triangles
                cluster_buffer_slots: 8192,
            },
            // Register our debug material for visualizing meshlets
            MaterialPlugin::<MeshletDebugMaterial>::default(),
            CameraControllerPlugin,
        ))
        .add_systems(Startup, setup)
        .run();

    ExitCode::SUCCESS
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut debug_materials: ResMut<Assets<MeshletDebugMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Spawn camera with environment lighting
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(1.8, 0.4, -0.1)).looking_at(Vec3::ZERO, Vec3::Y),
        // MSAA doesn't work well with meshlets currently
        Msaa::Off,
        // IBL (Image-Based Lighting) for realistic reflections
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 150.0,
            ..default()
        },
        CameraController::default(),
    ));

    // Spawn sun light
    commands.spawn((
        DirectionalLight {
            // Use physical light units - full daylight is ~100,000 lux
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        // Shadow cascade configuration
        CascadeShadowConfigBuilder {
            num_cascades: 1,        // Single cascade for this small scene
            maximum_distance: 15.0, // Shadows only within 15 units
            ..default()
        }
        .build(),
        // Rotate light to cast shadows at an angle
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI * -0.15, PI * -0.15)),
    ));

    // Load the pre-processed meshlet mesh
    // A custom file format storing a [`bevy_render::mesh::Mesh`]
    // that has been converted to a [`bevy_pbr::meshlet::MeshletMesh`]
    // using [`bevy_pbr::meshlet::MeshletMesh::from_mesh`], which is
    // a function only available when the `meshlet_processor` cargo feature is enabled.
    // 
    // The conversion process:
    // 1. Load regular mesh (e.g., from GLTF)
    // 2. Split into meshlets (clusters of ~64-128 triangles)
    // 3. Build spatial data structures for GPU culling
    // 4. Save as .meshlet_mesh file
    let meshlet_mesh_handle = asset_server.load("external/models/bunny.meshlet_mesh");
    let debug_material = debug_materials.add(MeshletDebugMaterial::default());

    // Spawn 5 bunnies with standard materials
    // Each has a different color and roughness to show material variety
    for x in -2..=2 {
        commands.spawn((
            // MeshletMesh3d is like Mesh3d but for meshlet-based rendering
            MeshletMesh3d(meshlet_mesh_handle.clone()),
            MeshMaterial3d(standard_materials.add(StandardMaterial {
                // Rainbow colors from red to blue
                base_color: match x {
                    -2 => Srgba::hex("#dc2626").unwrap().into(), // Red
                    -1 => Srgba::hex("#ea580c").unwrap().into(), // Orange
                    0 => Srgba::hex("#facc15").unwrap().into(),  // Yellow
                    1 => Srgba::hex("#16a34a").unwrap().into(),  // Green
                    2 => Srgba::hex("#0284c7").unwrap().into(),  // Blue
                    _ => unreachable!(),
                },
                // Roughness increases from left (shiny) to right (matte)
                perceptual_roughness: (x + 2) as f32 / 4.0,
                ..default()
            })),
            Transform::default()
                .with_scale(Vec3::splat(0.2))  // Scale down to 20%
                .with_translation(Vec3::new(x as f32 / 2.0, 0.0, -0.3)),
        ));
    }
    // Spawn 5 more bunnies with debug material
    // These will visualize the meshlet structure
    for x in -2..=2 {
        commands.spawn((
            MeshletMesh3d(meshlet_mesh_handle.clone()),
            // Debug material shows meshlet boundaries
            MeshMaterial3d(debug_material.clone()),
            Transform::default()
                .with_scale(Vec3::splat(0.2))
                .with_rotation(Quat::from_rotation_y(PI)) // Face backward
                .with_translation(Vec3::new(x as f32 / 2.0, 0.0, 0.3)),
        ));
    }

    // Ground plane (not using meshlets - it's simple enough)
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0, // Fully rough (no reflections)
            ..default()
        })),
    ));
}

// Custom material for debugging meshlet boundaries
// The actual shader logic is implemented in the meshlet rendering pipeline
// This material will colorize each meshlet differently so you can see
// how the mesh is divided into clusters
#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
struct MeshletDebugMaterial {
    _dummy: (), // Empty struct - the debug visualization is handled by the renderer
}
// Minimal Material implementation - the meshlet system handles the rest
impl Material for MeshletDebugMaterial {}
