//! Demonstrates how shadow biases affect shadows in a 3d scene.
//!
//! # Shadow Biases: Fighting the Shadow Acne
//!
//! When you're making shadows in 3D, you'll encounter "shadow acne" - 
//! surfaces incorrectly shadowing themselves, creating speckled patterns.
//! This happens because of floating-point precision limits.
//!
//! ## The Problem:
//!
//! When checking if a surface point is in shadow, we sample the shadow map.
//! Due to precision errors, a surface might think it's shadowing itself!
//!
//! ## The Solution: Shadow Biases
//!
//! 1. **Depth Bias**: Push shadows away from surfaces in light direction
//! 2. **Normal Bias**: Offset sample points along surface normals
//!
//! ## This Example Shows:
//!
//! - Interactive bias adjustment with keyboard controls
//! - Multiple shadow filtering methods
//! - Point lights vs directional lights
//! - Real-time parameter feedback
//!
//! Too little bias = shadow acne (speckles)
//! Too much bias = "Peter Panning" (shadows detached from objects)

// Rust: Module declaration with custom path
// #[path] attribute specifies file location relative to current file
#[path = "../helpers/camera_controller.rs"]
// Rust: Module declaration brings external file into scope
mod camera_controller;

// Rust: External crate imports
use bevy::{
    // Rust: Specific import from pbr module
    pbr::ShadowFilteringMethod, 
    // Rust: Glob import for common types
    prelude::*
};
// Rust: Import from local module
use camera_controller::{CameraController, CameraControllerPlugin};

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Standard Bevy plugins
        .add_plugins(DefaultPlugins)
        // Rust: Custom plugin for camera movement
        .add_plugins(CameraControllerPlugin)
        // Rust: Single system for startup
        .add_systems(Startup, setup)
        // Rust: Multiple systems as tuple for Update
        .add_systems(
            Update,
            // Rust: Tuple of function pointers
            // All run every frame, order not guaranteed
            (
                cycle_filter_methods,
                adjust_light_position,
                adjust_point_light_biases,
                toggle_light,
                adjust_directional_light_biases,
            ),
        )
        // Rust: Consume App and run game loop
        .run();
}

// Rust: Derive macro for component implementation
#[derive(Component)]
// Rust: Empty struct (zero-size type)
// Used as marker to identify light entities
struct Lights;

