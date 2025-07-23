//! Comprehensive Diagnostic Logging - The Performance Observatory Control Panel
//!
//! This example demonstrates the full spectrum of built-in diagnostic plugins that Bevy offers.
//! Think of this as the "mission control" for your game's performance monitoring - it shows
//! how to log critical metrics like FPS, memory usage, entity counts, and render performance
//! to the console with interactive filtering.
//!
//! Key Features Demonstrated:
//! - Frame timing diagnostics (FPS, frame time, frame count)
//! - Entity counting for memory management insights
//! - System resource monitoring (CPU and memory usage)
//! - Render pipeline diagnostics for GPU performance
//! - Interactive filtering to focus on specific metrics
//! - Real-time toggling of diagnostic categories
//!
//! This is essential for optimizing games and understanding performance bottlenecks.

// Color palettes for the interactive UI
use bevy::{
    color::palettes, // Tailwind CSS color palette for consistent UI colors
    diagnostic::{
        DiagnosticPath,                    // Unique identifier for each diagnostic type
        EntityCountDiagnosticsPlugin,       // Tracks the number of entities in the world
        FrameTimeDiagnosticsPlugin,         // Measures frame timing and FPS
        LogDiagnosticsPlugin,               // Outputs diagnostic data to console
        LogDiagnosticsState,                // Controls which diagnostics are logged
        SystemInformationDiagnosticsPlugin, // Monitors CPU and memory usage
    },
    prelude::*,
};

// Diagnostic path collections for easy batch operations.
// These constants group related diagnostics together for filtering.

// Frame timing diagnostics - the heartbeat of your game's performance
const FRAME_TIME_DIAGNOSTICS: [DiagnosticPath; 3] = [
    FrameTimeDiagnosticsPlugin::FPS,         // Frames per second - higher is smoother
    FrameTimeDiagnosticsPlugin::FRAME_COUNT, // Total frames rendered since start
    FrameTimeDiagnosticsPlugin::FRAME_TIME,  // Time to render each frame in milliseconds
];

// Entity management diagnostics - crucial for memory optimization
const ENTITY_COUNT_DIAGNOSTICS: [DiagnosticPath; 1] = [
    EntityCountDiagnosticsPlugin::ENTITY_COUNT, // Number of active entities in the world
];

// System resource diagnostics - monitor your game's impact on the computer
const SYSTEM_INFO_DIAGNOSTICS: [DiagnosticPath; 4] = [
    SystemInformationDiagnosticsPlugin::PROCESS_CPU_USAGE, // CPU used by your game process
    SystemInformationDiagnosticsPlugin::PROCESS_MEM_USAGE, // Memory used by your game process
    SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE,  // Total system CPU usage
    SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE,  // Total system memory usage
];

fn main() {
    // Create our diagnostic monitoring station
    App::new()
        .add_plugins((
            // CRITICAL: DefaultPlugins must come first! Diagnostic plugins depend on
            // core systems like Time for accurate timestamps and measurements.
            DefaultPlugins,
            
            // LogDiagnosticsPlugin is the "output device" - it takes diagnostic data
            // and prints it to the console. Without this, diagnostics are collected
            // but not displayed. You could replace this with an in-game UI overlay.
            LogDiagnosticsPlugin::default(),
            
            // FrameTimeDiagnosticsPlugin measures the most critical performance metrics:
            // - How long each frame takes to render (frame time)
            // - How many frames per second your game achieves (FPS)
            // - Total frame count since the application started
            // This is usually the first diagnostic you want in any game.
            FrameTimeDiagnosticsPlugin::default(),
            
            // EntityCountDiagnosticsPlugin tracks the number of entities in your world.
            // This is crucial for detecting memory leaks and understanding when
            // entity creation/destruction might be impacting performance.
            EntityCountDiagnosticsPlugin,
            
            // SystemInformationDiagnosticsPlugin monitors your game's impact on the host system:
            // - CPU usage (both your process and system-wide)
            // - Memory usage (both your process and system-wide)
            // Essential for ensuring your game is a "good citizen" on the player's computer.
            SystemInformationDiagnosticsPlugin,
            
            // RenderDiagnosticsPlugin provides deep insights into the rendering pipeline.
            // It's verbose but invaluable for diagnosing GPU performance issues.
            // Examples: draw calls, vertex counts, texture uploads, etc.
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
        ))
        
        // Render diagnostics only appear when something is being drawn,
        // so we create a simple 3D scene to generate meaningful data.
        .add_systems(Startup, setup)
        
        // Input handling system runs every frame to check for key presses
        .add_systems(Update, filters_inputs)
        
        // UI update system only runs when diagnostic settings change.
        // This is an optimization - we only rebuild the UI when necessary.
        // The condition uses Rust's boolean OR operator (||) for multiple triggers.
        .add_systems(
            Update,
            update_commands.run_if(
                resource_exists_and_changed::<LogDiagnosticsStatus>
                    .or(resource_exists_and_changed::<LogDiagnosticsFilters>),
            ),
        )
        .run();
}

