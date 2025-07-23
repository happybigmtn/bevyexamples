//! This example illustrates how to play a single-frequency sound (aka a pitch)
//! 
//! ## Key Concepts
//! 
//! - **Musical Pitch**: The perceived frequency of a sound. Higher frequencies sound
//!   higher in pitch. Musical notes are specific frequencies that sound pleasant together.
//! 
//! - **Octaves and Semitones**: In Western music, an octave is a doubling of frequency.
//!   Each octave is divided into 12 semitones (the keys on a piano). Moving up one
//!   semitone multiplies the frequency by the 12th root of 2 (approximately 1.0595).
//! 
//! - **Pitch vs AudioSource**: While AudioSource plays pre-recorded files, Pitch
//!   generates pure sine waves at runtime. This is useful for musical applications,
//!   sound effects, and testing.
//! 
//! ## Controls
//! 
//! - **Space**: Play the current pitch
//! - **Up Arrow**: Increase pitch by one semitone
//! - **Down Arrow**: Decrease pitch by one semitone
//!
//! ## Audio Theory: The Physics of Pitch
//!
//! **Sound Wave Fundamentals**:
//! - **Frequency**: Cycles per second (Hz). Middle C = 261.63 Hz
//! - **Period**: Time for one cycle = 1/frequency
//! - **Wavelength**: Distance traveled per cycle = speed of sound / frequency
//! - **Amplitude**: Maximum displacement, determines loudness
//!
//! **Pitch Perception**:
//! - **Audible Range**: 20 Hz to 20,000 Hz (varies with age)
//! - **Musical Range**: ~27.5 Hz (A0) to ~4186 Hz (C8)
//! - **Just Noticeable Difference**: ~1% frequency change
//! - **Pitch vs Frequency**: Logarithmic perception
//!
//! **Harmonic Series**:
//! ```text
//! Fundamental (f): 100 Hz
//! 2nd Harmonic:    200 Hz (octave)
//! 3rd Harmonic:    300 Hz (perfect fifth + octave)
//! 4th Harmonic:    400 Hz (two octaves)
//! 5th Harmonic:    500 Hz (major third + two octaves)
//! ```
//!
//! ## Game Design Context: Musical Sound Effects
//!
//! **Procedural Audio in Games**:
//! - **UI Feedback**: Rising pitch for success, falling for failure
//! - **Collectibles**: Musical notes when collecting items
//! - **Movement**: Footstep pitch based on material/speed
//! - **Proximity**: Pitch changes as player approaches objects
//!
//! **Musical Game Examples**:
//! - **Guitar Hero**: Pitch detection for vocals
//! - **Zelda**: Musical puzzles with specific note sequences
//! - **Journey**: Dynamic musical communication between players
//! - **Sound Shapes**: Gameplay creates musical compositions
//!
//! ## Performance Optimization: Efficient Pitch Generation
//!
//! **Sine Wave Generation Cost**:
//! ```text
//! Method              | CPU Cycles | Quality
//! --------------------|------------|--------
//! Math::sin()         | 20-50      | Perfect
//! Taylor Series (5th) | 15-30      | Very Good
//! Lookup Table        | 5-10       | Good
//! Linear Interp LUT   | 8-15       | Excellent
//! ```
//!
//! **Memory vs Computation Tradeoff**:
//! - Small lookup table (256 entries): 1KB memory, good quality
//! - Large lookup table (4096 entries): 16KB memory, excellent quality
//! - No lookup table: 0KB memory, highest CPU usage
//!
//! ## Real-World Applications
//!
//! **Musical Tuning Systems**:
//! 1. **Equal Temperament**: 12th root of 2 (modern standard)
//! 2. **Just Intonation**: Simple ratios (3:2, 4:3, 5:4)
//! 3. **Pythagorean**: Based on perfect fifths
//! 4. **Microtonal**: More than 12 notes per octave
//!
//! **Audio Test Equipment**:
//! - **Sine Sweep**: Test speaker frequency response
//! - **THD Testing**: Measure harmonic distortion
//! - **Room Acoustics**: Find resonant frequencies
//! - **Hearing Tests**: Audiometry uses pure tones
//!
//! ## Advanced Techniques: Beyond Simple Pitch
//!
//! **Enhancements You Could Add**:
//! 1. **Waveform Selection**: Square, sawtooth, triangle
//! 2. **Envelope Control**: ADSR for more natural sounds
//! 3. **Vibrato/Tremolo**: Pitch/amplitude modulation
//! 4. **Polyphony**: Multiple simultaneous pitches
//!
//! **Synthesis Methods**:
//! - **Additive**: Sum multiple sine waves
//! - **FM Synthesis**: Frequency modulation for complex timbres
//! - **Wavetable**: Morph between different waveforms
//! - **Physical Modeling**: Simulate real instruments
//!
//! ## Common Issues and Solutions
//!
//! **Problem**: Clicks when starting/stopping
//! - **Cause**: Sudden amplitude changes
//! - **Solution**: Fade in/out over ~10ms
//!
//! **Problem**: Aliasing at high frequencies
//! - **Cause**: Frequencies above Nyquist limit
//! - **Solution**: Band-limit or increase sample rate
//!
//! **Problem**: Pitch drift over time
//! - **Cause**: Floating-point precision errors
//! - **Solution**: Recalculate phase periodically

