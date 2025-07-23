//! This example showcases light transmission
//!
//! # Light Transmission: See-Through Materials
//!
//! Light transmission is how light passes through translucent materials.
//! Think of light shining through stained glass, frosted windows, or a
//! glass of water. This creates beautiful effects in games and visualizations!
//!
//! ## Two Types of Transmission:
//!
//! 1. **Diffuse Transmission**: Light scatters as it passes through
//!    - Like light through paper, curtains, or skin
//!    - Creates soft, glowing effects
//!    - No clear view of what's behind
//!
//! 2. **Specular Transmission**: Light passes straight through
//!    - Like looking through glass or water
//!    - Objects behind are visible but may be distorted
//!    - Affected by thickness and Index of Refraction (IOR)
//!
//! ## Key Properties:
//!
//! - **Thickness**: How far light travels through the material
//! - **IOR**: Index of Refraction - how much light bends (water=1.33, glass=1.5)
//! - **Roughness**: Surface texture affecting clarity
//! - **Reflectance**: How much light bounces off vs passes through
//!
//! ## Controls
//!
//! | Key Binding        | Action                                               |
//! |:-------------------|:-----------------------------------------------------|
//! | `J`/`K`/`L`/`;`    | Change Screen Space Transmission Quality             |
//! | `O` / `P`          | Decrease / Increase Screen Space Transmission Steps  |
//! | `1` / `2`          | Decrease / Increase Diffuse Transmission             |
//! | `Q` / `W`          | Decrease / Increase Specular Transmission            |
//! | `A` / `S`          | Decrease / Increase Thickness                        |
//! | `Z` / `X`          | Decrease / Increase IOR                              |
//! | `E` / `R`          | Decrease / Increase Perceptual Roughness             |
//! | `U` / `I`          | Decrease / Increase Reflectance                      |
//! | Arrow Keys         | Control Camera                                       |
//! | `C`                | Randomize Colors                                     |
//! | `H`                | Toggle HDR + Bloom                                   |
//! | `D`                | Toggle Depth Prepass                                 |
//! | `T`                | Toggle TAA                                           |

// Rust: Import mathematical constant PI (3.14159...)
use std::f32::consts::PI;

// Rust: Complex nested imports from Bevy
use bevy::{
    // Rust: CSS color palette with named colors
    // * imports all colors (RED, BLUE, ANTIQUE_WHITE, etc.)
    color::palettes::css::*,
    // Rust: Core rendering pipeline components
    core_pipeline::{
        bloom::Bloom, // Post-processing glow effect
        core_3d::ScreenSpaceTransmissionQuality, // Quality enum for transmission
        prepass::DepthPrepass, // Pre-render depth information
        tonemapping::Tonemapping, // HDR to SDR conversion
    },
    // Rust: Mathematical operations module
    math::ops,
    // Rust: Physically Based Rendering components
    pbr::{
        NotShadowCaster,          // Marker: object doesn't cast shadows
        PointLightShadowMap,      // Shadow map configuration
        TransmittedShadowReceiver, // Marker: receives shadows through transmission
    },
    // Rust: Common Bevy types
    prelude::*,
    // Rust: Rendering subsystem types
    render::{
        camera::{Exposure, TemporalJitter}, // Camera exposure and anti-aliasing
        view::{ColorGrading, ColorGradingGlobal, Hdr}, // Post-processing effects
    },
};

// *Note:* TAA is not _required_ for specular transmission, but
// it _greatly enhances_ the look of the resulting blur effects.
// Sadly, it's not available under WebGL.
// Rust: Conditional compilation with cfg attribute
// Only compile this import if webgpu feature is enabled OR not targeting wasm32
#[cfg(any(feature = "webgpu", not(target_arch = "wasm32")))]
use bevy::anti_aliasing::taa::TemporalAntiAliasing;

