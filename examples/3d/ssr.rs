//! Demonstrates screen space reflections in deferred rendering.
//!
//! 🪞 The Magic Mirror Screen: Understanding Screen Space Reflections
//!
//! Imagine you're looking at a perfectly still lake. You see the trees and
//! sky reflected in the water's surface, creating a mirror image of the world
//! above. Screen Space Reflections (SSR) creates this same magic digitally!
//! Unlike traditional reflection methods that require expensive ray tracing,
//! SSR is clever - it uses what's already drawn on screen to create reflections.
//! It's like having a digital mirror that only reflects what you can see!
//!
//! 🎯 What You'll See:
//! - Animated water with realistic ripples and reflections
//! - A rotating cube (Bevy logo) or flight helmet floating above
//! - Perfect reflections of the object in the water surface
//! - Environment map reflections of a beautiful Italian cathedral
//! - Real-time SSR toggle to see the dramatic difference
//!
//! 🎮 Controls:
//! - `Space`: Toggle Screen Space Reflections on/off
//! - `Enter`: Switch between cube and flight helmet models  
//! - `WASD`: Orbit camera around the scene
//! - `Mouse Wheel`: Zoom in/out
//!
//! 🔑 Key Concepts:
//! - Screen Space: Uses only visible pixels for reflections
//! - Deferred Rendering: Required for SSR efficiency
//! - Ray Marching: Steps through screen space to find reflections
//! - Fresnel Effect: Reflections stronger at glancing angles
//! - Animated Normals: Water ripples change reflection angles

use std::ops::Range;

use bevy::{
    anti_aliasing::fxaa::Fxaa,
    color::palettes::css::{BLACK, WHITE},
    core_pipeline::Skybox,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    input::mouse::MouseWheel,
    math::{vec3, vec4},
    pbr::{
        DefaultOpaqueRendererMethod, ExtendedMaterial, MaterialExtension, ScreenSpaceReflections,
    },
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef, ShaderType},
        view::Hdr,
    },
};

/// 🌊 Water shader path - creates the animated ripple effect
const SHADER_ASSET_PATH: &str = "shaders/water_material.wgsl";

// 🎥 Camera Control Constants
const CAMERA_KEYBOARD_ZOOM_SPEED: f32 = 0.1;    // WASD zoom speed
const CAMERA_KEYBOARD_ORBIT_SPEED: f32 = 0.02;  // WASD orbit speed
const CAMERA_MOUSE_WHEEL_ZOOM_SPEED: f32 = 0.25; // Mouse wheel sensitivity

// 📏 Camera distance limits - keeps objects in view
const CAMERA_ZOOM_RANGE: Range<f32> = 2.0..12.0;

static TURN_SSR_OFF_HELP_TEXT: &str = "Press Space to turn screen-space reflections off";
static TURN_SSR_ON_HELP_TEXT: &str = "Press Space to turn screen-space reflections on";
static MOVE_CAMERA_HELP_TEXT: &str =
    "Press WASD or use the mouse wheel to pan and orbit the camera";
static SWITCH_TO_FLIGHT_HELMET_HELP_TEXT: &str = "Press Enter to switch to the flight helmet model";
static SWITCH_TO_CUBE_HELP_TEXT: &str = "Press Enter to switch to the cube model";

/// 🌊 Custom Water Material: Bringing the Surface to Life
/// This extends StandardMaterial with animated water effects
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct Water {
    /// 🗺️ Normal map texture for surface detail
    /// 
    /// Normal maps define surface bumps and ripples. CRITICAL: This must
    /// NOT be loaded as sRGB - normal maps store direction vectors, not colors!
    #[texture(100)]
    #[sampler(101)]
    normals: Handle<Image>,

    /// ⚙️ Animation parameters passed to the water shader
    #[uniform(102)]
    settings: WaterSettings,
}

/// 🌊 Water Animation Parameters: The Physics of Digital Waves
#[derive(ShaderType, Debug, Clone)]
struct WaterSettings {
    /// 🔄 Wave motion vectors - how fast waves move in U/V directions
    /// We use multiple octaves (layers) for realistic complexity
    /// Two octaves packed per Vec4 for GPU efficiency
    octave_vectors: [Vec4; 2],
    
