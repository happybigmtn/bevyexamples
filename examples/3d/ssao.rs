//! A scene showcasing screen space ambient occlusion.
//!
//! 🌑 The Art of Digital Shadows: Understanding SSAO
//!
//! Have you ever noticed how the corners of a room appear slightly darker, even
//! when there's no direct shadow? Or how the crevices in a sculpture seem to
//! hold onto darkness? That's ambient occlusion - the subtle darkening that
//! happens when surfaces block ambient light from reaching nearby areas. In the
//! real world, light bounces everywhere, but some places receive less of this
//! bounced light. SSAO (Screen Space Ambient Occlusion) is a clever trick that
//! approximates this effect using only what's visible on screen!
//!
//! 🎯 What You'll See:
//! - Three cubes arranged at right angles (like a corner)
//! - A floating sphere that bobs up and down
//! - Subtle darkening where surfaces meet
//! - Real-time quality adjustments from Off to Ultra
//!
//! 🎮 Controls:
//! - `1-5`: SSAO quality (Off/Low/Medium/High/Ultra)
//! - `Up/Down Arrows`: Adjust object thickness assumption
//! - `Space`: Toggle temporal anti-aliasing
//!
//! 🔑 Key Concepts:
//! - Screen Space: Works only with visible geometry
//! - Depth Buffer: Uses depth to estimate geometry
//! - Sampling: Tests nearby pixels for occlusion
//! - Contact Shadows: Darkening where objects meet

use bevy::{
    anti_aliasing::taa::TemporalAntiAliasing,
    math::ops,
    pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel},
    prelude::*,
    render::{camera::TemporalJitter, view::Hdr},
};
use std::f32::consts::PI;

fn main() {
    App::new()
        // 🌟 Bright ambient light to make SSAO effect more visible
        // Without ambient light, SSAO has nothing to occlude!
        .insert_resource(AmbientLight {
            brightness: 1000.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

// 🏗️ Scene Setup: Building Our SSAO Showcase
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 📸 Camera Configuration for SSAO
    commands.spawn((
        Camera3d::default(),
        // 🎥 Position camera to see the corner where cubes meet
        Transform::from_xyz(-2.0, 2.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y),
        // 🌟 HDR for better lighting range
        Hdr,
        // ⚠️ MSAA must be off for SSAO (it's a post-processing effect)
        Msaa::Off,
        // 🌑 Enable SSAO with default settings
        ScreenSpaceAmbientOcclusion::default(),
        // 🔄 TAA helps smooth out SSAO noise
        TemporalAntiAliasing::default(),
    ));

    // 🎨 Neutral gray material - perfect for seeing AO effects
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),  // Medium gray
        perceptual_roughness: 1.0,               // Fully rough (no reflections)
        reflectance: 0.0,                        // No specular highlights
        ..default()
    });
    
    // 📦 Three Cubes: Creating corners for ambient occlusion
    // The arrangement creates inside corners where AO is most visible
    
    // Cube 1: Back wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));
    
    // Cube 2: Floor
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));
    
    // Cube 3: Right wall
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(material),
        Transform::from_xyz(1.0, 0.0, 0.0),
    ));
    // 🔮 Floating Sphere: Shows dynamic contact shadows
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.4).mesh().uv(72, 36))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.4, 0.4),  // Slightly darker gray
            perceptual_roughness: 1.0,               // Fully rough
            reflectance: 0.0,                        // No specular
            ..default()
        })),
        SphereMarker,  // Tag for animation system
    ));

    // ☀️ Directional Light: Provides some direct lighting
    // SSAO complements regular shadows, doesn't replace them
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,  // Real shadows work alongside SSAO
            ..default()
        },
        // 🔄 Angled light for interesting shadows
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI * -0.15, PI * -0.15)),
    ));

    // 📝 UI Text: Shows current settings
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// 🎮 Update System: Handle Input and Animation
fn update(
    camera: Single<
        (
            Entity,
            Option<&ScreenSpaceAmbientOcclusion>,
            Option<&TemporalJitter>,
        ),
        With<Camera>,
    >,
    mut text: Single<&mut Text>,
    mut sphere: Single<&mut Transform, With<SphereMarker>>,
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    // 🔮 Animate sphere up and down to show dynamic AO
    // As it approaches the floor, watch the contact shadow appear!
    sphere.translation.y = ops::sin(time.elapsed_secs() / 1.7) * 0.7;

    // 📊 Extract camera components and current SSAO settings
    let (camera_entity, ssao, temporal_jitter) = *camera;
    let current_ssao = ssao.cloned().unwrap_or_default();

    // 🎛️ Quality Level Controls (Keys 2-5)
    // Higher quality = more samples = better looking but slower
    let mut commands = commands.entity(camera_entity);
    commands
        // Low: Fast, good for low-end hardware
        .insert_if(
            ScreenSpaceAmbientOcclusion {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Low,
                ..current_ssao
            },
            || keycode.just_pressed(KeyCode::Digit2),
        )
        // Medium: Balanced quality/performance
        .insert_if(
            ScreenSpaceAmbientOcclusion {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium,
                ..current_ssao
            },
            || keycode.just_pressed(KeyCode::Digit3),
        )
        // High: Good quality for most use cases
        .insert_if(
            ScreenSpaceAmbientOcclusion {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
                ..current_ssao
            },
            || keycode.just_pressed(KeyCode::Digit4),
        )
        // Ultra: Maximum quality, may impact performance
        .insert_if(
            ScreenSpaceAmbientOcclusion {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Ultra,
                ..current_ssao
            },
            || keycode.just_pressed(KeyCode::Digit5),
        )
        // 🔧 Object Thickness Controls (Up/Down Arrows)
        // This tells SSAO how thick objects are assumed to be
        // Higher values = wider occlusion areas
        .insert_if(
            ScreenSpaceAmbientOcclusion {
                constant_object_thickness: (current_ssao.constant_object_thickness * 2.0).min(4.0),
                ..current_ssao
            },
            || keycode.just_pressed(KeyCode::ArrowUp),
        )
        .insert_if(
            ScreenSpaceAmbientOcclusion {
                constant_object_thickness: (current_ssao.constant_object_thickness * 0.5)
                    .max(0.0625),
                ..current_ssao
            },
            || keycode.just_pressed(KeyCode::ArrowDown),
        );
    
    // 🚫 Key 1: Disable SSAO completely
    if keycode.just_pressed(KeyCode::Digit1) {
        commands.remove::<ScreenSpaceAmbientOcclusion>();
    }
    
    // 🔄 Space: Toggle Temporal Anti-Aliasing
    // TAA helps reduce SSAO noise by blending frames over time
    if keycode.just_pressed(KeyCode::Space) {
        if temporal_jitter.is_some() {
            commands.remove::<TemporalJitter>();
        } else {
            commands.insert(TemporalJitter::default());
        }
    }

    // 📝 Update UI Text
    text.clear();

    // 🌟 Create asterisk indicators for current quality level
    let (o, l, m, h, u) = match ssao.map(|s| s.quality_level) {
        None => ("*", "", "", "", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::Low) => ("", "*", "", "", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::Medium) => ("", "", "*", "", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::High) => ("", "", "", "*", ""),
        Some(ScreenSpaceAmbientOcclusionQualityLevel::Ultra) => ("", "", "", "", "*"),
        _ => unreachable!(),
    };

    // 📏 Display current object thickness if SSAO is enabled
    if let Some(thickness) = ssao.map(|s| s.constant_object_thickness) {
        text.push_str(&format!(
            "Constant object thickness: {} (Up/Down)\n\n",
            thickness
        ));
    }

    // 🎨 Display quality options with current selection marked
    text.push_str("SSAO Quality:\n");
    text.push_str(&format!("(1) {o}Off{o}\n"));
    text.push_str(&format!("(2) {l}Low{l}\n"));
    text.push_str(&format!("(3) {m}Medium{m}\n"));
    text.push_str(&format!("(4) {h}High{h}\n"));
    text.push_str(&format!("(5) {u}Ultra{u}\n\n"));

    // 🔄 Display TAA status
    text.push_str("Temporal Antialiasing:\n");
    text.push_str(match temporal_jitter {
        Some(_) => "(Space) Enabled",
        None => "(Space) Disabled",
    });
}