/// Set up a simple 3D scene to generate render diagnostics.
/// Without geometry to render, many render diagnostics would remain at zero.
/// This creates a minimal but complete scene: geometry, lighting, and camera.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,      // Mesh asset storage
    mut materials: ResMut<Assets<StandardMaterial>>, // Material asset storage
) {
    // Create a circular base (floor) - this generates vertex/triangle count diagnostics
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))), // Circle with radius 4 units
        MeshMaterial3d(materials.add(Color::WHITE)), // Simple white material
        // Rotate 90 degrees around X to make it horizontal (like a floor)
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    
    // Create a cube - adds more geometry for meaningful render statistics
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))), // 1x1x1 unit cube
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))), // Nice blue color
        Transform::from_xyz(0.0, 0.5, 0.0), // Position it on top of the floor
    ));
    
    // Add a point light with shadows - this generates lighting computation diagnostics
    commands.spawn((
        PointLight {
            shadows_enabled: true, // Shadow rendering adds to diagnostic complexity
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0), // Position above and to the side
    ));
    
    // Spawn a camera to view the scene - essential for render pipeline activation
    commands.spawn((
        Camera3d::default(),
        // Position camera to get a good view of our simple scene
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Initialize our custom resources for managing diagnostic display
    commands.init_resource::<LogDiagnosticsFilters>(); // Tracks which categories are enabled
    commands.init_resource::<LogDiagnosticsStatus>();  // Tracks if filtering is active

    // Create the UI panel that shows diagnostic controls
    // This demonstrates how to build interactive diagnostic tools
    commands.spawn((
        LogDiagnosticsCommands, // Component marker for our control panel
        Node {
            top: Val::Px(5.),    // 5 pixels from top of screen
            left: Val::Px(5.),   // 5 pixels from left of screen
            flex_direction: FlexDirection::Column, // Stack elements vertically
            ..default()
        },
    ));
}

/// Handle keyboard input to dynamically control diagnostic filtering.
/// This system demonstrates real-time diagnostic management - a powerful
/// debugging technique for focusing on specific performance aspects.
fn filters_inputs(
    keys: Res<ButtonInput<KeyCode>>,              // Input system for keyboard
    mut status: ResMut<LogDiagnosticsStatus>,     // Whether filtering is active
    mut filters: ResMut<LogDiagnosticsFilters>,   // Which categories to show
    mut log_state: ResMut<LogDiagnosticsState>,   // Controls actual log output
) {
    // Q key toggles the entire filtering system on/off
    // When disabled: show ALL diagnostics (useful for comprehensive monitoring)
    // When enabled: show only selected categories (useful for focused debugging)
    if keys.just_pressed(KeyCode::KeyQ) {
        *status = match *status {
            LogDiagnosticsStatus::Enabled => {
                // Turn off filtering - now all diagnostics will be logged
                log_state.disable_filtering();
                LogDiagnosticsStatus::Disabled
            }
            LogDiagnosticsStatus::Disabled => {
                // Turn on filtering - only enabled categories will be logged
                log_state.enable_filtering();
                
                // Apply current filter settings to the log state
                // This demonstrates how to programmatically control diagnostics
                if filters.frame_time {
                    enable_filters(&mut log_state, FRAME_TIME_DIAGNOSTICS);
                }
                if filters.entity_count {
                    enable_filters(&mut log_state, ENTITY_COUNT_DIAGNOSTICS);
                }
                if filters.system_info {
                    enable_filters(&mut log_state, SYSTEM_INFO_DIAGNOSTICS);
                }
                LogDiagnosticsStatus::Enabled
            }
        };
    }

    // Only apply filter changes when filtering is enabled
    // This pattern allows you to configure filters while disabled, then apply them
    let enabled = *status == LogDiagnosticsStatus::Enabled;
    
    // Key 1: Toggle frame timing diagnostics (FPS, frame time, frame count)
    // This is typically the most important diagnostic category
    if keys.just_pressed(KeyCode::Digit1) {
        filters.frame_time = !filters.frame_time; // Flip the boolean state
        if enabled {
            if filters.frame_time {
                enable_filters(&mut log_state, FRAME_TIME_DIAGNOSTICS);
            } else {
                disable_filters(&mut log_state, FRAME_TIME_DIAGNOSTICS);
            }
        }
    }
    
    // Key 2: Toggle entity count diagnostics
    // Useful for tracking memory usage and entity lifecycle management
    if keys.just_pressed(KeyCode::Digit2) {
        filters.entity_count = !filters.entity_count;
        if enabled {
            if filters.entity_count {
                enable_filters(&mut log_state, ENTITY_COUNT_DIAGNOSTICS);
            } else {
                disable_filters(&mut log_state, ENTITY_COUNT_DIAGNOSTICS);
            }
        }
    }
    
    // Key 3: Toggle system information diagnostics (CPU and memory usage)
    // Critical for understanding your game's impact on the host system
    if keys.just_pressed(KeyCode::Digit3) {
        filters.system_info = !filters.system_info;
        if enabled {
            if filters.system_info {
                enable_filters(&mut log_state, SYSTEM_INFO_DIAGNOSTICS);
            } else {
                disable_filters(&mut log_state, SYSTEM_INFO_DIAGNOSTICS);
            }
        }
    }
}

/// Helper function to enable multiple diagnostics at once.
/// The `impl IntoIterator` parameter is a Rust idiom that accepts arrays,
/// vectors, or any other iterable collection of DiagnosticPath items.
fn enable_filters(
    log_state: &mut LogDiagnosticsState,
    diagnostics: impl IntoIterator<Item = DiagnosticPath>,
) {
    // extend_filter adds multiple diagnostics to the "allowed" list efficiently
    log_state.extend_filter(diagnostics);
}

/// Helper function to disable multiple diagnostics at once.
/// This demonstrates the manual iteration approach when a bulk method isn't available.
fn disable_filters(
    log_state: &mut LogDiagnosticsState,
    diagnostics: impl IntoIterator<Item = DiagnosticPath>,
) {
    // Remove each diagnostic individually from the filter list
    for diagnostic in diagnostics {
        log_state.remove_filter(&diagnostic);
    }
}

fn update_commands(
    mut commands: Commands,
    log_commands: Single<Entity, With<LogDiagnosticsCommands>>,
    status: Res<LogDiagnosticsStatus>,
    filters: Res<LogDiagnosticsFilters>,
) {
    let enabled = *status == LogDiagnosticsStatus::Enabled;
    let alpha = if enabled { 1. } else { 0.25 };
    let enabled_color = |enabled| {
        if enabled {
            Color::from(palettes::tailwind::GREEN_400)
        } else {
            Color::from(palettes::tailwind::RED_400)
        }
    };
    commands
        .entity(*log_commands)
        .despawn_related::<Children>()
        .insert(children![
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.),
                    ..default()
                },
                children![
                    Text::new("[Q] Toggle filtering:"),
                    (
                        Text::new(format!("{:?}", *status)),
                        TextColor(enabled_color(enabled))
                    )
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.),
                    ..default()
                },
                children![
                    (
                        Text::new("[1] Frame times:"),
                        TextColor(Color::WHITE.with_alpha(alpha))
                    ),
                    (
                        Text::new(format!("{:?}", filters.frame_time)),
                        TextColor(enabled_color(filters.frame_time).with_alpha(alpha))
                    )
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.),
                    ..default()
                },
                children![
                    (
                        Text::new("[2] Entity count:"),
                        TextColor(Color::WHITE.with_alpha(alpha))
                    ),
                    (
                        Text::new(format!("{:?}", filters.entity_count)),
                        TextColor(enabled_color(filters.entity_count).with_alpha(alpha))
                    )
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.),
                    ..default()
                },
                children![
                    (
                        Text::new("[3] System info:"),
                        TextColor(Color::WHITE.with_alpha(alpha))
                    ),
                    (
                        Text::new(format!("{:?}", filters.system_info)),
                        TextColor(enabled_color(filters.system_info).with_alpha(alpha))
                    )
                ]
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(5.),
                    ..default()
                },
                children![
                    (
                        Text::new("[4] Render diagnostics:"),
                        TextColor(Color::WHITE.with_alpha(alpha))
                    ),
                    (
                        Text::new("Private"),
                        TextColor(enabled_color(false).with_alpha(alpha))
                    )
                ]
            ),
        ]);
}

