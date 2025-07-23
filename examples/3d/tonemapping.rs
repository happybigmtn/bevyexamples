//! This examples compares Tonemapping options
//!
//! 🎬 The Digital Cinematographer: Understanding Tonemapping
//!
//! Imagine you're watching a movie in a dark theater. The screen can only get
//! so bright, yet you see everything from deep shadows to brilliant sunlight.
//! How? That's the magic of tonemapping! It's like a skilled translator who takes
//! the vast range of light in the real world (or a 3D scene) and carefully
//! compresses it to fit your screen while preserving the mood and detail.
//! Different tonemapping algorithms are like different film stocks - each has
//! its own personality and artistic style!
//!
//! 🎯 What You'll See:
//! - Three test scenes to compare tonemapping methods:
//!   1. Basic Scene: 3D models with realistic lighting
//!   2. Color Sweep: Full spectrum gradient for color accuracy
//!   3. HDR Viewer: Load your own HDR images (drag & drop!)
//! - Eight different tonemapping algorithms to choose from
//! - Real-time color grading controls
//! - Side-by-side comparison capabilities
//!
//! 🎮 Controls:
//! - `1-8`: Select tonemapping method
//! - `Q/W/E`: Switch between test scenes
//! - `Arrow Keys`: Adjust color grading parameters
//! - `Space`: Reset color grading to defaults
//! - `Enter`: Apply scene-specific recommendations
//! - `H`: Hide/show UI
//!
//! 🔑 Key Concepts:
//! - HDR vs LDR: High vs Low Dynamic Range
//! - Tone Curves: Mathematical functions that map brightness
//! - Color Grading: Film-style color adjustments
//! - Exposure: Digital equivalent of camera settings
//! - Gamma: Mid-tone brightness adjustment

use bevy::{
    asset::UnapprovedPathMode,
    core_pipeline::tonemapping::Tonemapping,
    pbr::CascadeShadowConfigBuilder,
    platform::collections::HashMap,
    prelude::*,
    reflect::TypePath,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        view::{ColorGrading, ColorGradingGlobal, ColorGradingSection, Hdr},
    },
};
use std::f32::consts::PI;

/// 🎨 Shader for color gradient test pattern - helps visualize color accuracy
const SHADER_ASSET_PATH: &str = "shaders/tonemapping_test_patterns.wgsl";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                // 📁 Allow drag & drop of local HDR/EXR files for testing
                // This lets users test tonemapping with their own images!
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            }),
            // 🎨 Custom material for color gradient test pattern
            MaterialPlugin::<ColorGradientMaterial>::default(),
        ))
        // 📷 Default camera position for good scene view
        .insert_resource(CameraTransform(
            Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        ))
        // 🎛️ Initialize per-method color grading settings
        .init_resource::<PerMethodSettings>()
        // 🎬 Start with the basic 3D scene
        .insert_resource(CurrentScene(1))
        // 🔢 Parameter selection for arrow key controls
        .insert_resource(SelectedParameter { value: 0, max: 4 })
        .add_systems(
            Startup,
            (
                setup,                      // Core camera and UI
                setup_basic_scene,          // 3D models test scene
                setup_color_gradient_scene, // Color accuracy test
                setup_image_viewer_scene,   // HDR image viewer
            ),
        )
        .add_systems(
            Update,
            (
                drag_drop_image,              // Handle file drops
                resize_image,                 // Adjust viewer to image size
                toggle_scene,                 // Q/W/E scene switching
                toggle_tonemapping_method,    // 1-8 method selection
                update_color_grading_settings,// Arrow key adjustments
                update_ui,                    // Keep UI text current
            ),
        )
        .run();
}

