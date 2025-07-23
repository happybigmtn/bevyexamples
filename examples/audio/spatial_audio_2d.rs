//! This example illustrates how to load and play an audio file, and control where the sounds seems to come from.
//! 
//! ## Key Concepts
//! 
//! - **Spatial Audio**: Sound that appears to come from a specific location in space.
//!   In games, this means sounds get quieter with distance and pan between speakers
//!   based on direction.
//! 
//! - **Stereo Panning**: When a sound source is to your left, it plays louder in the
//!   left speaker. When it's to your right, it plays louder in the right speaker.
//!   This creates the illusion of directional sound.
//! 
//! - **Distance Attenuation**: Sounds get quieter as they get farther away, following
//!   the inverse square law (like real sound in air).
//! 
//! - **Listener vs Emitter**: The listener represents the player's "ears" in the game
//!   world. Emitters are sound sources. Spatial audio calculates volume and panning
//!   based on their relative positions.
//!
//! ## Audio Theory: Spatial Perception
//!
//! **How Humans Locate Sound**:
//! 1. **Interaural Time Difference (ITD)**: Sound reaches nearer ear first
//!    - Most effective below 1500 Hz
//!    - Max delay: ~0.6ms (head width / speed of sound)
//! 2. **Interaural Level Difference (ILD)**: Head shadows high frequencies
//!    - Most effective above 1500 Hz  
//!    - Can be 20dB difference at 6kHz
//! 3. **Head-Related Transfer Function (HRTF)**: Ear shape filters sound
//!    - Helps distinguish front/back, up/down
//!    - Unique to each person
//!
//! **Distance Perception Cues**:
//! - **Volume**: Quieter = farther (inverse square law)
//! - **High Frequency Rolloff**: Air absorbs highs over distance
//! - **Reverb Ratio**: More reverb = farther from source
//! - **Motion Parallax**: Near sounds move faster across stereo field
//!
//! ## Game Design Context: 2D Spatial Audio
//!
//! **Why Spatial Audio in 2D Games?**
//! - **Off-screen Awareness**: Hear enemies before seeing them
//! - **Immersion**: World feels larger than visible area
//! - **Gameplay Feedback**: Audio hints for collectibles, secrets
//! - **Accessibility**: Audio cues for vision-impaired players
//!
//! **2D Spatial Audio Examples**:
//! - **Terraria**: Hear zombies approaching from sides
//! - **Hollow Knight**: Environmental ambience shifts with position
//! - **Don't Starve**: Directional creature sounds at night
//! - **Ori**: Musical elements fade by proximity
//!
//! ## Performance Optimization: Spatial Calculations
//!
//! **Per-Frame Costs** (per sound source):
//! ```text
//! Operation          | CPU Cycles | Note
//! -------------------|------------|-----
//! Distance calc      | 10-20      | Square root
//! Volume attenuation | 5-10       | Division
//! Stereo panning     | 10-15      | Trig functions
//! HRTF filter        | 100-500    | Optional quality
//! ```
//!
//! **Optimization Strategies**:
//! 1. **Distance Culling**: Don't process sounds beyond max range
//! 2. **LOD Audio**: Simpler processing for distant sounds
//! 3. **Update Rate**: Update positions every N frames
//! 4. **Squared Distance**: Avoid sqrt when possible
//!
//! ## Real-World Applications
//!
//! **Game Audio Middleware Comparison**:
//! - **FMOD**: Industry standard, excellent 2D spatial
//! - **Wwise**: More complex, better for 3D
//! - **OpenAL**: Open source, basic spatial
//! - **Web Audio**: Browser-based, good 2D support
//!
//! **Coordinate System Considerations**:
//! ```text
//! Screen Space:  Y+ is down, origin at top-left
//! World Space:   Y+ is up, origin at center (Bevy)
//! Audio Space:   Right-handed, listener looks down -Z
//! ```
//!
//! ## Advanced Techniques: Enhanced 2D Audio
//!
//! **Doppler Effect**: Pitch shift for moving sources
//! ```rust
//! let velocity = (current_pos - last_pos) / delta_time;
//! let doppler_factor = 1.0 + (velocity · listener_dir) / speed_of_sound;
//! pitch = base_pitch * doppler_factor;
//! ```
//!
//! **Reverb Zones**: Different reverb by area
//! - Cave: Long reverb, dampened highs
//! - Forest: Short reverb, scattered echoes
//! - Open field: No reverb, wind ambience
//!
//! ## Common Issues and Solutions
//!
//! **Problem**: Sounds pop in/out at max distance
//! - **Solution**: Fade over last 10% of range
//!
//! **Problem**: Stereo panning too subtle
//! - **Solution**: Increase listener ear separation
//!
//! **Problem**: Behind sounds identical to front
//! - **Solution**: Apply subtle filter for rear sounds

