//! # File Drag and Drop
//!
//! This example demonstrates how to handle drag and drop of files into a Bevy application window.
//! When users drag files from their file explorer onto the game window, Bevy generates events
//! that you can process in your systems.
//!
//! ## Key Concepts
//!
//! - **Drag and Drop Events**: OS-level file operations translated to Bevy events
//! - **Event Reading**: How to consume events in systems using EventReader
//! - **Cross-platform Support**: Works on Windows, macOS, and Linux with windowing support
//!
//! ## Event Types
//!
//! The `FileDragAndDrop` enum provides several event variants:
//! - `DroppedFile` - A file was released over the window
//! - `HoveredFile` - A file is being dragged over the window  
//! - `HoveredFileCanceled` - The drag operation was canceled
//!
//! ## Common Use Cases
//!
//! - Loading game assets at runtime (textures, models, save files)
//! - Level editors that accept external files
//! - Configuration file updates
//! - Importing user-generated content

// Import Bevy's prelude for common types
use bevy::prelude::*;

fn main() {
    App::new()
        // Add default plugins which include:
        // - Window plugin (required for drag/drop support)
        // - Render plugin (creates the window)
        // - Input plugins (handle OS events)
        // - And many other core systems
        .add_plugins(DefaultPlugins)
        
        // Register our event handler system to run every frame
        // The Update stage runs after input processing, ensuring events are available
        .add_systems(Update, file_drag_and_drop_system)
        
        // Start the application
        .run();
}

// System that processes file drag and drop events
//
// System parameters:
// - `EventReader<T>` - Provides access to events of type T
// - Events are buffered between frames and cleared after being read
// - The `mut` is required because reading events mutates the reader's cursor
//
// The FileDragAndDrop event contains:
// - `window` - Which window received the drop (for multi-window apps)
// - `path_buf` - The file system path of the dragged file
fn file_drag_and_drop_system(mut events: EventReader<FileDragAndDrop>) {
    // Iterate through all drag/drop events that occurred since last frame
    // `read()` returns an iterator that consumes events
    for event in events.read() {
        // Log the event details
        // The {:?} format specifier uses the Debug trait to print the event
        // 
        // Example output:
        // DroppedFile { window: Entity(0v1), path_buf: "/Users/me/image.png" }
        // HoveredFile { window: Entity(0v1), path_buf: "/Users/me/document.txt" }
        info!("{:?}", event);
        
        // In a real application, you would match on the event type:
        // match event {
        //     FileDragAndDrop::DroppedFile { window, path_buf } => {
        //         // Load the file, validate it, process it
        //     }
        //     FileDragAndDrop::HoveredFile { window, path_buf } => {
        //         // Show preview or highlight drop zone
        //     }
        //     FileDragAndDrop::HoveredFileCanceled { window } => {
        //         // Clean up any preview UI
        //     }
        // }
    }
}
