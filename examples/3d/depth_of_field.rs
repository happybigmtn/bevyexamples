//! Demonstrates depth of field (DOF).
//!
//! 📸 The Art of Digital Focus: Understanding Depth of Field
//!
//! Have you ever taken a portrait where the person is sharp but the background
//! is beautifully blurred? That's depth of field! It's what separates professional
//! photos from snapshots, and what makes movie scenes feel cinematic. In real cameras,
//! it happens because lenses can only perfectly focus light from one distance at a time.
//! Everything closer or farther gets progressively blurrier. Now we're bringing that
//! magic to our 3D world!
//!
//! 🎯 What You'll See:
//! - A circuit board test scene with objects at various distances
//! - Real-time focus adjustment like a camera lens
//! - Two different blur techniques: Gaussian (fast) and Bokeh (beautiful)
//! - Physical camera parameters that match real photography
//!
//! 🎮 Controls:
//! - `Up/Down Arrows`: Adjust focal distance (what's in focus)
//! - `Left/Right Arrows`: Adjust aperture f-stop (blur intensity)
//! - `Space`: Cycle through modes (Off → Bokeh → Gaussian)
//!
//! 🔑 Key Concepts:
//! - Focal Distance: The exact distance where objects are perfectly sharp
//! - Aperture/F-Stop: How wide the lens opening is (affects blur amount)
//! - Circle of Confusion: How blurred a point becomes when out of focus
//! - Bokeh: The aesthetic quality of the blur (those beautiful light circles!)
//!
//! The test scene is inspired by [a blog post on depth of field in Unity].
//! However, the technique used in Bevy has little to do with that blog post,
//! and all the assets are original.
//!
//! [a blog post on depth of field in Unity]: https://catlikecoding.com/unity/tutorials/advanced-rendering/depth-of-field/

use bevy::{
    core_pipeline::{
        bloom::Bloom,
        dof::{self, DepthOfField, DepthOfFieldMode},
        tonemapping::Tonemapping,
    },
    gltf::GltfMeshName,
    pbr::Lightmap,
    prelude::*,
    render::camera::PhysicalCameraParameters,
};

// 🎚️ Control Constants: Fine-Tuning Your Virtual Camera
/// The increments in which the user can adjust the focal distance, in meters
/// per frame.
const FOCAL_DISTANCE_SPEED: f32 = 0.05;
/// The increments in which the user can adjust the f-number, in units per frame.
const APERTURE_F_STOP_SPEED: f32 = 0.01;

/// The minimum distance that we allow the user to focus on.
const MIN_FOCAL_DISTANCE: f32 = 0.01;
/// The minimum f-number that we allow the user to set.
const MIN_APERTURE_F_STOPS: f32 = 0.05;

// 📷 Virtual Camera Settings
/// A resource that stores the settings that the user can change.
#[derive(Clone, Copy, Resource)]
struct AppSettings {
    /// The distance from the camera to the area in the most focus.
    /// Think of this as turning the focus ring on a camera lens!
    focal_distance: f32,

    /// The [f-number]. Lower numbers cause objects outside the focal distance
    /// to be blurred more.
    /// 
    /// In photography, f-stop controls the aperture (lens opening) size:
    /// - f/1.4 = Wide open = Lots of blur (shallow depth of field)
    /// - f/8 = Medium = Moderate blur
    /// - f/22 = Tiny opening = Everything sharp (deep depth of field)
    ///
    /// [f-number]: https://en.wikipedia.org/wiki/F-number
    aperture_f_stops: f32,

    /// Whether depth of field is on, and, if so, whether we're in Gaussian or
    /// bokeh mode.
    /// - None: No DOF effect (everything sharp)
    /// - Gaussian: Fast approximate blur
    /// - Bokeh: Realistic lens blur with circular highlights
    mode: Option<DepthOfFieldMode>,
}

fn main() {
    App::new()
        .init_resource::<AppSettings>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Depth of Field Example".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, tweak_scene)
        .add_systems(
            Update,
            // 🔄 Chain these systems for proper update order
            (adjust_focus, change_mode, update_dof_settings, update_text).chain(),
        )
        .run();
}

