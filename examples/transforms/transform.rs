//! Shows multiple transformations of objects.
//!
//! Transforms are the foundation of positioning in 3D space - they combine position, rotation,
//! and scale into one powerful system! Think of transforms like the controls on a robot arm:
//! you can move it to any position (translation), twist it in any direction (rotation), and
//! make it bigger or smaller (scale). This example creates a mini solar system where a cube
//! orbits around a sphere that shrinks as the cube travels - demonstrating all three transform
//! components working together in harmony!

use std::f32::consts::PI;

use bevy::{color::palettes::basic::YELLOW, prelude::*};

// ORBITAL CUBE DATA - Track the cube's journey around the sphere
// A struct for additional data of for a moving cube.
#[derive(Component)]
struct CubeState {
    start_pos: Vec3,    // Remember where we started (for distance calculation)
    move_speed: f32,    // How fast to move forward
    turn_speed: f32,    // How sharply to turn (orbit curvature)
}

// SHRINKING SPHERE DATA - Controls for the central object
// A struct adding information to a scalable entity,
// that will be stationary at the center of the scene.
#[derive(Component)]
struct Center {
    max_size: f32,      // Starting size
    min_size: f32,      // Smallest allowed size
    scale_factor: f32,  // How fast to shrink per unit of cube travel
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                move_cube,
                rotate_cube,
                scale_down_sphere_proportional_to_cube_travel_distance,
            )
                .chain(),
        )
        .run();
}

// THE STAGE SETUP - Create our mini solar system
// Startup system to setup the scene and spawn all relevant entities.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // THE SHRINKING SUN - Central sphere that scales down
    // Add an object (sphere) for visualizing scaling.
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(3.0).mesh().ico(32).unwrap())),  // High-detail sphere
        MeshMaterial3d(materials.add(Color::from(YELLOW))),
        Transform::from_translation(Vec3::ZERO),  // Center of the world
        Center {
            max_size: 1.0,      // Full size at start
            min_size: 0.1,      // Won't shrink below 10%
            scale_factor: 0.05, // Shrink rate
        },
    ));

    // THE ORBITING PLANET - Cube that circles the sphere
    // Add the cube to visualize rotation and translation.
    // This cube will circle around the center_sphere
    // by changing its rotation each frame and moving forward.
    // Define a start transform for an orbiting cube, that's away from our central object (sphere)
    // and rotate it so it will be able to move around the sphere and not towards it.
    let cube_spawn =
        Transform::from_translation(Vec3::Z * -10.0)  // Start 10 units back
            .with_rotation(Quat::from_rotation_y(PI / 2.));  // Face sideways to orbit
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::WHITE)),
        cube_spawn,
        CubeState {
            start_pos: cube_spawn.translation,
            move_speed: 2.0,    // Forward velocity
            turn_speed: 0.2,    // How tight the orbit is
        },
    ));

    // THE OBSERVATORY - Camera positioned to see the action
    // Spawn a camera looking at the entities to show what's happening in this example.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),  // Elevated view
    ));

    // STAGE LIGHTING - Illuminate our actors
    // Add a light source for better 3d visibility.
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(3.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// TRANSLATION SYSTEM - Move the cube forward in its current direction
// This system will move the cube forward.
fn move_cube(mut cubes: Query<(&mut Transform, &mut CubeState)>, timer: Res<Time>) {
    for (mut transform, cube) in &mut cubes {
        // FORWARD MOVEMENT - Like a car driving straight
        // Move the cube forward smoothly at a given move_speed.
        let forward = transform.forward();  // Which way are we facing?
        transform.translation += forward * cube.move_speed * timer.delta_secs();
        // Frame-independent movement: speed * time = distance
    }
}

// ROTATION SYSTEM - Turn the cube to create orbital motion
// This system will rotate the cube slightly towards the center_sphere.
// Due to the forward movement the resulting movement
// will be a circular motion around the center_sphere.
fn rotate_cube(
    mut cubes: Query<(&mut Transform, &mut CubeState), Without<Center>>,
    center_spheres: Query<&Transform, With<Center>>,
    timer: Res<Time>,
) {
    // FIND THE ORBIT CENTER - What are we circling around?
    // Calculate the point to circle around. (The position of the center_sphere)
    let mut center: Vec3 = Vec3::ZERO;
    for sphere in &center_spheres {
        center += sphere.translation;
    }
    
    // GRADUAL TURNING - Smoothly curve toward the center
    // Update the rotation of the cube(s).
    for (mut transform, cube) in &mut cubes {
        // WHERE SHOULD WE FACE? - Calculate ideal rotation to look at center
        // Calculate the rotation of the cube if it would be looking at the sphere in the center.
        let look_at_sphere = transform.looking_at(center, *transform.local_y());
        
        // SMOOTH INTERPOLATION - Don't snap, gradually turn
        // Interpolate between the current rotation and the fully turned rotation
        // when looking at the sphere, with a given turn speed to get a smooth motion.
        // With higher speed the curvature of the orbit would be smaller.
        let incremental_turn_weight = cube.turn_speed * timer.delta_secs();
        let old_rotation = transform.rotation;
        // Lerp = Linear Interpolation - blend between two rotations
        transform.rotation = old_rotation.lerp(look_at_sphere.rotation, incremental_turn_weight);
    }
}

// SCALE SYSTEM - Shrink the sphere based on cube's travel distance
// This system will scale down the sphere in the center of the scene
// according to the traveling distance of the orbiting cube(s) from their start position(s).
fn scale_down_sphere_proportional_to_cube_travel_distance(
    cubes: Query<(&Transform, &CubeState), Without<Center>>,
    mut centers: Query<(&mut Transform, &Center)>,
) {
    // MEASURE THE JOURNEY - How far has the cube traveled?
    // First we need to calculate the length of between
    // the current position of the orbiting cube and the spawn position.
    let mut distances = 0.0;
    for (cube_transform, cube_state) in &cubes {
        // Vector subtraction gives us the displacement
        distances += (cube_state.start_pos - cube_transform.translation).length();
    }
    
    // APPLY SHRINKAGE - Make sphere smaller based on distance
    // Now we use the calculated value to scale the sphere in the center accordingly.
    for (mut transform, center) in &mut centers {
        // SHRINKING FORMULA - Start big, get smaller with distance
        // Calculate the new size from the calculated distances and the centers scale_factor.
        // Since we want to have the sphere at its max_size at the cubes spawn location we start by
        // using the max_size as start value and subtract the distances scaled by a scaling factor.
        let mut new_size: f32 = center.max_size - center.scale_factor * distances;

        // MINIMUM SIZE LIMIT - Don't disappear completely!
        // The new size should also not be smaller than the centers min_size.
        // Therefore the max value out of (new_size, center.min_size) is used.
        new_size = new_size.max(center.min_size);

        // UNIFORM SCALING - Shrink equally in all dimensions
        // Now scale the sphere uniformly in all directions using new_size.
        // Here Vec3:splat is used to create a vector with new_size in x, y and z direction.
        transform.scale = Vec3::splat(new_size);  // (new_size, new_size, new_size)
    }
}
