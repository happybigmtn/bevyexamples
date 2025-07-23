//! This example compares MSAA (Multi-Sample Anti-aliasing), FXAA (Fast Approximate Anti-aliasing), and TAA (Temporal Anti-aliasing).
//!
//! 🎮 The Quest for Smooth Edges: Understanding Anti-Aliasing
//!
//! Imagine you're drawing a diagonal line on graph paper - you can't make it perfectly
//! smooth, only approximate it with stair-steps. That's aliasing! In 3D graphics,
//! this creates jagged edges (aka "jaggies") that break immersion. Anti-aliasing
//! techniques smooth these edges, each with unique trade-offs between quality and performance.
//!
//! 🎨 What You'll See:
//! - A scene with various geometric edges and textures
//! - Press 1-5 to switch between anti-aliasing methods
//! - Press Q/W/E/R/T to adjust quality settings for each method
//! - Press 0 to toggle contrast adaptive sharpening
//! - Watch how each technique handles different edge types!
//!
//! 🔑 Key Concepts:
//! - MSAA: Renders at higher resolution, then downsamples (hardware-based)
//! - FXAA: Post-process edge detection and smoothing (fast but can blur)
//! - SMAA: Enhanced edge detection with pattern recognition
//! - TAA: Uses motion vectors to accumulate samples over time (great quality, can ghost)

use std::{f32::consts::PI, fmt::Write};

use bevy::{
    anti_aliasing::{
        contrast_adaptive_sharpening::ContrastAdaptiveSharpening,
        fxaa::{Fxaa, Sensitivity},
        smaa::{Smaa, SmaaPreset},
        taa::TemporalAntiAliasing,
    },
    core_pipeline::prepass::{DepthPrepass, MotionVectorPrepass},
    image::{ImageSampler, ImageSamplerDescriptor},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{
        camera::{MipBias, TemporalJitter},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        view::Hdr,
    },
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (modify_aa, modify_sharpening, update_ui))
        .run();
}

// 🎯 TAA Component Bundle
// TAA needs several components working together for temporal accumulation
type TaaComponents = (
    TemporalAntiAliasing,
    TemporalJitter,      // Jitters camera slightly each frame
    MipBias,             // Adjusts texture sampling for stability
    DepthPrepass,        // Provides depth info for reprojection
    MotionVectorPrepass, // Tracks pixel movement between frames
);

