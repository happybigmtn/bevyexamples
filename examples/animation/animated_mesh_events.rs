//! Demonstrates animation events - triggering game logic at specific points in an animation.
//!
//! Animation events are like musical notes in a score - they fire at precise times!
//! In this example, we spawn dust particles whenever the fox's feet hit the ground.
//! This technique is essential for synchronizing sound effects, visual effects,
//! and gameplay mechanics with character animations.
//!
//! ## Animation Theory: Event-Driven Animation Systems
//!
//! Animation events solve the synchronization problem:
//! - **Frame-Perfect Timing**: Events fire at exact animation frames
//! - **Decoupled Logic**: Animation data doesn't know about game logic
//! - **Reusable**: Same animation can trigger different events in different contexts
//!
//! Common animation event types:
//! 1. **Sound Events**: Footsteps, weapon swooshes, voice lines
//! 2. **Effect Events**: Particles, screen shake, muzzle flashes
//! 3. **Gameplay Events**: Damage frames, combo windows, state changes
//! 4. **IK Events**: Hand/foot placement adjustments
//!
//! ## Game Feel Context: The Magic of Synchronization
//!
//! Well-timed animation events create "juice":
//! - **Footstep Dust**: Grounds characters in the world (this example!)
//! - **Impact Frames**: Fighting games flash on hit for emphasis
//! - **Anticipation Effects**: Charge-up particles before attacks
//! - **Follow-through**: Trails and afterimages on fast movements
//!
//! Disney's principles at work:
//! - **Secondary Action**: Dust clouds are secondary to the run
//! - **Exaggeration**: 14 particles per step is overkill but looks great
//! - **Timing**: Events must match visual contact exactly
//!
//! ## Performance Optimization: Event Systems
//!
//! Animation events have performance implications:
//! - **Event Dispatch**: O(n) for n events per frame
//! - **Particle Spawning**: Can stress the spawn system
//! - **Memory Allocation**: Events allocate per trigger
//!
//! Optimization strategies:
//! 1. **Event Pooling**: Reuse event objects
//! 2. **Particle Pooling**: Pre-allocate particle entities
//! 3. **LOD Systems**: Fewer events at distance
//! 4. **Batching**: Group similar events together
//! 5. **Async Events**: Defer non-critical events
//!
//! ## Real-World Applications: AAA Event Systems
//!
//! Modern games use sophisticated event pipelines:
//! - **Unreal Engine**: AnimNotifies with custom event types
//! - **Unity**: Animation Events with method calls
//! - **Frostbite**: Timeline-based event tracks
//!
//! Examples from games:
//! - **God of War**: Every axe swing has 5-10 events (sound, VFX, damage)
//! - **Street Fighter**: Frame-perfect hitboxes via animation events
//! - **Horizon Zero Dawn**: Robot footsteps trigger dynamic vegetation
//! - **The Last of Us**: Contextual breathing/grunts via animation events
//!
//! ## Advanced Techniques: Beyond Simple Events
//!
//! 1. **Parametric Events**: Pass data with events (force, color, size)
//! 2. **Conditional Events**: Only fire if conditions are met
//! 3. **Event Curves**: Continuous values over time, not just triggers
//! 4. **Predictive Events**: Fire slightly early to compensate for latency
//! 5. **Event Layers**: Different event sets for different gameplay modes
//!
//! ## Common Issues and Solutions
//!
//! 1. **Event Spam**: Too many particles killing performance
//!    - Solution: Event cooldowns and pooling systems
//!
//! 2. **Desync**: Events don't match visual animation
//!    - Solution: Frame-based timing, not time-based
//!
//! 3. **Missing Events**: Events lost during transitions
//!    - Solution: Event queuing during blends
//!
//! 4. **Network Sync**: Events inconsistent in multiplayer
//!    - Solution: Deterministic event timing, authoritative server

use std::{f32::consts::PI, time::Duration};

