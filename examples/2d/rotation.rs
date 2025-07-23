//! Demonstrates rotating entities in 2D using quaternions.
//!
//! Rotations in 2D are like spinning a record on a turntable - there's only one axis to spin
//! around (the Z-axis pointing out of your screen)! But don't be fooled by the simplicity;
//! this example shows three different rotation strategies: keyboard-controlled spinning like
//! a steering wheel, instant snapping like a compass needle finding north, and smooth tracking
//! like a sunflower following the sun. These techniques are the foundation for everything from
//! top-down shooters to tower defense games!

use bevy::{math::ops, prelude::*};

const BOUNDS: Vec2 = Vec2::new(1200.0, 640.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                player_movement_system,
                snap_to_player_system,
                rotate_to_player_system,
            ),
        )
        .run();
}

/// Player component
// THE PILOT'S CONTROLS - Settings for how the ship moves and turns
#[derive(Component)]
struct Player {
    /// Linear speed in meters per second
    movement_speed: f32,  // Like the gas pedal - how fast we go forward
    /// Rotation speed in radians per second
    rotation_speed: f32,  // Like the steering wheel - how fast we can turn
}

/// Snap to player ship behavior
// INSTANT TRACKING - Like a magnet snapping to point at metal
#[derive(Component)]
struct SnapToPlayer;  // No speed limit - rotates instantly!

/// Rotate to face player ship behavior
// SMOOTH TRACKING - Like a security camera panning to follow movement
#[derive(Component)]
struct RotateToPlayer {
    /// Rotation speed in radians per second
    rotation_speed: f32,  // Maximum turning speed - prevents jarring spins
}

/// Add the game's entities to our world and creates an orthographic camera for 2D rendering.
///
/// THE 2D COORDINATE COMPASS - Understanding which way is "up" in Bevy!
/// The Bevy coordinate system is the same for 2D and 3D, in terms of 2D this means that:
///
/// * `X` axis goes from left to right (`+X` points right) - Like reading a book
/// * `Y` axis goes from bottom to top (`+Y` point up) - Like a rocket launching
/// * `Z` axis goes from far to near (`+Z` points towards you, out of the screen) - Like an arrow shot at you
///
/// The origin is at the center of the screen.
/// Think of it like standing at the center of a compass rose painted on the ground!
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let ship_handle = asset_server.load("textures/simplespace/ship_C.png");
    let enemy_a_handle = asset_server.load("textures/simplespace/enemy_A.png");
    let enemy_b_handle = asset_server.load("textures/simplespace/enemy_B.png");

    commands.spawn(Camera2d);

    // Create a minimal UI explaining how to interact with the example
    commands.spawn((
        Text::new("Up Arrow: Move Forward\nLeft / Right Arrow: Turn"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    let horizontal_margin = BOUNDS.x / 4.0;
    let vertical_margin = BOUNDS.y / 4.0;

    // THE HERO SHIP - Player's vessel at the center of the action
    // Player controlled ship
    commands.spawn((
        Sprite::from_image(ship_handle),
        Player {
            movement_speed: 500.0,                  // Meters per second - zippy!
            rotation_speed: f32::to_radians(360.0), // Full spin in 1 second
            // Converting degrees to radians is like converting miles to kilometers
            // Computers prefer radians (based on pi) over degrees (based on 360)
        },
    ));

    // SNAP-TRACKING ENEMIES - Instant lock-on like radar tracking
    // Enemy that snaps to face the player spawns on the bottom and left
    commands.spawn((
        Sprite::from_image(enemy_a_handle.clone()),
        Transform::from_xyz(0.0 - horizontal_margin, 0.0, 0.0),  // Left side
        SnapToPlayer,  // Will instantly face the player - no rotation animation!
    ));
    commands.spawn((
        Sprite::from_image(enemy_a_handle),
        Transform::from_xyz(0.0, 0.0 - vertical_margin, 0.0),  // Bottom side
        SnapToPlayer,
    ));

    // SMOOTH-TRACKING ENEMIES - Gradual rotation like a turret turning
    // Enemy that rotates to face the player enemy spawns on the top and right
    commands.spawn((
        Sprite::from_image(enemy_b_handle.clone()),
        Transform::from_xyz(0.0 + horizontal_margin, 0.0, 0.0),  // Right side
        RotateToPlayer {
            rotation_speed: f32::to_radians(45.0), // Slow turner - tactical
        },
    ));
    commands.spawn((
        Sprite::from_image(enemy_b_handle),
        Transform::from_xyz(0.0, 0.0 + vertical_margin, 0.0),  // Top side
        RotateToPlayer {
            rotation_speed: f32::to_radians(90.0), // Faster turner - aggressive
        },
    ));
}

/// Demonstrates applying rotation and movement based on keyboard input.
// THE PILOT'S SEAT - Turn keyboard presses into ship movement
fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Single<(&Player, &mut Transform)>,
) {
    let (ship, mut transform) = query.into_inner();

    // CONTROL INPUTS - Like a video game controller's state
    let mut rotation_factor = 0.0;  // -1 = turn left, 0 = no turn, 1 = turn right
    let mut movement_factor = 0.0;  // 0 = stop, 1 = full speed ahead

    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        rotation_factor += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        rotation_factor -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        movement_factor += 1.0;
    }

    // SPINNING THE SHIP - Like turning a steering wheel
    // Update the ship rotation around the Z axis (perpendicular to the 2D plane of the screen)
    transform.rotate_z(rotation_factor * ship.rotation_speed * time.delta_secs());
    // Z-axis rotation in 2D is like spinning a pinwheel - there's only one axis to spin around!

    // THRUST CALCULATION - Converting rotation into movement direction
    // Get the ship's forward vector by applying the current rotation to the ships initial facing
    // vector
    let movement_direction = transform.rotation * Vec3::Y;
    // This is like asking "which way is my nose pointing?" - we start facing +Y (up)
    // and the rotation tells us where we're facing now
    
    // Get the distance the ship will move based on direction, the ship's movement speed and delta
    // time
    let movement_distance = movement_factor * ship.movement_speed * time.delta_secs();
    // Distance = Speed × Time (basic physics!)
    
    // Create the change in translation using the new movement direction and distance
    let translation_delta = movement_direction * movement_distance;
    // This is our velocity vector - direction × speed
    
    // Update the ship translation with our new translation delta
    transform.translation += translation_delta;
    // Add the movement to our current position

    // INVISIBLE WALLS - Keep the ship in the play area
    // Bound the ship within the invisible level bounds
    let extents = Vec3::from((BOUNDS / 2.0, 0.0));
    transform.translation = transform.translation.min(extents).max(-extents);
    // This is like having invisible bumpers at a bowling alley - you can't go past them!
}

