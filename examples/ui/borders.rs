//! # UI Borders Example
//! 
//! This example demonstrates how to create and style UI borders in Bevy 0.16.
//! 
//! ## What You'll Learn
//! - How to create UI borders using `UiRect` for different edge configurations
//! - How to style borders with different colors per edge using `BorderColor`
//! - How to combine borders with border radius for rounded corners
//! - How to use outlines to create additional visual effects around UI elements
//! - Understanding the box model in Bevy UI (content, border, margin, outline)
//! - How to create flexible UI layouts that demonstrate multiple border variations
//! 
//! ## Key Concepts
//! - **Border**: A visual frame around UI elements, defined by width and color
//! - **UiRect**: A rectangle specification that can define different values for each edge
//! - **BorderRadius**: Controls corner rounding, making rectangular borders appear circular
//! - **Outline**: An additional border-like effect that renders outside the border
//! - **Box Model**: The layering system of UI elements (content -> border -> margin -> outline)
//! 
//! The example creates two sets of border demonstrations:
//! 1. Regular borders showing all possible edge combinations
//! 2. Rounded borders with dynamic corner radius based on which edges have borders

// These imports give us everything we need for UI creation and styling
use bevy::{
    color::palettes::css::*, // Pre-defined CSS color constants like RED, BLUE, etc.
    ecs::spawn::SpawnIter,   // Utility for spawning multiple entities from an iterator
    prelude::*,              // Core Bevy types like App, Commands, Bundle traits
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Load Bevy's default systems for windowing, rendering, etc.
        .add_systems(Startup, setup) // Run our setup function once when the app starts
        .run(); // Start the main application loop
}

