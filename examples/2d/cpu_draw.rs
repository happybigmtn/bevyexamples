//! Example of how to draw to a texture from the CPU.
//!
//! # CPU-based Image Drawing: Pixel-Level Control
//!
//! While GPU rendering is fast, sometimes you need precise control over
//! individual pixels. CPU drawing lets you create procedural textures,
//! image filters, or data visualizations pixel by pixel.
//!
//! ## When to Use CPU Drawing:
//!
//! - **Procedural textures**: Generate noise, patterns, fractals
//! - **Image processing**: Apply filters or effects
//! - **Data visualization**: Plot graphs, heatmaps, charts
//! - **Pixel art creation**: Precise sprite editing
//! - **Dynamic textures**: Update textures based on game state
//!
//! ## This Example Creates:
//!
//! - A spiral pattern drawn one pixel at a time
//! - Fixed timestep for consistent animation speed
//! - Color collision detection to change drawing colors
//! - Alpha gradient for circular fade-out effect
//!
//! You can set the values of individual pixels to whatever you want.
//! Bevy provides user-friendly APIs that work with [`Color`](bevy::color::Color)
//! values and automatically perform any necessary conversions and encoding
//! into the texture's native pixel format.
//!
//! ## Game Design Context: When CPU Drawing Shines
//!
//! CPU-based drawing fills specific niches in game development:
//!
//! 1. **Procedural Content Generation**:
//!    - Terrain heightmaps for strategy games
//!    - Cave systems for roguelikes (cellular automata)
//!    - Cloud/smoke textures with Perlin noise
//!    - Starfields with proper distribution patterns
//!
//! 2. **Dynamic Visual Effects**:
//!    - Damage decals on surfaces (bullet holes, scratches)
//!    - Blood splatter that accumulates over time
//!    - Footprints in snow/sand that fade gradually
//!    - Burn marks that spread across materials
//!
//! 3. **Data Visualization in Games**:
//!    - Minimaps showing explored areas
//!    - Heat maps for strategy (enemy density, resources)
//!    - Graphs for management games (economy, population)
//!    - Radar displays with fading blips
//!
//! 4. **Retro Effects**:
//!    - CRT monitor simulation (scanlines, phosphor decay)
//!    - Dithering patterns for limited color palettes
//!    - ASCII art rendering for roguelikes
//!    - Pixel-perfect collision masks
//!
//! ## Rust Fundamentals: Direct Memory Manipulation
//!
//! This example showcases Rust's safe approach to low-level operations:
//!
//! 1. **Newtype Pattern**: `MyProcGenImage(Handle<Image>)`
//!    - Zero-cost abstraction (wrapper disappears at compile time)
//!    - Type safety: Can't accidentally pass wrong handle type
//!    - Implements Deref for ergonomic access
//!
//! 2. **Fixed Timestep**: Deterministic simulation
//!    - Game logic independent of framerate
//!    - Reproducible behavior for testing/replays
//!    - Network synchronization friendly
//!
//! 3. **Seeded RNG**: Reproducible randomness
//!    - Same seed = same pattern every time
//!    - Critical for procedural generation
//!    - Enables "random" but shareable levels
//!
//! ## Bevy Architecture: CPU/GPU Data Flow
//!
//! Understanding the texture pipeline:
//!
//! 1. **CPU Side**: Image data lives in system RAM
//!    - Direct pixel access via `image.data` slice
//!    - Format conversions handled automatically
//!    - Changes queued for GPU upload
//!
//! 2. **GPU Upload**: Happens between frames
//!    - Modified regions tracked for efficiency
//!    - Bandwidth limited - avoid full texture updates
//!    - Multiple small updates can batch together
//!
//! 3. **Render Usage**: How GPU uses the texture
//!    - COPY_SRC: Can copy from this texture
//!    - COPY_DST: Can copy to this texture
//!    - TEXTURE_BINDING: Can use in shaders
//!
//! ## Real-World Applications
//!
//! 1. **Minecraft**: Terrain generation, damage overlays
//! 2. **Terraria**: Liquid simulation, lighting
//! 3. **Dwarf Fortress**: Entire world visualization
//! 4. **SimCity**: Data overlay visualizations
//! 5. **No Man's Sky**: Procedural planet textures
//!
//! ## Performance Considerations
//!
//! 1. **Update Frequency**: 
//!    - Full texture updates are expensive
//!    - Track dirty regions, update only changed areas
//!    - Batch multiple pixel changes before upload
//!
//! 2. **Memory Bandwidth**:
//!    - 256x256 RGBA = 256KB per full update
//!    - At 60 FPS = 15MB/s bandwidth
//!    - Consider lower resolution or partial updates
//!
//! 3. **CPU Cache Efficiency**:
//!    - Access pixels in memory order (row by row)
//!    - Avoid random access patterns
//!    - Process in tiles for better cache usage
//!
//! ## Common Pitfalls
//!
//! 1. **Format Mismatches**: Using wrong color format for texture
//! 2. **Forgetting GPU Upload**: Changes not visible until next frame
//! 3. **Thread Safety**: Image data not accessible from other threads
//! 4. **Performance**: Updating entire texture every frame
//! 5. **Color Space**: sRGB vs Linear confusion in calculations

