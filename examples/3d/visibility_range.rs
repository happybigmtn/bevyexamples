//! Demonstrates visibility ranges, also known as HLODs.
//!
//! # Visibility Ranges & HLODs: Level-of-Detail Optimization
//!
//! HLODs (Hierarchical Level of Detail) optimize rendering by showing different
//! model versions based on camera distance. Far away? Show low-poly. Up close?
//! Show high-poly. This saves GPU power without sacrificing visual quality!
//!
//! ## Real-World Applications:
//!
//! - **Open World Games**: Trees switch from 3D to billboards at distance
//! - **Racing Games**: Trackside objects simplify when you zoom past
//! - **Flight Sims**: Cities show less detail from high altitude
//! - **Strategy Games**: Units simplify when zoomed out
//!
//! ## How It Works:
//!
//! 1. **Distance Calculation**: Measure camera-to-object distance
//! 2. **Range Check**: Compare against visibility thresholds
//! 3. **Model Swap**: Show appropriate detail level
//! 4. **Smooth Transition**: Fade between levels (margins)
//!
//! ## This Example Features:
//!
//! - Flight helmet with high-poly and low-poly versions
//! - Automatic LOD switching based on zoom
//! - Manual control over which model shows
//! - Prepass toggle for performance comparison

// Rust: Import from standard library
use std::f32::consts::PI;  // Mathematical constant π

// Rust: Complex nested imports from Bevy
use bevy::{
    // Rust: Prepass components for early depth/normal rendering
    core_pipeline::prepass::{DepthPrepass, NormalPrepass},
    // Rust: Mouse input events
    input::mouse::MouseWheel,
    // Rust: Convenience function for Vec3 creation
    math::vec3,
    // Rust: Lighting constants and shadow configuration
    pbr::{light_consts::lux::FULL_DAYLIGHT, CascadeShadowConfigBuilder},
    // Rust: Common Bevy types
    prelude::*,
    // Rust: Visibility range component for LOD
    render::view::VisibilityRange,
};

// Where the camera is focused.
// Rust: const creates compile-time constant
// vec3() is a const fn that creates Vec3 at compile time
const CAMERA_FOCAL_POINT: Vec3 = vec3(0.0, 0.3, 0.0);

// Speed in units per frame.
// Rust: f32 type annotation for floating-point constant
const CAMERA_KEYBOARD_ZOOM_SPEED: f32 = 0.05;

// Speed in radians per frame.
// Rust: Constants follow SCREAMING_SNAKE_CASE convention
const CAMERA_KEYBOARD_PAN_SPEED: f32 = 0.01;

// Speed in units per frame.
const CAMERA_MOUSE_MOVEMENT_SPEED: f32 = 0.25;

// The minimum distance that the camera is allowed to be from the model.
const MIN_ZOOM_DISTANCE: f32 = 0.5;

// The visibility ranges for high-poly and low-poly models respectively, when
// both models are being shown.
// Rust: static creates global variable with 'static lifetime
// Unlike const, static has a fixed memory location
static NORMAL_VISIBILITY_RANGE_HIGH_POLY: VisibilityRange = VisibilityRange {
    // Rust: Range literal start..end (exclusive end)
    start_margin: 0.0..0.0,  // Visible from distance 0
    end_margin: 3.0..4.0,    // Fade out between 3-4 units
    // Rust: bool literal
    use_aabb: false,         // Use sphere bounds, not box
};

// Rust: Another static for low-poly model
static NORMAL_VISIBILITY_RANGE_LOW_POLY: VisibilityRange = VisibilityRange {
    start_margin: 3.0..4.0,  // Fade in as high-poly fades out
    end_margin: 8.0..9.0,    // Fade out at far distance
    use_aabb: false,
};

// A visibility model that we use to always show a model (until the camera is so
// far zoomed out that it's culled entirely).
static SINGLE_MODEL_VISIBILITY_RANGE: VisibilityRange = VisibilityRange {
    start_margin: 0.0..0.0,
    end_margin: 8.0..9.0,
    use_aabb: false,
};

