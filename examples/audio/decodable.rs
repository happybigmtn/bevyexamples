//! Shows how to create a custom [`Decodable`] type by implementing a Sine wave.
//! 
//! ## Key Concepts
//! 
//! - **Digital Audio Fundamentals**: Audio is represented as a series of samples. Each sample
//!   is a number representing the air pressure at a specific moment in time. Playing audio
//!   means sending these samples to the speaker at a specific rate (sample rate).
//! 
//! - **Sample Rate**: How many samples per second. Common rates:
//!   - 44,100 Hz (CD quality) - 44,100 pressure readings per second
//!   - 48,000 Hz (professional audio)
//!   - 96,000 Hz (high-resolution audio)
//! 
//! - **Sine Wave**: The purest form of sound - a single frequency with no harmonics.
//!   It sounds like a clean whistle or tuning fork.
//! 
//! - **Custom Audio Sources**: Instead of loading audio from files, Bevy allows you to
//!   generate audio programmatically by implementing the `Decodable` trait.
//!
//! ## Audio Theory: Synthesis and Waveforms
//!
//! **Fourier's Theorem**: Any periodic sound can be decomposed into sine waves:
//! - **Fundamental**: The base frequency (determines pitch)
//! - **Harmonics**: Integer multiples of fundamental (determine timbre)
//! - **Amplitude**: Loudness of each frequency component
//! - **Phase**: Time shift of each component
//!
//! **Common Waveforms and Their Harmonics**:
//! ```text
//! Sine:     Fundamental only (pure tone)
//! Square:   Odd harmonics (1, 3, 5, 7...)
//! Sawtooth: All harmonics (1, 2, 3, 4...)
//! Triangle: Odd harmonics, decreasing faster than square
//! ```
//!
//! **Why Sine Waves Matter**:
//! 1. **Building Block**: All sounds are combinations of sines
//! 2. **Test Signal**: Perfect for testing audio systems
//! 3. **Efficiency**: Simplest waveform to generate
//! 4. **No Aliasing**: Contains single frequency
//!
//! ## Game Design Context: Procedural Audio
//!
//! **When to Use Procedural Audio**:
//! - **UI Feedback**: Beeps, clicks, notifications
//! - **Retro Games**: 8-bit style sound effects
//! - **Dynamic Music**: Adaptive, generative soundscapes
//! - **Memory Savings**: Generate instead of store
//!
//! **Real Game Examples**:
//! - **No Man's Sky**: Procedural creature vocalizations
//! - **Spore**: Dynamic creature sounds based on size/shape
//! - **Mini Metro**: Generative ambient music
//! - **Proteus**: Environmental audio synthesis
//!
//! ## Performance Optimization: Real-time Synthesis
//!
//! **CPU Considerations**:
//! - Sine calculation: ~10-20 CPU cycles per sample
//! - At 44.1kHz: ~882,000 calculations per second
//! - Optimization: Use lookup tables for complex waveforms
//!
//! **Memory Access Patterns**:
//! ```rust
//! // Bad: Random access, cache misses
//! samples[random_index] = value;
//! 
//! // Good: Sequential access, cache friendly
//! for i in 0..samples.len() {
//!     samples[i] = generate_sample(i);
//! }
//! ```
//!
//! ## Real-World Applications
//!
//! **Synthesis Techniques Used in Games**:
//! 1. **Additive**: Combine sine waves (organs, bells)
//! 2. **Subtractive**: Filter harmonics from rich waveforms
//! 3. **FM Synthesis**: Modulate frequency (Yamaha DX7 sounds)
//! 4. **Granular**: Combine tiny audio grains (textures)
//!
//! **Industry Tools**:
//! - **Pure Data**: Visual programming for audio
//! - **SuperCollider**: Code-based synthesis
//! - **ChucK**: Time-based audio programming
//! - **Web Audio API**: Browser-based synthesis
//!
//! ## Advanced Techniques: Beyond Sine Waves
//!
//! **Enhancements You Could Add**:
//! 1. **ADSR Envelope**: Attack, Decay, Sustain, Release
//! 2. **LFO Modulation**: Low-frequency oscillator for vibrato
//! 3. **Filters**: Low-pass, high-pass, band-pass
//! 4. **Effects**: Reverb, delay, distortion
//!
//! **Optimization Techniques**:
//! - **SIMD**: Process 4-8 samples simultaneously
//! - **Lookup Tables**: Pre-calculate expensive functions
//! - **Band-Limited**: Prevent aliasing with proper filtering
//!
//! ## Common Issues and Solutions
//!
//! **Problem**: Clicking/popping sounds
//! - **Cause**: Discontinuities in waveform
//! - **Solution**: Fade in/out, ensure phase continuity
//!
//! **Problem**: Aliasing (unwanted frequencies)
//! - **Cause**: Frequencies above Nyquist limit
//! - **Solution**: Band-limit waveforms before sampling
//!
//! **Problem**: CPU spikes during synthesis
//! - **Cause**: Complex calculations in audio thread
//! - **Solution**: Pre-calculate, use approximations