/// Demonstrates snapping the enemy ship to face the player ship immediately.
// THE COMPASS NEEDLE - Instant magnetic alignment to target
fn snap_to_player_system(
    mut query: Query<&mut Transform, (With<SnapToPlayer>, Without<Player>)>,
    player_transform: Single<&Transform, With<Player>>,
) {
    // Get the player translation in 2D
    let player_translation = player_transform.translation.xy();
    // Drop the Z coordinate - we only care about X,Y position on our 2D plane

    for mut enemy_transform in &mut query {
        // DIRECTION FINDING - Where is the player relative to me?
        // Get the vector from the enemy ship to the player ship in 2D and normalize it.
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();
        // Normalize makes the vector length 1.0 - we only care about direction, not distance
        // Like asking "which way?" not "how far?"

        // INSTANT ROTATION - Calculate the exact rotation needed
        // Get the quaternion to rotate from the initial enemy facing direction to the direction
        // facing the player
        let rotate_to_player = Quat::from_rotation_arc(Vec3::Y, to_player.extend(0.));
        // This calculates: "What rotation takes me from facing +Y to facing the player?"
        // from_rotation_arc is like a GPS recalculating - instant new direction!

        // SNAP TO TARGET - No animation, just instant alignment
        // Rotate the enemy to face the player
        enemy_transform.rotation = rotate_to_player;
        // BAM! Instantly facing the player - like a compass needle snapping to north
    }
}

