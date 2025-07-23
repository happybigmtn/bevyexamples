//! Demonstrate how to use animation events.
//!
//! Animation events are like alarm clocks embedded in your animations - they ring at
//! specific times to trigger actions! Just as a movie director might cue sound effects
//! or lighting changes at precise moments, animation events let you synchronize game
//! logic with animation playback. Perfect for footstep sounds, particle effects,
//! damage dealing, or any action that needs perfect timing with visual animation.
//!
//! ## Animation Theory: Timeline-Based Events
//!
//! Animation events solve critical synchronization problems:
//! - **Frame Independence**: Events fire at exact times, not frame counts
//! - **Decoupling**: Animation data doesn't know about game logic
//! - **Precision**: Sub-frame accuracy for perfect timing
//! - **Flexibility**: Same animation can trigger different events in different contexts
//!
//! Event timing considerations:
//! 1. **Anticipation**: Fire slightly early for responsive feel
//! 2. **Network Sync**: Events must be deterministic for multiplayer
//! 3. **Audio Sync**: Account for audio latency (~40ms typical)
//! 4. **Visual Sync**: Match the exact frame of visual change
//!
//! ## Game Feel Context: Rhythm and Flow
//!
//! Events create the "beat" of gameplay:
//! - **Attack Games**: Hit confirms, combo windows, parry timing
//! - **Rhythm Games**: Note hits, perfect timing bonuses
//! - **Platformers**: Jump dust, landing impacts, ledge grabs
//! - **RPGs**: Spell casts, buff applications, damage numbers
//!
//! This example demonstrates:
//! - **Text as Visual Feedback**: Words appear/change on cue
//! - **Color for Emotion**: Blue = hello, Red = goodbye
//! - **Fade for Continuity**: Smooth transitions between events
//!
//! ## Performance Optimization: Event Systems
//!
//! Event system performance tips:
//! 1. **Event Pooling**: Reuse event objects to avoid allocation
//! 2. **Sparse Storage**: Only check events near current time
//! 3. **Bucketing**: Group events by time for efficient lookup
//! 4. **Early Out**: Skip checking once past all events
//! 5. **LOD**: Fewer events for distant/unimportant animations
//!
//! Memory considerations:
//! - Events add overhead to animation clips
//! - Consider event references vs embedded data
//! - Compress event data when possible
//!
//! ## Real-World Applications: Event-Driven Games
//!
//! AAA examples:
//! - **Devil May Cry**: Combat rated by event timing precision
//! - **Guitar Hero**: Entire game based on animation events
//! - **Dark Souls**: I-frames triggered by roll animation events
//! - **Overwatch**: Ultimate voice lines and effects perfectly synced
//!
//! Common event types:
//! 1. **Audio**: Footsteps, voice, weapon sounds
//! 2. **Visual**: Particles, screen effects, UI updates
//! 3. **Gameplay**: Damage, healing, state changes
//! 4. **Physics**: Impulses, collider changes, ragdoll triggers
//! 5. **AI**: Alert signals, behavior changes
//!
//! ## Advanced Techniques: Event Systems
//!
//! 1. **Event Curves**: Continuous values instead of discrete events
//! 2. **Event Conditions**: Only fire if game state matches
//! 3. **Event Priority**: Handle important events first
//! 4. **Event Batching**: Process similar events together
//! 5. **Predictive Events**: Pre-trigger for online games
//!
//! ## Common Issues and Solutions
//!
//! 1. **Missed Events**: Animation seeks past event time
//!    - Solution: Check for passed events when seeking
//!
//! 2. **Double Events**: Same event fires multiple times
//!    - Solution: Track last fired time per event
//!
//! 3. **Event Spam**: Too many events overwhelming systems
//!    - Solution: Rate limiting and event priorities
//!
//! 4. **Timing Drift**: Events gradually desync from visuals
//!    - Solution: Use animation time, not wall clock time

use bevy::{
    color::palettes::css::{ALICE_BLUE, BLACK, CRIMSON},
    core_pipeline::bloom::Bloom,
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // EVENT REGISTRATION - Register our custom animation event
        .add_event::<MessageEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, animate_text_opacity)  // Continuous fade effect
        // EVENT OBSERVER - Listen for animation events and respond
        .add_observer(edit_message)  // Triggered when animation events fire
        .run();
}

// UI MARKER - Tag for our animated text element
#[derive(Component)]
struct MessageText;

// CUSTOM ANIMATION EVENT - Data that gets fired at specific animation times
//
// ## Rust Pattern: Custom Event Types
// Events in Bevy must implement Event (usually via #[derive(Event)])
// Clone is needed because events may be sent to multiple systems
// You can put ANY data in events - from simple flags to complex structs
//
// ## Design Consideration
// Keep events lightweight - they're cloned when sent
// For heavy data, consider storing it elsewhere and passing an ID
#[derive(Event, Clone)]
struct MessageEvent {
    value: String,  // What text to display
    color: Color,   // What color to make it
}

