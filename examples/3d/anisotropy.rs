//! Demonstrates anisotropy with the glTF sample barn lamp model.
//!
//! 🔬 The Science of Anisotropic Materials
//! Imagine brushing your hand across velvet - it feels different depending on
//! which direction you stroke. That's anisotropy! In graphics, anisotropic
//! materials reflect light differently based on the viewing angle and the
//! material's "grain" direction. Think brushed metal, hair, fabric, or vinyl records.
//!
//! 🎨 What You'll See:
//! - A barn lamp with brushed metal surfaces
//! - Press Space to cycle through different lighting conditions
//! - Press Enter to toggle anisotropy on/off and see the dramatic difference
//! - Press Q to switch between the barn lamp and a test sphere
//!
//! 💡 Key Concepts:
//! - Anisotropy: Direction-dependent light reflection
//! - Tangent space: The coordinate system that defines the "grain" direction
//! - Environment mapping: Using images to provide realistic lighting

use std::fmt::Display;

use bevy::{
    color::palettes::{self, css::WHITE},
    core_pipeline::Skybox,
    math::vec3,
    prelude::*,
    time::Stopwatch,
};

/// The initial position of the camera.
const CAMERA_INITIAL_POSITION: Vec3 = vec3(-0.4, 0.0, 0.0);

/// The current settings of the app, as chosen by the user.
#[derive(Resource)]
struct AppStatus {
    /// Which type of light is in the scene.
    light_mode: LightMode,
    /// Whether anisotropy is enabled.
    anisotropy_enabled: bool,
    /// Which mesh is visible
    visible_scene: Scene,
}

/// Which type of light we're using: a directional light, a point light, or an
/// environment map.
#[derive(Clone, Copy, PartialEq, Default)]
enum LightMode {
    /// A rotating directional light.
    #[default]
    Directional,
    /// A rotating point light.
    Point,
    /// An environment map (image-based lighting, including skybox).
    EnvironmentMap,
}

// 🎭 Material Variants: The A/B Testing Pattern
//
// This component stores both versions of each material - with and without
// anisotropy. This elegant pattern allows instant switching without recreating
// materials, perfect for comparing visual effects in real-time.
#[derive(Component)]
struct MaterialVariants {
    /// The version of the material in the glTF file, with anisotropy.
    anisotropic: Handle<StandardMaterial>,
    /// The version of the material with anisotropy removed.
    isotropic: Handle<StandardMaterial>,
}

// 🎬 Scene Selection
#[derive(Default, Clone, Copy, PartialEq, Eq, Component)]
enum Scene {
    #[default]
    BarnLamp,
    Sphere,
}

impl Scene {
    fn next(&self) -> Self {
        match self {
            Self::BarnLamp => Self::Sphere,
            Self::Sphere => Self::BarnLamp,
        }
    }
}

impl Display for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scene_name = match self {
            Self::BarnLamp => "Barn Lamp",
            Self::Sphere => "Sphere",
        };
        write!(f, "{scene_name}")
    }
}

fn main() {
    App::new()
        .init_resource::<AppStatus>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Anisotropy Example".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, create_material_variants)
        .add_systems(Update, animate_light)
        .add_systems(Update, rotate_camera)
        .add_systems(Update, (handle_input, update_help_text).chain())
        .run();
}

// 🏗️ Scene Construction
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, app_status: Res<AppStatus>) {
    // 📷 Camera positioned for optimal viewing
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(CAMERA_INITIAL_POSITION).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // ☀️ Start with directional light
    spawn_directional_light(&mut commands);

    // 🏮 Load the Barn Lamp Model
    // This glTF model includes anisotropic materials that simulate
    // brushed metal surfaces on the lamp shade
    commands.spawn((
        SceneRoot(asset_server.load("models/AnisotropyBarnLamp/AnisotropyBarnLamp.gltf#Scene0")),
        Transform::from_xyz(0.0, 0.07, -0.13),
        Scene::BarnLamp,
    ));

    // 🔮 Create Test Sphere with Anisotropic Material
    // This sphere helps visualize how anisotropy affects curved surfaces
    commands.spawn((
        Mesh3d(
            asset_server.add(
                Mesh::from(Sphere::new(0.1))
                    // 🔑 Critical: Generate tangents for anisotropy!
                    // Tangents define the "grain" direction at each vertex
                    .with_generated_tangents()
                    .unwrap(),
            ),
        ),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: palettes::tailwind::GRAY_300.into(),
            // 🌀 Anisotropy rotation: angle of the "grain" (0-1 maps to 0-2π)
            anisotropy_rotation: 0.5,
            // 💪 Anisotropy strength: how pronounced the effect is (0-1)
            anisotropy_strength: 1.,
            ..default()
        })),
        Scene::Sphere,
        Visibility::Hidden,  // Start with lamp visible
    ));

    spawn_text(&mut commands, &app_status);
}

