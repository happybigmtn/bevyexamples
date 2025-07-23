//! Illustrates different lights of various types and colors, some static, some moving over
//! a simple scene.
//!
//! 💡 The Art of Digital Illumination: Understanding 3D Lighting
//!
//! Imagine you're a stage lighting designer for a theater. You have spotlights,
//! floodlights, and colored gels at your disposal. Each light serves a different
//! purpose - some illuminate the whole stage, others highlight specific actors,
//! and some create mood with color. That's exactly what we're doing here, but
//! in a 3D world! This example showcases every type of light Bevy offers, all
//! working together in a colorful dance.
//!
//! 🎯 What You'll See:
//! - A simple room with colored walls and objects
//! - Four different types of lights in action:
//!   - Ambient light (orange glow everywhere)
//!   - Point lights (red and blue spheres of light)
//!   - Spot light (green cone of light)
//!   - Directional light (rotating sun)
//! - Real-time camera exposure controls (like a DSLR!)
//! - Moving objects casting dynamic shadows
//!
//! 🎮 Controls:
//! - `Arrow Keys`: Move objects around
//! - `1/2`: Decrease/Increase aperture (f-stop)
//! - `3/4`: Decrease/Increase shutter speed
//! - `5/6`: Decrease/Increase ISO sensitivity
//! - `R`: Reset camera exposure to defaults
//!
//! 🔑 Key Concepts:
//! - Light Types: Ambient, Point, Spot, and Directional
//! - Physical Camera Parameters: Simulating real camera behavior
//! - Shadow Casting: How different lights create shadows
//! - Light Intensity: Measured in realistic units (lumens/lux)
//! - Emissive Materials: Objects that glow

use std::f32::consts::PI;

