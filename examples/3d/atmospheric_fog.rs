//! This example showcases atmospheric fog
//!
//! 🌫️ The Art of Making Air Visible
//!
//! Think about a misty morning where distant trees fade into a soft haze.
//! That's atmospheric fog! Unlike simple distance fog that just fades to a
//! single color, atmospheric fog simulates how real air particles scatter light,
//! creating beautiful depth and mood in your scenes. It can make a small scene
//! feel vast, or create an intimate, mysterious atmosphere.
//!
//! 🎨 What You'll See:
//! - Mountain terrain that fades naturally into the distance
//! - Warm sunlight that creates a golden glow through the fog
//! - Interactive fog controls to see the effect in real-time
//! - How directional light influences fog color (sunset effect!)
//!
//! ## Controls
//!
//! | Key Binding        | Action                                 |
//! |:-------------------|:---------------------------------------|
//! | `Spacebar`         | Toggle Atmospheric Fog                 |
//! | `S`                | Toggle Directional Light Fog Influence |
//!
//! 🔑 Key Concepts:
//! - Atmospheric Extinction: How fog absorbs light over distance
//! - Atmospheric Inscattering: How fog glows with scattered sunlight
//! - Directional Light Influence: How the sun creates colored fog
//! - Visibility Distance: The point where objects become lost in fog

use bevy::{
    pbr::{CascadeShadowConfigBuilder, NotShadowCaster},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (setup_camera_fog, setup_terrain_scene, setup_instructions),
        )
        .add_systems(Update, toggle_system)
        .run();
}

// 🎥 Camera & Fog Configuration
//
// This is where we set up our viewpoint and the atmospheric fog that
// creates depth and mood in our mountain scene.
fn setup_camera_fog(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // 📍 Position camera for a nice view of the mountains
        Transform::from_xyz(-1.0, 0.1, 1.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        
        // 🌫️ THE STAR: Distance Fog Configuration
        DistanceFog {
            // 🎨 Base fog color - a soft blue-gray
            // This is what objects fade to in the distance
            color: Color::srgba(0.35, 0.48, 0.66, 1.0),
            
            // ☀️ Directional light contribution color
            // This warm color gets added where the sun shines through fog
            // Alpha controls the strength (0.5 = 50% influence)
            directional_light_color: Color::srgba(1.0, 0.95, 0.85, 0.5),
            
            // 📊 Light scattering concentration
            // Higher values = more focused glow around sun
            // Lower values = more spread out glow
            directional_light_exponent: 30.0,
            
            // 📐 Fog density falloff configuration
            falloff: FogFalloff::from_visibility_colors(
                // 👁️ Visibility distance (15 world units)
                // Objects beyond this distance have less than 5% contrast
                15.0,
                
                // 🌑 Extinction color - what remains after light is absorbed
                // Typically darker and more saturated than the base fog
                Color::srgb(0.35, 0.5, 0.66),
                
                // ☁️ Inscattering color - light scattered into view
                // Usually brighter and warmer, represents sunlight in fog
                Color::srgb(0.8, 0.844, 1.0),
            ),
        },
    ));
}

// 🏔️ Scene Setup: Mountains in the Mist
fn setup_terrain_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 🎬 Configure shadows for our scale
    // The mountain scene uses kilometers as units, so we adjust accordingly
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 0.3,  // Near shadows: 300m
        maximum_distance: 3.0,          // Far shadows: 3km
        ..default()
    }
    .build();

    // ☀️ Directional Light (The Sun)
    commands.spawn((
        DirectionalLight {
            // 🎨 Warm, slightly yellow sunlight
            color: Color::srgb(0.98, 0.95, 0.82),
            shadows_enabled: true,
            ..default()
        },
        // 🧭 Sun angle - low on horizon for dramatic lighting
        Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::new(-0.15, -0.05, 0.25), Vec3::Y),
        cascade_shadow_config,
    ));

    // 🏔️ Mountain Terrain
    // This model provides the perfect showcase for atmospheric fog
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/terrain/Mountains.gltf"),
    )));

    // 🌌 Sky Dome (Simple Cube Sky)
    // We use a large inverted cube as a simple sky
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            // 🎨 Neutral gray sky color
            base_color: Srgba::hex("888888").unwrap().into(),
            // 💡 Unlit - sky doesn't receive lighting
            unlit: true,
            // 🔄 No culling - visible from inside
            cull_mode: None,
            ..default()
        })),
        // 📏 Scale up to encompass the scene
        Transform::from_scale(Vec3::splat(20.0)),
        // 🚫 Sky shouldn't cast shadows
        NotShadowCaster,
    ));
}

// 📝 UI Instructions
fn setup_instructions(mut commands: Commands) {
    commands.spawn((Text::new("Press Spacebar to Toggle Atmospheric Fog.\nPress S to Toggle Directional Light Fog Influence."),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        })
    );
}

// 🎮 Interactive Fog Controls
//
// This system lets you toggle fog effects to understand their impact
fn toggle_system(keycode: Res<ButtonInput<KeyCode>>, mut fog: Single<&mut DistanceFog>) {
    // 🔄 Toggle fog on/off by animating alpha
    if keycode.just_pressed(KeyCode::Space) {
        let a = fog.color.alpha();
        fog.color.set_alpha(1.0 - a);  // Flip between 0 and 1
    }

    // ☀️ Toggle directional light influence
    if keycode.just_pressed(KeyCode::KeyS) {
        let a = fog.directional_light_color.alpha();
        fog.directional_light_color.set_alpha(0.5 - a);  // Flip between 0 and 0.5
    }
}

// 🎓 Deep Dive: Understanding Atmospheric Fog
//
// Real fog is created by tiny water droplets or particles suspended in air.
// These particles interact with light in two main ways:
//
// 1. **Extinction** (Absorption + Out-scattering)
//    - Light is absorbed or scattered away from the viewer
//    - Objects fade towards the extinction color
//    - Creates the "fading into fog" effect
//
// 2. **Inscattering** (In-scattering)
//    - Light from other sources (like the sun) scatters into view
//    - Adds a glow or brightness to the fog
//    - Creates the "glowing fog" effect in sunlight
//
// The math behind `FogFalloff::from_visibility_colors`:
// - Calculates extinction and inscattering coefficients
// - Based on desired visibility distance
// - Uses Beer-Lambert law for light attenuation
// - Ensures 5% contrast at visibility distance

// 💡 Artistic Tips for Fog:
//
// 1. **Morning Fog**: Cool colors, high inscattering, low visibility
// 2. **Sunset Fog**: Warm directional light color, high exponent
// 3. **Heavy Fog**: Very short visibility distance, gray colors
// 4. **Light Haze**: Long visibility, subtle blue extinction
// 5. **Pollution**: Brown/gray extinction, reduced inscattering
//
// Combine with:
// - Bloom for enhanced glow effects
// - Volumetric lighting for light shafts
// - Particle effects for localized fog patches
// - Animation for moving fog banks

// 🎯 Performance Notes:
//
// Distance fog is very efficient because it's calculated in the fragment shader:
// - No extra geometry or volumetric calculations
// - Simple distance-based interpolation
// - Minimal performance impact
// - Works great on all hardware levels
//
// For more complex fog effects, consider:
// - Volumetric fog for 3D density variations
// - Height fog for ground-hugging effects
// - Multiple fog layers for complex atmospheres