//! This example demonstrates how to use the `Camera::viewport_to_world` method.
//!
//! 🎯 The Big Picture - Viewport to World Transformation
//! Imagine you're pointing a laser pointer through a window. The window (viewport) is
//! your 2D screen, and the laser beam extends into the 3D world behind it. This example
//! shows how to convert your mouse cursor position (2D screen coordinates) into a ray
//! that shoots through the 3D world, and then find where that ray hits a ground plane.
//!
//! 📚 Core Concepts:
//! - Viewport: The 2D window through which we see the 3D world (your screen)
//! - Ray: An infinite line starting from a point and going in a direction
//! - Ray-Plane Intersection: Finding where a ray hits a flat surface
//!
//! 🎮 What You'll See:
//! A white circle follows your mouse cursor on a green ground plane, showing
//! exactly where your cursor is "pointing" in 3D space.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .run();
}

// 🎯 The Magic: Converting 2D Mouse Position to 3D World Position
//
// This function runs every frame and performs a fascinating transformation:
// taking your flat, 2D mouse coordinates and figuring out where they point
// in the 3D world. It's like asking "if I could shoot a laser from my eye
// through my mouse cursor, where would it hit the ground?"
fn draw_cursor(
    // 📷 We need the camera to know where we're looking from
    camera_query: Single<(&Camera, &GlobalTransform)>,
    // 🌍 We need the ground plane to know what we're hitting
    ground: Single<&GlobalTransform, With<Ground>>,
    // 🪟 We need the window to get mouse position
    windows: Query<&Window>,
    // ✏️ Gizmos let us draw debug shapes
    mut gizmos: Gizmos,
) {
    // 🔍 Pattern: Early Return on Missing Data
    // If we can't find the window, there's nothing to do
    let Ok(windows) = windows.single() else {
        return;
    };

    // 📦 Unpack our camera data
    let (camera, camera_transform) = *camera_query;

    // 🖱️ Get Mouse Position
    // cursor_position() returns None if the cursor is outside the window
    let Some(cursor_position) = windows.cursor_position() else {
        return;
    };

    // 🌟 The Core Magic: Viewport to World Ray
    //
    // This is where 2D becomes 3D! The camera takes our 2D cursor position
    // and creates a ray that shoots from the camera through that screen point
    // into the 3D world. Think of it like this:
    //
    // 1. Your eye is at the camera position
    // 2. Your screen is a window in front of you
    // 3. Your mouse cursor is a point on that window
    // 4. This creates a ray from your eye through the cursor point
    //
    // The ray has:
    // - origin: where it starts (near the camera)
    // - direction: which way it's pointing (normalized vector)
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // 📐 Ray-Plane Intersection Mathematics
    //
    // Now we need to find where our ray hits the ground. This is classic
    // 3D geometry: finding the intersection of a line and a plane.
    //
    // The ground plane is defined by:
    // - A point on the plane (ground.translation())
    // - A normal vector (ground.up()) - perpendicular to the plane
    //
    // The intersection calculation returns the distance along the ray
    // where it hits the plane, or None if they're parallel
    let Some(distance) =
        ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    else {
        return;
    };
    
    // 📍 Calculate the actual 3D point where the ray hits
    // ray.get_point(distance) = ray.origin + ray.direction * distance
    let point = ray.get_point(distance);

    // 🎨 Visual Feedback: Draw a Circle at the Hit Point
    //
    // We draw the circle slightly above the ground (+ 0.01) to prevent
    // z-fighting (flickering when two surfaces are at the exact same position)
    gizmos.circle(
        // 🔄 Isometry3d combines position and rotation
        Isometry3d::new(
            // Position: hit point, raised slightly above ground
            point + ground.up() * 0.01,
            // Rotation: align circle to lie flat on the ground
            // We rotate from Z-up (circle's default) to match ground's up vector
            Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
        ),
        0.2,  // radius
        Color::WHITE,
    );
}

// 🏷️ Marker component to identify which entity is the ground
#[derive(Component)]
struct Ground;

// 🏗️ Scene Setup: Creating Our 3D World
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 🌍 Create the Ground Plane
    // This is what we'll be projecting our mouse cursor onto
    commands.spawn((
        // 📐 Geometry: A flat plane, 20x20 units
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        // 🎨 Appearance: A pleasant green color
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        // 🏷️ Tag it as the ground so we can find it later
        Ground,
    ));

    // ☀️ Add Lighting
    // Without light, everything would be black!
    commands.spawn((
        DirectionalLight::default(),
        // 🔄 Position the light and aim it at the origin
        // Think of this as the sun shining down at an angle
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 📷 Position the Camera
    // We place it up and to the side for a nice perspective view
    commands.spawn((
        Camera3d::default(),
        // 📍 Position at (15, 5, 15) looking at the origin
        // This gives us a nice diagonal view of the ground plane
        Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// 🎓 Deep Dive: Understanding Ray-Plane Intersection
//
// The mathematics behind ray-plane intersection is elegant:
//
// Ray equation: P = O + t*D
// - P: point on ray
// - O: ray origin
// - D: ray direction (normalized)
// - t: distance parameter
//
// Plane equation: (P - P0) · N = 0
// - P: point on plane
// - P0: known point on plane
// - N: plane normal
//
// Solving for t gives us the distance to intersection.
// If t is negative, the intersection is behind the ray origin.
// If there's no solution, the ray is parallel to the plane.

// 💡 Common Use Cases for Viewport-to-World:
// 1. Mouse picking (selecting 3D objects with mouse)
// 2. Placing objects in 3D space with mouse
// 3. First-person shooter aiming
// 4. RTS-style unit selection
// 5. 3D painting/sculpting applications