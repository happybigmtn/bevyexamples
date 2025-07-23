//! Demonstrates specular tints and maps.
//!
//! # Specular Tints: Coloring Your Reflections
//!
//! In the real world, different materials reflect light differently. Gold has
//! a warm, yellow reflection. Copper has a reddish tint. Even "white" metals
//! like aluminum have subtle color shifts in their reflections.
//!
//! ## What is Specular Tint?
//!
//! Specular tint controls the COLOR of reflections on non-metallic surfaces.
//! Think of it as "what color should the shiny highlights be?"
//!
//! - **Default**: White highlights (like plastic or ceramic)
//! - **Colored tint**: Colored highlights (like tinted glass or special coatings)
//! - **Texture map**: Variable highlights across the surface
//!
//! ## This Example Shows:
//!
//! - A black sphere that only shows specular reflections
//! - Animated hue shifting for solid color tints
//! - Toggle between solid color and texture-based tints
//! - Environment map lighting for realistic reflections
//!
//! Press SPACE to toggle between solid tint and texture map!

// Rust: Import mathematical constant
use std::f32::consts::PI;

// Rust: Selective imports from Bevy modules
use bevy::{
    // Rust: CSS color palette import
    color::palettes::css::WHITE, 
    // Rust: Skybox for environment rendering
    core_pipeline::Skybox, 
    // Rust: Common Bevy types
    prelude::*, 
    // Rust: HDR rendering component
    render::view::Hdr
};

// Rust: Module-level constants with doc comments
/// The camera rotation speed in radians per frame.
// Rust: const declaration with explicit type
// f32 literal with decimal point
const ROTATION_SPEED: f32 = 0.005;

/// The rate at which the specular tint hue changes in degrees per frame.
const HUE_SHIFT_SPEED: f32 = 0.2;

// Rust: static string slices for UI text
// static variables have 'static lifetime (live for entire program)
// &str is a string slice (reference to string data)
static SWITCH_TO_MAP_HELP_TEXT: &str = "Press Space to switch to a specular map";
static SWITCH_TO_SOLID_TINT_HELP_TEXT: &str = "Press Space to switch to a solid specular tint";

/// The current settings the user has chosen.
// Rust: Derive macro with multiple traits
// Resource makes this globally accessible, Default provides default() method
#[derive(Resource, Default)]
struct AppStatus {
    /// The type of tint (solid or texture map).
    // Rust: Custom enum type as struct field
    tint_type: TintType,
    /// The hue of the solid tint in radians.
    // Rust: f32 field for numeric data
    hue: f32,
}

/// Assets needed by the demo.
// Rust: Resource without Default - requires manual initialization
#[derive(Resource)]
struct AppAssets {
    /// A color tileable 3D noise texture.
    // Rust: Handle<T> is a smart pointer to GPU assets
    // Handle<Image> specifically references texture data
    noise_texture: Handle<Image>,
}

// Rust: Custom trait implementation
// FromWorld trait allows creation from World (Bevy's central data store)
impl FromWorld for AppAssets {
    // Rust: Associated function (no self parameter)
    // Takes mutable reference to World
    fn from_world(world: &mut World) -> Self {
        // Rust: Method call on World to get resource
        // Turbofish syntax ::<AssetServer> specifies exact type
        let asset_server = world.resource::<AssetServer>();
        
        // Rust: Self refers to the implementing type (AppAssets)
        Self {
            // Rust: Method call on asset_server
            // load() returns Handle<Image> for the texture file
            noise_texture: asset_server.load("textures/AlphaNoise.png"),
        }
    }
}

/// The type of specular tint that the user has selected.
// Rust: Enum with multiple derived traits
// Clone + Copy allow cheap duplication, PartialEq enables == comparison
#[derive(Clone, Copy, PartialEq, Default)]
enum TintType {
    /// A solid color.
    // Rust: #[default] attribute specifies default enum variant
    // Required when deriving Default trait for enums
    #[default]
    Solid,
    /// A Perlin noise texture.
    // Rust: Simple enum variant (no associated data)
    Map,
}

