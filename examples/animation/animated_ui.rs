//! Shows how to use animation clips to animate UI properties.
//!
//! UI animation breathes life into interfaces! Just like how a physicist might study
//! oscillating systems, we'll create text that pulses in size and shifts through colors.
//! This demonstrates that Bevy's animation system works on ANY component properties,
//! not just transforms - it's a universal animation machine!
//!
//! ## Animation Theory: UI Motion Design
//!
//! UI animation serves specific purposes:
//! - **Feedback**: Confirm user actions (button press, hover)
//! - **Guidance**: Direct attention (pulsing call-to-action)
//! - **Continuity**: Smooth transitions between states
//! - **Personality**: Express brand identity through motion
//!
//! Key principles for UI animation:
//! 1. **Duration**: 200-300ms for micro-interactions
//! 2. **Easing**: Ease-out for entering, ease-in for exiting
//! 3. **Consistency**: Similar elements animate similarly
//! 4. **Performance**: 60 FPS is non-negotiable
//!
//! ## Game Feel Context: Emotional UI
//!
//! Great UI animation creates emotional responses:
//! - **Bouncy**: Playful, energetic (games for kids)
//! - **Smooth**: Professional, trustworthy (strategy games)
//! - **Snappy**: Responsive, powerful (action games)
//! - **Flowing**: Relaxing, meditative (puzzle games)
//!
//! This example uses:
//! - **Pulsing**: Creates urgency and draws attention
//! - **Color Cycling**: Adds visual interest and energy
//! - **Synchronized**: Both animations share timing for cohesion
//!
//! ## Performance Optimization: UI Animation
//!
//! UI animation has unique performance considerations:
//! 1. **Batch Updates**: Group UI changes to minimize layout recalculation
//! 2. **Transform-Only**: Prefer transform animations over layout changes
//! 3. **GPU Acceleration**: Use opacity and transform for GPU optimization
//! 4. **Culling**: Don't animate off-screen UI elements
//! 5. **LOD**: Reduce animation complexity on low-end devices
//!
//! Memory tips:
//! - Reuse animation clips for similar UI elements
//! - Use animation events instead of continuous curves when possible
//! - Pool UI elements instead of creating/destroying
//!
//! ## Real-World Applications: UI in Games
//!
//! AAA examples of UI animation:
//! - **Overwatch**: Every UI element has personality animations
//! - **Hearthstone**: Cards flip, glow, and burst with effects
//! - **Destiny**: Sci-fi UI with holographic animations
//! - **Persona 5**: Stylized UI with bold motion design
//!
//! Common UI animations:
//! - Health bars (damage flash, heal glow)
//! - Score counters (rolling numbers)
//! - Notifications (slide in/out)
//! - Menus (accordion, fade, slide)
//! - Loading screens (spinners, progress bars)
//!
//! ## Advanced Techniques: Custom Properties
//!
//! This example demonstrates custom animatable properties:
//! 1. **Type Safety**: Compile-time validation of animations
//! 2. **Flexibility**: Animate ANY component field
//! 3. **Composition**: Combine multiple property animations
//! 4. **Interpolation**: Custom blending logic for complex types
//!
//! You can animate:
//! - Shader parameters
//! - Physics properties
//! - Audio parameters
//! - Game state values
//!
//! ## Common Issues and Solutions
//!
//! 1. **Jittery Text**: Sub-pixel positioning issues
//!    - Solution: Round positions to nearest pixel
//!
//! 2. **Layout Thrashing**: Animations cause reflow
//!    - Solution: Use transforms instead of size/position
//!
//! 3. **Z-Fighting**: Animated UI overlaps incorrectly
//!    - Solution: Explicit z-ordering with ZIndex
//!
//! 4. **Text Blur**: Scaling makes text fuzzy
//!    - Solution: Use distance field fonts or animate font size directly

use bevy::{
    animation::{
        animated_field, AnimationEntityMut, AnimationEvaluationError, AnimationTarget,
        AnimationTargetId,
    },
    prelude::*,
};
use std::any::TypeId;

