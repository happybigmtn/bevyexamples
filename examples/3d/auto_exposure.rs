//! This example showcases auto exposure,
//! which automatically (but not instantly) adjusts the brightness of the scene in a way that mimics the function of the human eye.
//! Auto exposure requires compute shader capabilities, so it's not available on WebGL.
//!
//! 👁️ Your Virtual Eye: Understanding Auto Exposure
//!
//! Have you ever walked from a dark room into bright sunlight? For a moment,
//! everything seems blindingly white, then your eyes adjust and you can see normally.
//! That's your eye's auto exposure system at work! This example demonstrates how
//! we can simulate this biological marvel in our games, creating more realistic
//! and comfortable viewing experiences.
//!
//! 🎨 What You'll See:
//! - A dimly lit colored box with an opening to a bright skybox
//! - Automatic brightness adjustment as you rotate the camera
//! - The "eye adaptation" effect when transitioning between dark and bright areas
//! - Optional metering masks and compensation curves for artistic control
//!
//! ## Controls
//!
//! | Key Binding        | Action                                 |
//! |:-------------------|:---------------------------------------|
//! | `Left` / `Right`   | Rotate Camera                          |
//! | `C`                | Toggle Compensation Curve              |
//! | `M`                | Toggle Metering Mask                   |
//! | `V`                | Visualize Metering Mask                |
//!
//! 🔑 Key Concepts:
//! - Luminance Metering: Measuring scene brightness
//! - Adaptation Speed: How quickly the "eye" adjusts
//! - Compensation Curves: Artistic control over exposure
//! - Metering Masks: Weighted brightness sampling

use bevy::{
    core_pipeline::{
        auto_exposure::{AutoExposure, AutoExposureCompensationCurve, AutoExposurePlugin},
        Skybox,
    },
    math::{cubic_splines::LinearSpline, primitives::Plane3d, vec2},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // 📸 Enable the auto exposure plugin
        .add_plugins(AutoExposurePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, example_control_system)
        .run();
}

