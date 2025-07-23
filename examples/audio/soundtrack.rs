//! This example illustrates how to load and play different soundtracks,
//! transitioning between them as the game state changes.
//! 
//! ## Key Concepts
//! 
//! - **Dynamic Soundtracks**: Modern games need music that responds to gameplay. Battle music
//!   during combat, peaceful music during exploration, tense music during stealth sections.
//! 
//! - **Audio Fading**: Smooth volume transitions prevent jarring cuts between tracks. This
//!   creates a more professional and immersive audio experience.
//! 
//! - **AudioSink**: While AudioPlayer starts playback, AudioSink gives you runtime control
//!   over playing audio - volume, speed, pausing, and stopping.
//! 
//! - **State-Driven Audio**: By tying audio changes to game state changes, the soundtrack
//!   automatically stays synchronized with gameplay.
//!
//! ## Audio Theory: Adaptive Music Systems
//!
//! **Horizontal vs Vertical Approaches**:
//! - **Horizontal Re-sequencing**: Switch between different tracks
//! - **Vertical Remixing**: Layer/remove instruments in same track
//! - **Hybrid Systems**: Combine both for maximum flexibility
//!
//! **Crossfading Mathematics**:
//! ```text
//! Linear:     Volume = t              (sounds unnatural)
//! Equal Power: Volume = sin(t * π/2)   (constant perceived loudness)
//! Logarithmic: Volume = log(1 + 9t)/log(10) (matches human perception)
//! ```
//!
//! **Musical Timing Considerations**:
//! - **Beat Matching**: Transition on musical boundaries
//! - **Key Compatibility**: Avoid clashing harmonies
//! - **Tempo Synchronization**: Match BPM or gradually shift
//!
//! ## Game Design Context: Emotional Soundscapes
//!
//! **Music as Gameplay Feedback**:
//! - **Proximity**: Music intensity based on danger distance
//! - **Health**: Desperate music at low health
//! - **Progress**: Triumphant swells near objectives
//! - **Stealth**: Music absence creates tension
//!
//! **Famous Examples**:
//! - **Red Dead Redemption**: Seamless exploration/combat transitions
//! - **DOOM (2016)**: Music intensity matches combat state
//! - **Hades**: Unique tracks for each game area and boss
//! - **Celeste**: Music tempo linked to game speed
//!
//! ## Performance Optimization: Efficient Music Systems
//!
//! **Memory Management**:
//! - **Streaming**: Load music in chunks, not all at once
//! - **Compression**: OGG Vorbis ~10:1 ratio vs WAV
//! - **Preloading**: Cache next likely tracks
//! - **Unloading**: Free memory from unused tracks
//!
//! **CPU Optimization**:
//! ```text
//! Technique        | CPU Cost | Quality
//! -----------------|----------|--------
//! Hard Cut         | None     | Jarring
//! Linear Fade      | Minimal  | Acceptable
//! Equal Power Fade | Low      | Professional
//! Crossfade + EQ   | Medium   | Seamless
//! ```
//!
//! ## Real-World Applications
//!
//! **Interactive Music Middleware**:
//! - **FMOD Studio**: Industry standard, visual programming
//! - **Wwise**: Advanced integration, used in AAA games
//! - **Elias**: AI-driven adaptive music
//! - **Pure Data**: Open-source alternative
//!
//! **Music Stem Organization**:
//! ```
//! Track_Battle/
//! ├── Drums.ogg      (always playing)
//! ├── Bass.ogg       (fade in at 25% intensity)
//! ├── Melody.ogg     (fade in at 50% intensity)
//! └── Orchestra.ogg  (fade in at 75% intensity)
//! ```
//!
//! ## Advanced Techniques: Dynamic Mixing
//!
//! **Real-time Audio Effects**:
//! 1. **Low-pass Filter**: Muffle music when underwater
//! 2. **Reverb**: Add space depth in caverns
//! 3. **Distortion**: Signal damage or altered reality
//! 4. **Pitch Shift**: Slow motion or time dilation
//!
//! **Procedural Music Generation**:
//! - **Algorithmic Composition**: Generate melodies from gameplay
//! - **Markov Chains**: Probabilistic note sequences
//! - **Neural Networks**: AI-composed adaptive scores
//!
//! ## Common Issues and Solutions
//!
//! **Problem**: Music transitions feel abrupt
//! - **Solution**: Crossfade over musical bars, not seconds
//!
//! **Problem**: Multiple tracks playing causes clipping
//! - **Solution**: Implement mixing bus with compression
//!
//! **Problem**: Music doesn't loop seamlessly
//! - **Solution**: Ensure no silence at start/end, match tempo