    /// 📏 Wave scales - controls wavelength (how wide the waves are)
    /// Smaller values = tighter ripples, larger = broad waves
    octave_scales: Vec4,
    
    /// 💪 Wave amplitudes - controls wave height and intensity
    /// Higher values = more pronounced surface disturbance
    octave_strengths: Vec4,
}

/// 🎛️ Application State: User's Current Choices
#[derive(Resource)]
struct AppSettings {
    /// 🪞 Whether screen space reflections are enabled
    /// Toggle this to see the dramatic difference SSR makes!
    ssr_on: bool,
    
    /// 🎭 Which 3D model is currently being displayed
    /// Switch between cube and flight helmet to see different reflection shapes
    displayed_model: DisplayedModel,
}

/// 🎭 Model Selection: Choose Your Reflection Subject
#[derive(Default)]
enum DisplayedModel {
    /// 📦 Simple cube with Bevy logo - great for seeing SSR basics
    #[default]
    Cube,
    /// 🛡️ Complex flight helmet - shows SSR with detailed geometry
    FlightHelmet,
}

/// A marker component for the cube model.
#[derive(Component)]
struct CubeModel;

/// A marker component for the flight helmet model.
#[derive(Component)]
struct FlightHelmetModel;

fn main() {
    // 🎬 SSR requires deferred rendering for efficiency
    // In deferred rendering, we first render geometry info to multiple
    // render targets (G-buffers), then do lighting in a second pass.
    // This gives SSR access to all the screen-space data it needs!
    App::new()
        // 📊 Enable deferred rendering - REQUIRED for SSR
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        // 🎛️ Initialize our app settings
        .init_resource::<AppSettings>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Screen Space Reflections Example".into(),
                ..default()
            }),
            ..default()
        }))
        // 🌊 Register our custom water material
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default())
        // 🚀 Set up systems
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_model)      // Spin the objects
        .add_systems(Update, move_camera)       // Handle camera controls
        .add_systems(Update, adjust_app_settings) // Handle SSR/model toggle
        .run();
}

// 🏗️ Scene Setup: Building Our Reflection Showcase
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
    asset_server: Res<AssetServer>,
    app_settings: Res<AppSettings>,
) {
    // 🎭 Create our reflection subjects
    spawn_cube(&mut commands, &asset_server, &mut meshes, &mut standard_materials);
    spawn_flight_helmet(&mut commands, &asset_server);
    
    // 🌊 Create the reflective water surface
    spawn_water(&mut commands, &asset_server, &mut meshes, &mut water_materials);
    
    // 📸 Set up the camera with SSR enabled
    spawn_camera(&mut commands, &asset_server);
    
    // 📝 Add interactive help text
    spawn_text(&mut commands, &app_settings);
}

// 📦 Bevy Logo Cube: Simple but Effective Reflection Subject
fn spawn_cube(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    standard_materials: &mut Assets<StandardMaterial>,
) {
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(standard_materials.add(StandardMaterial {
                base_color: Color::from(WHITE),  // White base for logo visibility
                // 🦆 Bevy logo texture adds visual interest to reflections
                base_color_texture: Some(asset_server.load("branding/icon.png")),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, 0.0),  // Hover above water surface
        ))
        .insert(CubeModel);  // Tag for visibility control
}

// 🛡️ Flight Helmet: Complex Geometry for Advanced SSR Testing
fn spawn_flight_helmet(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        SceneRoot(
            // 🎨 High-quality glTF model with multiple materials
            asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
        Transform::from_scale(Vec3::splat(2.5)),  // Scale up for visibility
        FlightHelmetModel,    // Tag for visibility control
        Visibility::Hidden,   // Start hidden, cube is default
    ));
}

