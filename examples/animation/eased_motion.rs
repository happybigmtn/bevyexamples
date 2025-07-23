//! Demonstrates the application of easing curves to animate a transition.
//!
//! Easing functions make animations feel natural! Instead of moving at constant speed
//! (which looks robotic), objects can accelerate and decelerate smoothly.
//! Think of a car: it doesn't instantly reach top speed - it eases into motion.
//! This example shows a cube moving back and forth with smooth acceleration,
//! while also rotating with a bouncy "elastic" effect.
//!
//! ## Animation Theory: Easing Functions Mathematics
//!
//! Easing functions map time to progress non-linearly:
//! - **Linear**: f(t) = t (constant speed, robotic)
//! - **Quadratic**: f(t) = t² (gentle acceleration)
//! - **Cubic**: f(t) = t³ (smooth start/stop)
//! - **Elastic**: Overshoots with spring physics
//! - **Bounce**: Simulates gravity and restitution
//!
//! The easing equation for CubicInOut:
//! ```text
//! f(t) = t < 0.5 ? 4t³ : 1 - pow(-2t + 2, 3) / 2
//! ```
//!
//! Key principles:
//! 1. **Ease In**: Slow start, fast end (acceleration)
//! 2. **Ease Out**: Fast start, slow end (deceleration)
//! 3. **Ease InOut**: Slow start AND end (most natural)
//! 4. **Overshoot**: Goes past target, returns (anticipation)
//!
//! ## Game Feel Context: Motion and Emotion
//!
//! Easing creates emotional responses:
//! - **Linear**: Mechanical, artificial, tense
//! - **Ease InOut**: Natural, comfortable, expected
//! - **Bounce**: Playful, energetic, cartoony
//! - **Elastic**: Surprising, dynamic, responsive
//!
//! Common uses in games:
//! - UI elements sliding in (ease out)
//! - Character jumps (ease in + gravity)
//! - Camera movements (ease in-out)
//! - Menu transitions (various effects)
//!
//! ## Performance Optimization: Easing Calculations
//!
//! Optimization strategies:
//! 1. **Lookup Tables**: Pre-calculate easing values
//! 2. **Approximations**: Use simpler functions when possible
//! 3. **GPU Curves**: Evaluate in vertex/fragment shaders
//! 4. **Caching**: Store computed values for static animations
//! 5. **LOD**: Use linear for distant/fast objects
//!
//! Performance tips:
//! - Avoid complex easing (Elastic, Bounce) on many objects
//! - Batch similar animations together
//! - Use animation compression for long curves
//!
//! ## Real-World Applications: Industry Standards
//!
//! Famous easing in games:
//! - **Mario Jump**: Custom gravity curve, not physics
//! - **Angry Birds**: Elastic slingshot release
//! - **Monument Valley**: Ease in-out for all movements
//! - **Alto's Adventure**: Smooth ease-out for landing
//!
//! CSS/Web standards adopted by games:
//! - ease = cubic-bezier(0.25, 0.1, 0.25, 1)
//! - ease-in-out = cubic-bezier(0.42, 0, 0.58, 1)
//! - Material Design uses specific easing curves
//!
//! ## Advanced Techniques: Custom Easing
//!
//! 1. **Bezier Curves**: Define custom easing with control points
//! 2. **Spring Physics**: Damped harmonic oscillator
//! 3. **Verlet Integration**: Velocity-less physics simulation
//! 4. **Procedural Easing**: Generate curves from parameters
//! 5. **Composite Easing**: Chain multiple functions
//!
//! ## Common Issues and Solutions
//!
//! 1. **Jarring Transitions**: Easing doesn't match previous motion
//!    - Solution: Match derivatives at transition points
//!
//! 2. **Feels Sluggish**: Ease in-out too extreme
//!    - Solution: Use gentler curves or reduce duration
//!
//! 3. **Overshoot Problems**: Elastic goes out of bounds
//!    - Solution: Clamp values or use gentler elastic
//!
//! 4. **Performance**: Too many easing calculations
//!    - Solution: Bake animations or use simpler curves

use std::f32::consts::FRAC_PI_2; // π/2 radians = 90 degrees

use bevy::{
    animation::{animated_field, AnimationTarget, AnimationTargetId},
    color::palettes::css::{ORANGE, SILVER}, // CSS color constants
    math::vec3, // Shorthand for Vec3::new()
    prelude::*,
};