use bevy::{audio::Volume, prelude::*};

// ## Rust Programming Fundamentals: Import Specificity
//
// **Why import Volume separately?**
// - Not in prelude::* to avoid namespace pollution
// - Only needed for advanced audio control
// - Makes code intent clearer
//
// **Alternative import styles**:
// ```rust
// use bevy::audio::{Volume, PlaybackMode, AudioPlugin};
// use bevy::audio::*;  // Import everything from audio
// use bevy::{prelude::*, audio};  // Import module itself
// ```

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // These systems work together to create smooth audio transitions:
        // - cycle_game_state: Simulates gameplay by changing states periodically
        // - fade_in/fade_out: Handle smooth volume transitions
        // - change_track: Responds to state changes by switching music
        .add_systems(Update, (cycle_game_state, fade_in, fade_out))
        .add_systems(Update, change_track)
        .run();
}

/// Represents different states in our game that require different music.
/// 
/// In a real game, this might include states like:
/// - MainMenu, Gameplay, Cutscene
/// - Exploration, Combat, Puzzle, Boss
/// - Day, Night, Underground, Space
/// 
/// The `Default` derive provides the initial state via `default()`.
/// The `#[default]` attribute specifies which variant is the default.
///
/// ## State Machine Design
///
/// **Simple State vs Hierarchical**:
/// ```rust
/// // Simple (this example)
/// enum GameState { Peaceful, Battle }
///
/// // Hierarchical (complex games)
/// enum GameState {
///     Menu(MenuState),
///     Playing(PlayState),
///     Cutscene(CutsceneState),
/// }
/// enum PlayState { Exploring, Fighting, Puzzle }
/// ```
///
/// **State Transition Matrix**:
/// ```text
/// From \ To  | Peaceful | Battle
/// -----------|----------|--------
/// Peaceful   | No       | Yes
/// Battle     | Yes      | No
/// ```
///
/// **Music Transition Rules**:
/// - Peaceful → Battle: Quick fade (0.5s) for responsiveness
/// - Battle → Peaceful: Slow fade (3s) for gradual calm
/// - Menu → Game: Crossfade at menu timing
#[derive(Resource, Default)]
enum GameState {
    #[default]
    Peaceful,
    Battle,
}

/// A timer that simulates game state transitions.
/// 
/// In a real game, state changes would be triggered by gameplay events:
/// - Entering combat when enemies are nearby
/// - Returning to peaceful when combat ends
/// - Changing to boss music when the boss appears
/// 
/// We wrap the Timer in a newtype struct for type safety.
///
/// ## Timer Systems in Games
///
/// **Types of Game Timers**:
/// 1. **Real Time**: Wall clock time (UI, networking)
/// 2. **Game Time**: Can be paused/scaled (gameplay)
/// 3. **Music Time**: Synced to tempo (rhythm games)
/// 4. **Fixed Time**: Consistent steps (physics)
///
/// **Bevy Timer Features**:
/// ```rust
/// Timer::from_seconds(5.0, TimerMode::Once)     // One-shot
/// Timer::from_seconds(5.0, TimerMode::Repeating) // Loops
/// timer.pause();                                  // Pause
/// timer.unpause();                                // Resume
/// timer.set_speed(2.0);                          // Fast forward
/// ```
///
/// **Musical Beat Timing**:
/// ```rust
/// // Sync to 120 BPM music
/// let beat_duration = 60.0 / 120.0;  // 0.5 seconds per beat
/// let bar_duration = beat_duration * 4;  // 2 seconds per bar
/// Timer::from_seconds(bar_duration, TimerMode::Repeating)
/// ```
#[derive(Resource)]
struct GameStateTimer(Timer);