// 🎮 Anti-Aliasing Control System
//
// This system handles user input to switch between different AA methods
// and adjust their quality settings. Each method has unique strengths!
fn modify_aa(
    keys: Res<ButtonInput<KeyCode>>,
    camera: Single<
        (
            Entity,
            Option<&mut Fxaa>,
            Option<&mut Smaa>,
            Option<&TemporalAntiAliasing>,
            &mut Msaa,
        ),
        With<Camera>,
    >,
    mut commands: Commands,
) {
    let (camera_entity, fxaa, smaa, taa, mut msaa) = camera.into_inner();
    let mut camera = commands.entity(camera_entity);

    // 🚫 No AA - See the raw, jaggy truth!
    if keys.just_pressed(KeyCode::Digit1) {
        *msaa = Msaa::Off;
        camera
            .remove::<Fxaa>()
            .remove::<Smaa>()
            .remove::<TaaComponents>();
    }

    // 🔲 MSAA (Multi-Sample Anti-Aliasing)
    // The classic hardware solution - renders multiple samples per pixel
    if keys.just_pressed(KeyCode::Digit2) && *msaa == Msaa::Off {
        camera
            .remove::<Fxaa>()
            .remove::<Smaa>()
            .remove::<TaaComponents>();

        *msaa = Msaa::Sample4;
    }

    // 📊 MSAA Sample Count Options
    // More samples = smoother edges but higher GPU cost
    if *msaa != Msaa::Off {
        if keys.just_pressed(KeyCode::KeyQ) {
            *msaa = Msaa::Sample2;  // Light smoothing
        }
        if keys.just_pressed(KeyCode::KeyW) {
            *msaa = Msaa::Sample4;  // Balanced quality
        }
        if keys.just_pressed(KeyCode::KeyE) {
            *msaa = Msaa::Sample8;  // Premium smoothness
        }
    }

    // 🏃 FXAA (Fast Approximate Anti-Aliasing)
    // Post-process edge detection - very fast but can blur textures
    if keys.just_pressed(KeyCode::Digit3) && fxaa.is_none() {
        *msaa = Msaa::Off;
        camera
            .remove::<Smaa>()
            .remove::<TaaComponents>()
            .insert(Fxaa::default());
    }

    // 🎚️ FXAA Sensitivity Settings
    // Controls how aggressively it detects and smooths edges
    if let Some(mut fxaa) = fxaa {
        if keys.just_pressed(KeyCode::KeyQ) {
            // Low: Only obvious edges
            fxaa.edge_threshold = Sensitivity::Low;
            fxaa.edge_threshold_min = Sensitivity::Low;
        }
        if keys.just_pressed(KeyCode::KeyW) {
            // Medium: Balanced detection
            fxaa.edge_threshold = Sensitivity::Medium;
            fxaa.edge_threshold_min = Sensitivity::Medium;
        }
        if keys.just_pressed(KeyCode::KeyE) {
            // High: More aggressive smoothing
            fxaa.edge_threshold = Sensitivity::High;
            fxaa.edge_threshold_min = Sensitivity::High;
        }
        if keys.just_pressed(KeyCode::KeyR) {
            // Ultra: Smooth most edges
            fxaa.edge_threshold = Sensitivity::Ultra;
            fxaa.edge_threshold_min = Sensitivity::Ultra;
        }
        if keys.just_pressed(KeyCode::KeyT) {
            // Extreme: Maximum smoothing (may over-blur)
            fxaa.edge_threshold = Sensitivity::Extreme;
            fxaa.edge_threshold_min = Sensitivity::Extreme;
        }
    }

    // 🧠 SMAA (Enhanced Subpixel Morphological Anti-Aliasing)
    // Smarter edge detection with pattern recognition
    if keys.just_pressed(KeyCode::Digit4) && smaa.is_none() {
        *msaa = Msaa::Off;
        camera
            .remove::<Fxaa>()
            .remove::<TaaComponents>()
            .insert(Smaa::default());
    }

    // 🎯 SMAA Quality Presets
    if let Some(mut smaa) = smaa {
        if keys.just_pressed(KeyCode::KeyQ) {
            smaa.preset = SmaaPreset::Low;     // Fast, basic edge detection
        }
        if keys.just_pressed(KeyCode::KeyW) {
            smaa.preset = SmaaPreset::Medium;  // Good balance
        }
        if keys.just_pressed(KeyCode::KeyE) {
            smaa.preset = SmaaPreset::High;    // Quality focus
        }
        if keys.just_pressed(KeyCode::KeyR) {
            smaa.preset = SmaaPreset::Ultra;   // Maximum quality
        }
    }

    // ⏰ TAA (Temporal Anti-Aliasing)
    // Uses previous frames to accumulate samples - excellent quality!
    if keys.just_pressed(KeyCode::Digit5) && taa.is_none() {
        *msaa = Msaa::Off;
        camera
            .remove::<Fxaa>()
            .remove::<Smaa>()
            .insert(TemporalAntiAliasing::default());
    }
}

