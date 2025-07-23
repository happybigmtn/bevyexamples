//! Loads and renders a glTF file as a scene.
//!
//! # What is glTF?
//!
//! glTF (GL Transmission Format) is the "JPEG of 3D" - an open standard for
//! efficiently transmitting and loading 3D scenes and models. Think of it as
//! a universal format that any 3D software can read and write.
//!
//! A glTF file can contain:
//! - **Meshes**: 3D geometry (vertices, triangles, etc.)
//! - **Materials**: How surfaces look (colors, textures, metallic/roughness)
//! - **Animations**: How things move over time
//! - **Scenes**: Hierarchical node structure organizing everything
//! - **Cameras & Lights**: Though Bevy typically uses its own
//!
//! # Why glTF?
//!
//! - **Efficient**: Designed for real-time rendering, not just storage
//! - **Complete**: Materials, textures, and animations all in one package
//! - **Extensible**: Supports custom data through extensions
//! - **Industry Standard**: Supported by major 3D tools (Blender, Maya, etc.)
//!
//! This example loads a flight helmet model and demonstrates:
//! - Loading glTF assets
//! - Environment lighting (IBL - Image Based Lighting)
//! - Shadow mapping with cascades
//! - Animated directional lighting

use bevy::{
    pbr::{
        CascadeShadowConfigBuilder,  // Helper for configuring shadow cascades
        DirectionalLightShadowMap,   // Resource controlling shadow map resolution
    },
    prelude::*,
};
// Import mathematical constants like PI and FRAC_PI_4 (π/4)
use std::f32::consts::*;

fn main() {
    App::new()
        // Configure shadow map resolution to 4096x4096 pixels
        // Higher resolution = sharper shadows but more GPU memory
        // Default is 2048, we're using 4096 for better quality
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // Animate the sun/directional light to show dynamic shadows
        .add_systems(Update, animate_light_direction)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera with environment lighting
    commands.spawn((
        Camera3d::default(),
        // Position camera to get a nice 3/4 view of the helmet
        Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        // Environment Map Light (Image-Based Lighting)
        // This simulates light coming from all directions using HDR images
        EnvironmentMapLight {
            // Diffuse map: For rough/matte surfaces - heavily blurred environment
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            // Specular map: For shiny surfaces - less blurred for reflections
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            // Brightness multiplier
            intensity: 250.0,
            ..default()
        },
    ));

    // Spawn directional light (sun light) with shadows
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // Configure shadow cascades
        // Cascade shadows split the view frustum into segments,
        // each with its own shadow map for better quality
        CascadeShadowConfigBuilder {
            // Use only 1 cascade (default is 4)
            // Good for small scenes, saves performance
            num_cascades: 1,
            // Shadows only needed within 1.6 units of camera
            // Smaller distance = better shadow resolution
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));
    
    // Load the glTF model
    commands.spawn(SceneRoot(asset_server.load(
        // GltfAssetLabel::Scene(0) loads the first (index 0) scene from the file
        // Most glTF files have a single scene, but they can contain multiple
        GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf"),
    )));
}

// Animate the directional light to simulate a moving sun
// This creates dynamic shadows that move across the model
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        // Create rotation using Euler angles (rotations around axes)
        transform.rotation = Quat::from_euler(
            // EulerRot::ZYX means apply rotations in order: Z, then Y, then X
            EulerRot::ZYX,
            0.0,                              // No rotation around Z
            time.elapsed_secs() * PI / 5.0,   // Rotate around Y based on time
                                             // PI/5 radians per second = full rotation every 10 seconds
            -FRAC_PI_4,                      // Tilt down 45 degrees (π/4 radians)
        );
    }
}