/// Manages the game's soundtrack collection.
/// 
/// This resource stores handles to all music tracks that might be played.
/// Using a Vec allows for easy expansion - you could have multiple tracks
/// per state and select randomly, or sequence through them.
///
/// ## Audio Asset Management
///
/// **Handle<T> Memory Model**:
/// ```rust
/// // Handle is just a lightweight ID (8-16 bytes)
/// struct Handle<T> {
///     id: AssetId,
///     marker: PhantomData<T>,
/// }
/// // Actual audio data lives in Assets<AudioSource> resource
/// ```
///
/// **Advanced Soundtrack Systems**:
/// ```rust
/// struct SoundtrackPlayer {
///     // Multiple tracks per state
///     tracks: HashMap<GameState, Vec<TrackInfo>>,
///     // Currently playing for smooth transitions
///     current: Option<Handle<AudioSource>>,
///     // Playback history to avoid repetition
///     history: VecDeque<Handle<AudioSource>>,
///     // Gameplay-reactive parameters
///     intensity: f32,
///     urgency: f32,
/// }
/// 
/// struct TrackInfo {
///     handle: Handle<AudioSource>,
///     bpm: f32,
///     key: MusicalKey,
///     energy_level: f32,
/// }
/// ```
#[derive(Resource)]
struct SoundtrackPlayer {
    track_list: Vec<Handle<AudioSource>>,
}

impl SoundtrackPlayer {
    /// Creates a new soundtrack player with the given track list.
    /// 
    /// In a more complex game, you might want:
    /// - A HashMap<GameState, Vec<Handle<AudioSource>>> for multiple tracks per state
    /// - Metadata about each track (name, composer, mood)
    /// - Playlists that can be shuffled or played in sequence
    fn new(track_list: Vec<Handle<AudioSource>>) -> Self {
        Self { track_list }
    }
}

/// Marker component for audio entities that should fade in.
/// 
/// This is an example of the "tag component" pattern in ECS.
/// Components don't need data - they can just mark entities for processing.
/// The fade_in system will query for entities with this component.
///
/// ## Tag Components vs Data Components
///
/// **Tag Component** (Zero-Sized Type):
/// ```rust
/// #[derive(Component)]
/// struct IsPlayer;  // No data, just identification
/// struct IsFading;  // State marker
/// struct NeedsUpdate;  // Processing flag
/// ```
///
/// **Data Component Alternative**:
/// ```rust
/// #[derive(Component)]
/// struct FadeState {
///     direction: FadeDirection,
///     start_volume: f32,
///     target_volume: f32,
///     elapsed: f32,
///     duration: f32,
/// }
/// ```
///
/// **Performance Comparison**:
/// - Tag: 0 bytes, fast queries, less flexible
/// - Data: 20+ bytes, slower queries, more flexible
/// - Best: Use tags for simple states, data for complex
#[derive(Component)]
struct FadeIn;

/// Marker component for audio entities that should fade out.
/// 
/// Like FadeIn, this marks entities for the fade_out system to process.
/// Once fading completes, the entity is despawned to free resources.
///
/// ## Entity Lifecycle Management
///
/// **Audio Entity States**:
/// ```text
/// [Spawned] -> [FadeIn] -> [Playing] -> [FadeOut] -> [Despawned]
///                |                           |
///                v                           v
///            [Playing]                  [Despawned]
///          (if instant)               (if stopped)
/// ```
///
/// **Component Addition/Removal**:
/// ```rust
/// // Add component
/// commands.entity(entity).insert(FadeOut);
/// 
/// // Remove component
/// commands.entity(entity).remove::<FadeIn>();
/// 
/// // Replace component
/// commands.entity(entity)
///     .remove::<FadeIn>()
///     .insert(FadeOut);
/// ```
#[derive(Component)]
struct FadeOut;