fn setup(mut commands: Commands) {
    // Spawn a 2D camera to view our UI elements
    // Unlike 3D cameras, 2D cameras use orthographic projection and look down the Z-axis
    commands.spawn(Camera2d);

    // Create descriptive labels for each border configuration we'll demonstrate
    // These correspond 1:1 with the border configurations in the `borders` array below
    let border_labels = [
        "None",            // No borders at all
        "All",             // Borders on all four edges
        "Left",            // Only left edge has a border
        "Right",           // Only right edge has a border
        "Top",             // Only top edge has a border
        "Bottom",          // Only bottom edge has a border
        "Horizontal",      // Left and right edges have borders
        "Vertical",        // Top and bottom edges have borders
        "Top Left",        // Two adjacent edges (corner)
        "Bottom Left",     // Two adjacent edges (corner)
        "Top Right",       // Two adjacent edges (corner)
        "Bottom Right",    // Two adjacent edges (corner)
        "Top Bottom Right", // Three edges, missing left
        "Top Bottom Left",  // Three edges, missing right
        "Top Left Right",   // Three edges, missing bottom
        "Bottom Left Right", // Three edges, missing top
    ];

    // Define all the different border configurations using UiRect
    // UiRect represents a rectangle with potentially different values for each edge
    // Val::Px(n) specifies a size in pixels (as opposed to percentages or other units)
    let borders = [
        // No borders - uses default() which sets all edges to 0
        UiRect::default(),
        
        // All edges have the same 10px border - convenient shorthand
        UiRect::all(Val::Px(10.)),
        
        // Single edge borders - only one edge gets a border
        UiRect::left(Val::Px(10.)),    // Convenience method for left-only border
        UiRect::right(Val::Px(10.)),   // Convenience method for right-only border
        UiRect::top(Val::Px(10.)),     // Convenience method for top-only border
        UiRect::bottom(Val::Px(10.)),  // Convenience method for bottom-only border
        
        // Paired edge borders - two opposite edges
        UiRect::horizontal(Val::Px(10.)), // Left and right edges
        UiRect::vertical(Val::Px(10.)),   // Top and bottom edges
        
        // Corner borders - two adjacent edges
        // Using struct syntax allows us to specify individual edges
        // ..default() fills in the remaining fields with their default values (0)
        UiRect {
            left: Val::Px(20.),  // Thicker left border for visual distinction
            top: Val::Px(10.),
            ..default()          // right and bottom default to 0
        },
        UiRect {
            left: Val::Px(10.),
            bottom: Val::Px(20.), // Thicker bottom border for visual distinction
            ..default()
        },
        UiRect {
            right: Val::Px(20.),
            top: Val::Px(10.),
            ..default()
        },
        UiRect {
            right: Val::Px(10.),
            bottom: Val::Px(10.),
            ..default()
        },
        
        // Three-edge borders - only one edge is missing
        UiRect {
            right: Val::Px(10.),
            top: Val::Px(20.),    // Thicker top border for visual distinction
            bottom: Val::Px(10.),
            ..default()           // left defaults to 0
        },
        UiRect {
            left: Val::Px(10.),
            top: Val::Px(10.),
            bottom: Val::Px(10.),
            ..default()           // right defaults to 0
        },
        UiRect {
            left: Val::Px(20.),   // Thicker left border for visual distinction
            right: Val::Px(10.),
            top: Val::Px(10.),
            ..default()           // bottom defaults to 0
        },
        UiRect {
            left: Val::Px(10.),
            right: Val::Px(10.),
            bottom: Val::Px(20.), // Thicker bottom border for visual distinction
            ..default()           // top defaults to 0
        },
    ];

    // Create the first set of border examples (regular, non-rounded borders)
    // This demonstrates the bundle pattern in Bevy UI - grouping related components
    let borders_examples = (
        // The container node that will hold all our border examples
        Node {
            margin: UiRect::all(Val::Px(25.0)),        // Space around the entire container
            align_self: AlignSelf::Stretch,            // Stretch to fill parent's cross axis
            justify_self: JustifySelf::Stretch,        // Stretch to fill parent's main axis
            flex_wrap: FlexWrap::Wrap,                 // Allow items to wrap to new lines
            justify_content: JustifyContent::FlexStart, // Align items to start of main axis
            align_items: AlignItems::FlexStart,        // Align items to start of cross axis
            align_content: AlignContent::FlexStart,    // Align wrapped lines to start
            ..default()                                // Use default values for other fields
        },
        
        // Children::spawn with SpawnIter creates multiple child entities from an iterator
        // This is a powerful pattern for creating repetitive UI elements
        Children::spawn(SpawnIter(
            border_labels
                .into_iter()
                .zip(borders)  // Combine labels with their corresponding border configs
                .map(|(label, border)| {
                    // Each iteration creates a bundle for one border example
                    (
                        // Container for each individual border example (vertical layout)
                        Node {
                            flex_direction: FlexDirection::Column, // Stack children vertically
                            align_items: AlignItems::Center,       // Center children horizontally
                            ..default()
                        },
                        
                        // Create child elements for this border example
                        children![
                            // The bordered box itself
                            (
                                Node {
                                    width: Val::Px(50.),               // Fixed width
                                    height: Val::Px(50.),              // Fixed height (square)
                                    border,                            // Apply the border config
                                    margin: UiRect::all(Val::Px(20.)), // Space around the box
                                    align_items: AlignItems::Center,    // Center child content
                                    justify_content: JustifyContent::Center, // Center child content
                                    ..default()
                                },
                                
                                // Style the background of the bordered box
                                BackgroundColor(MAROON.into()),
                                
                                // BorderColor allows different colors for each edge
                                // This clearly shows which edges have borders by using distinct colors
                                BorderColor {
                                    top: RED.into(),      // Red border on top edge
                                    bottom: YELLOW.into(), // Yellow border on bottom edge
                                    left: GREEN.into(),   // Green border on left edge
                                    right: BLUE.into(),   // Blue border on right edge
                                },
                                
                                // Outline creates an additional border outside the regular border
                                // This demonstrates the UI box model: content -> border -> outline
                                Outline {
                                    width: Val::Px(6.),   // 6px wide outline
                                    offset: Val::Px(6.),  // 6px gap between border and outline
                                    color: Color::WHITE,  // White color for contrast
                                },
                                
                                // Inner content - a small yellow square to show the content area
                                children![(
                                    Node {
                                        width: Val::Px(10.),
                                        height: Val::Px(10.),
                                        ..default()
                                    },
                                    BackgroundColor(YELLOW.into()),
                                )]
                            ),
                            
                            // Text label below each border example
                            (Text::new(label), TextFont::from_font_size(9.0))
                        ],
                    )
                }),
        )),
    );

    // Helper closures for dynamic border radius calculation
    // These functions help us create rounded corners only where borders exist
    
    // Checks if both border edges are non-zero 
    // Used to determine if a corner should be rounded
    let non_zero = |x, y| x != Val::Px(0.) && y != Val::Px(0.);
    
    // Calculates border radius for a corner based on adjacent border widths
    // Returns maximum radius if both adjacent borders exist, 0 otherwise
    // This creates rounded corners only where two borders meet
    let border_size = move |x, y| {
        if non_zero(x, y) {
            f32::MAX  // Use maximum radius for dramatic rounding effect
        } else {
            0.        // No rounding where borders don't meet
        }
    };

    // Create the second set of border examples with dynamic rounded corners
    // This demonstrates how BorderRadius interacts with borders
    let borders_examples_rounded = (
        // Container node with identical layout to the first example
        Node {
            margin: UiRect::all(Val::Px(25.0)),
            align_self: AlignSelf::Stretch,
            justify_self: JustifySelf::Stretch,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            ..default()
        },
        
        // Similar structure to the first example, but with rounded corners
        Children::spawn(SpawnIter(
            border_labels
                .into_iter()
                .zip(borders)
                .map(move |(label, border)| {
                    // The `move` keyword is needed because we capture the closure `border_size`
                    (
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        children![
                            (
                                Node {
                                    width: Val::Px(50.),
                                    height: Val::Px(50.),
                                    border,
                                    margin: UiRect::all(Val::Px(20.)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(MAROON.into()),
                                BorderColor {
                                    top: RED.into(),
                                    bottom: YELLOW.into(),
                                    left: GREEN.into(),
                                    right: BLUE.into(),
                                },
                                
                                // BorderRadius defines rounding for each corner individually
                                // Each corner's radius is calculated based on its adjacent borders
                                // This creates intelligent rounding that only applies where borders meet
                                BorderRadius::px(
                                    border_size(border.left, border.top),    // Top-left corner
                                    border_size(border.right, border.top),   // Top-right corner
                                    border_size(border.right, border.bottom), // Bottom-right corner
                                    border_size(border.left, border.bottom), // Bottom-left corner
                                ),
                                
                                Outline {
                                    width: Val::Px(6.),
                                    offset: Val::Px(6.),
                                    color: Color::WHITE,
                                },
                                
                                // Inner content with maximum border radius (perfect circle)
                                children![(
                                    Node {
                                        width: Val::Px(10.),
                                        height: Val::Px(10.),
                                        ..default()
                                    },
                                    BorderRadius::MAX,  // Maximum radius makes it circular
                                    BackgroundColor(YELLOW.into()),
                                )],
                            ),
                            (Text::new(label), TextFont::from_font_size(9.0))
                        ],
                    )
                }),
        )),
    );

    // Create the main container that holds both sets of border examples
    // This demonstrates hierarchical UI layout with multiple sections
    commands.spawn((
        // Root container node with vertical layout
        Node {
            margin: UiRect::all(Val::Px(25.0)),           // Outer margin around everything
            flex_direction: FlexDirection::Column,        // Stack sections vertically
            align_self: AlignSelf::Stretch,               // Fill available width
            justify_self: JustifySelf::Stretch,           // Fill available height
            flex_wrap: FlexWrap::Wrap,                    // Allow wrapping if needed
            justify_content: JustifyContent::FlexStart,   // Align to top
            align_items: AlignItems::FlexStart,           // Align to left
            align_content: AlignContent::FlexStart,       // Align wrapped content to top
            ..default()
        },
        
        // Dark gray background to provide contrast for the border examples
        BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
        
        // Define child elements using the children! macro
        // This creates a clean hierarchy: main container -> sections -> examples
        children![
            label("Borders"),              // Section heading for regular borders
            borders_examples,              // First set of border examples
            label("Borders Rounded"),      // Section heading for rounded borders
            borders_examples_rounded       // Second set of border examples
        ],
    ));
}

// Helper function to create section labels
// This demonstrates the Bundle pattern - returning multiple components as a tuple
// The `impl Bundle` return type allows us to return any type that implements Bundle
fn label(text: &str) -> impl Bundle {
    (
        // Node component defines the layout properties
        Node {
            margin: UiRect::all(Val::Px(25.0)),  // Add space around the label
            ..default()                          // Use default values for other fields
        },
        
        // Use children! macro to create a text child element
        // This pattern separates layout (Node) from content (Text)
        children![(
            Text::new(text),                     // Create text with the provided string
            TextFont::from_font_size(20.0)      // Set font size to 20 pixels
        )],
    )
}
