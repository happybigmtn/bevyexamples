//! Demonstrates Bevy's built-in postprocessing features.
//!
//! Currently, this simply consists of chromatic aberration.
//!
//! # What is Post-Processing?
//!
//! Post-processing applies visual effects AFTER the scene is rendered.
//! Think of it like Instagram filters for your game - the image is already
//! taken, but you apply effects to enhance or stylize it.
//!
//! ## Chromatic Aberration
//!
//! This effect simulates a lens defect where different colors of light
//! don't converge at the same point. You've seen this:
//! - Rainbow fringes on the edges of objects
//! - Common in cheap cameras or VR headsets
//! - Used artistically to create a retro/analog feel
//! - Or to simulate damaged/low-quality optics
//!
//! ## How It Works
//!
//! 1. The scene renders normally to a texture
//! 2. Post-processing shader reads that texture
//! 3. Red, green, and blue channels are offset slightly
//! 4. Result: Color separation at edges, especially visible on high-contrast areas

// Rust: Importing from the standard library
// `std::f32::consts::PI` gives us the mathematical constant π (3.14159...)
// The `::` is the path separator in Rust, like `/` in file paths
use std::f32::consts::PI;

// Rust: Importing from external crates (dependencies)
// `use` brings items into scope so we don't need full paths
use bevy::{
    // Destructuring import - we're pulling specific items from modules
    core_pipeline::post_process::ChromaticAberration, // The post-process effect
    pbr::CascadeShadowConfigBuilder,                  // For shadow cascades
    prelude::*,                                        // Common Bevy types
    render::view::Hdr,                                 // High Dynamic Range flag
};

/// The number of units per frame to add to or subtract from intensity when the
/// arrow keys are held.
// Rust: `const` defines compile-time constants
// - UPPER_SNAKE_CASE naming convention for constants
// - Type annotation `: f32` is required for const
// - Value must be known at compile time
const CHROMATIC_ABERRATION_INTENSITY_ADJUSTMENT_SPEED: f32 = 0.002;

/// The maximum supported chromatic aberration intensity level.
// Rust: f32 is a 32-bit floating-point number
// - Use f64 for double precision (rarely needed in games)
// - Literals like 0.4 default to f64, but type annotation makes it f32
const MAX_CHROMATIC_ABERRATION_INTENSITY: f32 = 0.4;

/// The settings that the user can control.
// Rust: Derive macros generate code at compile time
// `#[derive(Resource)]` implements the Resource trait automatically
// This makes AppSettings usable as a Bevy resource (global data)
#[derive(Resource)]
// Rust: Structs are custom data types that group related data
// - Use `struct` for data, `enum` for variants
// - Fields can be public (pub) or private (default)
struct AppSettings {
    /// The intensity of the chromatic aberration effect.
    // Rust: Structure fields
    // - No `pub` keyword means this field is private to the module
    // - Type annotation is required for struct fields
    chromatic_aberration_intensity: f32,
}

/// The entry point.
// Rust: Every executable needs exactly one `main` function
// - No return type means it returns `()` (unit type, like void)
// - Could also write `fn main() -> ()` explicitly
fn main() {
    // Rust: Method chaining pattern (builder pattern)
    // Each method returns Self, allowing `.method().method()` chains
    App::new()
        // Rust: Turbofish syntax `::<Type>` specifies generic type
        // Tells Rust which concrete type to use for generic parameter
        .init_resource::<AppSettings>()
        // Rust: Method calls can take complex expressions
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // Rust: Struct construction syntax
            // Field: value pairs, order doesn't matter
            primary_window: Some(Window {
                // Rust: `.into()` converts between types
                // Here: &str -> String (many types implement Into<String>)
                title: "Bevy Chromatic Aberration Example".into(),
                // Rust: Struct update syntax `..expr`
                // Fills remaining fields from another instance
                // `default()` creates instance with default values
                ..default()
            }),
            ..default()
        }))
        // Rust: Functions are first-class values
        // Can pass function names as arguments
        .add_systems(Startup, setup)
        .add_systems(Update, handle_keyboard_input)
        .add_systems(
            Update,
            // Rust: Tuple of functions - multiple systems at once
            (update_chromatic_aberration_settings, update_help_text)
                // Rust: Conditional execution
                // Systems only run when resource changes
                .run_if(resource_changed::<AppSettings>)
                // Rust: System ordering
                // These systems run after keyboard input
                .after(handle_keyboard_input),
        )
        // Rust: Consumes App and starts the game loop
        // Never returns (loops forever or until exit)
        .run();
}

