//! Demonstrates the clearcoat PBR feature.
//!
//! 🌟 The Art of Glossy Finishes: Understanding Clearcoat
//!
//! Have you ever admired the deep, lustrous shine of a new car? That glossy layer
//! isn't the paint itself - it's a transparent coating on top! Clearcoat in 3D
//! graphics simulates this effect: a thin, transparent layer over your material
//! that adds depth and realism to surfaces. Think of it as digital varnish!
//!
//! 🎨 What You'll See:
//! - Car paint sphere: Shiny blue metallic with glossy clearcoat
//! - Glass bubble: Transparent sphere with protective coating
//! - Golf ball: Dimpled surface with scratched varnish
//! - Scratched gold: Metallic gold with worn clearcoat showing scratches
//!
//! 🎮 Controls:
//! - `Space`: Toggle between point light and directional light
//!
//! 🔑 Key Concepts:
//! - Clearcoat Layer: A separate transparent material layer
//! - Multi-layer Materials: Base layer + clearcoat = complex surfaces
//! - Normal Maps: Can be applied to both base and clearcoat layers
//! - Real-world Applications: Car paint, lacquered wood, phone screens
//!
//! Clearcoat is a separate material layer that represents a thin translucent
//! layer over a material. Examples include (from the Filament spec [1]) car paint,
//! soda cans, and lacquered wood.
//!
//! In glTF, clearcoat is supported via the `KHR_materials_clearcoat` [2]
//! extension. This extension is well supported by tools; in particular,
//! Blender's glTF exporter maps the clearcoat feature of its Principled BSDF
//! node to this extension, allowing it to appear in Bevy.
//!
//! This Bevy example is inspired by the corresponding three.js example [3].
//!
//! [1]: https://google.github.io/filament/Filament.html#materialsystem/clearcoatmodel
//!
//! [2]: https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_clearcoat/README.md
//!
//! [3]: https://threejs.org/examples/webgl_materials_physical_clearcoat.html

use std::f32::consts::PI;

use bevy::{
    color::palettes::css::{BLUE, GOLD, WHITE},
    core_pipeline::{tonemapping::Tonemapping::AcesFitted, Skybox},
    image::ImageLoaderSettings,
    math::vec3,
    prelude::*,
    render::view::Hdr,
};

/// The size of each sphere.
const SPHERE_SCALE: f32 = 0.9;

/// The speed at which the spheres rotate, in radians per second.
const SPHERE_ROTATION_SPEED: f32 = 0.8;

/// Which type of light we're using: a point light or a directional light.
#[derive(Clone, Copy, PartialEq, Resource, Default)]
enum LightMode {
    #[default]
    Point,
    Directional,
}

/// Tags the example spheres.
#[derive(Component)]
struct ExampleSphere;

fn main() {
    App::new()
        .init_resource::<LightMode>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light)
        .add_systems(Update, animate_spheres)
        .add_systems(Update, (handle_input, update_help_text).chain())
        .run();
}

// 🏗️ Scene Setup: Creating Our Clearcoat Gallery
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    light_mode: Res<LightMode>,
) {
    // 🌐 Create our sphere mesh once and reuse it
    let sphere = create_sphere_mesh(&mut meshes);

    // 🎨 Spawn our four example spheres
    spawn_car_paint_sphere(&mut commands, &mut materials, &asset_server, &sphere);
    spawn_coated_glass_bubble_sphere(&mut commands, &mut materials, &sphere);
    spawn_golf_ball(&mut commands, &asset_server);
    spawn_scratched_gold_ball(&mut commands, &mut materials, &asset_server, &sphere);

    spawn_light(&mut commands);
    spawn_camera(&mut commands, &asset_server);
    spawn_text(&mut commands, &light_mode);
}

// 🌐 Generate Sphere Mesh with Tangents
//
// Tangents are crucial for normal mapping - they define the "grain" direction
// of the surface, allowing normal maps to create believable surface details
fn create_sphere_mesh(meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    // 📐 We MUST generate tangents for normal maps to work correctly!
    // Without tangents, the GPU won't know how to interpret the normal map
    let mut sphere_mesh = Sphere::new(1.0).mesh().build();
    sphere_mesh
        .generate_tangents()
        .expect("Failed to generate tangents");
    meshes.add(sphere_mesh)
}

