//! This example shows how to place reflection probes in the scene.
//!
//! 🪞 The Hall of Mirrors: Understanding Reflection Probes
//!
//! Have you ever noticed how a shiny car reflects not just the sky, but also
//! nearby buildings? That's what reflection probes do! They're like 360-degree
//! cameras that capture everything around them, then project those reflections
//! onto shiny objects. While environment maps show a distant world (like the sky),
//! reflection probes capture the local scene - including those colorful cubes
//! floating around our golden sphere!
//!
//! 🎯 What You'll See:
//! - A perfectly reflective golden sphere in the center
//! - Colorful cubes arranged around it
//! - Three reflection modes to compare:
//!   1. No reflections (sphere looks dull)
//!   2. Environment map only (reflects sky but not cubes)
//!   3. Reflection probe (reflects both sky AND cubes!)
//!
//! 🎮 Controls:
//! - `Space`: Cycle through reflection modes
//! - `Enter`: Start/stop camera rotation
//!
//! 🔑 Key Concepts:
//! - Environment Maps: Global reflections (skybox)
//! - Reflection Probes: Local reflections (nearby objects)
//! - Light Probes: Capture lighting at specific positions
//! - Metallic Materials: Only metals show clear reflections
//!
//! ⚠️ Note: Reflection probes don't work on WebGL 2 or WebGPU.
//!
//! Press Space to switch between no reflections, environment map reflections
//! (i.e. the skybox only, not the cubes), and a full reflection probe that
//! reflects the skybox and the cubes. Press Enter to pause rotation.

use bevy::{core_pipeline::Skybox, prelude::*, render::view::Hdr};

use std::{
    f32::consts::PI,
    fmt::{Display, Formatter, Result as FmtResult},
};

static STOP_ROTATION_HELP_TEXT: &str = "Press Enter to stop rotation";
static START_ROTATION_HELP_TEXT: &str = "Press Enter to start rotation";

static REFLECTION_MODE_HELP_TEXT: &str = "Press Space to switch reflection mode";

// 🎮 Application State: Managing Our Reflection Demo
// This resource tracks the current state of our interactive example
#[derive(Resource)]
struct AppStatus {
    // 🔄 Which environment maps the user has requested to display
    // We cycle through modes to demonstrate the visual differences
    reflection_mode: ReflectionMode,
    
    // 🎥 Whether the user has requested the scene to rotate
    // Camera rotation helps showcase how reflections change with viewing angle
    rotating: bool,
}

// 🎨 Reflection Display Modes: Different Ways to Reflect the World
#[derive(Clone, Copy)]
enum ReflectionMode {
    // 🚫 No environment maps are shown
    // The sphere appears dull, showing only direct lighting
    None = 0,
    
    // 🌍 Only a world environment map is shown
    // The sphere reflects the skybox but NOT the nearby cubes
    // This is the traditional approach - one reflection for everything
    EnvironmentMap = 1,
    
    // 🪞 Both world environment map AND reflection probe are present
    // The reflection probe takes precedence on the sphere
    // This creates accurate local reflections including the cubes!
    ReflectionProbe = 2,
}

// 🎭 Cubemap Collection: Our Reflection Texture Library
// Cubemaps are 6-sided textures that capture a 360° view of the environment
// Think of them as six cameras arranged in a cube, each capturing one direction
#[derive(Resource)]
struct Cubemaps {
    // 🌤️ The blurry diffuse cubemap for ambient lighting
    // This provides soft, indirect illumination from all directions
    // Used for both world environment map and reflection probe
    // (In production, you'd have separate ones for accuracy)
    diffuse: Handle<Image>,

    // 🌍 The specular cubemap that reflects the world, but NOT the cubes
    // This is what we traditionally use - captured from far away
    // Shows the skybox/distant environment
    specular_environment_map: Handle<Image>,

    // 🪞 The specular cubemap that reflects BOTH world AND cubes
    // This is our reflection probe - captured from inside the scene
    // Includes local geometry for accurate reflections
    specular_reflection_probe: Handle<Image>,

    // 🌌 The skybox cubemap image
    // What we see when looking at the sky
    // Almost identical to specular_environment_map for this example
    skybox: Handle<Image>,
}

fn main() {
    // 🚀 Initialize the Bevy App with reflection showcase systems
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<AppStatus>()      // Initialize demo state
        .init_resource::<Cubemaps>()       // Load reflection textures
        .add_systems(Startup, setup)       // Build the scene
        // 🎬 PreUpdate: Setup camera environment before rendering
        .add_systems(PreUpdate, add_environment_map_to_camera)
        // 🎮 Update: Handle user input and animations
        .add_systems(Update, change_reflection_type)  // Space key handler
        .add_systems(Update, toggle_rotation)         // Enter key handler
        .add_systems(
            Update,
            rotate_camera
                .after(toggle_rotation)         // Respect rotation toggle
                .after(change_reflection_type), // Camera might be recreated
        )
        .add_systems(Update, update_text.after(rotate_camera))  // UI last
        .run();
}