/// Demonstrates rotating an enemy ship to face the player ship at a given rotation speed.
///
/// THE SMOOTH PURSUIT - Like a sunflower slowly turning to face the sun
/// This method uses the vector dot product to determine if the enemy is facing the player and
/// if not, which way to rotate to face the player. The dot product on two unit length vectors
/// will return a value between -1.0 and +1.0 which tells us the following about the two vectors:
///
/// THE DOT PRODUCT COMPASS - A mathematical way to measure alignment:
/// * If the result is 1.0 the vectors are pointing in the same direction, the angle between them is
///   0 degrees. (Like two arrows pointing the same way)
/// * If the result is 0.0 the vectors are perpendicular, the angle between them is 90 degrees.
///   (Like the hands of a clock at 3:00)
/// * If the result is -1.0 the vectors are parallel but pointing in opposite directions, the angle
///   between them is 180 degrees. (Like two people back-to-back)
/// * If the result is positive the vectors are pointing in roughly the same direction, the angle
///   between them is greater than 0 and less than 90 degrees. (Somewhat aligned)
/// * If the result is negative the vectors are pointing in roughly opposite directions, the angle
///   between them is greater than 90 and less than 180 degrees. (Mostly opposite)
///
/// It is possible to get the angle by taking the arc cosine (`acos`) of the dot product. It is
/// often unnecessary to do this though. Beware than `acos` will return `NaN` if the input is less
/// than -1.0 or greater than 1.0. This can happen even when working with unit vectors due to
/// floating point precision loss, so it pays to clamp your dot product value before calling
/// `acos`.
fn rotate_to_player_system(
    time: Res<Time>,
    mut query: Query<(&RotateToPlayer, &mut Transform), Without<Player>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    // Get the player translation in 2D
    let player_translation = player_transform.translation.xy();
    // We work in 2D, so we drop the Z coordinate

    for (config, mut enemy_transform) in &mut query {
        // CURRENT FACING - Which way is the enemy looking?
        // Get the enemy ship forward vector in 2D (already unit length)
        let enemy_forward = (enemy_transform.rotation * Vec3::Y).xy();
        // Ships start facing +Y, rotation tells us where they face now

        // TARGET DIRECTION - Which way should we be looking?
        // Get the vector from the enemy ship to the player ship in 2D and normalize it.
        let to_player = (player_translation - enemy_transform.translation.xy()).normalize();

        // ALIGNMENT CHECK - How well are we facing the target?
        // Get the dot product between the enemy forward vector and the direction to the player.
        let forward_dot_player = enemy_forward.dot(to_player);
        // Dot product tells us how "aligned" two directions are
        // 1.0 = perfectly aligned, 0.0 = perpendicular, -1.0 = opposite

        // EARLY EXIT - Already facing the right way?
        // If the dot product is approximately 1.0 then the enemy is already facing the player and
        // we can early out.
        if (forward_dot_player - 1.0).abs() < f32::EPSILON {
            continue;  // We're already looking at the player - nothing to do!
        }

        // TURN DIRECTION - Which way should we rotate?
        // Get the right vector of the enemy ship in 2D (already unit length)
        let enemy_right = (enemy_transform.rotation * Vec3::X).xy();
        // The "right" vector is perpendicular to forward - like your right hand when facing forward

        // CLOCKWISE OR COUNTER? - Use the right vector as a compass
        // Get the dot product of the enemy right vector and the direction to the player ship.
        // If the dot product is negative them we need to rotate counter clockwise, if it is
        // positive we need to rotate clockwise. Note that `copysign` will still return 1.0 if the
        // dot product is 0.0 (because the player is directly behind the enemy, so perpendicular
        // with the right vector).
        let right_dot_player = enemy_right.dot(to_player);
        // This trick tells us which way to turn:
        // Positive = player is to our right = turn clockwise
        // Negative = player is to our left = turn counter-clockwise

        // ROTATION DIRECTION - Account for Bevy's coordinate system
        // Determine the sign of rotation from the right dot player. We need to negate the sign
        // here as the 2D bevy co-ordinate system rotates around +Z, which is pointing out of the
        // screen. Due to the right hand rule, positive rotation around +Z is counter clockwise and
        // negative is clockwise.
        let rotation_sign = -f32::copysign(1.0, right_dot_player);
        // The right-hand rule: Point your thumb along +Z (toward you), your fingers curl
        // in the positive rotation direction (counter-clockwise when looking at the screen)

        // OVERSHOOT PREVENTION - Don't spin past the target!
        // Limit rotation so we don't overshoot the target. We need to convert our dot product to
        // an angle here so we can get an angle of rotation to clamp against.
        let max_angle = ops::acos(forward_dot_player.clamp(-1.0, 1.0)); // Clamp acos for safety
        // acos gives us the actual angle between the vectors
        // We clamp to prevent NaN from floating point errors

        // SMOOTH ROTATION - Turn at our speed limit, but not past the target
        // Calculate angle of rotation with limit
        let rotation_angle =
            rotation_sign * (config.rotation_speed * time.delta_secs()).min(max_angle);
        // This is like a car's steering: we can only turn so fast (rotation_speed),
        // but we also won't turn past our destination (max_angle)

        // APPLY THE TURN - Rotate toward the player
        // Rotate the enemy to face the player
        enemy_transform.rotate_z(rotation_angle);
        // Each frame we turn a little bit closer to facing the player
    }
}