// 🌊 Animated Water Surface: The Star of Our SSR Show
fn spawn_water(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    water_materials: &mut Assets<ExtendedMaterial<StandardMaterial, Water>>,
) {
    commands.spawn((
        // 🏞️ Large horizontal plane to serve as our water surface
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)))),
        MeshMaterial3d(water_materials.add(ExtendedMaterial {
            // 🎨 Base material: Perfect mirror properties
            base: StandardMaterial {
                base_color: BLACK.into(),        // Dark base lets reflections shine
                perceptual_roughness: 0.0,       // Perfect smoothness = sharp reflections
                ..default()
            },
            // 🌊 Water animation extension
            extension: Water {
                // 🗺️ Normal map for surface ripples
                normals: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                    "textures/water_normals.png",
                    |settings| {
                        // ⚠️ CRITICAL: Normal maps are NOT sRGB!
                        settings.is_srgb = false;
                        // 🔄 Repeating sampler for tiled water texture
                        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            ..default()
                        });
                    },
                ),
                // 🌊 Multi-octave wave animation parameters
                // These create complex, natural-looking water motion
                settings: WaterSettings {
                    // 🔄 Direction and speed vectors for wave motion
                    octave_vectors: [
                        vec4(0.080, 0.059, 0.073, -0.062),   // Octaves 1&2
                        vec4(0.153, 0.138, -0.149, -0.195), // Octaves 3&4
                    ],
                    // 📏 Wave sizes (smaller = tighter ripples)
                    octave_scales: vec4(1.0, 2.1, 7.9, 14.9) * 5.0,
                    // 💪 Wave heights (larger = more pronounced waves)
                    octave_strengths: vec4(0.16, 0.18, 0.093, 0.044),
                },
            },
        })),
        Transform::from_scale(Vec3::splat(100.0)),  // Huge surface for endless water
    ));
}

// 📸 Camera Setup: Your Window into the SSR World
fn spawn_camera(commands: &mut Commands, asset_server: &AssetServer) {
    // 🎥 Position camera for optimal SSR viewing
    // Slight angle to water surface is perfect for seeing reflections!
    commands
        .spawn((
            Camera3d::default(),
            // 🎯 Position: slightly above and to the side for great reflection view
            Transform::from_translation(vec3(-1.25, 2.25, 4.5)).looking_at(Vec3::ZERO, Vec3::Y),
            // 🌟 HDR for proper reflection brightness
            Hdr,
            // ❌ MSAA must be off for deferred rendering
            Msaa::Off,
        })
        // 🏛️ Beautiful Italian cathedral environment for rich reflections
        .insert(EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 5000.0,  // Bright enough to see in reflections
            ..default()
        })
        // 🌌 Skybox provides background visible in reflections
        .insert(Skybox {
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            brightness: 5000.0,
            ..default()
        })
        // 🪞 THE STAR: Enable Screen Space Reflections!
        .insert(ScreenSpaceReflections::default())
        // ✨ FXAA for smoother edges (works with deferred rendering)
        .insert(Fxaa::default());
}

