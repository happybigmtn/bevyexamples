//! Showcases wireframe rendering.
//!
//! # Wireframe Rendering: See-Through Mesh Structure
//!
//! Wireframe mode shows the underlying triangle structure of 3D models,
//! displaying only the edges of polygons. It's incredibly useful for:
//!
//! ## Development & Debugging:
//! - **Mesh Topology**: Check triangle distribution
//! - **LOD Analysis**: Compare detail levels
//! - **Performance**: Count polygon density
//! - **Art Direction**: Verify model structure
//!
//! ## Artistic Effects:
//! - **Sci-fi Aesthetics**: Holographic displays
//! - **Technical Drawings**: Blueprint style
//! - **Retro Gaming**: Classic 3D wireframe look
//! - **X-ray Views**: See internal structure
//!
//! ## Platform Support:
//! Wireframes currently do not work when using webgl or webgpu.
//! Supported platforms:
//! - DX12
//! - Vulkan
//! - Metal
//!
//! This is a native only feature.

// Rust: Complex nested imports from Bevy
use bevy::{
    // Rust: Glob import of CSS color constants
    // * imports RED, BLUE, WHITE, etc.
    color::palettes::css::*,
    // Rust: Wireframe-specific components and configuration
    pbr::wireframe::{
        NoWireframe,        // Component: prevents wireframe
        Wireframe,          // Component: forces wireframe
        WireframeColor,     // Component: custom wireframe color
        WireframeConfig,    // Resource: global wireframe settings
        WireframePlugin,    // Plugin: enables wireframe system
    },
    // Rust: Common Bevy types
    prelude::*,
    // Rust: Low-level rendering configuration
    render::{
        render_resource::WgpuFeatures,  // GPU features enum
        settings::{RenderCreation, WgpuSettings},  // Render settings
        RenderPlugin,  // Core rendering plugin
    },
};

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Add multiple plugins as tuple
        .add_plugins((
            // Rust: Configure DefaultPlugins with custom RenderPlugin
            DefaultPlugins.set(RenderPlugin {
                // Rust: Render creation settings
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARN this is a native only feature. It will not work with webgl or webgpu
                    // Rust: Enable GPU feature for wireframe rendering
                    // POLYGON_MODE_LINE allows drawing triangles as lines
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    // Rust: Use default for other settings
                    ..default()
                }),
                ..default()
            }),
            // You need to add this plugin to enable wireframe rendering
            // Rust: Default wireframe plugin configuration
            WireframePlugin::default(),
        ))
        // Wireframes can be configured with this resource. This can be changed at runtime.
        // Rust: Insert global wireframe configuration
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            // Rust: bool field - enable wireframes globally
            global: true,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            // Rust: Color conversion with .into()
            // CSS WHITE constant -> LinearRgba via Into trait
            default_color: WHITE.into(),
        })
        // Rust: System registration
        .add_systems(Startup, setup)
        .add_systems(Update, update_colors)
        // Rust: Start the game loop
        .run();
}