/// Sets up the initial game resources and loads the soundtrack.
///
/// ## System Parameter Order
///
/// **Convention**: Resources before Commands
/// ```rust
/// fn system(
///     // Resources first (read-only)
///     asset_server: Res<AssetServer>,
///     time: Res<Time>,
///     // Mutable resources
///     mut score: ResMut<Score>,
///     // Queries
///     query: Query<&Transform>,
///     // Commands last (deferred changes)
///     mut commands: Commands,
/// ) {}
/// ```
fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    // Initialize the game in the default (Peaceful) state
    //
    // ## Resource Initialization Patterns
    //
    // **Options for initial state**:
    // ```rust
    // // Default trait (used here)
    // commands.insert_resource(GameState::default());
    // 
    // // Explicit value
    // commands.insert_resource(GameState::Peaceful);
    // 
    // // From config file
    // let config = load_config();
    // commands.insert_resource(config.initial_state);
    // ```
    commands.insert_resource(GameState::default());
    
    // Create a timer that switches states every 10 seconds.
    // TimerMode::Repeating means it automatically resets after finishing.
    //
    // ## Timer Precision and Musical Timing
    //
    // **Frame Time Variance**:
    // - 60 FPS target = 16.67ms per frame
    // - Actual: 15-18ms (varies)
    // - Timer accumulates exact time
    //
    // **Musical Synchronization**:
    // ```rust
    // // For 120 BPM music
    // let bpm = 120.0;
    // let beat_duration = 60.0 / bpm;  // 0.5 seconds
    // let bars = 8;  // Change every 8 bars
    // let transition_time = beat_duration * 4.0 * bars;  // 16 seconds
    // Timer::from_seconds(transition_time, TimerMode::Repeating)
    // ```
    commands.insert_resource(GameStateTimer(Timer::from_seconds(
        10.0,
        TimerMode::Repeating,
    )));

    // Load our music tracks. The paths are relative to the "assets" folder.
    // Using the turbofish syntax ::<AudioSource> explicitly tells Rust what type
    // of asset we're loading, though it could be inferred from context.
    //
    // ## Asset Loading Best Practices
    //
    // **Preloading Strategy**:
    // ```rust
    // // Load all music at startup (simple)
    // let tracks = vec![
    //     asset_server.load("music/menu.ogg"),
    //     asset_server.load("music/level1.ogg"),
    //     asset_server.load("music/boss.ogg"),
    // ];
    // 
    // // Load on demand (memory efficient)
    // fn enter_level(level: u32, asset_server: Res<AssetServer>) {
    //     let path = format!("music/level{}.ogg", level);
    //     asset_server.load(path);
    // }
    // ```
    //
    // **Audio File Naming Convention**:
    // - `bgm_`: Background music
    // - `sfx_`: Sound effects
    // - `vo_`: Voice over
    // - `amb_`: Ambient sounds
    let track_1 = asset_server.load::<AudioSource>("sounds/Mysterious acoustic guitar.ogg");
    let track_2 = asset_server.load::<AudioSource>("sounds/Epic orchestra music.ogg");
    
    // Store both tracks in our soundtrack player.
    // In a real game, you might load these from a configuration file
    // or have different track lists for different game areas.
    //
    // ## Music Asset Organization
    //
    // **Metadata-Driven Approach**:
    // ```rust
    // #[derive(Deserialize)]
    // struct MusicConfig {
    //     tracks: Vec<TrackConfig>,
    // }
    // 
    // #[derive(Deserialize)]
    // struct TrackConfig {
    //     path: String,
    //     state: GameState,
    //     bpm: f32,
    //     loop_point: Option<f32>,
    //     tags: Vec<String>,  // ["tense", "boss", "orchestral"]
    // }
    // ```
    let track_list = vec![track_1, track_2];
    commands.insert_resource(SoundtrackPlayer::new(track_list));
}

/// Monitors game state changes and triggers soundtrack transitions.
/// 
/// This system demonstrates Bevy's change detection feature. The `is_changed()`
/// method returns true only on the frame when the resource was modified.
fn change_track(
    mut commands: Commands,
    soundtrack_player: Res<SoundtrackPlayer>,
    // Query for all entities that have an AudioSink (actively playing audio)
    soundtrack: Query<Entity, With<AudioSink>>,
    game_state: Res<GameState>,
) {
    // Only run this logic when the game state actually changes
    if game_state.is_changed() {
        // Step 1: Fade out any currently playing music
        // This creates smooth transitions instead of abrupt cuts
        for track in soundtrack.iter() {
            commands.entity(track).insert(FadeOut);
        }

        // Step 2: Start playing the new track for the current state
        // Each track starts at zero volume and fades in
        match game_state.as_ref() {
            GameState::Peaceful => {
                commands.spawn((
                    // Create an audio player with the peaceful track
                    AudioPlayer(soundtrack_player.track_list.first().unwrap().clone()),
                    // Configure playback settings
                    PlaybackSettings {
                        // Loop the music indefinitely
                        mode: bevy::audio::PlaybackMode::Loop,
                        // Start silent (Volume::SILENT = 0.0 linear volume)
                        volume: Volume::SILENT,
                        // Use default values for other settings (speed, paused state)
                        ..default()
                    },
                    // Mark this entity for fade-in processing
                    FadeIn,
                ));
            }
            GameState::Battle => {
                commands.spawn((
                    // Use get(1) instead of first() to access the second track
                    AudioPlayer(soundtrack_player.track_list.get(1).unwrap().clone()),
                    PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Loop,
                        volume: Volume::SILENT,
                        ..default()
                    },
                    FadeIn,
                ));
            }
        }
    }
}

