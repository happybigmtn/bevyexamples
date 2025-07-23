//! Illustrates bloom post-processing using HDR and emissive materials.
//!
//! ✨ The Magic of Digital Glow: Understanding Bloom
//!
//! Have you ever noticed how bright lights seem to glow and spill beyond their
//! boundaries in movies and games? That's bloom! It simulates how extremely bright
//! light overwhelms camera sensors (or our eyes), creating a dreamy, ethereal glow.
//! Think of looking at streetlights on a foggy night - that soft halo effect is
//! what we're recreating digitally.
//!
//! 🎨 What You'll See:
//! - A field of spheres with different emissive intensities
//! - Bright spheres creating beautiful glowing halos
//! - Spheres gently bouncing in a wave pattern
//! - Real-time control over all bloom parameters
//!
//! 🎮 Controls:
//! - `Space`: Toggle bloom on/off
//! - `Q/A`: Adjust intensity (overall bloom strength)
//! - `W/S`: Adjust low-frequency boost (larger glow size)
//! - `E/D`: Adjust boost curvature (glow falloff shape)
//! - `R/F`: Adjust high-pass frequency (detail preservation)
//! - `T/G`: Switch between Energy-conserving/Additive modes
//! - `Y/H`: Adjust threshold (minimum brightness for bloom)
//! - `U/J`: Adjust threshold softness (transition smoothness)
//! - `I/K`: Adjust horizontal scale (horizontal bloom stretch)
//!
//! 🔑 Key Concepts:
//! - HDR (High Dynamic Range): Allows brightness beyond screen limits
//! - Emissive Materials: Self-illuminating surfaces
//! - Tone Mapping: Converting HDR values to displayable range
//! - Multi-resolution Blur: Creating realistic light spread

use bevy::{
    core_pipeline::{
        bloom::{Bloom, BloomCompositeMode},
        tonemapping::Tonemapping,
    },
    math::ops,
    prelude::*,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (update_bloom_settings, bounce_spheres))
        .run();
}

// 🎬 Scene Setup: Creating a Glowing Sphere Field
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 📷 Camera Configuration for Bloom
    commands.spawn((
        Camera3d::default(),
        Camera {
            // 🌑 Black background makes bloom more visible
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        // 🎨 Tone mapping is crucial for bloom!
        // TonyMcMapface desaturates to white, perfect for bloom
        Tonemapping::TonyMcMapface,
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        // ✨ THE STAR: Enable bloom with natural settings
        Bloom::NATURAL,
    ));

    // 🌟 Create Emissive Materials with Different Intensities
    
    // 💙 Cool blue glow - moderate intensity
    let material_emissive1 = materials.add(StandardMaterial {
        // 🔑 Emissive colors use LINEAR color space for accurate light math
        emissive: LinearRgba::rgb(0.0, 0.0, 150.0),
        ..default()
    });
    
    // ☀️ Intense white glow - VERY bright!
    let material_emissive2 = materials.add(StandardMaterial {
        // 💡 Values > 1.0 are possible with HDR!
        emissive: LinearRgba::rgb(1000.0, 1000.0, 1000.0),
        ..default()
    });
    
    // ❤️ Warm red glow - subtle intensity
    let material_emissive3 = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(50.0, 0.0, 0.0),
        ..default()
    });
    
    // ⚫ Non-emissive material for contrast
    let material_non_emissive = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        ..default()
    });

    // 🌐 High-quality sphere mesh
    let mesh = meshes.add(Sphere::new(0.4).mesh().ico(5).unwrap());

    // 🎲 Create a Grid of Spheres with Pseudo-Random Materials
    for x in -5..5 {
        for z in -5..5 {
            // 🎯 Deterministic randomness based on position
            // This ensures the same sphere always has the same material
            let mut hasher = DefaultHasher::new();
            (x, z).hash(&mut hasher);
            let rand = (hasher.finish() + 3) % 6;

            // 🎨 Select material and scale based on "random" value
            let (material, scale) = match rand {
                0 => (material_emissive1.clone(), 0.5),    // Small blue orbs
                1 => (material_emissive2.clone(), 0.1),    // Tiny bright stars
                2 => (material_emissive3.clone(), 1.0),    // Medium red spheres
                3..=5 => (material_non_emissive.clone(), 1.5), // Large dark spheres
                _ => unreachable!(),
            };

            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_xyz(x as f32 * 2.0, 0.0, z as f32 * 2.0)
                    .with_scale(Vec3::splat(scale)),
                // 🏀 Mark for bouncing animation
                Bouncing,
            ));
        }
    }

    // 📝 UI for controls display
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