// 🎬 Core Setup: Camera, Lighting, and UI Foundation
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_transform: Res<CameraTransform>,
) {
    // 📷 Main Camera with HDR Support
    commands.spawn((
        Camera3d::default(),
        // 🌟 HDR is ESSENTIAL for tonemapping!
        // Without HDR, we only have 0-1 range, nothing to tonemap
        Hdr,
        camera_transform.0,
        // 🌫️ Atmospheric fog for depth and mood
        DistanceFog {
            color: Color::srgb_u8(43, 44, 47),  // Dark gray fog
            falloff: FogFalloff::Linear {
                start: 1.0,   // Fog starts close
                end: 8.0,     // Fully foggy at 8 units
            },
            ..default()
        },
        // 🏛️ Beautiful Italian cathedral lighting
        // HDR environment maps provide realistic lighting that tests tonemapping
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2000.0,  // Bright HDR lighting
            ..default()
        },
    ));

    // 📝 UI Text for controls and current settings
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// 🎬 Scene 1: Realistic 3D Models for Testing
fn setup_basic_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 🏛️ Main test scene with various materials
    // This scene has metals, plastics, and different colors
    // Perfect for seeing how tonemapping affects different materials
    commands.spawn((
        SceneRoot(asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("models/TonemappingTest/TonemappingTest.gltf"),
        )),
        SceneNumber(1),
    ));

    // 🪖 Flight Helmet: Complex materials and details
    // Great for testing how tonemapping preserves fine details
    commands.spawn((
        SceneRoot(
            asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
        Transform::from_xyz(0.5, 0.0, -0.5)
            .with_rotation(Quat::from_rotation_y(-0.15 * PI)),  // Slight rotation for interest
        SceneNumber(1),
    ));

    // ☀️ Strong directional light to create HDR highlights
    // This creates bright spots that really test tonemapping
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.,  // Bright sunlight level
            shadows_enabled: true, // Shadows add contrast
            ..default()
        },
        // 📐 Angled for dramatic lighting
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI * -0.15, PI * -0.15)),
        // 🗺️ Cascade shadows for quality close and far
        CascadeShadowConfigBuilder {
            maximum_distance: 3.0,
            first_cascade_far_bound: 0.9,
            ..default()
        }
        .build(),
        SceneNumber(1),
    ));
}

// 🎨 Scene 2: Color Gradient Test Pattern
fn setup_color_gradient_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorGradientMaterial>>,
    camera_transform: Res<CameraTransform>,
) {
    // 📍 Position gradient directly in front of camera
    let mut transform = camera_transform.0;
    transform.translation += *transform.forward();

    // 🌈 Spawn the color gradient test pattern
    // This shows the full spectrum of colors and brightness
    // Essential for checking color accuracy and clipping
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(0.7, 0.7))),
        MeshMaterial3d(materials.add(ColorGradientMaterial {})),
        transform,
        Visibility::Hidden,  // Start hidden, show with 'W' key
        SceneNumber(2),
    ));
}

// 🖼️ Scene 3: HDR Image Viewer for Custom Testing
fn setup_image_viewer_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_transform: Res<CameraTransform>,
) {
    // 📍 Position viewer in front of camera
    let mut transform = camera_transform.0;
    transform.translation += *transform.forward();

    // 🖼️ HDR/EXR Image Display Plane
    // Perfect for testing tonemapping with real HDR photography
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: None,  // Will be set on file drop
            unlit: true,               // No lighting, show pure image
            ..default()
        })),
        transform,
        Visibility::Hidden,  // Start hidden, show with 'E' key
        SceneNumber(3),
        HDRViewer,          // Marker for drag & drop system
    ));

    // 📝 Instructions for drag & drop
    commands.spawn((
        Text::new("Drag and drop an HDR or EXR file"),
        TextFont {
            font_size: 36.0,
            ..default()
        },
        TextColor(Color::BLACK),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            align_self: AlignSelf::Center,
            margin: UiRect::all(Val::Auto),  // Center in viewport
            ..default()
        },
        SceneNumber(3),
        Visibility::Hidden,
    ));
}

// ----------------------------------------------------------------------------

