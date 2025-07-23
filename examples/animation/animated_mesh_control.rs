//! Plays animations from a skinned glTF with keyboard controls.
//!
//! This example builds on the basic animated_mesh example by adding interactive controls.
//! Think of an animation player like a music player - you can play, pause, speed up,
//! slow down, seek through the timeline, and switch between different tracks (animations).
//! We'll also explore animation transitions - smoothly blending from one animation to another!
//!
//! ## Animation Theory: State Machines and Transitions
//!
//! Animation state machines are the backbone of character control:
//! - **States**: Each animation (idle, walk, run, jump) is a state
//! - **Transitions**: Rules for moving between states (walk→run when speed > threshold)
//! - **Blend Trees**: Mixing multiple animations based on parameters (walk/run blend)
//!
//! Transition mathematics:
//! ```
//! output = animation_a * (1 - blend_factor) + animation_b * blend_factor
//! ```
//! Where blend_factor goes from 0 to 1 over the transition duration.
//!
//! ## Game Feel Context: Responsive Animation Control
//!
//! Great animation control creates the illusion of direct control:
//! - **Instant Feedback**: <100ms response time feels immediate
//! - **Smooth Transitions**: 200-300ms blends feel natural
//! - **Anticipation**: Start transition animations before the action
//! - **Follow-through**: Let animations settle naturally
//!
//! Common game feel techniques:
//! - **Animation Canceling**: Fighting games let you interrupt attacks
//! - **Input Buffering**: Queue the next move during current animation
//! - **Blend Masking**: Only blend certain bones (upper body aim while running)
//! - **Time Dilation**: Slow-mo for dramatic moments
//!
//! ## Performance Optimization: Animation Systems
//!
//! Animation is often the most expensive part of character rendering:
//! - **Bone Matrix Calculation**: O(n) for n bones per character
//! - **Vertex Skinning**: O(v*b) for v vertices and b bones per vertex
//! - **Memory Bandwidth**: Bone matrices updated every frame
//!
//! Optimization strategies:
//! 1. **LOD (Level of Detail)**: Reduce bone count at distance
//! 2. **Animation Culling**: Don't update off-screen characters
//! 3. **Pose Caching**: Reuse poses for multiple characters
//! 4. **GPU Skinning**: Move vertex calculations to GPU
//! 5. **Animation Compression**: Quantize rotations, remove redundant keys
//!
//! ## Real-World Applications: AAA Animation Systems
//!
//! Modern games use sophisticated animation systems:
//! - **Uncharted 4**: 500+ animations per character, motion matching
//! - **Horizon Zero Dawn**: Procedural animation for robot creatures
//! - **Red Dead Redemption 2**: Layered animation for realistic NPCs
//! - **The Last of Us Part II**: Per-muscle contraction simulation
//!
//! Industry techniques:
//! - **Motion Matching**: Find best animation frame from motion database
//! - **IK (Inverse Kinematics)**: Adjust limbs to reach targets
//! - **Ragdoll Blending**: Mix physics with animation
//! - **Facial Rigs**: 50+ bones just for facial expressions
//!
//! ## Advanced Techniques: Beyond Keyframes
//!
//! Modern animation goes beyond traditional keyframing:
//! 1. **Procedural Animation**: Generate motion algorithmically
//! 2. **Physics-Based**: Simulate muscles, tendons, and joints
//! 3. **Machine Learning**: Neural networks learn natural motion
//! 4. **Motion Capture**: Real actor performances
//! 5. **Crowd Simulation**: Thousands of unique animations
//!
//! ## Common Issues and Solutions
//!
//! 1. **Foot Sliding**: Character moves but feet don't match
//!    - Solution: Root motion or IK foot planting
//!
//! 2. **Animation Popping**: Sudden jumps between animations
//!    - Solution: Proper transition curves and blend times
//!
//! 3. **Desynchronization**: Animation doesn't match game state
//!    - Solution: Animation events and state validation
//!
//! 4. **Memory Bloat**: Too many animation clips
//!    - Solution: Animation compression and streaming