// 🚗 Car Paint Sphere: Classic Clearcoat Application
//
// This demonstrates the most common use of clearcoat - automotive paint.
// The blue metallic base is protected by a glossy transparent layer.
fn spawn_car_paint_sphere(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    sphere: &Handle<Mesh>,
) {
    commands
        .spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                // 🌟 Clearcoat Settings
                clearcoat: 1.0,                      // Full strength clearcoat
                clearcoat_perceptual_roughness: 0.1, // Very smooth, glossy finish

                // 🎨 Base Layer: Blue Metallic Paint
                metallic: 0.9,             // Highly metallic
                perceptual_roughness: 0.5, // Somewhat rough base
                base_color: BLUE.into(),

                // 🗺️ Normal map adds orange peel texture (common in car paint)
                normal_map_texture: Some(asset_server.load_with_settings(
                    "textures/BlueNoise-Normal.png",
                    |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
                )),
                ..default()
            })),
            Transform::from_xyz(-1.0, 1.0, 0.0).with_scale(Vec3::splat(SPHERE_SCALE)),
        ))
        .insert(ExampleSphere);
}

// 🫧 Glass Bubble: Clearcoat on Transparent Materials
//
// Shows that clearcoat works with transparency too! Like a soap bubble
// with an extra protective film.
fn spawn_coated_glass_bubble_sphere(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    sphere: &Handle<Mesh>,
) {
    commands
        .spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                // 🌟 Clearcoat on transparent material
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 0.1,

                // 🫧 Base Layer: Semi-transparent glass
                metallic: 0.5,
                perceptual_roughness: 0.1, // Smooth glass
                base_color: Color::srgba(0.9, 0.9, 0.9, 0.3), // 30% opacity
                alpha_mode: AlphaMode::Blend, // Enable transparency
                ..default()
            })),
            Transform::from_xyz(-1.0, -1.0, 0.0).with_scale(Vec3::splat(SPHERE_SCALE)),
        ))
        .insert(ExampleSphere);
}

// ⛳ Golf Ball: Multiple Normal Maps
//
// This showcases a complex scenario: dimples on the ball (base normal map)
// with scratches on the varnish (clearcoat normal map). The glTF file
// uses the KHR_materials_clearcoat extension.
fn spawn_golf_ball(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/GolfBall/GolfBall.glb")),
        ),
        Transform::from_xyz(1.0, 1.0, 0.0).with_scale(Vec3::splat(SPHERE_SCALE)),
        ExampleSphere,
    ));
}

// 🏆 Scratched Gold: Clearcoat Normal Maps
//
// Demonstrates how scratches in the clearcoat layer don't affect the
// underlying material - just like real scratched varnish!
fn spawn_scratched_gold_ball(
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
    sphere: &Handle<Mesh>,
) {
    commands
        .spawn((
            Mesh3d(sphere.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                // 🌟 Scratched clearcoat
                clearcoat: 1.0,
                clearcoat_perceptual_roughness: 0.3, // Rougher due to scratches

                // 🗺️ Clearcoat normal map shows scratch pattern
                clearcoat_normal_texture: Some(asset_server.load_with_settings(
                    "textures/ScratchedGold-Normal.png",
                    |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
                )),

                // 🏆 Base Layer: Smooth gold metal
                metallic: 0.9,
                perceptual_roughness: 0.1, // Very smooth gold
                base_color: GOLD.into(),
                ..default()
            })),
            Transform::from_xyz(1.0, -1.0, 0.0).with_scale(Vec3::splat(SPHERE_SCALE)),
        ))
        .insert(ExampleSphere);
}

// 💡 Dynamic Lighting
fn spawn_light(commands: &mut Commands) {
    commands.spawn(create_point_light());
}

// 📷 Camera with Environment
fn spawn_camera(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn((
            Camera3d::default(),
            Hdr, // High Dynamic Range for realistic lighting
            Projection::Perspective(PerspectiveProjection {
                fov: 27.0 / 180.0 * PI, // Narrower FOV for less distortion
                ..default()
            }),
            Transform::from_xyz(0.0, 0.0, 10.0),
            AcesFitted, // Filmic tone mapping
        ))
        // 🌍 Skybox for reflections
        .insert(Skybox {
            brightness: 5000.0,
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            ..default()
        })
        // 🌞 Environment lighting
        .insert(EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2000.0,
            ..default()
        });
}