// Rust: Multiple selective imports from bevy modules
use bevy::color::{
    // Rust: Color distance calculation trait
    // GAME DESIGN: Used for "collision detection" between colors
    // When drawing pixel hits existing color, change drawing color
    color_difference::EuclideanDistance, 
    // Rust: CSS color constants
    // VISUAL DESIGN: Standard web colors for consistency
    palettes::css
};
use bevy::prelude::*;
// Rust: Low-level rendering types
use bevy::render::{
    // Rust: Asset usage flags
    // BEVY ARCHITECTURE: Tells GPU how texture will be used
    // Optimization hint for driver/hardware
    render_asset::RenderAssetUsages,
    // Rust: Texture creation types
    render_resource::{Extent3d, TextureDimension, TextureFormat},
};
// Rust: External random number generation
// RUST ECOSYSTEM: rand is the de facto RNG library
use rand::{Rng, SeedableRng};  // Traits for RNG
use rand_chacha::ChaCha8Rng;   // Specific RNG implementation
// WHY ChaCha8? Cryptographically secure, deterministic, fast
// Good for procedural generation where security matters less

// Rust: Module-level constants
// u32 type for texture dimensions (no decimals needed)
// PERFORMANCE: Power of 2 dimensions (256) are GPU-friendly
// Better texture cache alignment, mipmap generation
const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 256;

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        .add_plugins(DefaultPlugins)
        // In this example, we will use a fixed timestep to draw a pattern on the screen
        // one pixel at a time, so the pattern will gradually emerge over time, and
        // the speed at which it appears is not tied to the framerate.
        // Let's make the fixed update very fast, so it doesn't take too long. :)
        // Rust: Fixed timestep resource with generic parameter
        // Time::<Fixed> specifies fixed update timing
        // from_hz(1024.0) = 1024 updates per second
        //
        // GAME DESIGN: Fixed timestep benefits
        // - Deterministic: Same behavior on all hardware
        // - Networkable: Easy to sync across clients
        // - Reproducible: Bugs easier to track down
        // - Fair: Game speed independent of framerate
        //
        // PERFORMANCE: 1024 Hz is VERY fast!
        // - Each update draws 1 pixel
        // - 256x256 = 65,536 pixels total
        // - Full image in ~64 seconds
        .insert_resource(Time::<Fixed>::from_hz(1024.0))
        // Rust: System registration
        .add_systems(Startup, setup)
        // Rust: FixedUpdate schedule runs at fixed intervals
        .add_systems(FixedUpdate, draw)
        .run();
}

/// Store the image handle that we will draw to, here.
// Rust: Derive Resource trait for global access
#[derive(Resource)]
// Rust: Tuple struct (newtype pattern)
// Wraps Handle<Image> in a named type
struct MyProcGenImage(Handle<Image>);

// Rust: Resource for deterministic random number generation
#[derive(Resource)]
// Rust: Tuple struct wrapping RNG state
struct SeededRng(ChaCha8Rng);

