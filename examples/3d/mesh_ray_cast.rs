//! Demonstrates how to use the [`MeshRayCast`] system parameter to chain multiple ray casts
//! and bounce off of surfaces.
//!
//! # What is Ray Casting?
//!
//! Ray casting is finding where a line (ray) intersects with objects in 3D space.
//! Think of it like shining a laser pointer and seeing where the dot appears.
//!
//! Common uses:
//! - **Mouse picking**: Click on 3D objects
//! - **Line of sight**: Can enemy see player?
//! - **Physics**: Bullet trajectories
//! - **Lighting**: Ray tracing for realistic shadows
//!
//! # Ray Bouncing (Reflection)
//!
//! When a ray hits a surface, it can bounce off like a ball or light beam.
//! The angle of reflection equals the angle of incidence - just like a mirror!
//!
//! This example shows:
//! 1. An automated bouncing laser that moves in a pattern
//! 2. Mouse-controlled rays that bounce when you click
//! 3. Visual feedback with gizmos showing the ray paths
//!
//! The rays bounce up to 64 times, creating complex patterns as they reflect
//! around the inside of a box made of planes.

use std::f32::consts::{FRAC_PI_2, PI}; // FRAC_PI_2 = π/2 (90 degrees)

use bevy::{
    color::palettes::css,       // CSS color constants (RED, GREEN, etc.)
    core_pipeline::{
        bloom::Bloom,           // Post-processing glow effect
        tonemapping::Tonemapping, // HDR to screen color mapping
    },
    math::vec3,                 // Convenience function for Vec3::new
    picking::backend::ray::RayMap, // Stores rays cast from mouse cursor
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, bouncing_raycast)
        // Black background makes the glowing rays more visible
        .insert_resource(ClearColor(Color::BLACK))
        .run();
}

// Maximum number of times a ray can bounce before stopping
// More bounces = more complex patterns but worse performance
const MAX_BOUNCES: usize = 64;

// Speed multiplier for the automated laser animation
// Lower values make the laser move more slowly
const LASER_SPEED: f32 = 0.03;

fn bouncing_raycast(
    // MeshRayCast is a system parameter that provides ray casting functionality
    mut ray_cast: MeshRayCast,
    // Gizmos for drawing debug visualizations
    mut gizmos: Gizmos,
    time: Res<Time>,
    // The ray map stores rays cast by the cursor (populated by picking system)
    ray_map: Res<RayMap>,
) {
    // Cast an automatically moving ray and bounce it off of surfaces
    // Create a complex motion pattern using trigonometric functions
    let t = ops::cos((time.elapsed_secs() - 4.0).max(0.0) * LASER_SPEED) * PI;
    // Ray origin moves in a 3D Lissajous curve pattern
    let ray_pos = Vec3::new(
        ops::sin(t),           // X: Simple sine wave
        ops::cos(3.0 * t) * 0.5, // Y: Higher frequency cosine, scaled down
        ops::cos(t)            // Z: Simple cosine wave
    ) * 0.5; // Scale everything to fit within the box
    
    // Ray direction points toward the center (negative of position)
    let ray_dir = Dir3::new(-ray_pos).unwrap();
    let ray = Ray3d::new(ray_pos, ray_dir);
    
    // Draw a white sphere at the ray origin
    gizmos.sphere(ray_pos, 0.1, Color::WHITE);
    // Bounce the red laser ray
    bounce_ray(ray, &mut ray_cast, &mut gizmos, Color::from(css::RED));

    // Cast a ray from the cursor and bounce it off of surfaces
    // The picking system automatically populates ray_map with cursor rays
    for (_, ray) in ray_map.iter() {
        bounce_ray(*ray, &mut ray_cast, &mut gizmos, Color::from(css::GREEN));
    }
}

