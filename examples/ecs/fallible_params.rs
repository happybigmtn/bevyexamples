//! # Fallible System Parameters in Bevy ECS
//!
//! This example demonstrates how certain system parameters can "fail" and prevent
//! their systems from running when specific conditions aren't met. This is different
//! from runtime errors - these are validation checks that happen before the system runs.
//!
//! ## What Are Fallible Parameters?
//!
//! Fallible parameters are system parameters that have preconditions. If these
//! preconditions aren't met, the system won't run at all. This is useful for:
//! - Systems that only make sense when certain entities exist
//! - Preventing crashes from missing resources
//! - Creating conditional system behavior based on world state
//!
//! ## Types of Fallible Parameters
//!
//! 1. **Resource Parameters** ([`Res<R>`], [`ResMut<R>`])
//!    - Requirement: The resource must exist in the world
//!    - Failure behavior: Calls the error handler (default: panic)
//!
//! 2. **Single Query** ([`Single<D, F>`])
//!    - Requirement: EXACTLY one entity must match the query
//!    - Failure behavior: System is silently skipped
//!    - Use case: Systems that operate on a unique entity (like the player)
//!
//! 3. **Optional Single** ([`Option<Single<D, F>>`])
//!    - Requirement: Zero or one entity must match (not more)
//!    - Failure behavior: System runs with None if 0 or 2+ entities match
//!    - Use case: Systems that can handle missing entities gracefully
//!
//! 4. **Populated Query** ([`Populated<D, F>`])
//!    - Requirement: At least one entity must match the query
//!    - Failure behavior: System is silently skipped
//!    - Use case: Systems that need to process a group of entities
//!
//! ## Non-Fallible Parameters
//!
//! Regular [`Query`] parameters never fail validation. An empty query (no matching
//! entities) is perfectly valid and the system will run with an empty iterator.
//!
//! ## How Validation Works
//!
//! Before each system runs, Bevy validates all its parameters. If any parameter
//! fails validation, the system is handled according to that parameter's error type.
//! This happens every frame, so systems can start/stop running as entities are
//! added/removed from the world.
//!
//! [`SystemParamValidationError`]: bevy::ecs::system::SystemParamValidationError
//! [`SystemParam::validate_param`]: bevy::ecs::system::SystemParam::validate_param
//! [`default_error_handler`]: bevy::ecs::error::default_error_handler

use bevy::ecs::error::warn;
use bevy::prelude::*;
// For generating random enemy positions and movement
use rand::Rng;

fn main() {
    // Print instructions for the interactive demo
    println!();
    println!("Press 'A' to add enemy ships and 'R' to remove them.");
    println!("Player ship will wait for enemy ships and track one if it exists,");
    println!("but will stop tracking if there are more than one.");
    println!();

    App::new()
        // === Error Handler Configuration ===
        // By default, if a parameter fails to be fetched (like a missing resource),
        // Bevy will panic and crash. Here we set it to just warn instead.
        // This is important for fallible parameters like Res<T> and ResMut<T>.
        .set_error_handler(warn)
        
        .add_plugins(DefaultPlugins)
        
        // Setup system spawns the player
        .add_systems(Startup, setup)
        
        // === Main Game Systems ===
        // Chain ensures these run in order:
        // 1. user_input: Handle keyboard input to add/remove enemies
        // 2. move_targets: Move enemies in circles (only runs if enemies exist)
        // 3. track_targets: Player tracks a single enemy (behavior changes based on enemy count)
        .add_systems(Update, (user_input, move_targets, track_targets).chain())
        
        // === Demonstration of Always-Failing Validation ===
        // This system will NEVER run because no entity can be both Player AND Enemy.
        // It demonstrates how Single<> validation prevents impossible queries.
        .add_systems(Update, do_nothing_fail_validation)
        
        .run();
}

/// Enemy component stores data for circular movement pattern.
/// Enemies orbit around a fixed point in space.
#[derive(Component, Default)]
struct Enemy {
    /// Center point of the circular path
    origin: Vec2,
    /// Distance from origin to enemy position
    radius: f32,
    /// Current angle in radians (0 to 2π)
    rotation: f32,
    /// How fast the enemy orbits (radians per second)
    rotation_speed: f32,
}

/// Player component stores data for tracking behavior.
/// The player ship will pursue enemies when they exist.
#[derive(Component, Default)]
struct Player {
    /// Movement speed in units per second
    speed: f32,
    /// Rotation speed when searching (radians per second)
    rotation_speed: f32,
    /// Stop this far away from the target
    min_follow_radius: f32,
}

/// Startup system that creates the initial game entities.
/// This uses infallible parameters (Commands, Res) so it always runs.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn 2D camera
    commands.spawn(Camera2d);

    // Spawn the player ship at the origin
    let texture = asset_server.load("textures/simplespace/ship_C.png");
    commands.spawn((
        Player {
            speed: 100.0,             // Units per second
            rotation_speed: 2.0,      // Radians per second when searching
            min_follow_radius: 50.0,  // Maintain distance from target
        },
        Sprite {
            image: texture,
            // Blue tint to distinguish from enemies
            color: bevy::color::palettes::tailwind::BLUE_800.into(),
            ..Default::default()
        },
        // Start at world origin
        Transform::from_translation(Vec3::ZERO),
    ));
}

