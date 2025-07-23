//! Shows how to create a 3D orthographic view (for isometric-look games or CAD applications).
//!
//! # Orthographic vs Perspective Projection
//!
//! There are two main ways to project 3D scenes onto a 2D screen:
//!
//! ## Perspective Projection (Default)
//! - Objects get smaller as they get farther away
//! - Parallel lines converge at vanishing points
//! - Looks like human vision or a camera photo
//! - Used in most 3D games (FPS, racing, etc.)
//!
//! ## Orthographic Projection
//! - Objects stay the same size regardless of distance
//! - Parallel lines remain parallel
//! - No perspective distortion
//! - Used in:
//!   - Isometric games (SimCity, Civilization, many strategy games)
//!   - CAD/engineering applications
//!   - Architecture visualization
//!   - Technical drawings
//!
//! # Why Use Orthographic?
//!
//! 1. **Accurate measurements**: Can measure distances on screen
//! 2. **Clean aesthetics**: Popular retro/indie game look
//! 3. **Easier gameplay**: No perspective distortion to confuse players
//! 4. **Technical clarity**: Perfect for showing how things fit together

use bevy::{
    prelude::*,
    render::camera::ScalingMode, // Controls how the orthographic view scales
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with orthographic projection
    commands.spawn((
        Camera3d::default(),
        // Convert from the default perspective to orthographic projection
        Projection::from(OrthographicProjection {
            // Define how many world units fit in the viewport
            // FixedVertical means we specify the height in world units
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0, // 6 world units tall viewport
            },
            // Other scaling modes available:
            // - FixedHorizontal: Specify width instead
            // - WindowSize: Use window pixels directly
            // - AutoMin/AutoMax: Fit content automatically
            
            // Use 3D defaults (near/far planes appropriate for 3D)
            ..OrthographicProjection::default_3d()
        }),
        // Classic isometric camera angle
        // Position at equal distances on each axis for symmetric view
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))), // Green grass color
    ));
    
    // Four cubes arranged in a grid pattern
    // Notice with orthographic projection:
    // - All cubes appear the same size
    // - Grid alignment is perfectly preserved
    // - No convergence of parallel edges
    
    // Reuse the same mesh and material for efficiency
    let cube_mesh = meshes.add(Cuboid::default());
    let cube_material = materials.add(Color::srgb(0.8, 0.7, 0.6)); // Beige/sand color
    
    // Top-right cube
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(1.5, 0.5, 1.5),
    ));
    
    // Bottom-right cube
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(1.5, 0.5, -1.5),
    ));
    
    // Top-left cube
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(-1.5, 0.5, 1.5),
    ));
    
    // Bottom-left cube
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(-1.5, 0.5, -1.5),
    ));
    
    // Light source
    // Even with orthographic projection, we still need lighting!
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));
}
