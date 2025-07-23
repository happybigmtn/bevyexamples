//! Plays an animation on a skinned glTF model of a fox.
//!
//! Skeletal animation is how we bring 3D characters to life! Think of a skeleton as
//! a hierarchical structure of bones (like your own skeleton). Each bone influences
//! nearby vertices in the mesh - when the bone moves, the skin follows.
//! This example shows the fundamental pattern: load a model with animations,
//! find its animation player, and start playing!
//!
//! ## Animation Theory: Mathematical Foundations
//!
//! Skeletal animation uses a technique called "Linear Blend Skinning" (LBS):
//! - Each vertex has weights assigned to multiple bones (usually 4 max)
//! - Final vertex position = Σ(bone_transform × vertex_position × bone_weight)
//! - This creates smooth deformation as bones rotate and translate
//!
//! The bone hierarchy forms a kinematic chain where each bone's transform is:
//! world_transform = parent_transform × local_transform
//!
//! ## Game Feel Context: Creating Life Through Motion
//!
//! The "12 Principles of Animation" apply to games:
//! 1. **Squash and Stretch**: Not shown here, but fox's body could compress during run
//! 2. **Anticipation**: Fox could crouch before jumping
//! 3. **Staging**: Camera angle showcases the run cycle clearly
//! 4. **Straight Ahead vs Pose-to-Pose**: This uses pre-made poses (pose-to-pose)
//! 5. **Follow Through**: Fox's tail and ears continue moving after body stops
//! 6. **Slow In/Out**: Natural acceleration in the run cycle
//! 7. **Arc**: Fox's body follows curved paths, not straight lines
//! 8. **Secondary Action**: Ears bouncing adds life without distracting
//! 9. **Timing**: Run cycle speed affects perceived weight and energy
//! 10. **Exaggeration**: Games often exaggerate for clarity and impact
//! 11. **Solid Drawing**: 3D model maintains volume through deformation
//! 12. **Appeal**: Fox design is appealing and readable
//!
//! ## Performance Optimization
//!
//! Skeletal animation is GPU-friendly because:
//! - Bone matrices are uploaded once per frame as uniforms
//! - Vertex shader does the skinning math in parallel
//! - Only ~50-200 bones vs thousands of vertices
//! - Animation data compresses well (often 10:1 ratio)
//!
//! Memory optimization strategies:
//! - Quantize rotations to 16-bit quaternions (saves 50% memory)
//! - Store positions/scales at lower precision when possible
//! - Use animation compression (remove redundant keyframes)
//! - Stream animations on-demand for open world games
//!
//! ## Real-World Applications
//!
//! AAA games typically have:
//! - 200-500 animations per character
//! - 30-120 bones for main characters
//! - 15-30 bones for NPCs/enemies
//! - LOD systems that reduce bone count at distance
//!
//! Games like "The Last of Us" use advanced techniques:
//! - Motion matching for seamless transitions
//! - Layered animations (upper body aim + lower body walk)
//! - IK for foot placement on uneven terrain
//! - Facial bones for emotional expression
//!
//! ## Advanced Techniques
//!
//! Beyond basic playback:
//! - **Animation Blending**: Smooth transitions between animations
//! - **Additive Animation**: Layer animations (breathing + running)
//! - **Procedural Animation**: Physics-driven secondary motion
//! - **Motion Retargeting**: Share animations between different skeletons
//! - **Animation Compression**: Curve fitting, PCA, wavelets
//!
//! ## Common Issues and Solutions
//!
//! 1. **Animation Pops**: Sudden jumps between animations
//!    - Solution: Blend between animations over 0.1-0.3 seconds
//!
//! 2. **Foot Sliding**: Feet move when they should be planted
//!    - Solution: Use IK to lock feet to ground during contact
//!
//! 3. **Performance Spikes**: Too many animated characters
//!    - Solution: LOD system, animation culling, GPU skinning
//!
//! 4. **Memory Bloat**: Animations taking too much RAM
//!    - Solution: Compression, streaming, shared animation libraries