/// The entry point.
// Rust: Program main function
fn main() {
    // Rust: App builder pattern with complex configuration
    App::new()
        // Rust: Plugin configuration with method chaining
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            // Rust: Option<Window> with Some variant
            primary_window: Some(Window {
                // Rust: String conversion with .into()
                title: "Bevy Specular Tint Example".into(),
                // Rust: Default remaining fields
                ..default()
            }),
            ..default()
        }))
        // Rust: Resource initialization using FromWorld trait
        // Calls AppAssets::from_world() automatically
        .init_resource::<AppAssets>()
        // Rust: Resource initialization using Default trait
        .init_resource::<AppStatus>()
        // Rust: Manual resource insertion with specific values
        .insert_resource(AmbientLight {
            color: Color::BLACK,    // No ambient light
            brightness: 0.0,        // Completely dark
            ..default()
        })
        // Rust: System registration for different schedules
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_camera)
        // Rust: Tuple of systems with ordering
        // .chain() ensures sequential execution
        .add_systems(Update, (toggle_specular_map, update_text).chain())
        // Rust: System with explicit ordering dependency
        // .after() ensures this runs after toggle_specular_map
        .add_systems(Update, shift_hue.after(toggle_specular_map))
        .run();
}

/// Creates the scene.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    app_status: Res<AppStatus>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawns a camera.
    commands.spawn((
        Transform::from_xyz(-2.0, 0.0, 3.5).looking_at(Vec3::ZERO, Vec3::Y),
        Hdr,
        Camera3d::default(),
        Skybox {
            image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            brightness: 3000.0,
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            // We want relatively high intensity here in order for the specular
            // tint to show up well.
            intensity: 25000.0,
            ..default()
        },
    ));

    // Spawn the sphere.
    commands.spawn((
        Transform::from_rotation(Quat::from_rotation_x(PI * 0.5)),
        Mesh3d(meshes.add(Sphere::default().mesh().uv(32, 18))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            // We want only reflected specular light here, so we set the base
            // color as black.
            base_color: Color::BLACK,
            reflectance: 1.0,
            specular_tint: Color::hsva(app_status.hue, 1.0, 1.0, 1.0),
            // The object must not be metallic, or else the reflectance is
            // ignored per the Filament spec:
            //
            // <https://google.github.io/filament/Filament.html#listing_fnormal>
            metallic: 0.0,
            perceptual_roughness: 0.0,
            ..default()
        })),
    ));

    // Spawn the help text.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        app_status.create_text(),
    ));
}