// 📝 UI Text
fn spawn_text(commands: &mut Commands, light_mode: &LightMode) {
    commands.spawn((
        light_mode.create_help_text(),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// 🎬 Light Animation System
//
// Moves the light in a complex 3D pattern to showcase how clearcoat
// responds to changing light angles
fn animate_light(
    mut lights: Query<&mut Transform, Or<(With<PointLight>, With<DirectionalLight>)>>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs();
    for mut transform in lights.iter_mut() {
        // 🌀 Complex orbital motion
        transform.translation = vec3(
            ops::sin(now * 1.4),
            ops::cos(now * 1.0),
            ops::cos(now * 0.6),
        ) * vec3(3.0, 4.0, 3.0);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

// 🔄 Sphere Rotation System
//
// Slowly rotates spheres to show how clearcoat interacts with light
// from different angles
fn animate_spheres(mut spheres: Query<&mut Transform, With<ExampleSphere>>, time: Res<Time>) {
    let now = time.elapsed_secs();
    for mut transform in spheres.iter_mut() {
        transform.rotation = Quat::from_rotation_y(SPHERE_ROTATION_SPEED * now);
    }
}

// 🎮 Input Handler: Toggle Light Type
fn handle_input(
    mut commands: Commands,
    mut light_query: Query<Entity, Or<(With<PointLight>, With<DirectionalLight>)>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut light_mode: ResMut<LightMode>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    for light in light_query.iter_mut() {
        match *light_mode {
            LightMode::Point => {
                *light_mode = LightMode::Directional;
                commands
                    .entity(light)
                    .remove::<PointLight>()
                    .insert(create_directional_light());
            }
            LightMode::Directional => {
                *light_mode = LightMode::Point;
                commands
                    .entity(light)
                    .remove::<DirectionalLight>()
                    .insert(create_point_light());
            }
        }
    }
}

// 📝 Update Help Text
fn update_help_text(mut text_query: Query<&mut Text>, light_mode: Res<LightMode>) {
    for mut text in text_query.iter_mut() {
        *text = light_mode.create_help_text();
    }
}

// 💡 Point Light Configuration
fn create_point_light() -> PointLight {
    PointLight {
        color: WHITE.into(),
        intensity: 100000.0,
        ..default()
    }
}

// ☀️ Directional Light Configuration
fn create_directional_light() -> DirectionalLight {
    DirectionalLight {
        color: WHITE.into(),
        illuminance: 1000.0,
        ..default()
    }
}

impl LightMode {
    fn create_help_text(&self) -> Text {
        let help_text = match *self {
            LightMode::Point => "Press Space to switch to a directional light",
            LightMode::Directional => "Press Space to switch to a point light",
        };

        Text::new(help_text)
    }
}

// 🎓 Deep Dive: The Physics of Clearcoat
//
// Clearcoat simulates a thin dielectric (non-metallic) layer over a material.
// In the real world, this layer:
//
// 1. **Adds Fresnel Reflections**: More reflective at grazing angles
// 2. **Has Its Own Roughness**: Can be smooth even if base is rough
// 3. **Can Have Normal Maps**: Scratches that don't affect the base
// 4. **Is Always Dielectric**: Even over metals (like car paint)
//
// The rendering equation treats it as two separate BRDF layers:
// - Base layer: Your standard PBR material
// - Clearcoat layer: Always dielectric, typically IOR ~1.5
//
// Light interacts with both layers:
// 1. Some light reflects off the clearcoat
// 2. Remaining light passes through to the base layer
// 3. Base layer reflection comes back through clearcoat

// 💡 Practical Applications:
//
// **Automotive**: Car paint, headlights, trim pieces
// **Consumer Products**: Phones, laptops, glossy plastics
// **Architecture**: Lacquered wood, polished floors, varnished surfaces
// **Food/Beverage**: Candy coating, wet surfaces, packaging
//
// Performance Note: Clearcoat adds a second specular lobe calculation,
// roughly 20-30% more expensive than standard materials.

