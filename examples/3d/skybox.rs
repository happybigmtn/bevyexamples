//! Load a cubemap texture onto a cube like a skybox and cycle through different compressed texture formats
//!
//! # Skyboxes: Creating Infinite Horizons
//!
//! Imagine you're standing in a huge painted sphere - no matter where you look,
//! you see distant mountains, clouds, and sky. That's what a skybox does in 3D!
//! It creates the illusion of infinite distance using a clever trick.
//!
//! ## How Skyboxes Work:
//!
//! 1. **Cubemap Texture**: 6 square images forming a cube (up, down, left, right, front, back)
//! 2. **Infinite Distance**: Rendered behind everything else, moves with camera
//! 3. **Seamless Blending**: GPU smoothly blends between cube faces
//! 4. **Environment Lighting**: Can provide ambient light and reflections
//!
//! ## This Example Demonstrates:
//!
//! - Loading different cubemap formats (PNG, KTX2)
//! - Cycling through compression formats (ASTC, BC7, ETC2)
//! - Detecting GPU-supported formats
//! - Converting 2D strip images to cubemaps
//! - Animating directional light to see lighting changes

// Rust: Module path attribute to include external file
// #[path] specifies relative path from current file
#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

// Rust: Complex imports from external crate
use bevy::{
    // Rust: Skybox component for infinite backgrounds
    core_pipeline::Skybox,
    // Rust: Bitflags for texture compression formats
    image::CompressedImageFormats,
    // Rust: Common Bevy types (App, Transform, etc.)
    prelude::*,
    // Rust: Low-level rendering types
    render::{
        // Rust: GPU texture configuration
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        // Rust: Access to GPU capabilities
        renderer::RenderDevice,
    },
};
// Rust: Import from local module
use camera_controller::{CameraController, CameraControllerPlugin};
// Rust: Mathematical constant π ≈ 3.14159
use std::f32::consts::PI;

// Rust: const creates compile-time constant
// &[T] is a slice reference - fixed-size view into array
// Each tuple contains (file_path, required_format)
const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[
    (
        // Rust: Uncompressed PNG - works everywhere but large
        "textures/Ryfjallet_cubemap.png",
        CompressedImageFormats::NONE,  // No compression required
    ),
    (
        // Rust: ASTC format - great for mobile GPUs
        "textures/Ryfjallet_cubemap_astc4x4.ktx2",
        CompressedImageFormats::ASTC_LDR,  // Adaptive Scalable Texture Compression
    ),
    (
        // Rust: BC7 format - optimized for desktop GPUs
        "textures/Ryfjallet_cubemap_bc7.ktx2",
        CompressedImageFormats::BC,  // Block Compression (DirectX)
    ),
    (
        // Rust: ETC2 format - standard for OpenGL ES
        "textures/Ryfjallet_cubemap_etc2.ktx2",
        CompressedImageFormats::ETC2,  // Ericsson Texture Compression
    ),
];

// Rust: Program entry point
fn main() {
    // Rust: Builder pattern for app configuration
    App::new()
        // Rust: Standard Bevy plugins
        .add_plugins(DefaultPlugins)
        // Rust: Custom camera movement plugin
        .add_plugins(CameraControllerPlugin)
        // Rust: System that runs once at startup
        .add_systems(Startup, setup)
        // Rust: Systems that run every frame
        .add_systems(
            Update,
            (
                // Rust: Cycle through cubemap formats
                cycle_cubemap_asset,
                // Rust: Load and configure cubemap when ready
                // .after() creates system ordering dependency
                asset_loaded.after(cycle_cubemap_asset),
                // Rust: Rotate the sun light
                animate_light_direction,
            ),
        )
        // Rust: Start the game loop
        .run();
}

// Rust: Derive macro generates Resource trait implementation
// Resources are globally accessible data in ECS
#[derive(Resource)]
struct Cubemap {
    // Rust: Track loading state
    is_loaded: bool,
    // Rust: Current index in CUBEMAPS array
    index: usize,
    // Rust: Handle to current cubemap texture
    image_handle: Handle<Image>,
}

