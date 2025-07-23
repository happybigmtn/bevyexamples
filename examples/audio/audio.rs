//! This example illustrates how to load and play an audio file.
//! For loading additional audio formats, you can enable the corresponding feature for that audio format.
//! 
//! ## Key Concepts
//! 
//! - **Audio Pipeline**: Bevy's audio system is built on top of the `rodio` library, which handles
//!   audio decoding, mixing, and playback across different platforms.
//! 
//! - **Asset Loading**: Audio files are loaded asynchronously through Bevy's asset system, meaning
//!   the game doesn't freeze while loading large audio files.
//! 
//! - **AudioPlayer Component**: A component that tells Bevy to play an audio source. Once spawned,
//!   it automatically starts playing and removes itself when finished (unless looping).
//! 
//! ## Supported Audio Formats
//! 
//! By default, Bevy supports:
//! - OGG Vorbis (.ogg) - Open-source, compressed format ideal for game audio
//! - WAV (.wav) - Uncompressed format, larger files but no decoding overhead
//! - FLAC (.flac) - Lossless compression, balance between size and quality
//! 
//! Additional formats can be enabled in Cargo.toml:
//! - MP3 support: Add feature "mp3" to bevy dependency
//! - AAC support: Add feature "aac" to bevy dependency
//!
//! ## Audio Theory: Digital Sound Fundamentals
//!
//! **How Digital Audio Works**: Sound waves are continuous analog signals that must be converted
//! to discrete digital samples. This happens through:
//!
//! 1. **Sampling**: Measuring the amplitude of a sound wave at regular intervals
//!    - Common sample rates: 44.1kHz (CD quality), 48kHz (professional), 96kHz (high-res)
//!    - Higher sample rates capture higher frequencies (Nyquist theorem: max frequency = sample rate / 2)
//!
//! 2. **Quantization**: Converting continuous amplitude values to discrete steps
//!    - Bit depth determines dynamic range: 16-bit = 96dB, 24-bit = 144dB
//!    - Each bit adds ~6dB of dynamic range
//!
//! 3. **Encoding**: Storing or compressing the digital data
//!    - Lossless (FLAC, ALAC): Preserves all data, ~50% compression
//!    - Lossy (MP3, OGG, AAC): Removes "inaudible" data, ~90% compression
//!
//! **Psychoacoustics in Audio Compression**: Lossy formats exploit human hearing limitations:
//! - **Frequency Masking**: Loud sounds hide nearby quiet frequencies
//! - **Temporal Masking**: Loud sounds hide quiet sounds immediately before/after
//! - **Critical Bands**: Ear groups frequencies into ~24 bands for processing
//!
//! ## Game Design Context: Why Audio Matters
//!
//! **Player Engagement Through Sound**:
//! - **Emotional Response**: Music triggers limbic system, creating mood instantly
//! - **Spatial Awareness**: 3D audio provides information without visual attention
//! - **Feedback Loops**: Audio confirms actions faster than visual (audio latency ~20ms vs visual ~50ms)
//! - **Immersion**: Consistent soundscape maintains suspension of disbelief
//!
//! **The One-Shot Audio Pattern**: This example demonstrates the simplest audio pattern:
//! fire-and-forget playback. Perfect for:
//! - Menu sounds and UI feedback
//! - Ambient music loops
//! - Victory/defeat stingers
//! - Simple sound effects without position
//!
//! ## Performance Optimization: Audio Efficiency
//!
//! **Memory Considerations**:
//! - WAV files load entirely into RAM: 1 minute stereo @ 44.1kHz = ~10MB
//! - Compressed formats decode on-the-fly: Higher CPU usage, lower memory
//! - Streaming large files prevents memory spikes but increases I/O
//!
//! **CPU Usage Patterns**:
//! ```text
//! Format     | Decode Cost | Memory Usage | Use Case
//! -----------|-------------|--------------|----------
//! WAV        | None        | High         | Short SFX, low latency
//! OGG        | Medium      | Low          | Music, ambient loops  
//! MP3        | High        | Low          | Legacy content
//! ```
//!
//! **Bevy's Audio Threading**: Audio runs on a dedicated thread to prevent stuttering:
//! - Main thread sends commands via channels
//! - Audio thread maintains consistent timing
//! - Decoding happens ahead of playback in ring buffers
//!
//! ## Real-World Applications
//!
//! **Industry Best Practices**:
//! 1. **Format Selection**:
//!    - SFX < 1 second: WAV for zero latency
//!    - SFX > 1 second: OGG to save memory
//!    - Music: OGG with streaming for large files
//!    - Voice: OGG at lower bitrates (64-96 kbps)
//!
//! 2. **File Organization**:
//!    ```
//!    assets/audio/
//!    ├── sfx/        # Short sound effects
//!    ├── music/      # Background music
//!    ├── ambient/    # Environmental sounds
//!    └── voice/      # Dialog and narration
//!    ```
//!
//! 3. **Loudness Standards**: Game audio should target:
//!    - Integrated loudness: -23 LUFS (console) to -16 LUFS (mobile)
//!    - True peak: -1 dBFS to prevent clipping
//!    - Dynamic range: 7-20 LU depending on genre
//!
//! ## Advanced Techniques: Beyond Basic Playback
//!
//! **Future Enhancements** you might add to this example:
//! 1. **Fade In/Out**: Smooth volume transitions prevent harsh cuts
//! 2. **Crossfading**: Seamless music transitions for different game states  
//! 3. **Dynamic Loading**: Load/unload audio based on game area
//! 4. **Audio Pooling**: Reuse AudioPlayer entities for frequent sounds
//!
//! **Procedural Audio Possibilities**:
//! - Generate tones with sine waves for retro effects
//! - Create noise-based textures for wind/water
//! - Synthesize explosions with filtered noise bursts
//! - Build dynamic music from layered stems
//!
//! ## Common Issues and Solutions
//!
//! **Problem**: Audio doesn't play
//! - **Check**: File path is relative to 'assets' folder
//! - **Check**: File format is supported (enable features if needed)
//! - **Check**: Audio file isn't corrupted (test in media player)
//! - **Debug**: Add `.after(AssetEvents)` to ensure assets are loaded
//!
//! **Problem**: Audio cuts off abruptly  
//! - **Cause**: Entity despawned while playing
//! - **Solution**: Keep entity alive with marker component
//! - **Alternative**: Use AudioSink for more control
//!
//! **Problem**: Multiple sounds play simultaneously causing distortion
//! - **Cause**: Audio summing exceeds 0 dBFS
//! - **Solution**: Reduce individual volumes or implement mixing groups
//! - **Prevention**: Design audio with headroom for overlapping sounds

