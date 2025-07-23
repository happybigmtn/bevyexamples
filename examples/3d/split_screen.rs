//! Renders two cameras to the same window to accomplish "split screen".
//!
//! # Split Screen: Multiple Views, One World
//!
//! Remember playing split-screen games with friends on the couch? Each player
//! gets their own view of the same game world. This example shows how to create
//! that classic split-screen experience with multiple cameras!
//!
//! ## How Split Screen Works:
//!
//! 1. **Multiple Cameras**: Each viewport has its own camera
//! 2. **Viewport Subdivision**: Screen divided into regions
//! 3. **Render Order**: Cameras render in specific order to avoid conflicts
//! 4. **Per-Camera UI**: Each viewport can have its own UI elements
//!
//! ## This Example Creates:
//!
//! - 4-player split screen (2x2 grid)
//! - Each player can rotate their camera independently
//! - UI buttons specific to each viewport
//! - Dynamic viewport resizing when window changes
//!
//! Perfect for local multiplayer games or security camera systems!

// Rust: Import PI constant for rotation calculations
use std::f32::consts::PI;

// Rust: Selective imports from Bevy modules
use bevy::{
    // Rust: Shadow cascade configuration builder
    pbr::CascadeShadowConfigBuilder, 
    // Rust: Common Bevy types
    prelude::*, 
    // Rust: Viewport for camera render regions
    render::camera::Viewport, 
    // Rust: Window resize event
    window::WindowResized,
};

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // Rust: Tuple of systems for Update schedule
        // Both run every frame
        .add_systems(Update, (set_camera_viewports, button_system))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/animated/Fox.glb")),
    ));

    // Light
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            num_cascades: if cfg!(all(
                feature = "webgl2",
                target_arch = "wasm32",
                not(feature = "webgpu")
            )) {
                // Limited to 1 cascade in WebGL
                1
            } else {
                2
            },
            first_cascade_far_bound: 200.0,
            maximum_distance: 280.0,
            ..default()
        }
        .build(),
    ));

    // Cameras and their dedicated UI
    // Rust: Array literal with tuples
    // [(name, position), ...] creates static data
    for (index, (camera_name, camera_pos)) in [
        ("Player 1", Vec3::new(0.0, 200.0, -150.0)),
        ("Player 2", Vec3::new(150.0, 150., 50.0)),
        ("Player 3", Vec3::new(100.0, 150., -150.0)),
        ("Player 4", Vec3::new(-100.0, 80., 150.0)),
    ]
    // Rust: Iterator method chain
    .iter()         // Create iterator over array references
    .enumerate()    // Add index to each item: (index, item)
    {
        // Rust: Spawn camera and capture its entity ID
        let camera = commands
            .spawn((
                Camera3d::default(),
                // Rust: Dereference camera_pos (it's a reference from .iter())
                Transform::from_translation(*camera_pos)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                Camera {
                    // Renders cameras with different priorities to prevent ambiguities
                    // Rust: Type casting with 'as'
                    order: index as isize,  // 0, 1, 2, 3
                    ..default()
                },
                // Rust: Custom component with calculated position
                CameraPosition {
                    // Rust: Grid position calculation
                    // index % 2 gives column (0 or 1)
                    // index / 2 gives row (0 or 1)
                    pos: UVec2::new((index % 2) as u32, (index / 2) as u32),
                },
            ))
            // Rust: Get entity ID for later reference
            .id();

        // Set up UI
        commands
            .spawn((
                UiTargetCamera(camera),
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(*camera_name),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(12.),
                        left: Val::Px(12.),
                        ..default()
                    },
                ));
                buttons_panel(parent);
            });
    }

    fn buttons_panel(parent: &mut ChildSpawnerCommands) {
        parent
            .spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            })
            .with_children(|parent| {
                rotate_button(parent, "<", Direction::Left);
                rotate_button(parent, ">", Direction::Right);
            });
    }

    fn rotate_button(parent: &mut ChildSpawnerCommands, caption: &str, direction: Direction) {
        parent
            .spawn((
                RotateCamera(direction),
                Button,
                Node {
                    width: Val::Px(40.),
                    height: Val::Px(40.),
                    border: UiRect::all(Val::Px(2.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            ))
            .with_children(|parent| {
                parent.spawn(Text::new(caption));
            });
    }
}

// Rust: Component derive macro
#[derive(Component)]
// Rust: Struct with single field
struct CameraPosition {
    // Rust: UVec2 is unsigned integer 2D vector
    // Used for grid coordinates (0,0), (1,0), etc.
    pos: UVec2,
}

// Rust: Tuple struct component
#[derive(Component)]
// Rust: Single unnamed field (newtype pattern)
struct RotateCamera(Direction);

// Rust: Simple enum without derive
// No need for Clone/Copy since used by reference
enum Direction {
    Left,
    Right,
}

// Rust: System for dynamic viewport sizing
fn set_camera_viewports(
    // Rust: Query for window information
    windows: Query<&Window>,
    // Rust: Event reader for window resize events
    mut resize_events: EventReader<WindowResized>,
    // Rust: Query for cameras with positions
    mut query: Query<(&CameraPosition, &mut Camera)>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    
    // Rust: Iterate over resize events
    for resize_event in resize_events.read() {
        // Rust: Get window from entity ID
        // unwrap() is safe here - resize event guarantees window exists
        let window = windows.get(resize_event.window).unwrap();
        
        // Rust: Calculate quarter-screen size
        // Division operator on UVec2 divides both components
        let size = window.physical_size() / 2;

        // Rust: Update all camera viewports
        for (camera_position, mut camera) in &mut query {
            // Rust: Set viewport with calculated position and size
            camera.viewport = Some(Viewport {
                // Rust: Vector multiplication
                // Multiplies grid position by size to get pixel position
                physical_position: camera_position.pos * size,
                physical_size: size,
                ..default()
            });
        }
    }
}

// Rust: System for handling button interactions
fn button_system(
    // Rust: Complex query with filters
    interaction_query: Query<
        // Rust: Tuple of component references
        (&Interaction, &ComputedNodeTarget, &RotateCamera),
        // Rust: Query filters - only buttons that changed
        (Changed<Interaction>, With<Button>),
    >,
    // Rust: Query for camera transforms
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    // Rust: Destructure tuple in for loop
    // Pattern matching extracts inner direction value
    for (interaction, computed_target, RotateCamera(direction)) in &interaction_query {
        // Rust: Pattern matching on enum reference
        // * dereferences &Interaction to compare values
        if let Interaction::Pressed = *interaction {
            // Since TargetCamera propagates to the children, we can use it to find
            // which side of the screen the button is on.
            
            // Rust: Option method chaining
            if let Some(mut camera_transform) = computed_target
                .camera()  // Returns Option<Entity>
                // Rust: and_then for Option chaining
                // Converts Option<Entity> to Option<Mut<Transform>>
                .and_then(|camera| camera_query.get_mut(camera).ok())
            {
                // Rust: Match expression for angle calculation
                let angle = match direction {
                    Direction::Left => -0.1,   // Negative = counter-clockwise
                    Direction::Right => 0.1,   // Positive = clockwise
                };
                
                // Rust: Rotate camera around world origin
                camera_transform.rotate_around(
                    Vec3::ZERO,  // Pivot point
                    Quat::from_axis_angle(Vec3::Y, angle)  // Y-axis rotation
                );
            }
        }
    }
}

// 🎯 Advanced Rust Concepts in This Example:
//
// 1. **Complex Query Filters**:
//    - `Changed<T>` - Only entities where T changed this frame
//    - `With<T>` - Only entities that have component T
//    - Combine with tuples for multiple filters
//
// 2. **Option Chaining**:
//    - `.and_then()` - Chain operations that return Option
//    - `.ok()` - Convert Result to Option
//    - Handles potential failures gracefully
//
// 3. **Viewport Calculations**:
//    - Grid position * size = pixel position
//    - Integer division for grid layout
//    - Dynamic resizing on window changes
//
// 4. **Entity References**:
//    - `.id()` captures entity for later use
//    - `UiTargetCamera(entity)` associates UI with camera
//    - Allows per-camera UI elements
//
// 5. **Pattern Matching in Parameters**:
//    - `RotateCamera(direction)` extracts enum data
//    - Destructuring in function signatures
//    - More concise than manual extraction
//
// 6. **Event Systems**:
//    - `EventReader<T>` consumes events
//    - `.read()` iterates over new events
//    - Events cleared after processing
//
// 7. **Conditional Compilation**:
//    - `cfg!()` macro for compile-time checks
//    - Different behavior for WebGL vs native
//    - Platform-specific optimizations