// Bounces a ray off of surfaces `MAX_BOUNCES` times.
fn bounce_ray(mut ray: Ray3d, ray_cast: &mut MeshRayCast, gizmos: &mut Gizmos, color: Color) {
    // Pre-allocate vector for performance
    let mut intersections = Vec::with_capacity(MAX_BOUNCES + 1);
    // Start with the ray origin (very bright red for visibility)
    intersections.push((ray.origin, Color::srgb(30.0, 0.0, 0.0)));

    for i in 0..MAX_BOUNCES {
        // Cast the ray and get the first hit
        // cast_ray returns a vector of (Entity, RayMeshHit) sorted by distance
        let Some((_, hit)) = ray_cast
            .cast_ray(ray, &MeshRayCastSettings::default())
            .first()
        else {
            // No hit - ray escaped or missed everything
            break;
        };

        // Calculate brightness based on bounce number
        // Earlier bounces are brighter (creates a fading effect)
        let brightness = 1.0 + 10.0 * (1.0 - i as f32 / MAX_BOUNCES as f32);
        
        // Add intersection point to our line strip
        intersections.push((hit.point, Color::BLACK.mix(&color, brightness)));
        // Draw a small sphere at each bounce point
        gizmos.sphere(hit.point, 0.005, Color::BLACK.mix(&color, brightness * 2.0));

        // Reflect the ray off of the surface
        // The reflect() method implements the law of reflection:
        // reflected = incident - 2 * (incident · normal) * normal
        ray.direction = Dir3::new(ray.direction.reflect(hit.normal)).unwrap();
        // Move origin slightly away from surface to prevent self-intersection
        // 1e-6 = 0.000001 units (very small offset)
        ray.origin = hit.point + ray.direction * 1e-6;
    }
    // Draw all the connected line segments with gradient colors
    gizmos.linestrip_gradient(intersections);
}

// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Make a box of planes facing inward so the laser gets trapped inside
    let plane_mesh = meshes.add(Plane3d::default());
    // Very transparent gray material (alpha = 0.01) so we can see through
    let plane_material = materials.add(Color::from(css::GRAY).with_alpha(0.01));
    
    // Helper closure to create a plane with position and rotation
    let create_plane = move |translation, rotation| {
        (
            Transform::from_translation(translation)
                // from_scaled_axis creates rotation from axis * angle
                .with_rotation(Quat::from_scaled_axis(rotation)),
            Mesh3d(plane_mesh.clone()),
            MeshMaterial3d(plane_material.clone()),
        )
    };

    // Create six planes to form a box
    // Top plane (rotated 180° around X to face down)
    commands.spawn(create_plane(vec3(0.0, 0.5, 0.0), Vec3::X * PI));
    // Bottom plane (no rotation, already faces up)
    commands.spawn(create_plane(vec3(0.0, -0.5, 0.0), Vec3::ZERO));
    // Right plane (rotated 90° around Z to face left)
    commands.spawn(create_plane(vec3(0.5, 0.0, 0.0), Vec3::Z * FRAC_PI_2));
    // Left plane (rotated -90° around Z to face right)
    commands.spawn(create_plane(vec3(-0.5, 0.0, 0.0), Vec3::Z * -FRAC_PI_2));
    // Back plane (rotated -90° around X to face forward)
    commands.spawn(create_plane(vec3(0.0, 0.0, 0.5), Vec3::X * -FRAC_PI_2));
    // Front plane (rotated 90° around X to face backward)
    commands.spawn(create_plane(vec3(0.0, 0.0, -0.5), Vec3::X * FRAC_PI_2));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        // Slight angle for more interesting lighting
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.1, 0.2, 0.0)),
    ));

    // Camera
    commands.spawn((
        Camera3d::default(),
        // Position outside the box looking at the center
        Transform::from_xyz(1.5, 1.5, 1.5).looking_at(Vec3::ZERO, Vec3::Y),
        // TonyMcMapface tonemapping for nice HDR rendering
        Tonemapping::TonyMcMapface,
        // Bloom makes bright objects glow - perfect for laser effects!
        Bloom::default(),
    ));
}