// Rust: External crate for random number generation
use rand::random;  // random() function for f32 values

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Add standard Bevy plugins
        .add_plugins(DefaultPlugins)
        // Rust: Insert global resources
        // ClearColor sets background color
        .insert_resource(ClearColor(Color::BLACK))
        // Rust: Configure shadow map resolution
        // Struct literal with field name
        .insert_resource(PointLightShadowMap { size: 2048 })  // 2048x2048 shadow map
        // Rust: Disable ambient lighting for dramatic effect
        .insert_resource(AmbientLight {
            brightness: 0.0,  // Completely dark ambient
            ..default()       // Other fields use defaults
        })
        // Rust: Register systems for different schedules
        .add_systems(Startup, setup)
        // Rust: Tuple of systems run every frame
        .add_systems(Update, (example_control_system, flicker_system))
        // Rust: Start the game loop
        .run();
}

/// set up a simple 3D scene
// Rust: System function with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Mutable access to mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to material assets
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Rust: Read-only asset server for loading files
    asset_server: Res<AssetServer>,
) {
    // Rust: Create mesh assets and store handles
    // Method chaining: new() -> mesh() -> ico() -> unwrap()
    let icosphere_mesh = meshes.add(
        Sphere::new(0.9)    // Sphere with radius 0.9
            .mesh()         // Convert to mesh builder
            .ico(7)         // Icosphere with 7 subdivisions
            .unwrap()       // Unwrap Result (ico can fail)
    );
    // Rust: Simple cube mesh
    let cube_mesh = meshes.add(Cuboid::new(0.7, 0.7, 0.7));  // 0.7x0.7x0.7 cube
    // Rust: Plane mesh with custom size
    let plane_mesh = meshes.add(
        Plane3d::default()     // Default plane
            .mesh()            // Convert to mesh builder
            .size(2.0, 2.0)    // 2x2 units
    );
    // Rust: Cylinder with resolution for smoothness
    let cylinder_mesh = meshes.add(
        Cylinder::new(0.5, 2.0)    // radius 0.5, height 2.0
            .mesh()                // Convert to mesh builder
            .resolution(50)        // 50 segments for smooth circle
    );

    // Cube #1
    // Rust: Spawn entity with tuple of components
    commands.spawn((
        // Rust: Clone mesh handle for reuse
        Mesh3d(cube_mesh.clone()),
        // Rust: Create default material inline
        // :: is the path separator for associated functions
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        // Rust: Transform with position and rotation
        Transform::from_xyz(0.25, 0.5, -2.0)  // Position
            .with_rotation(Quat::from_euler(   // Add rotation
                EulerRot::XYZ,  // Rotation order (X then Y then Z)
                1.4,            // X rotation in radians
                3.7,            // Y rotation in radians
                21.3,           // Z rotation in radians
            )),
        // Rust: Custom component with struct literal
        ExampleControls {
            color: true,                      // Can change color
            specular_transmission: false,     // Not glass-like
            diffuse_transmission: false,      // Not translucent
        },
    ));

    // Cube #2
    commands.spawn((
        // Rust: Move cube_mesh (no clone - transferring ownership)
        Mesh3d(cube_mesh),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(-0.75, 0.7, -2.0).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.4,
            2.3,
            4.7,
        )),
        ExampleControls {
            color: true,
            specular_transmission: false,
            diffuse_transmission: false,
        },
    ));

    // Candle
    commands.spawn((
        // Rust: Use cylinder mesh for candle body
        Mesh3d(cylinder_mesh),
        // Rust: Create translucent wax material
        MeshMaterial3d(materials.add(StandardMaterial {
            // Rust: RGB color values (0.0-1.0 range)
            base_color: Color::srgb(0.9, 0.2, 0.3),  // Reddish wax
            // Rust: f32 literal for transmission amount
            diffuse_transmission: 0.7,  // 70% light passes through (subsurface scattering)
            perceptual_roughness: 0.32, // Slightly rough surface
            thickness: 0.2,             // Thin wax walls
            // Rust: Fill other fields with defaults
            ..default()
        })),
        // Rust: Position at left side
        Transform::from_xyz(-1.0, 0.0, 0.0),
        // Rust: Mark as having diffuse transmission controls
        ExampleControls {
            color: true,
            specular_transmission: false,  // Not glass-like
            diffuse_transmission: true,    // Wax is translucent
        },
    ));

    // Candle Flame
    // Rust: Color math in linear color space for accurate blending
    // From trait converts CSS colors to LinearRgba
    let scaled_white = LinearRgba::from(ANTIQUE_WHITE) * 20.;   // Bright white core
    let scaled_orange = LinearRgba::from(ORANGE_RED) * 4.;      // Orange glow
    // Rust: Manual struct construction with field names
    let emissive = LinearRgba {
        // Rust: Add color components for flame gradient
        red: scaled_white.red + scaled_orange.red,
        green: scaled_white.green + scaled_orange.green,
        blue: scaled_white.blue + scaled_orange.blue,
        alpha: 1.0,  // Fully opaque
    };

    commands.spawn((
        // Rust: Clone icosphere for flame shape
        Mesh3d(icosphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            // Rust: Field shorthand - emissive: emissive
            emissive,  // Makes the flame glow
            // Rust: Full transmission for volumetric effect
            diffuse_transmission: 1.0,  // 100% translucent
            ..default()
        })),
        // Rust: Position and scale for flame shape
        Transform::from_xyz(-1.0, 1.15, 0.0)  // Above candle
            .with_scale(Vec3::new(0.1, 0.2, 0.1)),  // Stretched vertically
        // Rust: Marker components
        Flicker,         // Animate this entity
        NotShadowCaster, // Flames don't cast shadows
    ));

    // Glass Sphere
    commands.spawn((
        Mesh3d(icosphere_mesh.clone()),
        // Rust: Glass-like material with transmission
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,     // Clear glass
            // Rust: High specular transmission for transparency
            specular_transmission: 0.9,   // 90% see-through
            diffuse_transmission: 1.0,    // Also allows diffuse light
            // Rust: Physical properties of glass
            thickness: 1.8,               // Thick glass sphere
            ior: 1.5,                     // Index of Refraction (glass ~= 1.5)
            perceptual_roughness: 0.12,   // Slightly rough for realism
            ..default()
        })),
        Transform::from_xyz(1.0, 0.0, 0.0),  // Right side position
        // Rust: Enable specular transmission controls
        ExampleControls {
            color: true,
            specular_transmission: true,   // Can adjust glass properties
            diffuse_transmission: false,   // Focus on specular only
        },
    ));

    // R Sphere
    commands.spawn((
        Mesh3d(icosphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.into(),
            specular_transmission: 0.9,
            diffuse_transmission: 1.0,
            thickness: 1.8,
            ior: 1.5,
            perceptual_roughness: 0.12,
            ..default()
        })),
        Transform::from_xyz(1.0, -0.5, 2.0).with_scale(Vec3::splat(0.5)),
        ExampleControls {
            color: true,
            specular_transmission: true,
            diffuse_transmission: false,
        },
    ));

    // G Sphere
    commands.spawn((
        Mesh3d(icosphere_mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: LIME.into(),
            specular_transmission: 0.9,
            diffuse_transmission: 1.0,
            thickness: 1.8,
            ior: 1.5,
            perceptual_roughness: 0.12,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 2.0).with_scale(Vec3::splat(0.5)),
        ExampleControls {
            color: true,
            specular_transmission: true,
            diffuse_transmission: false,
        },
    ));

    // B Sphere
    commands.spawn((
        Mesh3d(icosphere_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: BLUE.into(),
            specular_transmission: 0.9,
            diffuse_transmission: 1.0,
            thickness: 1.8,
            ior: 1.5,
            perceptual_roughness: 0.12,
            ..default()
        })),
        Transform::from_xyz(-1.0, -0.5, 2.0).with_scale(Vec3::splat(0.5)),
        ExampleControls {
            color: true,
            specular_transmission: true,
            diffuse_transmission: false,
        },
    ));

    // Chessboard Plane
    let black_material = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        reflectance: 0.3,
        perceptual_roughness: 0.8,
        ..default()
    });

    let white_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        reflectance: 0.3,
        perceptual_roughness: 0.8,
        ..default()
    });

    for x in -3..4 {
        for z in -3..4 {
            commands.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(if (x + z) % 2 == 0 {
                    black_material.clone()
                } else {
                    white_material.clone()
                }),
                Transform::from_xyz(x as f32 * 2.0, -1.0, z as f32 * 2.0),
                ExampleControls {
                    color: true,
                    specular_transmission: false,
                    diffuse_transmission: false,
                },
            ));
        }
    }

    // Paper
    commands.spawn((
        Mesh3d(plane_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            diffuse_transmission: 0.6,
            perceptual_roughness: 0.8,
            reflectance: 1.0,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, -3.0)
            .with_scale(Vec3::new(2.0, 1.0, 1.0))
            .with_rotation(Quat::from_euler(EulerRot::XYZ, PI / 2.0, 0.0, 0.0)),
        TransmittedShadowReceiver,
        ExampleControls {
            specular_transmission: false,
            color: false,
            diffuse_transmission: true,
        },
    ));

    // Candle Light
    commands.spawn((
        Transform::from_xyz(-1.0, 1.7, 0.0),
        PointLight {
            color: Color::from(
                LinearRgba::from(ANTIQUE_WHITE).mix(&LinearRgba::from(ORANGE_RED), 0.2),
            ),
            intensity: 4_000.0,
            radius: 0.2,
            range: 5.0,
            shadows_enabled: true,
            ..default()
        },
        Flicker,
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.0, 1.8, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
        ColorGrading {
            global: ColorGradingGlobal {
                post_saturation: 1.2,
                ..default()
            },
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Exposure { ev100: 6.0 },
        #[cfg(any(feature = "webgpu", not(target_arch = "wasm32")))]
        Msaa::Off,
        #[cfg(any(feature = "webgpu", not(target_arch = "wasm32")))]
        TemporalAntiAliasing::default(),
        EnvironmentMapLight {
            intensity: 25.0,
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            ..default()
        },
        Bloom::default(),
    ));

    // Controls Text
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        ExampleDisplay,
    ));
}