// The prelude module contains the most commonly used types and traits in Bevy.
// It's a convenience import that brings in everything you need for basic Bevy development.
use bevy::prelude::*;

// ## Rust Programming Fundamentals: Understanding the Main Function
//
// The `fn main()` function is the entry point of every Rust program. In game development,
// this is where we configure the engine before entering the game loop. Rust's ownership
// system ensures that resources like audio buffers are properly managed without manual
// memory management.
//
// **Memory Safety in Audio**: Rust prevents common audio programming bugs:
// - Buffer overruns (reading past allocated memory)
// - Use-after-free (accessing deallocated audio data)  
// - Data races (multiple threads modifying audio state)
// - Null pointer access (missing audio resources)
fn main() {
    // ## Bevy Architecture: The App Builder Pattern
    //
    // Bevy uses the "builder pattern" - chaining method calls to configure the app.
    // This pattern is common in Rust for constructing complex objects with many options.
    // Each method call returns `Self`, allowing the chain to continue.
    //
    // **Why Builder Pattern for Game Engines?**
    // - Type safety: Compiler catches configuration errors
    // - Flexibility: Add only the features you need
    // - Discoverability: IDE autocomplete shows available options
    // - Zero cost: Optimized away at compile time
    App::new()
        // DefaultPlugins includes all the standard Bevy functionality:
        // - Window creation and event handling
        // - Rendering pipeline  
        // - Asset loading system
        // - Audio playback system (via AudioPlugin)
        // - Input handling
        // - Time and scheduling
        //
        // ## Audio Plugin Architecture
        //
        // The AudioPlugin (included in DefaultPlugins) sets up:
        // 1. **Audio Output Device**: Connects to system audio (WASAPI/CoreAudio/ALSA)
        // 2. **Mixer Thread**: Combines multiple audio sources in real-time
        // 3. **Decoder Pool**: Parallel decoding of compressed formats
        // 4. **Spatial Audio Engine**: 3D positioning and attenuation
        // 5. **Effect Pipeline**: Reverb, filters, and DSP effects
        //
        // **Thread Architecture**:
        // ```
        // Main Thread          Audio Thread         Decoder Threads
        // -----------          ------------         ---------------
        // [Commands]  ------>  [Mixer]     <-----  [OGG Decoder]
        //                      [Output]     <-----  [MP3 Decoder]
        //                                   <-----  [FLAC Decoder]
        // ```
        .add_plugins(DefaultPlugins)
        // The Startup schedule runs once when the app starts, before the first frame.
        // It's perfect for loading assets and setting up the initial game state.
        //
        // ## System Scheduling and Audio Timing
        //
        // Bevy's scheduling ensures audio systems run at the right time:
        // - **Startup**: Load audio assets, initialize audio state
        // - **PreUpdate**: Process audio events from last frame
        // - **Update**: Update audio sources, positions, parameters
        // - **PostUpdate**: Send commands to audio thread
        //
        // **Real-time Constraints**: Audio has strict timing requirements:
        // - Audio callbacks run every ~5-20ms (depends on buffer size)  
        // - Missing a callback causes audible glitches
        // - That's why audio runs on a separate high-priority thread
        .add_systems(Startup, setup)
        // Start the application's main loop
        //
        // ## The Game Loop and Audio Synchronization
        //
        // Once run() is called, Bevy enters an infinite loop:
        // 1. Poll input events
        // 2. Update game logic (including audio triggers)
        // 3. Render frame
        // 4. Sleep to maintain target framerate
        //
        // Meanwhile, the audio thread runs independently, consuming
        // commands from a lock-free queue and mixing audio in real-time.
        .run();
}