// EVENT HANDLER - Responds when animation events fire
//
// ## Bevy Pattern: Observer Functions
// Observers are Bevy's reactive programming model
// They automatically run when specific events are triggered
// More efficient than polling every frame!
//
// ## Rust Pattern: Single Query
// Single<T> ensures exactly one entity matches
// It panics if 0 or 2+ entities match - perfect for unique UI elements
fn edit_message(
    trigger: Trigger<MessageEvent>,  // The event that was fired
    text: Single<(&mut Text2d, &mut TextColor), With<MessageText>>,
) {
    let (mut text, mut color) = text.into_inner();
    // UPDATE TEXT AND COLOR - Apply the event's data to the UI
    text.0 = trigger.event().value.clone();  // Change the message
    color.0 = trigger.event().color;         // Change the color
    
    // ## Visual Feedback Theory
    // Instant color change + gradual fade creates:
    // 1. Clear event timing (sudden appearance)
    // 2. Smooth continuity (gradual disappearance)
    // 3. Anticipation for next event (empty space)
}

fn setup(
    mut commands: Commands,
    mut animations: ResMut<Assets<AnimationClip>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // DRAMATIC SETUP - Dark background with bloom for glowing text effect
    // Camera
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(BLACK.into()),  // Black background
            ..Default::default()
        },
        Bloom {
            intensity: 0.4,     // Soft glow effect
            ..Bloom::NATURAL
        },
    ));

    // THE ANIMATED TEXT TARGET - What our events will control
    // The text that will be changed by animation events.
    commands.spawn((
        MessageText,              // Marker component for targeting
        Text2d::default(),        // Empty text initially
        TextFont {
            font_size: 119.0,     // Large, dramatic text
            ..default()
        },
        TextColor(Color::NONE),   // Invisible initially (fully transparent)
    ));

    // ANIMATION TIMELINE CREATION - Build our sequence of timed events
    // Create a new animation clip.
    let mut animation = AnimationClip::default();

    // SET TOTAL DURATION - Define how long the complete animation runs
    // This is only necessary if you want the duration of the
    // animation to be longer than the last event in the clip.
    animation.set_duration(2.0);  // 2 seconds total

    // SCHEDULE THE EVENTS - Add alarm clocks at specific times
    // Add events at the specified time.
    //
    // ## Event Timing Design
    // Events at 0.0 and 1.0 seconds create a rhythm:
    // - 0.0s: "HELLO" appears (friendly greeting)
    // - 0.0-1.0s: Text fades out gradually
    // - 1.0s: "BYE" appears (emotional farewell)
    // - 1.0-2.0s: Text fades out again
    // - 2.0s: Loop restarts
    //
    // ## Color Psychology
    // - ALICE_BLUE: Cool, calm, welcoming
    // - CRIMSON: Warm, urgent, departure
    // The color shift emphasizes the emotional transition
    animation.add_event(
        0.0,  // At the very start (0 seconds)
        MessageEvent {
            value: "HELLO".into(),
            color: ALICE_BLUE.into(),  // Light blue greeting
        },
    );
    animation.add_event(
        1.0,  // Halfway through (1 second)
        MessageEvent {
            value: "BYE".into(),
            color: CRIMSON.into(),     // Red farewell
        },
    );

    // ANIMATION SYSTEM SETUP - Connect our event timeline to the animation player
    // Create the animation graph.
    let (graph, animation_index) = AnimationGraph::from_clip(animations.add(animation));
    let mut player = AnimationPlayer::default();
    player.play(animation_index).repeat();  // Loop the animation forever

    // SPAWN THE ANIMATION CONTROLLER - Entity that plays our timed events
    commands.spawn((AnimationGraphHandle(graphs.add(graph)), player));
}

// CONTINUOUS FADE EFFECT - Makes text gradually disappear
// Slowly fade out the text opacity.
//
// ## Animation Layering
// This demonstrates a powerful pattern: combining discrete events with continuous animation
// - Events provide the "beats" (text changes)
// - Continuous animation provides the "flow" (fading)
// Together they create a breathing, living interface
//
// ## Performance Note
// This runs every frame for every TextColor in the scene
// For many UI elements, consider:
// - Batching updates
// - Using animation curves instead
// - Culling off-screen elements
fn animate_text_opacity(mut colors: Query<&mut TextColor>, time: Res<Time>) {
    for mut color in &mut colors {
        let a = color.0.alpha();  // Current transparency level
        // GRADUAL FADE - Reduce opacity each frame
        color.0.set_alpha(a - time.delta_secs());  // Fade at 1 alpha per second
        // This creates the effect where text appears bright when events fire,
        // then slowly fades to invisible until the next event
        
        // ## Mathematical Note
        // Fading at 1.0 alpha/second means:
        // - Full opacity (1.0) fades to invisible (0.0) in 1 second
        // - This matches our event timing perfectly
        // - Text is fully faded by the time the next event fires
    }
}