use bevy::{
    // Audio-specific imports for creating custom audio sources
    audio::{AddAudioSource, AudioPlugin, Source, Volume},
    // Mathematical operations - we need sin() to generate sine waves
    math::ops,
    // Standard Bevy imports
    prelude::*,
    // TypePath is needed for the Asset derive macro to work properly
    reflect::TypePath,
};
// Duration type from the standard library for representing time spans
use core::time::Duration;

// ## Rust Programming Fundamentals: Import Organization
//
// **Module Structure in Bevy**:
// - `bevy`: Root module containing all engine functionality
// - `bevy::audio`: Audio subsystem with playback and processing
// - `bevy::math`: Mathematical utilities and types
// - `bevy::prelude`: Common types for convenience
// - `bevy::reflect`: Runtime type information system
//
// **Why core::time instead of std::time?**
// - `core`: Available in no_std environments (embedded, WASM)
// - `std`: Requires operating system (includes core + OS features)
// - Bevy uses core for maximum compatibility

/// Represents the parameters for our sine wave audio source.
/// 
/// In a real audio file, this struct would contain the compressed or uncompressed
/// audio data. For our sine wave generator, we only need to store the frequency.
/// 
/// The `Asset` derive macro allows this type to be managed by Bevy's asset system,
/// meaning it can be loaded, tracked, and hot-reloaded like any other asset.
/// 
/// The `TypePath` derive provides type information needed by the asset system
/// for serialization and debugging purposes.
///
/// ## Asset System Architecture
///
/// **What #[derive(Asset)] generates**:
/// ```rust
/// impl Asset for SineAudio {}
/// impl VisitAssetDependencies for SineAudio {
///     fn visit_dependencies(&self, visit: &mut impl FnMut(UntypedAssetId)) {}
/// }
/// ```
///
/// **Asset Storage Layout**:
/// - Assets stored in `Assets<T>` resource (type-erased storage)
/// - Each asset gets unique AssetId (UUID in debug, u64 in release)
/// - Reference counted for automatic cleanup
/// - Thread-safe access via RwLock
///
/// **TypePath Requirements**:
/// - Provides fully qualified type name at runtime
/// - Enables reflection and serialization
/// - Required for asset type registration
#[derive(Asset, TypePath)]
struct SineAudio {
    /// The frequency of the sine wave in Hertz (cycles per second).
    /// Common musical note frequencies:
    /// - A4 (middle A): 440 Hz
    /// - C4 (middle C): 261.63 Hz
    /// - A3: 220 Hz
    /// - A5: 880 Hz
    ///
    /// ## Musical Mathematics: Equal Temperament
    ///
    /// **Frequency Relationships**:
    /// - Octave up: frequency × 2
    /// - Octave down: frequency ÷ 2
    /// - Semitone up: frequency × 2^(1/12) ≈ 1.0595
    /// - Perfect fifth: frequency × 3/2 = 1.5
    ///
    /// **Note Frequency Formula**:
    /// f(n) = 440 × 2^((n-49)/12)
    /// where n = piano key number (A4 = 49)
    ///
    /// **Why 440 Hz?**
    /// - International standard since 1955
    /// - Some orchestras use 442 Hz (brighter)
    /// - Baroque music used 415 Hz (warmer)
    frequency: f32,
}

