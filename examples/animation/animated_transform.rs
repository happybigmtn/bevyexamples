//! Create and play an animation defined by code that operates on the [`Transform`] component.
//!
//! Animation is bringing things to life! In physics, we describe motion with equations.
//! In Bevy, we describe motion with curves - mathematical functions that map time to values.
//! This example shows a planetary system: a planet moving in a square, with a satellite
//! orbiting around it. It's like a simplified solar system where we control every movement!
//!
//! ## Animation Theory: Procedural Animation
//!
//! Procedural animation means generating motion through code rather than artist-created data:
//! - **Parametric Curves**: Mathematical functions that define motion
//! - **Interpolation**: Smooth transitions between keyframes
//! - **Hierarchical Animation**: Parent-child relationships create complex motion
//!
//! Common curve types:
//! 1. **Linear**: Constant speed between points
//! 2. **Bezier**: Smooth curves with control points
//! 3. **Hermite**: Curves defined by positions and tangents
//! 4. **Catmull-Rom**: Smooth curves through all control points
//!
//! ## Game Feel Context: The Art of Motion
//!
//! Great procedural animation follows principles:
//! - **Ease In/Out**: Natural acceleration and deceleration
//! - **Overlapping Action**: Different parts move at different rates
//! - **Arcs**: Natural motion follows curved paths
//! - **Timing**: Speed conveys weight and energy
//!
//! This example demonstrates:
//! - **Primary Motion**: Planet's square path
//! - **Secondary Motion**: Satellite's orbit
//! - **Tertiary Motion**: Satellite's pulsing and spinning
//!
//! ## Performance Optimization: Animation Systems
//!
//! Procedural animation performance tips:
//! 1. **Curve Sampling**: Pre-compute curves when possible
//! 2. **LOD Systems**: Simpler animations at distance
//! 3. **Culling**: Don't animate off-screen objects
//! 4. **Batching**: Update similar animations together
//! 5. **SIMD**: Use vector math for multiple objects
//!
//! Memory considerations:
//! - Curves are more memory-efficient than baked animations
//! - Trade computation for storage
//! - Cache frequently evaluated curves
//!
//! ## Real-World Applications: Procedural Motion
//!
//! Games using procedural animation:
//! - **No Man's Sky**: Creature animation from parameters
//! - **Spore**: Entire animation system is procedural
//! - **Rain World**: Physics-based creature movement
//! - **Journey**: Cloth and scarf simulation
//!
//! Common uses:
//! - UI animations (tweening)
//! - Camera movement (cinematics)
//! - Environmental animation (waves, grass)
//! - Character customization (different body types)
//!
//! ## Advanced Techniques: Beyond Keyframes
//!
//! 1. **Physics Simulation**: Ragdolls, cloth, hair
//! 2. **Inverse Kinematics**: Goal-based limb positioning
//! 3. **Procedural Locomotion**: Adaptive walking/running
//! 4. **Neural Animation**: ML-driven motion synthesis
//! 5. **Motion Matching**: Blend between motion captures
//!
//! ## Common Issues and Solutions
//!
//! 1. **Jerky Motion**: Sudden changes in velocity
//!    - Solution: Use smooth interpolation curves
//!
//! 2. **Floating Movement**: Objects don't feel grounded
//!    - Solution: Add weight through timing and arcs
//!
//! 3. **Robotic Feel**: Too perfect, mechanical motion
//!    - Solution: Add noise, imperfections, overlap
//!
//! 4. **Performance Issues**: Too many animated objects
//!    - Solution: LOD, instancing, GPU animation

use std::f32::consts::PI;