// 📝 UI Helper Text
fn spawn_text(commands: &mut Commands, app_status: &AppStatus) {
    commands.spawn((
        app_status.create_help_text(),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// 🔄 Material Variant Creation System
//
// This clever system automatically creates non-anisotropic versions of all
// anisotropic materials. It runs whenever new meshes with materials are added.
fn create_material_variants(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // 🔍 Query for newly added meshes that don't have variants yet
    new_meshes: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        (
            Added<MeshMaterial3d<StandardMaterial>>,
            Without<MaterialVariants>,
        ),
    >,
) {
    for (entity, anisotropic_material_handle) in new_meshes.iter() {
        // 📦 Clone the original material to preserve all properties
        let Some(anisotropic_material) = materials.get(anisotropic_material_handle).cloned() else {
            continue;
        };

        // 🎭 Create the variant pair
        commands.entity(entity).insert(MaterialVariants {
            anisotropic: anisotropic_material_handle.0.clone(),
            // 🚫 Disable anisotropy by zeroing all related properties
            isotropic: materials.add(StandardMaterial {
                anisotropy_texture: None,
                anisotropy_strength: 0.0,
                anisotropy_rotation: 0.0,
                ..anisotropic_material  // Keep everything else the same
            }),
        });
    }
}

// 🌟 Light Animation System
//
// Creates dynamic lighting by rotating lights around the scene.
// This helps showcase how anisotropic materials respond to changing light angles.
fn animate_light(
    mut lights: Query<&mut Transform, Or<(With<DirectionalLight>, With<PointLight>)>>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs();
    for mut transform in lights.iter_mut() {
        // 🎡 Circular motion in the XZ plane
        transform.translation = vec3(ops::cos(now), 1.0, ops::sin(now)) * vec3(3.0, 4.0, 3.0);
        // 👀 Always look at the center
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

// 📷 Camera Rotation for Environment Map Mode
//
// When using environment mapping, we rotate the camera to show how
// the anisotropic reflections change with viewing angle.
fn rotate_camera(
    mut camera: Query<&mut Transform, With<Camera>>,
    app_status: Res<AppStatus>,
    time: Res<Time>,
    mut stopwatch: Local<Stopwatch>,
) {
    // ⏱️ Only tick the stopwatch in environment map mode
    if app_status.light_mode == LightMode::EnvironmentMap {
        stopwatch.tick(time.delta());
    }

    let now = stopwatch.elapsed_secs();
    for mut transform in camera.iter_mut() {
        // 🔄 Orbit around the origin
        *transform = Transform::from_translation(
            Quat::from_rotation_y(now).mul_vec3(CAMERA_INITIAL_POSITION),
        )
        .looking_at(Vec3::ZERO, Vec3::Y);
    }
}

// 🎮 Input Handling: The Interactive Experience
fn handle_input(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, With<Camera>>,
    lights: Query<Entity, Or<(With<DirectionalLight>, With<PointLight>)>>,
    mut meshes: Query<(&mut MeshMaterial3d<StandardMaterial>, &MaterialVariants)>,
    mut scenes: Query<(&mut Visibility, &Scene)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_status: ResMut<AppStatus>,
) {
    // 🔦 Space: Cycle through lighting modes
    if keyboard.just_pressed(KeyCode::Space) {
        match app_status.light_mode {
            LightMode::Directional => {
                // Switch to point light
                app_status.light_mode = LightMode::Point;
                for light in lights.iter() {
                    commands.entity(light).despawn();
                }
                spawn_point_light(&mut commands);
            }

            LightMode::Point => {
                // Switch to environment map
                app_status.light_mode = LightMode::EnvironmentMap;
                for light in lights.iter() {
                    commands.entity(light).despawn();
                }
                for camera in cameras.iter() {
                    add_skybox_and_environment_map(&mut commands, &asset_server, camera);
                }
            }

            LightMode::EnvironmentMap => {
                // Back to directional light
                app_status.light_mode = LightMode::Directional;
                for camera in cameras.iter() {
                    commands
                        .entity(camera)
                        .remove::<Skybox>()
                        .remove::<EnvironmentMapLight>();
                }
                spawn_directional_light(&mut commands);
            }
        }
    }

    // 🔄 Enter: Toggle anisotropy
    if keyboard.just_pressed(KeyCode::Enter) {
        app_status.anisotropy_enabled = !app_status.anisotropy_enabled;

        // 🎭 Swap materials on all meshes
        for (mut material_handle, material_variants) in meshes.iter_mut() {
            material_handle.0 = if app_status.anisotropy_enabled {
                material_variants.anisotropic.clone()
            } else {
                material_variants.isotropic.clone()
            }
        }
    }

    // 🎬 Q: Switch scenes
    if keyboard.just_pressed(KeyCode::KeyQ) {
        app_status.visible_scene = app_status.visible_scene.next();
        for (mut visibility, scene) in scenes.iter_mut() {
            let new_vis = if *scene == app_status.visible_scene {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            *visibility = new_vis;
        }
    }
}

// 📝 Dynamic Help Text
fn update_help_text(mut text_query: Query<&mut Text>, app_status: Res<AppStatus>) {
    for mut text in text_query.iter_mut() {
        *text = app_status.create_help_text();
    }
}

// 🌍 Environment Map Setup
//
// Environment maps provide 360° lighting from high-dynamic-range images.
// This creates the most realistic lighting and reflections for our materials.
fn add_skybox_and_environment_map(
    commands: &mut Commands,
    asset_server: &AssetServer,
    entity: Entity,
) {
    commands
        .entity(entity)
        .insert(Skybox {
            brightness: 5000.0,
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            ..default()
        })
        .insert(EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2500.0,
            ..default()
        });
}

// ☀️ Directional Light: Like the Sun
fn spawn_directional_light(commands: &mut Commands) {
    commands.spawn(DirectionalLight {
        color: WHITE.into(),
        illuminance: 3000.0,
        ..default()
    });
}

// 💡 Point Light: Like a Light Bulb
fn spawn_point_light(commands: &mut Commands) {
    commands.spawn(PointLight {
        color: WHITE.into(),
        intensity: 200000.0,
        ..default()
    });
}

impl AppStatus {
    /// Creates the help text as appropriate for the current app status.
    fn create_help_text(&self) -> Text {
        let material_variant_help_text = if self.anisotropy_enabled {
            "Press Enter to disable anisotropy"
        } else {
            "Press Enter to enable anisotropy"
        };

        let light_help_text = match self.light_mode {
            LightMode::Directional => "Press Space to switch to a point light",
            LightMode::Point => "Press Space to switch to an environment map",
            LightMode::EnvironmentMap => "Press Space to switch to a directional light",
        };

        let mesh_help_text = format!("Press Q to change to {}", self.visible_scene.next());

        format!(
            "{}\n{}\n{}",
            material_variant_help_text, light_help_text, mesh_help_text,
        )
        .into()
    }
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            light_mode: default(),
            anisotropy_enabled: true,
            visible_scene: default(),
        }
    }
}

// 🎓 Deep Dive: Understanding Anisotropy
//
// Anisotropic materials have microscopic grooves or fibers that affect light
// reflection. Unlike isotropic materials (same in all directions), anisotropic
// materials create elongated highlights perpendicular to the surface orientation.
//
// Common Examples:
// - Brushed metal: Linear scratches from manufacturing
// - Hair/fur: Cylindrical fibers all pointing one direction  
// - Vinyl records: Circular grooves
// - Satin/silk fabric: Woven fibers
//
// Technical Implementation:
// 1. Tangent vectors define the "grain" direction at each point
// 2. The anisotropy rotation parameter rotates this grain
// 3. The strength parameter controls how much the effect influences reflection
// 4. Special BRDF (Bidirectional Reflectance Distribution Function) math
//    creates the characteristic stretched highlights
//
// 💡 Performance Note:
// Anisotropy requires additional calculations in the shader, but modern
// GPUs handle it efficiently. The visual impact often justifies the cost!