// 🔪 Sharpening Control System
//
// Anti-aliasing can soften the image. Contrast Adaptive Sharpening (CAS)
// intelligently restores detail without creating artifacts.
fn modify_sharpening(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut ContrastAdaptiveSharpening>,
) {
    for mut cas in &mut query {
        // Toggle sharpening on/off
        if keys.just_pressed(KeyCode::Digit0) {
            cas.enabled = !cas.enabled;
        }
        
        if cas.enabled {
            // Decrease sharpening strength
            if keys.just_pressed(KeyCode::Minus) {
                cas.sharpening_strength -= 0.1;
                cas.sharpening_strength = cas.sharpening_strength.clamp(0.0, 1.0);
            }
            // Increase sharpening strength
            if keys.just_pressed(KeyCode::Equal) {
                cas.sharpening_strength += 0.1;
                cas.sharpening_strength = cas.sharpening_strength.clamp(0.0, 1.0);
            }
            // Toggle denoising (reduces grain in dark areas)
            if keys.just_pressed(KeyCode::KeyD) {
                cas.denoise = !cas.denoise;
            }
        }
    }
}

// 📊 UI Update System
//
// Shows the current anti-aliasing method and settings
fn update_ui(
    camera: Single<
        (
            Option<&Fxaa>,
            Option<&Smaa>,
            Option<&TemporalAntiAliasing>,
            &ContrastAdaptiveSharpening,
            &Msaa,
        ),
        With<Camera>,
    >,
    mut ui: Single<&mut Text>,
) {
    let (fxaa, smaa, taa, cas, msaa) = *camera;

    let ui = &mut ui.0;
    *ui = "Antialias Method\n".to_string();

    // Show which AA method is active
    draw_selectable_menu_item(
        ui,
        "No AA",
        '1',
        *msaa == Msaa::Off && fxaa.is_none() && taa.is_none() && smaa.is_none(),
    );
    draw_selectable_menu_item(ui, "MSAA", '2', *msaa != Msaa::Off);
    draw_selectable_menu_item(ui, "FXAA", '3', fxaa.is_some());
    draw_selectable_menu_item(ui, "SMAA", '4', smaa.is_some());
    draw_selectable_menu_item(ui, "TAA", '5', taa.is_some());

    // Show MSAA sample options
    if *msaa != Msaa::Off {
        ui.push_str("\n----------\n\nSample Count\n");
        draw_selectable_menu_item(ui, "2", 'Q', *msaa == Msaa::Sample2);
        draw_selectable_menu_item(ui, "4", 'W', *msaa == Msaa::Sample4);
        draw_selectable_menu_item(ui, "8", 'E', *msaa == Msaa::Sample8);
    }

    // Show FXAA sensitivity options
    if let Some(fxaa) = fxaa {
        ui.push_str("\n----------\n\nSensitivity\n");
        draw_selectable_menu_item(ui, "Low", 'Q', fxaa.edge_threshold == Sensitivity::Low);
        draw_selectable_menu_item(
            ui,
            "Medium",
            'W',
            fxaa.edge_threshold == Sensitivity::Medium,
        );
        draw_selectable_menu_item(ui, "High", 'E', fxaa.edge_threshold == Sensitivity::High);
        draw_selectable_menu_item(ui, "Ultra", 'R', fxaa.edge_threshold == Sensitivity::Ultra);
        draw_selectable_menu_item(
            ui,
            "Extreme",
            'T',
            fxaa.edge_threshold == Sensitivity::Extreme,
        );
    }

    // Show SMAA quality options
    if let Some(smaa) = smaa {
        ui.push_str("\n----------\n\nQuality\n");
        draw_selectable_menu_item(ui, "Low", 'Q', smaa.preset == SmaaPreset::Low);
        draw_selectable_menu_item(ui, "Medium", 'W', smaa.preset == SmaaPreset::Medium);
        draw_selectable_menu_item(ui, "High", 'E', smaa.preset == SmaaPreset::High);
        draw_selectable_menu_item(ui, "Ultra", 'R', smaa.preset == SmaaPreset::Ultra);
    }

    // Show sharpening options
    ui.push_str("\n----------\n\n");
    draw_selectable_menu_item(ui, "Sharpening", '0', cas.enabled);

    if cas.enabled {
        ui.push_str(&format!("(-/+) Strength: {:.1}\n", cas.sharpening_strength));
        draw_selectable_menu_item(ui, "Denoising", 'D', cas.denoise);
    }
}