/// Duration of fade in/out effects in seconds.
/// 
/// 2 seconds provides a smooth transition that's noticeable but not too slow.
/// Faster fades (0.5-1.0s) work for action games, slower (3-5s) for atmospheric games.
///
/// ## Fade Curve Mathematics
///
/// **Linear Fade** (simple but unnatural):
/// ```text
/// volume(t) = start + (end - start) * (t / duration)
/// ```
///
/// **Equal Power Fade** (constant perceived loudness):
/// ```text
/// fade_out(t) = cos((t / duration) * π/2)
/// fade_in(t) = sin((t / duration) * π/2)
/// crossfade: out² + in² = 1 (constant power)
/// ```
///
/// **Logarithmic Fade** (matches human perception):
/// ```text
/// dB(t) = 20 * log10(linear_volume)
/// fade_db(t) = -60 + (60 * t / duration)
/// volume(t) = 10^(fade_db(t) / 20)
/// ```
///
/// ## Perceptual Considerations
///
/// **Why 2 seconds?**
/// - **0.1-0.5s**: Barely noticeable, feels like a glitch
/// - **0.5-1.0s**: Quick transition, maintains energy
/// - **1.0-2.0s**: Smooth, professional (our choice)
/// - **2.0-5.0s**: Dramatic, contemplative
/// - **5.0s+**: Special effect, scene transition
///
/// **Context-Dependent Timing**:
/// ```rust
/// const BATTLE_START_FADE: f32 = 0.5;   // Quick response
/// const BATTLE_END_FADE: f32 = 3.0;     // Gradual calm
/// const DEATH_FADE: f32 = 5.0;          // Dramatic
/// const MENU_FADE: f32 = 1.0;           // Snappy UI
/// ```
const FADE_TIME: f32 = 2.0;

/// Gradually increases volume for entities marked with FadeIn.
/// 
/// This system runs every frame, smoothly interpolating volume from 0 to 1.
///
/// ## Real-time Audio Processing
///
/// **Frame-based vs Sample-accurate**:
/// - Frame-based (this example): Updates every 16ms @ 60 FPS
/// - Sample-accurate: Updates every sample (0.02ms @ 48kHz)
/// - Good enough for: Volume, pan, basic effects
/// - Need sample-accurate for: Synthesis, precise timing
///
/// **Interpolation Quality**:
/// ```text
/// Update Rate  | Smoothness | Use Case
/// -------------|------------|----------
/// Per Frame    | Good       | Volume fades
/// Per 8 Samples| Better     | Filter sweeps  
/// Per Sample   | Perfect    | Synthesis
/// ```
fn fade_in(
    mut commands: Commands,
    // Query for audio sinks that need to fade in
    // The tuple gives us both the AudioSink component and the Entity ID
    //
    // ## Query Tuple Patterns
    //
    // **Common Audio Queries**:
    // ```rust
    // // Just the sink
    // Query<&AudioSink>
    // 
    // // Sink with position (3D audio)
    // Query<(&AudioSink, &Transform)>
    // 
    // // Multiple audio components
    // Query<(&AudioSink, &AudioEmitter, &AudioSettings)>
    // 
    // // With entity for commands
    // Query<(Entity, &AudioSink, &FadeSettings)>
    // ```
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeIn>>,
    time: Res<Time>,
) {
    for (mut audio, entity) in audio_sink.iter_mut() {
        // Get the current volume (could be partially faded in already)
        let current_volume = audio.volume();
        
        // fade_towards is a Bevy helper that smoothly interpolates between volumes.
        // We pass the target volume (1.0 = full) and the interpolation amount.
        // time.delta_secs() / FADE_TIME gives us the fraction to move this frame.
        //
        // ## Interpolation Mathematics
        //
        // **Linear Interpolation (lerp)**:
        // ```rust
        // fn lerp(start: f32, end: f32, t: f32) -> f32 {
        //     start + (end - start) * t
        // }
        // ```
        //
        // **Fade Progress Calculation**:
        // - Delta time: 0.016s (60 FPS)
        // - Fade time: 2.0s
        // - Progress per frame: 0.016 / 2.0 = 0.008 (0.8%)
        // - Frames to complete: 2.0 / 0.016 = 125 frames
        //
        // **Frame Rate Independence**:
        // - 30 FPS: 0.033s delta, 0.0165 progress, 60 frames
        // - 60 FPS: 0.016s delta, 0.008 progress, 125 frames
        // - 144 FPS: 0.007s delta, 0.0035 progress, 286 frames
        // - Result: Same 2 second fade regardless of framerate
        audio.set_volume(
            current_volume.fade_towards(Volume::Linear(1.0), time.delta_secs() / FADE_TIME),
        );
        
        // Check if we've reached full volume
        //
        // ## Floating Point Precision Issues
        //
        // **Why >= instead of ==?**
        // ```rust
        // // Problematic
        // if volume == 1.0 {  // Might never be exactly 1.0!
        // 
        // // Better
        // if volume >= 1.0 {  // Catches overshoot
        // 
        // // Best
        // if (volume - 1.0).abs() < 0.001 {  // Epsilon comparison
        // ```
        //
        // **Accumulation Error Example**:
        // - Start: 0.0
        // - Add 0.008 per frame (125 times)
        // - Expected: 1.0
        // - Actual: 0.9999999 or 1.0000001
        if audio.volume().to_linear() >= 1.0 {
            // Ensure we're exactly at 1.0 (not slightly over due to float precision)
            //
            // ## Volume Clamping
            //
            // **Why clamp to exact values?**
            // - Prevents accumulation errors
            // - Ensures consistent final state
            // - Some audio APIs reject values > 1.0
            // - Makes debugging easier
            audio.set_volume(Volume::Linear(1.0));
            // Remove the FadeIn component - we're done fading
            //
            // ## Component Lifecycle
            //
            // **Remove vs Despawn**:
            // ```rust
            // // Remove component (entity continues)
            // commands.entity(entity).remove::<FadeIn>();
            // 
            // // Remove multiple
            // commands.entity(entity)
            //     .remove::<(FadeIn, Temporary)>();
            // 
            // // Despawn entity (removes all)
            // commands.entity(entity).despawn();
            // ```
            commands.entity(entity).remove::<FadeIn>();
        }
    }
}