use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*, scene::SceneInstanceReady};

// The path to our animated fox model - a glTF file containing:
// - A mesh (the fox's 3D shape)
// - A skeleton (the bone structure)
// - Animations (pre-made movements like running, walking, etc.)
// glTF is like a zip file for 3D content - it packages everything together!
const GLTF_PATH: &str = "models/animated/Fox.glb";

fn main() {
    App::new()
        // Global ambient lighting - illuminates everything equally
        // High brightness (2000) because our fox model has dark textures
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 2000.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // These systems run once at startup:
        .add_systems(Startup, setup_mesh_and_animation)    // Load the fox and its animations
        .add_systems(Startup, setup_camera_and_environment) // Create the scene around it
        .run();
}

// A component that stores animation data we'll need later.
// This is a common pattern in Bevy: store data during loading,
// use it when assets are ready. It's like writing a note to your future self!
//
// ## Rust Pattern: Newtype Pattern with Component
// The #[derive(Component)] macro generates the necessary trait implementations
// to make this struct work with Bevy's ECS. Under the hood, it's implementing
// the Component trait which requires Send + Sync + 'static.
//
// ## Memory Layout
// This struct is optimized for cache performance:
// - Handle<T> is just a wrapper around Arc<StrongHandle> (8 bytes on 64-bit)
// - AnimationNodeIndex is likely a simple integer index (4-8 bytes)
// - Total size: ~16 bytes, fits nicely in a cache line
//
// ## Bevy Architecture: Asset Handles
// Handles are Bevy's way of referencing assets without owning them directly.
// They're reference-counted and automatically clean up when dropped.
// This prevents memory leaks and allows hot-reloading of assets.
#[derive(Component)]
struct AnimationToPlay {
    // The animation graph contains all our animation clips and how they connect
    graph_handle: Handle<AnimationGraph>,
    // The index tells us which specific animation to play within the graph
    index: AnimationNodeIndex,
}

fn setup_mesh_and_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // ## Rust Pattern: System Parameters
    // Bevy systems use dependency injection through their parameters:
    // - Commands: For spawning/despawning entities (mutable world access)
    // - Res<T>: Immutable access to a global resource
    // - ResMut<T>: Mutable access to a global resource
    // The 'mut' keyword on parameters means we can mutate the wrapper, not the resource itself
    
    // LOADING ANIMATIONS FROM glTF
    // glTF files can contain multiple animations. Our fox has:
    // - Animation 0: "Survey" (looking around)
    // - Animation 1: "Walk" 
    // - Animation 2: "Run" (what we'll use)
    // We use GltfAssetLabel to pick a specific animation by index
    //
    // ## Animation Data Structure in glTF
    // glTF animations are stored as:
    // - Channels: What property to animate (translation, rotation, scale)
    // - Samplers: The keyframe data (times and values)
    // - Targets: Which node/bone to apply the animation to
    //
    // ## Rust Pattern: Tuple Destructuring
    // from_clip returns (AnimationGraph, AnimationNodeIndex)
    // We immediately destructure it into named variables for clarity
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(2).from_asset(GLTF_PATH)),
    );

    // Store the animation graph as an asset - this is Bevy's way of managing resources
    // The handle is like a ticket that we can exchange for the actual data later
    let graph_handle = graphs.add(graph);

    // Package our animation data for later use
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    // LOADING THE 3D MODEL
    // SceneRoot is a special component that tells Bevy:
    // "When this asset loads, spawn its entire scene hierarchy as my children"
    // Scene 0 typically contains the main mesh and skeleton
    let mesh_scene = SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(GLTF_PATH)));

    // Here's the clever part: we spawn an entity with both the scene and our animation data,
    // then attach an observer that waits for the scene to finish loading.
    // It's like setting up a domino effect - when loading completes, animation starts!
    commands
        .spawn((animation_to_play, mesh_scene))
        .observe(play_animation_when_ready);
}