// Holds information about the animation we programmatically create.
// This is like a blueprint for our animation - all the pieces needed to make it work
//
// ## Rust Pattern: Struct for Related Data
// Instead of passing around 4 separate values, we bundle them together
// This makes our code more maintainable and less error-prone
// It's also more efficient - Rust can optimize struct layout
struct AnimationInfo {
    // The name of the animation target (in this case, the text).
    target_name: Name,
    // The ID of the animation target, derived from the name.
    target_id: AnimationTargetId,
    // The animation graph asset.
    graph: Handle<AnimationGraph>,
    // The index of the node within that graph.
    node_index: AnimationNodeIndex,
}

// The entry point.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Note that we don't need any systems other than the setup system,
        // because Bevy automatically updates animations every frame.
        // The animation system is like a perpetual motion machine - once started, it runs forever!
        .add_systems(Startup, setup)
        .run();
}

impl AnimationInfo {
    // Programmatically creates the UI animation.
    // This is like composing a piece of music - we define how properties change over time
    fn create(
        animation_graphs: &mut Assets<AnimationGraph>,
        animation_clips: &mut Assets<AnimationClip>,
    ) -> AnimationInfo {
        // Create an ID that identifies the text node we're going to animate.
        let animation_target_name = Name::new("Text");
        let animation_target_id = AnimationTargetId::from_name(&animation_target_name);

        // Allocate an animation clip - our timeline of changes
        let mut animation_clip = AnimationClip::default();

        // FONT SIZE ANIMATION
        // Create a curve that makes the text pulse like a heartbeat
        //
        // ## Animation Psychology
        // The pulsing motion mimics biological rhythms:
        // - Heartbeat: 60-100 BPM (1-1.6Hz)
        // - Breathing: 12-20 breaths/min (0.2-0.3Hz)
        // Our 3-beat pattern over 3 seconds = 1Hz, right in the sweet spot!
        //
        // ## Interpolation Details
        // Between keyframes, Bevy linearly interpolates:
        // At t=0.25: size = 24 + (80-24) * (0.25/0.5) = 52
        // This creates smooth transitions, not jumps
        animation_clip.add_curve_to_target(
            animation_target_id,
            AnimatableCurve::new(
                // We're animating the font_size field of TextFont component
                animated_field!(TextFont::font_size),
                // Keyframes: (time, value) pairs that define the animation
                AnimatableKeyframeCurve::new(
                    [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0]
                        .into_iter()
                        .zip([
                            24.0, // t=0.0: small
                            80.0, // t=0.5: BIG!
                            24.0, // t=1.0: small again
                            80.0, // t=1.5: BIG!
                            24.0, // t=2.0: small
                            80.0, // t=2.5: BIG!
                            24.0, // t=3.0: back to start
                        ]),
                )
                .expect(
                    "should be able to build font size curve because we pass in valid samples",
                ),
            ),
        );

        // COLOR ANIMATION
        // Create a curve that animates font color - like a rainbow cycle!
        // Note that this should have the same time duration as the previous curve.
        //
        // This demonstrates custom properties - we can animate ANY data type
        // as long as we tell Bevy how to access and modify it
        animation_clip.add_curve_to_target(
            animation_target_id,
            AnimatableCurve::new(
                // TextColorProperty is our custom accessor (defined below)
                TextColorProperty,
                AnimatableKeyframeCurve::new([0.0, 1.0, 2.0, 3.0].into_iter().zip([
                    Srgba::RED,   // t=0: Red
                    Srgba::GREEN, // t=1: Green
                    Srgba::BLUE,  // t=2: Blue
                    Srgba::RED,   // t=3: Back to red for seamless loop
                ]))
                .expect(
                    "should be able to build color curve because we pass in valid samples",
                ),
            ),
        );

        // Save our animation clip as an asset - it's like recording our composition
        let animation_clip_handle = animation_clips.add(animation_clip);

        // Create an animation graph with that clip
        // The graph manages how clips play and blend (though we only have one here)
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

// Creates all the entities in the scene.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
) {
    // Create the animation using our factory method
    let AnimationInfo {
        target_name: animation_target_name,
        target_id: animation_target_id,
        graph: animation_graph,
        node_index: animation_node_index,
    } = AnimationInfo::create(&mut animation_graphs, &mut animation_clips);

    // Build an animation player that automatically plays the UI animation
    let mut animation_player = AnimationPlayer::default();
    animation_player.play(animation_node_index).repeat(); // Loop forever!

    // Add a camera to see our UI
    commands.spawn(Camera2d);

    // BUILD THE UI HIERARCHY
    // We need a specific structure for UI animation to work:
    // 1. Parent node with the AnimationPlayer
    // 2. Child node(s) that get animated
    commands
        .spawn((
            // This node covers the whole screen and centers its children
            // It's like a stage for our animated text
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),    // Anchor to all edges
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center, // Center horizontally
                align_items: AlignItems::Center,         // Center vertically
                ..default()
            },
            // The animation player must be on the parent!
            animation_player,
            AnimationGraphHandle(animation_graph),
        ))
        .with_children(|builder| {
            // Get the entity ID of the parent (which has the player)
            let player = builder.target_entity();
            
            // Create the text that will be animated
            builder
                .spawn((
                    Text::new("Bevy"), // The star of our show!
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 24.0, // Starting size (will be animated)
                        ..default()
                    },
                    TextColor(Color::Srgba(Srgba::RED)), // Starting color (will be animated)
                    TextLayout::new_with_justify(JustifyText::Center),
                ))
                // These components link the animation to this text node
                .insert(AnimationTarget {
                    id: animation_target_id,
                    player, // Points to parent entity with the AnimationPlayer
                })
                .insert(animation_target_name); // The name we're targeting
        });
}