/// System that handles keyboard input for spawning/removing enemies.
/// 
/// This system uses only infallible parameters:
/// - Commands: Always available
/// - Query: Returns empty iterator if no enemies exist
/// - Res<ButtonInput>: Would panic if missing, but DefaultPlugins provides it
/// - Res<AssetServer>: Also provided by DefaultPlugins
fn user_input(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();
    
    // 'A' key: Add a new enemy with random parameters
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        let texture = asset_server.load("textures/simplespace/enemy_A.png");
        commands.spawn((
            Enemy {
                // Random center point for circular movement
                origin: Vec2::new(rng.gen_range(-200.0..200.0), rng.gen_range(-200.0..200.0)),
                // Random orbit radius
                radius: rng.gen_range(50.0..150.0),
                // Random starting angle (TAU = 2π radians = full circle)
                rotation: rng.gen_range(0.0..std::f32::consts::TAU),
                // Random orbit speed
                rotation_speed: rng.gen_range(0.5..1.5),
            },
            Sprite {
                image: texture,
                // Red tint for enemies
                color: bevy::color::palettes::tailwind::RED_800.into(),
                ..default()
            },
            // Will be positioned by move_targets system
            Transform::from_translation(Vec3::ZERO),
        ));
    }

    // 'R' key: Remove the first enemy (if any exist)
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Query.iter() returns empty iterator if no matches
        // This is NOT fallible - the system still runs
        if let Some(entity) = enemies.iter().next() {
            commands.entity(entity).despawn();
        }
    }
}

/// System that moves enemies in circular paths.
/// 
/// This system uses the `Populated` parameter, which is FALLIBLE:
/// - If NO enemies exist, this system is skipped entirely
/// - If one or more enemies exist, the system runs normally
/// 
/// This is more efficient than using a regular Query and checking if it's empty,
/// because Bevy doesn't even attempt to run the system when there are no enemies.
fn move_targets(
    // Populated<T> requires at least one matching entity
    // Note the &mut *enemies syntax to iterate over the Populated wrapper
    mut enemies: Populated<(&mut Transform, &mut Enemy)>,
    time: Res<Time>,
) {
    // This loop only executes if enemies exist (guaranteed by Populated)
    for (mut transform, mut target) in &mut *enemies {
        // Update rotation angle based on speed and delta time
        target.rotation += target.rotation_speed * time.delta_secs();
        
        // Apply rotation to face direction of movement
        transform.rotation = Quat::from_rotation_z(target.rotation);
        
        // Calculate position on circle:
        // - transform.right() gives us the local X axis (right direction)
        // - Multiply by radius to get offset from origin
        let offset = transform.right() * target.radius;
        
        // Position = origin + offset (extend adds Z=0)
        transform.translation = target.origin.extend(0.0) + offset;
    }
}

/// System that controls player movement and enemy tracking.
/// 
/// This demonstrates TWO different fallible parameter behaviors:
/// 
/// 1. `Single<(&mut Transform, &Player)>` - FALLIBLE
///    - Only runs if EXACTLY ONE entity has Player component
///    - If 0 or 2+ players exist, system is skipped entirely
///    - This ensures we only run for a unique player entity
/// 
/// 2. `Option<Single<&Transform, ...>>` - INFALLIBLE (wrapped in Option)
///    - System ALWAYS runs because Option makes it non-failing
///    - Some(enemy) if exactly one enemy exists
///    - None if 0 or 2+ enemies exist
///    - Allows different behavior based on enemy count
fn track_targets(
    // Single<T> fails validation if not exactly one match
    mut player: Single<(&mut Transform, &Player)>,
    // Option<Single<T>> never fails - just returns None for 0 or 2+ matches
    // The filter (With<Enemy>, Without<Player>) ensures we only look at enemies
    enemy: Option<Single<&Transform, (With<Enemy>, Without<Player>)>>,
    time: Res<Time>,
) {
    // Dereference Single to get the tuple inside
    let (player_transform, player) = &mut *player;
    
    if let Some(enemy_transform) = enemy {
        // === Exactly One Enemy: Track It ===
        
        // Calculate direction vector from player to enemy
        let delta = enemy_transform.translation - player_transform.translation;
        let distance = delta.length();
        
        // Normalize to get unit direction vector
        let front = delta / distance;
        
        // Build rotation matrix from direction
        // - front: forward direction (towards enemy)
        // - up: world up (positive Z)
        // - side: right direction (cross product)
        let up = Vec3::Z;
        let side = front.cross(up);
        player_transform.rotation = Quat::from_mat3(&Mat3::from_cols(side, front, up));
        
        // Move towards enemy, but stop at min_follow_radius
        let max_step = distance - player.min_follow_radius;
        if 0.0 < max_step {
            // Clamp movement to not overshoot the target
            let velocity = (player.speed * time.delta_secs()).min(max_step);
            player_transform.translation += front * velocity;
        }
    } else {
        // === Zero or Multiple Enemies: Search Mode ===
        // Rotate in place to look for enemies
        player_transform.rotate_axis(Dir3::Z, player.rotation_speed * time.delta_secs());
    }
}

/// Demonstration system that ALWAYS fails validation.
/// 
/// This system requires exactly one entity that is BOTH a Player AND an Enemy.
/// Since these are mutually exclusive components in our game design, this
/// condition can never be met.
/// 
/// Result: This system will never run, and Bevy will skip it every frame.
/// 
/// This demonstrates that fallible parameters provide compile-time system
/// definitions with runtime validation. The system compiles fine, but its
/// preconditions prevent it from ever executing.
fn do_nothing_fail_validation(
    // Single<> requires exactly one match
    // The filter (With<Player>, With<Enemy>) requires BOTH components
    _: Single<(), (With<Player>, With<Enemy>)>
) {
    // This code is unreachable - the system never runs!
}
