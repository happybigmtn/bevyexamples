//! Demonstrates clustered decals, which affix decals to surfaces.
//!
//! # What Are Clustered Decals?
//! 
//! Imagine you want to paint graffiti on a wall, add bullet holes to a surface, or place
//! stickers on objects in your 3D world. You could modify the textures of every object,
//! but that's inefficient and inflexible. Instead, we use decals - 2D images that get
//! projected onto 3D surfaces.
//!
//! "Clustered" refers to how these decals are organized for efficient rendering. Just like
//! how clustered forward rendering divides the view into spatial clusters to handle many
//! lights efficiently, clustered decals use the same system to handle many decals without
//! checking every decal against every pixel.
//!
//! # How This Example Works
//!
//! This example creates a rotating cube and lets you place two decals on it. You can:
//! - Move the camera or decals around the cube
//! - Scale the decals (make them bigger or smaller)
//! - Roll the decals (rotate them around their forward axis)
//!
//! The decals use a custom shader that tints them different colors based on a "tag" value,
//! demonstrating how you can customize decal appearance beyond just the texture.
//!
//! # Graphics Theory: Deferred Decal Rendering
//!
//! Clustered decals build upon deferred rendering concepts. Here's the mathematical foundation:
//!
//! ## Projection Mathematics
//! Decals are essentially oriented bounding boxes (OBBs) that project textures:
//! - **Projection Matrix**: P = M_view * M_proj * M_decal
//! - **UV Calculation**: uv = (world_pos - decal_center) * inverse(decal_transform)
//! - **Depth Testing**: z_test = dot(world_normal, decal_forward) > threshold
//!
//! ## Clustering Algorithm
//! The view frustum is divided into a 3D grid of clusters (typically 16×16×24):
//! 1. **Z-Slicing**: Logarithmic depth partitioning for better near/far distribution
//! 2. **Tile Assignment**: Each decal is assigned to overlapping clusters
//! 3. **Bitmasking**: Clusters store decal indices in a compact bitfield
//!
//! ## GPU Memory Layout
//! ```text
//! Cluster Buffer: [cluster_id] -> [decal_count, decal_indices...]
//! Decal Buffer: [decal_id] -> [transform, texture_id, parameters]
//! ```
//!
//! # Game Design Context: Environmental Storytelling
//!
//! Decals are crucial for environmental narrative:
//!
//! ## Visual History
//! - **Battle Damage**: Bullet holes, scorch marks tell of past conflicts
//! - **Wear Patterns**: Footprints, scratches show how spaces are used
//! - **Graffiti/Signage**: Adds personality and world-building details
//!
//! ## Dynamic Feedback
//! - **Player Actions**: Blood splatter, explosion marks provide combat feedback
//! - **Navigation Aids**: Projected arrows, markers guide players subtly
//! - **Emotional Tone**: Decay, grime create atmosphere without changing geometry
//!
//! ## Level Design Integration
//! 1. **Modular Environments**: Same geometry, different decals = new locations
//! 2. **Performance Budget**: Decals cheaper than unique textures per object
//! 3. **Runtime Variety**: Procedural decal placement prevents repetition
//!
//! # Performance Deep Dive: GPU Optimization
//!
//! ## Bandwidth Optimization
//! Clustered decals minimize overdraw through spatial culling:
//! - **Traditional**: O(N × M) where N=pixels, M=decals
//! - **Clustered**: O(N × K) where K=decals per cluster (typically < 10)
//!
//! ## Cache Efficiency
//! ```rust
//! // Optimal memory access pattern
//! for cluster in visible_clusters {
//!     let decal_list = cluster_buffer[cluster.id];
//!     for decal_idx in decal_list {
//!         // Coherent memory access - nearby pixels access same decals
//!         apply_decal(decal_buffer[decal_idx]);
//!     }
//! }
//! ```
//!
//! ## GPU Occupancy
//! - **Warp Divergence**: Minimized by cluster coherency
//! - **Register Pressure**: Shared decal data reduces per-thread storage
//! - **Texture Cache**: Spatial locality improves texture fetch efficiency
//!
//! # Real-World Applications: Industry Usage
//!
//! ## AAA Game Examples
//! - **The Last of Us**: Moss, blood, graffiti create post-apocalyptic atmosphere
//! - **DOOM Eternal**: Dynamic gore system using projected decals
//! - **Star Citizen**: Ship damage visualization with layered decals
//!
//! ## Technical Implementation
//! - **Unreal Engine 5**: Mesh decals with Nanite integration
//! - **Unity HDRP**: Decal layers with material property overrides
//! - **CryEngine**: Deferred decals with parallax occlusion mapping
//!
//! ## Production Workflows
//! 1. **Artist Pipeline**: Substance Designer -> Decal Atlas -> Engine
//! 2. **Memory Budgets**: Typically 64-256MB for decal textures
//! 3. **LOD Systems**: Distance-based decal culling and quality reduction
//!
//! # Advanced Techniques: Next-Level Implementation
//!
//! ## Screen-Space Decals
//! For even better performance on complex geometry:
//! ```glsl
//! // Fragment shader pseudocode
//! vec3 world_pos = reconstruct_world_position(depth_texture, uv);
//! vec3 decal_uv = world_to_decal_space(world_pos, decal_matrix);
//! if (all(greaterThan(decal_uv, vec3(0))) && all(lessThan(decal_uv, vec3(1)))) {
//!     out_color = mix(out_color, sample_decal(decal_uv.xy), decal_alpha);
//! }
//! ```
//!
//! ## Volumetric Decals
//! - **3D Textures**: Full volume projection for fog, clouds
//! - **Ray Marching**: Integration along view ray through decal volume
//! - **Temporal Filtering**: Reduce noise in volumetric calculations
//!
//! ## Material Blending
//! - **Normal Blending**: Reoriented Normal Mapping (RNM) for correct lighting
//! - **PBR Integration**: Metallic/roughness modification per decal
//! - **Multilayer System**: Decal ordering and blend modes
//!
//! # Debugging and Profiling: Diagnostic Tools
//!
//! ## Visual Debugging
//! ```rust
//! // Common debug visualizations
//! fn debug_draw_decals(gizmos: &mut Gizmos, decals: &Query<&ClusteredDecal>) {
//!     // 1. Decal bounds - shows projection volume
//!     // 2. Cluster grid - visualizes spatial partitioning
//!     // 3. Overdraw heatmap - identifies performance hotspots
//!     // 4. Mip level visualization - texture sampling quality
//! }
//! ```
//!
//! ## Performance Metrics
//! - **GPU Timers**: Measure decal pass duration (target < 2ms)
//! - **Overdraw Factor**: Track pixels touched multiple times
//! - **Memory Bandwidth**: Monitor texture fetch rates
//! - **Cluster Occupancy**: Ensure even distribution
//!
//! ## Common Issues
//! 1. **Z-Fighting**: Decals flickering with surface - add depth bias
//! 2. **Texture Bleeding**: Atlas padding prevents color leaks
//! 3. **Performance Spikes**: Too many decals in one cluster
//! 4. **Visual Artifacts**: Normal map discontinuities at edges
//!
//! ## Profiler Integration
//! ```rust
//! puffin::profile_scope!("clustered_decals");
//! // RenderDoc markers for GPU debugging
//! // NSight/PIX events for detailed analysis
//! ```