/// The decoder is responsible for generating audio samples on demand.
/// 
/// Think of this as a "player" for our SineAudio asset. While SineAudio
/// contains the parameters (what to play), SineDecoder contains the state
/// needed to actually generate the audio samples (how to play it).
///
/// ## State Machine Design Pattern
///
/// **Separation of Concerns**:
/// - **Asset (SineAudio)**: Immutable parameters, shareable
/// - **Decoder (SineDecoder)**: Mutable playback state, unique per playback
///
/// **Why This Pattern?**
/// 1. Multiple simultaneous playbacks of same asset
/// 2. Asset can be reused without state contamination
/// 3. Decoder can be optimized for streaming
/// 4. Clean separation of data and behavior
///
/// ## Phase Accumulator Synthesis
///
/// This decoder uses a "phase accumulator" - the most common
/// technique in digital synthesis:
///
/// ```text
/// Phase += Increment
/// if (Phase >= 1.0) Phase -= 1.0
/// Output = Waveform(Phase)
/// ```
///
/// **Advantages**:
/// - No cumulative rounding errors
/// - Easy frequency modulation
/// - Works for any waveform
/// - Constant time complexity
struct SineDecoder {
    /// How far along one period of the sine wave we are.
    /// This value ranges from 0.0 to 1.0, where:
    /// - 0.0 = start of the wave
    /// - 0.5 = halfway through (crossing zero, going negative)
    /// - 1.0 = complete cycle (back to start)
    ///
    /// ## Numerical Precision Considerations
    ///
    /// Using normalized phase (0-1) instead of radians (0-2π):
    /// - Better floating-point precision distribution
    /// - Easier wrapping with modulo
    /// - More intuitive for debugging
    /// - Multiplication by 2π only at output
    current_progress: f32,
    
    /// How much we advance through the wave per audio sample.
    /// This determines the frequency: higher values = higher pitch.
    /// Calculated as: frequency / sample_rate
    ///
    /// ## Phase Increment Mathematics
    ///
    /// **Derivation**:
    /// - Need `frequency` complete cycles per second
    /// - Have `sample_rate` samples per second
    /// - Each sample advances: frequency/sample_rate cycles
    ///
    /// **Example**: 440 Hz at 44100 Hz sample rate
    /// - Increment = 440/44100 = 0.00997732
    /// - Takes 100.227 samples per cycle
    /// - Slight phase drift handled by modulo
    progress_per_frame: f32,
    
    /// The length of one complete sine wave cycle in radians.
    /// This is always 2π (approximately 6.28) for a sine wave.
    ///
    /// ## Why Store 2π?
    ///
    /// Could calculate inline, but:
    /// - Avoids repeated multiplication
    /// - Documents the constant's purpose
    /// - Could be different for other waveforms
    /// - Compiler likely optimizes anyway
    period: f32,
    
    /// How many audio samples we generate per second.
    /// This must match the audio system's expectations.
    ///
    /// ## Sample Rate Standards
    ///
    /// **Common Rates and Their Uses**:
    /// - 8,000 Hz: Telephone quality (human speech)
    /// - 22,050 Hz: AM radio quality
    /// - 44,100 Hz: CD quality (11.025 × 4)
    /// - 48,000 Hz: Digital video standard
    /// - 96,000 Hz: High-resolution audio
    /// - 192,000 Hz: Professional mastering
    ///
    /// **Why 44,100?**
    /// Historical: Compatible with both PAL (25 fps)
    /// and NTSC (30 fps) video standards
    sample_rate: u32,
}