fn drag_drop_image(
    image_mat: Query<&MeshMaterial3d<StandardMaterial>, With<HDRViewer>>,
    text: Query<Entity, (With<Text>, With<SceneNumber>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drop_events: EventReader<FileDragAndDrop>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(new_image) = drop_events.read().find_map(|e| match e {
        FileDragAndDrop::DroppedFile { path_buf, .. } => {
            Some(asset_server.load(path_buf.to_string_lossy().to_string()))
        }
        _ => None,
    }) else {
        return;
    };

    for mat_h in &image_mat {
        if let Some(mat) = materials.get_mut(mat_h) {
            mat.base_color_texture = Some(new_image.clone());

            // Despawn the image viewer instructions
            if let Ok(text_entity) = text.single() {
                commands.entity(text_entity).despawn();
            }
        }
    }
}

fn resize_image(
    image_mesh: Query<(&MeshMaterial3d<StandardMaterial>, &Mesh3d), With<HDRViewer>>,
    materials: Res<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    images: Res<Assets<Image>>,
    mut image_events: EventReader<AssetEvent<Image>>,
) {
    for event in image_events.read() {
        let (AssetEvent::Added { id } | AssetEvent::Modified { id }) = event else {
            continue;
        };

        for (mat_h, mesh_h) in &image_mesh {
            let Some(mat) = materials.get(mat_h) else {
                continue;
            };

            let Some(ref base_color_texture) = mat.base_color_texture else {
                continue;
            };

            if *id != base_color_texture.id() {
                continue;
            };

            let Some(image_changed) = images.get(*id) else {
                continue;
            };

            let size = image_changed.size_f32().normalize_or_zero() * 1.4;
            // Resize Mesh
            let quad = Mesh::from(Rectangle::from_size(size));
            meshes.insert(mesh_h, quad);
        }
    }
}

fn toggle_scene(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Visibility, &SceneNumber)>,
    mut current_scene: ResMut<CurrentScene>,
) {
    let mut pressed = None;
    if keys.just_pressed(KeyCode::KeyQ) {
        pressed = Some(1);
    } else if keys.just_pressed(KeyCode::KeyW) {
        pressed = Some(2);
    } else if keys.just_pressed(KeyCode::KeyE) {
        pressed = Some(3);
    }

    if let Some(pressed) = pressed {
        current_scene.0 = pressed;

        for (mut visibility, scene) in query.iter_mut() {
            if scene.0 == pressed {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }
}

// 🎛️ Tonemapping Method Selection: Choose Your Film Stock!
fn toggle_tonemapping_method(
    keys: Res<ButtonInput<KeyCode>>,
    mut tonemapping: Single<&mut Tonemapping>,
    mut color_grading: Single<&mut ColorGrading>,
    per_method_settings: Res<PerMethodSettings>,
) {
    // 🔢 Number keys select different tonemapping algorithms
    // Each has its own "look" and characteristics
    if keys.just_pressed(KeyCode::Digit1) {
        **tonemapping = Tonemapping::None;  // Raw HDR values (will clip!)
    } else if keys.just_pressed(KeyCode::Digit2) {
        **tonemapping = Tonemapping::Reinhard;  // Simple, pioneering algorithm
    } else if keys.just_pressed(KeyCode::Digit3) {
        **tonemapping = Tonemapping::ReinhardLuminance;  // Reinhard with luminance preservation
    } else if keys.just_pressed(KeyCode::Digit4) {
        **tonemapping = Tonemapping::AcesFitted;  // Film industry standard
    } else if keys.just_pressed(KeyCode::Digit5) {
        **tonemapping = Tonemapping::AgX;  // Blender's new standard
    } else if keys.just_pressed(KeyCode::Digit6) {
        **tonemapping = Tonemapping::SomewhatBoringDisplayTransform;  // Neutral, accurate
    } else if keys.just_pressed(KeyCode::Digit7) {
        **tonemapping = Tonemapping::TonyMcMapface;  // Modern, pleasing curve
    } else if keys.just_pressed(KeyCode::Digit8) {
        **tonemapping = Tonemapping::BlenderFilmic;  // Blender's previous standard
    }

    // 🎨 Apply the saved color grading settings for this method
    // Each method can have its own color adjustments
    **color_grading = (*per_method_settings
        .settings
        .get::<Tonemapping>(&tonemapping)
        .as_ref()
        .unwrap())
    .clone();
}

// 🎚️ UI Parameter Selection: Track Which Setting We're Adjusting
#[derive(Resource)]
struct SelectedParameter {
    value: i32,  // Current selected parameter (0-3)
    max: i32,    // Total number of parameters
}

impl SelectedParameter {
    fn next(&mut self) {
        // 🔽 Move to next parameter, wrap around
        self.value = (self.value + 1).rem_euclid(self.max);
    }
    fn prev(&mut self) {
        // 🔼 Move to previous parameter, wrap around
        self.value = (self.value - 1).rem_euclid(self.max);
    }
}

// 🎨 Color Grading Controls: Fine-Tune Your Look
fn update_color_grading_settings(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut per_method_settings: ResMut<PerMethodSettings>,
    tonemapping: Single<&Tonemapping>,
    current_scene: Res<CurrentScene>,
    mut selected_parameter: ResMut<SelectedParameter>,
) {
    let color_grading = per_method_settings.settings.get_mut(*tonemapping).unwrap();
    
    // 🕹️ Calculate adjustment speed
    let mut dt = time.delta_secs() * 0.25;
    if keys.pressed(KeyCode::ArrowLeft) {
        dt = -dt;  // Negative for decrease
    }

    // 🔼🔽 Navigate between parameters
    if keys.just_pressed(KeyCode::ArrowDown) {
        selected_parameter.next();
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        selected_parameter.prev();
    }
    
    // ⬅️➡️ Adjust selected parameter
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::ArrowRight) {
        match selected_parameter.value {
            0 => {
                // 📸 Exposure: Overall brightness (like camera exposure)
                color_grading.global.exposure += dt;
            }
            1 => {
                // 🌗 Gamma: Mid-tone brightness (affects contrast)
                color_grading
                    .all_sections_mut()
                    .for_each(|section| section.gamma += dt);
            }
            2 => {
                // 🎨 Pre-Saturation: Color intensity before tonemapping
                color_grading
                    .all_sections_mut()
                    .for_each(|section| section.saturation += dt);
            }
            3 => {
                // 🌈 Post-Saturation: Color intensity after tonemapping
                color_grading.global.post_saturation += dt;
            }
            _ => {}
        }
    }

    // 🔄 Space: Reset ALL methods to defaults
    if keys.just_pressed(KeyCode::Space) {
        for (_, grading) in per_method_settings.settings.iter_mut() {
            *grading = ColorGrading::default();
        }
    }

    // 🎬 Enter: Apply scene-specific recommendations (only for basic scene)
    if keys.just_pressed(KeyCode::Enter) && current_scene.0 == 1 {
        for (mapper, grading) in per_method_settings.settings.iter_mut() {
            *grading = PerMethodSettings::basic_scene_recommendation(*mapper);
        }
    }
}

fn update_ui(
    mut text_query: Single<&mut Text, Without<SceneNumber>>,
    settings: Single<(&Tonemapping, &ColorGrading)>,
    current_scene: Res<CurrentScene>,
    selected_parameter: Res<SelectedParameter>,
    mut hide_ui: Local<bool>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyH) {
        *hide_ui = !*hide_ui;
    }

    if *hide_ui {
        if !text_query.is_empty() {
            // single_mut() always triggers change detection,
            // so only access if text actually needs changing
            text_query.clear();
        }
        return;
    }

    let (tonemapping, color_grading) = *settings;
    let tonemapping = *tonemapping;

    let mut text = String::with_capacity(text_query.len());

    let scn = current_scene.0;
    text.push_str("(H) Hide UI\n\n");
    text.push_str("Test Scene: \n");
    text.push_str(&format!(
        "(Q) {} Basic Scene\n",
        if scn == 1 { ">" } else { "" }
    ));
    text.push_str(&format!(
        "(W) {} Color Sweep\n",
        if scn == 2 { ">" } else { "" }
    ));
    text.push_str(&format!(
        "(E) {} Image Viewer\n",
        if scn == 3 { ">" } else { "" }
    ));

    text.push_str("\n\nTonemapping Method:\n");
    text.push_str(&format!(
        "(1) {} Disabled\n",
        if tonemapping == Tonemapping::None {
            ">"
        } else {
            ""
        }
    ));
    text.push_str(&format!(
        "(2) {} Reinhard\n",
        if tonemapping == Tonemapping::Reinhard {
            "> "
        } else {
            ""
        }
    ));
    text.push_str(&format!(
        "(3) {} Reinhard Luminance\n",
        if tonemapping == Tonemapping::ReinhardLuminance {
            ">"
        } else {
            ""
        }
    ));
    text.push_str(&format!(
        "(4) {} ACES Fitted\n",
        if tonemapping == Tonemapping::AcesFitted {
            ">"
        } else {
            ""
        }
    ));
    text.push_str(&format!(
        "(5) {} AgX\n",
        if tonemapping == Tonemapping::AgX {
            ">"
        } else {
            ""
        }
    ));
    text.push_str(&format!(
        "(6) {} SomewhatBoringDisplayTransform\n",
        if tonemapping == Tonemapping::SomewhatBoringDisplayTransform {
            ">"
        } else {
            ""
        }
    ));
    text.push_str(&format!(
        "(7) {} TonyMcMapface\n",
        if tonemapping == Tonemapping::TonyMcMapface {
            ">"
        } else {
            ""
        }
    ));
    text.push_str(&format!(
        "(8) {} Blender Filmic\n",
        if tonemapping == Tonemapping::BlenderFilmic {
            ">"
        } else {
            ""
        }
    ));

    text.push_str("\n\nColor Grading:\n");
    text.push_str("(arrow keys)\n");
    if selected_parameter.value == 0 {
        text.push_str("> ");
    }
    text.push_str(&format!("Exposure: {}\n", color_grading.global.exposure));
    if selected_parameter.value == 1 {
        text.push_str("> ");
    }
    text.push_str(&format!("Gamma: {}\n", color_grading.shadows.gamma));
    if selected_parameter.value == 2 {
        text.push_str("> ");
    }
    text.push_str(&format!(
        "PreSaturation: {}\n",
        color_grading.shadows.saturation
    ));
    if selected_parameter.value == 3 {
        text.push_str("> ");
    }
    text.push_str(&format!(
        "PostSaturation: {}\n",
        color_grading.global.post_saturation
    ));
    text.push_str("(Space) Reset all to default\n");

    if current_scene.0 == 1 {
        text.push_str("(Enter) Reset all to scene recommendation\n");
    }

    if text != text_query.as_str() {
        // single_mut() always triggers change detection,
        // so only access if text actually changed
        text_query.0 = text;
    }
}