// A visibility range that we use to completely hide a model.
static INVISIBLE_VISIBILITY_RANGE: VisibilityRange = VisibilityRange {
    start_margin: 0.0..0.0,
    end_margin: 0.0..0.0,
    use_aabb: false,
};

// Allows us to identify the main model.
// Rust: Multiple derive macros on enum
#[derive(
    Component,  // Can be attached to entities
    Debug,      // Enables {:?} formatting
    Clone,      // Can be duplicated
    Copy,       // Cheap bit-wise copy
    PartialEq   // Enables == comparison
)]
enum MainModel {
    // The high-poly version.
    HighPoly,
    // The low-poly version.
    LowPoly,
}

// The current mode.
// Rust: Derive macros for struct
#[derive(Default, Resource)]
struct AppStatus {
    // Whether to show only one model.
    // Rust: Option<T> represents nullable value
    // None = show both, Some(model) = show only that model
    show_one_model_only: Option<MainModel>,
    // Whether to enable the prepass.
    // Rust: bool field with default value (false)
    prepass: bool,
}

// Sets up the app.
// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Plugin configuration with method chaining
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // Rust: Option<Window> with Some variant
            primary_window: Some(Window {
                // Rust: String conversion with .into()
                // &str -> String via Into trait
                title: "Bevy Visibility Range Example".into(),
                // Rust: Struct update syntax
                ..default()
            }),
            ..default()
        }))
        // Rust: Initialize resource using Default trait
        // Turbofish ::<AppStatus> specifies type
        .init_resource::<AppStatus>()
        // Rust: Single system for Startup
        .add_systems(Startup, setup)
        // Rust: Multiple systems as tuple for Update
        .add_systems(
            Update,
            (
                move_camera,
                set_visibility_ranges,
                update_help_text,
                update_mode,
                toggle_prepass,
            ),
        )
        // Rust: Consume app and run
        .run();
}

// Set up a simple 3D scene. Load the two meshes.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    app_status: Res<AppStatus>,
) {
    // Spawn a plane.
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.2, 0.1))),
    ));

    // Spawn the two HLODs.

    commands.spawn((
        SceneRoot(
            asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
        MainModel::HighPoly,
    ));

    commands.spawn((
        SceneRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0)
                    .from_asset("models/FlightHelmetLowPoly/FlightHelmetLowPoly.gltf"),
            ),
        ),
        MainModel::LowPoly,
    ));

    // Spawn a light.
    commands.spawn((
        DirectionalLight {
            illuminance: FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI * -0.15, PI * -0.15)),
        CascadeShadowConfigBuilder {
            maximum_distance: 30.0,
            first_cascade_far_bound: 0.9,
            ..default()
        }
        .build(),
    ));

    // Spawn a camera.
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.7, 0.7, 1.0).looking_at(CAMERA_FOCAL_POINT, Vec3::Y),
        ))
        .insert(EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 150.0,
            ..default()
        });

    // Create the text.
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