/// Gradually decreases volume for entities marked with FadeOut, then despawns them.
/// 
/// This is the mirror of fade_in, but with entity cleanup at the end.
fn fade_out(
    mut commands: Commands,
    mut audio_sink: Query<(&mut AudioSink, Entity), With<FadeOut>>,
    time: Res<Time>,
) {
    for (mut audio, entity) in audio_sink.iter_mut() {
        let current_volume = audio.volume();
        // Fade towards silence (0.0 volume)
        audio.set_volume(
            current_volume.fade_towards(Volume::Linear(0.0), time.delta_secs() / FADE_TIME),
        );
        
        // Once we reach silence, clean up the entity
        if audio.volume().to_linear() <= 0.0 {
            // Despawn the entire entity, freeing all resources
            commands.entity(entity).despawn();
        }
    }
}

/// Simulates game state changes by toggling between Peaceful and Battle every 10 seconds.
/// 
/// In a real game, this would be replaced by actual gameplay logic:
/// - Detecting enemies in range
/// - Entering/exiting combat zones
/// - Story triggers and cutscenes
///
/// ## Music-Driven Game State
///
/// **Real Combat Detection Examples**:
/// ```rust
/// fn detect_combat(
///     player: Query<&Transform, With<Player>>,
///     enemies: Query<&Transform, With<Enemy>>,
///     mut state: ResMut<GameState>,
/// ) {
///     let player_pos = player.single().translation;
///     
///     // Check if any enemy is within aggro range
///     let in_combat = enemies.iter().any(|enemy| {
///         player_pos.distance(enemy.translation) < 50.0
///     });
///     
///     // Update state with hysteresis
///     match (*state, in_combat) {
///         (GameState::Peaceful, true) => *state = GameState::Battle,
///         (GameState::Battle, false) => {
///             // Delay transition to prevent flickering
///             // Could use a timer here
///         }
///         _ => {}
///     }
/// }
/// ```
///
/// **Music Intensity Layers**:
/// ```rust
/// enum CombatIntensity {
///     None,        // Peaceful exploration
///     Ambient,     // Enemies nearby but unaware
///     Alert,       // Enemies searching
///     Combat,      // Active fighting
///     Boss,        // Boss encounter
/// }
/// ```
fn cycle_game_state(
    mut timer: ResMut<GameStateTimer>,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
    // Advance the timer by the time elapsed since the last frame
    timer.0.tick(time.delta());
    
    // Check if the timer just completed a cycle
    if timer.0.just_finished() {
        // Toggle between states
        // as_ref() gives us a reference to the enum value for matching
        match game_state.as_ref() {
            GameState::Battle => *game_state = GameState::Peaceful,
            GameState::Peaceful => *game_state = GameState::Battle,
        }
        // The timer automatically resets because we created it with TimerMode::Repeating
    }
}