// ----------------------------------------------------------------------------

// 🎛️ Per-Method Settings: Each Tonemapper Gets Its Own Color Grade
#[derive(Resource)]
struct PerMethodSettings {
    settings: HashMap<Tonemapping, ColorGrading>,
}

impl PerMethodSettings {
    // 🎬 Scene-Specific Recommendations: Optimized for Our Test Scene
    fn basic_scene_recommendation(method: Tonemapping) -> ColorGrading {
        match method {
            // 📊 Reinhard tends to be a bit dark, needs exposure boost
            Tonemapping::Reinhard | Tonemapping::ReinhardLuminance => ColorGrading {
                global: ColorGradingGlobal {
                    exposure: 0.5,  // Brighten up the shadows
                    ..default()
                },
                ..default()
            },
            // 🎥 ACES is film-like but can be too contrasty
            Tonemapping::AcesFitted => ColorGrading {
                global: ColorGradingGlobal {
                    exposure: 0.35,  // Slight exposure boost
                    ..default()
                },
                ..default()
            },
            // 🎨 AgX needs saturation boost to avoid looking washed out
            Tonemapping::AgX => ColorGrading::with_identical_sections(
                ColorGradingGlobal {
                    exposure: -0.2,         // Slightly darker for better contrast
                    post_saturation: 1.1,   // Boost final color intensity
                    ..default()
                },
                ColorGradingSection {
                    saturation: 1.1,  // Pre-tonemap saturation boost
                    ..default()
                },
            ),
            // 🔧 Others work well with defaults
            _ => ColorGrading::default(),
        }
    }
}

