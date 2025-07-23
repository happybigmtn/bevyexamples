//! Displays a single [`Sprite`], created from an image, but flipped on one axis.
//!
//! Sprite flipping is like looking at an image in a mirror - it's the same picture but
//! reversed! This is incredibly useful for game development: instead of creating separate
//! left-facing and right-facing sprites for a character, you can use one sprite and flip
//! it horizontally. It's like having a reversible jacket - two looks from one asset!
//! This saves memory, reduces art work, and makes direction changes instant.

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // THE MIRROR IMAGE - Creating a flipped sprite
    commands.spawn(Sprite {
        image: asset_server.load("branding/bevy_bird_dark.png"),
        // HORIZONTAL FLIP - Like looking in a mirror
        // Flip the logo to the left
        flip_x: true,  // true = mirror horizontally, false = normal
        // Think of it as flipping a playing card face-down horizontally
        
        // VERTICAL FLIP - Like standing on your head
        // And don't flip it upside-down ( the default )
        flip_y: false, // true = upside down, false = right-side up
        // Useful for reflections in water or ceiling-mounted enemies!
        
        ..Default::default()
    });
    
    // FLIPPING USE CASES IN GAMES:
    // - Character direction: flip_x when moving left vs right
    // - Card games: flip_y to show face-down cards
    // - Reflections: flip_y for water reflections
    // - Symmetry: Use one sprite for both sides of symmetric objects
}
