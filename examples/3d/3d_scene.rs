//! A simple 3D scene with light shining over a cube sitting on a plane.
//!
//! Welcome to the third dimension! This is the "Hello World" of 3D graphics.
//! We're creating a classic scene: a cube on a surface with lighting.
//! It's like setting up a photography studio - you need objects, lights, and a camera.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

/// Set up a simple 3D scene
/// 
/// In 3D, we need four fundamental elements:
/// 1. Objects (meshes) - the "stuff" in our world
/// 2. Materials - how surfaces interact with light
/// 3. Lights - without light, everything is black!
/// 4. Camera - our viewpoint into this world
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // StandardMaterial is Bevy's physically-based rendering (PBR) material
    // PBR simulates how real materials interact with light - metal, plastic, etc.
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // OBJECT 1: Circular base (our "ground")
    // We use a circle, but rotate it to lie flat like a floor
    commands.spawn((
        // Mesh3d is the 3D equivalent of Mesh2d - it marks this for 3D rendering
        Mesh3d(meshes.add(Circle::new(4.0))), // 4 unit radius circle
        // MeshMaterial3d pairs with StandardMaterial for 3D rendering
        MeshMaterial3d(materials.add(Color::WHITE)),
        // Here's the key: we rotate the circle 90 degrees around X axis
        // Why? Circles are created in the XY plane (facing forward)
        // We want it in the XZ plane (facing up) to be a floor
        // -PI/2 radians = -90 degrees rotation around X axis
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    
    // OBJECT 2: The cube - our main subject
    commands.spawn((
        // Cuboid::new(width, height, depth) - all 1.0 makes a unit cube
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        // srgb_u8 takes RGB values from 0-255 (like web colors)
        // This is a nice periwinkle blue: R=124, G=144, B=255
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        // Position the cube at Y=0.5 so its bottom touches the ground at Y=0
        // Remember: the cube is 1 unit tall, centered at its origin
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    
    // LIGHT: Without this, our scene would be completely black!
    commands.spawn((
        PointLight {
            // Shadows make everything look more realistic
            // They're expensive to compute but worth it for most scenes
            shadows_enabled: true,
            // ..default() fills in standard values for intensity, range, etc.
            ..default()
        },
        // Position the light up and to the side
        // Think of it as a lamp on a tall stand
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    // CAMERA: Our eye into this world
    commands.spawn((
        // Camera3d::default() gives us a perspective camera with sensible settings
        Camera3d::default(),
        // This is a builder pattern for positioning AND rotating the camera:
        // 1. from_xyz(-2.5, 4.5, 9.0) positions the camera
        // 2. looking_at(Vec3::ZERO, Vec3::Y) rotates it to look at origin
        //    - First arg: what to look at (origin)
        //    - Second arg: which way is "up" (positive Y)
        // The result: a nice 3/4 view of our scene, like a movie shot
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