impl SineDecoder {
    /// Creates a new sine wave decoder with the specified frequency.
    ///
    /// ## Constructor Design Patterns
    ///
    /// **Associated Function vs Method**:
    /// - `new()` is convention for constructors in Rust
    /// - Not a method (no `self` parameter)
    /// - Called via `Type::new()` syntax
    /// - Could be named anything (from_frequency, with_freq)
    ///
    /// **Initialization Best Practices**:
    /// - Calculate derived values once in constructor
    /// - Validate inputs (we should check frequency > 0)
    /// - Set sensible defaults
    /// - Document assumptions
    fn new(frequency: f32) -> Self {
        // CD-quality sample rate: 44,100 samples per second.
        // This is the standard for most audio playback.
        // The underscore in 44_100 is a Rust feature for number readability.
        //
        // ## Hard-coded vs Configurable
        //
        // **Why hard-code sample rate?**
        // - Simplifies example
        // - 44.1kHz works everywhere
        // - Matches most audio content
        //
        // **Production Approach**:
        // ```rust
        // fn new(frequency: f32, sample_rate: u32) -> Self {
        //     // Validate inputs
        //     assert!(frequency > 0.0 && frequency < sample_rate as f32 / 2.0);
        // }
        // ```
        let sample_rate = 44_100;
        
        SineDecoder {
            // Start at the beginning of the sine wave
            //
            // ## Phase Initialization
            //
            // Starting at 0 means:
            // - First sample = sin(0) = 0
            // - Avoids initial click
            // - Predictable behavior
            //
            // Could randomize for multiple decoders:
            // `current_progress: rand::random::<f32>()`
            current_progress: 0.,
            
            // Calculate how much to advance per sample.
            // Example: 440 Hz / 44100 samples/sec = 0.00997 progress per sample
            // This means we complete 440 full cycles in one second.
            //
            // ## Type Casting in Audio Code
            //
            // `as f32` converts u32 to f32:
            // - Required for division with f32
            // - Lossless for sample rates (< 2^24)
            // - Alternative: `f32::from()` (safer but verbose)
            progress_per_frame: frequency / sample_rate as f32,
            
            // One complete sine wave cycle is 2π radians.
            // std::f32::consts::PI is Rust's built-in constant for π.
            //
            // ## Mathematical Constants
            //
            // Rust provides high-precision constants:
            // - PI: 3.14159265358979323846
            // - E: 2.71828182845904523536
            // - SQRT_2: 1.41421356237309504880
            //
            // Using `* 2.` instead of `* 2.0`:
            // - Both work (type inference)
            // - 2. is shorter and clear
            // - Common in audio DSP code
            period: std::f32::consts::PI * 2.,
            
            // Field init shorthand: `sample_rate: sample_rate` → `sample_rate`
            sample_rate,
        }
    }
}

/// Implement Iterator to generate audio samples one at a time.
/// 
/// The audio system will call `next()` repeatedly to get samples.
/// This is a "pull" model - the audio system requests samples as needed.
/// 
/// Each call to `next()` represents one moment in time, separated by
/// 1/sample_rate seconds from the previous call.
///
/// ## Iterator Pattern in Audio
///
/// **Why Iterator?**
/// - Lazy evaluation: Generate samples on-demand
/// - Composable: Can chain audio processors
/// - Memory efficient: No buffer allocation
/// - Rust idiomatic: Integrates with ecosystem
///
/// **Alternative Patterns**:
/// ```rust
/// // Callback (push model)
/// fn process(&mut self, output: &mut [f32])
/// 
/// // Stream (async)
/// async fn samples(&mut self) -> Stream<Item = f32>
/// ```
impl Iterator for SineDecoder {
    /// Each audio sample is a 32-bit floating-point number.
    /// The value typically ranges from -1.0 to 1.0, representing
    /// the maximum negative and positive air pressure.
    ///
    /// ## Audio Sample Representation
    ///
    /// **Why f32?**
    /// - Industry standard for audio processing
    /// - Good balance of precision and performance
    /// - SIMD operations available
    /// - Sufficient for human hearing
    ///
    /// **Sample Value Ranges**:
    /// - [-1.0, 1.0]: Normalized range (standard)
    /// - [-32768, 32767]: 16-bit integer range
    /// - [0.0, 1.0]: Unsigned (rare in audio)
    ///
    /// **Headroom**: Keep peaks at ~0.7 to prevent clipping
    type Item = f32;