// Rust's standard library imports for mathematical constants and formatting
use std::f32::consts::{FRAC_PI_3, PI}; // FRAC_PI_3 = π/3, PI = π
use std::fmt::{self, Formatter};       // For implementing Display trait (pretty printing)
use std::process;                      // For process::exit() if decals aren't supported

use bevy::{
    // CSS color constants for visual styling - these are sRGB colors
    color::palettes::css::{LIME, ORANGE_RED, SILVER},
    // Tracks accumulated mouse movement between frames for smooth dragging
    input::mouse::AccumulatedMouseMotion,
    pbr::{
        // The decal module and ClusteredDecal component for projecting images onto surfaces
        decal::{self, clustered::ClusteredDecal},
        // ExtendedMaterial allows us to add custom shader logic to the standard material
        ExtendedMaterial, MaterialExtension,
    },
    prelude::*,
    render::{
        // AsBindGroup generates GPU bindings, ShaderRef references our custom shader
        render_resource::{AsBindGroup, ShaderRef},
        // Device/adapter info to check if the GPU supports clustered decals
        renderer::{RenderAdapter, RenderDevice},
    },
    // Cursor icon support for visual feedback during different operations
    window::SystemCursorIcon,
    winit::cursor::CursorIcon,
};
use ops::{acos, cos, sin};
use widgets::{
    WidgetClickEvent, WidgetClickSender, BUTTON_BORDER, BUTTON_BORDER_COLOR,
    BUTTON_BORDER_RADIUS_SIZE, BUTTON_PADDING,
};

#[path = "../helpers/widgets.rs"]
mod widgets;

/// The custom material shader that we use to demonstrate how to use the decal
/// `tag` field.
/// 
/// This shader extends Bevy's standard PBR shader with custom logic that reads
/// the decal's "tag" value and uses it to tint the decal different colors.
/// Tags are useful for categorizing decals (e.g., damage decals vs decorative decals)
/// and applying different visual effects to each category.
///
/// ## Shader Architecture Deep Dive
/// 
/// Modern GPU architectures process shaders in parallel SIMD units (warps/wavefronts):
/// - **NVIDIA**: 32 threads per warp
/// - **AMD**: 64 threads per wavefront
/// - **Intel**: Variable EU width
///
/// Clustered decals minimize divergence by ensuring neighboring pixels
/// (which execute together) access the same decal list.
///
/// ## Memory Access Patterns
/// 
/// The shader accesses memory in this order:
/// 1. **Cluster lookup**: Coalesced read from cluster buffer
/// 2. **Decal list**: Sequential access to decal indices
/// 3. **Texture fetch**: 2D texture cache with spatial locality
///
/// This pattern maximizes L1/L2 cache hit rates on modern GPUs.
const SHADER_ASSET_PATH: &str = "shaders/custom_clustered_decal.wgsl";

