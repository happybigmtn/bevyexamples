//! This example shows how irradiance volumes affect the indirect lighting of
//! objects in a scene.
//!
//! The controls are as follows:
//!
//! * Space toggles the irradiance volume on and off.
//!
//! * Enter toggles the camera rotation on and off.
//!
//! * Tab switches the object between a plain sphere and a running fox.
//!
//! * Backspace shows and hides the voxel cubes.
//!
//! * Clicking anywhere moves the object.
//!
//! # What Are Irradiance Volumes?
//!
//! Irradiance volumes are a technique for adding indirect (bounced) lighting to a scene.
//! Imagine sunlight entering a room through a window - it doesn't just light up where
//! it directly hits, but also bounces off walls and objects to softly illuminate the
//! entire room. That's indirect lighting!
//!
//! Traditional real-time rendering only calculates direct lighting (light straight from
//! the source). Irradiance volumes pre-calculate how light bounces around a scene and
//! store this information in a 3D texture (voxel grid).
//!
//! # How It Works
//!
//! 1. **Baking**: The indirect lighting is pre-calculated offline and stored in a 3D texture
//! 2. **Voxels**: Each voxel (3D pixel) stores the incoming light from all directions
//! 3. **Runtime**: Objects sample from nearby voxels to determine their indirect lighting
//!
//! This technique is particularly useful for:
//! - Indoor scenes with complex light bounces
//! - Stylized games where you want specific ambient lighting
//! - Mobile/VR where real-time global illumination is too expensive

use bevy::{
    // CSS color constants (RED, YELLOW, SILVER, etc.)
    color::palettes::css::*,
    // Skybox component for environment lighting
    core_pipeline::Skybox,
    // Convenience functions for creating vectors
    math::{uvec3, vec3},
    pbr::{
        // The main component that defines an irradiance volume
        irradiance_volume::IrradianceVolume,
        // For creating custom materials that extend the standard material
        ExtendedMaterial, MaterialExtension,
        // Prevents an object from casting shadows
        NotShadowCaster,
    },
    prelude::*,
    render::render_resource::{
        AsBindGroup,  // Derive macro for GPU binding
        ShaderRef,    // Reference to a shader file
        ShaderType,   // For types that can be sent to shaders
    },
    // To identify the primary window for mouse input
    window::PrimaryWindow,
};

/// This example uses a shader source file from the assets subdirectory
/// The shader visualizes individual voxels of the irradiance volume
const SHADER_ASSET_PATH: &str = "shaders/irradiance_volume_voxel_visualization.wgsl";

// Rotation speed in radians per frame.
// At 60 FPS, this is about 12 radians/second or ~2 rotations per second
const ROTATION_SPEED: f32 = 0.2;

// The fox model is quite large, so we scale it down
const FOX_SCALE: f32 = 0.05;
// The sphere is created at unit size, so we scale it up
const SPHERE_SCALE: f32 = 2.0;

// Intensity multiplier for the irradiance volume
// Higher values make the indirect lighting brighter
const IRRADIANCE_VOLUME_INTENSITY: f32 = 1800.0;

// When irradiance volume is disabled, we use dim ambient light instead
// This is 6% of the irradiance volume intensity
const AMBIENT_LIGHT_BRIGHTNESS: f32 = 0.06;

// Size of the visualization cubes for each voxel
const VOXEL_CUBE_SCALE: f32 = 0.4;

static DISABLE_IRRADIANCE_VOLUME_HELP_TEXT: &str = "Space: Disable the irradiance volume";
static ENABLE_IRRADIANCE_VOLUME_HELP_TEXT: &str = "Space: Enable the irradiance volume";

static HIDE_VOXELS_HELP_TEXT: &str = "Backspace: Hide the voxels";
static SHOW_VOXELS_HELP_TEXT: &str = "Backspace: Show the voxels";

static STOP_ROTATION_HELP_TEXT: &str = "Enter: Stop rotation";
static START_ROTATION_HELP_TEXT: &str = "Enter: Start rotation";

static SWITCH_TO_FOX_HELP_TEXT: &str = "Tab: Switch to a skinned mesh";
static SWITCH_TO_SPHERE_HELP_TEXT: &str = "Tab: Switch to a plain sphere mesh";

static CLICK_TO_MOVE_HELP_TEXT: &str = "Left click: Move the object";

static GIZMO_COLOR: Color = Color::Srgba(YELLOW);