// 🏗️ Scene Setup: Building a Light/Dark Test Environment
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut compensation_curves: ResMut<Assets<AutoExposureCompensationCurve>>,
    asset_server: Res<AssetServer>,
) {
    // 🎭 Load the metering mask - controls which parts of the screen
    // influence exposure calculations (center-weighted in this case)
    let metering_mask = asset_server.load("textures/basic_metering_mask.png");

    // 📷 Camera with Auto Exposure
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        // 👁️ THE STAR: Auto Exposure Component
        AutoExposure {
            // 🎯 Metering mask: weights different screen areas
            // Bright center = prioritize center for exposure
            metering_mask: metering_mask.clone(),
            ..default()
        },
        // 🌅 Bright skybox visible through the opening
        Skybox {
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            // ☀️ Very bright - simulating outdoor daylight
            brightness: light_consts::lux::DIRECT_SUNLIGHT,
            ..default()
        },
    ));

    // 📊 Create Resources for Runtime Control
    commands.insert_resource(ExampleResources {
        // 🎛️ Compensation curve: artistic exposure adjustment
        // This S-curve darkens shadows and brightens highlights
        basic_compensation_curve: compensation_curves.add(
            AutoExposureCompensationCurve::from_curve(LinearSpline::new([
                vec2(-4.0, -2.0),  // Deep shadows: darken more
                vec2(0.0, 0.0),    // Midtones: no change
                vec2(2.0, 0.0),    // Highlights: no change
                vec2(4.0, 2.0),    // Bright highlights: brighten more
            ]))
            .unwrap(),
        ),
        basic_metering_mask: metering_mask.clone(),
    });

    // 🏗️ Create a plane mesh for the box walls
    let plane = meshes.add(Mesh::from(
        Plane3d {
            normal: -Dir3::Z,
            half_size: Vec2::new(2.0, 0.5),
        }
        .mesh(),
    ));

    // 📦 Build the Box: A Dimly Lit Interior
    // We create a box with colored walls and an opening to see the bright sky
    for level in -1..=1 {  // Three levels: floor, middle, ceiling
        for side in [-Vec3::X, Vec3::X, -Vec3::Z, Vec3::Z] {  // Four sides
            // 🚪 Skip one wall section to create an opening
            if level == 0 && Vec3::Z == side {
                continue;
            }

            let height = Vec3::Y * level as f32;

            // 🎨 Spawn colored wall segments
            commands.spawn((
                Mesh3d(plane.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    // 🌈 Color walls based on position for visual variety
                    base_color: Color::srgb(
                        0.5 + side.x * 0.5,      // Red: varies with X
                        0.75 - level as f32 * 0.25,  // Green: varies with height
                        0.5 + side.z * 0.5,      // Blue: varies with Z
                    ),
                    ..default()
                })),
                Transform::from_translation(side * 2.0 + height).looking_at(height, Vec3::Y),
            ));
        }
    }

    // 🌑 Disable ambient light - we want darkness in the box
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.0,
        ..default()
    });

    // 💡 Dim point light inside the box
    commands.spawn((
        PointLight {
            intensity: 2000.0,  // Relatively dim compared to sunlight
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 🖼️ UI: Metering mask visualization (hidden by default)
    commands.spawn((
        ImageNode {
            image: metering_mask,
            ..default()
        },
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));

    let text_font = TextFont::default();

    // 📝 Instructions text
    commands.spawn((Text::new("Left / Right - Rotate Camera\nC - Toggle Compensation Curve\nM - Toggle Metering Mask\nV - Visualize Metering Mask"),
            text_font.clone(), Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        })
    );

    // 📊 Status display
    commands.spawn((
        Text::default(),
        text_font,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        ExampleDisplay,
    ));
}

// 🏷️ Marker component for the status display
#[derive(Component)]
struct ExampleDisplay;

// 📦 Resources for runtime configuration
#[derive(Resource)]
struct ExampleResources {
    basic_compensation_curve: Handle<AutoExposureCompensationCurve>,
    basic_metering_mask: Handle<Image>,
}

// 🎮 Interactive Controls System
fn example_control_system(
    camera: Single<(&mut Transform, &mut AutoExposure), With<Camera3d>>,
    mut display: Single<&mut Text, With<ExampleDisplay>>,
    mut mask_image: Single<&mut Node, With<ImageNode>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    resources: Res<ExampleResources>,
) {
    let (mut camera_transform, mut auto_exposure) = camera.into_inner();

    // 🔄 Camera rotation - transition between dark interior and bright exterior
    let rotation = if input.pressed(KeyCode::ArrowLeft) {
        time.delta_secs()
    } else if input.pressed(KeyCode::ArrowRight) {
        -time.delta_secs()
    } else {
        0.0
    };

    camera_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(rotation));

    // 📊 Toggle compensation curve
    // This affects how exposure values are mapped to final brightness
    if input.just_pressed(KeyCode::KeyC) {
        auto_exposure.compensation_curve =
            if auto_exposure.compensation_curve == resources.basic_compensation_curve {
                Handle::default()  // Linear (no compensation)
            } else {
                resources.basic_compensation_curve.clone()  // S-curve compensation
            };
    }

    // 🎯 Toggle metering mask
    // Switch between center-weighted and full-screen metering
    if input.just_pressed(KeyCode::KeyM) {
        auto_exposure.metering_mask =
            if auto_exposure.metering_mask == resources.basic_metering_mask {
                Handle::default()  // Full screen metering
            } else {
                resources.basic_metering_mask.clone()  // Center-weighted
            };
    }

    // 👁️ Show/hide metering mask visualization
    mask_image.display = if input.pressed(KeyCode::KeyV) {
        Display::Flex
    } else {
        Display::None
    };

    // 📝 Update status display
    display.0 = format!(
        "Compensation Curve: {}\nMetering Mask: {}",
        if auto_exposure.compensation_curve == resources.basic_compensation_curve {
            "Enabled"
        } else {
            "Disabled"
        },
        if auto_exposure.metering_mask == resources.basic_metering_mask {
            "Enabled"
        } else {
            "Disabled"
        },
    );
}

// 🎓 Deep Dive: How Auto Exposure Works
//
// Auto exposure simulates the human eye's iris and retinal adaptation:
//
// 1. **Luminance Calculation**:
//    - Convert each pixel to luminance (perceived brightness)
//    - Apply metering mask weights
//    - Calculate weighted average or histogram
//
// 2. **Temporal Adaptation**:
//    - Smoothly transition from current to target exposure
//    - Fast adaptation to brightness increases (pupil constriction)
//    - Slower adaptation to darkness (pupil dilation)
//
// 3. **Exposure Application**:
//    - Scale all pixel values by exposure multiplier
//    - Apply compensation curve for artistic control
//    - Clamp to displayable range
//
// The algorithm mimics biology:
// - Rods & Cones: Different adaptation speeds
// - Pupil Response: Quick constriction, slow dilation
// - Neural Adaptation: Brain adjusts perception over time

// 💡 Artistic Tips:
//
// 1. **Horror Games**: Slow dark adaptation, fast bright adaptation
// 2. **Realistic**: Medium speeds, center-weighted metering
// 3. **Arcade**: Fast adaptation, full-screen metering
// 4. **Cinematic**: Custom compensation curves for mood
//
// Common Techniques:
// - **Spot Metering**: Small central area (portraits)
// - **Center-Weighted**: Emphasize center (general use)
// - **Matrix/Evaluative**: Smart zones (landscapes)
// - **Histogram**: Prevent clipping (high contrast)

// 🎮 Gameplay Applications:
//
// - **Flashbang Effects**: Sudden overexposure
// - **Cave Emergence**: Gradual brightness adaptation
// - **Night Vision**: Different adaptation curves
// - **HDR Tone Mapping**: Compress wide brightness range
// - **Photographic Modes**: Simulate camera behavior