// 🏗️ Scene Setup: A Perfect Anti-Aliasing Test Lab
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    // 🌍 Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.1))),
    ));

    // 🎨 Create a material with our debug texture
    let cube_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    // 📦 Small cubes with high-contrast textures
    // These show aliasing artifacts clearly
    for i in 0..5 {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.25, 0.25, 0.25))),
            MeshMaterial3d(cube_material.clone()),
            Transform::from_xyz(i as f32 * 0.25 - 1.0, 0.125, -i as f32 * 0.5),
        ));
    }

    // 🚁 Flight Helmet - Complex geometry with fine details
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf"),
    )));

    // ☀️ Directional light with shadows
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI * -0.15, PI * -0.15)),
        // Shadow cascades for better shadow quality
        CascadeShadowConfigBuilder {
            maximum_distance: 3.0,
            first_cascade_far_bound: 0.9,
            ..default()
        }
        .build(),
    ));

    // 📷 Camera setup
    commands.spawn((
        Camera3d::default(),
        Hdr,  // High Dynamic Range for better lighting
        Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        // Start with sharpening disabled
        ContrastAdaptiveSharpening {
            enabled: false,
            ..default()
        },
        // Environment lighting
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 150.0,
            ..default()
        },
        // Fog for depth
        DistanceFog {
            color: Color::srgba_u8(43, 44, 47, 255),
            falloff: FogFalloff::Linear {
                start: 1.0,
                end: 4.0,
            },
            ..default()
        },
    ));

    // 📝 UI text
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

// 🖊️ Helper function for menu formatting
fn draw_selectable_menu_item(ui: &mut String, label: &str, shortcut: char, enabled: bool) {
    let star = if enabled { "*" } else { "" };
    let _ = writeln!(*ui, "({shortcut}) {star}{label}{star}");
}

// 🎨 Creates a High-Contrast Debug Texture
//
// This texture is perfect for seeing aliasing artifacts - it has
// sharp color transitions that make jagged edges very visible
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    // Define a colorful palette with high contrast
    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    // Create the texture data
    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);  // Shift colors for each row
    }

    // Build the image
    let mut img = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::default());
    img
}

// 🎓 Deep Dive: Anti-Aliasing Techniques Compared
//
// **No AA**: Raw pixels, maximum sharpness but jagged edges
// - Best for: Pixel art, retro aesthetics
// - Performance: Fastest possible
//
// **MSAA (Multi-Sample Anti-Aliasing)**:
// - How: Renders multiple samples per pixel at geometry edges
// - Pros: Hardware accelerated, no blur, works well with deferred rendering
// - Cons: High memory usage, doesn't help with shader aliasing
// - Performance: 2x = Good, 4x = Moderate, 8x = Heavy
//
// **FXAA (Fast Approximate Anti-Aliasing)**:
// - How: Post-process filter that detects and smooths edges
// - Pros: Very fast, works on everything (geometry + shaders)
// - Cons: Can blur textures, may miss some edges
// - Performance: Minimal impact
//
// **SMAA (Enhanced Subpixel Morphological Anti-Aliasing)**:
// - How: Advanced edge detection with pattern matching
// - Pros: Better edge detection than FXAA, less blur
// - Cons: More expensive than FXAA
// - Performance: Low to moderate impact
//
// **TAA (Temporal Anti-Aliasing)**:
// - How: Accumulates samples across multiple frames
// - Pros: Excellent quality, handles all aliasing types
// - Cons: Can cause ghosting on fast motion, adds latency
// - Performance: Moderate impact
//
// 💡 Pro Tips:
// - For competitive gaming: FXAA or low MSAA for minimal latency
// - For cinematic quality: TAA with sharpening
// - For best performance: FXAA with slight sharpening
// - For best quality/perf ratio: SMAA High or TAA