/// The speed at which the cube rotates, in radians per frame.
/// At 60 FPS, this is about 1.2 radians/second or ~69 degrees/second.
const CUBE_ROTATION_SPEED: f32 = 0.02;

/// The speed at which the selection can be moved, in spherical coordinate
/// radians per mouse unit.
/// This controls how fast objects orbit when you drag the mouse.
/// Spherical coordinates represent 3D positions using:
/// - radius (distance from origin)
/// - theta (angle from vertical Y axis)
/// - phi (angle around Y axis)
const MOVE_SPEED: f32 = 0.008;

/// The speed at which the selection can be scaled, in reciprocal mouse units.
/// Positive mouse X movement increases scale, negative decreases it.
/// The reciprocal relationship means small movements = small scale changes.
const SCALE_SPEED: f32 = 0.05;

/// The speed at which the selection can be rotated around its forward axis,
/// in radians per mouse unit. This is called "roll" in aviation terms.
const ROLL_SPEED: f32 = 0.01;

/// Various settings for the demo.
/// 
/// Resources in Bevy are globally accessible data that exist outside the ECS
/// (Entity Component System) world. They're perfect for app-wide state like
/// user preferences, game settings, or in this case, what the user has selected
/// and what operation they're performing.
#[derive(Resource, Default)]
struct AppStatus {
    /// The object that will be moved, scaled, or rotated when the mouse is
    /// dragged.
    selection: Selection,
    /// What happens when the mouse is dragged: one of a move, rotate, or scale
    /// operation.
    drag_mode: DragMode,
}

/// The object that will be moved, scaled, or rotated when the mouse is dragged.
/// 
/// This enum serves double duty:
/// 1. As a Component - entities are tagged with this to identify what they are
/// 2. As state in AppStatus - tracks what the user has currently selected
/// 
/// The #[derive] attributes:
/// - Clone, Copy: Allows the enum to be copied by value (it's small)
/// - Component: Makes this usable as an ECS component
/// - Default: The #[default] attribute makes Camera the default variant
/// - PartialEq: Allows == comparison between Selection values
#[derive(Clone, Copy, Component, Default, PartialEq)]
enum Selection {
    /// The camera.
    ///
    /// The camera can only be moved, not scaled or rotated.
    #[default]
    Camera,
    /// The first decal, which an orange bounding box surrounds.
    DecalA,
    /// The second decal, which a lime green bounding box surrounds.
    DecalB,
}

// The Display trait lets us convert Selection to a human-readable string.
// This is used in the help text: "Click and drag to move camera"
impl fmt::Display for Selection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // The *self syntax dereferences self to get the actual enum value
        match *self {
            Selection::Camera => f.write_str("camera"),
            Selection::DecalA => f.write_str("decal A"),
            Selection::DecalB => f.write_str("decal B"),
        }
    }
}

/// What happens when the mouse is dragged: one of a move, rotate, or scale
/// operation.
/// 
/// This is also used as a Component on the UI buttons to identify which
/// operation each button triggers when hovered over.
#[derive(Clone, Copy, Component, Default, PartialEq, Debug)]
enum DragMode {
    /// The mouse moves the current selection.
    /// For camera: orbits around the origin
    /// For decals: moves them around the cube surface
    #[default]
    Move,
    /// The mouse scales the current selection.
    ///
    /// This only applies to decals, not cameras.
    /// Dragging right makes decals bigger, left makes them smaller.
    Scale,
    /// The mouse rotates the current selection around its local Z axis.
    ///
    /// This only applies to decals, not cameras.
    /// Think of this like rotating a sticker after you've placed it.
    Roll,
}

impl fmt::Display for DragMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            DragMode::Move => f.write_str("move"),
            DragMode::Scale => f.write_str("scale"),
            DragMode::Roll => f.write_str("roll"),
        }
    }
}

/// A marker component for the help text in the top left corner of the window.
/// 
/// Marker components are zero-sized types used purely for identification.
/// They let us query for specific entities without storing any data.
/// Here, we use it to find and update the help text UI element.
#[derive(Clone, Copy, Component)]
struct HelpText;

/// A shader extension that demonstrates how to use the `tag` field to customize
/// the appearance of your decals.
/// 
/// MaterialExtension is Bevy's way of adding custom shader logic to existing
/// materials. Instead of writing a complete shader from scratch, you write
/// just the parts you want to customize.
/// 
/// The derives:
/// - Asset: Makes this loadable as a Bevy asset
/// - AsBindGroup: Automatically generates GPU binding code
/// - Reflect: Enables runtime type inspection (useful for editors/debugging)
/// - Debug, Clone: Standard Rust traits for debugging and cloning
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
struct CustomDecalExtension {
    // This struct is empty because our shader reads the decal tag directly
    // from the decal data, not from material properties
}

