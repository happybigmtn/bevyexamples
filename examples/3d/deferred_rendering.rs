//! This example compares Forward, Forward + Prepass, and Deferred rendering.
//!
//! 🎭 The Three Faces of Rendering: Understanding Rendering Techniques
//!
//! Imagine you're painting a complex scene with hundreds of objects and lights.
//! Would you paint each object completely with all its lighting effects one by one?
//! Or would you first sketch everything, then add colors, then lighting? This is
//! the fundamental choice between Forward and Deferred rendering!
//!
//! 🎯 What You'll See:
//! - A complex scene with multiple objects, lights, and materials
//! - Real-time switching between three rendering methods
//! - Performance characteristics of each approach
//! - Parallax mapping and advanced material effects
//!
//! 🎮 Controls:
//! - `1`: Switch to Deferred Rendering
//! - `2`: Switch to Forward Rendering  
//! - `3`: Switch to Forward + Prepass Rendering
//! - `Space`: Pause/resume animation
//! - `H`: Hide/show UI
//!
//! 🔑 Key Concepts:
//! - Forward Rendering: Paint each object completely, one at a time
//! - Deferred Rendering: First draw geometry, then apply lighting
//! - Prepass: Render depth/normals first to optimize shading
//! - G-Buffer: The "geometry buffer" storing per-pixel data

use std::f32::consts::*;

use bevy::{
    anti_aliasing::fxaa::Fxaa,
    core_pipeline::prepass::{DeferredPrepass, DepthPrepass, MotionVectorPrepass, NormalPrepass},
    image::ImageLoaderSettings,
    math::ops,
    pbr::{
        CascadeShadowConfigBuilder, DefaultOpaqueRendererMethod, DirectionalLightShadowMap,
        NotShadowCaster, NotShadowReceiver, OpaqueRendererMethod,
    },
    prelude::*,
};

fn main() {
    App::new()
        // 🎨 Start with deferred rendering as the default
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        // 🗺️ High-resolution shadow maps for quality
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        // ⏸️ Start paused so you can see the initial state
        .insert_resource(Pause(true))
        .add_systems(Startup, (setup, setup_parallax))
        .add_systems(Update, (animate_light_direction, switch_mode, spin))
        .run();
}