// ## Rust Pattern: Selective Imports
// We import only what we need from bevy::animation
// This makes code clearer about dependencies
// animated_field! is a macro that generates property accessors

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup) // Create scene and start animation
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
) {
    // CREATE THE ANIMATION
    // We use a helper struct to organize all the animation data
    //
    // ## Design Pattern: Destructuring Assignment
    // This Rust pattern unpacks a struct into local variables
    // More readable than: let info = AnimationInfo::create(...);
    // Then using info.target_name, info.target_id, etc.
    let AnimationInfo {
        target_name: animation_target_name,    // Name to identify what we're animating
        target_id: animation_target_id,        // Unique ID derived from the name
        graph: animation_graph,                // The animation graph asset
        node_index: animation_node_index,      // Which node to play in the graph
    } = AnimationInfo::create(&mut animation_graphs, &mut animation_clips);

    // Set up the animation player to play our animation on loop
    let mut animation_player = AnimationPlayer::default();
    animation_player.play(animation_node_index).repeat(); // Play forever

    // ANIMATED CUBE
    // Create a cube that will move and rotate
    //
    // ## Entity Component Composition
    // Bevy uses composition over inheritance:
    // - Mesh3d: Visual representation
    // - MeshMaterial3d: Surface properties
    // - Transform: Position, rotation, scale
    // - Name: Human-readable identifier
    // - AnimationPlayer: Controls playback
    // - AnimationGraphHandle: Links to animation data
    //
    // ## Why Orange?
    // High contrast against silver ground
    // Warm color draws attention to the animated object
    let cube_entity = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_length(2.0))), // 2x2x2 cube
            MeshMaterial3d(materials.add(Color::from(ORANGE))),
            Transform::from_translation(vec3(-6., 2., 0.)), // Start position: left side
            animation_target_name,     // Links this entity to the animation
            animation_player,          // The animation playback controller
            AnimationGraphHandle(animation_graph), // Reference to our animation data
        ))
        .id();

    // Add the animation target component separately
    // This tells the animation system which entity to animate
    //
    // ## Bevy Animation Architecture
    // AnimationTarget creates the link between:
    // 1. The animation data (curves targeting an ID)
    // 2. The entity to animate (this cube)
    // 3. The player controlling timing
    //
    // The self-reference (player: cube_entity) means
    // the animation player is on the same entity it's animating
    commands.entity(cube_entity).insert(AnimationTarget {
        id: animation_target_id,
        player: cube_entity, // Self-reference: the player is on the same entity
    });

    // LIGHTING
    // Point light with shadows for depth perception
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000., // Very bright
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(8., 16., 8.), // Upper right corner
    ));

    // GROUND PLANE
    // Large silver plane to show shadows and give spatial reference
    //
    // ## Visual Design: Shadow Importance
    // Shadows provide crucial depth cues:
    // - Show height above ground
    // - Indicate light direction
    // - Create sense of weight/contact
    // Without shadows, objects appear to float!
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50., 50.))),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
    ));

    // CAMERA
    // Positioned to see the cube's full motion path
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 6., 12.) // Centered, elevated, pulled back
            .looking_at(Vec3::new(0., 1.5, 0.), Vec3::Y), // Look at cube height
    ));
}

// Helper struct to organize all the animation-related data
//
// ## Rust Pattern: Builder Return Type
// Instead of returning a tuple (Name, AnimationTargetId, Handle, NodeIndex)
// we use a struct for clarity and type safety
// This pattern is common in Rust for functions that return multiple values
struct AnimationInfo {
    // The name identifying what we're animating
    target_name: Name,
    // Unique ID for the animation target (derived from name)
    target_id: AnimationTargetId,
    // Handle to the animation graph asset
    graph: Handle<AnimationGraph>,
    // Index of the animation node in the graph
    node_index: AnimationNodeIndex,
}