// Rust: Setup system creates initial scene
fn setup(
    // Rust: Mutable access to spawn entities
    mut commands: Commands,
    // Rust: Asset loading service
    asset_server: Res<AssetServer>,
) {
    // directional 'sun' light
    // Rust: Spawn entity with component tuple
    commands.spawn((
        // Rust: Struct literal with partial initialization
        DirectionalLight {
            // Rust: f32 literal in lux (bright sunny day)
            illuminance: 32000.0,
            ..default()
        },
        // Rust: Transform with position and rotation
        // Position: 2 units up, Rotation: 45° downward tilt
        Transform::from_xyz(0.0, 2.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
    ));

    // Rust: Load first cubemap texture
    // CUBEMAPS[0].0 accesses first element's first tuple field
    let skybox_handle = asset_server.load(CUBEMAPS[0].0);
    
    // camera
    // Rust: Camera entity with multiple components
    commands.spawn((
        // Rust: Default 3D camera settings
        Camera3d::default(),
        // Rust: Camera 8 units back, looking at origin
        Transform::from_xyz(0.0, 0.0, 8.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        // Rust: Enable camera controls
        CameraController::default(),
        // Rust: Skybox component configuration
        Skybox {
            // Rust: Clone handle (cheap - reference counted)
            image: skybox_handle.clone(),
            // Rust: Skybox brightness multiplier
            brightness: 1000.0,
            ..default()
        },
    ));

    // ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate color and brightness to match
    // Rust: Insert global resource
    commands.insert_resource(AmbientLight {
        // Rust: Color from 8-bit RGB values (0-255)
        // Light blue tint mimics sky color
        color: Color::srgb_u8(210, 220, 240),
        // Rust: Full brightness
        brightness: 1.0,
        ..default()
    });

    // Rust: Create cubemap tracking resource
    commands.insert_resource(Cubemap {
        is_loaded: false,  // Not loaded yet
        index: 0,          // Start with first cubemap
        image_handle: skybox_handle,  // Move handle ownership
    });
}

// Rust: Time between cubemap format changes (seconds)
const CUBEMAP_SWAP_DELAY: f32 = 3.0;

// Rust: System that cycles through different cubemap formats
fn cycle_cubemap_asset(
    // Rust: Time resource for elapsed time
    time: Res<Time>,
    // Rust: Local<T> is system-local state (persists between calls)
    mut next_swap: Local<f32>,
    // Rust: Mutable access to cubemap resource
    mut cubemap: ResMut<Cubemap>,
    // Rust: Asset loading service
    asset_server: Res<AssetServer>,
    // Rust: GPU device info for feature detection
    render_device: Res<RenderDevice>,
) {
    // Rust: Get total elapsed time since app start
    let now = time.elapsed_secs();
    
    // Rust: Initialize timer on first run
    if *next_swap == 0.0 {
        // Rust: Dereference and assign
        *next_swap = now + CUBEMAP_SWAP_DELAY;
        return;  // Wait for first delay
    } else if now < *next_swap {
        return;  // Not time to swap yet
    }
    
    // Rust: Schedule next swap
    *next_swap += CUBEMAP_SWAP_DELAY;

    // Rust: Query GPU for supported texture formats
    // Different GPUs support different compression formats
    let supported_compressed_formats =
        CompressedImageFormats::from_features(render_device.features());

    // Rust: Find next supported format
    let mut new_index = cubemap.index;
    // Rust: Loop through all formats to find compatible one
    for _ in 0..CUBEMAPS.len() {
        // Rust: Modulo wraps around to 0 after last element
        new_index = (new_index + 1) % CUBEMAPS.len();
        // Rust: Check if GPU supports this format
        if supported_compressed_formats.contains(CUBEMAPS[new_index].1) {
            break;  // Found supported format
        }
        // Rust: info! macro for logging (visible in console)
        info!(
            "Skipping format which is not supported by current hardware: {:?}",
            CUBEMAPS[new_index]
        );
    }

    // Skip swapping to the same texture. Useful for when ktx2, zstd, or compressed texture support
    // is missing
    // Rust: Equality comparison
    if new_index == cubemap.index {
        return;  // No compatible format found
    }

    // Rust: Update resource fields
    cubemap.index = new_index;
    // Rust: Start loading new cubemap file
    cubemap.image_handle = asset_server.load(CUBEMAPS[cubemap.index].0);
    cubemap.is_loaded = false;  // Mark as not loaded yet
}

// Rust: System that handles loaded cubemap configuration
fn asset_loaded(
    // Rust: Check asset loading status
    asset_server: Res<AssetServer>,
    // Rust: Mutable access to image assets
    mut images: ResMut<Assets<Image>>,
    // Rust: Mutable cubemap resource
    mut cubemap: ResMut<Cubemap>,
    // Rust: Query all skybox components
    mut skyboxes: Query<&mut Skybox>,
) {
    // Rust: Check if asset finished loading
    if !cubemap.is_loaded && asset_server.load_state(&cubemap.image_handle).is_loaded() {
        // Rust: Log format change
        info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
        // Rust: Get mutable reference to image
        // unwrap() panics if image not found (shouldn't happen)
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        // Rust: Check if image needs cubemap conversion
        if image.texture_descriptor.array_layer_count() == 1 {
            // Rust: Convert 2D strip to 6-face cubemap array
            // Assumes 6 square faces stacked vertically
            image.reinterpret_stacked_2d_as_array(
                image.height() / image.width()  // Should be 6
            );
            // Rust: Configure GPU to interpret as cubemap
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                // Rust: Cube dimension for 6-face sampling
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        // Rust: Update all skyboxes with new image
        for mut skybox in &mut skyboxes {
            // Rust: Clone handle to share between skyboxes
            skybox.image = cubemap.image_handle.clone();
        }

        // Rust: Mark as successfully loaded
        cubemap.is_loaded = true;
    }
}

// Rust: System that rotates the sun light
fn animate_light_direction(
    // Rust: Time for frame delta
    time: Res<Time>,
    // Rust: Query transforms that have DirectionalLight
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    // Rust: Iterate over all directional lights
    for mut transform in &mut query {
        // 🔄 Gentle rotation around the Y axis
        // Half a radian per second = about 30 degrees per second
        // Rust: Rotate by angle = speed * time
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}

// 🎯 Key Rust Patterns in This Example:
//
// 1. **Const Arrays**:
//    - `const CUBEMAPS: &[...]` - compile-time constant
//    - Slice of tuples for configuration data
//    - No runtime allocation needed
//
// 2. **Local State**:
//    - `Local<T>` persists between system runs
//    - Each system instance has its own copy
//    - Useful for timers and counters
//
// 3. **Resource Pattern**:
//    - `#[derive(Resource)]` for global data
//    - `Res<T>` for immutable access
//    - `ResMut<T>` for mutable access
//
// 4. **Handle Management**:
//    - `Handle<Image>` is reference-counted
//    - `clone()` is cheap (just increments count)
//    - Automatic cleanup when count reaches zero
//
// 5. **Query Filters**:
//    - `Query<&mut Transform, With<DirectionalLight>>`
//    - Gets only transforms that have DirectionalLight
//    - Type-safe component filtering
//
// 6. **Option Handling**:
//    - `Some(value)` wraps nullable values
//    - Used for optional texture view descriptor
//    - Rust's null safety mechanism
//
// 7. **System Ordering**:
//    - `.after(system)` creates dependency
//    - Ensures correct execution order
//    - Prevents race conditions

// 🎓 Deep Dive: The Magic of Skyboxes
//
// **What is a Skybox?**
// A skybox is a rendering technique that creates the illusion of infinite
// distance. It's like being inside a giant sphere with a 360° image painted
// on the inside. No matter where you move, you're always at the center.
//
// **How Cubemaps Work:**
// Instead of a sphere, we use a cube with 6 faces:
// - +X (Right)  - -X (Left)
// - +Y (Up)    - -Y (Down)
// - +Z (Front) - -Z (Back)
//
// Each face is a square texture. When you look in any direction, the GPU
// samples the appropriate face(s) and blends them seamlessly.
//
// **The "Infinite Distance" Trick:**
// Skyboxes are rendered with a special trick - they're drawn "behind"
// everything else. The depth buffer is set to maximum distance, and the
// skybox geometry moves with the camera. This creates the illusion that
// the environment is infinitely far away.
//
// **Texture Compression:**
// Different platforms prefer different compression formats:
//
// - **Uncompressed**: Highest quality, largest files
// - **ASTC**: Modern, efficient, great for mobile
// - **BC (DirectX)**: Optimized for PC/Xbox
// - **ETC2**: Standard for mobile OpenGL
//
// The best format depends on your target platform's GPU architecture.

// 💡 Skybox Creation Tips:
//
// **Photography:**
// - Use 360° cameras or spherical panorama techniques
// - Capture at different times of day for variety
// - Ensure seamless stitching between faces
// - High dynamic range (HDR) for realistic lighting
//
// **Artistic Considerations:**
// - Match the skybox mood to your game's atmosphere
// - Consider how the skybox affects ambient lighting
// - Avoid obvious repeating patterns or landmarks
// - Test visibility from different camera angles
//
// **Technical Guidelines:**
// - Use power-of-2 dimensions (1024x1024, 2048x2048)
// - All faces should be the same resolution
// - Consider mip-mapping for distant viewing
// - Choose compression format based on target platform
//
// **Performance:**
// - Skyboxes are cheap to render (single draw call)
// - Early-Z rejection eliminates most overdraw
// - Modern GPUs handle cubemap sampling efficiently
// - Compressed formats save memory bandwidth

// 🌟 Advanced Skybox Techniques:
//
// **Procedural Skies:**
// - Generate skyboxes mathematically (sun, clouds, stars)
// - Real-time weather and time-of-day changes
// - Atmospheric scattering for realistic colors
//
// **Dynamic Environment:**
// - Update skybox based on game state
// - Blend between different skyboxes
// - Animated cloud movement or aurora effects
//
// **Lighting Integration:**
// - Extract ambient lighting from skybox
// - Generate reflection probes from skybox
// - Use for image-based lighting (IBL)
//
// **Multi-layer Skies:**
// - Separate layers for sky, clouds, stars
// - Different scroll speeds for parallax effect
// - Blend between layers based on time/weather
