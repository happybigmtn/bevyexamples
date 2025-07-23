//! # Audio Playback Control
//!
//! This example demonstrates how to load and play audio files in Bevy with interactive controls.
//! You'll learn how to manipulate audio playback including play/pause, volume, speed, and muting.
//!
//! ## Key Concepts
//!
//! - **AudioPlayer**: Component that plays audio assets
//! - **AudioSink**: Handle for controlling active audio playback
//! - **Audio Assets**: Loading and playing sound files (OGG, MP3, WAV, etc.)
//! - **Playback Control**: Real-time manipulation of audio properties
//!
//! ## Controls
//!
//! - **Space**: Play/Pause toggle
//! - **M**: Mute/Unmute toggle
//! - **-/=**: Volume down/up (10% increments)
//! - Speed automatically oscillates with time
//!
//! ## Audio Architecture
//!
//! 1. **AudioPlayer** starts playback when spawned with an audio asset
//! 2. **AudioSink** is automatically created and provides control interface
//! 3. Systems query for AudioSink to manipulate playback
//!
//! ## Audio Theory: The AudioSink Pattern
//!
//! **What is an Audio Sink?**
//! In audio programming, a "sink" is where audio data flows to. Think of it like a kitchen sink:
//! - Audio data (water) flows from the source (faucet)
//! - Through processing (pipes)
//! - Into the sink (speakers)
//!
//! The AudioSink provides controls like the faucet handles:
//! - **Volume**: How much audio flows through (0-100%)
//! - **Speed**: How fast the audio plays (pitch shifting)
//! - **Pause**: Stop the flow temporarily
//! - **Position**: Where we are in the audio stream
//!
//! **Digital Signal Processing (DSP) Chain**:
//! ```text
//! [Audio File] -> [Decoder] -> [Resampler] -> [Volume] -> [Mixer] -> [Output]
//!                               ↑               ↑           ↑
//!                             Speed          Mute      Other Sounds
//! ```
//!
//! ## Game Design Context: Interactive Audio
//!
//! **Why Control Audio?**
//! 1. **Adaptive Music**: Change tempo during intense moments
//! 2. **Environmental Effects**: Muffle sounds underwater
//! 3. **Player Preferences**: Accessibility and comfort
//! 4. **Dynamic Mixing**: Duck music during dialog
//!
//! **Common Game Audio Patterns**:
//! - **Action Intensity**: Speed up music during combat
//! - **Stealth Mode**: Lower volume when sneaking
//! - **Time Manipulation**: Slow audio with time dilation
//! - **Health Warning**: Muffle audio when near death
//!
//! ## Performance Optimization: Real-time Audio Control
//!
//! **CPU Considerations**:
//! - Volume changes: Nearly free (simple multiplication)
//! - Speed changes: Expensive (requires resampling)
//! - Pause/Resume: Free (stops processing)
//! - Position seeking: Very expensive (requires decode)
//!
//! **Best Practices**:
//! 1. **Batch Changes**: Update multiple properties at once
//! 2. **Smooth Transitions**: Interpolate volume over frames
//! 3. **Limit Speed Range**: Extreme speeds need more CPU
//! 4. **Cache Sinks**: Store references to avoid queries
//!
//! ## Real-World Applications
//!
//! **Music Systems in Popular Games**:
//! - **DOOM (2016)**: Dynamic music intensity based on combat
//! - **Celeste**: Music speed changes with game speed
//! - **Portal 2**: Muffled audio through portals
//! - **Red Dead Redemption 2**: Environmental audio ducking
//!
//! **Professional Audio Middleware**:
//! - FMOD: Industry standard for adaptive audio
//! - Wwise: Used in AAA games for complex mixing
//! - Both use similar sink/source concepts
//!
//! ## Advanced Techniques: Audio Effects
//!
//! **Effects You Could Add**:
//! 1. **Low-pass Filter**: Muffle sounds (underwater effect)
//! 2. **Reverb**: Add space and depth
//! 3. **Distortion**: Damage or radio effects
//! 4. **Echo/Delay**: Cave or mountain environments
//!
//! **Pitch Shifting vs Time Stretching**:
//! - **Pitch Shift**: Change frequency (chipmunk effect)
//! - **Time Stretch**: Change duration without pitch
//! - Current example does both (linked by speed)
//!
//! ## Common Issues and Solutions
//!
//! **Problem**: Audio pops when changing volume
//! - **Cause**: Instant volume changes create discontinuities
//! - **Solution**: Interpolate volume over ~10ms
//!
//! **Problem**: Speed changes sound robotic
//! - **Cause**: Simple resampling introduces artifacts
//! - **Solution**: Use higher quality resampling algorithms
//!
//! **Problem**: Multiple sinks for same sound
//! - **Cause**: Spawning AudioPlayer multiple times
//! - **Solution**: Check for existing sink before spawning