// This matrix transforms from world space to voxel space
// It defines the position, orientation, and scale of the irradiance volume
// in the world. The seemingly arbitrary numbers come from the tool that
// generated the irradiance volume data.
static VOXEL_FROM_WORLD: Mat4 = Mat4::from_cols_array_2d(&[
    [-42.317566, 0.0, 0.0, 0.0],  // X axis (note negative - flips X)
    [0.0, 0.0, 44.601563, 0.0],   // Y axis (maps to Z in voxel space)
    [0.0, 16.73776, 0.0, 0.0],    // Z axis (maps to Y in voxel space)
    [0.0, 6.544792, 0.0, 1.0],    // Translation and W
]);

// The mode the application is in.
// Resources in Bevy are global data accessible from any system
#[derive(Resource)]
struct AppStatus {
    // Whether the user wants the irradiance volume to be applied.
    irradiance_volume_present: bool,
    // Whether the user wants the unskinned sphere mesh or the skinned fox mesh.
    model: ExampleModel,
    // Whether the user has requested the scene to rotate.
    rotating: bool,
    // Whether the user has requested the voxels to be displayed.
    voxels_visible: bool,
}

// Which model the user wants to display.
// Skinned meshes have bones and animations, while static meshes don't
#[derive(Clone, Copy, PartialEq)]
enum ExampleModel {
    // The plain sphere - a simple static mesh
    Sphere,
    // The fox - a skinned mesh with bone animations
    Fox,
}

// Handles to all the assets used in this example.
// In Bevy, Handle<T> is a reference to an asset of type T.
// Assets are loaded asynchronously and accessed through these handles.
#[derive(Resource)]
struct ExampleAssets {
    // The glTF scene containing the colored floor.
    main_scene: Handle<Scene>,

    // The 3D texture containing the irradiance volume.
    // This stores pre-calculated indirect lighting data
    irradiance_volume: Handle<Image>,

    // The plain sphere mesh.
    main_sphere: Handle<Mesh>,

    // The material used for the sphere.
    main_sphere_material: Handle<StandardMaterial>,

    // The glTF scene containing the animated fox.
    fox: Handle<Scene>,

    // The graph containing the animation that the fox will play.
    // Animation graphs allow complex animation blending and state machines
    fox_animation_graph: Handle<AnimationGraph>,

    // The node within the animation graph containing the animation.
    fox_animation_node: AnimationNodeIndex,

    // The voxel cube mesh (for visualization)
    voxel_cube: Handle<Mesh>,

    // The skybox texture for the background
    skybox: Handle<Image>,
}

// The sphere and fox both have this component.
// This marker lets us query for the main object regardless of which model is shown
#[derive(Component)]
struct MainObject;

// Marks each of the voxel cubes.
// Used to identify the small cubes that visualize the irradiance volume grid
#[derive(Component)]
struct VoxelCube;

// Marks the voxel cube parent object.
// All voxel cubes are children of this entity for easy show/hide
#[derive(Component)]
struct VoxelCubeParent;

// Type alias for our custom material that extends StandardMaterial
type VoxelVisualizationMaterial = ExtendedMaterial<StandardMaterial, VoxelVisualizationExtension>;

// Custom material extension for visualizing voxels
// The derives:
// - Asset: Can be stored in Assets<T>
// - TypePath: Required for asset type identification  
// - AsBindGroup: Generates GPU binding code
// - Debug, Clone: Standard traits
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct VoxelVisualizationExtension {
    // This will be bound to binding 100 in the shader
    #[uniform(100)]
    irradiance_volume_info: VoxelVisualizationIrradianceVolumeInfo,
}

// Information about the irradiance volume sent to the shader
#[derive(ShaderType, Debug, Clone)]
struct VoxelVisualizationIrradianceVolumeInfo {
    // Transform from voxel space to world space
    world_from_voxel: Mat4,
    // Transform from world space to voxel space
    voxel_from_world: Mat4,
    // Resolution of the 3D texture (width, height, depth)
    resolution: UVec3,
    // Intensity multiplier
    intensity: f32,
}

