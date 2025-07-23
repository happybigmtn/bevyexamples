//! This example showcases a 2D top-down camera with smooth player tracking.
//!
//! ## Key Concepts Demonstrated
//!
//! - **Camera Following**: Implementing smooth camera tracking of a moving player
//! - **2D Coordinate Systems**: Understanding how camera position relates to world space
//! - **Smooth Interpolation**: Using mathematical smoothing for natural camera movement
//! - **System Chaining**: Ensuring player movement is processed before camera updates
//! - **Bloom Effect**: Adding visual polish with HDR rendering effects
//!
//! ## The Camera System
//!
//! This example demonstrates a fundamental pattern in 2D games: the camera that follows
//! the player. The camera smoothly interpolates between its current position and the
//! player's position, creating a natural tracking effect without jarring snaps.
//!
//! ## Controls
//!
//! | Key Binding          | Action        |
//! |:---------------------|:--------------|
//! | `W`                  | Move up       |
//! | `S`                  | Move down     |
//! | `A`                  | Move left     |
//! | `D`                  | Move right    |

use bevy::{
    // Bloom is a visual effect that makes bright objects glow
    // It requires HDR rendering and enhances the visual appeal
    core_pipeline::bloom::Bloom,
    prelude::*,
};

// MOVEMENT AND CAMERA CONFIGURATION
// =================================

/// Player movement speed in world units per second
/// This determines how fast the player can move around the world
const PLAYER_SPEED: f32 = 100.;

/// Camera interpolation rate - how quickly the camera follows the player
/// - Higher values = snappier camera movement (instant at very high values)
/// - Lower values = slower, more delayed camera movement
/// - 2.0 provides a nice balance between responsiveness and smoothness
const CAMERA_DECAY_RATE: f32 = 2.;

/// Marker component to identify the player entity
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_scene, setup_instructions, setup_camera))
        // CRITICAL: We use .chain() to ensure move_player runs BEFORE update_camera
        // This ensures the camera always tracks the player's current position
        // Without chaining, the systems could run in any order, causing jittery camera movement
        .add_systems(Update, (move_player, update_camera).chain())
        .run();
}

/// Sets up the game scene with a background world and a glowing player
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // WORLD BACKGROUND: Large rectangle representing the game world
    // This provides a visual reference for camera movement and helps players
    // understand the coordinate system
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1000., 700.))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.3))), // Dark blue-gray
    ));

    // PLAYER: A glowing circle that will be tracked by the camera
    commands.spawn((
        Player,  // Component to identify this as the player entity
        
        // Create a circular mesh for the player
        Mesh2d(meshes.add(Circle::new(25.))),  // 25-unit radius circle
        
        // HDR COLOR: Values > 1.0 enable bloom effect
        // In standard RGB, values are typically 0.0-1.0, but HDR allows higher values
        // These high values make the player glow with the bloom effect
        MeshMaterial2d(materials.add(Color::srgb(6.25, 9.4, 9.1))),
        
        // Position the player at the world origin, slightly above the background
        // Z=2 ensures the player renders in front of the background (Z=0)
        Transform::from_xyz(0., 0., 2.),
    ));
}

/// Creates on-screen instructions for the user
fn setup_instructions(mut commands: Commands) {
    commands.spawn((
        Text::new("Move the light with WASD.\nThe camera will smoothly track the light."),
        Node {
            // Absolute positioning places the text at specific screen coordinates
            // independent of other UI elements
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),  // 12 pixels from bottom of screen
            left: Val::Px(12.0),    // 12 pixels from left of screen
            ..default()
        },
    ));
}

/// Sets up the camera with bloom effect for visual enhancement
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,  // Basic 2D camera component
        
        // BLOOM EFFECT: Creates a glow around bright objects
        // Bloom::NATURAL provides realistic bloom settings
        // This works with HDR colors (values > 1.0) to create glowing effects
        // The player's high RGB values will cause it to glow beautifully
        Bloom::NATURAL,
    ));
}

/// Updates the camera position to smoothly follow the player
/// 
/// This system demonstrates a fundamental camera pattern in 2D games:
/// the camera that follows the player with smooth interpolation.
fn update_camera(
    // Query for the camera transform - it must have Camera2d but NOT Player component
    // The Without<Player> filter prevents conflicts if the player also had a Camera2d
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
    
    // Query for the player transform - it must have Player but NOT Camera2d component
    // The Without<Camera2d> filter prevents the same entity conflict
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    
    time: Res<Time>,
) {
    // EXTRACT PLAYER POSITION
    // Destructure the player's translation to get x and y coordinates
    // The `..` ignores the z component since we don't need it
    let Vec3 { x, y, .. } = player.translation;
    
    // CREATE TARGET POSITION
    // The camera should move to the player's X,Y position but keep its own Z
    // This maintains the camera's depth while following the player in 2D space
    let target_position = Vec3::new(x, y, camera.translation.z);

    // SMOOTH INTERPOLATION
    // smooth_nudge() gradually moves the camera toward the target position
    // instead of instantly snapping to it. This creates natural-feeling camera movement.
    // 
    // Parameters:
    // - target: where we want to be (player position)
    // - rate: how quickly to move there (CAMERA_DECAY_RATE)
    // - delta_time: frame time for frame-rate independent movement
    camera.translation.smooth_nudge(
        &target_position, 
        CAMERA_DECAY_RATE, 
        time.delta_secs()
    );
}

/// Handles player movement based on keyboard input
/// 
/// This system reads WASD keys and moves the player accordingly.
/// The camera system will then automatically follow the player's new position.
/// 
/// Note: This approach is for demonstration purposes. For production games,
/// consider more robust solutions like those in `examples/movement/physics_in_fixed_timestep.rs`.
fn move_player(
    mut player: Single<&mut Transform, With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    // START WITH NO MOVEMENT
    // Vec2::ZERO is equivalent to Vec2::new(0.0, 0.0)
    let mut direction = Vec2::ZERO;

    // CHECK EACH MOVEMENT KEY
    // We accumulate direction instead of setting it to allow diagonal movement
    // Multiple keys can be pressed simultaneously for combined movement
    
    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;  // Move up (positive Y)
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;  // Move down (negative Y)
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;  // Move left (negative X)
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;  // Move right (positive X)
    }

    // CALCULATE MOVEMENT DELTA
    // normalize_or_zero() ensures diagonal movement isn't faster than cardinal movement
    // Without normalization: diagonal = sqrt(1² + 1²) = 1.414 (faster than intended)
    // With normalization: all movement vectors have magnitude 1.0 (consistent speed)
    let normalized_direction = direction.normalize_or_zero();
    
    // Scale by movement speed and frame time for frame-rate independent movement
    let move_delta = normalized_direction * PLAYER_SPEED * time.delta_secs();
    
    // APPLY MOVEMENT
    // extend(0.) converts Vec2 to Vec3 by adding a Z component of 0
    // This is necessary because Transform.translation is a Vec3
    player.translation += move_delta.extend(0.);
    
    // COORDINATE SYSTEM NOTE:
    // In Bevy's default 2D coordinate system:
    // - X increases to the right
    // - Y increases upward  
    // - Z increases toward the camera (out of the screen)
    // This matches standard mathematical conventions
}