use bevy::{
    color::palettes::css::*,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::camera::{Exposure, PhysicalCameraParameters},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // 📷 Physical Camera Settings - Like a Real DSLR!
        .insert_resource(Parameters(PhysicalCameraParameters {
            aperture_f_stops: 1.0,      // f/1.0 - Very wide aperture
            shutter_speed_s: 1.0 / 125.0,  // 1/125s - Standard daylight speed
            sensitivity_iso: 100.0,     // ISO 100 - Low sensitivity, less noise
            sensor_height: 0.01866,     // APS-C sensor size (in meters)
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (update_exposure, movement, animate_light_direction))
        .run();
}

// 📸 Camera Parameters Wrapper
#[derive(Resource, Default, Deref, DerefMut)]
struct Parameters(PhysicalCameraParameters);

// 🏃 Movable Component - For objects we can control
#[derive(Component)]
struct Movable;

// 🏗️ Scene Setup: Building Our Lighting Showcase
/// set up a simple 3D scene
fn setup(
    parameters: Res<Parameters>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 🏠 Room Construction
    
    // 🟦 Ground Plane - White floor to show colored light reflections
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,  // Rough surface scatters light
            ..default()
        })),
    ));

    // 🟪 Left Wall - Indigo colored for contrast
    let mut transform = Transform::from_xyz(2.5, 2.5, 0.0);
    transform.rotate_z(PI / 2.);  // Rotate to vertical
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.0, 0.15, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: INDIGO.into(),
            perceptual_roughness: 1.0,
            ..default()
        })),
        transform,
    ));
    
    // 🟪 Back Wall - Another indigo wall to catch shadows
    let mut transform = Transform::from_xyz(0.0, 2.5, -2.5);
    transform.rotate_x(PI / 2.);  // Rotate to vertical
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.0, 0.15, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: INDIGO.into(),
            perceptual_roughness: 1.0,
            ..default()
        })),
        transform,
    ));

    // 🎨 Test Objects
    
    // 🦫 Bevy Logo - Demonstrates Alpha Mask Shadows!
    // Alpha masking lets textures cast accurate shadows through transparent areas
    let mut transform = Transform::from_xyz(-2.2, 0.5, 1.0);
    transform.rotate_y(PI / 8.);
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(2.0, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("branding/bevy_logo_light.png")),
            perceptual_roughness: 1.0,
            alpha_mode: AlphaMode::Mask(0.5),  // Pixels < 50% opacity are invisible
            cull_mode: None,  // Render both sides
            ..default()
        })),
        transform,
        Movable,  // Can be moved with arrow keys
    ));

    // 🟩 Pink Cube - Shows how different lights affect flat surfaces
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: DEEP_PINK.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Movable,
    ));
    
    // 🟢 Green Sphere - Shows how lights affect curved surfaces
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5).mesh().uv(32, 18))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: LIMEGREEN.into(),
            ..default()
        })),
        Transform::from_xyz(1.5, 1.0, 1.5),
        Movable,
    ));

    // 🌅 Ambient Light: The Foundation
    // Like the general glow in a room even without direct lights
    // This prevents complete darkness in unlit areas
    commands.insert_resource(AmbientLight {
        color: ORANGE_RED.into(),  // Warm ambient for cozy feeling
        brightness: 0.02,          // Very dim - just enough to see
        ..default()
    });

    // 🔴 Red Point Light: Like a Glowing Bulb
    // Point lights emit light equally in all directions from a single point
    commands.spawn((
        PointLight {
            intensity: 100_000.0,      // Lumens - brightness of a 100W bulb
            color: RED.into(),
            shadows_enabled: true,     // Cast realistic shadows
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 0.0),
        // 🔴 Visual indicator - a glowing red sphere
        children![(
            Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(32, 18))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: RED.into(),
                emissive: LinearRgba::new(4.0, 0.0, 0.0, 0.0),  // Glows red!
                ..default()
            })),
        )],
    ));

    // 🟢 Green Spot Light: Like a Flashlight
    // Spot lights emit a cone of light in a specific direction
    commands.spawn((
        SpotLight {
            intensity: 100_000.0,      // Lumens
            color: LIME.into(),
            shadows_enabled: true,
            inner_angle: 0.6,          // Full brightness cone (radians)
            outer_angle: 0.8,          // Falloff to darkness (radians)
            ..default()
        },
        Transform::from_xyz(-1.0, 2.0, 0.0)
            .looking_at(Vec3::new(-1.0, 0.0, 0.0), Vec3::Z),  // Aim the spotlight
        // 🟢 Visual indicator - a glowing green capsule
        children![(
            Mesh3d(meshes.add(Capsule3d::new(0.1, 0.125))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: LIME.into(),
                emissive: LinearRgba::new(0.0, 4.0, 0.0, 0.0),  // Glows green!
                ..default()
            })),
            Transform::from_rotation(Quat::from_rotation_x(PI / 2.0)),
        )],
    ));

    // 🔵 Blue Point Light: Another Glowing Bulb
    // This one is positioned high up to create interesting shadow angles
    commands.spawn((
        PointLight {
            intensity: 100_000.0,
            color: BLUE.into(),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 4.0, 0.0),  // High position
        // 🔵 Visual indicator - a glowing blue sphere
        children![(
            Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(32, 18))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: BLUE.into(),
                emissive: LinearRgba::new(0.0, 0.0, 713.0, 0.0),  // VERY bright blue!
                ..default()
            })),
        )],
    ));

    // ☀️ Directional Light: The Sun
    // Directional lights simulate infinitely distant light sources
    // All rays are parallel - perfect for sun/moon
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,  // Realistic daylight levels
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),  // Angled like afternoon sun
            ..default()
        },
        // 🗺️ Cascaded Shadow Maps: Better shadows for large scenes
        // Splits the view into multiple shadow maps for quality + performance
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,   // High detail shadows up to 4 units
            maximum_distance: 10.0,         // Shadows visible up to 10 units
            ..default()
        }
        .build(),
    ));

    // 📝 UI Instructions and Camera Info

    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        children![
            // 📷 Current camera settings display
            TextSpan(format!("Aperture: f/{:.0}\n", parameters.aperture_f_stops,)),
            TextSpan(format!(
                "Shutter speed: 1/{:.0}s\n",
                1.0 / parameters.shutter_speed_s
            )),
            TextSpan(format!(
                "Sensitivity: ISO {:.0}\n",
                parameters.sensitivity_iso
            )),
            TextSpan::new("\n\n"),
            TextSpan::new("Controls\n"),
            TextSpan::new("---------------\n"),
            TextSpan::new("Arrow keys - Move objects\n"),
            TextSpan::new("1/2 - Decrease/Increase aperture\n"),
            TextSpan::new("3/4 - Decrease/Increase shutter speed\n"),
            TextSpan::new("5/6 - Decrease/Increase sensitivity\n"),
            TextSpan::new("R - Reset exposure"),
        ],
    ));

    // 📸 Camera Setup with Physical Parameters
    // Using real camera settings for realistic exposure
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Exposure::from_physical_camera(**parameters),  // Apply physical camera settings
    ));
}