// 🏷️ Marker component for the animated sphere
#[derive(Component)]
struct SphereMarker;

// 🎓 Deep Dive: How Screen Space Ambient Occlusion Works
//
// **The Problem**:
// In real life, ambient light (indirect lighting) doesn't reach everywhere
// equally. Corners, crevices, and contact points receive less ambient light
// because nearby surfaces block it. Calculating this accurately requires
// complex global illumination - too expensive for real-time!
//
// **The SSAO Solution**:
// 1. For each pixel on screen, we know its 3D position (from depth buffer)
// 2. Sample random points in a hemisphere around that position
// 3. Check how many sample points are "inside" geometry (using depth)
// 4. The more occluded samples, the darker the pixel
//
// **The Algorithm**:
// ```
// for each pixel:
//     position = reconstruct_3d_position(depth)
//     occlusion = 0
//     for each sample in hemisphere:
//         sample_pos = position + sample_offset
//         if (sample_depth > scene_depth):
//             occlusion += 1
//     pixel_ao = 1.0 - (occlusion / num_samples)
// ```
//
// **Quality Levels**:
// - Low: ~16 samples, larger radius
// - Medium: ~32 samples, medium radius
// - High: ~64 samples, smaller radius
// - Ultra: ~80+ samples, multiple radii
//
// **Object Thickness**:
// Since we only see the front surfaces, we must assume how thick objects are.
// This prevents occlusion from appearing behind thin objects.

// 💡 SSAO Artifacts and Solutions:
//
// **Noise/Grain**:
// - Caused by using random samples
// - Solution: Temporal filtering (TAA), blur passes
//
// **Halos**:
// - Dark borders around objects against sky
// - Solution: Depth-aware blur, better sampling
//
// **Flatness**:
// - Loss of detail in the AO
// - Solution: Multiple sampling radii, bent normals
//
// **Performance Tips**:
// - Half-resolution rendering (compute at 50% size)
// - Temporal caching (reuse previous frames)
// - Interleaved sampling (different samples per pixel)
// - Depth-aware upsampling

// 🎮 When to Use SSAO:
//
// **Great for**:
// - Indoor scenes (lots of corners)
// - Mechanical objects (many crevices)
// - Architectural visualization
// - Enhancing depth perception
//
// **Less effective for**:
// - Outdoor scenes with few occluders
// - Very dark scenes (AO barely visible)
// - Fast-paced action (can be distracting)
// - Mobile/low-end hardware