use bevy::{
    animation::{animated_field, AnimationTarget, AnimationTargetId},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Ambient light illuminates everything equally - like a cloudy day
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 150.0,
            ..default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Animation assets! These are new:
    mut animations: ResMut<Assets<AnimationClip>>,  // Stores animation data
    mut graphs: ResMut<Assets<AnimationGraph>>,     // Manages animation flow
) {
    // Camera - positioned to see our planetary system
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light source
    commands.spawn((
        PointLight {
            intensity: 500_000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 2.5, 0.0),
    ));

    // Animation targeting works by finding entities with specific identifiers.
    // We'll use Name components as our targeting system - like addresses for our animations.
    // Think of these as "stage names" for our actors in the animation theater
    let planet = Name::new("planet");
    let orbit_controller = Name::new("orbit_controller");
    let satellite = Name::new("satellite");

    // Creating the animation - think of this as choreographing a dance
    let mut animation = AnimationClip::default();
    
    // PLANET TRANSLATION ANIMATION
    // A curve defines how a value changes over time - it's a function f(t) = value
    let planet_animation_target_id = AnimationTargetId::from_name(&planet);
    
    // ## Animation Curves: The Mathematics of Motion
    // AnimatableCurve combines:
    // 1. A property to animate (Transform::translation)
    // 2. A curve that maps time to values
    // 3. An interpolation method (how to blend between keyframes)
    //
    // ## Rust Pattern: Macro for Type Safety
    // animated_field! macro ensures we're animating a valid, animatable field
    // It generates the correct property path at compile time
    animation.add_curve_to_target(
        planet_animation_target_id,
        AnimatableCurve::new(
            // We're animating the translation (position) field of Transform
            animated_field!(Transform::translation),
            // UnevenSampleAutoCurve = keyframe animation with irregular time intervals
            // The iterator pairs time values with position values: (time, position)
            //
            // ## Interpolation Theory
            // Between keyframes, values are linearly interpolated:
            // value = start + (end - start) * t
            // Where t is the normalized time between keyframes
            UnevenSampleAutoCurve::new([0.0, 1.0, 2.0, 3.0, 4.0].into_iter().zip([
                Vec3::new(1.0, 0.0, 1.0),   // t=0: top-right corner
                Vec3::new(-1.0, 0.0, 1.0),  // t=1: top-left corner
                Vec3::new(-1.0, 0.0, -1.0), // t=2: bottom-left corner
                Vec3::new(1.0, 0.0, -1.0),  // t=3: bottom-right corner
                // For seamless looping, end where we began!
                Vec3::new(1.0, 0.0, 1.0),   // t=4: back to start
            ]))
            .expect("should be able to build translation curve because we pass in valid samples"),
        ),
    );
    // ORBIT CONTROLLER ROTATION ANIMATION
    // Now we animate rotation - making things spin!
    // The target ID uses a path through the hierarchy: planet -> orbit_controller
    // It's like giving directions: "Go to the planet, then find its orbit_controller child"
    let orbit_controller_animation_target_id =
        AnimationTargetId::from_names([planet.clone(), orbit_controller.clone()].iter());
    animation.add_curve_to_target(
        orbit_controller_animation_target_id,
        AnimatableCurve::new(
            // Now we're animating rotation instead of position
            animated_field!(Transform::rotation),
            // Quaternions represent rotations - they're like complex numbers for 3D
            //
            // ## Quaternion Mathematics
            // Quaternions (w, x, y, z) avoid gimbal lock and interpolate smoothly
            // from_axis_angle creates a quaternion from:
            // - An axis vector (the line to rotate around)
            // - An angle in radians (how far to rotate)
            //
            // ## Why Quaternions?
            // 1. No gimbal lock (unlike Euler angles)
            // 2. Smooth interpolation (SLERP)
            // 3. Compact representation (4 floats)
            // 4. Efficient composition (quaternion multiplication)
            UnevenSampleAutoCurve::new([0.0, 1.0, 2.0, 3.0, 4.0].into_iter().zip([
                Quat::IDENTITY,                          // t=0: no rotation
                Quat::from_axis_angle(Vec3::Y, PI / 2.), // t=1: 90° around Y axis
                Quat::from_axis_angle(Vec3::Y, PI),      // t=2: 180° around Y axis
                Quat::from_axis_angle(Vec3::Y, PI * 1.5), // t=3: 270° around Y axis
                Quat::IDENTITY,                          // t=4: full circle (360° = 0°)
            ]))
            .expect("Failed to build rotation curve"),
        ),
    );
    // SATELLITE SCALE ANIMATION
    // Important timing note: All curves in one AnimationClip share the same timeline!
    // If one curve is shorter, it waits for the others to finish before the clip loops.
    // For different periods, use separate AnimationClips.
    let satellite_animation_target_id = AnimationTargetId::from_names(
        // Path: planet -> orbit_controller -> satellite
        [planet.clone(), orbit_controller.clone(), satellite.clone()].iter(),
    );
    animation.add_curve_to_target(
        satellite_animation_target_id,
        AnimatableCurve::new(
            // Animating scale - making things grow and shrink
            animated_field!(Transform::scale),
            // Notice: 9 keyframes over 4 seconds = pulsing twice as fast!
            UnevenSampleAutoCurve::new(
                [0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0]
                    .into_iter()
                    .zip([
                        Vec3::splat(0.8), // splat(x) = Vec3(x, x, x) - uniform scale
                        Vec3::splat(1.2), // Bigger
                        Vec3::splat(0.8), // Smaller
                        Vec3::splat(1.2), // Bigger
                        Vec3::splat(0.8), // Smaller
                        Vec3::splat(1.2), // Bigger
                        Vec3::splat(0.8), // Smaller
                        Vec3::splat(1.2), // Bigger
                        Vec3::splat(0.8), // End small for smooth loop
                    ]),
            )
            .expect("Failed to build scale curve"),
        ),
    );
    // SATELLITE ROTATION ANIMATION
    // Multiple curves can target the same entity! Here we add rotation to our scaling satellite.
    // It's like a moon that spins while also pulsing in size.
    animation.add_curve_to_target(
        AnimationTargetId::from_names(
            [planet.clone(), orbit_controller.clone(), satellite.clone()].iter(),
        ),
        AnimatableCurve::new(
            animated_field!(Transform::rotation),
            // Same rotation pattern as the orbit controller - full 360° rotation
            UnevenSampleAutoCurve::new([0.0, 1.0, 2.0, 3.0, 4.0].into_iter().zip([
                Quat::IDENTITY,
                Quat::from_axis_angle(Vec3::Y, PI / 2.),
                Quat::from_axis_angle(Vec3::Y, PI / 2. * 2.),
                Quat::from_axis_angle(Vec3::Y, PI / 2. * 3.),
                Quat::IDENTITY,
            ]))
            .expect("should be able to build rotation curve because we pass in valid samples"),
        ),
    );

    // ANIMATION SETUP
    // AnimationGraph manages how animations flow and blend
    // For now, we just have one clip that plays on loop
    let (graph, animation_index) = AnimationGraph::from_clip(animations.add(animation));

    // AnimationPlayer is the "playback device" - it reads the animation data
    // and applies it to entities over time
    let mut player = AnimationPlayer::default();
    player.play(animation_index).repeat(); // Play forever!

    // CREATE THE ANIMATED SCENE
    // Now we build the actual entities that will be animated
    
    // The planet - our main celestial body
    let planet_entity = commands
        .spawn((
            Mesh3d(meshes.add(Sphere::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))), // Sandy color
            planet, // The name we're targeting with animations
            // These two components make animation work:
            AnimationGraphHandle(graphs.add(graph)), // The animation data
            player,                                  // The playback controller
        ))
        .id(); // We need the ID to reference this entity
        
    // Add the animation target component after spawning
    // This links the animation curves to this specific entity
    commands
        .entity(planet_entity)
        .insert(AnimationTarget {
            id: planet_animation_target_id,
            player: planet_entity, // Points back to the entity with the player
        })
        .with_children(|p| {
            // The orbit controller - an invisible entity that just rotates
            // This is a clever trick: by making the satellite a child of this,
            // the satellite orbits when this rotates!
            p.spawn((
                Transform::default(),
                Visibility::default(), // Invisible but still in the hierarchy
                orbit_controller,
                AnimationTarget {
                    id: orbit_controller_animation_target_id,
                    player: planet_entity, // Same player controls all animations
                },
            ))
            .with_children(|p| {
                // The satellite - a small cube that orbits the planet
                p.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
                    MeshMaterial3d(materials.add(Color::srgb(0.3, 0.9, 0.3))), // Green
                    // Start 1.5 units away from parent (orbit radius)
                    Transform::from_xyz(1.5, 0.0, 0.0),
                    AnimationTarget {
                        id: satellite_animation_target_id,
                        player: planet_entity,
                    },
                    satellite,
                ));
            });
        });
    
    // The hierarchy creates compound motion:
    // - Planet moves in a square (its own translation)
    // - Orbit controller rotates (carries satellite around)
    // - Satellite rotates and scales (while being carried by parent transforms)
    // It's like a carnival ride with multiple moving parts!
    //
    // ## Transform Hierarchy Mathematics
    // World transform = Parent * Local
    // For deep hierarchies: World = Root * Parent1 * Parent2 * ... * Local
    //
    // ## Performance Note
    // Transform propagation happens automatically each frame
    // Deep hierarchies can impact performance
    // Consider flattening for many animated objects
}