impl Default for PerMethodSettings {
    fn default() -> Self {
        let mut settings = <HashMap<_, _>>::default();

        // 🎬 Initialize each tonemapping method with appropriate settings
        for method in [
            Tonemapping::None,
            Tonemapping::Reinhard,
            Tonemapping::ReinhardLuminance,
            Tonemapping::AcesFitted,
            Tonemapping::AgX,
            Tonemapping::SomewhatBoringDisplayTransform,
            Tonemapping::TonyMcMapface,
            Tonemapping::BlenderFilmic,
        ] {
            settings.insert(
                method,
                PerMethodSettings::basic_scene_recommendation(method),
            );
        }

        Self { settings }
    }
}

impl Material for ColorGradientMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

// 🎨 Custom material for the color gradient test pattern
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ColorGradientMaterial {}

// 📷 Shared camera transform for all scenes
#[derive(Resource)]
struct CameraTransform(Transform);

// 🎬 Currently displayed scene (1-3)
#[derive(Resource)]
struct CurrentScene(u32);

// 🏷️ Component to mark which scene entities belong to
#[derive(Component)]
struct SceneNumber(u32);

// 🖼️ Marker for the HDR image viewer entity
#[derive(Component)]
struct HDRViewer;

// 🎓 Deep Dive: The Art and Science of Tonemapping
//
// **What is Tonemapping?**
// Tonemapping is the process of converting High Dynamic Range (HDR) values
// to Low Dynamic Range (LDR) for display. Real world has brightness ratios
// of 1,000,000:1 or more, but screens can only show about 1,000:1.
//
// **Why Different Algorithms?**
// Just like film stocks in photography, each algorithm has its own "look":
//
// 1. **None**: No tonemapping - values above 1.0 are clipped to white
//    - Use case: When you've already tonemapped in your art pipeline
//
// 2. **Reinhard**: The pioneering algorithm (2002)
//    - Formula: color = color / (1 + color)
//    - Pros: Simple, prevents clipping
//    - Cons: Can look washed out, loses color in bright areas
//
// 3. **Reinhard Luminance**: Reinhard applied to luminance only
//    - Preserves color ratios better than basic Reinhard
//    - Good for maintaining color in bright areas
//
// 4. **ACES Fitted**: Academy Color Encoding System
//    - Film industry standard developed by the Academy
//    - Pros: Cinematic look, good contrast
//    - Cons: Can crush blacks, adds slight color shift
//
// 5. **AgX**: Blender's new default (2023)
//    - Designed for wide gamut displays
//    - Better color preservation in extremes
//    - More neutral than filmic options
//
// 6. **SomewhatBoringDisplayTransform**: Troy Sobotka's neutral curve
//    - Designed for accuracy over aesthetics
//    - Great for technical visualization
//    - Minimal color shifts
//
// 7. **TonyMcMapface**: Modern algorithm by Tomasz Stachowiak
//    - Good balance of contrast and color preservation
//    - Popular in games for its pleasant look
//    - Handles extreme values well
//
// 8. **Blender Filmic**: Blender's previous default
//    - Film-like response curve
//    - Good for photorealistic rendering
//    - Can desaturate bright areas