// Rust: Derive macro generates Component trait implementation
#[derive(Component)]
// Rust: Zero-size marker type (no fields)
struct Flicker;  // Tags entities that should flicker

// Rust: Component for controlling which properties can be adjusted
#[derive(Component)]
struct ExampleControls {
    // Rust: bool fields for feature flags
    diffuse_transmission: bool,   // Can adjust diffuse transmission?
    specular_transmission: bool,  // Can adjust specular transmission?
    color: bool,                  // Can randomize color?
}

// Rust: Regular struct (not a component) for UI state
// No derive macros - manually implementing traits
struct ExampleState {
    // Rust: f32 fields for material properties
    diffuse_transmission: f32,    // 0.0-1.0 range
    specular_transmission: f32,   // 0.0-1.0 range
    thickness: f32,               // Material thickness
    ior: f32,                     // Index of refraction
    perceptual_roughness: f32,    // Surface roughness
    reflectance: f32,             // How reflective
    // Rust: bool for camera behavior
    auto_camera: bool,            // Auto-rotate camera?
}

// Rust: Marker component for UI text
#[derive(Component)]
struct ExampleDisplay;

// Rust: Manual trait implementation for Default
impl Default for ExampleState {
    // Rust: Associated function returns Self (the implementing type)
    fn default() -> Self {
        // Rust: Construct with sensible defaults
        ExampleState {
            diffuse_transmission: 0.5,    // 50% diffuse
            specular_transmission: 0.9,   // 90% specular (glass-like)
            thickness: 1.8,               // Medium thickness
            ior: 1.5,                     // Glass IOR
            perceptual_roughness: 0.12,   // Slightly rough
            reflectance: 0.5,             // 50% reflective
            auto_camera: true,            // Start with auto-rotation
        }
    }
}

