//! Dynamic Diagnostic Control - The Master Switch
//!
//! Sometimes you need to turn diagnostics on and off during gameplay, like dimming 
//! the lights in a laboratory. This example demonstrates the powerful ability to
//! dynamically control diagnostic collection at runtime.
//!
//! Why would you want to do this?
//! - Performance profiling: Enable diagnostics only when needed to reduce overhead
//! - Debug modes: Toggle detailed monitoring based on user settings
//! - Adaptive performance: Disable diagnostics in performance-critical moments
//! - Development vs. production: Different diagnostic levels for different builds
//!
//! This example uses a timer-based system that toggles ALL diagnostics every 10 seconds,
//! demonstrating both the control mechanism and how diagnostics behave when disabled.

// Duration is needed for creating time-based conditions
use std::time::Duration;

use bevy::{
    diagnostic::{
        DiagnosticsStore,          // The central storage system for all diagnostic data
        FrameTimeDiagnosticsPlugin, // Built-in plugin that measures frame timing
        LogDiagnosticsPlugin,      // Plugin that prints diagnostic values to console
    },
    prelude::*,
    time::common_conditions::on_timer, // Convenient condition for timer-based execution
};

fn main() {
    // Set up our diagnostic control experiment
    App::new()
        .add_plugins((
            // Core Bevy functionality
            DefaultPlugins,
            
            // FrameTimeDiagnosticsPlugin measures critical performance metrics:
            // - Frame time (how long each frame takes to render)
            // - Frames per second (FPS) 
            // - Frame time variance (consistency of performance)
            // This is one of the most important diagnostics for game development!
            FrameTimeDiagnosticsPlugin::default(),
            
            // LogDiagnosticsPlugin outputs diagnostic values to the console
            // When diagnostics are disabled, you'll see the values stop updating,
            // demonstrating the on/off behavior in real-time.
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(
            Update,
            // This is a conditional system - it only runs when the timer condition is met.
            // The `run_if` method is like an "if" statement for systems:
            // "Run the toggle system IF the timer has elapsed"
            // 
            // on_timer(Duration::from_secs_f32(10.0)) creates a condition that returns
            // true every 10 seconds. This gives you time to observe the diagnostics
            // both enabled and disabled in the console output.
            toggle.run_if(on_timer(Duration::from_secs_f32(10.0))),
        )
        .run();
}

// This system demonstrates the power of runtime diagnostic control.
// It's called every 10 seconds thanks to the timer condition above.
fn toggle(mut store: ResMut<DiagnosticsStore>) {
    // DiagnosticsStore is the central repository where all diagnostic data lives.
    // Think of it as the control panel for your entire monitoring system.
    // 
    // The `mut` keyword is crucial here - we need mutable access to change
    // the enabled/disabled state of each diagnostic.
    
    // Iterate through every diagnostic currently registered in the system.
    // This includes both built-in diagnostics (like frame time) and any
    // custom diagnostics you've created.
    for diag in store.iter_mut() {
        // Log which diagnostic we're toggling - this helps you see what's happening
        // The path() method returns the unique identifier for this diagnostic
        info!("toggling diagnostic {}", diag.path());
        
        // Here's the magic! The `is_enabled` field controls whether this diagnostic
        // actively collects data. When false:
        // - No new measurements are recorded
        // - Existing data remains in memory
        // - The diagnostic "goes dark" until re-enabled
        // 
        // The `!` operator flips the boolean: true becomes false, false becomes true
        diag.is_enabled = !diag.is_enabled;
        
        // This is a powerful pattern for performance optimization:
        // - Enable diagnostics during development and debugging
        // - Disable them in shipped games to save CPU cycles
        // - Toggle them dynamically based on user settings or game states
    }
    
    // Watch the console output carefully! You'll see the diagnostic values
    // appear and disappear as this system toggles them on and off every 10 seconds.
    // This demonstrates real-time control over your monitoring infrastructure.
}