    /// Generate the next audio sample.
    /// 
    /// This function is called 44,100 times per second during playback!
    ///
    /// ## Real-time Performance Requirements
    ///
    /// **Time Budget**: At 44.1kHz, we have:
    /// - 22.7 μs per sample
    /// - ~1000 CPU cycles at 4GHz
    /// - Must be deterministic (no allocation)
    ///
    /// **Performance Tips**:
    /// - Avoid branches (use branchless math)
    /// - Minimize function calls
    /// - Use lookup tables for complex functions
    /// - Batch process when possible
    fn next(&mut self) -> Option<Self::Item> {
        // Advance our position in the sine wave
        self.current_progress += self.progress_per_frame;
        
        // Wrap around when we complete a full cycle.
        // The modulo operator (%) keeps our progress between 0 and 1.
        // This prevents floating-point errors from accumulating over time.
        //
        // ## Modulo vs Conditional Wrapping
        //
        // **Option 1 - Modulo (used here)**:
        // ```rust
        // phase %= 1.0;  // Works for any overshoot
        // ```
        //
        // **Option 2 - Conditional**:
        // ```rust
        // if phase >= 1.0 { phase -= 1.0; }  // Faster, single overshoot only
        // ```
        //
        // **Option 3 - Bit masking (power-of-2 table)**:
        // ```rust
        // phase &= TABLE_SIZE - 1;  // Fastest, requires lookup table
        // ```
        self.current_progress %= 1.;
        
        // Calculate the sine value for our current position.
        // We multiply progress (0 to 1) by period (2π) to get radians (0 to 2π).
        // The sine function then gives us a smooth wave from -1 to 1.
        //
        // ## Sine Calculation Deep Dive
        //
        // **What happens in ops::sin()?**
        // 1. Range reduction to [-π, π]
        // 2. Taylor series or CORDIC algorithm
        // 3. ~20-50 CPU cycles typically
        //
        // **Optimization for production**:
        // ```rust
        // // Lookup table approach
        // const TABLE_SIZE: usize = 4096;
        // static SINE_TABLE: [f32; TABLE_SIZE] = /* pre-calculated */;
        // let index = (self.current_progress * TABLE_SIZE as f32) as usize;
        // let sample = SINE_TABLE[index];
        // ```
        //
        // **Accuracy vs Performance**:
        // - f32 sine: ~7 decimal places
        // - Lookup table: Depends on size
        // - Linear interpolation: Smooths between entries
        Some(ops::sin(self.period * self.current_progress))
        
        // `Some` wraps our value in an Option. Returning None would end playback,
        // but we want to generate samples forever, so we always return Some.
        //
        // ## Option Pattern in Iterators
        //
        // **Why Option?**
        // - None signals end of iteration
        // - Some(value) continues with value
        // - Allows finite and infinite sequences
        //
        // **For finite sounds**:
        // ```rust
        // if self.samples_remaining > 0 {
        //     self.samples_remaining -= 1;
        //     Some(sample)
        // } else {
        //     None
        // }
        // ```
    }
}

/// Implement the Source trait to provide metadata about our audio stream.
/// 
/// This trait tells the audio system important information about our audio format
/// so it can be mixed and played correctly with other audio sources.
///
/// ## Audio Pipeline Architecture
///
/// **Source Trait in the Audio Graph**:
/// ```text
/// [Source] -> [Resampler] -> [ChannelMixer] -> [VolumeControl] -> [Output]
///     |            |              |                  |
///   Metadata   If needed     Mono to Stereo    Final mixing
/// ```
///
/// **Why These Methods Matter**:
/// - `channels()`: Determines mixing strategy
/// - `sample_rate()`: Triggers resampling if needed
/// - `total_duration()`: Enables seeking and progress bars
/// - `current_frame_len()`: Optimizes buffer allocation
impl Source for SineDecoder {
    /// How many samples are left in the current frame.
    /// We return None because our sine wave is infinite - it never ends.
    ///
    /// ## Audio Frames vs Samples
    ///
    /// **Terminology**:
    /// - **Sample**: Single audio value (one channel)
    /// - **Frame**: All channels for one time point
    /// - **Buffer**: Multiple frames for processing
    ///
    /// **Examples**:
    /// - Mono: 1 sample = 1 frame
    /// - Stereo: 2 samples = 1 frame
    /// - 5.1 Surround: 6 samples = 1 frame
    ///
    /// **Buffer Strategies**:
    /// - Fixed size: Predictable latency
    /// - Variable: Adapt to CPU load
    /// - None (streaming): Infinite sources
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    /// Number of audio channels.
    /// 1 = mono (single speaker)
    /// 2 = stereo (left and right speakers)
    /// We generate a simple mono signal.
    ///
    /// ## Channel Configurations
    ///
    /// **Common Layouts**:
    /// - 1: Mono (center)
    /// - 2: Stereo (left, right)
    /// - 3: LCR (left, center, right)
    /// - 4: Quadraphonic (FL, FR, BL, BR)
    /// - 6: 5.1 Surround (FL, FR, C, LFE, BL, BR)
    /// - 8: 7.1 Surround
    ///
    /// **Mono to Stereo Conversion**:
    /// - Center panning: L = R = mono
    /// - Wide panning: L = mono, R = -mono
    /// - Haas effect: L = mono, R = delayed mono
    fn channels(&self) -> u16 {
        1
    }