// 💡 Color Grading Explained:
//
// **Exposure**: Overall brightness adjustment
// - Measured in stops (doubling/halving light)
// - +1 exposure = 2x brighter, -1 = half as bright
//
// **Gamma**: Mid-tone brightness control
// - Values > 1 brighten mid-tones
// - Values < 1 darken mid-tones
// - Doesn't affect pure black/white
//
// **Saturation**: Color intensity
// - Pre-saturation: Before tonemapping (can affect how colors compress)
// - Post-saturation: After tonemapping (final color boost)
// - 0 = grayscale, 1 = normal, >1 = oversaturated
//
// **The Three-Way Color Corrector**:
// Professional colorists adjust shadows, midtones, and highlights separately.
// This gives precise control over the image mood and contrast.

// 🎬 Practical Usage Tips:
//
// **For Photorealism**:
// - Use ACES or TonyMcMapface
// - Keep exposure near 0
// - Minimal saturation adjustments
//
// **For Stylized Games**:
// - AgX or Blender Filmic work well
// - Boost saturation for vibrant colors
// - Adjust gamma for mood (higher = brighter/happier)
//
// **For Technical Visualization**:
// - SomewhatBoringDisplayTransform
// - No color grading adjustments
// - Focus on accuracy over aesthetics
//
// **For Mobile/Performance**:
// - Reinhard is fastest
// - Consider baking tonemapping into textures
// - Use "None" if pre-tonemapped

// 🔬 The Mathematics Behind It:
//
// **HDR to LDR Mapping**:
// The challenge is mapping infinite range to [0,1] while:
// - Preserving relative brightness relationships
// - Maintaining color ratios
// - Avoiding harsh clipping
// - Creating pleasing contrast
//
// **Filmic Curves**:
// Most modern tonemappers use S-curves inspired by film:
// - Toe: Gentle rolloff in shadows
// - Linear section: Faithful mid-tone reproduction  
// - Shoulder: Smooth highlight compression
//
// **Luminance vs RGB**:
// Some algorithms work on luminance (brightness) only,
// others process RGB channels independently. Luminance
// preserves color relationships better but can look less saturated.
