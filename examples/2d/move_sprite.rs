//! Renders a 2D scene containing a single, moving sprite.
//!
//! This example introduces MOTION - the heart of any game! We'll make a sprite
//! bounce back and forth like a ping-pong ball. It demonstrates custom components,
//! time-based movement, and the Query system - Bevy's powerful way to find entities.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // Now we have a system that runs EVERY FRAME in Update
        // This is where the magic happens - continuous motion!
        .add_systems(Update, sprite_movement)
        .run();
}

// Custom component! This is a huge concept in Bevy.
// Components are just data attached to entities. ANY type can be a component
// with #[derive(Component)]. It's like labeling objects with properties.
// Here, we track which way our sprite is moving.
#[derive(Component)]
enum Direction {
    Left,
    Right,
    // In Rust, enums are "sum types" - a value must be EXACTLY ONE of these.
    // Perfect for mutually exclusive states like direction!
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // Spawn with multiple components! The tuple syntax lets us add several at once.
    // Think of this as creating an object and immediately giving it properties:
    // "Here's a thing that has an image, a position, and a movement direction"
    commands.spawn((
        // The visual representation
        Sprite::from_image(asset_server.load("branding/icon.png")),
        // Transform::from_xyz is a convenience constructor
        // Position (0,0,0) is the center of the screen in 2D
        Transform::from_xyz(0., 0., 0.),
        // Our custom component! The sprite starts moving right
        Direction::Right,
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
///
/// This system demonstrates several key concepts:
/// 1. Time-based movement (frame-rate independent)
/// 2. The Query system for finding entities
/// 3. Mutable access to components
fn sprite_movement(
    // Time is a resource that tracks frame timing. CRUCIAL for smooth movement!
    // Without this, movement speed would depend on frame rate - chaos!
    time: Res<Time>,
    // Query: "Find all entities that have BOTH Direction AND Transform"
    // The &mut means we want to modify these components
    // This is like saying: "Give me all moveable objects in the scene"
    mut sprite_position: Query<(&mut Direction, &mut Transform)>,
) {
    // Iterate through all entities matching our query
    // In this example, it's just one sprite, but this scales to thousands!
    for (mut logo, mut transform) in &mut sprite_position {
        // Pattern matching on enum - Rust's powerful control flow
        // This is like a switch statement but more powerful
        match *logo {
            // Physics lesson: velocity = distance/time, so distance = velocity * time
            // 150 pixels/second * delta_time = pixels to move this frame
            // This ensures consistent speed regardless of frame rate!
            Direction::Right => transform.translation.x += 150. * time.delta_secs(),
            Direction::Left => transform.translation.x -= 150. * time.delta_secs(),
        }

        // Boundary checking - our sprite bounces between -200 and +200 pixels
        // This is like an invisible wall that reverses direction
        if transform.translation.x > 200. {
            *logo = Direction::Left;  // Hit right wall, go left
        } else if transform.translation.x < -200. {
            *logo = Direction::Right; // Hit left wall, go right
        }
        
        // Notice: we're modifying components directly, and Bevy tracks all changes!
        // This is the beauty of ECS - data and behavior are separate but connected.
    }
}