// 🏗️ Scene Setup: Creating a Complex Test Environment
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // 📷 Camera Configuration for Deferred Rendering
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        // ⚠️ MSAA (Multi-Sample Anti-Aliasing) MUST be off for Deferred rendering!
        // Why? MSAA works by sampling geometry edges multiple times, but deferred
        // rendering writes to textures (G-Buffer) which can't use MSAA directly
        Msaa::Off,
        // 🌫️ Atmospheric fog for depth
        DistanceFog {
            color: Color::srgb_u8(43, 44, 47),
            falloff: FogFalloff::Linear {
                start: 1.0,
                end: 8.0,
            },
            ..default()
        },
        // 🌍 IBL (Image-Based Lighting) for realistic reflections
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2000.0,
            ..default()
        },
        // 🎬 Prepass Components: The Secret Sauce!
        // These tell Bevy what data to render in the first pass:
        DepthPrepass,         // Z-depth for early Z rejection
        MotionVectorPrepass,  // For temporal effects like TAA
        DeferredPrepass,      // Full G-Buffer generation
        Fxaa::default(),      // Post-process anti-aliasing (works with deferred!)
    ));

    // ☀️ Directional Light with Cascaded Shadow Maps
    commands.spawn((
        DirectionalLight {
            illuminance: 15_000.,
            shadows_enabled: true,
            ..default()
        },
        // 🗺️ Cascaded shadows for better quality at different distances
        CascadeShadowConfigBuilder {
            num_cascades: 3,          // Three levels of detail
            maximum_distance: 10.0,   // Shadow range
            ..default()
        }
        .build(),
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, -FRAC_PI_4)),
    ));

    // 🚁 FlightHelmet: A Complex Model for Testing
    // This model has many materials and details - perfect for comparing render modes!
    let helmet_scene = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf"));

    // Spawn two helmets for visual variety
    commands.spawn(SceneRoot(helmet_scene.clone()));
    commands.spawn((
        SceneRoot(helmet_scene),
        Transform::from_xyz(-4.0, 0.0, -3.0),
    ));

    // 🎨 Create a Forward-Only Material
    // This demonstrates that you can mix rendering methods in the same scene!
    let mut forward_mat: StandardMaterial = Color::srgb(0.1, 0.2, 0.1).into();
    forward_mat.opaque_render_method = OpaqueRendererMethod::Forward;
    let forward_mat_h = materials.add(forward_mat);

    // 🌍 Ground Plane - Always rendered with forward rendering
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(forward_mat_h.clone()),
    ));

    let cube_h = meshes.add(Cuboid::new(0.1, 0.1, 0.1));
    let sphere_h = meshes.add(Sphere::new(0.125).mesh().uv(32, 18));

    // 📦 Test Cubes - Simple geometry with forward rendering
    commands.spawn((
        Mesh3d(cube_h.clone()),
        MeshMaterial3d(forward_mat_h.clone()),
        Transform::from_xyz(-0.3, 0.5, -0.2),
    ));
    commands.spawn((
        Mesh3d(cube_h),
        MeshMaterial3d(forward_mat_h),
        Transform::from_xyz(0.2, 0.5, 0.2),
    ));

    // 💡 Emissive Light Source
    // This creates both a visible glowing sphere AND a point light
    let sphere_color = Color::srgb(10.0, 4.0, 1.0);  // HDR color (>1.0)!
    let sphere_pos = Transform::from_xyz(0.4, 0.5, -0.8);
    
    // 🌟 The visible glowing sphere
    let mut unlit_mat: StandardMaterial = sphere_color.into();
    unlit_mat.unlit = true;  // Skip lighting calculations - it IS the light!
    commands.spawn((
        Mesh3d(sphere_h.clone()),
        MeshMaterial3d(materials.add(unlit_mat)),
        sphere_pos,
        NotShadowCaster,  // Lights don't cast shadows from themselves
    ));
    
    // 💡 The actual light source at the same position
    commands.spawn((
        PointLight {
            intensity: 800.0,
            radius: 0.125,
            shadows_enabled: true,
            color: sphere_color,
            ..default()
        },
        sphere_pos,
    ));

    // 🎨 Array of Test Spheres: Perfect for Comparing Rendering Methods!
    // We create 6 spheres in two rows with different colors
    for i in 0..6 {
        let j = i % 3;
        let s_val = if i < 3 { 0.0 } else { 0.2 };  // Different brightness for each row
        
        // 🌈 Create RGB-tinted materials
        let material = if j == 0 {
            materials.add(StandardMaterial {
                base_color: Color::srgb(s_val, s_val, 1.0),  // Blue tint
                perceptual_roughness: 0.089,  // Very shiny!
                metallic: 0.0,
                ..default()
            })
        } else if j == 1 {
            materials.add(StandardMaterial {
                base_color: Color::srgb(s_val, 1.0, s_val),  // Green tint
                perceptual_roughness: 0.089,
                metallic: 0.0,
                ..default()
            })
        } else {
            materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, s_val, s_val),  // Red tint
                perceptual_roughness: 0.089,
                metallic: 0.0,
                ..default()
            })
        };
        
        // 📍 Position spheres in a grid pattern
        commands.spawn((
            Mesh3d(sphere_h.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(
                j as f32 * 0.25 + if i < 3 { -0.15 } else { 0.15 } - 0.4,
                0.125,
                -j as f32 * 0.25 + if i < 3 { -0.15 } else { 0.15 } + 0.4,
            ),
        ));
    }

    // 🌌 Sky Box - A massive inverted cube for the background
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("888888").unwrap().into(),
            unlit: true,      // No lighting calculations needed
            cull_mode: None,  // Render both sides (we're inside it!)
            ..default()
        })),
        Transform::from_scale(Vec3::splat(1_000_000.0)),  // HUGE scale!
        NotShadowCaster,    // Sky doesn't cast shadows
        NotShadowReceiver,  // Sky doesn't receive shadows
    ));

    // 📝 UI Instructions
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

// ⏸️ Pause state for animation control
#[derive(Resource)]
struct Pause(bool);

// 🌅 Animate the Sun: Dynamic Lighting Demo
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
    pause: Res<Pause>,
) {
    if pause.0 {
        return;
    }
    
    // 🔄 Rotate the directional light to simulate sun movement
    // This helps showcase how different rendering methods handle dynamic lighting
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * PI / 5.0);
    }
}