use bevy::{
    animation::AnimationTargetId, color::palettes::css::WHITE, pbr::CascadeShadowConfigBuilder,
    prelude::*,
};
// Random number generation for particle effects
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng; // Deterministic RNG for reproducible results

const FOX_PATH: &str = "models/animated/Fox.glb";

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // Resources that implement FromWorld are initialized automatically
        .init_resource::<ParticleAssets>()  // Particle mesh and material
        .init_resource::<FoxFeetTargets>()  // Animation target IDs for each foot
        .add_systems(Startup, setup)
        .add_systems(Update, setup_scene_once_loaded)
        .add_systems(Update, simulate_particles)  // Updates particle physics
        // Observers are Bevy's event handlers - they react to specific events
        .add_observer(observe_on_step)  // Spawns particles when OnStep events fire
        .run();
}

// Wrapper for our random number generator
// Using a seeded RNG ensures particle effects are reproducible for testing
#[derive(Resource)]
struct SeededRng(ChaCha8Rng);

// Stores references to our loaded animations
#[derive(Resource)]
struct Animations {
    index: AnimationNodeIndex,       // Index of the "run" animation
    graph_handle: Handle<AnimationGraph>, // Handle to the animation graph asset
}

// Custom event that fires when a foot hits the ground
// Events are Bevy's way of communicating between systems
// Reflect allows this to be inspected in tools, Clone lets it be copied
//
// ## Rust Pattern: Zero-Sized Types (ZST)
// OnStep has no fields, making it a ZST - takes zero bytes of memory!
// ZSTs are perfect for marker types and simple events
// The compiler optimizes them away at runtime
//
// ## Bevy Architecture: Event System
// Events in Bevy are double-buffered:
// - Writers add events to the current frame's buffer
// - Readers consume events from the previous frame's buffer
// - Buffers swap each frame, preventing race conditions
#[derive(Event, Reflect, Clone)]
struct OnStep;

// This observer function is called whenever an OnStep event is triggered
// Observers are like event listeners - they react to specific events in the game
//
// ## Bevy Pattern: Observers vs Systems
// - Systems: Run every frame, check conditions manually
// - Observers: Run only when specific events occur
// Observers are more efficient for event-driven logic!
//
// ## Game Feel: Particle System Design
// Good particle effects need:
// - **Variety**: Random size, speed, and lifetime
// - **Directionality**: Particles spread in believable patterns
// - **Decay**: Particles fade/shrink over time
// - **Performance**: Use shared meshes and materials
fn observe_on_step(
    trigger: Trigger<OnStep>,
    particle: Res<ParticleAssets>,
    mut commands: Commands,
    transforms: Query<&GlobalTransform>,
    mut seeded_rng: ResMut<SeededRng>,
) {
    // Get the world position of the foot that triggered this event
    let translation = transforms.get(trigger.target()).unwrap().translation();
    
    // Spawn dust particles at the foot's position
    // 14 particles creates a nice dust cloud without overwhelming performance
    for _ in 0..14 {
        // Random horizontal direction and distance
        // Dir2 is a normalized 2D direction vector, perfect for circular spread
        let horizontal = seeded_rng.0.r#gen::<Dir2>() * seeded_rng.0.gen_range(8.0..12.0);
        
        // Upward velocity component (particles fly up a bit)
        let vertical = seeded_rng.0.gen_range(0.0..4.0);
        
        // Random particle size for variety
        let size = seeded_rng.0.gen_range(0.2..1.0);

        // Spawn a particle entity
        commands.spawn((
            Particle {
                // Each particle lives for 0.2-0.6 seconds
                lifetime_timer: Timer::from_seconds(
                    seeded_rng.0.gen_range(0.2..0.6),
                    TimerMode::Once, // Timer runs once then stops
                ),
                size,
                // Convert 2D horizontal spread to 3D velocity
                // Multiply by 10 for visible movement speed
                velocity: Vec3::new(horizontal.x, vertical, horizontal.y) * 10.0,
            },
            Mesh3d(particle.mesh.clone()),
            MeshMaterial3d(particle.material.clone()),
            Transform {
                translation, // Start at foot position
                scale: Vec3::splat(size), // Uniform scale based on random size
                ..Default::default()
            },
        ));
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // ANIMATION SETUP
    // Load just the "run" animation (index 2) from the fox model
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(FOX_PATH)),
    );

    // Store the animation data as a resource for later use
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        index,
        graph_handle,
    });

    // SCENE SETUP
    // Camera positioned for a good view of the running fox
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(100.0, 100.0, 150.0).looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Directional light with shadows
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
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(FOX_PATH)),
    ));

    // RANDOM NUMBER GENERATOR
    // We're seeding the PRNG to make particle effects deterministic
    // This ensures the example looks the same every time it runs
    // In a real game, you might use rand::thread_rng() instead
    let seeded_rng = ChaCha8Rng::seed_from_u64(19878367467712);
    commands.insert_resource(SeededRng(seeded_rng));
}

