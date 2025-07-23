//! Demonstrates volumetric fog and lighting (light shafts or god rays).
//!
//! 🌫️ Divine Light Through the Mist: Understanding Volumetric Fog
//!
//! Have you ever watched sunbeams streaming through a dusty attic window?
//! Or seen rays of light piercing through forest canopy on a misty morning?
//! That's volumetric lighting! Unlike simple fog that just fades objects,
//! volumetric fog interacts with light itself. Light scatters through tiny
//! particles in the air, creating those magical "god rays" that make scenes
//! feel atmospheric and alive. It's like making the air itself visible!
//!
//! 🎯 What You'll See:
//! - A mysterious stone chamber filled with swirling fog
//! - Dramatic light shafts from multiple light sources:
//!   - White directional light (like sunlight through windows)
//!   - Red point light (moving back and forth)
//!   - White spot light (illuminating from above)
//! - Real-time control over fog density and light behavior
//! - Dynamic shadows creating realistic light occlusion
//!
//! 🎮 Controls:
//! - `WASD/Arrow Keys`: Rotate the directional light
//! - `P`: Toggle volumetric point light on/off
//! - `L`: Toggle volumetric spot light on/off
//!
//! 🔑 Key Concepts:
//! - Volumetric Fog: 3D fog that fills space, not just a screen effect
//! - Light Scattering: How light bounces off particles in the air
//! - God Rays: Visible light shafts through participating media
//! - Shadow Volumes: Where light can't reach, creating realistic occlusion
//! - Fog Density: How thick the fog is (affects visibility and scattering)

use bevy::{
    color::palettes::css::RED,
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping, Skybox},
    math::vec3,
    pbr::{FogVolume, VolumetricFog, VolumetricLight},
    prelude::*,
};

// 🎮 How fast the directional light rotates with keyboard input
const DIRECTIONAL_LIGHT_MOVEMENT_SPEED: f32 = 0.02;

/// 🎛️ User Settings: Control Which Lights Create Volumetric Effects
#[derive(Resource)]
struct AppSettings {
    /// 💡 Whether the spot light creates light shafts
    volumetric_spotlight: bool,
    /// 🔴 Whether the red point light creates volumetric effects
    volumetric_pointlight: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            volumetric_spotlight: true,
            volumetric_pointlight: true,
        }
    }
}

// 🏃 Movement Component: Makes the Red Light Patrol Back and Forth
// This creates dynamic lighting effects as the light moves through fog
#[derive(Component)]
struct MoveBackAndForthHorizontally {
    min_x: f32,   // Left boundary
    max_x: f32,   // Right boundary  
    speed: f32,   // Current velocity (changes sign at boundaries)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // 🌑 Very dark background to make fog effects visible
        .insert_resource(ClearColor(Color::Srgba(Srgba {
            red: 0.02,
            green: 0.02,
            blue: 0.02,
            alpha: 1.0,
        })))
        // 🌑 No ambient light - all illumination comes from our fog lights
        .insert_resource(AmbientLight::NONE)
        // 🎛️ Initialize settings with volumetric effects on
        .init_resource::<AppSettings>()
        .add_systems(Startup, setup)
        // 🔧 Automatically enable volumetric on loaded directional lights
        .add_systems(Update, tweak_scene)
        // 🎮 Handle light movement and controls
        .add_systems(Update, (move_directional_light, move_point_light))
        // 🎛️ Toggle volumetric effects per user input
        .add_systems(Update, adjust_app_settings)
        .run();
}