// Rust: Setup system with mutable resource access
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Rust: Spawn 2D camera
    commands.spawn(Camera2d);

    // Create an image that we are going to draw into
    // Rust: Create mutable image for pixel manipulation
    let mut image = Image::new_fill(
        // 2D image of size 256x256
        // Rust: Struct literal for 3D extent (depth=1 for 2D)
        Extent3d {
            width: IMAGE_WIDTH,   // Const reference
            height: IMAGE_HEIGHT, // Const reference
            depth_or_array_layers: 1,  // Single layer for 2D
        },
        // Rust: Enum variant for 2D texture
        TextureDimension::D2,
        // Initialize it with a beige color
        // Rust: Method call on CSS color constant
        // & takes reference to array for function parameter
        &(css::BEIGE.to_u8_array()),
        // Use the same encoding as the color we set
        // Rust: RGBA format with 8 bits per channel
        TextureFormat::Rgba8UnormSrgb,
        // Rust: Bitwise OR of usage flags
        // | operator combines multiple enum flags
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    // To make it extra fancy, we can set the Alpha of each pixel,
    // so that it fades out in a circular fashion.
    // Rust: Nested for loops over image dimensions
    // Range 0..n is exclusive of n
    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            // Rust: Calculate center point with type conversion
            // as f32 converts u32 to f32 for floating-point math
            let center = Vec2::new(IMAGE_WIDTH as f32 / 2.0, IMAGE_HEIGHT as f32 / 2.0);
            // Rust: Method chaining - min() then as f32 conversion
            let max_radius = IMAGE_HEIGHT.min(IMAGE_WIDTH) as f32 / 2.0;
            // Rust: Calculate distance from center
            let r = Vec2::new(x as f32, y as f32).distance(center);
            // Rust: Alpha calculation with clamp to prevent values > 1.0
            let a = 1.0 - (r / max_radius).clamp(0.0, 1.0);

            // Here we will set the A value by accessing the raw data bytes.
            // (it is the 4th byte of each pixel, as per our `TextureFormat`)

            // Find our pixel by its coordinates
            // Rust: Get mutable reference to pixel bytes
            // UVec3::new() for 3D coordinates (z=0 for 2D)
            // unwrap() panics if pixel coordinates are invalid
            let pixel_bytes = image.pixel_bytes_mut(UVec3::new(x, y, 0)).unwrap();
            // Convert our f32 to u8
            // Rust: Type conversion chain - f32 * f32 as u8
            // u8::MAX is the maximum u8 value (255)
            pixel_bytes[3] = (a * u8::MAX as f32) as u8;
        }
    }

    // Add it to Bevy's assets, so it can be used for rendering
    // this will give us a handle we can use
    // (to display it in a sprite, or as part of UI, etc.)
    // Rust: Add image to asset storage, returns Handle<Image>
    let handle = images.add(image);

    // Create a sprite entity using our image
    // Rust: Associated function creates Sprite from image handle
    // clone() duplicates Handle (cheap - reference counted)
    commands.spawn(Sprite::from_image(handle.clone()));
    // Rust: Insert global resource with handle
    // MyProcGenImage() is tuple struct constructor
    commands.insert_resource(MyProcGenImage(handle));

    // We're seeding the PRNG here to make this example deterministic for testing purposes.
    // This isn't strictly required in practical use unless you need your app to be deterministic.
    // Rust: Create deterministic RNG with fixed seed
    let seeded_rng = ChaCha8Rng::seed_from_u64(19878367467712);
    // Rust: Insert RNG as global resource
    commands.insert_resource(SeededRng(seeded_rng));
}