/// set up a simple 3D scene
// Rust: System function with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Mutable access to mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to material assets
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Red cube: Never renders a wireframe
    // Rust: Spawn entity with wireframe disabled
    commands.spawn((
        // Rust: Create default cube mesh (1x1x1)
        Mesh3d(meshes.add(Cuboid::default())),
        // Rust: Convert CSS color constant to material
        // Color::from() trait conversion from CSS color
        MeshMaterial3d(materials.add(Color::from(RED))),
        // Rust: Position in 3D space
        Transform::from_xyz(-1.0, 0.5, -1.0),
        // Rust: NoWireframe marker prevents wireframe rendering
        // Even with global wireframes enabled
        NoWireframe,
    ));
    
    // Orange cube: Follows global wireframe setting
    // Rust: Entity without wireframe components
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::from(ORANGE))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        // Rust: No wireframe components = follows global setting
    ));
    
    // Green cube: Always renders a wireframe
    // Rust: Entity with forced wireframe
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::from(LIME))),
        Transform::from_xyz(1.0, 0.5, 1.0),
        // Rust: Wireframe marker forces wireframe rendering
        // Ignores global setting
        Wireframe,
        // This lets you configure the wireframe color of this entity.
        // If not set, this will use the color in `WireframeConfig`
        // Rust: Custom wireframe color component
        WireframeColor { color: LIME.into() },
    ));

    // plane
    // Rust: Ground plane with custom wireframe color
    commands.spawn((
        // Rust: Create 5x5 plane mesh
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::from(BLUE))),
        // You can insert this component without the `Wireframe` component
        // to override the color of the global wireframe for this mesh
        // Rust: Override global wireframe color without forcing wireframe
        WireframeColor {
            color: BLACK.into(),  // Black wireframe on blue surface
        },
    ));

    // light
    // Rust: Simple point light at elevated position
    commands.spawn((
        PointLight::default(), 
        Transform::from_xyz(2.0, 4.0, 2.0)
    ));

    // camera
    commands.spawn((
        // Rust: Default camera configuration
        Camera3d::default(),
        // Rust: Camera positioned to view all cubes
        Transform::from_xyz(-2.0, 2.5, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Text used to show controls
    // Rust: UI text for displaying controls
    commands.spawn((
        // Rust: Default text component (empty initially)
        Text::default(),
        // Rust: UI node for positioning
        Node {
            // Rust: Absolute positioning
            position_type: PositionType::Absolute,
            // Rust: Val::Px for pixel values
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

/// This system let's you toggle various wireframe settings
// Rust: Interactive system for wireframe control
fn update_colors(
    // Rust: Keyboard input resource
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Rust: Mutable access to global wireframe configuration
    mut config: ResMut<WireframeConfig>,
    // Rust: Query for entities with both WireframeColor and Wireframe
    // With<Wireframe> filter ensures only forced wireframes
    mut wireframe_colors: Query<&mut WireframeColor, With<Wireframe>>,
    // Rust: Single query for the UI text
    mut text: Single<&mut Text>,
) {
    // Rust: Update UI text with current settings
    // format! macro creates formatted string
    text.0 = format!(
        "Controls
---------------
Z - Toggle global
X - Change global color
C - Change color of the green cube wireframe

WireframeConfig
-------------
Global: {}
Color: {:?}",
        // Rust: Access config fields
        config.global, config.default_color,
    );

    // Toggle showing a wireframe on all meshes
    // Rust: just_pressed() detects single frame key press
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        // Rust: Boolean negation with !
        config.global = !config.global;
    }

    // Toggle the global wireframe color
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        // Rust: Conditional assignment with color comparison
        // == works because Color implements PartialEq
        config.default_color = if config.default_color == WHITE.into() {
            DEEP_PINK.into()  // Switch to hot pink
        } else {
            WHITE.into()      // Switch back to white
        };
    }

    // Toggle the color of a wireframe using WireframeColor and not the global color
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        // Rust: Iterate over mutable query results
        for mut color in &mut wireframe_colors {
            // Rust: Modify color field with conditional logic
            color.color = if color.color == LIME.into() {
                RED.into()   // Green -> Red
            } else {
                LIME.into() // Red -> Green
            };
        }
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **GPU Features**:
//    - `WgpuFeatures::POLYGON_MODE_LINE` enables wireframe
//    - Platform-specific rendering capabilities
//    - Requires native graphics API support
//
// 2. **Marker Components**:
//    - `Wireframe` - Forces wireframe rendering
//    - `NoWireframe` - Prevents wireframe rendering
//    - Zero-size types for entity behavior
//
// 3. **Color Conversions**:
//    - `Color::from(RED)` - CSS constant to Color
//    - `WHITE.into()` - Into trait conversion
//    - Multiple ways to create colors
//
// 4. **Query Filters**:
//    - `With<Wireframe>` - Only entities with component
//    - Precise entity selection
//    - Efficient system targeting
//
// 5. **Single Query**:
//    - More efficient for unique entities
//    - `.0` accesses wrapped value
//    - Good for UI text elements
//
// 6. **Resource Mutation**:
//    - `ResMut<WireframeConfig>` for global settings
//    - Changes affect all entities
//    - Runtime configuration updates
//
// 7. **Boolean Logic**:
//    - `!config.global` - Negation operator
//    - `==` comparison for colors
//    - Conditional expressions for toggling