// 🎛️ Interactive Bloom Control System
//
// This system provides real-time control over every bloom parameter,
// perfect for understanding how each setting affects the final look!
fn update_bloom_settings(
    camera: Single<(Entity, Option<&mut Bloom>), With<Camera>>,
    mut text: Single<&mut Text>,
    mut commands: Commands,
    keycode: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let bloom = camera.into_inner();

    match bloom {
        (entity, Some(mut bloom)) => {
            // 📊 Display current bloom settings
            text.0 = "Bloom (Toggle: Space)\n".to_string();
            text.push_str(&format!("(Q/A) Intensity: {}\n", bloom.intensity));
            text.push_str(&format!(
                "(W/S) Low-frequency boost: {}\n",
                bloom.low_frequency_boost
            ));
            text.push_str(&format!(
                "(E/D) Low-frequency boost curvature: {}\n",
                bloom.low_frequency_boost_curvature
            ));
            text.push_str(&format!(
                "(R/F) High-pass frequency: {}\n",
                bloom.high_pass_frequency
            ));
            text.push_str(&format!(
                "(T/G) Mode: {}\n",
                match bloom.composite_mode {
                    BloomCompositeMode::EnergyConserving => "Energy-conserving",
                    BloomCompositeMode::Additive => "Additive",
                }
            ));
            text.push_str(&format!("(Y/H) Threshold: {}\n", bloom.prefilter.threshold));
            text.push_str(&format!(
                "(U/J) Threshold softness: {}\n",
                bloom.prefilter.threshold_softness
            ));
            text.push_str(&format!("(I/K) Horizontal Scale: {}\n", bloom.scale.x));

            // 🔄 Toggle bloom on/off
            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).remove::<Bloom>();
            }

            let dt = time.delta_secs();

            // 🎨 Intensity: Overall bloom strength (0-1)
            // Higher = more pronounced glow
            if keycode.pressed(KeyCode::KeyA) {
                bloom.intensity -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyQ) {
                bloom.intensity += dt / 10.0;
            }
            bloom.intensity = bloom.intensity.clamp(0.0, 1.0);

            // 🌊 Low-frequency boost: Enhances larger, softer glows
            // Higher = bigger bloom radius
            if keycode.pressed(KeyCode::KeyS) {
                bloom.low_frequency_boost -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyW) {
                bloom.low_frequency_boost += dt / 10.0;
            }
            bloom.low_frequency_boost = bloom.low_frequency_boost.clamp(0.0, 1.0);

            // 📐 Boost curvature: Controls falloff shape
            // Higher = sharper falloff at edges
            if keycode.pressed(KeyCode::KeyD) {
                bloom.low_frequency_boost_curvature -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyE) {
                bloom.low_frequency_boost_curvature += dt / 10.0;
            }
            bloom.low_frequency_boost_curvature =
                bloom.low_frequency_boost_curvature.clamp(0.0, 1.0);

            // 🔍 High-pass frequency: Preserves sharp details
            // Higher = less bloom on fine details
            if keycode.pressed(KeyCode::KeyF) {
                bloom.high_pass_frequency -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyR) {
                bloom.high_pass_frequency += dt / 10.0;
            }
            bloom.high_pass_frequency = bloom.high_pass_frequency.clamp(0.0, 1.0);

            // 🎭 Composite mode: How bloom combines with the scene
            if keycode.pressed(KeyCode::KeyG) {
                bloom.composite_mode = BloomCompositeMode::Additive;
            }
            if keycode.pressed(KeyCode::KeyT) {
                bloom.composite_mode = BloomCompositeMode::EnergyConserving;
            }

            // 🚪 Threshold: Minimum brightness to trigger bloom
            // Lower = more things bloom
            if keycode.pressed(KeyCode::KeyH) {
                bloom.prefilter.threshold -= dt;
            }
            if keycode.pressed(KeyCode::KeyY) {
                bloom.prefilter.threshold += dt;
            }
            bloom.prefilter.threshold = bloom.prefilter.threshold.max(0.0);

            // 🌈 Threshold softness: Smooth transition at threshold
            // Higher = smoother bloom onset
            if keycode.pressed(KeyCode::KeyJ) {
                bloom.prefilter.threshold_softness -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyU) {
                bloom.prefilter.threshold_softness += dt / 10.0;
            }
            bloom.prefilter.threshold_softness = bloom.prefilter.threshold_softness.clamp(0.0, 1.0);

            // ↔️ Horizontal scale: Anamorphic bloom effect
            // >1 = horizontal streaks (like movie lenses)
            if keycode.pressed(KeyCode::KeyK) {
                bloom.scale.x -= dt * 2.0;
            }
            if keycode.pressed(KeyCode::KeyI) {
                bloom.scale.x += dt * 2.0;
            }
            bloom.scale.x = bloom.scale.x.clamp(0.0, 8.0);
        }

        (entity, None) => {
            // 🚫 Bloom is disabled
            text.0 = "Bloom: Off (Toggle: Space)".to_string();

            if keycode.just_pressed(KeyCode::Space) {
                commands.entity(entity).insert(Bloom::NATURAL);
            }
        }
    }
}

