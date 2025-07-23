//! Displays a single [`Sprite`], created from an image.
//!
//! Welcome to 2D graphics! If hello_world was the atom, this is the molecule - 
//! we're combining multiple concepts to create something visible.

use bevy::prelude::*;

fn main() {
    App::new()
        // DefaultPlugins is like a pre-built chemistry set - it includes everything
        // a typical game needs: window creation, rendering, input handling, audio, etc.
        // Without this, we'd have no window to draw in! It's the difference between
        // theoretical physics and experimental physics - we need apparatus to observe.
        .add_plugins(DefaultPlugins)
        // The Startup schedule runs ONCE when your app begins - perfect for setup.
        // Think of it as the "initial conditions" of your universe.
        // Unlike Update (which runs every frame), Startup is for one-time initialization.
        .add_systems(Startup, setup)
        .run();
}

// Now our system has parameters! This is where Bevy's magic shines.
// These aren't random parameters - they're "system parameters" that Bevy injects.
// It's like having a lab assistant who hands you exactly the tools you need.
fn setup(
    // Commands let us spawn entities and components - think of it as our "matter creator"
    // The 'mut' is important - we're modifying the game world, not just observing it.
    mut commands: Commands,
    // AssetServer is our "library" - it loads files from disk into memory.
    // Res<T> means "give me shared read access to resource T"
    // Resources are single-instance data shared across the whole game.
    asset_server: Res<AssetServer>,
) {
    // First law of 2D graphics: you need a camera to see anything!
    // This spawns an entity with Camera2d component. In Bevy, everything is an entity
    // with components. It's like LEGO - entities are the bricks, components are their properties.
    // Camera2d is a "bundle" - a pre-made collection of components that work together.
    commands.spawn(Camera2d);

    // Now we spawn our sprite. Let's break this down like a physics equation:
    // 1. commands.spawn() creates a new entity (a new "thing" in our world)
    // 2. Sprite::from_image() creates a Sprite component with an image
    // 3. asset_server.load() starts loading the image file (it returns a Handle immediately)
    //
    // The Handle is like a promise - "I'll have your image ready soon!"
    // Bevy will automatically display it once loaded. No callback hell, no manual checking.
    commands.spawn(Sprite::from_image(
        asset_server.load("branding/bevy_bird_dark.png"),
    ));
    
    // That's it! But notice what we DIDN'T have to do:
    // - No manual render loop
    // - No OpenGL/Vulkan/DirectX calls  
    // - No matrix math for positioning (it defaults to origin)
    // - No texture binding or shader setup
    //
    // Bevy handles the complex machinery so you can focus on game logic.
    // It's like the difference between quantum field theory and chemistry -
    // you work at the level of abstraction that makes sense for your problem.
}