use bevy::{
    // Import math operations for the frequency calculations
    math::ops,
    prelude::*,
};
// Duration is used to specify how long the pitch should play
use std::time::Duration;

// ## Rust Programming Fundamentals: Mathematical Functions
//
// **bevy::math::ops** provides:
// - Trigonometric: sin, cos, tan, asin, acos, atan
// - Exponential: exp, ln, log2, log10, powf
// - Rounding: floor, ceil, round, trunc
// - Other: abs, sqrt, cbrt, hypot
//
// **Why not std::f32?**
// Bevy re-exports for consistency and potential optimization

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Register our custom event that triggers pitch playback.
        // Events in Bevy are a way to communicate between systems without
        // direct coupling. Any system can send events, any system can receive them.
        .add_event::<PlayPitch>()
        // Initialize our frequency resource
        .add_systems(Startup, setup)
        // These systems run every frame:
        // - keyboard_input_system: checks for key presses and sends events
        // - play_pitch: responds to events by playing sounds
        .add_systems(Update, (play_pitch, keyboard_input_system))
        .run();
}

/// An event that signals we should play a pitch.
/// 
/// Events in Bevy are temporary - they only exist for one or two frames.
/// This ensures that actions happen exactly once when triggered.
/// 
/// The `Default` derive allows us to create events with `PlayPitch::default()`
/// or just `PlayPitch` when the type is known.
///
/// ## Event System Architecture
///
/// **Event Lifecycle**:
/// 1. **Frame N**: Event sent via EventWriter
/// 2. **Frame N**: Systems with EventReader see event
/// 3. **Frame N+1**: Late systems can still read event
/// 4. **Frame N+2**: Event is cleared from memory
///
/// **Double Buffering**:
/// Bevy uses two buffers to ensure all systems see events:
/// ```text
/// Frame N:   [Write Buffer] <- EventWriter
///            [Read Buffer]  -> EventReader
/// Frame N+1: Buffers swap roles
/// ```
///
/// **Why Events Instead of Direct Calls?**
/// - Decoupling: Sender doesn't know about receivers
/// - Parallelism: Systems can run simultaneously
/// - Flexibility: Multiple listeners, conditional handling
/// - Debugging: Can log/inspect event flow
#[derive(Event, Default)]
struct PlayPitch;

/// A resource storing the current frequency to play.
/// 
/// Resources in Bevy are global data accessible from any system.
/// We wrap the f32 in a newtype struct for type safety and clarity.
/// 
/// The frequency is measured in Hertz (Hz) - cycles per second.
///
/// ## Newtype Pattern in Rust
///
/// **Why Wrap f32?**
/// ```rust
/// // Without newtype - unclear what the number means
/// fn play_sound(volume: f32, frequency: f32, duration: f32) {}
/// play_sound(440.0, 0.5, 1.0); // Oops! Wrong order
///
/// // With newtypes - compiler catches errors
/// fn play_sound(volume: Volume, frequency: Frequency, duration: Duration) {}
/// play_sound(Frequency(440.0), Volume(0.5), Duration(1.0)); // Clear!
/// ```
///
/// **Zero-Cost Abstraction**:
/// - No runtime overhead (optimized away)
/// - Type safety at compile time
/// - Better documentation
/// - Can add methods via impl blocks
///
/// ## Resource Storage in ECS
///
/// **How Resources Work**:
/// - Stored in World, outside entity/component tables
/// - Accessed via type ID (one instance per type)
/// - Thread-safe with interior mutability
/// - Persistent across frames
///
/// **Resource vs Component**:
/// - Resource: Global state (settings, scores)
/// - Component: Per-entity data (position, health)
#[derive(Resource)]
struct PitchFrequency(f32);