/// 🏗️ Scene Setup: Creating Our Atmospheric Chamber
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, app_settings: Res<AppSettings>) {
    // 🏛️ Load the stone chamber scene
    // This scene has windows and openings perfect for light shafts
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/VolumetricFogExample/VolumetricFogExample.glb"),
    )));

    // 📸 Camera Setup: Positioned for Best View of Volumetric Effects
    commands
        .spawn((
            Camera3d::default(),
            // 🎯 Looking into the chamber where fog effects are most visible
            Transform::from_xyz(-1.7, 1.5, 4.5).looking_at(vec3(-1.5, 1.7, 3.5), Vec3::Y),
            // 🎨 TonyMcMapface tonemapping for nice contrast
            Tonemapping::TonyMcMapface,
            // ✨ Bloom makes light shafts glow beautifully
            Bloom::default(),
        ))
        // 🌌 Skybox for reflections (though mostly hidden by fog)
        .insert(Skybox {
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            brightness: 1000.0,
            ..default()
        })
        // 🌫️ Enable volumetric fog rendering on this camera
        .insert(VolumetricFog {
            // No ambient fog light since we want dramatic light shafts
            ambient_intensity: 0.0,
            ..default()
        });

    // 🔴 Red Point Light: Moving Through the Fog
    // This creates dynamic volumetric effects as it patrols
    commands.spawn((
        Transform::from_xyz(-0.4, 1.9, 1.0),
        PointLight {
            shadows_enabled: true,  // REQUIRED for volumetric effects!
            range: 150.0,          // How far the light reaches
            color: RED.into(),     // Dramatic red color
            intensity: 1000.0,     // Bright enough to pierce fog
            ..default()
        },
        VolumetricLight,  // ✨ This makes the light volumetric!
        // 🏃 Movement parameters for back-and-forth motion
        MoveBackAndForthHorizontally {
            min_x: -1.93,  // Left boundary
            max_x: -0.4,   // Right boundary
            speed: -0.2,   // Start moving left
        },
    ));

    // 💡 White Spot Light: Dramatic Downward Beam
    // Creates a cone of light through the fog
    commands.spawn((
        Transform::from_xyz(-1.8, 3.9, -2.7).looking_at(Vec3::ZERO, Vec3::Y),
        SpotLight {
            intensity: 5000.0,      // Very bright for strong light shafts
            color: Color::WHITE,
            shadows_enabled: true,  // REQUIRED for volumetric effects!
            inner_angle: 0.76,      // Full brightness cone
            outer_angle: 0.94,      // Falloff cone edge
            ..default()
        },
        VolumetricLight,  // ✨ Enable volumetric rendering
    ));

    // 🌫️ The Fog Volume: Filling the Space with Mist
    commands.spawn((
        FogVolume::default(),
        // 📏 Scale to 35x35x35 units - covers the entire scene
        Transform::from_scale(Vec3::splat(35.0)),
    ));

    // 📝 Help Text UI
    commands.spawn((
        create_text(&app_settings),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn create_text(app_settings: &AppSettings) -> Text {
    format!(
        "{}\n{}\n{}",
        "Press WASD or the arrow keys to change the direction of the directional light",
        if app_settings.volumetric_pointlight {
            "Press P to turn volumetric point light off"
        } else {
            "Press P to turn volumetric point light on"
        },
        if app_settings.volumetric_spotlight {
            "Press L to turn volumetric spot light off"
        } else {
            "Press L to turn volumetric spot light on"
        }
    )
    .into()
}

/// 🔧 Scene Tweaker: Automatically Enable Volumetric on Loaded Lights
/// The glTF scene contains a directional light that needs volumetric setup
fn tweak_scene(
    mut commands: Commands,
    mut lights: Query<(Entity, &mut DirectionalLight), Changed<DirectionalLight>>,
) {
    for (light, mut directional_light) in lights.iter_mut() {
        // ⚠️ Shadows MUST be enabled for volumetric effects!
        // Without shadows, there's no occlusion data for light shafts
        directional_light.shadows_enabled = true;
        
        // ✨ Add volumetric component to create god rays
        commands.entity(light).insert(VolumetricLight);
    }
}

/// 🎮 Directional Light Control: Rotate the Sun!
/// This lets you see how light shaft direction changes
fn move_directional_light(
    input: Res<ButtonInput<KeyCode>>,
    mut directional_lights: Query<&mut Transform, With<DirectionalLight>>,
) {
    let mut delta_theta = Vec2::ZERO;
    
    // ⬆️ W/Up: Tilt light upward
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        delta_theta.y += DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }
    // ⬇️ S/Down: Tilt light downward
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        delta_theta.y -= DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }
    // ⬅️ A/Left: Rotate light left
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        delta_theta.x += DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }
    // ➡️ D/Right: Rotate light right
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        delta_theta.x -= DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }

    if delta_theta == Vec2::ZERO {
        return;
    }

    // 🔄 Apply rotation to all directional lights
    let delta_quat = Quat::from_euler(EulerRot::XZY, delta_theta.y, 0.0, delta_theta.x);
    for mut transform in directional_lights.iter_mut() {
        transform.rotate(delta_quat);
    }
}

// 🏃 Point Light Movement: Patrol Animation
// The moving light creates dynamic volumetric effects
fn move_point_light(
    timer: Res<Time>,
    mut objects: Query<(&mut Transform, &mut MoveBackAndForthHorizontally)>,
) {
    for (mut transform, mut move_data) in objects.iter_mut() {
        let mut translation = transform.translation;
        let mut need_toggle = false;
        
        // 🚶 Move the light horizontally
        translation.x += move_data.speed * timer.delta_secs();
        
        // 🚧 Check boundaries and reverse if needed
        if translation.x > move_data.max_x {
            translation.x = move_data.max_x;
            need_toggle = true;  // Hit right wall
        } else if translation.x < move_data.min_x {
            translation.x = move_data.min_x;
            need_toggle = true;  // Hit left wall
        }
        
        // 🔄 Reverse direction at boundaries
        if need_toggle {
            move_data.speed = -move_data.speed;
        }
        
        transform.translation = translation;
    }
}