impl AnimationInfo {
    // Creates the animation programmatically (instead of loading from a file)
    fn create(
        animation_graphs: &mut Assets<AnimationGraph>,
        animation_clips: &mut Assets<AnimationClip>,
    ) -> AnimationInfo {
        // Set up the animation target identification
        let animation_target_name = Name::new("Cube");
        let animation_target_id = AnimationTargetId::from_name(&animation_target_name);

        // Create an empty animation clip to add curves to
        let mut animation_clip = AnimationClip::default();

        // TRANSLATION ANIMATION
        // Define the time domain: 0 to 3 seconds for one direction
        let animation_domain = interval(0.0, 3.0).unwrap();

        // Create an easing curve for smooth movement
        //
        // ## Cubic In-Out Mathematics
        // This creates an S-curve velocity profile:
        // - Starts slow (acceleration from rest)
        // - Reaches maximum speed at midpoint
        // - Ends slow (deceleration to rest)
        //
        // The math: t' = t < 0.5 ? 4t³ : 1 - (-2t + 2)³ / 2
        // This gives C¹ continuity (smooth velocity)
        let translation_curve = EasingCurve::new(
            vec3(-6., 2., 0.), // Start position (left)
            vec3(6., 2., 0.),  // End position (right)
            EaseFunction::CubicInOut, // Smooth acceleration and deceleration
        )
        // Easing curves work on [0,1], so we stretch it to our 3-second duration
        // This is called "reparametrization" - changing the input domain
        .reparametrize_linear(animation_domain)
        .expect("this curve has bounded domain, so this should never fail")
        // ping_pong() makes it reverse direction, creating a back-and-forth motion
        // Total duration: 6 seconds (3 seconds each way)
        //
        // ## Why Ping-Pong?
        // Creates continuous motion without teleporting
        // Common in demos, idle animations, patrols
        .ping_pong()
        .expect("this curve has bounded domain, so this should never fail");

        // ROTATION ANIMATION
        // The cube rotates 90 degrees with a bouncy "elastic" effect
        // Note: Due to the cube's symmetry, it appears to keep rotating,
        // but it actually just rotates once and stays there!
        //
        // ## Elastic Easing Physics
        // Simulates a spring attached to the rotation:
        // - Overshoots the target
        // - Oscillates back and forth
        // - Settles at final position
        //
        // The elastic function uses:
        // f(t) = sin(13π/2 * t) * pow(2, -10t) + 1
        // This creates exponentially decaying oscillation
        //
        // ## Visual Trick
        // A cube has 4-fold rotational symmetry
        // 90° rotation makes it look identical!
        // Try changing to 45° to see the actual motion
        let rotation_curve = EasingCurve::new(
            Quat::IDENTITY,                    // Start: no rotation
            Quat::from_rotation_y(FRAC_PI_2),  // End: 90° rotation around Y axis
            EaseFunction::ElasticInOut,        // Bouncy/springy motion
        )
        // Rotation takes 4 seconds (different from translation timing)
        // Different timing creates visual interest - not everything moves in sync
        .reparametrize_linear(interval(0.0, 4.0).unwrap())
        .expect("this curve has bounded domain, so this should never fail");

        // ADD CURVES TO THE ANIMATION CLIP
        // Each curve animates a different property of the Transform component
        //
        // ## Animation Architecture
        // AnimationClip is a container for multiple curves
        // Each curve:
        // 1. Targets a specific entity (by ID)
        // 2. Modifies a specific field (via animated_field!)
        // 3. Provides values over time (the curve)
        //
        // ## The animated_field! Macro
        // Generates code to access struct fields by path
        // Transform::translation becomes a function that
        // can read/write the translation field
        
        // Animate the translation (position) field
        animation_clip.add_curve_to_target(
            animation_target_id,
            AnimatableCurve::new(animated_field!(Transform::translation), translation_curve),
        );
        
        // Animate the rotation field
        animation_clip.add_curve_to_target(
            animation_target_id,
            AnimatableCurve::new(animated_field!(Transform::rotation), rotation_curve),
        );

        // FINALIZE THE ANIMATION
        // Store the clip as an asset
        let animation_clip_handle = animation_clips.add(animation_clip);

        // Create an animation graph containing our clip
        // Graphs can blend multiple clips, but we just have one
        //
        // ## Why Animation Graphs?
        // Even for single animations, graphs provide:
        // - Consistent API with complex animations
        // - Future flexibility to add blending
        // - State machine capabilities
        // - Runtime modification options
        //
        // ## Asset Management
        // Assets in Bevy are reference-counted
        // The Handle allows multiple entities to share
        // the same animation data efficiently
        let (animation_graph, animation_node_index) =
            AnimationGraph::from_clip(animation_clip_handle);
        let animation_graph_handle = animation_graphs.add(animation_graph);

        // Return all the pieces needed to play this animation
        AnimationInfo {
            target_name: animation_target_name,
            target_id: animation_target_id,
            graph: animation_graph_handle,
            node_index: animation_node_index,
        }
    }
}