    /// The sample rate must match what we're actually generating.
    /// Mismatched sample rates would cause the audio to play at the wrong speed!
    ///
    /// ## Sample Rate Conversion (SRC)
    ///
    /// **When Rates Don't Match**:
    /// - 44.1kHz source -> 48kHz output: Upsample
    /// - 48kHz source -> 44.1kHz output: Downsample
    ///
    /// **Resampling Quality Levels**:
    /// 1. **Linear**: Fast but aliasing (games)
    /// 2. **Cubic**: Good quality/speed balance
    /// 3. **Sinc**: Best quality, expensive (audio production)
    ///
    /// **Common Conversion Ratios**:
    /// - 44.1 -> 48: multiply by 160/147
    /// - 48 -> 44.1: multiply by 147/160
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Total duration of the audio.
    /// None means infinite duration - our sine wave plays forever.
    ///
    /// ## Duration in Audio Systems
    ///
    /// **Uses of Duration**:
    /// - Progress bars in media players
    /// - Scheduling next track
    /// - Memory allocation hints
    /// - Seek bar boundaries
    ///
    /// **Finite Sound Example**:
    /// ```rust
    /// fn total_duration(&self) -> Option<Duration> {
    ///     let total_samples = self.sample_count;
    ///     let seconds = total_samples as f64 / self.sample_rate as f64;
    ///     Some(Duration::from_secs_f64(seconds))
    /// }
    /// ```
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Connect our SineAudio asset type to its decoder.
/// 
/// This trait implementation tells Bevy: "When you need to play a SineAudio asset,
/// create a SineDecoder to generate the actual audio samples."
///
/// ## The Decodable Pattern
///
/// **Separation of Concerns**:
/// ```text
/// Asset (Data)        Decoder (State)       Output
/// ------------        ---------------       ------
/// SineAudio    --->   SineDecoder    --->  f32 samples
/// (frequency)         (phase, etc)          (audio out)
/// ```
///
/// **Why This Architecture?**
/// 1. **Stateless Assets**: Can be shared, cached, serialized
/// 2. **Stateful Decoders**: Unique per playback instance
/// 3. **Type Safety**: Compiler ensures compatibility
/// 4. **Flexibility**: Different decoders for same asset
///
/// ## Generic Associated Types (GATs)
///
/// This trait uses associated types, a powerful Rust feature:
/// - `type DecoderItem`: What each sample looks like
/// - `type Decoder`: What generates the samples
///
/// **Benefits over Generics**:
/// - Cleaner syntax
/// - Better type inference  
/// - Enforces single implementation
impl Decodable for SineAudio {
    /// The type of samples our decoder produces (f32 from the Iterator impl)
    ///
    /// ## Type Projection Syntax
    ///
    /// `<Type as Trait>::AssocType` explicitly names an associated type:
    /// - Disambiguates when multiple traits have same associated type
    /// - Documents where the type comes from
    /// - Required when type inference can't determine it
    ///
    /// **Could also write**:
    /// ```rust
    /// type DecoderItem = f32;  // Direct, but less flexible
    /// ```
    type DecoderItem = <SineDecoder as Iterator>::Item;