/// set up a 3D scene to test shadow biases and perspective projections
// Rust: System function signature
fn setup(
    // Rust: Mutable Commands for spawning entities
    mut commands: Commands,
    // Rust: Mutable asset storage access
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Rust: Local variable bindings with type annotations
    // f32 suffix explicitly specifies float type
    let spawn_plane_depth = 300.0f32;  // Scene depth
    let spawn_height = 2.0;             // Sphere height
    let sphere_radius = 0.25;           // Sphere radius

    // Rust: Creating shared material asset
    let white_handle = materials.add(StandardMaterial {
        // Rust: Constant color value
        base_color: Color::WHITE,
        // Rust: f32 literal for surface roughness
        perceptual_roughness: 1.0,  // Completely rough (no reflections)
        // Rust: Default remaining fields
        ..default()
    });
    // Rust: Creating shared mesh asset
    let sphere_handle = meshes.add(
        // Rust: Function with parameter
        Sphere::new(sphere_radius)
    );

    // Rust: Transform creation with method chaining
    let light_transform = Transform::from_xyz(5.0, 5.0, 0.0)
        // Rust: Method that modifies and returns self
        .looking_at(Vec3::ZERO, Vec3::Y);
    
    // Rust: Entity spawning with method chaining
    commands
        // Rust: Tuple of components
        .spawn((light_transform, Visibility::default(), Lights))
        // Rust: Child spawning with closure
        .with_children(|builder| {
            // Rust: Spawn point light child
            builder.spawn(PointLight {
                intensity: 0.0,  // Start disabled
                // Rust: Variable reference in struct
                range: spawn_plane_depth,
                color: Color::WHITE,
                // Rust: bool literal
                shadows_enabled: true,
                ..default()
            });
            // Rust: Spawn directional light child
            builder.spawn(DirectionalLight {
                shadows_enabled: true,
                ..default()
            });
        });

    // camera
    // Rust: Camera entity with multiple components
    commands.spawn((
        // Rust: Default camera
        Camera3d::default(),
        // Rust: Chained transform methods
        Transform::from_xyz(-1.0, 1.0, 1.0)
            .looking_at(Vec3::new(-1.0, 1.0, 0.0), Vec3::Y),
        // Rust: Default camera controller
        CameraController::default(),
        // Rust: Enum variant for shadow filtering
        ShadowFilteringMethod::Hardware2x2,
    ));

    // Rust: for loop with range and iterator methods
    for z_i32 in (-spawn_plane_depth as i32..=0)  // Range from -300 to 0
        .step_by(2)  // Every 2 units
    {
        // Rust: Spawn sphere at each position
        commands.spawn((
            // Rust: Clone Handle (cheap - reference counted)
            Mesh3d(sphere_handle.clone()),
            MeshMaterial3d(white_handle.clone()),
            // Rust: Transform with conditional Y position
            Transform::from_xyz(
                0.0,  // X position
                // Rust: if expression (returns value)
                if z_i32 % 4 == 0 {  // Every 4th sphere
                    spawn_height     // Higher position
                } else {
                    sphere_radius    // Lower position
                },
                // Rust: Type casting i32 to f32
                z_i32 as f32,
            ),
        ));
    }

    // ground plane
    // Rust: Arithmetic expression
    let plane_size = 2.0 * spawn_plane_depth;
    commands.spawn((
        // Rust: Complex nested method calls
        Mesh3d(meshes.add(
            Plane3d::default()      // Default plane primitive
                .mesh()             // Convert to mesh builder
                .size(plane_size, plane_size)  // Set dimensions
        )),
        // Rust: Move white_handle (no clone needed)
        MeshMaterial3d(white_handle),
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.75)),
            GlobalZIndex(i32::MAX),
        ))
        .with_children(|p| {
            p.spawn(Text::default()).with_children(|p| {
                p.spawn(TextSpan::new("Controls:\n"));
                p.spawn(TextSpan::new("R / Z - reset biases to default / zero\n"));
                p.spawn(TextSpan::new(
                    "L     - switch between directional and point lights [",
                ));
                p.spawn(TextSpan::new("DirectionalLight"));
                p.spawn(TextSpan::new("]\n"));
                p.spawn(TextSpan::new(
                    "F     - switch directional light filter methods [",
                ));
                p.spawn(TextSpan::new("Hardware2x2"));
                p.spawn(TextSpan::new("]\n"));
                p.spawn(TextSpan::new("1/2   - change point light depth bias ["));
                p.spawn(TextSpan::new("0.00"));
                p.spawn(TextSpan::new("]\n"));
                p.spawn(TextSpan::new("3/4   - change point light normal bias ["));
                p.spawn(TextSpan::new("0.0"));
                p.spawn(TextSpan::new("]\n"));
                p.spawn(TextSpan::new("5/6   - change direction light depth bias ["));
                p.spawn(TextSpan::new("0.00"));
                p.spawn(TextSpan::new("]\n"));
                p.spawn(TextSpan::new(
                    "7/8   - change direction light normal bias [",
                ));
                p.spawn(TextSpan::new("0.0"));
                p.spawn(TextSpan::new("]\n"));
                p.spawn(TextSpan::new(
                    "left/right/up/down/pgup/pgdown - adjust light position (looking at 0,0,0) [",
                ));
                p.spawn(TextSpan(format!("{:.1},", light_transform.translation.x)));
                p.spawn(TextSpan(format!(" {:.1},", light_transform.translation.y)));
                p.spawn(TextSpan(format!(" {:.1}", light_transform.translation.z)));
                p.spawn(TextSpan::new("]\n"));
            });
        });
}

// Rust: System function with multiple parameters
fn toggle_light(
    // Rust: Input resource for keyboard state
    input: Res<ButtonInput<KeyCode>>,
    // Rust: Query for mutable point lights
    mut point_lights: Query<&mut PointLight>,
    // Rust: Query for mutable directional lights
    mut directional_lights: Query<&mut DirectionalLight>,
    // Rust: Single entity query (exactly one entity)
    example_text: Single<Entity, With<Text>>,
    // Rust: UI text writer for updating display
    mut writer: TextUiWriter,
) {
    // Rust: Method for single-frame key press detection
    if input.just_pressed(KeyCode::KeyL) {
        // Rust: Iterate over query results mutably
        for mut light in &mut point_lights {
            // Rust: if expression for conditional assignment
            light.intensity = if light.intensity == 0.0 {
                // Rust: Dereferencing and text indexing
                // *writer.text() gets mutable reference to text span
                *writer.text(*example_text, 4) = "PointLight".to_string();
                100000000.0  // Very bright
            } else {
                0.0  // Turn off
            };
        }
        // Rust: Same pattern for directional lights
        for mut light in &mut directional_lights {
            light.illuminance = if light.illuminance == 0.0 {
                *writer.text(*example_text, 4) = "DirectionalLight".to_string();
                100000.0  // Bright in lux
            } else {
                0.0
            };
        }
    }
}