// Sets up animation events once the fox model is loaded
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    feet: Res<FoxFeetTargets>,
    graphs: Res<Assets<AnimationGraph>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    // Helper function to extract a mutable AnimationClip from the graph
    // This is necessary because animation clips are stored inside the graph structure
    fn get_clip<'a>(
        node: AnimationNodeIndex,
        graph: &AnimationGraph,
        clips: &'a mut Assets<AnimationClip>,
    ) -> &'a mut AnimationClip {
        let node = graph.get(node).unwrap();
        let clip = match &node.node_type {
            AnimationNodeType::Clip(handle) => clips.get_mut(handle),
            _ => unreachable!(), // We know this is a clip node
        };
        clip.unwrap()
    }

    for (entity, mut player) in &mut players {
        // ADD ANIMATION EVENTS
        // We'll trigger OnStep events at specific times when each foot hits the ground
        
        let graph = graphs.get(&animations.graph_handle).unwrap();
        let running_animation = get_clip(animations.index, graph, &mut clips);

        // TIMING ANIMATION EVENTS
        // Animation time is in seconds. To find the right time:
        // 1. Count the frame number where the foot hits the ground
        // 2. Divide by the animation's frame rate
        // Example: Frame 15 at 24 FPS = 15 / 24 = 0.625 seconds
        //
        // These times were found by analyzing the fox run cycle:
        //
        // ## Animation Analysis: Quadruped Gaits
        // The fox uses a "rotary gallop" gait pattern:
        // - Back feet push off in sequence (power phase)
        // - Front feet catch and pull in sequence (recovery phase)
        // - Brief aerial phase where all feet are off ground
        // This creates the characteristic bounding motion
        //
        // ## Timing Precision
        // Event times must be frame-perfect for believability
        // Too early: Dust appears before contact (floaty)
        // Too late: Foot slides before dust appears (slippery)
        running_animation.add_event_to_target(feet.front_left, 0.625, OnStep);
        running_animation.add_event_to_target(feet.front_right, 0.5, OnStep);
        running_animation.add_event_to_target(feet.back_left, 0.0, OnStep);    // Start of cycle
        running_animation.add_event_to_target(feet.back_right, 0.125, OnStep);

        // START THE ANIMATION
        let mut transitions = AnimationTransitions::new();

        // Always use AnimationTransitions to start animations when it exists
        // It manages blending and ensures smooth transitions between clips
        transitions
            .play(&mut player, animations.index, Duration::ZERO)
            .repeat(); // Loop the running animation

        // Attach the animation components to the entity
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}