/// Sets up the audio playback by spawning an entity with an AudioPlayer component.
/// 
/// # Parameters
/// 
/// - `asset_server`: A resource that handles loading files from disk. When you request
///   an asset, it returns a `Handle<T>` immediately, even if the asset isn't loaded yet.
/// 
/// - `commands`: Allows us to spawn entities and add components. The `mut` keyword means
///   we can modify the world through this parameter.
///
/// ## Rust System Parameters Deep Dive
///
/// Bevy systems can request different types of parameters:
/// - `Res<T>`: Immutable access to a resource (read-only)
/// - `ResMut<T>`: Mutable access to a resource (read-write)
/// - `Query<T>`: Access to entities with specific components
/// - `Commands`: Deferred world modifications
/// - `EventReader<T>`/`EventWriter<T>`: Event handling
///
/// **Why Separate Read/Write Access?**
/// Rust's borrow checker ensures data race safety. By declaring whether you need
/// read or write access, Bevy can:
/// 1. Run systems in parallel when they don't conflict
/// 2. Catch errors at compile time instead of runtime
/// 3. Optimize cache usage by grouping reads
fn setup(
    // Res<T> is a system parameter that gives read-only access to a resource.
    // Resources are global data accessible from any system.
    //
    // ## Asset Loading Architecture
    //
    // The AssetServer manages a sophisticated loading pipeline:
    // 1. **Asset Discovery**: Scans 'assets' folder at startup
    // 2. **Format Detection**: Identifies file type by extension
    // 3. **Async Loading**: Loads files without blocking main thread
    // 4. **Hot Reloading**: Detects file changes during development
    // 5. **Reference Counting**: Automatically unloads unused assets
    //
    // **Handle Pattern**: Instead of loading assets directly, Bevy returns
    // lightweight Handle<T> structs that act as smart pointers to assets.
    // This enables:
    // - Loading assets before they're needed
    // - Sharing assets between multiple users
    // - Automatic cleanup when no longer referenced
    asset_server: Res<AssetServer>,
    // Commands let us make structural changes to the world (spawn/despawn entities).
    // These changes are applied after all systems in the current stage finish.
    //
    // ## The Commands Pattern: Deferred Execution
    //
    // Why doesn't Bevy modify the world immediately? 
    // 1. **Parallelism**: Multiple systems can run simultaneously
    // 2. **Consistency**: All systems see the same world state
    // 3. **Performance**: Batching changes is more cache-friendly
    //
    // Commands are stored in a queue and applied between stages:
    // ```
    // [System A] --> [Commands Queue] <-- [System B]
    //                       |
    //                       v
    //                [Apply Commands]
    //                       |
    //                       v
    //                 [Next Stage]
    // ```
    mut commands: Commands,
) {
    // ## Audio Memory Management
    //
    // When we spawn an audio entity, several allocations occur:
    // 1. **Entity Storage**: ~16 bytes for entity ID and component pointers
    // 2. **AudioPlayer Component**: ~24 bytes for handle and settings
    // 3. **Audio Buffer**: Depends on format (OGG is streamed, not fully loaded)
    //
    // The actual audio data lives in a separate arena allocator managed
    // by the asset system, preventing fragmentation.
    
    // Spawn a new entity with an AudioPlayer component
    commands.spawn(
        // AudioPlayer::new() creates a component that plays audio
        //
        // ## Component Design Philosophy
        //
        // AudioPlayer is a "tag component" - it signals intent rather than
        // storing complex state. The actual playback state lives in the
        // audio thread. This separation enables:
        // - Lock-free communication between threads
        // - Multiple views of the same audio (position, volume, etc.)
        // - Clean separation of concerns
        AudioPlayer::new(
            // load() returns a Handle<AudioSource> immediately, even before the file is loaded.
            // The path is relative to the "assets" folder in your project root.
            //
            // ## The OGG Vorbis Format: Why It's Perfect for Games
            //
            // OGG Vorbis was specifically designed for game audio:
            // 1. **Variable Bitrate**: Allocates bits where needed
            //    - Complex sections: 192-320 kbps
            //    - Simple sections: 64-128 kbps
            //    - Silence: Near 0 kbps
            //
            // 2. **Gapless Playback**: No padding at start/end
            //    - MP3 adds ~1000 samples of silence
            //    - OGG has sample-accurate loops
            //
            // 3. **Low Latency Decoding**: 
            //    - Small frame size (64-8192 samples)
            //    - No bit reservoir like MP3
            //    - Predictable CPU usage
            //
            // 4. **Metadata Support**: 
            //    - Loop points for seamless music
            //    - Cue points for synchronized events
            //    - Multiple streams in one file
            //
            // Bevy will:
            // 1. Start loading "assets/sounds/Windless Slopes.ogg" in the background
            // 2. Return a handle that will point to the audio once loaded
            // 3. The AudioPlayer will wait for loading to complete before playing
            asset_server.load("sounds/Windless Slopes.ogg"),
        ),
    );
    
    // What happens next:
    // 1. The audio system detects the new AudioPlayer component
    // 2. Once the audio file is loaded, playback begins automatically
    // 3. When playback finishes, the AudioPlayer component is removed
    // 4. If the entity has no other components, it's automatically despawned
    //
    // ## Behind the Scenes: Audio Playback Pipeline
    //
    // 1. **Entity Creation** (This System)
    //    - Allocates entity ID from generational arena
    //    - Stores AudioPlayer component in component storage
    //    - Adds entity to audio system's query cache
    //
    // 2. **Asset Loading** (Asset Thread)  
    //    - Opens file using memory-mapped I/O
    //    - Identifies OGG by magic bytes (OggS)
    //    - Creates streaming decoder
    //    - Signals asset ready via atomic flag
    //
    // 3. **Playback Start** (Audio System)
    //    - Queries for new AudioPlayer components
    //    - Checks if asset handle is loaded
    //    - Creates rodio Source from decoder
    //    - Adds to mixer queue
    //
    // 4. **Audio Mixing** (Audio Thread)
    //    - Decodes next chunk of OGG data
    //    - Converts to f32 samples
    //    - Mixes with other active sounds
    //    - Sends to OS audio buffer
    //
    // 5. **Cleanup** (Audio System)
    //    - Detects playback completion
    //    - Removes AudioPlayer component
    //    - Despawns entity if empty
    //
    // ## Performance Considerations
    //
    // **CPU Usage**: 
    // - OGG decoding: ~1-2% CPU per stream
    // - Mixing overhead: ~0.5% for 32 simultaneous sounds
    // - Total budget: Keep under 5% for audio
    //
    // **Memory Bandwidth**:
    // - 44.1kHz stereo = 352KB/s per stream
    // - Cache-friendly access patterns critical
    // - Consider memory layout for multiple sounds
    //
    // **Latency Chain**:
    // - Command queue: 0-16ms (one frame)
    // - Asset loading: 5-50ms (disk I/O)
    // - Audio buffer: 5-20ms (OS dependent)
    // - Total: 10-86ms from trigger to speaker
}