// This function is triggered by the SceneInstanceReady event - it's like a callback
// that says "The 3D model is loaded and spawned, now what?"
//
// ## Bevy Architecture: Observer Pattern
// Observers are Bevy's reactive programming model:
// - More efficient than checking every frame in Update systems
// - Automatically cleaned up when the observed entity is despawned
// - Can observe multiple event types on the same entity
//
// ## Game Feel: Timing is Everything
// Starting animations at the right moment is crucial for game feel:
// - Too early: Animation plays on invisible/loading model (looks broken)
// - Too late: Model appears frozen before animating (feels sluggish)
// - Just right: Model appears already in motion (feels alive!)
fn play_animation_when_ready(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    // The trigger tells us which entity just finished loading its scene
    // This is the entity we created in setup_mesh_and_animation
    if let Ok(animation_to_play) = animations_to_play.get(trigger.target()) {
        // When a glTF scene spawns, it creates a hierarchy like this:
        // - Our entity (with SceneRoot)
        //   - Fox mesh entity
        //     - AnimationPlayer component (automatically added for animated models)
        //   - Bone entities (the skeleton)
        //   - Camera/Light entities (if the glTF contained any)
        //
        // We need to find the AnimationPlayer in this hierarchy
        for child in children.iter_descendants(trigger.target()) {
            if let Ok(mut player) = players.get_mut(child) {
                // Found it! Now we can control the animation playback
                // play() starts the animation, repeat() makes it loop forever
                // Without repeat(), it would play once and stop
                //
                // ## Animation Playback Control
                // The AnimationPlayer provides several methods:
                // - play(): Start from beginning
                // - play_from(): Start from specific time
                // - pause(): Freeze at current frame
                // - resume(): Continue from paused state
                // - set_speed(): Control playback rate (0.5 = half speed, 2.0 = double)
                //
                // ## Rust Pattern: Method Chaining
                // play() returns &mut AnimationPlayer, allowing us to chain .repeat()
                // This is the builder pattern - each method returns self for fluent API
                player.play(animation_to_play.index).repeat();

                // Connect the animation graph to the player
                // This tells the player what animations are available
                // Think of it like loading a DVD into a DVD player
                //
                // ## Memory Management: Clone vs Reference
                // We clone the handle here because:
                // - Handles are cheap to clone (just increments ref count)
                // - The original entity might be despawned before animation finishes
                // - Each AnimationPlayer needs its own handle for independent control
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}

// Creates the environment where our fox will run
// A good scene needs three things: something to see (the fox),
// somewhere to see it from (camera), and light to see it by!
fn setup_camera_and_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // CAMERA SETUP
    // Position the camera to get a nice view of our fox
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(100.0, 100.0, 150.0)  // Camera position: right, up, and back
            .looking_at(Vec3::new(0.0, 20.0, 0.0), Vec3::Y), // Look at fox's center, Y is up
    ));

    // GROUND PLANE
    // A huge green plane for our fox to run on
    commands.spawn((
        // Create a plane mesh - size is huge (500k x 500k) so it looks infinite
        Mesh3d(meshes.add(Plane3d::default().mesh().size(500000.0, 500000.0))),
        // Give it a grassy green color
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // DIRECTIONAL LIGHT (like the sun)
    commands.spawn((
        // Rotate the light to cast shadows at an angle
        // Euler angles: think of them as yaw, pitch, and roll of an airplane
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true, // Cast shadows for more realism
            ..default()
        },
        // Cascade shadow mapping - a technique for high-quality shadows at different distances
        // Think of it like having multiple shadow maps: detailed close up, less detailed far away
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,  // High detail shadows up to 200 units
            maximum_distance: 400.0,         // Shadows visible up to 400 units
            ..default()
        }
        .build(),
    ));
}
