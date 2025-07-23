//! Skinned mesh example with mesh and joints data defined in code.
//! Example taken from <https://github.com/KhronosGroup/glTF-Tutorials/blob/master/gltfTutorial/gltfTutorial_019_SimpleSkin.md>
//!
//! Skinned meshes are the foundation of character animation in 3D games!
//! Imagine a skeleton inside a rubber suit - as the bones move, the rubber stretches
//! and deforms naturally. This example creates a simple "arm" from scratch:
//! a rectangular mesh controlled by two joints (bones) that can bend and twist.
//! Each vertex is influenced by one or both joints based on weight values.
//!
//! ## Animation Theory: Skeletal Animation Deep Dive
//!
//! Skeletal animation (skinning) revolutionized character animation:
//! - **Linear Blend Skinning (LBS)**: Weighted average of bone transforms
//! - **Dual Quaternion Skinning**: Better volume preservation
//! - **Inverse Kinematics**: Solve bone positions from end goals
//! - **Forward Kinematics**: What this example demonstrates
//!
//! The skinning equation:
//! ```text
//! vertex_world = Σ(weight[i] * joint[i] * bindpose_inv[i] * vertex_local)
//! ```
//!
//! Key concepts:
//! 1. **Bind Pose**: The "rest" position of bones and mesh
//! 2. **Inverse Bind Pose**: Transforms from mesh space to bone space
//! 3. **Weights**: How much each bone influences each vertex
//! 4. **Joint Hierarchy**: Parent-child relationships between bones
//!
//! ## Game Feel Context: Character Deformation
//!
//! Skinning creates believable character motion:
//! - **Organic Movement**: Smooth deformation, no rigid parts
//! - **Joint Limits**: Real anatomy constraints
//! - **Secondary Motion**: Fat, muscle, cloth jiggle
//! - **Squash and Stretch**: Cartoon-style deformation
//!
//! This example demonstrates:
//! - Basic 2-bone chain (like an arm or leg)
//! - Weight painting visualization
//! - Different transform effects on bones
//! - How hierarchy affects deformation
//!
//! ## Performance Optimization: GPU Skinning
//!
//! Skinning performance considerations:
//! 1. **GPU Skinning**: Transform vertices on GPU (massive parallelism)
//! 2. **Bone Limits**: Most GPUs handle 4 bones per vertex efficiently
//! 3. **LOD Systems**: Fewer bones for distant characters
//! 4. **Instancing**: Share skeletons between multiple characters
//! 5. **Compute Skinning**: Use compute shaders for complex deformations
//!
//! Memory optimization:
//! - Pack bone indices into bytes (256 bones max)
//! - Quantize weights (8-bit often sufficient)
//! - Share bind poses between similar characters
//! - Stream animation data on demand
//!
//! ## Real-World Applications: AAA Skinning
//!
//! Industry examples:
//! - **GTA V**: 100+ bones per character, facial bones
//! - **The Last of Us**: Muscle simulation on top of skinning
//! - **Horizon Zero Dawn**: Machine creatures with 200+ bones
//! - **FIFA**: Cloth simulation integrated with skinning
//!
//! Common bone counts:
//! - Mobile game character: 15-30 bones
//! - Console game character: 50-100 bones
//! - Facial rig: 50+ bones just for face
//! - Hero character: 200+ bones including fingers
//!
//! ## Advanced Techniques: Beyond LBS
//!
//! 1. **Blend Shapes**: Vertex-level deformation for faces
//! 2. **Muscle Systems**: Anatomically correct deformation
//! 3. **Wrinkle Maps**: Dynamic normal maps based on joint angles
//! 4. **Volume Preservation**: Maintain character volume during bends
//! 5. **Helper Bones**: Extra bones for better deformation
//!
//! ## Common Issues and Solutions
//!
//! 1. **Candy Wrapper Effect**: Twisting creates pinching
//!    - Solution: Use dual quaternion skinning
//!
//! 2. **Vertex Weights Don't Sum to 1.0**: Incorrect deformation
//!    - Solution: Normalize weights after painting
//!
//! 3. **Bone Hierarchy Wrong**: Unexpected movement
//!    - Solution: Visualize skeleton, check parent-child
//!
//! 4. **Performance Issues**: Too many bones
//!    - Solution: LOD system, GPU skinning