// 🎛️ Settings Control: Toggle Volumetric Effects Per Light
fn adjust_app_settings(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_settings: ResMut<AppSettings>,
    mut point_lights: Query<Entity, With<PointLight>>,
    mut spot_lights: Query<Entity, With<SpotLight>>,
    mut text: Query<&mut Text>,
) {
    // 🏃 Performance optimization: only process if something changed
    let mut any_changes = false;

    // 🔴 P key: Toggle volumetric red point light
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        app_settings.volumetric_pointlight = !app_settings.volumetric_pointlight;
        any_changes = true;
    }
    
    // 💡 L key: Toggle volumetric spot light
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        app_settings.volumetric_spotlight = !app_settings.volumetric_spotlight;
        any_changes = true;
    }

    // ⚡ Early exit if nothing changed
    if !any_changes {
        return;
    }

    // 🔧 Update point light volumetric components
    for point_light in point_lights.iter_mut() {
        if app_settings.volumetric_pointlight {
            // ✨ Add volumetric component for light shafts
            commands.entity(point_light).insert(VolumetricLight);
        } else {
            // 🚫 Remove volumetric component (light still works, just no fog interaction)
            commands.entity(point_light).remove::<VolumetricLight>();
        }
    }
    
    // 🔧 Update spot light volumetric components
    for spot_light in spot_lights.iter_mut() {
        if app_settings.volumetric_spotlight {
            commands.entity(spot_light).insert(VolumetricLight);
        } else {
            commands.entity(spot_light).remove::<VolumetricLight>();
        }
    }

    // 📝 Update UI text to reflect new state
    for mut text in text.iter_mut() {
        *text = create_text(&app_settings);
    }
}

// 🎓 Deep Dive: The Physics of Volumetric Fog
//
// **What Makes Fog Volumetric?**
// Traditional fog just fades objects based on distance. Volumetric fog
// actually simulates light traveling through a participating medium (like
// real fog, smoke, or dust). Light scatters off particles, creating visible
// rays and realistic depth.
//
// **The Science of Light Scattering:**
// When light hits a particle in fog:
// 1. **Absorption**: Some light is absorbed (fog color affects this)
// 2. **Out-scattering**: Light bounces away from the viewer
// 3. **In-scattering**: Light from other directions bounces toward viewer
// 4. **Forward scattering**: Most light continues forward (Mie scattering)
//
// **How Volumetric Rendering Works:**
// 1. **Ray Marching**: For each pixel, step through the fog volume
// 2. **Shadow Sampling**: Check if each point is in shadow
// 3. **Scattering Calculation**: Compute light contribution at each step
// 4. **Integration**: Sum up all contributions along the ray
//
// **Performance Considerations:**
// Volumetric fog is expensive because it requires many samples per pixel:
// - More samples = better quality but slower
// - Lower resolution = faster but more artifacts
// - Temporal upsampling helps (reuse previous frames)

// 💡 Types of Volumetric Effects:
//
// **God Rays (Crepuscular Rays):**
// - Directional light through openings
// - Most dramatic with strong contrast
// - Named after their divine appearance
//
// **Volumetric Spotlights:**
// - Cone-shaped beams through fog
// - Great for dramatic scenes
// - Stage lighting effects
//
// **Point Light Volumes:**
// - Spherical light falloff
// - Good for torches, candles
// - Creates atmospheric pools of light
//
// **Fog Density Variations:**
// - Real fog isn't uniform
// - Height-based fog (ground fog)
// - Noise-based variations

// 🎨 Artistic Tips:
//
// **Creating Mood:**
// - Dense fog = mystery, horror
// - Light fog = dreamy, ethereal
// - Colored fog = alien, underwater
//
// **Light Placement:**
// - Backlight creates strongest rays
// - Side light adds depth
// - Multiple lights create complexity
//
// **Shadow Importance:**
// - Shadows create occlusion
// - Without shadows, no god rays
// - Shadow quality affects ray quality
//
// **Color Choices:**
// - Warm fog = sunset, cozy
// - Cool fog = morning, eerie
// - Match fog color to scene mood

// 🔧 Technical Parameters:
//
// **Fog Density:**
// - Controls how quickly light attenuates
// - Higher = thicker fog, shorter view distance
// - Affects performance (more scattering)
//
// **Scattering Coefficient:**
// - How much light bounces vs absorbs
// - Higher = brighter fog, more glow
// - Affects light shaft visibility
//
// **Anisotropy (g-factor):**
// - Direction bias of scattering
// - g > 0: Forward scatter (realistic)
// - g < 0: Backward scatter (stylized)
// - g = 0: Isotropic (equal all directions)
//
// **Sample Count:**
// - Steps along each ray
// - More samples = smoother gradients
// - Major performance factor

// ⚡ Optimization Strategies:
//
// **Downsampling:**
// - Render volumetrics at 1/2 or 1/4 resolution
// - Bilateral upsampling preserves edges
//
// **Temporal Integration:**
// - Accumulate samples over frames
// - Reduces noise, improves quality
//
// **Early Exit:**
// - Stop marching when accumulated opacity = 1
// - Skip fully shadowed regions
//
// **LOD System:**
// - Fewer samples for distant fog
// - Simpler scattering for background