/// Every fixed update tick, draw one more pixel to make a spiral pattern
// Rust: System with Local state and resources
fn draw(
    // Rust: Read-only access to our image handle resource
    my_handle: Res<MyProcGenImage>,
    // Rust: Mutable access to image assets
    mut images: ResMut<Assets<Image>>,
    // Used to keep track of where we are
    // Rust: Local<T> provides per-system persistent state
    // Survives between system runs, initialized with Default
    //
    // BEVY ARCHITECTURE: Local<T> vs Resource
    // - Local: Per-system state, not shared
    // - Resource: Global state, shared between systems
    // - Local perfect for iteration counters, caches
    mut i: Local<u32>,
    // Rust: Local state for current drawing color
    // GAME DESIGN: Dynamic color changes create visual interest
    // Prevents monotonous patterns
    mut draw_color: Local<Color>,
    // Rust: Mutable RNG resource
    // RUST PATTERN: Newtype wrapping for type safety
    // Can't accidentally pass wrong RNG type
    mut seeded_rng: ResMut<SeededRng>,
) {
    // Rust: Check if this is the first run
    // * dereferences Local<u32> to u32
    if *i == 0 {
        // Generate a random color on first run.
        // Rust: Create linear RGB color with random components
        *draw_color = Color::linear_rgb(
            // Rust: Raw identifier r#gen to avoid keyword conflict
            // .0 accesses tuple struct field (the ChaCha8Rng)
            seeded_rng.0.r#gen(),  // Random f32 for red
            seeded_rng.0.r#gen(),  // Random f32 for green
            seeded_rng.0.r#gen(),  // Random f32 for blue
        );
    }

    // Get the image from Bevy's asset storage.
    // Rust: Get mutable reference to image asset
    // &my_handle.0 accesses the Handle<Image> inside MyProcGenImage
    // expect() panics with message if asset not found
    let image = images.get_mut(&my_handle.0).expect("Image not found");

    // Compute the position of the pixel to draw.
    // Rust: Calculate spiral parameters
    let center = Vec2::new(IMAGE_WIDTH as f32 / 2.0, IMAGE_HEIGHT as f32 / 2.0);
    let max_radius = IMAGE_HEIGHT.min(IMAGE_WIDTH) as f32 / 2.0;
    // Rust: f64 literals get converted to f32
    let rot_speed = 0.0123;   // Radians per iteration
    let period = 0.12345;     // Frequency of radius oscillation

    // Rust: Parametric spiral equation
    // ops::sin() from bevy::math::ops module
    let r = ops::sin(*i as f32 * period) * max_radius;
    // Rust: Vec2::from_angle() creates unit vector from angle
    // * r scales it, + center translates to image center
    let xy = Vec2::from_angle(*i as f32 * rot_speed) * r + center;
    // Rust: Tuple destructuring with type conversion
    let (x, y) = (xy.x as u32, xy.y as u32);

    // Get the old color of that pixel.
    // Rust: Get current pixel color
    // unwrap() panics if coordinates are out of bounds
    let old_color = image.get_color_at(x, y).unwrap();

    // If the old color is our current color, change our drawing color.
    // Rust: Tolerance for floating-point color comparison
    // 1/255 is the smallest difference between 8-bit color values
    let tolerance = 1.0 / 255.0;
    // Rust: EuclideanDistance trait method for color comparison
    // &draw_color takes reference for comparison
    if old_color.distance(&draw_color) <= tolerance {
        // Rust: Generate new random color
        *draw_color = Color::linear_rgb(
            seeded_rng.0.r#gen(),
            seeded_rng.0.r#gen(),
            seeded_rng.0.r#gen(),
        );
    }

    // Set the new color, but keep old alpha value from image.
    // Rust: Method chaining to set pixel color
    image
        .set_color_at(
            x, y, 
            // Rust: Create new color with old alpha channel
            // with_alpha() method preserves RGB, changes alpha
            draw_color.with_alpha(old_color.alpha())
        )
        .unwrap();

    // Rust: Increment counter for next iteration
    *i += 1;
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Local<T> State**:
//    - Per-system persistent data
//    - Survives between system runs
//    - Initialized with Default trait
//    - Use * to dereference and access value
//
// 2. **Raw Identifiers**:
//    - `r#gen` - use keywords as identifiers
//    - Required when trait method names conflict
//    - Rng::gen() vs keyword "gen"
//
// 3. **Tuple Struct Access**:
//    - `.0` accesses first field of tuple struct
//    - SeededRng(ChaCha8Rng) -> .0 gets the RNG
//    - Common pattern for newtype wrappers
//
// 4. **Type Conversions**:
//    - `as f32` converts integers to floats
//    - `as u32` converts floats to integers
//    - Required for pixel coordinates and math
//
// 5. **Fixed Timestep**:
//    - Time::<Fixed> runs systems at constant rate
//    - Independent of framerate fluctuations
//    - Good for deterministic animations
//
// 6. **Image Manipulation**:
//    - `get_color_at()` / `set_color_at()` for pixels
//    - `pixel_bytes_mut()` for raw byte access
//    - Both approaches available in Bevy
//
// 7. **Asset Handle Pattern**:
//    - Handle<T> is reference-counted pointer
//    - .clone() is cheap (just increments counter)
//    - Assets stored in global collections
//
// 8. **Error Handling**:
//    - `.unwrap()` panics on errors
//    - `.expect("message")` panics with custom message
//    - Use when you're certain operation will succeed
//
// 9. **Bitwise Operations**:
//    - `|` combines enum flags
//    - `RenderAssetUsages::A | RenderAssetUsages::B`
//    - Common for configuration options