// Rust: Complex system with many parameters for interactive controls
fn example_control_system(
    // Rust: Commands for entity modifications
    mut commands: Commands,
    // Rust: Mutable access to material assets
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Rust: Query for controllable entities
    controllable: Query<(&MeshMaterial3d<StandardMaterial>, &ExampleControls)>,
    // Rust: Single query with complex tuple type
    camera: Single<
        (
            Entity,                      // Camera entity ID
            &mut Camera3d,               // Mutable camera component
            &mut Transform,              // Mutable transform
            Option<&DepthPrepass>,       // Optional depth prepass
            Option<&TemporalJitter>,     // Optional TAA jitter
            Has<Hdr>,                    // Bool-like: has HDR?
        ),
        With<Camera3d>,  // Filter: only entities with Camera3d
    >,
    // Rust: Single mutable text for UI display
    mut display: Single<&mut Text, With<ExampleDisplay>>,
    // Rust: Local<T> provides system-local state
    // Persists between system runs, initialized with Default
    mut state: Local<ExampleState>,
    // Rust: Time resource for delta calculations
    time: Res<Time>,
    // Rust: Input state for keyboard
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.pressed(KeyCode::Digit2) {
        state.diffuse_transmission = (state.diffuse_transmission + time.delta_secs()).min(1.0);
    } else if input.pressed(KeyCode::Digit1) {
        state.diffuse_transmission = (state.diffuse_transmission - time.delta_secs()).max(0.0);
    }

    if input.pressed(KeyCode::KeyW) {
        state.specular_transmission = (state.specular_transmission + time.delta_secs()).min(1.0);
    } else if input.pressed(KeyCode::KeyQ) {
        state.specular_transmission = (state.specular_transmission - time.delta_secs()).max(0.0);
    }

    if input.pressed(KeyCode::KeyS) {
        state.thickness = (state.thickness + time.delta_secs()).min(5.0);
    } else if input.pressed(KeyCode::KeyA) {
        state.thickness = (state.thickness - time.delta_secs()).max(0.0);
    }

    if input.pressed(KeyCode::KeyX) {
        state.ior = (state.ior + time.delta_secs()).min(3.0);
    } else if input.pressed(KeyCode::KeyZ) {
        state.ior = (state.ior - time.delta_secs()).max(1.0);
    }

    if input.pressed(KeyCode::KeyI) {
        state.reflectance = (state.reflectance + time.delta_secs()).min(1.0);
    } else if input.pressed(KeyCode::KeyU) {
        state.reflectance = (state.reflectance - time.delta_secs()).max(0.0);
    }

    if input.pressed(KeyCode::KeyR) {
        state.perceptual_roughness = (state.perceptual_roughness + time.delta_secs()).min(1.0);
    } else if input.pressed(KeyCode::KeyE) {
        state.perceptual_roughness = (state.perceptual_roughness - time.delta_secs()).max(0.0);
    }

    let randomize_colors = input.just_pressed(KeyCode::KeyC);

    for (material_handle, controls) in &controllable {
        let material = materials.get_mut(material_handle).unwrap();
        if controls.specular_transmission {
            material.specular_transmission = state.specular_transmission;
            material.thickness = state.thickness;
            material.ior = state.ior;
            material.perceptual_roughness = state.perceptual_roughness;
            material.reflectance = state.reflectance;
        }

        if controls.diffuse_transmission {
            material.diffuse_transmission = state.diffuse_transmission;
        }

        if controls.color && randomize_colors {
            material.base_color =
                Color::srgba(random(), random(), random(), material.base_color.alpha());
        }
    }

    let (camera_entity, mut camera_3d, mut camera_transform, depth_prepass, temporal_jitter, hdr) =
        camera.into_inner();

    if input.just_pressed(KeyCode::KeyH) {
        if hdr {
            commands.entity(camera_entity).remove::<Hdr>();
        } else {
            commands.entity(camera_entity).insert(Hdr);
        }
    }

    #[cfg(any(feature = "webgpu", not(target_arch = "wasm32")))]
    if input.just_pressed(KeyCode::KeyD) {
        if depth_prepass.is_none() {
            commands.entity(camera_entity).insert(DepthPrepass);
        } else {
            commands.entity(camera_entity).remove::<DepthPrepass>();
        }
    }

    #[cfg(any(feature = "webgpu", not(target_arch = "wasm32")))]
    if input.just_pressed(KeyCode::KeyT) {
        if temporal_jitter.is_none() {
            commands
                .entity(camera_entity)
                .insert((TemporalJitter::default(), TemporalAntiAliasing::default()));
        } else {
            commands
                .entity(camera_entity)
                .remove::<(TemporalJitter, TemporalAntiAliasing)>();
        }
    }

    if input.just_pressed(KeyCode::KeyO) && camera_3d.screen_space_specular_transmission_steps > 0 {
        camera_3d.screen_space_specular_transmission_steps -= 1;
    }

    if input.just_pressed(KeyCode::KeyP) && camera_3d.screen_space_specular_transmission_steps < 4 {
        camera_3d.screen_space_specular_transmission_steps += 1;
    }

    if input.just_pressed(KeyCode::KeyJ) {
        camera_3d.screen_space_specular_transmission_quality = ScreenSpaceTransmissionQuality::Low;
    }

    if input.just_pressed(KeyCode::KeyK) {
        camera_3d.screen_space_specular_transmission_quality =
            ScreenSpaceTransmissionQuality::Medium;
    }

    if input.just_pressed(KeyCode::KeyL) {
        camera_3d.screen_space_specular_transmission_quality = ScreenSpaceTransmissionQuality::High;
    }

    if input.just_pressed(KeyCode::Semicolon) {
        camera_3d.screen_space_specular_transmission_quality =
            ScreenSpaceTransmissionQuality::Ultra;
    }

    let rotation = if input.pressed(KeyCode::ArrowRight) {
        state.auto_camera = false;
        time.delta_secs()
    } else if input.pressed(KeyCode::ArrowLeft) {
        state.auto_camera = false;
        -time.delta_secs()
    } else if state.auto_camera {
        time.delta_secs() * 0.25
    } else {
        0.0
    };

    let distance_change =
        if input.pressed(KeyCode::ArrowDown) && camera_transform.translation.length() < 25.0 {
            time.delta_secs()
        } else if input.pressed(KeyCode::ArrowUp) && camera_transform.translation.length() > 2.0 {
            -time.delta_secs()
        } else {
            0.0
        };

    camera_transform.translation *= ops::exp(distance_change);

    camera_transform.rotate_around(
        Vec3::ZERO,
        Quat::from_euler(EulerRot::XYZ, 0.0, rotation, 0.0),
    );

    display.0 = format!(
        concat!(
            " J / K / L / ;  Screen Space Specular Transmissive Quality: {:?}\n",
            "         O / P  Screen Space Specular Transmissive Steps: {}\n",
            "         1 / 2  Diffuse Transmission: {:.2}\n",
            "         Q / W  Specular Transmission: {:.2}\n",
            "         A / S  Thickness: {:.2}\n",
            "         Z / X  IOR: {:.2}\n",
            "         E / R  Perceptual Roughness: {:.2}\n",
            "         U / I  Reflectance: {:.2}\n",
            "    Arrow Keys  Control Camera\n",
            "             C  Randomize Colors\n",
            "             H  HDR + Bloom: {}\n",
            "             D  Depth Prepass: {}\n",
            "             T  TAA: {}\n",
        ),
        camera_3d.screen_space_specular_transmission_quality,
        camera_3d.screen_space_specular_transmission_steps,
        state.diffuse_transmission,
        state.specular_transmission,
        state.thickness,
        state.ior,
        state.perceptual_roughness,
        state.reflectance,
        if hdr { "ON " } else { "OFF" },
        if cfg!(any(feature = "webgpu", not(target_arch = "wasm32"))) {
            if depth_prepass.is_some() {
                "ON "
            } else {
                "OFF"
            }
        } else {
            "N/A (WebGL)"
        },
        if cfg!(any(feature = "webgpu", not(target_arch = "wasm32"))) {
            if temporal_jitter.is_some() {
                if depth_prepass.is_some() {
                    "ON "
                } else {
                    "N/A (Needs Depth Prepass)"
                }
            } else {
                "OFF"
            }
        } else {
            "N/A (WebGL)"
        },
    );
}