// CUSTOM ANIMATABLE PROPERTY
// This is advanced magic! We're teaching Bevy how to animate a property it doesn't
// know about by default. It's like creating a new law of physics for our animation universe.
//
// We implement `AnimatableProperty` to define custom property accessor logic
//
// ## Rust Pattern: Zero-Sized Type as Marker
// TextColorProperty has no data - it's just a type that implements a trait
// This is a common Rust pattern for compile-time polymorphism
// The compiler optimizes away the struct entirely!
//
// ## Bevy Architecture: Property Animation
// The animation system is generic over properties:
// 1. AnimatableProperty tells how to access/modify a value
// 2. The animation system interpolates between values
// 3. Your implementation applies the interpolated value
//
// This separation of concerns allows animating ANYTHING!
#[derive(Clone)]
struct TextColorProperty;

impl AnimatableProperty for TextColorProperty {
    // The type of value we're animating
    type Property = Srgba;

    // A unique identifier for this property animator
    // EvaluatorId ensures each property type is handled uniquely
    fn evaluator_id(&self) -> EvaluatorId {
        EvaluatorId::Type(TypeId::of::<Self>())
    }

    // This is where the magic happens - we tell Bevy how to access and modify
    // the color within a TextColor component
    fn get_mut<'a>(
        &self,
        entity: &'a mut AnimationEntityMut,
    ) -> Result<&'a mut Self::Property, AnimationEvaluationError> {
        // First, try to get the TextColor component
        let text_color = entity
            .get_mut::<TextColor>()
            .ok_or(AnimationEvaluationError::ComponentNotPresent(TypeId::of::<
                TextColor,
            >(
            )))?
            .into_inner();
            
        // TextColor contains a Color enum, which can have different formats
        // We only handle Srgba format in this example
        //
        // ## Rust Pattern: Pattern Matching for Safety
        // The match ensures we handle all possible Color variants
        // ref mut allows us to return a mutable reference to the inner value
        // without taking ownership
        match text_color.0 {
            Color::Srgba(ref mut color) => Ok(color), // Success! Return mutable reference
            _ => Err(AnimationEvaluationError::PropertyNotPresent(TypeId::of::<
                Srgba,
            >(
            ))), // Wrong color format
        }
    }
    
    // This is like building a bridge between Bevy's animation system and our custom data
    // The animation system says "I have color values", we say "Put them HERE in the component"
    //
    // ## Extension Ideas
    // You could extend this to:
    // - Animate between different color spaces (HSL, LAB)
    // - Support gradients or color patterns
    // - Add easing functions for non-linear color transitions
    // - Animate alpha separately for fade effects
}