    /// The decoder type that will generate samples
    type Decoder = SineDecoder;

    /// Create a decoder instance for this audio asset.
    /// This is called when the audio starts playing.
    ///
    /// ## Decoder Lifecycle
    ///
    /// **When decoder() is called**:
    /// 1. User spawns AudioPlayer component
    /// 2. Audio system detects new AudioPlayer
    /// 3. Retrieves asset from Assets<SineAudio>
    /// 4. Calls decoder() to create playback state
    /// 5. Adds decoder to active sources
    /// 6. Begins pulling samples
    ///
    /// **Memory Efficiency**:
    /// - Asset: One instance, shared
    /// - Decoders: One per active playback
    /// - Samples: Generated on-demand
    ///
    /// **&self Access Pattern**:
    /// - Read-only access to asset
    /// - Can't modify during playback
    /// - Ensures thread safety
    fn decoder(&self) -> Self::Decoder {
        SineDecoder::new(self.frequency)
    }
}

fn main() {
    let mut app = App::new();
    
    // Configure the default plugins with custom audio settings
    //
    // ## Plugin Configuration Pattern
    //
    // **DefaultPlugins**: Bundle of essential plugins:
    // - WindowPlugin: Creates OS window
    // - RenderPlugin: Graphics pipeline
    // - AssetPlugin: Asset loading
    // - AudioPlugin: Audio playback
    // - InputPlugin: Keyboard/mouse/gamepad
    // - TimePlugin: Frame timing
    //
    // **The .set() Pattern**:
    // Replaces specific plugin in the bundle
    // while keeping others at defaults
    app.add_plugins(
        DefaultPlugins.set(
            AudioPlugin {
                // Set global volume to 20% to avoid hurting ears.
                // Volume::Linear(1.0) would be full volume.
                // Linear volume is intuitive: 0.5 = half volume, 0.2 = 20% volume
                //
                // ## Volume Types
                //
                // **Volume::Linear(x)**:
                // - Human-perceived loudness
                // - 0.5 sounds like "half volume"
                // - Good for user interfaces
                //
                // **Volume::Decibels(db)**:
                // - Physical amplitude scale  
                // - -6 dB = half amplitude
                // - -20 dB = 1/10 amplitude
                // - Professional audio standard
                //
                // **Conversion**:
                // dB = 20 * log10(linear)
                // linear = 10^(dB/20)
                global_volume: Volume::Linear(0.2).into(),
                // Use default values for other audio settings
                //
                // ## Other AudioPlugin Settings
                //
                // ```rust
                // AudioPlugin {
                //     global_volume: Volume::Linear(0.2).into(),
                //     default_spatial_scale: 1.0,  // 3D audio distance units
                //     default_reverb: None,        // Global reverb effect
                // }
                // ```
                ..default()
            }
        )
    )
    // Register our custom audio source type with Bevy's audio system.
    // This creates the necessary resources and systems to handle SineAudio assets.
    //
    // ## What add_audio_source Does
    //
    // **Creates Resources**:
    // - `Assets<SineAudio>`: Storage for SineAudio assets
    // - `AudioSources<SineAudio>`: Active playback tracking
    //
    // **Registers Systems**:
    // - Detect new AudioPlayer<SineAudio> components
    // - Create decoders and start playback
    // - Clean up finished audio
    //
    // **Type Registration**:
    // - Enables reflection for SineAudio
    // - Allows asset inspection in editor
    .add_audio_source::<SineAudio>()
    // Set up our audio when the app starts
    .add_systems(Startup, setup)
    // Start the app
    //
    // ## Application Lifecycle
    //
    // **What .run() does**:
    // 1. Initialize all plugins
    // 2. Create window and audio device
    // 3. Run Startup systems once
    // 4. Enter main loop:
    //    - Process events
    //    - Update systems
    //    - Render frame
    //    - Sleep if ahead of target FPS
    // 5. Cleanup on exit
    .run();
}

/// Create and play a sine wave audio source.
///
/// ## System Parameters for Audio
///
/// **Common Audio System Parameters**:
/// - `Res<AssetServer>`: Load audio from files
/// - `ResMut<Assets<T>>`: Create procedural audio
/// - `Res<AudioSink>`: Control playback
/// - `EventWriter<AudioEvent>`: Trigger audio events
/// - `Query<&AudioPlayer>`: Find audio entities
fn setup(
    // Direct access to the SineAudio asset storage.
    // Unlike AssetServer (for loading files), we manually add assets here.
    // ResMut means we have mutable access to this resource.
    //
    // ## Assets<T> Resource
    //
    // **Internal Structure**:
    // ```rust
    // pub struct Assets<T> {
    //     assets: HashMap<AssetId, T>,
    //     events: Events<AssetEvent<T>>,
    // }
    // ```
    //
    // **Thread Safety**:
    // - Wrapped in RwLock for concurrent access
    // - Read from multiple systems
    // - Write from one system at a time
    //
    // **Memory Management**:
    // - Reference counted handles
    // - Automatic cleanup at zero refs
    // - Can force retention with strong handles
    mut assets: ResMut<Assets<SineAudio>>,
    // For spawning entities
    //
    // ## Commands Buffer Pattern
    //
    // **Why Commands?**
    // - Structural changes (spawn/despawn) invalidate queries
    // - Multiple systems might conflict
    // - Commands queue changes for safe application
    //
    // **What Can Commands Do?**
    // - Spawn/despawn entities
    // - Add/remove components  
    // - Insert/remove resources
    // - Run one-shot systems
    mut commands: Commands,
) {
    // Create a SineAudio asset and add it to the asset storage.
    // This returns a Handle - a reference to the asset.
    //
    // ## Handle Types and Reference Counting
    //
    // **Handle Variants**:
    // - `Handle::Strong`: Keeps asset loaded
    // - `Handle::Weak`: Allows unloading
    //
    // **Lifecycle Example**:
    // ```rust
    // let handle = assets.add(data);        // Ref count: 1
    // let clone = handle.clone();           // Ref count: 2
    // drop(handle);                         // Ref count: 1
    // drop(clone);                          // Ref count: 0, asset freed
    // ```
    let audio_handle = assets.add(SineAudio {
        // 440 Hz is the musical note A4 (the A above middle C).
        // This is the standard tuning reference for orchestras.
        //
        // ## A440 Tuning Standard
        //
        // **Historical Context**:
        // - 1939: International conference chose 440 Hz
        // - 1955: ISO 16 standard
        // - 1975: Reaffirmed as ISO 16:1975
        //
        // **Regional Variations**:
        // - New York Philharmonic: 442 Hz (brighter)
        // - Berlin Philharmonic: 443 Hz
        // - Baroque pitch: 415 Hz (semitone lower)
        // - Scientific pitch: 430.54 Hz (C4 = 256 Hz)
        //
        // **Why It Matters**:
        // - Instruments tune together
        // - Electronic and acoustic match
        // - International recordings compatible
        frequency: 440.,
    });
    
    // Spawn an entity with an AudioPlayer component.
    // The AudioPlayer component tells Bevy's audio system to play this sound.
    // Unlike AudioSource (for file-based audio), we pass our custom handle here.
    //
    // ## AudioPlayer Component Internals
    //
    // **What AudioPlayer Contains**:
    // ```rust
    // pub struct AudioPlayer<T>(pub Handle<T>);
    // ```
    //
    // **Playback Flow**:
    // 1. System queries for Added<AudioPlayer<T>>
    // 2. Retrieves T from Assets<T> using handle
    // 3. Calls T::decoder() to create decoder
    // 4. Adds decoder to audio mixer
    // 5. Removes AudioPlayer when done (unless looping)
    //
    // **Entity Component Architecture**:
    // - Entity: Just an ID
    // - AudioPlayer: Component with audio to play
    // - Transform: Could add for 3D positioning
    // - Name: Could add for debugging
    commands.spawn(AudioPlayer(audio_handle));
}