/// Rotates the camera a bit every frame.
fn rotate_camera(mut cameras: Query<&mut Transform, With<Camera3d>>) {
    for mut camera_transform in cameras.iter_mut() {
        camera_transform.translation =
            Quat::from_rotation_y(ROTATION_SPEED) * camera_transform.translation;
        camera_transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

/// Alters the hue of the solid color a bit every frame.
// Rust: System function with multiple parameter types
fn shift_hue(
    // Rust: Mutable access to app state resource
    mut app_status: ResMut<AppStatus>,
    // Rust: Query for material handles on objects
    objects_with_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    // Rust: Mutable access to material asset storage
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Rust: Early return pattern with != comparison
    // Only animate hue for solid tints, not texture maps
    if app_status.tint_type != TintType::Solid {
        return;  // Exit function early
    }

    // Rust: Compound assignment operator +=
    // Incrementally shifts hue over time for rainbow effect
    app_status.hue += HUE_SHIFT_SPEED;

    // Rust: Iterate over query results
    for material_handle in objects_with_materials.iter() {
        // Rust: let-else pattern (Rust 1.65+)
        // Combines pattern matching with early continue
        let Some(material) = standard_materials.get_mut(material_handle) else {
            continue;  // Skip if material not found
        };
        
        // Rust: HSVA color creation with animated hue
        // hsva(hue, saturation, value, alpha)
        material.specular_tint = Color::hsva(app_status.hue, 1.0, 1.0, 1.0);
    }
}

// Rust: Implementation block for custom methods
impl AppStatus {
    /// Returns appropriate help text that reflects the current app status.
    // Rust: Method with immutable self reference
    // &self allows reading but not modifying the struct
    fn create_text(&self) -> Text {
        // Rust: Pattern matching on enum field
        // match expression returns a value based on enum variant
        let tint_map_help_text = match self.tint_type {
            // Rust: Enum variant patterns
            TintType::Solid => SWITCH_TO_MAP_HELP_TEXT,
            TintType::Map => SWITCH_TO_SOLID_TINT_HELP_TEXT,
        };

        // Rust: Function call with string slice
        // Text::new() constructor takes &str parameter
        Text::new(tint_map_help_text)
    }
}

/// Changes the specular tint to a solid color or map when the user presses
/// Space.
// Rust: Complex system with multiple resource and query parameters
fn toggle_specular_map(
    // Rust: Keyboard input resource for detecting key presses
    keyboard: Res<ButtonInput<KeyCode>>,
    // Rust: Mutable app state for toggling mode
    mut app_status: ResMut<AppStatus>,
    // Rust: Asset resource for accessing loaded textures
    app_assets: Res<AppAssets>,
    // Rust: Query for finding objects with materials
    objects_with_materials: Query<&MeshMaterial3d<StandardMaterial>>,
    // Rust: Mutable access to material asset storage
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Rust: Early return with negation operator !
    // Only proceed if Space was pressed this frame
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    // Swap tint type.
    // Rust: Pattern matching for state toggling
    // match expression assigns new value based on current state
    app_status.tint_type = match app_status.tint_type {
        TintType::Solid => TintType::Map,   // Solid -> Map
        TintType::Map => TintType::Solid,   // Map -> Solid
    };

    // Rust: Iterate through all objects with materials
    for material_handle in objects_with_materials.iter() {
        // Rust: let-else pattern for Option handling
        let Some(material) = standard_materials.get_mut(material_handle) else {
            continue;  // Skip if material asset not found
        };

        // Adjust the tint type.
        // Rust: Pattern matching for different configuration modes
        match app_status.tint_type {
            TintType::Solid => {
                // Rust: Direct field assignment
                material.reflectance = 1.0;
                // Rust: Option<Handle<Image>> set to None
                material.specular_tint_texture = None;
            }
            TintType::Map => {
                // Set reflectance to 2.0 to spread out the map's reflectance
                // range from the default [0.0, 0.5] to [0.0, 1.0].
                material.reflectance = 2.0;
                
                // As the tint map is multiplied by the tint color, we set the
                // latter to white so that only the map has an effect.
                // Rust: Into trait conversion from color constant
                material.specular_tint = WHITE.into();
                
                // Rust: Option<Handle<Image>> with Some variant
                // Clone Handle (cheap - reference counted)
                material.specular_tint_texture = Some(app_assets.noise_texture.clone());
            }
        };
    }
}

/// Updates the help text at the bottom of the screen to reflect the current app
/// status.
// Rust: Simple system for UI updates
fn update_text(
    // Rust: Query for mutable text components
    mut text_query: Query<&mut Text>, 
    // Rust: Read app state to determine what text to show
    app_status: Res<AppStatus>
) {
    // Rust: Iterate over all text entities
    for mut text in text_query.iter_mut() {
        // Rust: Dereference and assignment
        // *text assigns new Text value, replacing the old one
        *text = app_status.create_text();
    }
}

// 🎯 Advanced Rust Concepts in This Example:
//
// 1. **let-else Pattern** (Rust 1.65+):
//    - `let Some(x) = option else { continue; }`
//    - Combines pattern matching with early exit
//    - More concise than if-let + else
//
// 2. **FromWorld Trait**:
//    - Custom initialization from World context
//    - Allows access to other resources during creation
//    - Alternative to Default when you need dependencies
//
// 3. **Static vs Const**:
//    - `static` - Global variable with fixed memory location
//    - `const` - Compile-time constant, inlined at use sites
//    - String literals usually use static for efficiency
//
// 4. **System Ordering**:
//    - `.chain()` - Sequential execution within same schedule
//    - `.after()` - Explicit dependency ordering
//    - Important for systems that depend on each other
//
// 5. **Enum Default Variants**:
//    - `#[default]` attribute specifies which variant is default
//    - Required when deriving Default for enums
//    - Makes code more explicit about intended defaults
//
// 6. **Asset Handle Cloning**:
//    - `Handle<T>` implements Clone (cheap operation)
//    - Reference-counted smart pointer to GPU resources
//    - Multiple entities can share same asset efficiently
//
// 7. **Option Field Patterns**:
//    - `Some(handle)` vs `None` for optional textures
//    - Allows materials to conditionally use features
//    - Common pattern in graphics programming