// 🏗️ Scene Setup: Building Our Reflection Showcase
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    app_status: Res<AppStatus>,
    cubemaps: Res<Cubemaps>,
) {
    // 🎨 Compose our scene step by step
    spawn_scene(&mut commands, &asset_server);       // Colorful cubes
    spawn_camera(&mut commands);                      // View camera
    spawn_sphere(&mut commands, &mut meshes, &mut materials);  // Mirror sphere
    spawn_reflection_probe(&mut commands, &cubemaps); // Local reflections
    spawn_text(&mut commands, &app_status);          // Help text
}

// 🎭 Scene Objects: The Colorful Cube Array
// These cubes are what make reflection probes special - they're local geometry
// that ONLY appears in reflections when using a reflection probe!
fn spawn_scene(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn(SceneRoot(
        // 📦 Load a pre-made scene with colorful cubes arranged around the origin
        // This GLTF file contains the cubes and lighting setup
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cubes/Cubes.glb")),
    ));
}

// 📸 Camera Setup: Our Window into the Reflection World
fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        Camera3d::default(),
        // 🎥 Position camera for a good view of sphere and cubes
        Transform::from_xyz(-6.483, 0.325, 4.381).looking_at(Vec3::ZERO, Vec3::Y),
        // 🌟 Enable HDR for better reflections and bloom
        Hdr,
    ));
}

// 🔮 The Golden Sphere: Our Perfect Mirror
// This sphere is the star of the show - it reflects everything around it!
fn spawn_sphere(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    // 🌐 Create a high-resolution sphere mesh
    // ico(7) creates a sphere with 7 subdivisions for smooth reflections
    let sphere_mesh = meshes.add(Sphere::new(1.0).mesh().ico(7).unwrap());

    // ✨ Spawn our reflective golden sphere
    commands.spawn((
        Mesh3d(sphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            // 🌟 Golden base color for that luxurious look
            base_color: Srgba::hex("#ffd891").unwrap().into(),
            
            // 🪞 Perfect mirror material settings:
            metallic: 1.0,              // Full metal = full reflections
            perceptual_roughness: 0.0,   // Zero roughness = mirror finish
            
            // The combination of metallic=1 and roughness=0 creates
            // a perfect mirror that will show crisp reflections
            ..StandardMaterial::default()
        })),
    ));
}

// 🪞 The Reflection Probe: Capturing Local Reflections
// This is the magic component that makes our sphere reflect the nearby cubes!
fn spawn_reflection_probe(commands: &mut Commands, cubemaps: &Cubemaps) {
    commands.spawn((
        // 🏷️ LightProbe marker - tells Bevy this is a local reflection volume
        LightProbe,
        
        // 🌍 Environment map configuration for this probe
        EnvironmentMapLight {
            // 🌤️ Diffuse lighting for soft ambient illumination
            diffuse_map: cubemaps.diffuse.clone(),
            
            // 🪞 Specular map WITH the cubes baked in!
            // This is what makes reflection probes special - they capture
            // the local scene, not just the distant environment
            specular_map: cubemaps.specular_reflection_probe.clone(),
            
            // 💡 High intensity for bright, visible reflections
            intensity: 5000.0,
            ..default()
        },
        
        // 📏 Scale the probe to encompass our sphere
        // The probe's influence volume is a sphere with radius = scale
        // We use 2.0 because our sphere has radius 1.0, giving some margin
        Transform::from_scale(Vec3::splat(2.0)),
    ));
}