use bevy::{
    // Import audio-specific types for spatial audio configuration
    audio::{AudioPlugin, SpatialScale},
    // CSS color palette for convenient color names
    color::palettes::css::*,
    // math::ops is included in prelude, providing sin/cos functions
    prelude::*,
    // Stopwatch for timing the emitter's movement
    time::Stopwatch,
};

// ## Rust Programming Fundamentals: Import Organization
//
// **Nested Imports**:
// ```rust
// // Verbose
// use bevy::audio::AudioPlugin;
// use bevy::audio::SpatialScale;
// use bevy::color::palettes::css::RED;
// use bevy::color::palettes::css::BLUE;
//
// // Grouped (better)
// use bevy::{
//     audio::{AudioPlugin, SpatialScale},
//     color::palettes::css::*,
// };
// ```
//
// **Glob Imports**: `*` imports all public items
// - Use sparingly (namespace pollution)
// - OK for well-known sets like colors
// - Prefer explicit imports in libraries

/// Spatial audio scale factor for 2D.
/// 
/// By default, Bevy's spatial audio treats 1 world unit = 1 meter for audio calculations.
/// In 2D games, 1 pixel often = 1 world unit, which would make sounds fade out after
/// just a few pixels! 
/// 
/// This scale factor makes 100 pixels = 1 audio meter, giving more reasonable falloff
/// distances for a 2D game. At this scale, a sound might be audible up to 1000-2000 pixels away.
///
/// ## Scale Factor Mathematics
///
/// **Distance Attenuation Formula**:
/// ```text
/// volume = min_volume + (1 - min_volume) / (1 + distance² / reference_distance²)
/// ```
///
/// **With Different Scales**:
/// ```text
/// Scale    | 1 Audio Meter | Audible Range | Use Case
/// ---------|---------------|---------------|----------
/// 1.0      | 1 pixel       | 10-20 px      | Miniature games
/// 0.1      | 10 pixels     | 100-200 px    | Retro arcade
/// 0.01     | 100 pixels    | 1000-2000 px  | Standard 2D (this example)
/// 0.001    | 1000 pixels   | 10k-20k px    | Large open world
/// ```
///
/// **Real-World Audio Distances**:
/// - Whisper: Audible to 2m
/// - Normal speech: Audible to 10m  
/// - Shout: Audible to 100m
/// - Gunshot: Audible to 3000m
///
/// **Choosing Your Scale**:
/// 1. Measure typical screen width in pixels
/// 2. Decide how many screens away sounds should be audible
/// 3. Scale = 1 / (screens × screen_width / desired_audio_meters)
const AUDIO_SCALE: f32 = 1. / 100.0;

fn main() {
    App::new()
        .add_plugins(
            // Configure the audio plugin with our custom spatial scale
            DefaultPlugins.set(AudioPlugin {
                // Tell the audio system to use our 2D scale factor
                default_spatial_scale: SpatialScale::new_2d(AUDIO_SCALE),
                // Keep other audio settings at their defaults
                ..default()
            })
        )
        .add_systems(Startup, setup)
        // Update systems for interactive spatial audio demo:
        .add_systems(Update, update_emitters)  // Moves the sound source
        .add_systems(Update, update_listener)  // Moves the listener with keyboard
        .run();
}

