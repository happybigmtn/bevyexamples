//! Illustrates bloom post-processing in 2d.
//!
//! # Bloom Effect: Making Bright Things Glow
//!
//! Bloom is a post-processing effect that makes bright areas "glow" by bleeding
//! light into surrounding pixels. It simulates how camera lenses and human eyes
//! perceive very bright light sources.
//!
//! ## How Bloom Works:
//!
//! 1. **Threshold**: Only pixels brighter than a threshold glow
//! 2. **Blur**: Bright areas are blurred to create the glow
//! 3. **Composite**: Blurred layer is added back to the original
//! 4. **Scale**: Controls how far the glow spreads
//!
//! ## Common Uses:
//!
//! - **Neon signs** and LED displays
//! - **Sun and light sources** in games
//! - **Magic effects** and energy beams
//! - **UI highlights** for important elements
//! - **Retro aesthetics** (synthwave, cyberpunk)
//!
//! ## This Example Shows:
//!
//! - Interactive bloom parameter adjustment
//! - Different composite modes (energy-conserving vs additive)
//! - Tonemapping integration
//! - Bright 2D sprites and meshes with glow effects
//!
//! ## Game Design Context: Visual Polish and Atmosphere
//!
//! Bloom is a crucial tool for creating atmosphere and guiding player attention:
//!
//! 1. **Visual Hierarchy**: Bright glowing objects naturally draw the eye
//!    - Use bloom for important collectibles or objectives
//!    - Make dangerous elements glow red/orange
//!    - Healing items often glow green/blue
//!
//! 2. **Atmosphere Creation**:
//!    - Sci-fi: Neon lights, holographic displays, energy weapons
//!    - Fantasy: Magic spells, enchanted items, portals
//!    - Horror: Eerie glows in darkness, supernatural phenomena
//!    - Retro: Arcade cabinet screens, synthwave aesthetics
//!
//! 3. **Gameplay Communication**:
//!    - Power-up states: Player glows when invincible
//!    - Charge indicators: Weapons glow brighter as they charge
//!    - Environmental hints: Interactable objects have subtle glow
//!    - Status effects: Poisoned = green glow, burning = orange glow
//!
//! ## Rust Fundamentals: Option Types and Pattern Matching
//!
//! This example showcases advanced Rust patterns:
//!
//! 1. **Option in Queries**: `Option<&mut Bloom>` represents components that might not exist
//!    - Safe alternative to null pointers
//!    - Forces explicit handling of missing data
//!    - No runtime crashes from accessing missing components
//!
//! 2. **Match Expressions**: Exhaustive pattern matching on enums
//!    ```rust
//!    match bloom {
//!        Some(mut bloom) => { /* bloom exists */ }
//!        None => { /* no bloom */ }
//!    }
//!    ```
//!
//! 3. **Mutable References**: `&mut` allows modification
//!    - Only one mutable reference at a time (prevents data races)
//!    - Compiler enforces memory safety
//!
//! ## Bevy Architecture: Post-Processing Pipeline
//!
//! Understanding Bevy's rendering pipeline:
//!
//! 1. **Render Graph**: Bloom is a node in the render graph
//!    - Scene renders to HDR texture
//!    - Bloom extracts bright pixels
//!    - Multiple blur passes create glow
//!    - Final composite combines results
//!
//! 2. **Component-Based Effects**: Effects are just components
//!    - Add `Bloom` component to enable
//!    - Remove component to disable
//!    - No complex state management needed
//!
//! 3. **HDR Pipeline**: High Dynamic Range rendering
//!    - Colors can exceed 1.0 (standard white)
//!    - Allows realistic light intensities
//!    - Tonemapping compresses to display range
//!
//! ## Real-World Applications
//!
//! 1. **Mobile Games**: Use bloom sparingly (performance cost)
//! 2. **PC/Console**: Can afford more bloom passes for quality
//! 3. **VR**: Bloom can reduce eye strain from bright objects
//! 4. **Accessibility**: Provide bloom toggle for photosensitive players
//!
//! ## Performance Considerations
//!
//! 1. **Fill Rate**: Bloom uses multiple full-screen passes
//!    - Each blur pass processes every pixel
//!    - Higher resolution = more pixels to process
//!    - Consider dynamic resolution scaling
//!
//! 2. **Optimization Strategies**:
//!    - Reduce bloom scale for distant objects
//!    - Use lower resolution for blur passes
//!    - Limit number of blur iterations
//!    - Disable bloom on low-end hardware
//!
//! 3. **Memory Usage**: 
//!    - Multiple render targets for blur pyramid
//!    - HDR textures use more memory (16-bit per channel)
//!    - Consider memory budget on mobile/web
//!
//! ## Common Pitfalls
//!
//! 1. **Overuse**: Too much bloom looks unrealistic
//! 2. **Wrong Threshold**: Too low = everything glows, too high = no effect
//! 3. **Energy Conservation**: Additive mode can blow out colors
//! 4. **Forgetting Tonemapping**: HDR colors look wrong without it
//! 5. **Performance**: Not profiling bloom impact on target hardware