// We need to add the `VisibilityRange` components manually, as glTF currently
// has no way to specify visibility ranges. This system watches for new meshes,
// determines which `Scene` they're under, and adds the `VisibilityRange`
// component as appropriate.
// Rust: System for adding LOD components to loaded meshes
fn set_visibility_ranges(
    // Rust: Commands for entity modifications
    mut commands: Commands,
    // Rust: Query for newly added meshes
    // Added<T> filter only returns entities where T was added this frame
    mut new_meshes: Query<Entity, Added<Mesh3d>>,
    // Rust: Query for parent-child relationships
    // Tuple of Option types for components that might not exist
    children: Query<(Option<&ChildOf>, Option<&MainModel>)>,
) {
    // Loop over each newly-added mesh.
    // Rust: Iterate over query results
    for new_mesh in new_meshes.iter_mut() {
        // Search for the nearest ancestor `MainModel` component.
        // Rust: Mutable variable bindings with tuple
        let (mut current, mut main_model) = (new_mesh, None);
        
        // Rust: while-let loop for pattern matching
        // Continues while pattern matches successfully
        while let Ok((child_of, maybe_main_model)) = children.get(current) {
            // Rust: Nested if-let for Option handling
            if let Some(model) = maybe_main_model {
                main_model = Some(model);
                break;  // Found it, exit loop
            }
            // Rust: Match expression on Option
            match child_of {
                // Rust: If has parent, move up hierarchy
                Some(child_of) => current = child_of.parent(),
                // Rust: No parent, reached root
                None => break,
            }
        }

        // Add the `VisibilityRange` component.
        match main_model {
            Some(MainModel::HighPoly) => {
                commands
                    .entity(new_mesh)
                    .insert(NORMAL_VISIBILITY_RANGE_HIGH_POLY.clone())
                    .insert(MainModel::HighPoly);
            }
            Some(MainModel::LowPoly) => {
                commands
                    .entity(new_mesh)
                    .insert(NORMAL_VISIBILITY_RANGE_LOW_POLY.clone())
                    .insert(MainModel::LowPoly);
            }
            None => {}
        }
    }
}

// Process the movement controls.
fn move_camera(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let (mut zoom_delta, mut theta_delta) = (0.0, 0.0);

    // Process zoom in and out via the keyboard.
    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        zoom_delta -= CAMERA_KEYBOARD_ZOOM_SPEED;
    } else if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        zoom_delta += CAMERA_KEYBOARD_ZOOM_SPEED;
    }

    // Process left and right pan via the keyboard.
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        theta_delta -= CAMERA_KEYBOARD_PAN_SPEED;
    } else if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        theta_delta += CAMERA_KEYBOARD_PAN_SPEED;
    }

    // Process zoom in and out via the mouse wheel.
    for event in mouse_wheel_events.read() {
        zoom_delta -= event.y * CAMERA_MOUSE_MOVEMENT_SPEED;
    }

    // Update the camera transform.
    for transform in cameras.iter_mut() {
        let transform = transform.into_inner();

        let direction = transform.translation.normalize_or_zero();
        let magnitude = transform.translation.length();

        let new_direction = Mat3::from_rotation_y(theta_delta) * direction;
        let new_magnitude = (magnitude + zoom_delta).max(MIN_ZOOM_DISTANCE);

        transform.translation = new_direction * new_magnitude;
        transform.look_at(CAMERA_FOCAL_POINT, Vec3::Y);
    }
}

// Toggles modes if the user requests.
fn update_mode(
    mut meshes: Query<(&mut VisibilityRange, &MainModel)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_status: ResMut<AppStatus>,
) {
    // Toggle the mode as requested.
    if keyboard_input.just_pressed(KeyCode::Digit1) || keyboard_input.just_pressed(KeyCode::Numpad1)
    {
        app_status.show_one_model_only = None;
    } else if keyboard_input.just_pressed(KeyCode::Digit2)
        || keyboard_input.just_pressed(KeyCode::Numpad2)
    {
        app_status.show_one_model_only = Some(MainModel::HighPoly);
    } else if keyboard_input.just_pressed(KeyCode::Digit3)
        || keyboard_input.just_pressed(KeyCode::Numpad3)
    {
        app_status.show_one_model_only = Some(MainModel::LowPoly);
    } else {
        return;
    }

    // Update the visibility ranges as appropriate.
    for (mut visibility_range, main_model) in meshes.iter_mut() {
        *visibility_range = match (main_model, app_status.show_one_model_only) {
            (&MainModel::HighPoly, Some(MainModel::LowPoly))
            | (&MainModel::LowPoly, Some(MainModel::HighPoly)) => {
                INVISIBLE_VISIBILITY_RANGE.clone()
            }
            (&MainModel::HighPoly, Some(MainModel::HighPoly))
            | (&MainModel::LowPoly, Some(MainModel::LowPoly)) => {
                SINGLE_MODEL_VISIBILITY_RANGE.clone()
            }
            (&MainModel::HighPoly, None) => NORMAL_VISIBILITY_RANGE_HIGH_POLY.clone(),
            (&MainModel::LowPoly, None) => NORMAL_VISIBILITY_RANGE_LOW_POLY.clone(),
        }
    }
}