/// Sets up the spatial audio demo scene.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Distance between the listener's "ears" in pixels.
    // A wider gap makes stereo effects more pronounced.
    // Real human ears are about 17cm apart; we exaggerate for visual clarity.
    //
    // ## Ear Separation and Stereo Imaging
    //
    // **Real vs Virtual Ear Spacing**:
    // - Human average: 17cm (6.7 inches)
    // - At our scale: 17cm = 17 pixels (too narrow!)
    // - Exaggerated: 400 pixels = 4 meters (better stereo)
    //
    // **ITD (Interaural Time Difference) Calculation**:
    // ```text
    // max_delay = ear_separation / speed_of_sound
    // At 17cm: 0.17m / 343m/s = 0.495ms
    // At 400px (4m): 4m / 343m/s = 11.7ms (unrealistic but clear)
    // ```
    //
    // **Panning Law** (constant power):
    // ```text
    // angle = atan2(source.y - listener.y, source.x - listener.x)
    // pan = sin(angle)  // -1 (left) to +1 (right)
    // left_gain = cos((pan + 1) * π/4)
    // right_gain = sin((pan + 1) * π/4)
    // ```
    let gap = 400.0;

    // Spawn the sound emitter (the blue circle that makes sound)
    //
    // ## Entity Bundle Pattern
    //
    // **Bundle Composition**:
    // - Visuals: Mesh2d + MeshMaterial2d + Transform
    // - Logic: Emitter (custom component)
    // - Audio: AudioPlayer + PlaybackSettings
    //
    // **Component Order**: Doesn't matter functionally, but convention:
    // 1. Transform (position/rotation/scale)
    // 2. Visual components (mesh, material, sprite)
    // 3. Game logic components
    // 4. Audio components
    // 5. Physics components (if any)
    commands.spawn((
        // Visual representation - blue circle with 15 pixel radius
        //
        // ## 2D Mesh Creation
        //
        // **Circle::new(radius)** creates a circle mesh:
        // - Default: 32 vertices for smooth appearance
        // - Vertices arranged in a fan from center
        // - UV coordinates map texture circularly
        //
        // **Performance Note**:
        // ```rust
        // // For many circles, reuse the mesh:
        // let circle_mesh = meshes.add(Circle::new(15.0));
        // for _ in 0..100 {
        //     commands.spawn(Mesh2d(circle_mesh.clone()));
        // }
        // ```
        Mesh2d(meshes.add(Circle::new(15.0))),
        MeshMaterial2d(materials.add(Color::from(BLUE))),
        // Start position: center X, slightly up from center
        //
        // ## Transform in 2D vs 3D
        //
        // Even in 2D, Transform uses Vec3:
        // - X: Horizontal (right is positive)
        // - Y: Vertical (up is positive in Bevy)
        // - Z: Depth ordering (higher renders on top)
        //
        // **Common 2D Z values**:
        // - Background: -100.0 to -1.0
        // - Gameplay: 0.0
        // - UI overlay: 1.0 to 100.0
        Transform::from_translation(Vec3::new(0.0, 50.0, 0.0)),
        // Custom component to track movement
        Emitter::default(),
        // Load and play the audio file
        AudioPlayer::new(asset_server.load("sounds/Windless Slopes.ogg")),
        // Configure playback: loop forever with spatial audio enabled
        //
        // ## Spatial Audio Configuration
        //
        // **PlaybackSettings::LOOP**: Predefined constant:
        // ```rust
        // PlaybackSettings {
        //     mode: PlaybackMode::Loop,
        //     volume: Volume::Linear(1.0),
        //     speed: 1.0,
        //     paused: false,
        //     spatial: false,  // We override this
        // }
        // ```
        //
        // **with_spatial(true)** enables 3D/2D positioning:
        // - Automatic volume based on distance
        // - Automatic panning based on direction
        // - Respects SpatialScale settings
        PlaybackSettings::LOOP.with_spatial(true),
    ));

    // Create and spawn the spatial listener (the player's ears)
    //
    // ## SpatialListener Architecture
    //
    // **What SpatialListener::new(gap) creates**:
    // ```rust
    // SpatialListener {
    //     left_ear_offset: Vec3::new(-gap / 2.0, 0.0, 0.0),
    //     right_ear_offset: Vec3::new(gap / 2.0, 0.0, 0.0),
    // }
    // ```
    //
    // **How Bevy Calculates Spatial Audio**:
    // 1. Get world positions of left and right ears
    // 2. For each audio source:
    //    - Calculate distance to each ear
    //    - Apply distance attenuation
    //    - Set left/right channel volumes
    // 3. Mix all sources together
    let listener = SpatialListener::new(gap);
    commands.spawn((
        // The listener starts at the origin (0, 0)
        Transform::default(),
        // Required for parent-child relationships
        //
        // ## Visibility in Audio Entities
        //
        // **Why Visibility for non-visual entities?**
        // - Parent entities need Visibility for children to render
        // - Even though listener itself is invisible
        // - Children (ear indicators) inherit visibility
        //
        // **Visibility::default()** equals:
        // ```rust
        // Visibility::Visible
        // ```
        Visibility::default(),
        // The actual spatial listener component
        listener.clone(),
        // Spawn two colored squares as visual indicators for the ears
        //
        // ## Bevy's children! Macro
        //
        // **What children! does**:
        // 1. Spawns each child entity
        // 2. Returns their Entity IDs
        // 3. Creates Parent/Children relationship
        //
        // **Parent-Child Transform Math**:
        // ```
        // child_world_pos = parent_transform * child_local_transform
        // ```
        //
        // **Alternative syntax without macro**:
        // ```rust
        // let parent = commands.spawn(...).id();
        // let child1 = commands.spawn(...).id();
        // let child2 = commands.spawn(...).id();
        // commands.entity(parent).add_children(&[child1, child2]);
        // ```
        children![
            // Left ear - red square
            (
                Sprite::from_color(RED, Vec2::splat(20.0)),
                // Position relative to parent: left by half the gap
                Transform::from_xyz(-gap / 2.0, 0.0, 0.0),
            ),
            // Right ear - green square  
            (
                Sprite::from_color(LIME, Vec2::splat(20.0)),
                // Position relative to parent: right by half the gap
                Transform::from_xyz(gap / 2.0, 0.0, 0.0),
            )
        ],
    ));

    // Add instruction text to the screen
    commands.spawn((
        Text::new("Up/Down/Left/Right: Move Listener\nSpace: Toggle Emitter Movement"),
        Node {
            // Absolute positioning places the text relative to the window
            position_type: PositionType::Absolute,
            // Position in bottom-left corner with 12px padding
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    // Spawn a 2D camera so we can see the scene
    commands.spawn(Camera2d);
}

/// Component for the sound emitter entity.
/// 
/// Tracks elapsed time to create smooth sinusoidal movement.
///
/// ## Component Design Philosophy
///
/// **Data-Oriented Design**: Store only the data needed for behavior
/// ```rust
/// // Bad: Mixing behavior with data
/// struct Emitter {
///     fn update(&mut self) { /* logic here */ }
/// }
///
/// // Good: Data only, behavior in systems
/// struct Emitter {
///     stopwatch: Stopwatch,
/// }
/// ```
///
/// **Why Stopwatch over f32 elapsed?**
/// - Built-in pause/resume functionality
/// - Prevents floating point precision loss
/// - Clearer intent in code
/// - Consistent with Bevy patterns
#[derive(Component, Default)]
struct Emitter {
    /// Tracks time for movement animation.
    /// Using Stopwatch instead of raw time allows pausing.
    ///
    /// ## Stopwatch vs Timer
    ///
    /// **Stopwatch**: Counts up indefinitely
    /// ```rust
    /// stopwatch.elapsed_secs()  // 0.0, 0.5, 1.0, 1.5...
    /// ```
    ///
    /// **Timer**: Counts to target then resets/stops
    /// ```rust
    /// timer.fraction()  // 0.0, 0.5, 1.0, 0.0, 0.5...
    /// ```
    ///
    /// For continuous sine waves, Stopwatch is ideal
    stopwatch: Stopwatch,
}

/// Updates the position of sound emitters, creating horizontal oscillation.
/// 
/// This system demonstrates how moving sound sources affect spatial audio.
/// As the emitter moves left and right, you'll hear the sound pan between speakers.
fn update_emitters(
    time: Res<Time>,
    mut emitters: Query<(&mut Transform, &mut Emitter), With<Emitter>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Process each emitter in the scene (though we only have one in this example)
    for (mut emitter_transform, mut emitter) in emitters.iter_mut() {
        // Toggle movement when spacebar is pressed
        if keyboard.just_pressed(KeyCode::Space) {
            if emitter.stopwatch.is_paused() {
                emitter.stopwatch.unpause();
            } else {
                emitter.stopwatch.pause();
            }
        }

        // Advance the stopwatch by the time since last frame
        // This ensures consistent movement regardless of framerate
        emitter.stopwatch.tick(time.delta());

        // If not paused, move the emitter in a sinusoidal pattern
        if !emitter.stopwatch.is_paused() {
            // Sin function creates smooth back-and-forth motion:
            // - elapsed_secs() gives us time in seconds
            // - sin() returns values between -1 and 1
            // - Multiply by 500 to get movement range of -500 to +500 pixels
            //
            // ## Sinusoidal Motion Mathematics
            //
            // **Sine Wave Properties**:
            // ```text
            // x(t) = A × sin(ωt + φ)
            // where:
            //   A = amplitude (500 pixels)
            //   ω = angular frequency (1 rad/sec here)
            //   t = time (seconds)
            //   φ = phase offset (0 here)
            // ```
            //
            // **Motion Profile**:
            // ```text
            // Time  | sin(t) | Position | Direction
            // ------|--------|----------|----------
            // 0     | 0.0    | 0        | Right
            // π/2   | 1.0    | +500     | Turning
            // π     | 0.0    | 0        | Left
            // 3π/2  | -1.0   | -500     | Turning
            // 2π    | 0.0    | 0        | Right
            // ```
            //
            // **Audio Panning During Motion**:
            // - At x=-500: Sound fully in right speaker
            // - At x=0: Sound centered  
            // - At x=+500: Sound fully in left speaker
            //
            // **Doppler Effect** (not implemented):
            // ```rust
            // let velocity = ops::cos(elapsed) * 500.0;  // dx/dt
            // let doppler = 1.0 + (velocity.dot(to_listener) / 343.0);
            // ```
            emitter_transform.translation.x = ops::sin(emitter.stopwatch.elapsed_secs()) * 500.0;
            
            // The emitter completes one full cycle every 2π seconds (about 6.28 seconds)
            // You'll hear the sound pan from left speaker to right and back
            //
            // ## Customizing Motion Patterns
            //
            // **Different Patterns**:
            // ```rust
            // // Figure-8 motion
            // x = 500.0 * ops::sin(t);
            // y = 250.0 * ops::sin(2.0 * t);
            //
            // // Circular motion
            // x = 300.0 * ops::cos(t);
            // y = 300.0 * ops::sin(t);
            //
            // // Square wave (sharp panning)
            // x = 500.0 * ops::signum(ops::sin(t));
            // ```
        }
    }
}

/// Moves the spatial listener based on keyboard input.
/// 
/// This simulates moving the player (and their ears) through the game world.
/// Moving the listener has the opposite effect of moving the emitter:
/// - Moving listener left makes sounds seem to come from the right
/// - Moving listener right makes sounds seem to come from the left
///
/// ## Frame-Rate Independent Movement
///
/// **The Problem**: Different frame rates without delta time
/// ```text
/// At 30 FPS: 30 updates/sec × 5 pixels = 150 pixels/sec
/// At 60 FPS: 60 updates/sec × 5 pixels = 300 pixels/sec
/// At 144 FPS: 144 updates/sec × 5 pixels = 720 pixels/sec
/// ```
///
/// **The Solution**: Multiply by delta time
/// ```text
/// movement = speed * delta_time
/// At 30 FPS:  200 × 0.0333 = 6.67 pixels/frame
/// At 60 FPS:  200 × 0.0167 = 3.33 pixels/frame  
/// At 144 FPS: 200 × 0.0069 = 1.39 pixels/frame
/// All = 200 pixels/second!
/// ```
fn update_listener(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    // Single<T> is a query that expects exactly one entity
    // It's more efficient than Query when you know there's only one
    mut listener: Single<&mut Transform, With<SpatialListener>>,
) {
    // Movement speed in pixels per second
    let speed = 200.;

    // Move right: positive X direction
    if keyboard.pressed(KeyCode::ArrowRight) {
        // time.delta_secs() ensures frame-rate independent movement
        // At 60 FPS: delta = 0.0167s, movement = 3.33 pixels per frame
        // At 30 FPS: delta = 0.0333s, movement = 6.67 pixels per frame
        // Total movement per second is the same regardless of framerate
        listener.translation.x += speed * time.delta_secs();
    }
    
    // Move left: negative X direction
    if keyboard.pressed(KeyCode::ArrowLeft) {
        listener.translation.x -= speed * time.delta_secs();
    }
    
    // Move up: positive Y direction (in Bevy 2D, Y+ is up)
    if keyboard.pressed(KeyCode::ArrowUp) {
        listener.translation.y += speed * time.delta_secs();
    }
    
    // Move down: negative Y direction
    if keyboard.pressed(KeyCode::ArrowDown) {
        listener.translation.y -= speed * time.delta_secs();
    }
    
    // The spatial audio system automatically updates based on the new positions
    // As you move the listener:
    // - Closer to emitter = louder sound
    // - Further from emitter = quieter sound
    // - Left of emitter = sound in right speaker
    // - Right of emitter = sound in left speaker
    //
    // ## Spatial Audio Update Pipeline
    //
    // **Each Frame**:
    // 1. Transform systems update positions
    // 2. Audio system queries all (SpatialListener, Transform)
    // 3. Audio system queries all (AudioEmitter, Transform)  
    // 4. For each emitter-listener pair:
    //    - Calculate relative position
    //    - Apply distance attenuation
    //    - Calculate stereo panning
    //    - Update audio mixer
    //
    // **Coordinate Space Confusion**:
    // ```text
    // Listener at (100, 0), Emitter at (0, 0):
    // - Relative position: (-100, 0)
    // - Emitter is LEFT of listener
    // - But sound comes from LEFT
    // - So it plays in LEFT speaker (not right!)
    // ```
    //
    // This matches real life: sounds on your left 
    // come from the left speaker!
}