// 🎭 Parallax Mapping: Adding Depth Without Geometry!
//
// Parallax mapping creates the illusion of depth on flat surfaces by offsetting
// texture coordinates based on view angle. It's like a magic trick for your GPU!
fn setup_parallax(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    // 🗺️ Normal Map Setup
    // Normal maps store surface direction in RGB channels
    // Pro tip: To create in GIMP: Filters → Generic → Normal Map with "flip X"
    let normal_handle = asset_server.load_with_settings(
        "textures/parallax_example/cube_normal.png",
        // ⚠️ CRITICAL: Normal maps use linear color space, not sRGB!
        // Wrong color space = wrong lighting direction
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );

    let mut cube = Mesh::from(Cuboid::new(0.15, 0.15, 0.15));

    // 📐 Generate Tangents: Essential for Normal Mapping!
    // Tangents define the "texture space" coordinate system
    // Without them, the GPU can't interpret normal map directions correctly
    cube.generate_tangents().unwrap();

    // 🎨 Parallax Material: The Full Package
    let parallax_material = materials.add(StandardMaterial {
        perceptual_roughness: 0.4,
        base_color_texture: Some(asset_server.load("textures/parallax_example/cube_color.png")),
        normal_map_texture: Some(normal_handle),
        // 🏔️ Depth Map: Height information
        // Black pixels = raised, White pixels = recessed
        // This creates the 3D illusion!
        depth_map: Some(asset_server.load("textures/parallax_example/cube_depth.png")),
        parallax_depth_scale: 0.09,  // How "deep" the effect appears
        // 🔍 Relief Mapping: The highest quality parallax technique
        // More steps = better quality but slower
        parallax_mapping_method: ParallaxMappingMethod::Relief { max_steps: 4 },
        // 💻 Layer count: 2^5 = 32 layers for smooth transitions
        max_parallax_layer_count: ops::exp2(5.0f32),
        ..default()
    });
    
    // 🎲 Spawn the parallax cube with rotation
    commands.spawn((
        Mesh3d(meshes.add(cube)),
        MeshMaterial3d(parallax_material),
        Transform::from_xyz(0.4, 0.2, -0.8),
        Spin { speed: 0.3 },  // Slow rotation to show the effect
    ));
}
// 🔄 Spinning component for animated objects
#[derive(Component)]
struct Spin {
    speed: f32,
}

// 🎲 Spin Animation System
// Creates a tumbling motion to showcase materials from all angles
fn spin(time: Res<Time>, mut query: Query<(&mut Transform, &Spin)>, pause: Res<Pause>) {
    if pause.0 {
        return;
    }
    
    // 🌀 Rotate on all axes for maximum visual interest
    for (mut transform, spin) in query.iter_mut() {
        transform.rotate_local_y(spin.speed * time.delta_secs());
        transform.rotate_local_x(spin.speed * time.delta_secs());
        transform.rotate_local_z(-spin.speed * time.delta_secs());
    }
}

// 🎮 Rendering Mode Selection
#[derive(Resource, Default)]
enum DefaultRenderMode {
    #[default]
    Deferred,
    Forward,
    ForwardPrepass,
}