// Rust: System for adjusting light position with arrow keys
fn adjust_light_position(
    input: Res<ButtonInput<KeyCode>>,
    // Rust: Query with filter - Transform AND With<Lights>
    // Only gets transforms from entities that have Lights component
    mut lights: Query<&mut Transform, With<Lights>>,
    example_text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
) {
    // Rust: Mutable variable for accumulating movement
    let mut offset = Vec3::ZERO;  // Start with no movement
    
    // Rust: Build up offset vector based on key presses
    // Multiple keys can be pressed simultaneously
    if input.just_pressed(KeyCode::ArrowLeft) {
        // Rust: Field access and compound assignment
        offset.x -= 1.0;
    }
    if input.just_pressed(KeyCode::ArrowRight) {
        offset.x += 1.0;
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        offset.z -= 1.0;  // Z forward in Bevy's coordinate system
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        offset.z += 1.0;
    }
    if input.just_pressed(KeyCode::PageDown) {
        offset.y -= 1.0;  // Y down
    }
    if input.just_pressed(KeyCode::PageUp) {
        offset.y += 1.0;  // Y up
    }
    
    // Rust: Only update if there was movement
    if offset != Vec3::ZERO {
        // Rust: Dereference Single to get Entity value
        let example_text = *example_text;
        
        for mut light in &mut lights {
            // Rust: Vector addition with compound assignment
            light.translation += offset;
            // Rust: Make light look at origin after moving
            light.look_at(Vec3::ZERO, Vec3::Y);
            
            // Rust: Update UI with new position (formatted to 1 decimal)
            // Different text indices for X, Y, Z components
            *writer.text(example_text, 22) = format!("{:.1},", light.translation.x);
            *writer.text(example_text, 23) = format!(" {:.1},", light.translation.y);
            *writer.text(example_text, 24) = format!(" {:.1}", light.translation.z);
        }
    }
}

// Rust: System for cycling shadow filter methods
fn cycle_filter_methods(
    input: Res<ButtonInput<KeyCode>>,
    // Rust: Query for mutable shadow filtering method
    mut filter_methods: Query<&mut ShadowFilteringMethod>,
    example_text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
) {
    if input.just_pressed(KeyCode::KeyF) {
        for mut filter_method in &mut filter_methods {
            // Rust: Variable declaration without initialization
            let filter_method_string;
            // Rust: Pattern matching with dereferencing
            // *filter_method dereferences the &mut to get owned value
            *filter_method = match *filter_method {
                // Rust: Enum pattern matching
                ShadowFilteringMethod::Hardware2x2 => {
                    // Rust: Assignment in match arm
                    filter_method_string = "Gaussian".to_string();
                    ShadowFilteringMethod::Gaussian
                }
                ShadowFilteringMethod::Gaussian => {
                    filter_method_string = "Temporal".to_string();
                    ShadowFilteringMethod::Temporal
                }
                ShadowFilteringMethod::Temporal => {
                    filter_method_string = "Hardware2x2".to_string();
                    ShadowFilteringMethod::Hardware2x2
                }
            };
            // Rust: Update UI text with new method name
            *writer.text(*example_text, 7) = filter_method_string;
        }
    }
}