use std::{f32::consts::PI, time::Duration};

use bevy::{animation::RepeatAnimation, pbr::CascadeShadowConfigBuilder, prelude::*};

// Path to our animated fox model with multiple animations
const FOX_PATH: &str = "models/animated/Fox.glb";

fn main() {
    App::new()
        // Bright ambient light for good visibility
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // These systems run every frame:
        .add_systems(Update, setup_scene_once_loaded) // Watches for the fox to load
        .add_systems(Update, keyboard_control)         // Handles user input
        .run();
}

// A resource to store our animation data globally
// Resources in Bevy are like global variables that systems can access
//
// ## Rust Pattern: Resource vs Component
// - Resources: Global data, one instance per type (like this Animations struct)
// - Components: Per-entity data, many instances (like Transform, AnimationPlayer)
// Use resources for game-wide state, components for entity-specific data
//
// ## Memory Consideration
// Vec<AnimationNodeIndex> stores indices, not the actual animation data
// This is memory-efficient - indices are typically 4-8 bytes each
// The actual animation data lives in the Assets<AnimationClip> storage
#[derive(Resource)]
struct Animations {
    // Indices to each animation in our graph - like bookmarks in a video file
    animations: Vec<AnimationNodeIndex>,
    // Handle to the animation graph that contains all our clips
    graph_handle: Handle<AnimationGraph>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // BUILD THE ANIMATION GRAPH
    // Unlike the simple example, we're loading ALL animations from the fox model
    // Animation graphs can contain multiple clips and define how they connect
    //
    // ## Animation Graph Architecture
    // Animation graphs are directed acyclic graphs (DAGs) that define:
    // - Nodes: Animation clips, blend nodes, or state machines
    // - Edges: Transitions with blend parameters
    // - Weights: How much each animation contributes to the final pose
    //
    // ## Rust Pattern: Array vs Vec
    // We use an array [T; N] here because we know exactly 3 animations at compile time
    // Arrays are stack-allocated and have better cache locality than Vec<T>
    let (graph, node_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(FOX_PATH)), // "Run"
        asset_server.load(GltfAssetLabel::Animation(1).from_asset(FOX_PATH)), // "Walk"
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(FOX_PATH)), // "Survey"
    ]);

    // Store the graph as a resource - we can't attach it to the fox yet because
    // the fox entity doesn't exist until the scene loads!
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices, // Save the indices so we can switch between animations
        graph_handle,
    });

    // SCENE SETUP - Camera, ground, and lighting
    
    // Camera positioned for a nice view
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(100.0, 100.0, 150.0).looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Directional light (sun-like)
    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));

    // Spawn the fox model
    // SceneRoot will automatically create all the child entities when loaded
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOX_PATH)),
    ));

    // UI INSTRUCTIONS
    // Display controls in the top-left corner
    // concat! macro joins string literals at compile time - no runtime cost!
    commands.spawn((
        Text::new(concat!(
            "space: play / pause\n",
            "up / down: playback speed\n",
            "left / right: seek\n",
            "1-3: play N times\n",
            "L: loop forever\n",
            "return: change animation\n",
        )),
        Node {
            position_type: PositionType::Absolute, // Position relative to window
            top: Val::Px(12.0),                   // 12 pixels from top
            left: Val::Px(12.0),                  // 12 pixels from left
            ..default()
        },
    ));
}

// This system waits for the fox to load, then sets up its animation player
// The Added<AnimationPlayer> filter is key - it only runs when a NEW AnimationPlayer appears!
//
// ## Bevy Pattern: Change Detection Filters
// Added<T> is one of several change detection filters:
// - Added<T>: Component was added this frame or last frame
// - Changed<T>: Component was mutated this frame or last frame
// - Without<T>: Entity doesn't have this component
// These filters make systems efficient - only process what changed!
//
// ## Game Architecture: Deferred Initialization
// We can't set up animations in setup() because the model isn't loaded yet
// This pattern of "wait for asset, then initialize" is common in games
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        // ANIMATION TRANSITIONS
        // This component manages smooth blending between animations
        // Think of it as a DJ's mixing board - it can fade out one track while fading in another
        //
        // ## Animation Blending Mathematics
        // Cross-fade blending: result = anim_a * (1-t) + anim_b * t
        // Where t goes from 0 to 1 over the transition duration
        // More complex blends use curves: ease-in, ease-out, ease-in-out
        let mut transitions = AnimationTransitions::new();

        // Start the first animation (Run) immediately
        // Duration::ZERO means no transition time - it starts instantly
        // The transitions component MUST be used to start animations when it exists!
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat(); // Loop the animation forever

        // Attach our animation data to the entity that has the player
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions); // Don't forget to add the transitions component!
    }
}