fn setup(mut commands: Commands) {
    // Insert our frequency resource with an initial value of 220 Hz.
    // This is the musical note A3, one octave below A4 (440 Hz).
    // A3 is a comfortable starting pitch - not too high, not too low.
    //
    // ## Musical Note Frequencies
    //
    // **A-based Octaves** (International Standard):
    // - A0: 27.5 Hz (lowest piano key)
    // - A1: 55 Hz
    // - A2: 110 Hz
    // - A3: 220 Hz (our starting pitch)
    // - A4: 440 Hz (concert pitch)
    // - A5: 880 Hz
    // - A6: 1760 Hz
    // - A7: 3520 Hz
    // - A8: 7040 Hz (highest piano key is C8)
    //
    // **Why Start at A3?**
    // - Well within human vocal range
    // - Clear on most speakers/headphones
    // - Room to go up or down
    // - Not fatiguing to hear repeatedly
    commands.insert_resource(PitchFrequency(220.0));
}

/// System that plays a pitch when it receives a PlayPitch event.
/// 
/// This demonstrates Bevy's event-driven architecture: instead of checking
/// conditions every frame, we respond to events when they occur.
///
/// ## Event-Driven vs Polling
///
/// **Polling Approach** (inefficient):
/// ```rust
/// fn update(keys: Res<ButtonInput<KeyCode>>) {
///     if keys.pressed(KeyCode::Space) { // Checks every frame!
///         play_sound();
///     }
/// }
/// ```
///
/// **Event-Driven** (efficient):
/// ```rust
/// fn update(mut events: EventReader<PlaySound>) {
///     for event in events.read() { // Only runs when events exist
///         play_sound();
///     }
/// }
/// ```
///
/// **Benefits**:
/// - No wasted checks when nothing happens
/// - Clear cause-and-effect relationships
/// - Easy to trace execution flow
/// - Natural batching of similar operations
fn play_pitch(
    // Direct access to the Pitch asset storage.
    // Unlike loading audio files, we create Pitch assets programmatically.
    //
    // ## Assets<T> for Dynamic Content
    //
    // **Dynamic Asset Creation**:
    // - File-based: AssetServer loads from disk
    // - Procedural: Assets<T> creates at runtime
    // - Hybrid: Load base asset, modify procedurally
    //
    // **Handle Management**:
    // ```rust
    // let handle = assets.add(data);  // Strong handle created
    // // Asset lives as long as handle exists
    // drop(handle);  // Asset may be freed
    // ```
    mut pitch_assets: ResMut<Assets<Pitch>>,
    // Read the current frequency setting
    frequency: Res<PitchFrequency>,
    // EventReader gives us all PlayPitch events sent since last frame
    //
    // ## EventReader Details
    //
    // **Cursor Tracking**:
    // Each EventReader maintains its own position:
    // ```rust
    // System A reads events 0-5
    // System B reads events 0-5 (independent cursor)
    // System C reads events 3-5 (started reading late)
    // ```
    //
    // **Iteration Methods**:
    // - `.read()`: Iterator over all unread events
    // - `.clear()`: Mark all as read without processing
    // - `.len()`: Count of unread events
    // - `.is_empty()`: Check if any unread events
    mut events: EventReader<PlayPitch>,
    // For spawning audio entities
    mut commands: Commands,
) {
    // Process each PlayPitch event. Usually there's only one per frame,
    // but the loop handles multiple events correctly.
    //
    // ## Iterator Pattern in Events
    //
    // **Why for _ instead of for event?**
    // Our event type is empty (no data), so we ignore the value.
    // If events carried data:
    // ```rust
    // for event in events.read() {
    //     info!("Play at volume: {}", event.volume);
    // }
    // ```
    for _ in events.read() {
        // Log the frequency we're about to play (visible with RUST_LOG=info)
        //
        // ## Logging in Bevy
        //
        // **Log Levels** (set via RUST_LOG environment variable):
        // - `error!`: Critical problems
        // - `warn!`: Potential issues
        // - `info!`: Important events
        // - `debug!`: Detailed information
        // - `trace!`: Very detailed debugging
        //
        // **Usage**: `RUST_LOG=info cargo run --example pitch`
        info!("playing pitch with frequency: {}", frequency.0);
        
        // Spawn an entity with audio components
        commands.spawn((
            // Create a Pitch asset and wrap it in an AudioPlayer.
            // Pitch::new takes frequency in Hz and duration.
            // Duration::new(1, 0) means 1 second, 0 nanoseconds.
            //
            // ## Duration Construction
            //
            // **Duration::new(secs, nanos)**:
            // - First param: Whole seconds (u64)
            // - Second param: Additional nanoseconds (u32)
            // - Total time = secs + (nanos / 1_000_000_000)
            //
            // **Examples**:
            // ```rust
            // Duration::new(1, 0)           // 1 second
            // Duration::new(0, 500_000_000) // 0.5 seconds
            // Duration::from_millis(1500)   // 1.5 seconds
            // Duration::from_secs_f32(2.5)  // 2.5 seconds
            // ```
            //
            // ## Pitch Generation Pipeline
            //
            // **What Happens Inside**:
            // 1. Pitch::new creates parameters struct
            // 2. Assets.add stores it, returns Handle
            // 3. AudioPlayer component holds Handle
            // 4. Audio system detects new AudioPlayer
            // 5. Creates sine wave generator
            // 6. Mixes into audio output stream
            AudioPlayer(pitch_assets.add(Pitch::new(frequency.0, Duration::new(1, 0)))),
            
            // PlaybackSettings::DESPAWN tells Bevy to remove this entity
            // when the audio finishes playing. This prevents memory leaks
            // from accumulating finished audio entities.
            //
            // ## Audio Entity Lifecycle
            //
            // **Without DESPAWN**:
            // ```text
            // Frame 0: Entity spawned with AudioPlayer
            // Frame 1-60: Audio plays
            // Frame 61: Audio finishes, entity remains
            // Frame 62+: Dead entity wastes memory
            // ```
            //
            // **With DESPAWN**:
            // ```text
            // Frame 0: Entity spawned with AudioPlayer
            // Frame 1-60: Audio plays
            // Frame 61: Audio finishes, entity removed
            // ```
            //
            // **Other PlaybackSettings Options**:
            // - `LOOP`: Repeat forever
            // - `ONCE`: Play once (default)
            // - `REMOVE`: Remove component but keep entity
            // - Custom: Set volume, speed, loop count
            PlaybackSettings::DESPAWN,
        ));
        
        // Log how many pitch assets exist (useful for debugging memory leaks)
        //
        // ## Asset Memory Management
        //
        // **Reference Counting**:
        // - Each Handle increments reference count
        // - When count reaches 0, asset is freed
        // - DESPAWN entities drop their Handles
        //
        // **Memory Leak Indicators**:
        // - Asset count growing without bound
        // - Memory usage increasing over time
        // - Performance degrading gradually
        info!("number of pitch assets: {}", pitch_assets.len());
    }
}

