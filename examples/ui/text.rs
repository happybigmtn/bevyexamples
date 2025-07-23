//! This example illustrates how to create UI text and update it in a system.
//!
//! It displays the current FPS in the top left corner, as well as text that changes color
//! in the bottom right. For text within a scene, please see the text2d example.
//!
//! Text in UI is like digital typography - we can change content, color, size, and style
//! dynamically. This example shows both static text and text that updates every frame,
//! demonstrating the reactive nature of UI systems.

use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

fn main() {
    App::new()
        // FrameTimeDiagnosticsPlugin tracks performance metrics like FPS
        .add_plugins((DefaultPlugins, FrameTimeDiagnosticsPlugin::default()))
        .add_systems(Startup, setup)
        // Two update systems - one for FPS display, one for color animation
        .add_systems(Update, (text_update_system, text_color_system))
        .run();
}

// Marker components are like name tags - they help us find specific entities
// Without these, we'd have to search through ALL text components!

// Marker struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// Marker struct to help identify the color-changing Text component
#[derive(Component)]
struct AnimatedText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    commands.spawn(Camera2d);
    
    // ANIMATED TEXT - Bottom right corner
    // This text will change color over time
    commands.spawn((
        // Text accepts strings - \n creates line breaks!
        Text::new("hello\nbevy!"),
        TextFont {
            // Load a custom font - fonts are assets just like images
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 67.0, // Nice and big!
            ..default()
        },
        TextShadow::default(), // Adds depth with a subtle shadow
        // Text alignment - like word processor settings
        TextLayout::new_with_justify(JustifyText::Center),
        // Position the text using absolute positioning
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0), // 5 pixels from bottom
            right: Val::Px(5.0),  // 5 pixels from right
            ..default()
        },
        AnimatedText, // Our marker component
    ));

    // FPS COUNTER - Top left corner (default position)
    // This demonstrates text with multiple sections (spans)
    commands
        .spawn((
            // Parent text starts with "FPS: "
            Text::new("FPS: "),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 42.0,
                ..default()
            },
        ))
        .with_child((
            // Child span will hold the actual FPS number
            TextSpan::default(), // Empty for now - we'll update it!
            // Conditional compilation - different behavior based on features
            if cfg!(feature = "default_font") {
                (
                    TextFont {
                        font_size: 33.0,
                        // Uses built-in font if available
                        ..default()
                    },
                    TextColor(GOLD.into()), // Golden color for the number
                )
            } else {
                (
                    // Fallback - load a specific font
                    TextFont {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 33.0,
                        ..Default::default()
                    },
                    TextColor(GOLD.into()),
                )
            },
            FpsText, // Marker so we can find this span later
        ));

    #[cfg(feature = "default_font")]
    commands.spawn((
        // Here we are able to call the `From` method instead of creating a new `TextSection`.
        // This will use the default font (a minimal subset of FiraMono) and apply the default styling.
        Text::new("From an &str into a Text with the default font!"),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));

    #[cfg(not(feature = "default_font"))]
    commands.spawn((
        Text::new("Default font disabled"),
        TextFont {
            font: asset_server.load("fonts/FiraMono-Medium.ttf"),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        },
    ));
}

// This system animates text color using sine waves
// Each color channel oscillates at a different frequency - like RGB disco lights!
fn text_color_system(time: Res<Time>, mut query: Query<&mut TextColor, With<AnimatedText>>) {
    for mut text_color in &mut query {
        let seconds = time.elapsed_secs();

        // Create smooth color transitions using sine waves
        // sin() gives us values from -1 to 1
        // We transform this to 0 to 1 range: (sin(x) + 1) / 2
        text_color.0 = Color::srgb(
            ops::sin(1.25 * seconds) / 2.0 + 0.5, // Red channel - fastest
            ops::sin(0.75 * seconds) / 2.0 + 0.5, // Green channel - medium
            ops::sin(0.50 * seconds) / 2.0 + 0.5, // Blue channel - slowest
        );
        // Different frequencies create a complex, ever-changing color pattern
        // It's like mixing three different wave generators!
    }
}

// This system updates the FPS counter text
fn text_update_system(
    // DiagnosticsStore contains performance metrics
    diagnostics: Res<DiagnosticsStore>,
    // Find only the TextSpan entities with FpsText marker
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        // Try to get FPS diagnostic data
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            // Use smoothed value to avoid jittery display
            if let Some(value) = fps.smoothed() {
                // Update the span's text content
                // **span dereferences twice: Mut<TextSpan> -> TextSpan -> String
                **span = format!("{value:.2}"); // .2 = two decimal places
            }
        }
    }
    
    // This pattern shows reactive UI:
    // - Data changes (FPS updates)
    // - System detects change
    // - UI automatically updates
    // No manual refresh needed!
}
