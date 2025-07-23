//! Illustrates how to create parent-child relationships between entities and how parent transforms
//! are propagated to their descendants.
//!
//! # Transform Hierarchies in Games
//!
//! Parent-child relationships are fundamental to game development!
//!
//! ## What is Transform Parenting?
//!
//! When entity B is a child of entity A:
//! - B's position is relative to A (not world space)
//! - When A moves, B moves with it
//! - When A rotates, B orbits around A
//! - When A scales, B scales too
//!
//! ## Real-World Examples
//!
//! 1. **Character with Equipment**
//!    - Parent: Character body
//!    - Children: Sword, shield, hat
//!    - Result: Equipment moves with character
//!
//! 2. **Vehicle with Passengers**
//!    - Parent: Car
//!    - Children: Driver, wheels
//!    - Result: Everything moves together
//!
//! 3. **Solar System**
//!    - Parent: Sun
//!    - Children: Planets
//!    - Grandchildren: Moons
//!    - Result: Orbital motion!
//!
//! 4. **Robot Arm**
//!    - Parent: Shoulder
//!    - Child: Upper arm
//!    - Grandchild: Forearm
//!    - Great-grandchild: Hand
//!    - Result: Realistic joint movement
//!
//! This example shows two cubes where the child orbits around the parent.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotator_system)
        .run();
}

/// This component indicates what entities should rotate
/// In this example, only the parent has this component
#[derive(Component)]
struct Rotator;

/// Rotates entities with the Rotator component
/// When we rotate the parent, the child automatically follows!
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in &mut query {
        // Rotate around X axis at 3 radians per second
        // This is about 172 degrees per second
        transform.rotate_x(3.0 * time.delta_secs());
        
        // Note: We're only rotating the parent entity
        // The child will orbit around the parent automatically
        // because its transform is relative to the parent
    }
}

/// Set up a simple scene with a "parent" cube and a "child" cube
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create shared resources for both cubes
    let cube_handle = meshes.add(Cuboid::new(2.0, 2.0, 2.0));
    let cube_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.7, 0.6), // Light brown/beige color
        ..default()
    });

    // Parent cube - this is the one that rotates
    commands
        .spawn((
            Mesh3d(cube_handle.clone()),
            MeshMaterial3d(cube_material_handle.clone()),
            Transform::from_xyz(0.0, 0.0, 1.0), // World position
            Rotator, // Only the parent has this component
        ))
        // with_children creates a parent-child relationship
        .with_children(|parent| {
            // Child cube - automatically becomes a child of the parent above
            parent.spawn((
                Mesh3d(cube_handle),
                MeshMaterial3d(cube_material_handle),
                // IMPORTANT: This position is RELATIVE to the parent!
                // So this is 3 units along the parent's Z axis
                // As the parent rotates, this will orbit around it
                Transform::from_xyz(0.0, 0.0, 3.0),
            ));
        });
    
    // The transform hierarchy looks like this:
    // Parent (at world position 0,0,1)
    //   └── Child (at parent position 0,0,3)
    // 
    // So the child's world position is actually (0,0,4) initially
    // But as the parent rotates, the child orbits around it!
    // Light
    commands.spawn((
        PointLight::default(),
        Transform::from_xyz(4.0, 5.0, -4.0),
    ));
    
    // Camera - positioned to see the rotation clearly
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