/// Creates the example scene and spawns the UI.
// Rust: Function parameters in Bevy systems
// - `mut commands: Commands` - Mutable access to spawn entities
// - `asset_server: Res<AssetServer>` - Read-only resource access
// - `app_settings: Res<AppSettings>` - Another read-only resource
// Bevy automatically provides these based on the function signature!
fn setup(
    // Rust: `mut` keyword makes the binding mutable
    // Commands needs to be mutable to spawn entities
    mut commands: Commands,
    // Rust: Res<T> is a "smart pointer" to a resource
    // Provides read-only access, automatically dereferenced
    asset_server: Res<AssetServer>,
    app_settings: Res<AppSettings>
) {
    // Spawn the camera.
    // Rust: `&mut` creates a mutable reference (borrow)
    // Allows function to modify without taking ownership
    spawn_camera(&mut commands, &asset_server);

    // Create the scene.
    // Rust: References are explicit in Rust
    // `&` for immutable reference, `&mut` for mutable
    spawn_scene(&mut commands, &asset_server);

    // Spawn the help text.
    // Rust: Can borrow from smart pointers like Res<T>
    // `&app_settings` borrows from the Res wrapper
    spawn_text(&mut commands, &app_settings);
}

/// Spawns the camera, including the [`ChromaticAberration`] component.
// Rust: Function takes references as parameters
// - `&mut Commands` - mutable reference (can modify)
// - `&AssetServer` - immutable reference (read-only)
fn spawn_camera(commands: &mut Commands, asset_server: &AssetServer) {
    // Rust: Tuple bundle pattern in Bevy
    // Single spawn call with multiple components
    commands.spawn((
        // Rust: Associated function `default()`
        // Type::default() creates default instance
        Camera3d::default(),
        // Rust: Unit struct (no fields)
        // Just a marker component
        Hdr,
        // Rust: Builder pattern with method chaining
        // Each method returns Self for chaining
        Transform::from_xyz(0.7, 0.7, 1.0)
            .looking_at(
                // Rust: Associated function `new`
                // Common pattern for constructors
                Vec3::new(0.0, 0.3, 0.0), 
                // Rust: Associated constant
                // Vec3::Y is const Vec3 { x: 0.0, y: 1.0, z: 0.0 }
                Vec3::Y
            ),
        // Rust: Struct literal with named fields
        DistanceFog {
            // Rust: Method call on type
            color: Color::srgb_u8(43, 44, 47),
            // Rust: Enum with data
            falloff: FogFalloff::Linear {
                start: 1.0,
                end: 8.0,
            },
            // Rust: Fill rest from default
            ..default()
        },
        EnvironmentMapLight {
            // Rust: Method calls can be used in struct construction
            // `load` returns a Handle<Image>
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            // Rust: f32 literal with decimal point
            intensity: 2000.0,
            ..default()
        },
        // Include the `ChromaticAberration` component.
        ChromaticAberration::default(),
    ));
}