// This trait implementation tells Bevy how to extend the standard material
impl MaterialExtension for CustomDecalExtension {
    // We only override the fragment shader, which runs once per pixel
    // to determine the final color. The vertex shader (which positions
    // vertices) uses the standard implementation.
    fn fragment_shader() -> ShaderRef {
        // ShaderRef can be created from a path string using .into()
        SHADER_ASSET_PATH.into()
    }
}

/// Entry point.
fn main() {
    App::new()
        // DefaultPlugins includes rendering, input, windowing, etc.
        // We customize the WindowPlugin to set our window title
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Clustered Decals Example".into(),
                ..default()
            }),
            ..default()
        }))
        // Register our custom material type. ExtendedMaterial combines
        // StandardMaterial (Bevy's PBR material) with our custom extension
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, CustomDecalExtension>,
        >::default())
        // Initialize our app state resource with default values
        .init_resource::<AppStatus>()
        // Register custom event type for UI widget clicks
        .add_event::<WidgetClickEvent<Selection>>()
        // Startup system runs once when the app starts
        .add_systems(Startup, setup)
        // Update systems run every frame
        .add_systems(Update, draw_gizmos)           // Draw bounding boxes around decals
        .add_systems(Update, rotate_cube)           // Rotate the cube continuously
        .add_systems(Update, widgets::handle_ui_interactions::<Selection>) // Process UI clicks
        // These systems must run after UI interactions are processed
        .add_systems(
            Update,
            (handle_selection_change, update_radio_buttons)
                .after(widgets::handle_ui_interactions::<Selection>),
        )
        .add_systems(Update, process_move_input)    // Handle mouse dragging for movement
        .add_systems(Update, process_scale_input)   // Handle mouse dragging for scaling
        .add_systems(Update, process_roll_input)    // Handle mouse dragging for rotation
        .add_systems(Update, switch_drag_mode)      // Switch modes when hovering buttons
        .add_systems(Update, update_help_text)      // Update the help text display
        .add_systems(Update, update_button_visibility) // Show/hide buttons based on selection
        .run();
}

/// Creates the scene.
/// 
/// This system runs once at startup and sets up all the initial entities:
/// the cube, camera, light, decals, and UI elements.
fn setup(
    // Commands let us spawn entities and add components
    mut commands: Commands,
    // AssetServer loads external files (images, models, etc.)
    asset_server: Res<AssetServer>,
    // Our app state resource
    app_status: Res<AppStatus>,
    // GPU device info - needed to check feature support
    render_device: Res<RenderDevice>,
    render_adapter: Res<RenderAdapter>,
    // Asset storage for meshes (3D geometry)
    mut meshes: ResMut<Assets<Mesh>>,
    // Asset storage for our custom materials
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, CustomDecalExtension>>>,
) {
    // Error out if clustered decals aren't supported on the current platform.
    // Some older GPUs or graphics APIs might not support the features needed
    // for clustered decals (like storage buffers or certain texture formats).
    if !decal::clustered::clustered_decals_are_usable(&render_device, &render_adapter) {
        eprintln!("Clustered decals aren't usable on this platform.");
        process::exit(1);
    }

    // Set up the scene components
    spawn_cube(&mut commands, &mut meshes, &mut materials);
    spawn_camera(&mut commands);
    spawn_light(&mut commands);
    spawn_decals(&mut commands, &asset_server);
    spawn_buttons(&mut commands);
    spawn_help_text(&mut commands, &app_status);
}

/// Spawns the cube onto which the decals are projected.
fn spawn_cube(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ExtendedMaterial<StandardMaterial, CustomDecalExtension>>,
) {
    // Start with an identity transform (no rotation, scale 1, position at origin)
    let mut transform = Transform::IDENTITY;
    // Rotate by π/3 radians (60 degrees) around Y axis to show the decals better
    transform.rotate_y(FRAC_PI_3);

    commands.spawn((
        // Mesh3d component references a mesh asset
        // Cuboid::new(3.0, 3.0, 3.0) creates a cube with side length 3
        Mesh3d(meshes.add(Cuboid::new(3.0, 3.0, 3.0))),
        // MeshMaterial3d component references our extended material
        MeshMaterial3d(materials.add(ExtendedMaterial {
            // The base material is standard PBR with a silver color
            base: StandardMaterial {
                base_color: SILVER.into(),
                ..default()
            },
            // Our extension doesn't need any data - it reads from decals
            extension: CustomDecalExtension {},
        })),
        // The transform positions and orients the cube
        transform,
    ));
}