// Rust: System for animating candle flame flicker
fn flicker_system(
    // Rust: Single query for flame mesh transform
    // Complex filter: has Flicker AND has Mesh3d
    mut flame: Single<&mut Transform, (With<Flicker>, With<Mesh3d>)>,
    // Rust: Single query for light with two mutable components
    // Filter: has Flicker but NOT Mesh3d (excludes flame mesh)
    light: Single<(&mut PointLight, &mut Transform), (With<Flicker>, Without<Mesh3d>)>,
    // Rust: Time for animation
    time: Res<Time>,
) {
    // Rust: Get elapsed time in seconds
    let s = time.elapsed_secs();
    // Rust: Layered sine waves for organic flicker
    // Multiple frequencies create natural randomness
    let a = ops::cos(s * 6.0) * 0.0125 + ops::cos(s * 4.0) * 0.025;
    let b = ops::cos(s * 5.0) * 0.0125 + ops::cos(s * 3.0) * 0.025;
    let c = ops::cos(s * 7.0) * 0.0125 + ops::cos(s * 2.0) * 0.025;
    
    // Rust: Destructure Single query result
    // into_inner() extracts the tuple from Single wrapper
    let (mut light, mut light_transform) = light.into_inner();
    
    // Rust: Animate light intensity
    light.intensity = 4_000.0 + 3000.0 * (a + b + c);  // Base + flicker
    
    // Rust: Reset flame position
    flame.translation = Vec3::new(-1.0, 1.23, 0.0);
    // Rust: Make flame "look at" flickering target
    flame.look_at(Vec3::new(-1.0 - c, 1.7 - b, 0.0 - a), Vec3::X);
    // Rust: Additional rotation for flame orientation
    flame.rotate(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, PI / 2.0));
    
    // Rust: Sync light position with flame movement
    light_transform.translation = Vec3::new(-1.0 - c, 1.7, 0.0 - a);
    flame.translation = Vec3::new(-1.0 - c, 1.23, 0.0 - a);
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Conditional Compilation**:
//    - `#[cfg(...)]` attributes control what code is compiled
//    - `any()` = OR condition, `not()` = negation
//    - Used for platform-specific features (WebGL vs native)
//
// 2. **Single Query**:
//    - More efficient than Query when expecting one result
//    - Panics if not exactly one match
//    - `.into_inner()` extracts the wrapped value
//
// 3. **Local State**:
//    - `Local<T>` provides per-system persistent state
//    - Initialized with Default trait on first run
//    - Survives between system executions
//
// 4. **Option in Queries**:
//    - `Option<&Component>` for components that might not exist
//    - `Has<Component>` for bool-like existence checks
//    - Allows flexible entity queries
//
// 5. **Complex Tuple Queries**:
//    - Can query many components at once
//    - Mix mutable and immutable references
//    - Combine with filters for precise selection
//
// 6. **Linear Color Space**:
//    - `LinearRgba` for accurate color math
//    - Multiplication and addition work correctly
//    - Convert from sRGB for display colors
//
// 7. **Transmission Properties**:
//    - `diffuse_transmission` - subsurface scattering
//    - `specular_transmission` - see-through clarity
//    - `ior` - how light bends (refraction)
//    - `thickness` - how far light travels
//
// 8. **Marker Components**:
//    - Zero-size types like `Flicker`
//    - Used for entity identification/behavior
//    - No runtime memory cost