// 🏗️ Scene Setup: Creating Our Photography Studio
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, app_settings: Res<AppSettings>) {
    // 📸 Camera Configuration
    // We need HDR and bloom to make the DOF effect really shine!
    let mut camera = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 4.5, 8.25).looking_at(Vec3::ZERO, Vec3::Y),
        Tonemapping::TonyMcMapface,  // Great for high contrast scenes
        Bloom::NATURAL,               // Adds that dreamy glow to out-of-focus lights
    ));

    // 🎯 Apply initial depth of field settings
    if let Some(depth_of_field) = Option::<DepthOfField>::from(*app_settings) {
        camera.insert(depth_of_field);
    }

    // 🎬 Load the test scene - a circuit board with various objects
    // Perfect for testing DOF because objects are at different distances!
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/DepthOfFieldExample/DepthOfFieldExample.glb"),
    )));

    // 📝 UI Help Text
    commands.spawn((
        create_text(&app_settings),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// 🎯 Focus Control System: Like Turning the Lens Ring!
/// Adjusts the focal distance and f-number per user inputs.
fn adjust_focus(input: Res<ButtonInput<KeyCode>>, mut app_settings: ResMut<AppSettings>) {
    // 🔄 Focal Distance Control (Up/Down)
    // This is like turning the focus ring on a camera lens
    let distance_delta = if input.pressed(KeyCode::ArrowDown) {
        -FOCAL_DISTANCE_SPEED  // Focus closer
    } else if input.pressed(KeyCode::ArrowUp) {
        FOCAL_DISTANCE_SPEED   // Focus farther
    } else {
        0.0
    };

    // 🔀 Aperture Control (Left/Right)
    // This is like changing the f-stop on a camera
    // Remember: SMALLER f-number = WIDER aperture = MORE blur!
    let f_stop_delta = if input.pressed(KeyCode::ArrowLeft) {
        -APERTURE_F_STOP_SPEED  // Open aperture (more blur)
    } else if input.pressed(KeyCode::ArrowRight) {
        APERTURE_F_STOP_SPEED   // Close aperture (less blur)
    } else {
        0.0
    };

    // 📏 Apply changes with safety limits
    app_settings.focal_distance =
        (app_settings.focal_distance + distance_delta).max(MIN_FOCAL_DISTANCE);
    app_settings.aperture_f_stops =
        (app_settings.aperture_f_stops + f_stop_delta).max(MIN_APERTURE_F_STOPS);
}

// 🎨 Blur Mode Selector: Choose Your Style!
/// Changes the depth of field mode (Gaussian, bokeh, off) per user inputs.
fn change_mode(input: Res<ButtonInput<KeyCode>>, mut app_settings: ResMut<AppSettings>) {
    if !input.just_pressed(KeyCode::Space) {
        return;
    }

    // 🔄 Cycle through modes: Bokeh → Gaussian → Off → Bokeh
    app_settings.mode = match app_settings.mode {
        Some(DepthOfFieldMode::Bokeh) => Some(DepthOfFieldMode::Gaussian),
        Some(DepthOfFieldMode::Gaussian) => None,
        None => Some(DepthOfFieldMode::Bokeh),
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            // 🎯 Objects 7 meters away will be in perfect focus
            // This is roughly the middle of our test scene
            focal_distance: 7.0,

            // 📸 Set a dramatic blur level for demonstration
            // f/0.125 is impossibly wide in real life (f/0.95 is the widest ever made!)
            // but it creates a beautiful effect in our virtual world
            aperture_f_stops: 1.0 / 8.0,

            // ✨ Bokeh mode creates the most photorealistic blur
            // with those characteristic circular highlights
            mode: Some(DepthOfFieldMode::Bokeh),
        }
    }
}

// 🔄 DOF Update System: Applying Camera Settings
/// Writes the depth of field settings into the camera.
fn update_dof_settings(
    mut commands: Commands,
    view_targets: Query<Entity, With<Camera>>,
    app_settings: Res<AppSettings>,
) {
    // 🎨 Convert our settings into the DOF component
    let depth_of_field: Option<DepthOfField> = (*app_settings).into();
    
    // 📷 Apply to all cameras (usually just one)
    for view in view_targets.iter() {
        match depth_of_field {
            None => {
                // 🚫 Remove DOF component to disable the effect
                commands.entity(view).remove::<DepthOfField>();
            }
            Some(depth_of_field) => {
                // ✅ Insert/update DOF component to enable the effect
                commands.entity(view).insert(depth_of_field);
            }
        }
    }
}

// 🎬 Scene Enhancement: Adding the Final Touches
/// Makes one-time adjustments to the scene that can't be encoded in glTF.
fn tweak_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lights: Query<&mut DirectionalLight, Changed<DirectionalLight>>,
    mut named_entities: Query<
        (Entity, &GltfMeshName, &MeshMaterial3d<StandardMaterial>),
        (With<Mesh3d>, Without<Lightmap>),
    >,
) {
    // 🌞 Enable shadows for more realistic lighting
    for mut light in lights.iter_mut() {
        light.shadows_enabled = true;
    }

    // 💡 Add HDR lightmap to the circuit board
    // This creates realistic indirect lighting and makes the electronics glow!
    for (entity, name, material) in named_entities.iter_mut() {
        if &**name == "CircuitBoard" {
            // 🔆 Boost exposure for dramatic effect
            materials.get_mut(material).unwrap().lightmap_exposure = 10000.0;
            // 🗺️ Apply pre-baked lighting
            commands.entity(entity).insert(Lightmap {
                image: asset_server.load("models/DepthOfFieldExample/CircuitBoardLightmap.hdr"),
                ..default()
            });
        }
    }
}