/// Spawns the directional light.
/// 
/// Directional lights simulate sunlight - parallel rays from infinitely far away.
/// The position doesn't matter for lighting, but we position it anyway
/// to indicate where the light is "coming from" conceptually.
fn spawn_light(commands: &mut Commands) {
    commands.spawn((
        // Default directional light has reasonable intensity and color
        DirectionalLight::default(),
        // Position the light above and to the side, looking at the origin
        // looking_at() creates a transform that faces a target position
        // Vec3::Y is the "up" vector used to control the light's roll
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Spawns the camera.
fn spawn_camera(commands: &mut Commands) {
    commands
        // Camera3d includes perspective projection and other 3D camera settings
        .spawn(Camera3d::default())
        // Position the camera back and slightly up, looking at the origin
        // This gives us a nice view of the cube and decals
        .insert(Transform::from_xyz(0.0, 2.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y))
        // Tag the camera with `Selection::Camera`.
        // This lets us identify it when the user selects "Camera" in the UI
        .insert(Selection::Camera);
}

/// Spawns the actual clustered decals.
/// 
/// Each decal is an entity with:
/// - ClusteredDecal component (the image and tag)
/// - Transform component (position, rotation, scale)
/// - Selection component (for UI interaction)
fn spawn_decals(commands: &mut Commands, asset_server: &AssetServer) {
    // Load the Bevy logo image that we'll use for both decals
    let image = asset_server.load("branding/icon.png");

    // First decal - will be tinted red by the shader
    commands.spawn((
        ClusteredDecal {
            image: image.clone(),
            // Tag value 1 - our custom shader uses this to apply red tint
            tag: 1,
        },
        // Position at (1, 3, 5), looking at origin, size 1.1x1.1
        calculate_initial_decal_transform(vec3(1.0, 3.0, 5.0), Vec3::ZERO, Vec2::splat(1.1)),
        // Tag for selection system
        Selection::DecalA,
    ));

    // Second decal - will be tinted blue by the shader
    commands.spawn((
        ClusteredDecal {
            image: image.clone(),
            // Tag value 2 - our custom shader uses this to apply blue tint
            tag: 2,
        },
        // Position at (-2, -1, 4), looking at origin, size 2x2 (bigger than first)
        calculate_initial_decal_transform(vec3(-2.0, -1.0, 4.0), Vec3::ZERO, Vec2::splat(2.0)),
        // Tag for selection system
        Selection::DecalB,
    ));
}

/// Spawns the buttons at the bottom of the screen.
fn spawn_buttons(commands: &mut Commands) {
    // Spawn the radio buttons that allow the user to select an object to
    // control.
    commands
        .spawn(widgets::main_ui_node())
        .with_children(|parent| {
            widgets::spawn_option_buttons(
                parent,
                "Drag to Move",
                &[
                    (Selection::Camera, "Camera"),
                    (Selection::DecalA, "Decal A"),
                    (Selection::DecalB, "Decal B"),
                ],
            );
        });

    // Spawn the drag buttons that allow the user to control the scale and roll
    // of the selected object.
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            bottom: Val::Px(10.0),
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|parent| {
            spawn_drag_button(parent, "Scale").insert(DragMode::Scale);
            spawn_drag_button(parent, "Roll").insert(DragMode::Roll);
        });
}

/// Spawns a button that the user can drag to change a parameter.
fn spawn_drag_button<'a>(
    commands: &'a mut ChildSpawnerCommands,
    label: &str,
) -> EntityCommands<'a> {
    let mut kid = commands.spawn(Node {
        border: BUTTON_BORDER,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: BUTTON_PADDING,
        ..default()
    });
    kid.insert((
        Button,
        BackgroundColor(Color::BLACK),
        BorderRadius::all(BUTTON_BORDER_RADIUS_SIZE),
        BUTTON_BORDER_COLOR,
    ))
    .with_children(|parent| {
        widgets::spawn_ui_text(parent, label, Color::WHITE);
    });
    kid
}