// 📷 Camera Exposure Control System
// Simulates real camera controls for realistic lighting response
fn update_exposure(
    key_input: Res<ButtonInput<KeyCode>>,
    mut parameters: ResMut<Parameters>,
    mut exposure: Single<&mut Exposure>,
    text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
) {
    let entity = *text;
    
    // 🔍 Aperture Control (f-stop)
    // Wider aperture (lower f-number) = more light
    if key_input.just_pressed(KeyCode::Digit2) {
        parameters.aperture_f_stops *= 2.0;  // Smaller aperture (less light)
    } else if key_input.just_pressed(KeyCode::Digit1) {
        parameters.aperture_f_stops *= 0.5;  // Wider aperture (more light)
    }
    
    // ⏱️ Shutter Speed Control
    // Slower shutter = more light (but motion blur in real cameras!)
    if key_input.just_pressed(KeyCode::Digit4) {
        parameters.shutter_speed_s *= 2.0;   // Slower shutter (more light)
    } else if key_input.just_pressed(KeyCode::Digit3) {
        parameters.shutter_speed_s *= 0.5;   // Faster shutter (less light)
    }
    
    // 🎞️ ISO Sensitivity Control
    // Higher ISO = more sensitive to light (but more noise in real cameras!)
    if key_input.just_pressed(KeyCode::Digit6) {
        parameters.sensitivity_iso += 100.0;  // More sensitive
    } else if key_input.just_pressed(KeyCode::Digit5) {
        parameters.sensitivity_iso -= 100.0;  // Less sensitive
    }
    
    // 🔄 Reset to defaults
    if key_input.just_pressed(KeyCode::KeyR) {
        *parameters = Parameters::default();
    }

    // 📊 Update UI display
    *writer.text(entity, 1) = format!("Aperture: f/{:.0}\n", parameters.aperture_f_stops);
    *writer.text(entity, 2) = format!(
        "Shutter speed: 1/{:.0}s\n",
        1.0 / parameters.shutter_speed_s
    );
    *writer.text(entity, 3) = format!("Sensitivity: ISO {:.0}\n", parameters.sensitivity_iso);

    // 🎨 Apply new exposure settings to camera
    **exposure = Exposure::from_physical_camera(**parameters);
}

// 🌅 Sun Animation System
// Rotates the directional light to simulate time passing
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.5);  // Gentle rotation
    }
}

// 🏃 Object Movement System
// Move objects with arrow keys to see how shadows change
fn movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Movable>>,
) {
    for mut transform in &mut query {
        let mut direction = Vec3::ZERO;
        
        // ⬆️⬇️ Vertical movement
        if input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        
        // ⬅️➡️ Horizontal movement
        if input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        // Apply movement scaled by time for smooth motion
        transform.translation += time.delta_secs() * 2.0 * direction;
    }
}

// 🎓 Deep Dive: Understanding Light Types
//
// **Ambient Light**: 
// - No direction, affects everything equally
// - Simulates indirect light bouncing around
// - Use sparingly - too much makes scenes look flat
//
// **Point Light**:
// - Emits from a single point in all directions
// - Intensity falls off with distance (inverse square law)
// - Perfect for: Light bulbs, torches, candles
// - Real-world unit: Lumens (total light output)
//
// **Spot Light**:
// - Cone of light with defined angle
// - Inner angle: Full brightness cone
// - Outer angle: Falloff to darkness
// - Perfect for: Flashlights, stage lights, car headlights
//
// **Directional Light**:
// - Parallel rays from infinitely far away
// - No position, only direction matters
// - No falloff with distance
// - Perfect for: Sun, moon, distant lights
// - Real-world unit: Lux (illuminance)

// 💡 Physical Camera Exposure:
//
// The exposure triangle in photography:
// 1. **Aperture** (f-stop): Size of lens opening
//    - Controls depth of field (not simulated here)
//    - f/1.4 = wide open, f/22 = tiny opening
//
// 2. **Shutter Speed**: How long sensor is exposed
//    - Controls motion blur (not simulated here)
//    - 1/30s = slow, 1/1000s = fast
//
// 3. **ISO**: Sensor sensitivity
//    - Controls noise/grain (not simulated here)
//    - ISO 100 = low sensitivity, ISO 6400 = high
//
// In Bevy, these combine to control overall brightness:
// Exposure ∝ (ISO × ShutterSpeed) / (Aperture²)

// 🎨 Artistic Lighting Tips:
//
// **Three-Point Lighting** (Classic setup):
// - Key Light: Main light source (brightest)
// - Fill Light: Softens shadows (dimmer)
// - Rim Light: Separates subject from background
//
// **Color Temperature**:
// - Warm lights (orange/red): Cozy, sunset, fire
// - Cool lights (blue/white): Modern, moonlight, tech
// - Mix warm and cool for visual interest
//
// **Shadow Quality**:
// - Hard shadows: Small/distant light sources
// - Soft shadows: Large/close light sources
// - Use cascade settings for better shadow quality
