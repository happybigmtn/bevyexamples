//! # Custom App Runner
//!
//! This example demonstrates how to create a custom runner to manually control the application's
//! main loop. Instead of using Bevy's default runner (which creates a window and runs at 60 FPS),
//! we create a console application that reads input from stdin and processes it through the ECS.
//!
//! ## Key Concepts
//!
//! - **App Runner**: The function that controls how your Bevy app executes its main loop
//! - **Manual Update**: Calling `app.update()` yourself instead of letting Bevy handle it
//! - **Resource**: Shared data accessible across systems (like our Input buffer)
//! - **System**: Functions that process data within the ECS
//!
//! ## Use Cases
//!
//! Custom runners are useful for:
//! - Console applications without graphics
//! - Server applications with custom networking loops
//! - Testing frameworks that need precise control over updates
//! - Integrating Bevy into existing applications

// Import Bevy's application exit type and standard prelude
// The prelude contains the most commonly used types and traits
use bevy::{app::AppExit, prelude::*};

// Standard library's I/O module for reading from stdin
use std::io;

// Define a Resource to hold user input
// Resources are globally accessible data stored in the World
// 
// The #[derive(Resource)] macro automatically implements the Resource trait
// This tells Bevy that this type can be inserted into and queried from the World
//
// We use a tuple struct (a struct with unnamed fields) containing a String
// This pattern is common for simple wrapper types in Rust
#[derive(Resource)]
struct Input(String);

// Custom runner function that replaces Bevy's default event loop
//
// Function signature breakdown:
// - `mut app: App` - Takes ownership of the App instance (moved, not borrowed)
// - `-> AppExit` - Returns an exit status (Success or Error with code)
//
// This function will be called instead of the default runner when we call app.run()
fn my_runner(mut app: App) -> AppExit {
    // Complete the app building process
    // These methods are normally called by the default runner
    
    // `finish()` - Finalizes plugin installation and performs one-time setup
    // This includes registering components, setting up render pipelines, etc.
    app.finish();
    
    // `cleanup()` - Runs any cleanup operations registered by plugins
    // This ensures the app is in a consistent state before the main loop
    app.cleanup();

    // Simple user prompt
    println!("Type stuff into the console");
    
    // Main application loop - read lines from standard input
    // `io::stdin().lines()` returns an iterator over input lines
    // Each iteration blocks until the user presses Enter
    for line in io::stdin().lines() {
        // Update the Input resource with the new line
        // We use a scope block to limit the lifetime of the mutable borrow
        {
            // Get mutable access to the World and then to our Input resource
            // `world_mut()` returns &mut World - exclusive access to all ECS data
            // `resource_mut::<Input>()` returns Mut<Input> - a change-tracking wrapper
            let mut input = app.world_mut().resource_mut::<Input>();
            
            // Extract the String from the Result<String, Error> that lines() yields
            // `unwrap()` will panic on I/O errors - okay for this example
            input.0 = line.unwrap();
        }
        
        // Run one frame of the app - this executes all scheduled systems
        // Systems will see the updated Input resource
        app.update();

        // Check if any system requested the app to exit
        // `should_exit()` returns Some(AppExit) if exit was requested
        if let Some(exit) = app.should_exit() {
            // Return early with the requested exit status
            return exit;
        }
    }

    // If stdin closes (e.g., EOF), exit successfully
    AppExit::Success
}

// System that prints the current input
//
// System function requirements:
// - Parameters must implement SystemParam trait
// - Common SystemParams: Res<T>, ResMut<T>, Query<T>, EventReader<T>, etc.
//
// `Res<Input>` provides immutable access to the Input resource
// Res is a smart pointer that derefs to &Input
fn print_system(input: Res<Input>) {
    // Access the inner String via the .0 field (tuple struct syntax)
    println!("You typed: {}", input.0);
}

// System that checks for exit command
//
// This system demonstrates:
// - Reading resources (Res<Input>)
// - Writing events (EventWriter<AppExit>)
// - Conditional logic in systems
//
// EventWriter<T> allows systems to send events that other systems can read
// Events are stored in a ring buffer and cleared periodically
fn exit_system(input: Res<Input>, mut exit_event: EventWriter<AppExit>) {
    // Check if user typed "exit"
    if input.0 == "exit" {
        // Send an AppExit event to gracefully shut down the application
        // `write()` queues the event to be processed after this system completes
        exit_event.write(AppExit::Success);
    }
}

// Main function with non-standard return type
//
// Rust's main function can return:
// - () (unit type) - the default
// - Result<(), E> where E: Debug - for error handling
// - Any type implementing Termination trait
//
// AppExit implements Termination, converting:
// - AppExit::Success -> exit code 0
// - AppExit::Error(code) -> exit code `code`
fn main() -> AppExit {
    // Build and run the Bevy application
    App::new()
        // Initialize the Input resource with an empty string
        // Resources must be inserted before systems can access them
        .insert_resource(Input(String::new()))
        
        // Replace the default runner with our custom implementation
        // The runner takes ownership of the App and controls its execution
        .set_runner(my_runner)
        
        // Register our systems to run during the Update stage
        // 
        // System execution order:
        // 1. Both systems are in the same stage (Update)
        // 2. They can run in parallel if they don't conflict
        // 3. print_system and exit_system both read Input (no conflict)
        // 4. Only exit_system writes events (no conflict)
        //
        // The tuple syntax (system1, system2) is shorthand for adding multiple systems
        .add_systems(Update, (print_system, exit_system))
        
        // Start the application
        // This calls our custom runner, which returns the exit status
        .run()
}