/// Spawns the help text at the top of the screen.
fn spawn_help_text(commands: &mut Commands, app_status: &AppStatus) {
    commands.spawn((
        Text::new(create_help_string(app_status)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        HelpText,
    ));
}

/// Draws the outlines that show the bounds of the clustered decals.
/// 
/// Gizmos are debug visualizations drawn on top of the scene.
/// They're perfect for showing bounding boxes, axes, or other helpers.
fn draw_gizmos(
    // Gizmos is a system parameter for drawing debug shapes
    mut gizmos: Gizmos,
    // Query all entities that have both GlobalTransform and Selection components,
    // but only if they also have a ClusteredDecal component
    decals: Query<(&GlobalTransform, &Selection), With<ClusteredDecal>>,
) {
    for (global_transform, selection) in &decals {
        // Choose color based on which decal this is
        let color = match *selection {
            Selection::Camera => continue, // Skip - cameras aren't decals
            Selection::DecalA => ORANGE_RED,
            Selection::DecalB => LIME,
        };

        // Draw a wireframe cuboid (box) at the decal's position
        gizmos.primitive_3d(
            &Cuboid {
                // Decals internally use a unit cube (1×1×1) that gets scaled.
                // The half_size is half the full size, so we multiply scale by 0.5
                half_size: global_transform.scale() * 0.5,
            },
            // Isometry3d represents position and rotation (no scale)
            Isometry3d {
                rotation: global_transform.rotation(),
                // translation_vec3a() returns the position as a Vec3A (aligned vector)
                translation: global_transform.translation_vec3a(),
            },
            color,
        );
    }
}

/// Calculates the initial transform of the clustered decal.
/// 
/// Decals are represented as stretched cubes that project their texture
/// onto whatever geometry they intersect. This function creates the
/// transform that positions and orients the decal correctly.
/// 
/// # Parameters
/// - `start`: Where the decal projection begins
/// - `looking_at`: What point the decal faces toward
/// - `size`: Width and height of the decal in world units
///
/// ## Mathematical Foundation: Oriented Bounding Box (OBB)
///
/// A decal is fundamentally an OBB with texture projection. The math:
/// ```text
/// OBB = Center + (u × half_width) + (v × half_height) + (w × half_depth)
/// where u,v,w are the local axes
/// ```
///
/// ## Projection Matrix Decomposition
///
/// The transform encodes a projection matrix that maps 3D space to UV:
/// 1. **Translation**: Moves origin to decal center
/// 2. **Rotation**: Aligns Z axis with projection direction
/// 3. **Scale**: Maps unit cube to decal volume
///
/// Combined: `UV = (inverse(Transform) × WorldPos + 0.5) × aspect_ratio`
///
/// ## Performance Considerations
///
/// This transform is uploaded to GPU as a 4×3 matrix (no projection row):
/// - **Memory**: 48 bytes per decal (12 floats)
/// - **Bandwidth**: Cached in constant buffer for reuse
/// - **ALU Cost**: 12 FMA operations per vertex/pixel
///
/// ## Rust Optimization Notes
///
/// The compiler optimizes this function through:
/// - **SIMD**: Vec3 operations use SSE/AVX on x86
/// - **Inlining**: Transform methods are #[inline]
/// - **Const Propagation**: Known values (0.5, Y) are compile-time constants
fn calculate_initial_decal_transform(start: Vec3, looking_at: Vec3, size: Vec2) -> Transform {
    // Calculate the direction vector from start to target
    let direction = looking_at - start;
    // Position the decal at the midpoint between start and target
    // This creates a "projector" that shoots from start through looking_at
    let center = start + direction * 0.5;
    
    Transform::from_translation(center)
        // Scale X and Y by half the size (since the base cube is 1×1×1)
        // Scale Z by the distance to control projection depth
        // 
        // Why half? The unit cube goes from -0.5 to 0.5 in each dimension,
        // so scaling by size/2 gives us the desired world size
        .with_scale((size * 0.5).extend(direction.length()))
        // Orient the decal to face along the direction vector
        // Vec3::Y is used as the "up" reference
        //
        // looking_to() uses the Gram-Schmidt process to create an orthonormal basis:
        // 1. w = normalize(direction)  // Forward
        // 2. u = normalize(cross(up, w))  // Right
        // 3. v = cross(w, u)  // Recalculated up
        .looking_to(direction, Vec3::Y)
}

/// Rotates the cube a bit every frame.
/// 
/// This creates visual interest and shows how decals stay properly
/// projected even as the surface moves.
fn rotate_cube(
    // Query for Transform components on entities that also have Mesh3d
    // The &mut means we want to modify the transforms
    mut meshes: Query<&mut Transform, With<Mesh3d>>
) {
    // In this example there's only one mesh (the cube), but the query
    // pattern works for any number of meshes
    for mut transform in &mut meshes {
        // Rotate around the Y axis (vertical) at constant speed
        transform.rotate_y(CUBE_ROTATION_SPEED);
    }
}

/// Updates the state of the radio buttons when the user clicks on one.
fn update_radio_buttons(
    mut widgets: Query<(
        Entity,
        Option<&mut BackgroundColor>,
        Has<Text>,
        &WidgetClickSender<Selection>,
    )>,
    app_status: Res<AppStatus>,
    mut writer: TextUiWriter,
) {
    for (entity, maybe_bg_color, has_text, sender) in &mut widgets {
        let selected = app_status.selection == **sender;
        if let Some(mut bg_color) = maybe_bg_color {
            widgets::update_ui_radio_button(&mut bg_color, selected);
        }
        if has_text {
            widgets::update_ui_radio_button_text(entity, &mut writer, selected);
        }
    }
}

/// Changes the selection when the user clicks a radio button.
fn handle_selection_change(
    mut events: EventReader<WidgetClickEvent<Selection>>,
    mut app_status: ResMut<AppStatus>,
) {
    for event in events.read() {
        app_status.selection = **event;
    }
}

/// Process a drag event that moves the selected object.
/// 
/// This system implements orbital movement - objects rotate around the origin
/// while maintaining their distance. We use spherical coordinates for this:
/// - radius: distance from origin
/// - theta (θ): angle from vertical (0 = top, π = bottom)  
/// - phi (φ): angle around the vertical axis
///
/// ## Mathematical Deep Dive: Spherical Coordinates
///
/// The conversion between Cartesian and spherical coordinates:
/// ```text
/// Cartesian to Spherical:           Spherical to Cartesian:
/// r = √(x² + y² + z²)              x = r·sin(θ)·cos(φ)
/// θ = arccos(y/r)                  y = r·cos(θ)
/// φ = arctan2(z, x)                z = r·sin(θ)·sin(φ)
/// ```
///
/// ## Numerical Stability Considerations
///
/// 1. **Gimbal Lock**: When θ ≈ 0 or θ ≈ π, the φ rotation becomes undefined
/// 2. **Precision Loss**: Near poles, small Cartesian changes = large angular changes
/// 3. **Normalization Drift**: Repeated conversions accumulate floating-point error
///
/// ## Game Camera Patterns
///
/// This implements the "orbital camera" pattern common in:
/// - **Third-person games**: Rotating around the player
/// - **RTS games**: Examining units from all angles
/// - **Level editors**: Inspecting 3D scenes
///
/// ## Performance Optimization
///
/// The trigonometric operations here are expensive (~50-100 cycles each).
/// Optimization strategies:
/// 1. **Lookup Tables**: Pre-compute sin/cos for common angles
/// 2. **SIMD**: Use vector instructions for parallel trig
/// 3. **Quaternion SLERP**: Avoid trig entirely for smooth rotation
fn process_move_input(
    // Query for entities with Transform and Selection components
    mut selections: Query<(&mut Transform, &Selection)>,
    // Input state for mouse buttons
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    // Accumulated mouse movement since last frame
    mouse_motion: Res<AccumulatedMouseMotion>,
    // Current app state (what's selected, what mode we're in)
    app_status: Res<AppStatus>,
) {
    // Only process drags when movement is selected.
    if !mouse_buttons.pressed(MouseButton::Left) || app_status.drag_mode != DragMode::Move {
        return;
    }

    for (mut transform, selection) in &mut selections {
        // Skip entities that aren't currently selected
        if app_status.selection != *selection {
            continue;
        }

        let position = transform.translation;

        // Convert Cartesian coordinates (x,y,z) to spherical (r,θ,φ)
        let radius = position.length();
        // theta: angle from Y axis (acos gives us 0 at top, π at bottom)
        let mut theta = acos(position.y / radius);
        // phi: angle around Y axis (we use signum to handle the full circle)
        //
        // ## Why signum() × acos()?
        // This handles the full 360° range. acos only gives 0 to π,
        // so we use the sign of z to disambiguate the hemisphere.
        // A more robust approach would use atan2(z, x) but this is faster.
        let mut phi = position.z.signum() * acos(position.x * position.xz().length_recip());

        // Camera movement is the inverse of object movement.
        // When you drag right, the camera should orbit right (objects appear to move left)
        //
        // ## Input Mapping Design
        // This inverted control scheme matches user expectations from:
        // - 3D modeling software (Blender, Maya)
        // - Map applications (Google Earth)
        // - Most third-person games
        let (phi_factor, theta_factor) = match *selection {
            Selection::Camera => (1.0, -1.0),
            Selection::DecalA | Selection::DecalB => (-1.0, 1.0),
        };

        // Apply mouse movement to spherical coordinates
        phi += phi_factor * mouse_motion.delta.x * MOVE_SPEED;
        // Clamp theta to avoid gimbal lock at the poles (0 and π)
        //
        // ## Epsilon Selection
        // 0.001 radians ≈ 0.057 degrees
        // This prevents:
        // 1. Division by zero in xz().length_recip()
        // 2. Unstable phi calculations near poles
        // 3. Visual glitches from exactly vertical orientations
        theta = f32::clamp(
            theta + theta_factor * mouse_motion.delta.y * MOVE_SPEED,
            0.001,      // Just above 0
            PI - 0.001, // Just below π
        );

        // Convert spherical coordinates back to Cartesian
        // x = r * sin(θ) * cos(φ)
        // y = r * cos(θ)
        // z = r * sin(θ) * sin(φ)
        //
        // ## SIMD Optimization Note
        // On modern CPUs, the sin/cos pairs can be computed together:
        // - x86: FSINCOS instruction
        // - ARM: Dedicated SIMD lanes
        // Rust's optimizer may recognize this pattern.
        transform.translation =
            radius * vec3(sin(theta) * cos(phi), cos(theta), sin(theta) * sin(phi));

        // Make the object face the origin, but preserve its roll angle
        // First, extract the current roll angle
        //
        // ## Euler Angle Extraction
        // EulerRot::YXZ means: Yaw(Y) → Pitch(X) → Roll(Z)
        // This order prevents gimbal lock for typical camera orientations
        let roll = transform.rotation.to_euler(EulerRot::YXZ).2;
        // Look at the origin
        transform.look_at(Vec3::ZERO, Vec3::Y);
        // Extract the new yaw and pitch
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        // Reconstruct rotation with the original roll
        //
        // ## Why Preserve Roll?
        // For decals: Maintains artistic orientation
        // For cameras: Prevents disorienting view tilting
        // This is a form of rotation constraint common in game engines
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

/// Processes a drag event that scales the selected target.
/// 
/// Scaling uses horizontal mouse movement:
/// - Drag right = increase scale (multiply by > 1.0)
/// - Drag left = decrease scale (multiply by < 1.0)
fn process_scale_input(
    mut selections: Query<(&mut Transform, &Selection)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    app_status: Res<AppStatus>,
) {
    // Only process drags when the scaling operation is selected.
    if !mouse_buttons.pressed(MouseButton::Left) || app_status.drag_mode != DragMode::Scale {
        return;
    }

    for (mut transform, selection) in &mut selections {
        if app_status.selection == *selection {
            // Multiply current scale by a factor based on mouse movement
            // Positive delta.x (moving right) gives factor > 1.0 (grow)
            // Negative delta.x (moving left) gives factor < 1.0 (shrink)
            // This creates exponential scaling which feels natural
            transform.scale *= 1.0 + mouse_motion.delta.x * SCALE_SPEED;
        }
    }
}

/// Processes a drag event that rotates the selected target along its local Z
/// axis.
/// 
/// "Roll" is rotation around the forward axis - imagine a plane doing a
/// barrel roll. For our decals, this rotates them in place on the surface.
fn process_roll_input(
    mut selections: Query<(&mut Transform, &Selection)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    app_status: Res<AppStatus>,
) {
    // Only process drags when the rolling operation is selected.
    if !mouse_buttons.pressed(MouseButton::Left) || app_status.drag_mode != DragMode::Roll {
        return;
    }

    for (mut transform, selection) in &mut selections {
        if app_status.selection != *selection {
            continue;
        }

        // Extract Euler angles from the current rotation
        // EulerRot::YXZ means: first rotate Y (yaw), then X (pitch), then Z (roll)
        let (yaw, pitch, mut roll) = transform.rotation.to_euler(EulerRot::YXZ);
        // Add mouse movement to roll angle
        roll += mouse_motion.delta.x * ROLL_SPEED;
        // Reconstruct the rotation quaternion with the new roll
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

/// Creates the help string at the top left of the screen.
fn create_help_string(app_status: &AppStatus) -> String {
    format!(
        "Click and drag to {} {}",
        app_status.drag_mode, app_status.selection
    )
}

/// Changes the drag mode when the user hovers over the "Scale" and "Roll"
/// buttons in the lower right.
///
/// If the user is hovering over no such button, this system changes the drag
/// mode back to its default value of [`DragMode::Move`].
/// 
/// This creates a nice UX where hovering over a button immediately activates
/// that mode, and the cursor changes to indicate the mode is active.
fn switch_drag_mode(
    mut commands: Commands,
    // Query UI elements that have both Interaction (button state) and DragMode
    mut interactions: Query<(&Interaction, &DragMode)>,
    // Query for the window entity to change its cursor
    mut windows: Query<Entity, With<Window>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut app_status: ResMut<AppStatus>,
) {
    // Don't switch modes while dragging
    if mouse_buttons.pressed(MouseButton::Left) {
        return;
    }

    // Check if any drag mode button is hovered
    for (interaction, drag_mode) in &mut interactions {
        if *interaction != Interaction::Hovered {
            continue;
        }

        // Switch to the hovered button's mode
        app_status.drag_mode = *drag_mode;

        // Set the cursor to resize arrows to indicate drag capability
        for window in &mut windows {
            commands
                .entity(window)
                .insert(CursorIcon::from(SystemCursorIcon::EwResize));
        }
        return;
    }

    // No button is hovered - revert to default move mode
    app_status.drag_mode = DragMode::Move;

    // Reset cursor to default
    for window in &mut windows {
        commands.entity(window).remove::<CursorIcon>();
    }
}

/// Updates the help text in the top left of the screen to reflect the current
/// selection and drag mode.
fn update_help_text(mut help_text: Query<&mut Text, With<HelpText>>, app_status: Res<AppStatus>) {
    for mut text in &mut help_text {
        text.0 = create_help_string(&app_status);
    }
}

/// Updates the visibility of the drag mode buttons so that they aren't visible
/// if the camera is selected.
fn update_button_visibility(
    mut nodes: Query<&mut Visibility, With<DragMode>>,
    app_status: Res<AppStatus>,
) {
    for mut visibility in &mut nodes {
        *visibility = match app_status.selection {
            Selection::Camera => Visibility::Hidden,
            Selection::DecalA | Selection::DecalB => Visibility::Visible,
        };
    }
}