// Spawns the help text.
fn spawn_text(commands: &mut Commands, app_settings: &AppSettings) {
    commands.spawn((
        create_text(app_settings),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// Creates or recreates the help text.
fn create_text(app_settings: &AppSettings) -> Text {
    format!(
        "{}\n{}\n{}",
        match app_settings.displayed_model {
            DisplayedModel::Cube => SWITCH_TO_FLIGHT_HELMET_HELP_TEXT,
            DisplayedModel::FlightHelmet => SWITCH_TO_CUBE_HELP_TEXT,
        },
        if app_settings.ssr_on {
            TURN_SSR_OFF_HELP_TEXT
        } else {
            TURN_SSR_ON_HELP_TEXT
        },
        MOVE_CAMERA_HELP_TEXT
    )
    .into()
}

impl MaterialExtension for Water {
    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

/// 🎠 Model Animation: Slow Rotation for Dynamic Reflections
/// Rotation helps showcase how SSR updates in real-time
fn rotate_model(
    mut query: Query<&mut Transform, Or<(With<CubeModel>, With<FlightHelmetModel>)>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        // 🔄 Gentle Y-axis rotation - not too fast to distract from reflections
        transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, time.elapsed_secs(), 0.0);
    }
}

// 🎮 Camera Control System: Explore SSR from Different Angles
fn move_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_input: EventReader<MouseWheel>,
    mut cameras: Query<&mut Transform, With<Camera>>,
) {
    let (mut distance_delta, mut theta_delta) = (0.0, 0.0);

    // ⌨️ Keyboard controls for smooth camera movement
    if keyboard_input.pressed(KeyCode::KeyW) {
        distance_delta -= CAMERA_KEYBOARD_ZOOM_SPEED;  // Move closer
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        distance_delta += CAMERA_KEYBOARD_ZOOM_SPEED;  // Move farther
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        theta_delta += CAMERA_KEYBOARD_ORBIT_SPEED;    // Orbit left
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        theta_delta -= CAMERA_KEYBOARD_ORBIT_SPEED;    // Orbit right
    }

    // 🖱️ Mouse wheel for quick zoom
    for mouse_wheel_event in mouse_wheel_input.read() {
        distance_delta -= mouse_wheel_event.y * CAMERA_MOUSE_WHEEL_ZOOM_SPEED;
    }

    // 🎥 Apply camera movement
    for mut camera_transform in cameras.iter_mut() {
        let local_z = camera_transform.local_z().as_vec3().normalize_or_zero();
        
        // 🔍 Handle zoom (maintain direction, change distance)
        if distance_delta != 0.0 {
            camera_transform.translation = (camera_transform.translation.length() + distance_delta)
                .clamp(CAMERA_ZOOM_RANGE.start, CAMERA_ZOOM_RANGE.end)  // Prevent extreme zoom
                * local_z;
        }
        
        // 🔄 Handle orbit (rotate around center point)
        if theta_delta != 0.0 {
            camera_transform
                .translate_around(Vec3::ZERO, Quat::from_axis_angle(Vec3::Y, theta_delta));
            camera_transform.look_at(Vec3::ZERO, Vec3::Y);  // Always look at center
        }
    }
}

// 🎛️ Settings Control System: Toggle SSR and Switch Models
fn adjust_app_settings(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_settings: ResMut<AppSettings>,
    mut cameras: Query<Entity, With<Camera>>,
    mut cube_models: Query<&mut Visibility, (With<CubeModel>, Without<FlightHelmetModel>)>,
    mut flight_helmet_models: Query<&mut Visibility, (Without<CubeModel>, With<FlightHelmetModel>)>,
    mut text: Query<&mut Text>,
) {
    // 🎯 Track if we need to update anything (for efficiency)
    let mut any_changes = false;

    // 🪞 Space key: Toggle SSR on/off
    // This is the star of the show - see the dramatic difference!
    if keyboard_input.just_pressed(KeyCode::Space) {
        app_settings.ssr_on = !app_settings.ssr_on;
        any_changes = true;
    }

    // 🎭 Enter key: Switch between models
    // Different shapes create different reflection patterns
    if keyboard_input.just_pressed(KeyCode::Enter) {
        app_settings.displayed_model = match app_settings.displayed_model {
            DisplayedModel::Cube => DisplayedModel::FlightHelmet,
            DisplayedModel::FlightHelmet => DisplayedModel::Cube,
        };
        any_changes = true;
    }

    // ⚡ Early exit if nothing changed (performance optimization)
    if !any_changes {
        return;
    }

    // 🪞 Apply SSR settings to camera
    for camera in cameras.iter_mut() {
        if app_settings.ssr_on {
            // ✅ Enable SSR with default settings
            commands
                .entity(camera)
                .insert(ScreenSpaceReflections::default());
        } else {
            // ❌ Remove SSR component to disable
            commands.entity(camera).remove::<ScreenSpaceReflections>();
        }
    }

    // 📦 Control cube visibility
    for mut cube_visibility in cube_models.iter_mut() {
        *cube_visibility = match app_settings.displayed_model {
            DisplayedModel::Cube => Visibility::Visible,
            _ => Visibility::Hidden,
        }
    }

    // 🛡️ Control flight helmet visibility
    for mut flight_helmet_visibility in flight_helmet_models.iter_mut() {
        *flight_helmet_visibility = match app_settings.displayed_model {
            DisplayedModel::FlightHelmet => Visibility::Visible,
            _ => Visibility::Hidden,
        };
    }

    // 📝 Update help text to reflect current state
    for mut text in text.iter_mut() {
        *text = create_text(&app_settings);
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ssr_on: true,           // Start with SSR enabled to show the effect
            displayed_model: default(),  // Start with cube
        }
    }
}