// Handles all keyboard input for controlling animations
// Local<T> is like a static variable that's unique to this system - it remembers state between frames
//
// ## Rust Pattern: Local<T> for System State
// Local<T> provides per-system persistent storage:
// - Initialized with Default::default() on first run
// - Lives as long as the system is registered
// - Each system instance gets its own Local storage
// - Thread-safe because systems run on one thread at a time
//
// ## Input Handling Best Practices
// - just_pressed(): Triggers once when key goes down
// - pressed(): Triggers every frame while held
// - just_released(): Triggers once when key goes up
// Use just_pressed() for discrete actions, pressed() for continuous
fn keyboard_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>, // Tracks which animation we're playing (0, 1, or 2)
) {
    for (mut player, mut transitions) in &mut animation_players {
        // Get the currently playing animation
        // playing_animations() returns an iterator of (index, ActiveAnimation) pairs
        // We use 'let-else' pattern - if no animation is playing, skip this entity
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        // SPACE: Toggle play/pause
        if keyboard_input.just_pressed(KeyCode::Space) {
            // Get mutable access to the currently playing animation
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            // Toggle between paused and playing states
            if playing_animation.is_paused() {
                playing_animation.resume();
            } else {
                playing_animation.pause();
            }
        }

        // UP ARROW: Speed up by 20%
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 1.2); // 1.2x = 20% faster
        }

        // DOWN ARROW: Slow down by 20%
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let speed = playing_animation.speed();
            playing_animation.set_speed(speed * 0.8); // 0.8x = 20% slower
        }

        // LEFT ARROW: Seek backward 0.1 seconds
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time(); // Current position in the animation
            playing_animation.seek_to(elapsed - 0.1);    // Jump back 0.1 seconds
        }

        // RIGHT ARROW: Seek forward 0.1 seconds
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            let elapsed = playing_animation.seek_time();
            playing_animation.seek_to(elapsed + 0.1);    // Jump forward 0.1 seconds
        }

        // ENTER: Cycle through animations with smooth transitions
        if keyboard_input.just_pressed(KeyCode::Enter) {
            // Increment and wrap around using modulo
            // If we have 3 animations: 0 -> 1 -> 2 -> 0 -> 1 -> ...
            *current_animation = (*current_animation + 1) % animations.animations.len();

            // Start the new animation with a 250ms transition
            // During this transition, both animations play and blend together!
            transitions
                .play(
                    &mut player,
                    animations.animations[*current_animation],
                    Duration::from_millis(250), // Quarter second blend
                )
                .repeat();
        }

        // NUMBER KEYS 1-3: Play animation N times
        // This demonstrates different repeat modes
        if keyboard_input.just_pressed(KeyCode::Digit1) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(1)) // Play once then stop
                .replay();                             // Start from beginning
        }

        if keyboard_input.just_pressed(KeyCode::Digit2) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(2)) // Play twice then stop
                .replay();
        }

        if keyboard_input.just_pressed(KeyCode::Digit3) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation
                .set_repeat(RepeatAnimation::Count(3)) // Play three times then stop
                .replay();
        }

        // L KEY: Loop forever
        if keyboard_input.just_pressed(KeyCode::KeyL) {
            let playing_animation = player.animation_mut(playing_animation_index).unwrap();
            playing_animation.set_repeat(RepeatAnimation::Forever); // Infinite loop mode
        }
    }
}