// Updates particle physics and lifetime each frame
//
// ## Performance: Particle System Optimization
// This is a simple particle system. For production games:
// - Use GPU particles for thousands of particles
// - Implement object pooling to avoid allocation
// - Use spatial partitioning for collision detection
// - Consider LOD systems for distant particles
//
// ## Physics: Euler Integration
// We use simple Euler integration: position += velocity * dt
// This is fast but can be inaccurate for complex motion
// For better accuracy, consider Verlet or RK4 integration
fn simulate_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Particle)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in &mut query {
        // Update lifetime timer
        if particle.lifetime_timer.tick(time.delta()).just_finished() {
            // Particle has expired - remove it from the world
            commands.entity(entity).despawn();
            return; // Skip to next particle
        }

        // Physics simulation:
        // Move particle based on velocity (simple Euler integration)
        transform.translation += particle.velocity * time.delta_secs();
        
        // Scale particle down over lifetime (fade out effect)
        // lerp interpolates from size to 0 based on timer progress
        transform.scale = Vec3::splat(particle.size.lerp(0.0, particle.lifetime_timer.fraction()));
        
        // Apply drag to slow particles down over time
        // smooth_nudge gradually moves velocity toward zero
        particle
            .velocity
            .smooth_nudge(&Vec3::ZERO, 4.0, time.delta_secs());
    }
}

// Component for dust particle entities
#[derive(Component)]
struct Particle {
    lifetime_timer: Timer, // Controls how long the particle lives
    size: f32,            // Starting size (will shrink over time)
    velocity: Vec3,       // Movement direction and speed
}

// Shared assets for all particles (more efficient than creating new ones each time)
#[derive(Resource)]
struct ParticleAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

// FromWorld trait allows automatic resource initialization
// This runs once when we call init_resource::<ParticleAssets>()
impl FromWorld for ParticleAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            // All particles share the same sphere mesh
            mesh: world.add_asset::<Mesh>(Sphere::new(10.0)),
            // Simple white material for dust particles
            material: world.add_asset::<StandardMaterial>(StandardMaterial {
                base_color: WHITE.into(),
                ..Default::default()
            }),
        }
    }
}

/// Stores the AnimationTargetIds of the fox's feet bones
/// These IDs tell the animation system which bones to attach events to
#[derive(Resource)]
struct FoxFeetTargets {
    front_right: AnimationTargetId,
    front_left: AnimationTargetId,
    back_left: AnimationTargetId,
    back_right: AnimationTargetId,
}

// The bone names come from the fox model's skeleton hierarchy
// You can find these by inspecting the glTF file or using a 3D viewer
impl Default for FoxFeetTargets {
    fn default() -> Self {
        // Common path to the hip bone - all limbs branch from here
        let hip_node = ["root", "_rootJoint", "b_Root_00", "b_Hip_01"];
        
        // BONE HIERARCHY PATHS
        // Each foot is at the end of a chain of bones:
        // Hip -> Spine -> Upper Limb -> Lower Limb -> Foot
        
        // Front feet (in foxes, these are actually the "hands")
        let front_left_foot = hip_node.iter().chain(
            [
                "b_Spine01_02",
                "b_Spine02_03",
                "b_LeftUpperArm_09",
                "b_LeftForeArm_010",
                "b_LeftHand_011", // The "hand" is the front foot
            ]
            .iter(),
        );
        let front_right_foot = hip_node.iter().chain(
            [
                "b_Spine01_02",
                "b_Spine02_03",
                "b_RightUpperArm_06",
                "b_RightForeArm_07",
                "b_RightHand_08",
            ]
            .iter(),
        );
        
        // Back feet (actual legs)
        let back_left_foot = hip_node.iter().chain(
            [
                "b_LeftLeg01_015",   // Upper leg
                "b_LeftLeg02_016",   // Lower leg
                "b_LeftFoot01_017",  // Ankle
                "b_LeftFoot02_018",  // Foot tip
            ]
            .iter(),
        );
        let back_right_foot = hip_node.iter().chain(
            [
                "b_RightLeg01_019",
                "b_RightLeg02_020",
                "b_RightFoot01_021",
                "b_RightFoot02_022",
            ]
            .iter(),
        );
        
        // Create AnimationTargetIds from the bone paths
        Self {
            front_left: AnimationTargetId::from_iter(front_left_foot),
            front_right: AnimationTargetId::from_iter(front_right_foot),
            back_left: AnimationTargetId::from_iter(back_left_foot),
            back_right: AnimationTargetId::from_iter(back_right_foot),
        }
    }
}