use std::f32::consts::*; // Mathematical constants like PI, FRAC_PI_2

use bevy::{
    math::ops, // Floating-point operations like sin, cos
    prelude::*,
    render::{
        mesh::{
            skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
            Indices, PrimitiveTopology, VertexAttributeValues,
        },
        render_asset::RenderAssetUsages,
    },
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng; // Deterministic random number generator

// ## Rust Pattern: Import Organization
// Imports are organized hierarchically:
// 1. Standard library (std)
// 2. External crates (bevy, rand)
// 3. Local modules (none in this example)
// The math::ops import provides float operations as free functions

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Bright ambient light so we can see our meshes clearly
        .insert_resource(AmbientLight {
            brightness: 3000.0,
            ..default()
        })
        .add_systems(Startup, setup)           // Create the skinned meshes
        .add_systems(Update, joint_animation)  // Animate the joints each frame
        .run();
}

/// Marks a joint (bone) for animation in the [`joint_animation`] system.
/// The value determines which animation pattern to apply.
///
/// ## Component Design
/// Using isize allows negative values for different animation categories:
/// - Negative: Rotation and scale animations
/// - Zero/Positive: Translation animations
/// This creates visual groupings in the demo
#[derive(Component)]
struct AnimatedJoint(isize);

/// Creates multiple skinned meshes, each with 2 joints forming a simple "arm".
/// Each mesh demonstrates a different animation pattern.
/// The mesh geometry and skinning data are created programmatically.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skinned_mesh_inverse_bindposes_assets: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    // Camera setup - positioned to view all the skinned meshes
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.5, 2.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // INVERSE BIND POSE MATRICES
    // These matrices define the "rest position" of each joint relative to the mesh.
    // When a joint is at its bind pose, it has no effect on the mesh.
    // The inverse bind pose transforms from mesh space to joint space.
    // Here, both joints start at position (-0.5, -1.0, 0.0) in the mesh's local space.
    //
    // ## Mathematical Deep Dive: Inverse Bind Pose
    // The inverse bind pose is crucial for skinning math:
    // 1. Mesh vertices are defined in mesh space
    // 2. Joint transforms are in world space
    // 3. To apply joint transforms to vertices, we need to:
    //    a) Transform vertex from mesh space to joint space (inverse bind pose)
    //    b) Apply the joint's current transform
    //    c) Result is vertex in world space
    //
    // The translation (-0.5, -1.0, 0.0) centers the mesh on the joints
    // X: -0.5 centers the 1-unit wide mesh
    // Y: -1.0 places joint at bottom of 2-unit tall mesh
    let inverse_bindposes = skinned_mesh_inverse_bindposes_assets.add(vec![
        Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)), // Joint 0 (base)
        Mat4::from_translation(Vec3::new(-0.5, -1.0, 0.0)), // Joint 1 (tip)
    ]);

    // CREATE THE MESH GEOMETRY
    // ## Mesh Building Pattern
    // Bevy uses a builder pattern for meshes:
    // 1. Create base mesh with topology
    // 2. Add attributes (positions, normals, UVs, skinning data)
    // 3. Add indices for triangle definitions
    // This approach is memory efficient and GPU-friendly
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList, // We'll define triangles
        RenderAssetUsages::RENDER_WORLD,  // Used for rendering
    )
    // VERTEX POSITIONS
    // Creates a 1x2 unit rectangle divided into 4 segments vertically.
    // This forms our "arm" that will bend at the joints.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.0, 0.0], // Bottom-left
            [1.0, 0.0, 0.0], // Bottom-right
            [0.0, 0.5, 0.0], // 25% up-left
            [1.0, 0.5, 0.0], // 25% up-right
            [0.0, 1.0, 0.0], // 50% up-left (middle)
            [1.0, 1.0, 0.0], // 50% up-right (middle)
            [0.0, 1.5, 0.0], // 75% up-left
            [1.0, 1.5, 0.0], // 75% up-right
            [0.0, 2.0, 0.0], // Top-left
            [1.0, 2.0, 0.0], // Top-right
        ],
    )
    // UV COORDINATES
    // Maps texture coordinates to vertices. We use only the left half (0.0-0.5)
    // of the texture horizontally since our mesh is tall and narrow.
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            [0.0, 0.00], // Bottom-left vertex -> bottom-left of texture
            [0.5, 0.00], // Bottom-right vertex -> bottom-middle of texture
            [0.0, 0.25],
            [0.5, 0.25],
            [0.0, 0.50],
            [0.5, 0.50],
            [0.0, 0.75],
            [0.5, 0.75],
            [0.0, 1.00], // Top-left vertex -> top-left of texture
            [0.5, 1.00], // Top-right vertex -> top-middle of texture
        ],
    )
    // VERTEX NORMALS
    // All vertices face forward (+Z direction) for proper lighting
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 10])
    
    // JOINT INDICES - Which bones affect each vertex
    // Each vertex can be influenced by up to 4 joints (bones).
    // Here we use only 2 joints max per vertex.
    //
    // ## GPU Skinning Limits
    // Why 4 joints per vertex?
    // - GPU vertex shaders have limited uniform/attribute slots
    // - 4 provides good quality vs performance balance
    // - Most vertices need only 1-2 joints
    // - Complex areas (shoulders, hips) may use all 4
    //
    // ## Vertex Attribute Format
    // Uint16x4 = four 16-bit unsigned integers
    // - Supports up to 65,536 bones (plenty!)
    // - Unused slots must be 0 (points to first joint)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        // Format: [joint0, joint1, joint2, joint3] - unused slots are 0
        VertexAttributeValues::Uint16x4(vec![
            [0, 0, 0, 0], // Vertex 0-1: Only affected by joint 0 (base)
            [0, 0, 0, 0],
            [0, 1, 0, 0], // Vertex 2-3: Affected by both joints
            [0, 1, 0, 0],
            [0, 1, 0, 0], // Vertex 4-5: Affected by both joints
            [0, 1, 0, 0],
            [0, 1, 0, 0], // Vertex 6-7: Affected by both joints
            [0, 1, 0, 0],
            [0, 1, 0, 0], // Vertex 8-9: Affected by both joints
            [0, 1, 0, 0],
        ]),
    )
    // JOINT WEIGHTS - How much each joint influences each vertex
    // Weights must sum to 1.0 for each vertex.
    // This creates a smooth blend from joint 0 to joint 1 along the mesh height.
    //
    // ## Weight Painting in Production
    // Artists usually "paint" these weights visually:
    // - Red = 100% influence
    // - Blue = 0% influence
    // - Gradient = smooth blend
    //
    // ## Mathematical Importance
    // Weights MUST sum to 1.0 because:
    // - Ensures vertices stay attached to skeleton
    // - Prevents scaling artifacts
    // - Maintains mesh volume
    //
    // ## Blend Falloff
    // Our linear falloff (1.0 -> 0.75 -> 0.5 -> 0.25 -> 0.0) creates
    // smooth deformation. Sharp transitions would create creases!
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_WEIGHT,
        vec![
            [1.00, 0.00, 0.0, 0.0], // Bottom vertices: 100% joint 0
            [1.00, 0.00, 0.0, 0.0],
            [0.75, 0.25, 0.0, 0.0], // 75% joint 0, 25% joint 1
            [0.75, 0.25, 0.0, 0.0],
            [0.50, 0.50, 0.0, 0.0], // Middle: 50/50 blend
            [0.50, 0.50, 0.0, 0.0],
            [0.25, 0.75, 0.0, 0.0], // 25% joint 0, 75% joint 1
            [0.25, 0.75, 0.0, 0.0],
            [0.00, 1.00, 0.0, 0.0], // Top vertices: 100% joint 1
            [0.00, 1.00, 0.0, 0.0],
        ],
    )
    // TRIANGLE INDICES
    // Define triangles using vertex indices. Each group of 3 forms a triangle.
    // We create 8 triangles (2 per segment) to form the rectangular mesh.
    .with_inserted_indices(Indices::U16(vec![
        // Bottom segment
        0, 1, 3,  // Lower triangle
        0, 3, 2,  // Upper triangle
        // Lower-middle segment
        2, 3, 5,
        2, 5, 4,
        // Upper-middle segment
        4, 5, 7,
        4, 7, 6,
        // Top segment
        6, 7, 9,
        6, 9, 8,
    ]));

    // Store the mesh as an asset
    let mesh = meshes.add(mesh);

    // Seeded RNG for consistent random colors
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // CREATE 10 SKINNED MESHES WITH DIFFERENT ANIMATIONS
    for i in -5..5 {
        // JOINT HIERARCHY
        // Joint 0: The base/root joint
        //
        // ## Transform Hierarchy in Skinning
        // Parent-child relationships are crucial:
        // - Child inherits parent's transform
        // - Rotating parent rotates entire chain
        // - Just like your shoulder rotating moves your whole arm!
        //
        // ## Bevy's Entity Hierarchy
        // Transform propagation happens automatically:
        // world_transform = parent_world * local_transform
        let joint_0 = commands
            .spawn(Transform::from_xyz(
                i as f32 * 1.5,  // Spread meshes horizontally
                0.0,
                // Small Z offset to prevent overlapping with debug gizmos
                -(i as f32 * 0.01).abs(),
            ))
            .id();
        
        // Joint 1: The tip joint, child of joint 0
        // AnimatedJoint component marks it for animation
        // Transform::IDENTITY means no local offset from parent
        let joint_1 = commands.spawn((AnimatedJoint(i), Transform::IDENTITY)).id();

        // Create parent-child relationship
        // This means joint_1 inherits joint_0's transform
        commands.entity(joint_0).add_children(&[joint_1]);

        // The order here must match the inverse bindpose array
        let joint_entities = vec![joint_0, joint_1];

        // SPAWN THE SKINNED MESH
        // The mesh position is determined by its joints, not its own transform!
        //
        // ## Skinned Mesh Architecture
        // Unlike regular meshes, skinned meshes:
        // 1. Don't use their own Transform for vertices
        // 2. Vertex positions come from joint transforms
        // 3. The mesh entity transform affects the whole model
        //
        // ## Texture as Deformation Visualization
        // The checker pattern reveals how skinning deforms the mesh:
        // - Straight lines show no deformation
        // - Curved lines show bending
        // - Stretched squares show scaling
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                // Random color for each mesh
                base_color: Color::srgb(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ),
                // Checker texture helps visualize deformation
                base_color_texture: Some(asset_server.load("textures/uv_checker_bw.png")),
                ..default()
            })),
            // The SkinnedMesh component links the mesh to its skeleton
            SkinnedMesh {
                inverse_bindposes: inverse_bindposes.clone(),
                joints: joint_entities, // References to our joint entities
            },
        ));
    }
}

