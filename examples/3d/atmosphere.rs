//! This example showcases pbr atmospheric scattering
//!
//! 🌅 The Poetry of Light Through Air
//!
//! Have you ever wondered why the sky is blue during the day and red at sunset?
//! This example demonstrates atmospheric scattering - the beautiful physics that
//! creates our colorful skies. As sunlight travels through Earth's atmosphere,
//! it collides with gas molecules and scatters, with blue light scattering more
//! than red. This is why we see blue skies and orange sunsets!
//!
//! 🎨 What You'll See:
//! - A mountainous terrain bathed in realistic sunlight
//! - The sun rotating through the sky, creating a day/night cycle
//! - Beautiful sky colors that change based on sun position
//! - Accurate atmospheric haze that makes distant objects appear bluish
//! - Two probe spheres (metallic and rough) showing how materials look under sky lighting
//!
//! 🔑 Key Concepts:
//! - Rayleigh Scattering: Why the sky is blue
//! - Mie Scattering: Why clouds and haze appear white
//! - Aerial Perspective: How atmosphere affects distant objects
//! - Physical Units: Working with real-world light values

use std::f32::consts::PI;

use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    pbr::{light_consts::lux, Atmosphere, AtmosphereSettings, CascadeShadowConfigBuilder},
    prelude::*,
    render::camera::Exposure,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera_fog, setup_terrain_scene))
        .add_systems(Update, dynamic_scene)
        .run();
}

// 🎥 Camera Setup with Atmospheric Scattering
//
// This function sets up our viewpoint with realistic atmospheric effects.
// Think of it as configuring a virtual camera that can see air itself!
fn setup_camera_fog(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // 📍 Position camera low, looking slightly upward for dramatic sky view
        Transform::from_xyz(-1.2, 0.15, 0.0).looking_at(Vec3::Y * 0.1, Vec3::Y),
        
        // 🌍 THE MAGIC: Enable Earth-like atmospheric scattering
        // This single component adds all the complex light physics!
        Atmosphere::EARTH,
        
        // 🔧 Fine-tune the atmosphere simulation
        AtmosphereSettings {
            // 📏 Maximum distance for aerial perspective calculation (320km)
            // This controls how far we can see through the atmosphere
            aerial_view_lut_max_distance: 3.2e5,
            
            // 🗺️ Scene scale: 1 unit = 10km
            // Our terrain is modeled in 10km units, so we tell the
            // atmosphere system to scale accordingly
            scene_units_to_m: 1e+4,
            
            ..Default::default()
        },
        
        // ☀️ Exposure for bright daylight
        // Real sunlight is VERY bright, so we need proper exposure
        // to avoid everything appearing white
        Exposure::SUNLIGHT,
        
        // 🎨 Tone mapping: Convert HDR colors to screen colors
        // ACES Fitted gives a cinematic, film-like look
        Tonemapping::AcesFitted,
        
        // ✨ Bloom: Makes bright lights glow naturally
        // Essential for making the sun look realistic
        Bloom::NATURAL,
    ));
}

// 🏔️ Marker for our terrain entity
#[derive(Component)]
struct Terrain;

// 🌄 Scene Setup: Mountains Under the Sky
fn setup_terrain_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 🎬 Configure shadow cascades for our scale
    // Since our scene uses kilometers as units, we need smaller
    // cascade distances than the defaults
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,  // 3km in our scale
        maximum_distance: 3.0,          // 30km in our scale
        ..default()
    }
    .build();

    // ☀️ The Sun: Our Primary Light Source
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            // 🌟 RAW_SUNLIGHT: The unfiltered power of the sun!
            //
            // This is crucial - we use the sun's illuminance BEFORE
            // atmospheric filtering. The atmosphere system will then
            // realistically scatter and absorb this light, creating
            // the proper colors we see from Earth's surface.
            //
            // Fun fact: Direct sunlight is about 128,000 lux!
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        // 🔄 Initial sun position: mid-morning
        Transform::from_xyz(1.0, -0.4, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        cascade_shadow_config,
    ));

    // 🔮 Create sphere mesh for our material probes
    let sphere_mesh = meshes.add(Mesh::from(Sphere { radius: 1.0 }));

    // 🪩 Metallic Sphere: Perfect Mirror
    // Shows how the sky reflects off shiny surfaces
    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 1.0,                 // Full metal
            perceptual_roughness: 0.0,     // Mirror finish
            ..default()
        })),
        Transform::from_xyz(-0.3, 0.1, -0.1).with_scale(Vec3::splat(0.05)),
    ));

    // 🏐 Rough Sphere: Matte Surface
    // Shows how diffuse materials capture atmospheric light
    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            metallic: 0.0,                 // Non-metal
            perceptual_roughness: 1.0,     // Completely rough
            ..default()
        })),
        Transform::from_xyz(-0.3, 0.1, 0.1).with_scale(Vec3::splat(0.05)),
    ));

    // 🏔️ Mountain Terrain
    // Large-scale geometry to showcase aerial perspective
    commands.spawn((
        Terrain,
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/terrain/terrain.glb")),
        ),
        Transform::from_xyz(-1.0, 0.0, -0.5)
            .with_scale(Vec3::splat(0.5))
            .with_rotation(Quat::from_rotation_y(PI / 2.0)),
    ));
}

// 🌅 Animate the Sun for Day/Night Cycle
//
// This creates a time-lapse effect, rotating the sun to show
// how atmospheric colors change throughout the day
fn dynamic_scene(mut suns: Query<&mut Transform, With<DirectionalLight>>, time: Res<Time>) {
    suns.iter_mut()
        .for_each(|mut tf| {
            // 📐 Rotate sun around X axis
            // PI/10 radians per second = full rotation in 20 seconds
            tf.rotate_x(-time.delta_secs() * PI / 10.0)
        });
}

// 🎓 Deep Dive: The Science of Atmospheric Scattering
//
// When light enters Earth's atmosphere, it interacts with gas molecules
// and particles in two main ways:
//
// 1. **Rayleigh Scattering** (Gas Molecules)
//    - Affects short wavelengths (blue) more than long (red)
//    - Intensity ∝ 1/λ⁴ (wavelength to the fourth power!)
//    - This is why the sky is blue and sunsets are red
//    - Also creates the blue haze on distant mountains
//
// 2. **Mie Scattering** (Larger Particles)
//    - Affects all wavelengths roughly equally
//    - Creates white/gray appearance of clouds and fog
//    - Causes the bright halo around the sun
//
// The implementation uses precomputed lookup tables (LUTs) to efficiently
// calculate these complex light interactions in real-time.

// 💡 Artistic Tips:
//
// 1. **Golden Hour**: Sun at low angles creates warm, dramatic lighting
// 2. **Blue Hour**: Just after sunset, sky provides soft, even illumination
// 3. **Noon**: Harsh shadows but vibrant colors
// 4. **Scale Matters**: Larger scenes show more atmospheric depth
// 5. **Fog Enhancement**: Add DistanceFog for extra atmospheric density
//
// Try adjusting:
// - `scene_units_to_m`: Make scenes feel larger or smaller
// - Sun rotation speed: Create different time-of-day moods
// - Exposure: Balance between sky and terrain brightness
// - Add clouds: Particle systems can enhance the atmosphere effect