// 📝 UI Update System
/// Update the help text entity per the current app settings.
fn update_text(mut texts: Query<&mut Text>, app_settings: Res<AppSettings>) {
    for mut text in texts.iter_mut() {
        *text = create_text(&app_settings);
    }
}

/// Regenerates the app text component per the current app settings.
fn create_text(app_settings: &AppSettings) -> Text {
    app_settings.help_text().into()
}

// 🔄 Settings Conversion: From UI to Renderer
impl From<AppSettings> for Option<DepthOfField> {
    fn from(app_settings: AppSettings) -> Self {
        app_settings.mode.map(|mode| DepthOfField {
            mode,
            focal_distance: app_settings.focal_distance,
            aperture_f_stops: app_settings.aperture_f_stops,
            max_depth: 14.0,  // Maximum distance to apply DOF effect
            ..default()
        })
    }
}

impl AppSettings {
    // 📊 Generate Informative Help Text
    /// Builds the help text.
    fn help_text(&self) -> String {
        let Some(mode) = self.mode else {
            return "Mode: Off (Press Space to change)".to_owned();
        };

        // 📷 Physical Camera Parameters
        // These simulate a real camera sensor (default is 35mm full-frame)
        let sensor_height = PhysicalCameraParameters::default().sensor_height;
        let fov = PerspectiveProjection::default().fov;

        format!(
            "Focal distance: {} m (Press Up/Down to change)
Aperture F-stops: f/{} (Press Left/Right to change)
Sensor height: {}mm
Focal length: {}mm
Mode: {} (Press Space to change)",
            self.focal_distance,
            self.aperture_f_stops,
            sensor_height * 1000.0,  // Convert to millimeters
            dof::calculate_focal_length(sensor_height, fov) * 1000.0,
            match mode {
                DepthOfFieldMode::Bokeh => "Bokeh",
                DepthOfFieldMode::Gaussian => "Gaussian",
            }
        )
    }
}

// 🎓 Deep Dive: The Science of Depth of Field
//
// **Circle of Confusion (CoC)**:
// When a point of light is out of focus, it becomes a blurred circle on the
// image sensor. The size of this circle determines how blurry things appear.
//
// The CoC diameter depends on:
// 1. Distance from focal plane (how far out of focus)
// 2. Aperture size (f-stop)
// 3. Focal length of the lens
// 4. Distance to the subject
//
// **The Thin Lens Equation**:
// 1/f = 1/u + 1/v
// Where:
// - f = focal length
// - u = object distance
// - v = image distance
//
// **Bokeh vs Gaussian**:
// 
// Gaussian Blur:
// - Simple, fast approximation
// - Uniform blur kernel
// - Good for performance
// - Less realistic
//
// Bokeh:
// - Simulates actual lens optics
// - Creates circular/polygonal highlights
// - Shape depends on aperture blades
// - More expensive but beautiful
//
// **Real Camera Settings**:
// Common f-stops: f/1.4, f/2, f/2.8, f/4, f/5.6, f/8, f/11, f/16, f/22
// Each stop doubles/halves the light and blur amount
//
// Portrait photographers love f/1.4-f/2.8 for background blur
// Landscape photographers use f/8-f/11 for sharpness throughout

// 💡 Practical Tips:
//
// **For Portraits**: 
// - Focus on eyes (2-3m)
// - Use f/1.4-f/2.8
// - Background 5m+ away
//
// **For Products**:
// - Focus on main feature
// - Use f/4-f/5.6
// - Clean, simple background
//
// **For Architecture**:
// - Use f/8-f/11
// - Focus at hyperfocal distance
// - Maximum sharpness throughout
//
// **For Artistic Effects**:
// - Use extremely wide apertures
// - Focus on unexpected areas
// - Let bokeh create the mood