// 📝 UI Setup: Interactive Help Text
fn spawn_text(commands: &mut Commands, app_status: &AppStatus) {
    // 🔤 Create help text showing current mode and controls
    commands.spawn((
        app_status.create_text(),  // Generate text based on current state
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),  // Position at bottom-left
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// 🌍 Camera Environment Setup: Adding Global Reflections
// This system runs in PreUpdate to catch cameras spawned by the GLTF loader
// We need this because the camera might be part of the loaded scene file
fn add_environment_map_to_camera(
    mut commands: Commands,
    query: Query<Entity, Added<Camera3d>>,
    cubemaps: Res<Cubemaps>,
) {
    // 📷 Find any newly added cameras and enhance them
    for camera_entity in query.iter() {
        commands
            .entity(camera_entity)
            // 🌍 Add global environment lighting
            // This provides reflections when no reflection probe is nearby
            .insert(create_camera_environment_map_light(&cubemaps))
            // 🌌 Add visible skybox for background
            .insert(Skybox {
                image: cubemaps.skybox.clone(),
                brightness: 5000.0,  // Bright sky for visibility
                ..default()
            });
    }
}

// 🔄 Reflection Mode Switcher: Interactive Demo Controls
// This system handles the Space key to cycle through reflection modes
fn change_reflection_type(
    mut commands: Commands,
    light_probe_query: Query<Entity, With<LightProbe>>,
    camera_query: Query<Entity, With<Camera3d>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_status: ResMut<AppStatus>,
    cubemaps: Res<Cubemaps>,
) {
    // 🎮 Only respond to Space key press
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    // 🔄 Cycle through modes: None → EnvironmentMap → ReflectionProbe → None
    app_status.reflection_mode =
        ReflectionMode::try_from((app_status.reflection_mode as u32 + 1) % 3).unwrap();

    // 🧹 Clean up existing reflection probes
    // We'll recreate them if needed based on the new mode
    for light_probe in light_probe_query.iter() {
        commands.entity(light_probe).despawn();
    }
    
    // 🪞 Spawn reflection probe only in ReflectionProbe mode
    match app_status.reflection_mode {
        ReflectionMode::None | ReflectionMode::EnvironmentMap => {
            // No local probe needed
        }
        ReflectionMode::ReflectionProbe => {
            // Create the probe that captures local reflections
            spawn_reflection_probe(&mut commands, &cubemaps)
        },
    }

    // 🌍 Update camera environment map based on mode
    for camera in camera_query.iter() {
        match app_status.reflection_mode {
            ReflectionMode::None => {
                // 🚫 Remove all environment lighting
                commands.entity(camera).remove::<EnvironmentMapLight>();
            }
            ReflectionMode::EnvironmentMap | ReflectionMode::ReflectionProbe => {
                // ✅ Add/ensure environment map exists
                // This provides global reflections as a fallback
                commands
                    .entity(camera)
                    .insert(create_camera_environment_map_light(&cubemaps));
            }
        }
    }
}

// 🎥 Camera Rotation Toggle: Enter to Start/Stop
fn toggle_rotation(keyboard: Res<ButtonInput<KeyCode>>, mut app_status: ResMut<AppStatus>) {
    if keyboard.just_pressed(KeyCode::Enter) {
        // 🔄 Toggle rotation state - helps visualize reflections from different angles
        app_status.rotating = !app_status.rotating;
    }
}

// 📝 UI Update System: Keep Help Text Current
fn update_text(mut text_query: Query<&mut Text>, app_status: Res<AppStatus>) {
    // 🔄 Update all text entities with current status
    for mut text in text_query.iter_mut() {
        *text = app_status.create_text();
    }
}

// 🔢 Mode Conversion: Integer to Enum
impl TryFrom<u32> for ReflectionMode {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ReflectionMode::None),
            1 => Ok(ReflectionMode::EnvironmentMap),
            2 => Ok(ReflectionMode::ReflectionProbe),
            _ => Err(()),  // Invalid mode number
        }
    }
}

// 📝 Display Implementation: User-Friendly Mode Names
impl Display for ReflectionMode {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let text = match *self {
            ReflectionMode::None => "No reflections",
            ReflectionMode::EnvironmentMap => "Environment map",
            ReflectionMode::ReflectionProbe => "Reflection probe",
        };
        formatter.write_str(text)
    }
}

impl AppStatus {
    // 📝 Help Text Generator: Dynamic UI Based on Current State
    fn create_text(&self) -> Text {
        // 🎥 Choose rotation help text based on current state
        let rotation_help_text = if self.rotating {
            STOP_ROTATION_HELP_TEXT   // "Press Enter to stop rotation"
        } else {
            START_ROTATION_HELP_TEXT  // "Press Enter to start rotation"
        };

        // 🔤 Combine all help text elements
        format!(
            "{}\n{}\n{}",
            self.reflection_mode,      // Current mode (e.g., "Reflection probe")
            rotation_help_text,        // Rotation control hint
            REFLECTION_MODE_HELP_TEXT  // "Press Space to switch reflection mode"
        )
        .into()
    }
}

// 🌍 Global Environment Light Factory
// Creates the world environment map light, used as a fallback when no
// reflection probe affects a mesh. This provides the "distant" reflections.
fn create_camera_environment_map_light(cubemaps: &Cubemaps) -> EnvironmentMapLight {
    EnvironmentMapLight {
        // 🌤️ Diffuse for soft ambient lighting
        diffuse_map: cubemaps.diffuse.clone(),
        
        // 🌍 Specular WITHOUT the cubes - just the skybox
        // This is why environment maps alone don't show local objects
        specular_map: cubemaps.specular_environment_map.clone(),
        
        // 💡 Match intensity with reflection probe for consistency
        intensity: 5000.0,
        ..default()
    }
}

