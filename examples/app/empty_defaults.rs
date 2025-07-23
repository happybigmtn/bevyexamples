//! # Empty Application with Default Plugins
//!
//! This example shows the absolute minimum code needed to create a Bevy application
//! with all standard features enabled. It creates a window that displays a blank scene
//! and responds to basic input events.
//!
//! ## What Are Default Plugins?
//!
//! `DefaultPlugins` is a plugin group that includes all the core functionality
//! most games need:
//!
//! - **Window Management** - Creates and manages application windows
//! - **Rendering** - 2D/3D graphics pipeline, cameras, meshes, materials
//! - **Asset Loading** - Loading textures, models, sounds from files
//! - **Input Handling** - Keyboard, mouse, gamepad, touch input
//! - **Time & Scheduling** - Frame timing, system execution order
//! - **Audio** - Sound playback and spatial audio
//! - **Diagnostics** - FPS counter, performance metrics
//! - **Transform Hierarchy** - Position, rotation, scale, parenting
//! - **UI** - User interface layout and rendering
//!
//! ## When to Use This Pattern
//!
//! This is the standard starting point for:
//! - Games with graphics and audio
//! - Interactive applications
//! - Prototypes that might need any standard features
//!
//! ## When NOT to Use Default Plugins
//!
//! Consider a minimal plugin set if you need:
//! - Headless servers (no graphics/audio)
//! - Embedded systems with limited resources
//! - Specialized applications that only need specific features

// Import all commonly used Bevy types
// The prelude module re-exports the most frequently used items
use bevy::prelude::*;

// Application entry point
fn main() {
    // Create a new Bevy application
    App::new()
        // Add the default plugin group
        // This single line adds ~30 individual plugins that provide:
        // - A window (via WinitPlugin)
        // - Rendering pipeline (via RenderPlugin) 
        // - Input systems (via InputPlugin)
        // - Time management (via TimePlugin)
        // - Asset loading (via AssetPlugin)
        // - Audio playback (via AudioPlugin)
        // - Transform systems (via TransformPlugin)
        // - Hierarchy management (via HierarchyPlugin)
        // - And many more...
        .add_plugins(DefaultPlugins)
        
        // Start the application's main loop
        // This will:
        // 1. Initialize all plugins
        // 2. Create a window
        // 3. Begin running systems in a loop
        // 4. Handle events and render frames
        // 5. Continue until the window is closed
        .run();
}