// Rust: Complex nested imports from bevy modules
use bevy::{
    // Rust: Core rendering pipeline components
    core_pipeline::{
        // Rust: Bloom effect configuration
        bloom::{Bloom, BloomCompositeMode},
        // Rust: HDR to SDR tone mapping
        tonemapping::{DebandDither, Tonemapping},
    },
    // Rust: Common Bevy types
    prelude::*,
};

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Add default plugins
        .add_plugins(DefaultPlugins)
        // Rust: Register systems for different schedules
        .add_systems(Startup, setup)
        .add_systems(Update, update_bloom_settings)
        // Rust: Start the game loop
        .run();
}

// Rust: Setup system with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Mutable access to mesh assets (for 2D shapes)
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to 2D color materials
    mut materials: ResMut<Assets<ColorMaterial>>,
    // Rust: Asset server for loading images
    asset_server: Res<AssetServer>,
) {
    // Rust: Spawn camera with bloom effects
    // GAME DESIGN: Black background maximizes bloom contrast
    // Bright objects on dark backgrounds create the most dramatic glow
    commands.spawn((
        // Rust: 2D camera component
        Camera2d,
        // Rust: Custom camera configuration
        Camera {
            // Rust: Custom clear color (black background for bloom contrast)
            // BEVY ARCHITECTURE: ClearColorConfig enum options:
            // - Default: Uses global clear color resource
            // - Custom: Override with specific color
            // - None: Don't clear (useful for render-to-texture)
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            // Rust: Default other camera settings
            ..default()
        },
        // 2. Using a tonemapper that desaturates to white is recommended
        // Rust: Enum variant for tone mapping algorithm
        // RENDERING THEORY: Tonemapping algorithms:
        // - TonyMcMapface: Good for games, preserves color vibrancy
        // - AcesFitted: Film industry standard
        // - AgX: Blender's new default, good highlight handling
        // - Reinhard: Simple but can lose detail in bright areas
        Tonemapping::TonyMcMapface,
        // 3. Enable bloom for the camera
        // Rust: Default bloom configuration
        // PERFORMANCE: Default settings are optimized for quality/performance balance
        // Customize for your target hardware
        Bloom::default(),
        // Optional: bloom causes gradients which cause banding
        // Rust: Enable dithering to reduce color banding
        // VISUAL QUALITY: Dithering adds noise to break up color bands
        // Especially visible in dark gradients and fog
        DebandDither::Enabled,
    ));

    // Sprite
    // Rust: Spawn sprite entity with high brightness
    commands.spawn(Sprite {
        // Rust: Load image asset
        // BEVY ARCHITECTURE: Asset loading is async
        // Handle returned immediately, asset loads in background
        image: asset_server.load("branding/bevy_bird_dark.png"),
        // 4. Put something bright in a dark environment to see the effect
        // Rust: RGB values > 1.0 create HDR bright colors
        // RENDERING: HDR (High Dynamic Range) colors
        // - Standard range: 0.0 to 1.0 (LDR - Low Dynamic Range)
        // - HDR allows values > 1.0 for realistic lighting
        // - Real world has huge brightness variations (sun vs shadow)
        // GAME DESIGN: Use HDR for:
        // - Light sources (sun, fire, lasers)
        // - Magical effects (spells, power-ups)
        // - Emissive materials (screens, neon)
        color: Color::srgb(5.0, 5.0, 5.0),  // 5x brighter than white
        // Rust: Option<Vec2> for custom sprite size
        // splat() creates Vec2 with same value for x and y
        // RUST PATTERN: Constructor methods like splat() improve readability
        // Vec2::splat(160.0) clearer than Vec2::new(160.0, 160.0)
        custom_size: Some(Vec2::splat(160.0)),
        // Rust: Default other sprite properties
        ..default()
    });

    // Circle mesh
    commands.spawn((
        // Rust: Create circle mesh with radius 100
        // BEVY PATTERN: Mesh primitives automatically tessellated
        // Circle becomes ~32 triangles arranged in a fan
        Mesh2d(meshes.add(Circle::new(100.))),
        // 4. Put something bright in a dark environment to see the effect
        // Rust: Bright magenta color for glow effect
        // COLOR THEORY: Magenta (red + blue) creates cyberpunk aesthetic
        // Common in synthwave/retrowave visual styles
        MeshMaterial2d(materials.add(Color::srgb(7.5, 0.0, 7.5))),
        // Rust: Position to the left of center
        // COORDINATE SYSTEM: Bevy 2D uses screen-space coordinates
        // (0, 0) is center, +X right, +Y up (unlike many 2D engines)
        Transform::from_translation(Vec3::new(-200., 0., 0.)),
    ));

    // Hexagon mesh
    commands.spawn((
        // Rust: Create regular polygon (radius 100, 6 sides = hexagon)
        Mesh2d(meshes.add(RegularPolygon::new(100., 6))),
        // 4. Put something bright in a dark environment to see the effect
        // Rust: Bright cyan-like color
        MeshMaterial2d(materials.add(Color::srgb(6.25, 9.4, 9.1))),
        // Rust: Position to the right of center
        Transform::from_translation(Vec3::new(200., 0., 0.)),
    ));

    // UI
    // Rust: UI text for displaying controls
    commands.spawn((
        // Rust: Empty text (filled by update system)
        Text::default(),
        // Rust: UI node for positioning
        Node {
            // Rust: Absolute positioning mode
            position_type: PositionType::Absolute,
            // Rust: Position from edges using pixel values
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// ------------------------------------------------------------------------------------------------

// Rust: Complex interactive system for bloom parameter adjustment
fn update_bloom_settings(
    // Rust: Single query with complex tuple
    // (Entity, &Tonemapping, Option<&mut Bloom>) - camera data
    // With<Camera> filter ensures only camera entities
    //
    // BEVY ARCHITECTURE: Query types
    // - Query<T>: Iterator over all matching entities
    // - Single<T>: Exactly one entity (panics otherwise)
    // - Option<T> in query: Component might not exist
    //
    // RUST FUNDAMENTALS: Tuple queries
    // Each element can have different access patterns:
    // - Entity: Copy type, always available
    // - &Tonemapping: Immutable reference
    // - Option<&mut Bloom>: Optional mutable reference
    camera: Single<(Entity, &Tonemapping, Option<&mut Bloom>), With<Camera>>,
    // Rust: Single mutable text for UI display
    mut text: Single<&mut Text>,
    // Rust: Commands for adding/removing components
    mut commands: Commands,
    // Rust: Keyboard input resource
    // GAME DESIGN: ButtonInput tracks key state across frames
    // - just_pressed(): True for one frame when key goes down
    // - pressed(): True while key is held
    // - just_released(): True for one frame when key goes up
    keycode: Res<ButtonInput<KeyCode>>,
    // Rust: Time resource for delta calculations
    time: Res<Time>,
) {
    // Rust: Destructure Single query result into tuple
    // into_inner() extracts the wrapped value
    // RUST PATTERN: Destructuring assignment
    // More readable than accessing tuple fields (.0, .1, .2)
    let (camera_entity, tonemapping, bloom) = camera.into_inner();

    // Rust: Pattern match on Option<&mut Bloom>
    match bloom {
        // Rust: Some variant contains mutable reference to Bloom
        Some(mut bloom) => {
            // Rust: String literal with .to_string() conversion
            text.0 = "Bloom (Toggle: Space)\n".to_string();
            // Rust: format! macro with string interpolation
            // push_str() appends to existing string
            text.push_str(&format!("(Q/A) Intensity: {}\n", bloom.intensity));
            text.push_str(&format!(
                "(W/S) Low-frequency boost: {}\n",
                bloom.low_frequency_boost
            ));
            text.push_str(&format!(
                "(E/D) Low-frequency boost curvature: {}\n",
                bloom.low_frequency_boost_curvature
            ));
            text.push_str(&format!(
                "(R/F) High-pass frequency: {}\n",
                bloom.high_pass_frequency
            ));
            text.push_str(&format!(
                "(T/G) Mode: {}\n",
                // Rust: Nested match expression for enum display
                match bloom.composite_mode {
                    BloomCompositeMode::EnergyConserving => "Energy-conserving",
                    BloomCompositeMode::Additive => "Additive",
                }
            ));
            text.push_str(&format!("(Y/H) Threshold: {}\n", bloom.prefilter.threshold));
            text.push_str(&format!(
                "(U/J) Threshold softness: {}\n",
                bloom.prefilter.threshold_softness
            ));
            text.push_str(&format!("(I/K) Horizontal Scale: {}\n", bloom.scale.x));

            // Rust: Toggle bloom off with Space key
            if keycode.just_pressed(KeyCode::Space) {
                // Rust: Remove component from entity
                // Turbofish ::<Bloom> specifies component type
                commands.entity(camera_entity).remove::<Bloom>();
            }

            // Rust: Get delta time for smooth parameter adjustment
            let dt = time.delta_secs();

            // Rust: Continuous key input for parameter adjustment
            // pressed() returns true while key is held down
            if keycode.pressed(KeyCode::KeyA) {
                // Rust: Compound assignment with division
                bloom.intensity -= dt / 10.0;  // Slow adjustment
            }
            if keycode.pressed(KeyCode::KeyQ) {
                bloom.intensity += dt / 10.0;
            }
            // Rust: clamp() constrains value to range [min, max]
            bloom.intensity = bloom.intensity.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyS) {
                bloom.low_frequency_boost -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyW) {
                bloom.low_frequency_boost += dt / 10.0;
            }
            bloom.low_frequency_boost = bloom.low_frequency_boost.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyD) {
                bloom.low_frequency_boost_curvature -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyE) {
                bloom.low_frequency_boost_curvature += dt / 10.0;
            }
            bloom.low_frequency_boost_curvature =
                bloom.low_frequency_boost_curvature.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyF) {
                bloom.high_pass_frequency -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyR) {
                bloom.high_pass_frequency += dt / 10.0;
            }
            bloom.high_pass_frequency = bloom.high_pass_frequency.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyG) {
                bloom.composite_mode = BloomCompositeMode::Additive;
            }
            if keycode.pressed(KeyCode::KeyT) {
                bloom.composite_mode = BloomCompositeMode::EnergyConserving;
            }

            if keycode.pressed(KeyCode::KeyH) {
                bloom.prefilter.threshold -= dt;
            }
            if keycode.pressed(KeyCode::KeyY) {
                bloom.prefilter.threshold += dt;
            }
            bloom.prefilter.threshold = bloom.prefilter.threshold.max(0.0);

            if keycode.pressed(KeyCode::KeyJ) {
                bloom.prefilter.threshold_softness -= dt / 10.0;
            }
            if keycode.pressed(KeyCode::KeyU) {
                bloom.prefilter.threshold_softness += dt / 10.0;
            }
            bloom.prefilter.threshold_softness = bloom.prefilter.threshold_softness.clamp(0.0, 1.0);

            if keycode.pressed(KeyCode::KeyK) {
                bloom.scale.x -= dt * 2.0;
            }
            if keycode.pressed(KeyCode::KeyI) {
                bloom.scale.x += dt * 2.0;
            }
            bloom.scale.x = bloom.scale.x.clamp(0.0, 16.0);
        }

        // Rust: None variant means no Bloom component
        None => {
            text.0 = "Bloom: Off (Toggle: Space)\n".to_string();

            // Rust: Toggle bloom on with Space key
            if keycode.just_pressed(KeyCode::Space) {
                // Rust: Insert component with default values
                commands.entity(camera_entity).insert(Bloom::default());
            }
        }
    }

    // Rust: Display tonemapping mode (outside match)
    // {:?} uses Debug formatting
    text.push_str(&format!("(O) Tonemapping: {:?}\n", tonemapping));
    if keycode.just_pressed(KeyCode::KeyO) {
        // Rust: Method chaining for entity commands
        commands
            .entity(camera_entity)
            // Rust: Call helper function and insert result
            .insert(next_tonemap(tonemapping));
    }
}