use bevy::{
    // For the sin function used in speed modulation
    math::ops,
    prelude::*,
};

// ## Rust Programming Fundamentals: Module Imports
//
// The `use` statement brings items into scope. Bevy organizes its API into modules:
// - `prelude`: Common types for everyday use (re-exports from other modules)
// - `math::ops`: Mathematical operations like sin, cos, tan
//
// **Why Separate Math Ops?**
// Rust's standard library doesn't include trigonometric functions in the prelude.
// They live in `std::f32` and `std::f64`. Bevy re-exports them for convenience.
//
// **Import Patterns**:
// - `use bevy::prelude::*;` - Glob import, brings all items
// - `use bevy::math::ops::sin;` - Specific import, only sin function
// - `use bevy::math::ops;` - Module import, access as ops::sin

fn main() {
    App::new()
        // DefaultPlugins includes AudioPlugin for audio playback support
        .add_plugins(DefaultPlugins)
        // Setup system runs once to create our audio player and UI
        .add_systems(Startup, setup)
        // Update systems run every frame to handle controls and UI updates
        .add_systems(
            Update,
            (
                update_progress_text, // Show playback position
                update_speed,         // Modulate playback speed
                pause,               // Handle play/pause
                mute,                // Handle mute toggle
                volume,              // Handle volume changes
            ),
        )
        .run();
}

