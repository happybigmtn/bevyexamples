//! Custom Diagnostics - The Performance Observatory
//!
//! Diagnostics in Bevy are like scientific instruments measuring your game's vital signs.
//! Think of them as the gauges on a spacecraft dashboard - they tell you what's happening
//! under the hood so you can optimize performance and debug issues.
//!
//! This example demonstrates creating your own custom diagnostic system, like building
//! a specialized sensor for your game. While Bevy provides built-in diagnostics for
//! frame time, FPS, and memory usage, you can create custom ones to track anything:
//! - Player actions per minute
//! - Network latency
//! - AI computation time
//! - Custom game metrics

// Import the diagnostic infrastructure - these are the tools for building our measurement system
use bevy::{
    diagnostic::{
        Diagnostic,        // The core measurement container - holds values and metadata
        DiagnosticPath,    // Unique identifier for each diagnostic - like a street address
        Diagnostics,       // The global registry where all measurements are stored
        LogDiagnosticsPlugin, // Built-in plugin that prints diagnostics to console
        RegisterDiagnostic,   // Trait that lets us register new diagnostics with the app
    },
    prelude::*,  // Essential types for any Bevy application
};

fn main() {
    // Initialize the Bevy application - our laboratory for performance measurement
    App::new()
        .add_plugins((
            // Essential plugins for windowing, rendering, input handling, etc.
            DefaultPlugins,
            
            // LogDiagnosticsPlugin is like a data logger - it automatically prints
            // diagnostic values to the console at regular intervals (every few seconds).
            // This is incredibly useful for monitoring performance in real-time.
            // Without this plugin, diagnostics would still be collected but not displayed.
            LogDiagnosticsPlugin::default(),
        ))
        
        // CRITICAL: Register our custom diagnostic before any system tries to use it!
        // This is like installing a new gauge in your dashboard before trying to read it.
        // 
        // The registration process:
        // 1. Creates a new Diagnostic with our unique path identifier
        // 2. Adds a suffix (" iterations") that will appear in the console output
        // 3. Stores it in the global diagnostics registry
        // 
        // Think of this as "teaching" Bevy about our new measurement type.
        .register_diagnostic(Diagnostic::new(SYSTEM_ITERATION_COUNT).with_suffix(" iterations"))
        
        // Add our measurement system to run every frame during the Update schedule.
        // This system will continuously feed data to our diagnostic.
        .add_systems(Update, my_system)
        
        // Launch the game loop - our diagnostic will start collecting data immediately
        .run();
}

// DiagnosticPath is like a unique address for each diagnostic in the system.
// Just like how every house needs a unique address, every diagnostic needs a unique path.
// This prevents conflicts when multiple diagnostics are registered.
// 
// The string "system_iteration_count" is our diagnostic's identifier.
// - It should be descriptive but concise
// - It appears in logs and debug output
// - It must be unique across your entire application
// 
// Using `const_new` allows this to be computed at compile time for efficiency.
const SYSTEM_ITERATION_COUNT: DiagnosticPath = DiagnosticPath::const_new("system_iteration_count");

// This is our measurement system - it runs every frame and records data.
// The `mut diagnostics: Diagnostics` parameter gives us access to the global
// diagnostics registry where we can add measurements.
fn my_system(mut diagnostics: Diagnostics) {
    // Record a measurement for our diagnostic.
    // The value 10.0 is just for demonstration - in a real game, you'd measure something meaningful:
    // - Time taken to complete a task
    // - Number of entities processed
    // - Memory usage
    // - Network packets sent/received
    // 
    // The closure `|| 10.0` is a function that returns the value to record.
    // Using a closure allows for lazy evaluation - the value is only computed when needed.
    // This is useful for expensive calculations that you only want to run when diagnostics are active.
    diagnostics.add_measurement(&SYSTEM_ITERATION_COUNT, || 10.0);
    
    // Bevy automatically handles:
    // - Storing the measurement with a timestamp
    // - Calculating averages over time
    // - Managing memory (old measurements are cleaned up)
    // - Thread-safe access from multiple systems
    // 
    // You'll see this value printed to the console every few seconds thanks to LogDiagnosticsPlugin.
    // The output will show the current value, average, and other statistics.
}