// 🎮 Mode Switching System: The Heart of This Example!
//
// This system lets you compare rendering techniques in real-time
fn switch_mode(
    mut text: Single<&mut Text>,
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut default_opaque_renderer_method: ResMut<DefaultOpaqueRendererMethod>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<Entity, With<Camera>>,
    mut pause: ResMut<Pause>,
    mut hide_ui: Local<bool>,
    mut mode: Local<DefaultRenderMode>,
) {
    text.clear();

    // ⏸️ Toggle pause
    if keys.just_pressed(KeyCode::Space) {
        pause.0 = !pause.0;
    }

    // 🎨 Mode 1: Deferred Rendering
    // The "draw first, light later" approach
    if keys.just_pressed(KeyCode::Digit1) {
        *mode = DefaultRenderMode::Deferred;
        default_opaque_renderer_method.set_to_deferred();
        println!("DefaultOpaqueRendererMethod: Deferred");
        
        // 🔧 Force material updates (triggers shader recompilation)
        for _ in materials.iter_mut() {}
        
        // 📷 Configure camera for deferred rendering
        for camera in &cameras {
            commands.entity(camera).remove::<NormalPrepass>();
            commands.entity(camera).insert(DepthPrepass);
            commands.entity(camera).insert(MotionVectorPrepass);
            commands.entity(camera).insert(DeferredPrepass);  // The key component!
        }
    }
    
    // 🎨 Mode 2: Forward Rendering
    // The traditional "paint each object completely" approach
    if keys.just_pressed(KeyCode::Digit2) {
        *mode = DefaultRenderMode::Forward;
        default_opaque_renderer_method.set_to_forward();
        println!("DefaultOpaqueRendererMethod: Forward");
        
        for _ in materials.iter_mut() {}
        
        // 📷 Remove all prepass components for pure forward rendering
        for camera in &cameras {
            commands.entity(camera).remove::<NormalPrepass>();
            commands.entity(camera).remove::<DepthPrepass>();
            commands.entity(camera).remove::<MotionVectorPrepass>();
            commands.entity(camera).remove::<DeferredPrepass>();
        }
    }
    
    // 🎨 Mode 3: Forward + Prepass
    // A hybrid approach: depth/normal prepass + forward shading
    if keys.just_pressed(KeyCode::Digit3) {
        *mode = DefaultRenderMode::ForwardPrepass;
        default_opaque_renderer_method.set_to_forward();
        println!("DefaultOpaqueRendererMethod: Forward + Prepass");
        
        for _ in materials.iter_mut() {}
        
        // 📷 Enable prepasses but keep forward shading
        for camera in &cameras {
            commands.entity(camera).insert(NormalPrepass);
            commands.entity(camera).insert(DepthPrepass);
            commands.entity(camera).insert(MotionVectorPrepass);
            commands.entity(camera).remove::<DeferredPrepass>();  // No deferred!
        }
    }

    // 👁️ Toggle UI visibility
    if keys.just_pressed(KeyCode::KeyH) {
        *hide_ui = !*hide_ui;
    }

    // 📝 Update UI text
    if !*hide_ui {
        text.push_str("(H) Hide UI\n");
        text.push_str("(Space) Play/Pause\n\n");
        text.push_str("Rendering Method:\n");

        text.push_str(&format!(
            "(1) {} Deferred\n",
            if let DefaultRenderMode::Deferred = *mode {
                ">"
            } else {
                ""
            }
        ));
        text.push_str(&format!(
            "(2) {} Forward\n",
            if let DefaultRenderMode::Forward = *mode {
                ">"
            } else {
                ""
            }
        ));
        text.push_str(&format!(
            "(3) {} Forward + Prepass\n",
            if let DefaultRenderMode::ForwardPrepass = *mode {
                ">"
            } else {
                ""
            }
        ));
    }
}

// 🎓 Deep Dive: Forward vs Deferred Rendering
//
// **Forward Rendering** (Traditional approach):
// 1. For each object:
//    - Calculate lighting from ALL lights
//    - Write final color to framebuffer
// 
// Pros:
// - Works with transparency
// - Supports MSAA
// - Simple and straightforward
// - Good for few lights
//
// Cons:
// - O(objects × lights) complexity
// - Lots of redundant calculations
// - Poor with many lights
//
// **Deferred Rendering** (Modern approach):
// 1. First pass: Render geometry data to G-Buffer textures:
//    - Albedo (base color)
//    - Normals (surface direction)
//    - Depth (distance from camera)
//    - Material properties (roughness, metallic)
// 2. Second pass: Calculate lighting using G-Buffer data
//
// Pros:
// - O(pixels × lights) complexity
// - Excellent with many lights
// - No redundant shading
// - Easy to add screen-space effects
//
// Cons:
// - High memory bandwidth (G-Buffer)
// - No transparency support (traditionally)
// - No MSAA (must use post-process AA)
// - Limited material variety
//
// **Forward + Prepass** (Hybrid approach):
// 1. Z-Prepass: Render depth only
// 2. Forward pass with early-Z rejection
//
// Pros:
// - Reduces overdraw in forward rendering
// - Keeps transparency support
// - Good middle ground
//
// Cons:
// - Extra geometry pass
// - Still limited by light count

// 💡 When to Use Each:
//
// **Use Forward Rendering when:**
// - You have few lights (< 10)
// - You need transparency
// - You're on mobile/low-end hardware
// - You need MSAA
//
// **Use Deferred Rendering when:**
// - You have many lights (> 50)
// - Most objects are opaque
// - You want screen-space effects
// - You're on desktop/console
//
// **Use Forward + Prepass when:**
// - You want better forward performance
// - You have moderate light counts
// - You need some transparency