// 🎓 Deep Dive: The Science of Screen Space Reflections
//
// **The Problem with Traditional Reflections:**
// Real-time reflections are expensive! Ray tracing traces rays from the camera
// through reflection points, requiring complex scene traversal. Reflection
// probes work but need pre-computation and can't capture dynamic objects.
//
// **The SSR Solution:**
// SSR is clever - it reuses pixels already rendered to the screen! For each
// reflective pixel, we trace a ray through screen space (not world space)
// to find what should be reflected.
//
// **The Algorithm:**
// ```
// for each reflective pixel:
//     ray_start = pixel_world_position
//     ray_direction = reflect(view_direction, surface_normal)
//     
//     // March through screen space
//     for each step along ray:
//         screen_pos = project_to_screen(ray_position)
//         if (ray_depth > scene_depth[screen_pos]):
//             // Found intersection!
//             reflection_color = screen_color[screen_pos]
//             break
// ```
//
// **Why Deferred Rendering?**
// SSR needs access to:
// - World positions (from depth buffer)
// - Surface normals (for reflection direction)
// - Material properties (metallic/roughness)
// 
// Deferred rendering stores all this in G-buffers, making SSR efficient!
//
// **Ray Marching vs Ray Tracing:**
// - Ray Tracing: Precise intersections with scene geometry
// - Ray Marching: Steps through screen space checking depth
// SSR uses ray marching because it's faster and works with any geometry

// 💡 SSR Advantages:
//
// **Performance:**
// - Much faster than full ray tracing
// - Reuses already-rendered pixels
// - Works well with modern GPU parallel processing
//
// **Dynamic Objects:**
// - Reflects moving objects in real-time
// - No pre-computation required
// - Perfect for animated scenes
//
// **Easy Integration:**
// - Post-processing effect
// - Works with any material/shader
// - Minimal scene setup required

// ⚠️ SSR Limitations:
//
// **Screen Space Only:**
// - Can't reflect what's not on screen
// - Objects behind the camera won't appear
// - Reflections "fade out" at screen edges
//
// **Temporal Inconsistency:**
// - Reflections appear/disappear as objects move
// - Can cause "popping" artifacts
// - Solution: Temporal filtering, fallback to probes
//
// **Depth Buffer Precision:**
// - Limited by depth buffer resolution
// - Can miss thin objects
// - Self-intersection artifacts possible
//
// **Performance Scaling:**
// - Cost increases with reflection complexity
// - More reflective surfaces = higher cost
// - Ray marching steps affect quality vs performance

// 🎨 Artist Tips for Better SSR:
//
// **Material Setup:**
// - Use roughness to control reflection clarity
// - Higher metallic values show more reflection
// - Fresnel effect makes water/glass look realistic
//
// **Scene Design:**
// - Place interesting objects where they'll be reflected
// - Consider camera angles for best reflection views
// - Use environment maps as fallback for screen-space limits
//
// **Performance Optimization:**
// - Lower ray marching steps for mobile
// - Use half-resolution SSR rendering
// - Temporal upsampling for quality
// - Importance sampling for noise reduction

// 🔧 Technical Configuration:
//
// **Ray Marching Parameters:**
// - Step size: Smaller = more accurate, slower
// - Max steps: Higher = longer traces, more expensive
// - Thickness bias: Prevents self-intersection
//
// **Quality Settings:**
// - Sample count: More samples = less noise
// - Temporal filter strength: Reduces flickering
// - Depth fade: Smooth falloff at distance
// - Edge fade: Blend at screen boundaries
//
// **Integration with Other Effects:**
// - Works great with TAA (temporal anti-aliasing)
// - Complements environment maps perfectly
// - Enhanced by good HDR tone mapping