fn main() {
    // Create the example app.
    App::new()
        // Configure window with custom title
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Irradiance Volumes Example".into(),
                ..default()
            }),
            ..default()
        }))
        // Register our custom material for voxel visualization
        .add_plugins(MaterialPlugin::<VoxelVisualizationMaterial>::default())
        // Initialize resources with default values
        .init_resource::<AppStatus>()
        // Initialize assets using FromWorld trait
        .init_resource::<ExampleAssets>()
        // Start with no ambient light - we'll use irradiance volume instead
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.0,
            ..default()
        })
        // Setup system runs once at startup
        .add_systems(Startup, setup)
        // Create voxel cubes after assets load (PreUpdate runs before Update)
        .add_systems(PreUpdate, create_cubes)
        // Camera rotation
        .add_systems(Update, rotate_camera)
        // Animation playback
        .add_systems(Update, play_animations)
        // Input handling systems - these run after rotation/animation
        // to ensure consistent behavior
        .add_systems(
            Update,
            handle_mouse_clicks
                .after(rotate_camera)
                .after(play_animations),
        )
        .add_systems(
            Update,
            change_main_object
                .after(rotate_camera)
                .after(play_animations),
        )
        .add_systems(
            Update,
            toggle_irradiance_volumes
                .after(rotate_camera)
                .after(play_animations),
        )
        .add_systems(
            Update,
            toggle_voxel_visibility
                .after(rotate_camera)
                .after(play_animations),
        )
        .add_systems(
            Update,
            toggle_rotation.after(rotate_camera).after(play_animations),
        )
        // UI systems run last to reflect all state changes
        .add_systems(
            Update,
            draw_gizmo
                .after(handle_mouse_clicks)
                .after(change_main_object)
                .after(toggle_irradiance_volumes)
                .after(toggle_voxel_visibility)
                .after(toggle_rotation),
        )
        .add_systems(
            Update,
            update_text
                .after(handle_mouse_clicks)
                .after(change_main_object)
                .after(toggle_irradiance_volumes)
                .after(toggle_voxel_visibility)
                .after(toggle_rotation),
        )
        .run();
}

// Spawns all the scene objects.
// Breaking setup into smaller functions makes the code more maintainable
fn setup(mut commands: Commands, assets: Res<ExampleAssets>, app_status: Res<AppStatus>) {
    spawn_main_scene(&mut commands, &assets);        // The floor/environment
    spawn_camera(&mut commands, &assets);            // Camera with skybox
    spawn_irradiance_volume(&mut commands, &assets); // The indirect lighting data
    spawn_light(&mut commands);                      // Direct light source
    spawn_sphere(&mut commands, &assets);            // Default object (sphere)
    spawn_voxel_cube_parent(&mut commands);          // Parent for voxel visualization
    spawn_fox(&mut commands, &assets);               // Alternative object (animated fox)
    spawn_text(&mut commands, &app_status);          // UI help text
}

fn spawn_main_scene(commands: &mut Commands, assets: &ExampleAssets) {
    commands.spawn(SceneRoot(assets.main_scene.clone()));
}

fn spawn_camera(commands: &mut Commands, assets: &ExampleAssets) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-10.012, 4.8605, 13.281).looking_at(Vec3::ZERO, Vec3::Y),
        Skybox {
            image: assets.skybox.clone(),
            brightness: 150.0,
            ..default()
        },
    ));
}

fn spawn_irradiance_volume(commands: &mut Commands, assets: &ExampleAssets) {
    commands.spawn((
        // The transform defines where and how big the irradiance volume is
        // We use the pre-defined matrix that matches how the volume was baked
        Transform::from_matrix(VOXEL_FROM_WORLD),
        IrradianceVolume {
            // The 3D texture containing pre-baked lighting data
            voxels: assets.irradiance_volume.clone(),
            // Brightness multiplier for the indirect lighting
            intensity: IRRADIANCE_VOLUME_INTENSITY,
            ..default()
        },
        // LightProbe marks this as a source of indirect lighting
        LightProbe,
    ));
}

fn spawn_light(commands: &mut Commands) {
    commands.spawn((
        PointLight {
            // Bright point light (250,000 lumens)
            // For reference: a 100W light bulb is ~1,600 lumens
            intensity: 250000.0,
            // Enable shadow mapping for more realistic lighting
            shadows_enabled: true,
            ..default()
        },
        // Position the light above and to the side of the scene
        Transform::from_xyz(4.0762, 5.9039, 1.0055),
    ));
}

fn spawn_sphere(commands: &mut Commands, assets: &ExampleAssets) {
    commands
        .spawn((
            Mesh3d(assets.main_sphere.clone()),
            MeshMaterial3d(assets.main_sphere_material.clone()),
            Transform::from_xyz(0.0, SPHERE_SCALE, 0.0).with_scale(Vec3::splat(SPHERE_SCALE)),
        ))
        .insert(MainObject);
}

fn spawn_voxel_cube_parent(commands: &mut Commands) {
    commands.spawn((Visibility::Hidden, Transform::default(), VoxelCubeParent));
}