// Rust: System for adjusting point light shadow biases
fn adjust_point_light_biases(
    input: Res<ButtonInput<KeyCode>>,
    // Rust: Query targeting PointLight components specifically
    mut query: Query<&mut PointLight>,
    example_text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
) {
    // Rust: Const-like local variables
    // Step sizes control how much bias changes per key press
    let depth_bias_step_size = 0.01;   // Small steps for fine control
    let normal_bias_step_size = 0.1;   // Larger steps, less sensitive
    
    // Rust: Iterate over all point lights
    for mut light in &mut query {
        // Rust: Multiple if statements for different keys
        // Compound assignment operators (-=, +=)
        if input.just_pressed(KeyCode::Digit1) {
            light.shadow_depth_bias -= depth_bias_step_size;
        }
        if input.just_pressed(KeyCode::Digit2) {
            light.shadow_depth_bias += depth_bias_step_size;
        }
        if input.just_pressed(KeyCode::Digit3) {
            light.shadow_normal_bias -= normal_bias_step_size;
        }
        if input.just_pressed(KeyCode::Digit4) {
            light.shadow_normal_bias += normal_bias_step_size;
        }
        // Rust: Reset to default values
        if input.just_pressed(KeyCode::KeyR) {
            // Rust: Associated constants from PointLight
            light.shadow_depth_bias = PointLight::DEFAULT_SHADOW_DEPTH_BIAS;
            light.shadow_normal_bias = PointLight::DEFAULT_SHADOW_NORMAL_BIAS;
        }
        // Rust: Set to zero values
        if input.just_pressed(KeyCode::KeyZ) {
            light.shadow_depth_bias = 0.0;
            light.shadow_normal_bias = 0.0;
        }

        // Rust: format! macro with precision specifiers
        // {:.2} means 2 decimal places, {:.1} means 1 decimal place
        *writer.text(*example_text, 10) = format!("{:.2}", light.shadow_depth_bias);
        *writer.text(*example_text, 13) = format!("{:.1}", light.shadow_normal_bias);
    }
}

// Rust: Nearly identical system for directional lights
fn adjust_directional_light_biases(
    input: Res<ButtonInput<KeyCode>>,
    // Rust: Different query type - DirectionalLight instead of PointLight
    mut query: Query<&mut DirectionalLight>,
    example_text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
) {
    // Rust: Same step sizes as point lights
    let depth_bias_step_size = 0.01;
    let normal_bias_step_size = 0.1;
    
    for mut light in &mut query {
        // Rust: Different key codes (5-8 instead of 1-4)
        // This allows separate control of point vs directional lights
        if input.just_pressed(KeyCode::Digit5) {
            light.shadow_depth_bias -= depth_bias_step_size;
        }
        if input.just_pressed(KeyCode::Digit6) {
            light.shadow_depth_bias += depth_bias_step_size;
        }
        if input.just_pressed(KeyCode::Digit7) {
            light.shadow_normal_bias -= normal_bias_step_size;
        }
        if input.just_pressed(KeyCode::Digit8) {
            light.shadow_normal_bias += normal_bias_step_size;
        }
        // Rust: Same reset/zero logic but different defaults
        if input.just_pressed(KeyCode::KeyR) {
            // Rust: DirectionalLight has its own default constants
            light.shadow_depth_bias = DirectionalLight::DEFAULT_SHADOW_DEPTH_BIAS;
            light.shadow_normal_bias = DirectionalLight::DEFAULT_SHADOW_NORMAL_BIAS;
        }
        if input.just_pressed(KeyCode::KeyZ) {
            light.shadow_depth_bias = 0.0;
            light.shadow_normal_bias = 0.0;
        }

        // Rust: Update different text indices (16, 19 vs 10, 13)
        // Each light type updates its own UI section
        *writer.text(*example_text, 16) = format!("{:.2}", light.shadow_depth_bias);
        *writer.text(*example_text, 19) = format!("{:.1}", light.shadow_normal_bias);
    }
}

// 🎯 Advanced Rust Patterns in This Example:
//
// 1. **Module System**:
//    - `#[path]` attribute for custom file paths
//    - `mod` declaration brings code into scope
//    - Useful for organizing helper modules
//
// 2. **Iterator Methods**:
//    - `(-depth as i32..=0)` - Inclusive range
//    - `.step_by(2)` - Skip elements
//    - Range + step_by creates arithmetic sequence
//
// 3. **Pattern Matching in Assignment**:
//    - `*filter_method = match *filter_method { ... }`
//    - Dereference both sides of assignment
//    - Match returns new enum value
//
// 4. **Conditional Expressions**:
//    - `if condition { value1 } else { value2 }`
//    - if is expression, not statement
//    - Can be used in assignments
//
// 5. **Associated Constants**:
//    - `PointLight::DEFAULT_SHADOW_DEPTH_BIAS`
//    - Type-associated values (like static in other languages)
//    - Namespaced under type name
//
// 6. **Compound Operators**:
//    - `+=`, `-=` modify in place
//    - Syntactic sugar for `x = x + y`
//    - Must implement appropriate trait
//
// 7. **Format Specifiers**:
//    - `{:.2}` - 2 decimal places
//    - `{:.1}` - 1 decimal place
//    - Many other formatting options available