/// Resource that tracks whether diagnostic filtering is active.
/// This is a simple state machine with two modes:
#[derive(Debug, Default, PartialEq, Eq, Resource)]
enum LogDiagnosticsStatus {
    /// No filtering, showing all logs - the "fire hose" mode
    /// Useful when you want to see everything happening in your game
    #[default]
    Disabled,
    
    /// Filtering enabled, showing only subset of logs - the "focused" mode
    /// Useful when you want to concentrate on specific performance aspects
    Enabled,
}

/// Resource that stores which diagnostic categories should be displayed.
/// This is like a checklist of what you want to monitor.
#[derive(Default, Resource)]
struct LogDiagnosticsFilters {
    frame_time: bool,    // Show FPS, frame time, and frame count?
    entity_count: bool,  // Show how many entities exist?
    system_info: bool,   // Show CPU and memory usage?
    
    // Note: Render diagnostics are available but their paths are private,
    // so we can't toggle them individually in this example.
    #[expect(
        dead_code,
        reason = "Currently the diagnostic paths referent to RenderDiagnosticPlugin are private"
    )]
    render_diagnostics: bool, // Would control render pipeline diagnostics
}

/// Component marker for the UI element that displays diagnostic controls.
/// This is a "tag" component - it has no data, just marks an entity for identification.
/// The UI system uses this to find and update the instruction panel.
#[derive(Component)]
struct LogDiagnosticsCommands;