/// Get the next Tonemapping algorithm
// Rust: Helper function that cycles through tonemapping modes
fn next_tonemap(tonemapping: &Tonemapping) -> Tonemapping {
    // Rust: Exhaustive match on enum variants
    // Each variant maps to the next one in sequence
    match tonemapping {
        Tonemapping::None => Tonemapping::AcesFitted,
        Tonemapping::AcesFitted => Tonemapping::AgX,
        Tonemapping::AgX => Tonemapping::BlenderFilmic,
        Tonemapping::BlenderFilmic => Tonemapping::Reinhard,
        Tonemapping::Reinhard => Tonemapping::ReinhardLuminance,
        Tonemapping::ReinhardLuminance => Tonemapping::SomewhatBoringDisplayTransform,
        Tonemapping::SomewhatBoringDisplayTransform => Tonemapping::TonyMcMapface,
        // Rust: Cycles back to beginning
        Tonemapping::TonyMcMapface => Tonemapping::None,
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **HDR Colors**:
//    - RGB values > 1.0 create bright colors
//    - Color::srgb(5.0, 5.0, 5.0) = 5x brighter than white
//    - Essential for bloom effect visibility
//
// 2. **Option in Queries**:
//    - `Option<&mut Bloom>` - component might not exist
//    - Pattern match on Some/None variants
//    - Safe handling of optional components
//
// 3. **Single Query**:
//    - More efficient than Query for unique entities
//    - `into_inner()` extracts wrapped tuple
//    - Good for camera or UI systems
//
// 4. **Entity Commands**:
//    - `.insert()` adds components to entities
//    - `.remove::<T>()` removes specific component
//    - Changes applied at end of frame
//
// 5. **Time-based Input**:
//    - `delta_secs()` for frame-rate independent adjustment
//    - `pressed()` for continuous input
//    - `just_pressed()` for single-frame detection
//
// 6. **Method Chaining**:
//    - String methods: `.to_string()`, `.push_str()`
//    - Math methods: `.clamp()` for value constraints
//    - Builder patterns throughout Bevy
//
// 7. **2D vs 3D Components**:
//    - `Camera2d` instead of `Camera3d`
//    - `Mesh2d` instead of `Mesh3d`
//    - `ColorMaterial` instead of `StandardMaterial`
//
// 8. **Post-Processing Pipeline**:
//    - Bloom operates on final rendered image
//    - Tonemapping converts HDR to display range
//    - Dithering reduces color banding artifacts
