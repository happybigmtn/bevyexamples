//! # Minimal Empty Application
//!
//! This example demonstrates the absolute minimum Bevy application - one with no plugins,
//! systems, or resources. It immediately exits since there's nothing to do.
//!
//! ## Understanding the Minimal App
//!
//! Without any plugins:
//! - No window is created
//! - No rendering system exists
//! - No input handling occurs
//! - No game loop runs
//! - The app starts and immediately exits
//!
//! ## The App Structure
//!
//! Even this minimal app has:
//! - A `World` - The ECS database storing entities and components
//! - A `Schedule` - The system executor (though no systems are added)
//! - Basic app lifecycle methods
//!
//! ## Use Cases
//!
//! This pattern is useful as:
//! - A starting point for understanding Bevy's architecture
//! - A base for unit tests that need minimal overhead
//! - A template for building custom plugin configurations
//!
//! ## Next Steps
//!
//! To make this app do something, you would typically:
//! 1. Add plugins: `.add_plugins(DefaultPlugins)` or `.add_plugins(MinimalPlugins)`
//! 2. Add systems: `.add_systems(Update, my_system)`
//! 3. Insert resources: `.insert_resource(MyResource)`
//! 4. Spawn entities: Use `Commands` in a startup system

// Import Bevy's prelude
// Even though we're not using most of it, this is the standard import
use bevy::prelude::*;

// Minimal application entry point
fn main() {
    // Create a new App instance
    // The App is the core container that holds:
    // - The World (ECS data storage)
    // - Schedules (system execution plans)
    // - Plugins (modular feature sets)
    // - Resources (globally accessible data)
    App::new()
        // Run the app
        // Without any plugins or systems, this will:
        // 1. Initialize the empty World
        // 2. Check for any startup systems (none exist)
        // 3. Begin the main loop
        // 4. Check for any systems to run (none exist)
        // 5. Check if the app should exit (no exit conditions)
        // 6. Since there's nothing keeping it alive, exit immediately
        .run();
    
    // The application exits here
    // In a real app, .run() would block until the window closes
    // or an exit event is triggered
}