/// Animates joints based on their ID, demonstrating different transform effects.
/// Each animation shows how joint transforms affect the skinned mesh.
///
/// ## Animation Showcase Design
/// This function demonstrates all possible joint transformations:
/// - Rotation: How joints typically move in games
/// - Scale: Useful for squash-and-stretch effects
/// - Translation: For sliding/extending motions
///
/// ## Sine Wave Animation
/// We use sine waves for smooth, continuous motion:
/// - Period = 2π seconds ≈ 6.28 seconds
/// - No discontinuities or pops
/// - Easy to see the full range of motion
fn joint_animation(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &AnimatedJoint)>,
    mut gizmos: Gizmos,
) {
    for (mut transform, animated_joint) in &mut query {
        // Different animation patterns based on joint ID
        match animated_joint.0 {
            // ROTATION ANIMATIONS
            -5 => {
                // Rotate around X axis (pitch) - bends forward/backward
                // ## Anatomical Reference: Elbow or knee bend
                // Range: ±90° (FRAC_PI_2 radians)
                // This is how most hinged joints work in games
                transform.rotation =
                    Quat::from_rotation_x(FRAC_PI_2 * ops::sin(time.elapsed_secs()));
            }
            -4 => {
                // Rotate around Y axis (yaw) - twists left/right
                // ## Anatomical Reference: Forearm twist
                // Used for actions like turning doorknobs
                // Most joints have limited twist range
                transform.rotation =
                    Quat::from_rotation_y(FRAC_PI_2 * ops::sin(time.elapsed_secs()));
            }
            -3 => {
                // Rotate around Z axis (roll) - bends side to side
                // ## Anatomical Reference: Wrist deviation
                // Less common in real skeletons but useful for:
                // - Tentacles, tails, spines
                // - Cartoon deformations
                transform.rotation =
                    Quat::from_rotation_z(FRAC_PI_2 * ops::sin(time.elapsed_secs()));
            }
            // SCALE ANIMATIONS
            -2 => {
                // Scale X - stretches horizontally
                // +1.0 ensures scale stays positive (0.0 to 2.0)
                // ## Use Cases for Bone Scaling:
                // - Breathing (ribcage expansion)
                // - Muscle flexing (bicep bulge)
                // - Cartoon squash/stretch
                // - Growing/shrinking effects
                // Note: Most realistic rigs avoid bone scaling
                transform.scale.x = ops::sin(time.elapsed_secs()) + 1.0;
            }
            -1 => {
                // Scale Y - stretches vertically
                // ## Squash and Stretch Principle
                // Classic animation technique:
                // - Stretch on fast motion
                // - Squash on impact
                // Maintains volume for believability
                transform.scale.y = ops::sin(time.elapsed_secs()) + 1.0;
            }
            // TRANSLATION ANIMATIONS
            0 => {
                // Circular motion in XY plane
                // ## Use Case: Shoulder rotation
                // Simulates ball-and-socket joint movement
                // Radius = 0.5 keeps motion subtle
                transform.translation.x = 0.5 * ops::sin(time.elapsed_secs());
                transform.translation.y = ops::cos(time.elapsed_secs());
            }
            1 => {
                // Circular motion in YZ plane
                // ## Use Case: Hip sway
                // Common in walk cycles
                // Creates figure-8 motion when combined with forward movement
                transform.translation.y = ops::sin(time.elapsed_secs());
                transform.translation.z = ops::cos(time.elapsed_secs());
            }
            2 => {
                // Side-to-side motion
                // ## Use Case: Sliding joints
                // - Pistons, hydraulics
                // - Telescoping limbs
                // - Procedural tentacles
                transform.translation.x = ops::sin(time.elapsed_secs());
            }
            3 => {
                // Combined vertical motion and horizontal scaling
                // ## Use Case: Cartoon "boing" effect
                // Demonstrates how multiple transforms combine
                // Creates bouncing, stretchy motion
                transform.translation.y = ops::sin(time.elapsed_secs());
                transform.scale.x = ops::sin(time.elapsed_secs()) + 1.0;
            }
            _ => (), // Static joints
        }
        
        // VISUAL DEBUG: Draw coordinate axes at each joint position
        // ## Gizmo Visualization
        // Shows the local coordinate system of each joint:
        // - Red = X axis (right)
        // - Green = Y axis (up)
        // - Blue = Z axis (forward)
        // This helps understand how rotations affect the joint
        //
        // ## Why Offset the Gizmo?
        // The joint's actual position includes the mesh spread (i * 1.5)
        // We add this offset so gizmos appear at the visual joint location
        let mut axis = *transform;
        axis.translation.x += animated_joint.0 as f32 * 1.5; // Offset to match mesh position
        gizmos.axes(axis, 1.0); // Size = 1.0
    }
}