fn spawn_fox(commands: &mut Commands, assets: &ExampleAssets) {
    commands.spawn((
        SceneRoot(assets.fox.clone()),
        Visibility::Hidden,
        Transform::from_scale(Vec3::splat(FOX_SCALE)),
        MainObject,
    ));
}

fn spawn_text(commands: &mut Commands, app_status: &AppStatus) {
    commands.spawn((
        app_status.create_text(),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// A system that updates the help text.
fn update_text(mut text_query: Query<&mut Text>, app_status: Res<AppStatus>) {
    for mut text in text_query.iter_mut() {
        *text = app_status.create_text();
    }
}

impl AppStatus {
    // Constructs the help text at the bottom of the screen based on the
    // application status.
    fn create_text(&self) -> Text {
        let irradiance_volume_help_text = if self.irradiance_volume_present {
            DISABLE_IRRADIANCE_VOLUME_HELP_TEXT
        } else {
            ENABLE_IRRADIANCE_VOLUME_HELP_TEXT
        };

        let voxels_help_text = if self.voxels_visible {
            HIDE_VOXELS_HELP_TEXT
        } else {
            SHOW_VOXELS_HELP_TEXT
        };

        let rotation_help_text = if self.rotating {
            STOP_ROTATION_HELP_TEXT
        } else {
            START_ROTATION_HELP_TEXT
        };

        let switch_mesh_help_text = match self.model {
            ExampleModel::Sphere => SWITCH_TO_FOX_HELP_TEXT,
            ExampleModel::Fox => SWITCH_TO_SPHERE_HELP_TEXT,
        };

        format!(
            "{CLICK_TO_MOVE_HELP_TEXT}\n\
            {voxels_help_text}\n\
            {irradiance_volume_help_text}\n\
            {rotation_help_text}\n\
            {switch_mesh_help_text}"
        )
        .into()
    }
}

// Rotates the camera a bit every frame.
fn rotate_camera(
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    app_status: Res<AppStatus>,
) {
    if !app_status.rotating {
        return;
    }

    for mut transform in camera_query.iter_mut() {
        transform.translation = Vec2::from_angle(ROTATION_SPEED * time.delta_secs())
            .rotate(transform.translation.xz())
            .extend(transform.translation.y)
            .xzy();
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

// Toggles between the unskinned sphere model and the skinned fox model if the
// user requests it.
fn change_main_object(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_status: ResMut<AppStatus>,
    mut sphere_query: Query<&mut Visibility, (With<MainObject>, With<Mesh3d>, Without<SceneRoot>)>,
    mut fox_query: Query<&mut Visibility, (With<MainObject>, With<SceneRoot>)>,
) {
    if !keyboard.just_pressed(KeyCode::Tab) {
        return;
    }
    let Some(mut sphere_visibility) = sphere_query.iter_mut().next() else {
        return;
    };
    let Some(mut fox_visibility) = fox_query.iter_mut().next() else {
        return;
    };

    match app_status.model {
        ExampleModel::Sphere => {
            *sphere_visibility = Visibility::Hidden;
            *fox_visibility = Visibility::Visible;
            app_status.model = ExampleModel::Fox;
        }
        ExampleModel::Fox => {
            *sphere_visibility = Visibility::Visible;
            *fox_visibility = Visibility::Hidden;
            app_status.model = ExampleModel::Sphere;
        }
    }
}

impl Default for AppStatus {
    fn default() -> Self {
        Self {
            // Start with irradiance volume enabled to show the effect
            irradiance_volume_present: true,
            // Camera rotation on by default for dynamic view
            rotating: true,
            // Start with sphere (simpler to see lighting effects)
            model: ExampleModel::Sphere,
            // Voxel visualization off by default (can be distracting)
            voxels_visible: false,
        }
    }
}

// Turns on and off the irradiance volume as requested by the user.
// This lets you compare the scene with and without indirect lighting
fn toggle_irradiance_volumes(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    light_probe_query: Query<Entity, With<LightProbe>>,
    mut app_status: ResMut<AppStatus>,
    assets: Res<ExampleAssets>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    };

    let Some(light_probe) = light_probe_query.iter().next() else {
        return;
    };

    if app_status.irradiance_volume_present {
        // Remove irradiance volume component to disable it
        commands.entity(light_probe).remove::<IrradianceVolume>();
        // Add dim ambient light as a simple substitute
        // This helps see the model even without indirect lighting
        ambient_light.brightness = AMBIENT_LIGHT_BRIGHTNESS * IRRADIANCE_VOLUME_INTENSITY;
        app_status.irradiance_volume_present = false;
    } else {
        // Re-add the irradiance volume component
        commands.entity(light_probe).insert(IrradianceVolume {
            voxels: assets.irradiance_volume.clone(),
            intensity: IRRADIANCE_VOLUME_INTENSITY,
            ..default()
        });
        // Turn off ambient light - let irradiance volume handle it
        ambient_light.brightness = 0.0;
        app_status.irradiance_volume_present = true;
    }
}

fn toggle_rotation(keyboard: Res<ButtonInput<KeyCode>>, mut app_status: ResMut<AppStatus>) {
    if keyboard.just_pressed(KeyCode::Enter) {
        app_status.rotating = !app_status.rotating;
    }
}

// Handles clicks on the plane that reposition the object.
// This demonstrates ray-casting from screen space to world space
fn handle_mouse_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut main_objects: Query<&mut Transform, With<MainObject>>,
) {
    if !buttons.pressed(MouseButton::Left) {
        return;
    }
    // Get mouse position in window coordinates
    let Some(mouse_position) = windows.iter().next().and_then(Window::cursor_position) else {
        return;
    };
    let Some((camera, camera_transform)) = cameras.iter().next() else {
        return;
    };

    // Figure out where the user clicked on the plane.
    // viewport_to_world converts 2D screen coordinates to a 3D ray
    let Ok(ray) = camera.viewport_to_world(camera_transform, mouse_position) else {
        return;
    };
    // Find where the ray intersects the ground plane (Y=0)
    let Some(ray_distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) else {
        return;
    };
    // Calculate the actual intersection point
    let plane_intersection = ray.origin + ray.direction.normalize() * ray_distance;

    // Move all the main objects to the clicked position
    // We keep the Y coordinate unchanged to keep objects on the ground
    for mut transform in main_objects.iter_mut() {
        transform.translation = vec3(
            plane_intersection.x,
            transform.translation.y,
            plane_intersection.z,
        );
    }
}

// FromWorld trait allows initializing resources with access to the World
// This is useful when you need to load assets during initialization
impl FromWorld for ExampleAssets {
    fn from_world(world: &mut World) -> Self {
        // Load the fox's run animation (index 1 in the GLTF file)
        let fox_animation =
            world.load_asset(GltfAssetLabel::Animation(1).from_asset("models/animated/Fox.glb"));
        // Create an animation graph with a single clip
        let (fox_animation_graph, fox_animation_node) =
            AnimationGraph::from_clip(fox_animation.clone());

        ExampleAssets {
            // Create a sphere mesh with UV coordinates (32 segments, 18 stacks)
            main_sphere: world.add_asset(Sphere::default().mesh().uv(32, 18)),
            // Load the fox model (scene 0 in the GLTF file)
            fox: world.load_asset(GltfAssetLabel::Scene(0).from_asset("models/animated/Fox.glb")),
            // Silver-colored material for the sphere
            main_sphere_material: world.add_asset(Color::from(SILVER)),
            // Load the example scene with colored floor
            main_scene: world.load_asset(
                GltfAssetLabel::Scene(0)
                    .from_asset("models/IrradianceVolumeExample/IrradianceVolumeExample.glb"),
            ),
            // Load the pre-baked irradiance volume data
            // .vxgi.ktx2 is a compressed 3D texture format
            irradiance_volume: world.load_asset("irradiance_volumes/Example.vxgi.ktx2"),
            fox_animation_graph: world.add_asset(fox_animation_graph),
            fox_animation_node,
            // Simple cube for visualizing voxels
            voxel_cube: world.add_asset(Cuboid::default()),
            // Just use a specular map for the skybox since it's not too blurry.
            // In reality you wouldn't do this--you'd use a real skybox texture--but
            // reusing the textures like this saves space in the Bevy repository.
            skybox: world.load_asset("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
        }
    }
}

// Plays the animation on the fox.
fn play_animations(
    mut commands: Commands,
    assets: Res<ExampleAssets>,
    mut players: Query<(Entity, &mut AnimationPlayer), Without<AnimationGraphHandle>>,
) {
    for (entity, mut player) in players.iter_mut() {
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(assets.fox_animation_graph.clone()));
        player.play(assets.fox_animation_node).repeat();
    }
}

// Creates visualization cubes for each voxel in the irradiance volume
// This helps visualize the 3D grid structure of the lighting data
fn create_cubes(
    image_assets: Res<Assets<Image>>,
    mut commands: Commands,
    irradiance_volumes: Query<(&IrradianceVolume, &GlobalTransform)>,
    voxel_cube_parents: Query<Entity, With<VoxelCubeParent>>,
    voxel_cubes: Query<Entity, With<VoxelCube>>,
    example_assets: Res<ExampleAssets>,
    mut voxel_visualization_material_assets: ResMut<Assets<VoxelVisualizationMaterial>>,
) {
    // If voxel cubes have already been spawned, don't do anything.
    if !voxel_cubes.is_empty() {
        return;
    }

    let Some(voxel_cube_parent) = voxel_cube_parents.iter().next() else {
        return;
    };

    for (irradiance_volume, global_transform) in irradiance_volumes.iter() {
        // Get the actual image data to read its dimensions
        let Some(image) = image_assets.get(&irradiance_volume.voxels) else {
            continue;
        };

        let resolution = image.texture_descriptor.size;

        // Create a material that will color each voxel based on the lighting data
        let voxel_cube_material = voxel_visualization_material_assets.add(ExtendedMaterial {
            base: StandardMaterial::from(Color::from(RED)),
            extension: VoxelVisualizationExtension {
                irradiance_volume_info: VoxelVisualizationIrradianceVolumeInfo {
                    world_from_voxel: VOXEL_FROM_WORLD.inverse(),
                    voxel_from_world: VOXEL_FROM_WORLD,
                    resolution: uvec3(
                        resolution.width,
                        resolution.height,
                        resolution.depth_or_array_layers,
                    ),
                    intensity: IRRADIANCE_VOLUME_INTENSITY,
                },
            },
        });

        // Calculate the size of each voxel in normalized coordinates
        let scale = vec3(
            1.0 / resolution.width as f32,
            1.0 / resolution.height as f32,
            1.0 / resolution.depth_or_array_layers as f32,
        );

        // Spawn a cube for each voxel in the 3D grid
        for z in 0..resolution.depth_or_array_layers {
            for y in 0..resolution.height {
                for x in 0..resolution.width {
                    // Convert voxel indices to normalized coordinates (-0.5 to 0.5)
                    // Adding 0.5 centers each voxel in its grid cell
                    let uvw = (uvec3(x, y, z).as_vec3() + 0.5) * scale - 0.5;
                    // Transform from volume space to world space
                    let pos = global_transform.transform_point(uvw);
                    
                    let voxel_cube = commands
                        .spawn((
                            Mesh3d(example_assets.voxel_cube.clone()),
                            MeshMaterial3d(voxel_cube_material.clone()),
                            Transform::from_scale(Vec3::splat(VOXEL_CUBE_SCALE))
                                .with_translation(pos),
                        ))
                        .insert(VoxelCube)
                        // Don't cast shadows - these are just visualization
                        .insert(NotShadowCaster)
                        .id();

                    // Make it a child of the parent for easy show/hide
                    commands.entity(voxel_cube_parent).add_child(voxel_cube);
                }
            }
        }
    }
}

// Draws a gizmo showing the bounds of the irradiance volume.
// Gizmos are debug drawings that appear on top of the scene
fn draw_gizmo(
    mut gizmos: Gizmos,
    irradiance_volume_query: Query<&GlobalTransform, With<IrradianceVolume>>,
    app_status: Res<AppStatus>,
) {
    // Only draw when voxels are visible
    if app_status.voxels_visible {
        for transform in irradiance_volume_query.iter() {
            // Draw a wireframe box showing the volume bounds
            gizmos.cuboid(*transform, GIZMO_COLOR);
        }
    }
}

// Handles a request from the user to toggle the voxel visibility on and off.
fn toggle_voxel_visibility(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut app_status: ResMut<AppStatus>,
    mut voxel_cube_parent_query: Query<&mut Visibility, With<VoxelCubeParent>>,
) {
    if !keyboard.just_pressed(KeyCode::Backspace) {
        return;
    }

    app_status.voxels_visible = !app_status.voxels_visible;

    for mut visibility in voxel_cube_parent_query.iter_mut() {
        *visibility = if app_status.voxels_visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// Define how our material extension modifies the standard material
impl MaterialExtension for VoxelVisualizationExtension {
    fn fragment_shader() -> ShaderRef {
        // Use our custom fragment shader for voxel visualization
        // The shader will color each voxel based on the irradiance data
        SHADER_ASSET_PATH.into()
    }
}
