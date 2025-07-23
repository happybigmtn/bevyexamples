//! A minimal example that outputs "hello world"
//! 
//! This is where every Bevy journey begins. Think of this as the "atom" of game development -
//! the simplest possible program that still demonstrates the fundamental architecture.

// The prelude module is like a starter kit - it imports the most commonly used types and traits.
// In physics, we start with fundamental particles; in Bevy, we start with fundamental imports.
use bevy::prelude::*;

fn main() {
    // The App is the nucleus of your game - everything orbits around it.
    // Like how atoms form molecules, Apps form games by combining systems and resources.
    App::new()
        // Systems are like the laws of physics in your game universe.
        // This line says: "During the Update phase (every frame), run the hello_world_system"
        // The Update schedule is one of several "stages" where different types of work happen.
        // Think of it like the heartbeat of your game - regular, predictable, essential.
        .add_systems(Update, hello_world_system)
        // run() is the Big Bang moment - it starts the universe simulation.
        // From here, Bevy takes control and begins the game loop:
        // 1. Process inputs
        // 2. Update game state (this is where our system runs)
        // 3. Render the frame
        // 4. Repeat until the program exits
        .run();
}

// A system is just a regular Rust function - but it's special because Bevy will call it.
// Systems are the "forces" in your game universe - they make things happen.
// In a real game, systems might move characters, check collisions, or update scores.
// 
// The beauty is in the simplicity: Bevy doesn't require inheritance, special base classes,
// or complex rituals. A system is just a function. This one takes no parameters (yet).
fn hello_world_system() {
    // Every frame, this message appears. In a 60 FPS game, you'd see this 60 times per second!
    // This is like observing a particle in quantum mechanics - each observation is discrete,
    // but together they form the continuous experience of a game.
    println!("hello world");
    
    // In a real game, you'd rarely print every frame (it floods the console).
    // But for learning? It's perfect. You can SEE the game loop in action.
}
