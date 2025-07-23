//! This example demonstrates how to use interpolation to make one entity smoothly follow another.
//!
//! Smooth following is like the difference between a robotic arm and a human arm reaching for
//! an object. The robot snaps directly to the target position, while the human arm smoothly
//! accelerates and decelerates in a natural curve. We use exponential decay interpolation to
//! create this organic movement - the follower moves quickly when far from target, then slows
//! down as it approaches, just like a spring settling into position!

use bevy::{
    math::{prelude::*, vec3, NormedVectorSpace},
    prelude::*,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // SYSTEM ORDERING - Target moves first, then follower reacts
        .add_systems(Update, (move_target, move_follower).chain())
        .run();
}

// ACTOR IDENTIFICATION - Marker components for our two spheres
// The sphere that the following sphere targets at all times:
#[derive(Component)]
struct TargetSphere;      // The leader - moves randomly

// MOVEMENT PARAMETERS - Control how the motion behaves
// The speed of the target sphere moving to its next location:
#[derive(Resource)]
struct TargetSphereSpeed(f32);  // Units per second

// The position that the target sphere always moves linearly toward:
#[derive(Resource)]
struct TargetPosition(Vec3);    // Next waypoint for target

// INTERPOLATION CONTROL - How responsive the follower is
// The decay rate used by the smooth following:
#[derive(Resource)]
struct DecayRate(f32);          // Higher = more responsive (like spring stiffness)

// The sphere that follows the target sphere by moving towards it with nudging:
#[derive(Component)]
struct FollowingSphere;   // The follower - smoothly chases target

/// DETERMINISTIC RANDOMNESS - Seeded for consistent behavior
/// The source of randomness used by this example.
#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // STAGE SETUP - Create the environment
    // A plane (the floor):
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.15, 0.3))),  // Dark purple
        Transform::from_xyz(0.0, -2.5, 0.0),
    ));

    // THE LEADER - Blue sphere that moves randomly
    // The target sphere:
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.15, 0.9))),  // Blue
        TargetSphere,   // Marker component
    ));

    // THE FOLLOWER - Red sphere that smoothly chases the blue one
    // The sphere that follows it:
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.3))),
        MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.3))),  // Red
        Transform::from_translation(vec3(0.0, -2.0, 0.0)),          // Start below plane
        FollowingSphere, // Marker component
    ));

    // LIGHTING SETUP - Dramatic shadows to show movement
    // A light:
    commands.spawn((
        PointLight {
            intensity: 15_000_000.0,  // Bright enough to cast clear shadows
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),  // Above and to the side
    ));

    // CAMERA POSITIONING - Good view of the action
    // A camera:
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),  // Angled view
    ));

    // TUNING PARAMETERS - Adjust these to change behavior!
    // Set starting values for resources used by the systems:
    commands.insert_resource(TargetSphereSpeed(5.0));     // How fast target moves
    commands.insert_resource(DecayRate(2.0));             // How quickly follower catches up
    commands.insert_resource(TargetPosition(Vec3::ZERO)); // First waypoint
    commands.insert_resource(RandomSource(ChaCha8Rng::seed_from_u64(68941654987813521))); // Seed for reproducible randomness
}

fn move_target(
    mut target: Single<&mut Transform, With<TargetSphere>>,
    target_speed: Res<TargetSphereSpeed>,
    mut target_pos: ResMut<TargetPosition>,
    time: Res<Time>,
    mut rng: ResMut<RandomSource>,
) {
    // WAYPOINT NAVIGATION - Move toward current target, then pick a new one
    match Dir3::new(target_pos.0 - target.translation) {
        // STILL TRAVELING - We have a clear direction to move
        // The target and the present position of the target sphere are far enough to have a well-
        // defined direction between them, so let's move closer:
        Ok(dir) => {
            let delta_time = time.delta_secs();
            let abs_delta = (target_pos.0 - target.translation).norm();  // Distance remaining

            // FRAME-RATE INDEPENDENT MOVEMENT - Don't overshoot on slow frames
            // Avoid overshooting in case of high values of `delta_time`:
            let magnitude = f32::min(abs_delta, delta_time * target_speed.0);
            target.translation += dir * magnitude;  // Move toward waypoint
        }

        // DESTINATION REACHED - Pick a new random target
        // The two are really close, so let's generate a new target position:
        Err(_) => {
            let legal_region = Cuboid::from_size(Vec3::splat(4.0));  // 4x4x4 cube
            *target_pos = TargetPosition(legal_region.sample_interior(&mut rng.0));
        }
    }
}

fn move_follower(
    mut following: Single<&mut Transform, With<FollowingSphere>>,
    target: Single<&Transform, (With<TargetSphere>, Without<FollowingSphere>)>,
    decay_rate: Res<DecayRate>,
    time: Res<Time>,
) {
    let decay_rate = decay_rate.0;
    let delta_time = time.delta_secs();

    // THE MAGIC INTERPOLATION - One line creates smooth, natural movement!
    // Calling `smooth_nudge` is what moves the following sphere smoothly toward the target.
    // This implements exponential decay: fast when far apart, slow when close
    // Think of it like a rubber band pulling the follower toward the target
    following
        .translation
        .smooth_nudge(&target.translation, decay_rate, delta_time);
}