// Setup system - creates the audio player and UI elements
//
// ## Bevy Architecture: System Ordering and Audio Initialization
//
// This setup system runs during the Startup schedule, which happens:
// 1. After all plugins are initialized
// 2. Before the first Update frame
// 3. Only once during the application lifetime
//
// **Audio System Initialization Timeline**:
// ```
// [App::new] -> [Add Plugins] -> [Audio Device Init] -> [Startup Systems] -> [Main Loop]
//                                       ↓
//                              - Query audio devices
//                              - Select default output
//                              - Create mixer thread
//                              - Setup audio graph
// ```
fn setup(
    mut commands: Commands,
    // AssetServer loads files from the assets/ directory
    //
    // ## Asset Loading Deep Dive
    //
    // The AssetServer uses a multi-threaded architecture:
    // 1. **Main Thread**: Receives load requests, returns handles
    // 2. **IO Thread Pool**: Reads files from disk
    // 3. **Decode Thread Pool**: Processes file formats
    // 4. **Asset Storage**: Holds loaded assets in memory
    //
    // **Audio-Specific Loading**:
    // - Small files (<1MB): Loaded entirely into RAM
    // - Large files (>1MB): Streamed in chunks
    // - Compressed formats: Decoded progressively
    asset_server: Res<AssetServer>,
) {
    // Spawn an entity with audio playback components
    //
    // ## Memory Layout and Component Storage
    //
    // When we spawn an entity with AudioPlayer, Bevy creates:
    // 1. **Entity ID**: 64-bit identifier (32-bit index + 32-bit generation)
    // 2. **Component Storage**: AudioPlayer goes into its dedicated storage
    // 3. **Archetype**: Entity's "type" based on component combination
    //
    // **Storage Layout**:
    // ```
    // Entity Table:
    // [Entity 0] -> Archetype: (AudioPlayer, MyMusic)
    // 
    // Component Storages:
    // AudioPlayer: [Handle<AudioSource>]
    // MyMusic:     [()] // Zero-sized type
    // ```
    commands.spawn((
        // AudioPlayer component starts playing the audio immediately when spawned
        // The asset_server.load() returns a Handle<AudioSource> that will be loaded asynchronously
        //
        // ## The Handle Pattern Explained
        //
        // A Handle<T> is Bevy's smart pointer for assets:
        // - **Strong Handle**: Keeps asset alive (reference counted)
        // - **Weak Handle**: Doesn't prevent unloading
        // - **Async Loading**: Returns immediately, loads in background
        //
        // **Handle Internals**:
        // ```rust
        // pub struct Handle<T> {
        //     id: AssetId<T>,      // Unique identifier
        //     handle_type: HandleType, // Strong or Weak
        // }
        // ```
        //
        // **Loading State Machine**:
        // NotLoaded -> Loading -> Loaded -> Failed (on error)
        //                |                     |
        //                └─────── Retry ───────┘
        AudioPlayer::new(asset_server.load("sounds/Windless Slopes.ogg")),
        // Custom marker component to identify this specific audio player
        //
        // ## Zero-Sized Types (ZST) in ECS
        //
        // MyMusic is a "tag component" - it has no data, only type information.
        // In memory, it takes 0 bytes! Rust and Bevy optimize these away.
        //
        // **Why Use ZSTs?**
        // - Type-safe entity identification
        // - No runtime memory cost
        // - Fast query filtering
        // - Clear semantic meaning
        MyMusic,
    ));

    // Create progress display text (top-left corner)
    commands.spawn((
        // Empty text that will be updated with playback progress
        Text::new(""),
        // Node component for UI positioning
        Node {
            // Absolute positioning removes element from normal layout flow
            position_type: PositionType::Absolute,
            // Position from top edge of screen
            top: Val::Px(12.0),
            // Position from left edge of screen
            left: Val::Px(12.0),
            // Use default values for other properties
            ..default()
        },
        // Marker component to identify progress text for updates
        ProgressText,
    ));

    // Create control instructions text (bottom-left corner)
    commands.spawn((
        // Multi-line text showing keyboard controls
        // \n creates line breaks in the displayed text
        Text::new("-/=: Volume Down/Up\nSpace: Toggle Playback\nM: Toggle Mute"),
        Node {
            position_type: PositionType::Absolute,
            // Position from bottom edge of screen
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    // Spawn a basic 3D camera
    // Even though this is an audio example, we need a camera for the UI to render
    commands.spawn(Camera3d::default());
}

// Marker component to identify our music player entity
// This allows us to query specifically for our music's AudioSink
//
// ## Component Design Patterns in ECS
//
// **Marker Components** (Zero-Sized Types):
// - Identify entities without storing data
// - Enable precise queries with With/Without filters
// - Group related entities semantically
//
// **Why Not Use Entity IDs?**
// - IDs change between runs (not serializable)
// - Hard to track in complex scenes
// - No type safety or semantic meaning
// - Can't use in query filters
//
// **Advanced Pattern - Newtype Components**:
// ```rust
// #[derive(Component)]
// struct BackgroundMusic(Handle<AudioSource>);
// 
// #[derive(Component)]
// struct EffectSound(Handle<AudioSource>);
// ```
// Same data, different types = different queries!
#[derive(Component)]
struct MyMusic;

// Marker component to identify the progress text UI element
// Used to update the correct text entity with playback time
//
// ## Rust Derive Macros Explained
//
// `#[derive(Component)]` generates implementation of the Component trait:
// ```rust
// impl Component for ProgressText {
//     type Storage = TableStorage; // or SparseStorage
// }
// ```
//
// **Storage Types**:
// - **TableStorage**: Dense, cache-friendly (default)
// - **SparseStorage**: For rare components
//
// **Other Common Derives**:
// - `Debug`: Printing for debugging
// - `Clone`: Duplicating components
// - `PartialEq`: Comparing components
// - `Default`: Default values
#[derive(Component)]
struct ProgressText;

// System to update the progress display with current playback time
//
// ## Audio Timing and Synchronization
//
// Audio time is different from game time:
// - **Game Time**: Can pause, slow down, speed up
// - **Audio Time**: Always real-time (hardware driven)
// - **Position Tracking**: Calculated from samples played
//
// **Sample-Accurate Timing**:
// Position = Samples Played / Sample Rate
// Example: 441,000 samples at 44.1kHz = 10 seconds
//
// **Drift and Correction**:
// Audio hardware clocks can drift from CPU clocks.
// Professional audio uses Word Clock to sync devices.
fn update_progress_text(
    // Single<T> is like Query<T> but expects exactly one entity
    // Will panic if there are 0 or 2+ entities matching
    //
    // ## Query System Parameters: Single vs Query
    //
    // **Single<T>**: Expects exactly one match
    // - Pros: Cleaner syntax, clear intent
    // - Cons: Panics on 0 or 2+ matches
    // - Use when: System requires exactly one entity
    //
    // **Query<T>**: Can handle any number of matches  
    // - Pros: Flexible, error handling
    // - Cons: More verbose with .single()
    // - Use when: Entity might not exist
    //
    // **Performance**: Identical - Single is syntax sugar
    music_controller: Single<&AudioSink, With<MyMusic>>,
    // Mutable access to the progress text
    mut progress_text: Single<&mut Text, With<ProgressText>>,
) {
    // Update text content with formatted playback position
    // AudioSink::position() returns a Duration since the start of playback
    // as_secs_f32() converts Duration to floating-point seconds
    //
    // ## Duration and Time Representation
    //
    // Rust's Duration type stores time as:
    // - seconds: u64 (up to 584 billion years)
    // - nanoseconds: u32 (0-999,999,999)
    //
    // **Conversion Methods**:
    // - as_secs(): Whole seconds only
    // - as_millis(): Total milliseconds
    // - as_secs_f32(): Seconds with decimals (lossy)
    // - as_secs_f64(): Seconds with decimals (precise)
    //
    // **Format Specifier**: {:.1} means:
    // - : - Start format specification
    // - .1 - One decimal place
    // - s - Suffix (not part of format)
    //
    // ## String Allocation and UI Performance
    //
    // This allocates a new String every frame!
    // For 60 FPS, that's 3,600 allocations per minute.
    //
    // **Optimization Strategies**:
    // 1. Update only when position changes significantly
    // 2. Pre-allocate and reuse String buffer  
    // 3. Use fixed-size text with sprintf-style formatting
    // 4. Update at lower frequency (10Hz is plenty)
    progress_text.0 = format!("Progress: {:.1}s", music_controller.position().as_secs_f32());
}

// System to modulate playback speed over time (creates a "warping" effect)
//
// ## Audio Resampling Theory
//
// Changing playback speed requires resampling the audio:
// - **Original**: 44.1kHz sample rate
// - **2x Speed**: Play at 88.2kHz (higher pitch)
// - **0.5x Speed**: Play at 22.05kHz (lower pitch)
//
// **Resampling Algorithms**:
// 1. **Nearest Neighbor**: Fast but aliasing
// 2. **Linear Interpolation**: Better, some artifacts
// 3. **Sinc Interpolation**: Best quality, expensive
//
// **Nyquist Frequency**: When slowing down, high frequencies
// may exceed Nyquist limit and cause aliasing. Low-pass
// filtering prevents this but adds CPU cost.
fn update_speed(
    // Query for the audio sink - using Query instead of Single for error handling
    //
    // ## Error Handling Patterns in Systems
    //
    // Bevy systems can't return Result, so we handle errors by:
    // 1. **Early Return**: Skip processing on error
    // 2. **Logging**: Record issues for debugging
    // 3. **Fallback**: Use default values
    // 4. **State Tracking**: Mark entities as errored
    //
    // The `let-else` pattern combines matching with early return:
    // ```rust
    // let Ok(value) = result else { return; };
    // ```
    music_controller: Query<&AudioSink, With<MyMusic>>,
    // Time resource provides elapsed time since app start
    time: Res<Time>,
) {
    // Try to get single sink, return early if it doesn't exist
    // This pattern is more robust than Single when entity might not exist
    let Ok(sink) = music_controller.single() else {
        return;
    };
    
    // Don't change speed while paused
    if sink.is_paused() {
        return;
    }

    // Calculate speed using sine wave for smooth oscillation
    // - time.elapsed_secs() / 5.0: One complete cycle every 5 seconds
    // - sin() returns value between -1 and 1
    // - + 1.0 shifts range to 0 to 2
    // - .max(0.1) ensures minimum speed of 0.1x (10%)
    //
    // ## Trigonometric Audio Effects
    //
    // **Sine Wave Properties**:
    // - Period: 2π radians (360 degrees)
    // - Frequency: 1/5 Hz (0.2 Hz) in this example
    // - Amplitude: 1.0 (speed varies ±100%)
    // - Phase: 0 (starts at sin(0) = 0)
    //
    // **Musical Applications**:
    // - **Vibrato**: Pitch modulation (few Hz)
    // - **Tremolo**: Volume modulation
    // - **Chorus**: Multiple detuned copies
    // - **Flanger**: Time-varying delay
    //
    // **Why max(0.1)?**
    // - Speed 0 would stop playback
    // - Negative speed would play backwards (not supported)
    // - Very low speeds need excessive buffer memory
    // - 0.1x is practical minimum (10x slower)
    //
    // ## CPU Impact of Speed Changes
    //
    // Resampling cost scales with speed:
    // - 1.0x: No resampling needed
    // - 0.5x: Process 2x samples
    // - 2.0x: Process 0.5x samples
    // - Variable: Constant reallocation
    //
    // **Optimization**: Quantize speed to fixed ratios
    // (0.5x, 0.75x, 1.0x, 1.5x, 2.0x) for better caching.
    sink.set_speed((ops::sin(time.elapsed_secs() / 5.0) + 1.0).max(0.1));
}

// System to handle play/pause toggling with spacebar
fn pause(
    // ButtonInput tracks keyboard state (pressed, just_pressed, just_released)
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_controller: Query<&AudioSink, With<MyMusic>>,
) {
    let Ok(sink) = music_controller.single() else {
        return;
    };

    // just_pressed() returns true only on the frame the key was pressed
    // This prevents toggle from firing every frame while key is held
    if keyboard_input.just_pressed(KeyCode::Space) {
        // toggle_playback() switches between play and pause states
        sink.toggle_playback();
    }
}

// System to handle mute toggling with M key
fn mute(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Need mutable access to change mute state
    mut music_controller: Query<&mut AudioSink, With<MyMusic>>,
) {
    // single_mut() returns Result<Mut<T>, QuerySingleError>
    let Ok(mut sink) = music_controller.single_mut() else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::KeyM) {
        // toggle_mute() switches between muted and unmuted states
        // When muted, volume is 0 but original volume is remembered
        sink.toggle_mute();
    }
}

// System to handle volume control with -/= keys
//
// ## Audio Loudness and Perception
//
// **Decibels (dB) and Human Hearing**:
// - Human hearing is logarithmic, not linear
// - 10 dB increase = perceived 2x louder
// - 3 dB increase = 2x power (1.41x amplitude)
//
// **Volume Scales**:
// - **Linear**: 0.0 to 1.0 (what AudioSink uses)
// - **Logarithmic**: -∞ to 0 dB (professional audio)
// - **Percentage**: 0% to 100% (user interfaces)
//
// **Equal Loudness Curves**: Human ears are more sensitive
// to mid frequencies (1-4 kHz). Games often EQ music to
// compensate for volume changes.
fn volume(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut music_controller: Query<&mut AudioSink, With<MyMusic>>,
) {
    let Ok(mut sink) = music_controller.single_mut() else {
        return;
    };

    // Volume up with = key (10% increase)
    if keyboard_input.just_pressed(KeyCode::Equal) {
        // Get current volume (0.0 to 1.0 range)
        let current_volume = sink.volume();
        // increase_by_percentage is a custom trait method
        // Positive percentage increases volume
        //
        // ## Float Arithmetic and Audio Precision
        //
        // **Percentage Calculation**:
        // new_volume = current * (1.0 + percentage/100)
        // Example: 0.5 * (1.0 + 10/100) = 0.5 * 1.1 = 0.55
        //
        // **Floating Point Considerations**:
        // - f32 has ~7 decimal digits of precision
        // - Audio uses f32 internally (24-bit audio precision)
        // - Volume 0.0000001 is -140 dB (below noise floor)
        // - Clamped to [0.0, 1.0] to prevent clipping
        //
        // **Better Volume Control**:
        // Linear percentage changes don't feel natural.
        // Professional audio uses logarithmic faders:
        // ```rust
        // // Convert linear to dB
        // let db = 20.0 * (linear.max(0.0001)).log10();
        // // Adjust in dB space
        // let new_db = db + 1.0; // +1 dB
        // // Convert back to linear
        // let new_linear = 10.0f32.powf(new_db / 20.0);
        // ```
        sink.set_volume(current_volume.increase_by_percentage(10.0));
    } 
    // Volume down with - key (10% decrease)
    else if keyboard_input.just_pressed(KeyCode::Minus) {
        let current_volume = sink.volume();
        // Negative percentage decreases volume
        //
        // ## Volume Ramping and Click Prevention
        //
        // Instant volume changes can cause audible clicks.
        // This happens because the waveform has a discontinuity.
        //
        // **Click-Free Techniques**:
        // 1. **Zero-Crossing**: Change at waveform zero
        // 2. **Ramping**: Gradual change over ~10ms
        // 3. **Envelope**: Apply fade curve
        //
        // **Implementation Example**:
        // ```rust
        // // Smooth volume over 64 samples (~1.5ms at 44.1kHz)
        // let ramp_samples = 64;
        // let volume_step = (target - current) / ramp_samples;
        // for i in 0..ramp_samples {
        //     sample *= current + (volume_step * i);
        // }
        // ```
        sink.set_volume(current_volume.increase_by_percentage(-10.0));
    }
}