// 🏀 Marker component for bouncing spheres
#[derive(Component)]
struct Bouncing;

// 🌊 Sphere Animation System
//
// Creates a mesmerizing wave effect across the sphere field
fn bounce_spheres(time: Res<Time>, mut query: Query<&mut Transform, With<Bouncing>>) {
    for mut transform in query.iter_mut() {
        // 📐 Sine wave based on position and time
        // This creates a diagonal wave pattern across the field
        transform.translation.y =
            ops::sin(transform.translation.x + transform.translation.z + time.elapsed_secs());
    }
}

// 🎓 Deep Dive: How Bloom Works
//
// Bloom is a multi-step process that simulates camera/eye lens imperfections:
//
// 1. **Bright Pass Filter**: Extract only pixels above threshold
//    - Threshold determines what's "bright enough" to bloom
//    - Softness creates smooth transitions
//
// 2. **Multi-Resolution Blur**: Create glow at different scales
//    - Downsamples image multiple times (1/2, 1/4, 1/8, etc.)
//    - Blurs each resolution separately
//    - Combines all scales for natural falloff
//
// 3. **Composite**: Blend bloom with original image
//    - Additive: Simply adds glow (can blow out colors)
//    - Energy-conserving: Maintains overall brightness
//
// The math behind the glow:
// - Gaussian blur creates the soft spread
// - Multiple resolutions prevent "banding"
// - HDR values allow super-bright sources

// 💡 Artistic Uses of Bloom:
//
// 1. **Sci-Fi**: Intense bloom on energy weapons and shields
// 2. **Fantasy**: Soft bloom on magic and enchantments
// 3. **Horror**: Minimal bloom for harsh, stark lighting
// 4. **Dream Sequences**: Heavy bloom for ethereal feel
// 5. **Retro**: Horizontal bloom mimics old CRT monitors
//
// Performance Tips:
// - Lower resolution = faster (use scale)
// - Fewer blur passes = faster (adjust quality)
// - Higher threshold = fewer pixels to process
// - Energy-conserving mode is slightly slower

// 🎮 Common Bloom Settings:
//
// **Natural** (default):
// - Intensity: 0.15
// - Low-freq boost: 0.7
// - Threshold: 1.0
// - Good for realistic scenes
//
// **Dreamy**:
// - Intensity: 0.3
// - Low-freq boost: 0.9
// - Threshold: 0.5
// - Soft, ethereal look
//
// **Cinematic**:
// - Intensity: 0.1
// - Scale: (2.0, 1.0)
// - Creates anamorphic lens flares