// Toggles the prepass if the user requests.
fn toggle_prepass(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_status: ResMut<AppStatus>,
) {
    if !keyboard_input.just_pressed(KeyCode::Space) {
        return;
    }

    app_status.prepass = !app_status.prepass;

    for camera in cameras.iter() {
        if app_status.prepass {
            commands
                .entity(camera)
                .insert(DepthPrepass)
                .insert(NormalPrepass);
        } else {
            commands
                .entity(camera)
                .remove::<DepthPrepass>()
                .remove::<NormalPrepass>();
        }
    }
}

// A system that updates the help text.
fn update_help_text(mut text_query: Query<&mut Text>, app_status: Res<AppStatus>) {
    for mut text in text_query.iter_mut() {
        *text = app_status.create_text();
    }
}

// Rust: Implementation block for methods
impl AppStatus {
    // Creates and returns help text reflecting the app status.
    // Rust: Method with immutable self reference
    fn create_text(&self) -> Text {
        // Rust: format! macro for string interpolation
        format!(
            // Rust: Raw string literal with \ continuation
            "\
{} (1) Switch from high-poly to low-poly based on camera distance
{} (2) Show only the high-poly model
{} (3) Show only the low-poly model
Press 1, 2, or 3 to switch which model is shown
Press WASD or use the mouse wheel to move the camera
Press Space to {} the prepass",
            // Rust: Conditional expression (ternary-like)
            // Returns '>' or ' ' based on condition
            if self.show_one_model_only.is_none() {
                '>'  // Selected
            } else {
                ' '  // Not selected
            },
            // Rust: Option equality comparison
            // == works because MainModel derives PartialEq
            if self.show_one_model_only == Some(MainModel::HighPoly) {
                '>'
            } else {
                ' '
            },
            if self.show_one_model_only == Some(MainModel::LowPoly) {
                '>'
            } else {
                ' '
            },
            // Rust: Conditional string selection
            if self.prepass { "disable" } else { "enable" }
        )
        // Rust: .into() converts String to Text
        // Works via From/Into trait implementation
        .into()
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Static vs Const**:
//    - `const` - Inlined at compile time, no memory address
//    - `static` - Global variable with fixed address
//    - Use static for large data structures
//
// 2. **Added<T> Query Filter**:
//    - Only returns entities where T was added this frame
//    - Useful for reacting to new entities
//    - More efficient than checking all entities
//
// 3. **Option Patterns**:
//    - `is_none()` - Check if Option is None
//    - `Some(value)` - Wrap value in Option
//    - Pattern matching with if-let and match
//
// 4. **Range Types**:
//    - `start..end` - Exclusive range (doesn't include end)
//    - Used for visibility fade regions
//    - Can be iterated or used as bounds
//
// 5. **Entity Hierarchies**:
//    - `ChildOf` component tracks parent relationships
//    - Walk up tree to find components on ancestors
//    - Common pattern for scene graphs
//
// 6. **Conditional Logic**:
//    - `if condition { value1 } else { value2 }`
//    - Can be used as expression (returns value)
//    - Cleaner than match for simple cases
//
// 7. **LOD (Level of Detail)**:
//    - VisibilityRange controls when models show
//    - Margins create smooth transitions
//    - Reduces GPU load for distant objects
//
// 8. **Resource Initialization**:
//    - `init_resource::<T>()` uses Default trait
//    - Alternative to manual insert_resource
//    - Ensures resource exists before systems run