/// System that handles keyboard input to control pitch and trigger playback.
/// 
/// This system demonstrates:
/// - Reading keyboard input
/// - Modifying resources
/// - Sending events to other systems
///
/// ## Input Handling in Game Engines
///
/// **Frame-Based Input States**:
/// ```text
/// Frame 1: Key Down Event from OS
/// Frame 1: just_pressed() = true, pressed() = true
/// Frame 2: pressed() = true, just_pressed() = false
/// Frame 3: pressed() = true
/// Frame 4: Key Up Event from OS
/// Frame 4: just_released() = true, pressed() = false
/// ```
///
/// **Why Track "Just" States?**
/// - Prevent repeated actions from held keys
/// - Distinguish taps from holds
/// - Enable fighting game combos
/// - Implement UI navigation
fn keyboard_input_system(
    // ButtonInput tracks the state of keyboard keys.
    // It knows which keys are pressed, just pressed, or just released.
    //
    // ## ButtonInput<T> Generic
    //
    // **Can Track Multiple Input Types**:
    // - `ButtonInput<KeyCode>`: Keyboard keys
    // - `ButtonInput<MouseButton>`: Mouse buttons
    // - `ButtonInput<GamepadButton>`: Controller buttons
    //
    // **Internal Implementation**:
    // ```rust
    // struct ButtonInput<T> {
    //     pressed: HashSet<T>,
    //     just_pressed: HashSet<T>,
    //     just_released: HashSet<T>,
    // }
    // ```
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // We need mutable access to change the frequency
    mut frequency: ResMut<PitchFrequency>,
    // EventWriter lets us send events that other systems can respond to
    //
    // ## EventWriter vs EventReader
    //
    // **Symmetrical API**:
    // - Writer: `events.write(MyEvent)`
    // - Reader: `for event in events.read()`
    //
    // **Multiple Writers/Readers Allowed**:
    // - Many systems can send same event type
    // - Many systems can listen for same events
    // - Order determined by system scheduling
    mut events: EventWriter<PlayPitch>,
) {
    // Increase pitch by one semitone (musical half-step)
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        // The math behind this: In equal temperament tuning, each semitone
        // increases frequency by the 12th root of 2 (about 1.0595).
        // This ensures 12 semitones equals exactly one octave (2x frequency).
        //
        // ## Equal Temperament Mathematics
        //
        // **The Problem**: How to divide an octave into 12 equal parts?
        // - Octave = 2× frequency
        // - Need 12 equal multiplicative steps
        // - Each step multiplies by same ratio
        //
        // **The Solution**:
        // ```
        // ratio^12 = 2
        // ratio = 2^(1/12)
        // ratio ≈ 1.059463094359...
        // ```
        //
        // **Frequency Table** (starting from A4 = 440 Hz):
        // ```
        // Note | Semitones | Calculation        | Frequency
        // -----|-----------|--------------------|---------
        // A4   | 0         | 440 × 2^(0/12)     | 440.00
        // A#4  | 1         | 440 × 2^(1/12)     | 466.16
        // B4   | 2         | 440 × 2^(2/12)     | 493.88
        // C5   | 3         | 440 × 2^(3/12)     | 523.25
        // C#5  | 4         | 440 × 2^(4/12)     | 554.37
        // D5   | 5         | 440 × 2^(5/12)     | 587.33
        // D#5  | 6         | 440 × 2^(6/12)     | 622.25
        // E5   | 7         | 440 × 2^(7/12)     | 659.25
        // F5   | 8         | 440 × 2^(8/12)     | 698.46
        // F#5  | 9         | 440 × 2^(9/12)     | 739.99
        // G5   | 10        | 440 × 2^(10/12)    | 783.99
        // G#5  | 11        | 440 × 2^(11/12)    | 830.61
        // A5   | 12        | 440 × 2^(12/12)    | 880.00
        // ```
        //
        // **Type Annotation**: `2.0f32` forces f32 type
        // Without suffix, Rust might infer f64
        frequency.0 *= ops::powf(2.0f32, 1.0 / 12.0);
    }
    
    // Decrease pitch by one semitone
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        // Dividing by the 12th root of 2 moves down one semitone.
        // This is the inverse of the operation above.
        frequency.0 /= ops::powf(2.0f32, 1.0 / 12.0);
    }
    
    // Play the current pitch
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Send a PlayPitch event. The play_pitch system will receive this
        // and actually play the sound. This decoupling means we could have
        // multiple systems that trigger pitches or multiple systems that
        // respond to the event.
        //
        // ## Event System Benefits
        //
        // **Decoupling Example**:
        // ```rust
        // // Multiple triggers for same event
        // fn keyboard_system(mut events: EventWriter<PlaySound>) {
        //     if space_pressed { events.write(PlaySound); }
        // }
        // 
        // fn gamepad_system(mut events: EventWriter<PlaySound>) {
        //     if button_pressed { events.write(PlaySound); }
        // }
        // 
        // fn ai_system(mut events: EventWriter<PlaySound>) {
        //     if should_taunt { events.write(PlaySound); }
        // }
        // 
        // // Single handler for all triggers
        // fn audio_system(mut events: EventReader<PlaySound>) {
        //     for _ in events.read() { /* play sound */ }
        // }
        // ```
        //
        // **System Communication Patterns**:
        // 1. **Direct**: Modify shared resource
        // 2. **Events**: Fire-and-forget messages
        // 3. **State**: Set component flags
        // 4. **Channels**: For complex async work
        //
        // **Event Queue Implementation**:
        // - Lock-free multi-producer queue
        // - Systems can write in parallel
        // - Events ordered by write time
        // - Double-buffered for reader consistency
        events.write(PlayPitch);
    }
}