// 🎥 Camera Orbit System: Cinematic Rotation
fn rotate_camera(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    app_status: Res<AppStatus>,
) {
    // 🛑 Only rotate if enabled
    if !app_status.rotating {
        return;
    }

    // 🔄 Orbit camera around the origin
    for mut transform in camera_query.iter_mut() {
        // 📐 Calculate circular motion in the XZ plane
        // We rotate the camera's position while keeping Y constant
        transform.translation = Vec2::from_angle(time.delta_secs() * PI / 5.0)
            .rotate(transform.translation.xz())  // Rotate XZ coordinates
            .extend(transform.translation.y)     // Add back Y height
            .xzy();                             // Convert back to 3D
        
        // 👀 Always look at the center where our sphere is
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

// 🎨 Cubemap Asset Loading: Preparing Our Reflection Textures
impl FromWorld for Cubemaps {
    fn from_world(world: &mut World) -> Self {
        // 🏛️ Load the Pisa cathedral environment map
        // This beautiful Italian scene provides our skybox and distant reflections
        // KTX2 format with RGB9E5 encoding for HDR lighting
        let specular_map = world.load_asset("environment_maps/pisa_specular_rgb9e5_zstd.ktx2");

        Cubemaps {
            // 🌤️ Blurry version for diffuse lighting
            diffuse: world.load_asset("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            
            // 🪞 Special cubemap WITH the cubes baked in!
            // This is what makes reflection probes special
            specular_reflection_probe: world
                .load_asset("environment_maps/cubes_reflection_probe_specular_rgb9e5_zstd.ktx2"),
            
            // 🌍 Environment map without cubes (distant only)
            specular_environment_map: specular_map.clone(),
            
            // 🌌 Reuse specular map for skybox (saves space)
            // In production, you'd have a dedicated skybox texture
            skybox: specular_map,
        }
    }
}

// 🎬 Default Settings: Start with the Most Impressive Mode
impl Default for AppStatus {
    fn default() -> Self {
        Self {
            // 🪞 Start with reflection probe to show the full effect
            reflection_mode: ReflectionMode::ReflectionProbe,
            // 🎥 Start rotating for dynamic view
            rotating: true,
        }
    }
}

// 🎓 Deep Dive: The Science of Reflection Probes
//
// **What's the Problem?**
// Traditional environment maps assume the entire scene is infinitely far away.
// This works great for skyboxes but fails for local reflections. Imagine a
// shiny car in a garage - it should reflect the nearby walls, not just the sky!
//
// **The Solution: Reflection Probes**
// Reflection probes are like 360° cameras placed throughout your scene. Each
// probe captures a cubemap from its position, including all nearby geometry.
// Objects use the nearest probe for accurate local reflections.
//
// **How Cubemaps Work**:
// A cubemap is six square textures arranged like the faces of a cube:
// - +X (Right), -X (Left)
// - +Y (Up), -Y (Down)  
// - +Z (Forward), -Z (Back)
//
// To sample a reflection, we:
// 1. Calculate the reflection vector
// 2. Intersect it with the cubemap cube
// 3. Sample the texture at that point
//
// **Probe Influence Volumes**:
// Each probe has a volume of influence (usually a sphere or box). Objects
// inside this volume will use the probe's reflections. When multiple probes
// overlap, various blending strategies can be used.
//
// **Diffuse vs Specular**:
// - Diffuse: Heavily blurred for ambient lighting (irradiance)
// - Specular: Sharp or slightly blurred for reflections (radiance)
// The blur amount depends on surface roughness - rough surfaces need blurrier
// reflections!
//
// **Pre-filtered Environment Maps**:
// Instead of blurring in real-time, we pre-compute different blur levels
// (mip levels) offline. Each mip level corresponds to a different roughness
// value. This is much faster than real-time convolution!

// 💡 Practical Use Cases:
//
// **Indoor Scenes**:
// - Place probes in each room
// - Captures room-specific lighting and reflections
// - Essential for realistic interiors
//
// **Vehicles**:
// - Probe inside the car for dashboard reflections
// - Probe outside for body reflections
// - Switch based on camera position
//
// **Water & Mirrors**:
// - Probe at water surface level
// - Captures both above and below water
// - Creates perfect mirror reflections
//
// **Performance Tips**:
// - Fewer, well-placed probes > many probes
// - Use box volumes for rectangular rooms
// - Bake static probes offline when possible
// - Update dynamic probes sparingly (expensive!)
//
// **Common Issues**:
// - Light leaking: Probe captures light through walls
//   Solution: Use smaller influence volumes
// - Parallax errors: Reflections don't align with geometry
//   Solution: Box-projected cubemaps
// - Seams between probes: Visible transitions
//   Solution: Probe blending and careful placement