/// Spawns the scene.
///
/// This is just the tonemapping test scene, chosen for the fact that it uses a
/// variety of colors.
fn spawn_scene(commands: &mut Commands, asset_server: &AssetServer) {
    // Spawn the main scene.
    // Rust: Newtype pattern - SceneRoot wraps a Handle
    commands.spawn(SceneRoot(
        // Rust: Method chaining for asset loading
        asset_server.load(
            // Rust: Enum variant with data
            // Scene(0) means scene at index 0
            GltfAssetLabel::Scene(0)
                // Rust: Builder method to specify asset path
                .from_asset("models/TonemappingTest/TonemappingTest.gltf"),
        )
    ));

    // Spawn the flight helmet.
    // Rust: Tuple bundle with multiple components
    commands.spawn((
        SceneRoot(
            asset_server
                // Rust: Line continuation for readability
                // Compiler ignores whitespace
                .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
        // Rust: Method chaining on Transform
        Transform::from_xyz(0.5, 0.0, -0.5)
            // Rust: Negative float literals
            // -0.15 * PI = approximately -27 degrees
            .with_rotation(Quat::from_rotation_y(-0.15 * PI)),
    ));

    // Spawn the light.
    commands.spawn((
        // Rust: Struct literal
        DirectionalLight {
            // Rust: Float literals can use underscores for readability
            // Could write as 15_000.0
            illuminance: 15000.0,
            // Rust: bool literals: true/false (lowercase)
            shadows_enabled: true,
            ..default()
        },
        // Rust: Associated functions can take multiple parameters
        Transform::from_rotation(Quat::from_euler(
            // Rust: Imported enum variant
            EulerRot::ZYX, 
            0.0, 
            PI * -0.15, 
            PI * -0.15
        )),
        // Rust: Builder pattern
        // Struct builds into another type via .build()
        CascadeShadowConfigBuilder {
            maximum_distance: 3.0,
            first_cascade_far_bound: 0.9,
            ..default()
        }
        // Rust: Method consumes self, returns built type
        .build(),
    ));
}

/// Spawns the help text at the bottom of the screen.
// Rust: Function parameters with references
fn spawn_text(commands: &mut Commands, app_settings: &AppSettings) {
    commands.spawn((
        // Rust: Function call as component
        // Returns Text which implements Component
        create_help_text(app_settings),
        // Rust: UI positioning with Node
        Node {
            // Rust: Enum for positioning strategy
            position_type: PositionType::Absolute,
            // Rust: Enum with associated data
            // Val::Px(f32) represents pixel values
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// Rust: Implementing traits for custom types
// `impl Trait for Type` adds functionality to Type
impl Default for AppSettings {
    // Rust: Associated function of the Default trait
    // Must match trait definition exactly
    fn default() -> Self {
        // Rust: `Self` refers to the implementing type (AppSettings)
        // More maintainable than repeating type name
        Self {
            // Rust: Field access with dot notation
            // Creates default ChromaticAberration, then accesses its intensity field
            chromatic_aberration_intensity: ChromaticAberration::default().intensity,
        }
    }
}

/// Creates help text at the bottom of the screen.
// Rust: Function with return type annotation `-> Text`
fn create_help_text(app_settings: &AppSettings) -> Text {
    // Rust: format! macro creates formatted String
    // {} is placeholder for Display formatting
    format!(
        "Chromatic aberration intensity: {} (Press Left or Right to change)",
        // Rust: Field access through reference
        // Auto-dereferenced: (&AppSettings).field
        app_settings.chromatic_aberration_intensity
    )
    // Rust: .into() type conversion
    // String implements Into<Text>, so this converts String -> Text
    .into()
}

/// Handles requests from the user to change the chromatic aberration intensity.
// Rust: System with mutable and immutable resources
fn handle_keyboard_input(
    // Rust: ResMut<T> provides mutable access to resource
    // Like &mut T but for Bevy resources
    mut app_settings: ResMut<AppSettings>, 
    // Rust: Res<T> provides immutable access
    // Generic type parameter <KeyCode> specifies input type
    input: Res<ButtonInput<KeyCode>>
) {
    // Rust: Mutable local variable
    // Type inferred as f32 from usage
    let mut delta = 0.0;
    
    // Rust: if/else if control flow
    if input.pressed(KeyCode::ArrowLeft) {
        // Rust: Compound assignment operator -=
        // Equivalent to: delta = delta - CONST
        delta -= CHROMATIC_ABERRATION_INTENSITY_ADJUSTMENT_SPEED;
    } else if input.pressed(KeyCode::ArrowRight) {
        // Rust: += compound assignment
        delta += CHROMATIC_ABERRATION_INTENSITY_ADJUSTMENT_SPEED;
    }

    // If no arrow key was pressed, just bail out.
    // Rust: Floating point comparison
    // Note: Generally avoid == with floats due to precision
    // Here it's safe because we set it to exactly 0.0
    if delta == 0.0 {
        // Rust: Early return - no value needed for () return type
        return;
    }

    // Rust: Field access through smart pointer
    // ResMut<T> derefs to &mut T automatically
    app_settings.chromatic_aberration_intensity = 
        // Rust: Parentheses for operation precedence
        (app_settings.chromatic_aberration_intensity + delta)
            // Rust: Method on f32 primitive type
            // Ensures value stays within valid range
            .clamp(0.0, MAX_CHROMATIC_ABERRATION_INTENSITY);
}

/// Updates the [`ChromaticAberration`] settings per the [`AppSettings`].
// Rust: Doc comment links with [`Type`] syntax
fn update_chromatic_aberration_settings(
    // Rust: Query<T> is Bevy's way to access components
    // &mut T means we want mutable access to ChromaticAberration components
    mut chromatic_aberration: Query<&mut ChromaticAberration>,
    app_settings: Res<AppSettings>,
) {
    // Rust: Local variable binding
    // Copies the f32 value (f32 implements Copy trait)
    let intensity = app_settings.chromatic_aberration_intensity;

    // Pick a reasonable maximum sample size for the intensity to avoid an
    // artifact whereby the individual samples appear instead of producing
    // smooth streaks of color.
    //
    // Don't take this formula too seriously; it hasn't been heavily tuned.
    // Rust: Complex arithmetic expression
    let max_samples = 
        // Rust: Order of operations follows math rules
        // Parentheses make it explicit
        ((intensity - 0.02) / (0.20 - 0.02) * 56.0 + 8.0)
            // Rust: Method chaining on numeric types
            .clamp(8.0, 64.0)
            // Rust: round() returns f32
            .round() 
            // Rust: Type casting with `as`
            // Converts f32 to u32 (truncates decimal)
            as u32;

    // Rust: Iterating over query results
    // &mut query gives mutable iterator
    for mut chromatic_aberration in &mut chromatic_aberration {
        // Rust: Mutable access through iterator
        // mut binding allows field modification
        chromatic_aberration.intensity = intensity;
        chromatic_aberration.max_samples = max_samples;
    }
}

/// Updates the help text at the bottom of the screen to reflect the current
/// [`AppSettings`].
// Rust: Multiple doc comment lines with ///
fn update_help_text(
    // Rust: Query for mutable Text components
    mut text: Query<&mut Text>, 
    app_settings: Res<AppSettings>
) {
    // Rust: iter_mut() returns mutable iterator
    // Different from &mut text which borrows the whole query
    for mut text in text.iter_mut() {
        // Rust: Dereference and assign
        // *text dereferences the &mut Text to assign new value
        // Necessary because text is &mut Text, not Text
        *text = create_help_text(&app_settings);
    }
}
