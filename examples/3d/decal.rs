//! Decal rendering.
//!
//! # What Are Forward Decals?
//!
//! Forward decals are a simpler alternative to clustered decals. While clustered decals
//! use a spatial clustering system for efficient rendering of many decals, forward decals
//! are rendered in a more straightforward manner during the forward rendering pass.
//!
//! Think of forward decals like projectors that cast an image onto surfaces. They're great
//! for:
//! - Blood splatters or damage marks in games
//! - Graffiti or posters on walls
//! - Tire tracks or footprints
//! - Any temporary visual effect that needs to appear "on top" of existing geometry
//!
//! # Key Concepts in This Example
//!
//! 1. **Depth Prepass**: Forward decals require depth information to properly blend with
//!    the scene. The DepthPrepass component on the camera enables this.
//!
//! 2. **ForwardDecalMaterial**: A special material type that extends standard materials
//!    with decal-specific features like depth fade.
//!
//! 3. **Projection**: The decal acts like a cube that projects its texture onto any
//!    geometry it intersects. The decal's transform controls its position and size.
//!
//! # Graphics Theory: Forward vs Deferred Decal Rendering
//!
//! ## Rendering Pipeline Integration
//!
//! Forward decals integrate into the forward rendering pipeline:
//! ```text
//! 1. Depth Prepass: Generate depth buffer
//! 2. Forward Pass: For each pixel:
//!    a. Render opaque geometry
//!    b. Test decal bounds
//!    c. If inside decal: project UV and blend
//! 3. Transparency Pass: Render transparent objects
//! ```
//!
//! ## Mathematical Foundation
//!
//! The decal projection mathematics:
//! ```text
//! World Space → Decal Space → UV Space
//! P_decal = M_decal^-1 × P_world
//! UV = (P_decal.xy + 0.5) × scale_factor
//! Depth Test: |P_decal.z| < 0.5
//! ```
//!
//! ## Performance Characteristics
//!
//! Forward decals vs Clustered decals:
//! - **Forward**: O(N × M) where N = pixels, M = decals
//! - **Clustered**: O(N × K) where K = decals per cluster
//! - **Memory**: Forward uses less memory (no cluster buffers)
//! - **Drawcalls**: Each forward decal = 1 drawcall
//!
//! # Game Design Context: When to Use Forward Decals
//!
//! ## Use Cases
//!
//! Forward decals excel in specific scenarios:
//! 1. **Few Decals**: < 10 decals visible at once
//! 2. **Large Decals**: Cover significant screen space
//! 3. **Dynamic Decals**: Frequently created/destroyed
//! 4. **Mobile/WebGL**: Limited GPU feature support
//!
//! ## Visual Design Patterns
//!
//! Common game decal strategies:
//! - **Damage Indicators**: Show where player was hit
//! - **Navigation**: Subtle arrows guide players
//! - **Environmental Story**: Age, wear, history
//! - **Gameplay Feedback**: Ability ranges, AoE indicators
//!
//! ## Memory Budgeting
//!
//! Typical AAA game allocations:
//! - Decal textures: 64-128MB
//! - Decal instances: 100-500 active
//! - Atlas resolution: 2048×2048 or 4096×4096
//!
//! # Performance Deep Dive: GPU Architecture Impact
//!
//! ## Bandwidth Analysis
//!
//! Forward decals stress different GPU subsystems:
//! ```text
//! Per Pixel Cost:
//! - Depth read: 4 bytes
//! - Texture fetch: 4-16 bytes (with mips)
//! - Blend operation: 16 bytes (read + write)
//! Total: ~24-36 bytes per pixel per decal
//! ```
//!
//! ## GPU Architecture Considerations
//!
//! ### Tile-Based Renderers (Mobile/Apple Silicon)
//! - Depth prepass fits in on-chip tile memory
//! - Blending happens in fast tile memory
//! - Excellent for forward decals!
//!
//! ### Immediate Mode Renderers (Desktop)
//! - Depth reads from VRAM (slower)
//! - Consider clustered decals for many decals
//! - Z-buffer compression helps
//!
//! ## Optimization Strategies
//!
//! 1. **Frustum Culling**: Skip off-screen decals
//! 2. **LOD System**: Reduce quality at distance
//! 3. **Temporal Pooling**: Reuse decal instances
//! 4. **Atlas Packing**: Minimize texture switches
//!
//! # Real-World Applications: Production Examples
//!
//! ## Game Engine Implementations
//!
//! ### Unreal Engine 4/5
//! - Deferred decals by default
//! - Forward decals for mobile
//! - Mesh decals for complex geometry
//!
//! ### Unity Built-in/URP
//! - Forward decals in URP
//! - Screen-space decals in HDRP
//! - Projector component (legacy)
//!
//! ### Godot 4
//! - Forward+ renderer with decal support
//! - Clustered forward implementation
//!
//! ## Notable Games
//!
//! - **Left 4 Dead 2**: Dynamic blood system
//! - **Splatoon**: Entire gameplay around decals!
//! - **Portal 2**: Gel splatter mechanics
//!
//! # Advanced Techniques: Beyond Basic Projection
//!
//! ## Multi-Layer Decals
//!
//! Stack multiple decals for complex effects:
//! ```glsl
//! // Pseudo-code for layered decals
//! vec4 final_color = base_color;
//! for (int i = 0; i < num_decals; i++) {
//!     vec4 decal = sample_decal(i, uv);
//!     final_color = blend(final_color, decal, blend_modes[i]);
//! }
//! ```
//!
//! ## Normal Map Blending
//!
//! Proper normal blending for lighting:
//! ```glsl
//! // Reoriented Normal Mapping (RNM)
//! vec3 blend_rnm(vec3 n1, vec3 n2) {
//!     vec3 t = n1.xyz + vec3(0.0, 0.0, 1.0);
//!     vec3 u = n2.xyz * vec3(-1.0, -1.0, 1.0);
//!     return normalize(t * dot(t, u) - u * t.z);
//! }
//! ```
//!
//! ## Parallax Occlusion Decals
//!
//! Add depth to decals without geometry:
//! ```glsl
//! vec2 parallax_mapping(vec2 uv, vec3 view_dir) {
//!     float height = texture(height_map, uv).r;
//!     vec2 p = view_dir.xy * (height * height_scale);
//!     return uv - p;
//! }
//! ```
//!
//! # Debugging and Profiling: Developer Tools
//!
//! ## Visual Debug Modes
//!
//! Common debug visualizations:
//! 1. **Decal Bounds**: Wire boxes showing projection volume
//! 2. **UV Visualization**: Color-coded UV coordinates
//! 3. **Overdraw**: Heatmap of pixel coverage
//! 4. **Mip Levels**: Color by texture LOD
//!
//! ## Performance Profiling
//!
//! Key metrics to track:
//! - **GPU Time**: Target < 0.5ms for decal pass
//! - **Overdraw Factor**: Keep below 2x
//! - **Texture Bandwidth**: Monitor cache misses
//! - **Draw Calls**: Batch when possible
//!
//! ## Common Issues and Solutions
//!
//! 1. **Z-Fighting**
//!    - Solution: Add small depth bias (0.001)
//!    - Use depth fade for soft edges
//!
//! 2. **Texture Bleeding**
//!    - Solution: Add padding in texture atlas
//!    - Use border clamping
//!
//! 3. **Performance Spikes**
//!    - Solution: Implement LOD system
//!    - Cull distant decals
//!
//! 4. **Sorting Issues**
//!    - Solution: Sort by depth
//!    - Use order-independent transparency
//!
//! ## Rust-Specific Optimizations
//!
//! Leverage Rust's zero-cost abstractions:
//! ```rust
//! // Efficient decal culling with iterators
//! visible_decals = decals.iter()
//!     .filter(|d| frustum.contains(&d.bounds))
//!     .filter(|d| d.distance < max_distance)
//!     .take(MAX_VISIBLE_DECALS)
//!     .collect();
//! ```

// Import a helper module for camera controls
// The #[path] attribute tells Rust exactly where to find this module
#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

use bevy::{
    // DepthPrepass enables depth buffer generation before main rendering
    // This is required for forward decals to properly interact with geometry
    core_pipeline::prepass::DepthPrepass,
    // Forward decal components and materials
    pbr::decal::{
        ForwardDecal,           // Component that marks an entity as a decal
        ForwardDecalMaterial,   // Material wrapper for decals
        ForwardDecalMaterialExt,// Extension with decal-specific properties
    },
    prelude::*,
};
// Camera controller for interactive camera movement
use camera_controller::{CameraController, CameraControllerPlugin};
// Random number generation for placing cubes
use rand::{Rng, SeedableRng};
// ChaCha8 is a cryptographically secure RNG that's deterministic with a seed
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        // DefaultPlugins provides rendering, windowing, input, etc.
        // CameraControllerPlugin adds WASD + mouse camera controls
        .add_plugins((DefaultPlugins, CameraControllerPlugin))
        // Setup system runs once at startup to create our scene
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    // Commands for spawning entities
    mut commands: Commands,
    // Asset storage for 3D meshes
    mut meshes: ResMut<Assets<Mesh>>,
    // Asset storage for standard PBR materials
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    // Asset storage for decal materials - note the ForwardDecalMaterial wrapper
    mut decal_standard_materials: ResMut<Assets<ForwardDecalMaterial<StandardMaterial>>>,
    // For loading textures from files
    asset_server: Res<AssetServer>,
) {
    // Spawn the forward decal
    // A decal is essentially an invisible box that projects its texture onto
    // any geometry it intersects
    //
    // ## GPU Execution Model
    //
    // When the GPU renders this decal:
    // 1. Vertex shader transforms the decal box vertices
    // 2. Rasterizer determines which pixels are covered
    // 3. Fragment shader for each pixel:
    //    a. Read depth buffer to get world position
    //    b. Transform world position to decal space
    //    c. If inside unit cube, sample texture
    //    d. Blend with framebuffer
    //
    // ## Memory Layout
    //
    // The decal data in GPU memory:
    // ```text
    // Decal Uniform Buffer:
    // [0-63]:   Transform matrix (4x4 float)
    // [64-79]:  Inverse transform (4x4 float) 
    // [80-83]:  Depth fade parameters
    // [84-95]:  Material properties
    // ```
    commands.spawn((
        // Name component for easier identification in debugging tools
        Name::new("Decal"),
        // ForwardDecal marker component - this tells Bevy to render this as a decal
        ForwardDecal,
        // The decal material wraps a standard material
        MeshMaterial3d(decal_standard_materials.add(ForwardDecalMaterial {
            // The base material defines the appearance (texture, color, etc.)
            base: StandardMaterial {
                // UV checker pattern - great for visualizing how the decal projects
                base_color_texture: Some(asset_server.load("textures/uv_checker_bw.png")),
                ..default()
            },
            // Decal-specific extensions
            extension: ForwardDecalMaterialExt {
                // Controls how the decal fades out at its edges
                // 1.0 means full opacity throughout
                //
                // ## Depth Fade Mathematics
                //
                // The fade is calculated as:
                // ```glsl
                // float scene_depth = texture(depth_buffer, screen_uv).r;
                // float decal_depth = gl_FragCoord.z;
                // float depth_diff = abs(scene_depth - decal_depth);
                // float fade = saturate(depth_diff * depth_fade_factor);
                // ```
                //
                // This prevents hard edges where the decal intersects geometry,
                // creating a softer, more natural blend.
                depth_fade_factor: 1.0,
            },
        })),
        // Scale the decal to 4x4x4 units
        // The decal projects from a cube, so this defines the projection volume
        //
        // ## Transform Optimization
        //
        // Uniform scale (splat) is more efficient than non-uniform:
        // - Simplified inverse calculation
        // - No normal re-normalization needed
        // - Better for GPU's matrix units
        Transform::from_scale(Vec3::splat(4.0)),
    ));

    commands.spawn((
        Name::new("Camera"),
        // Standard 3D camera with perspective projection
        Camera3d::default(),
        // Enables WASD movement and mouse look
        CameraController::default(),
        // CRITICAL: DepthPrepass must be enabled for forward decals to work!
        // This generates a depth buffer before the main render pass, allowing
        // decals to properly blend with and clip against scene geometry
        DepthPrepass,
        // Position camera above and to the side, looking at the origin
        Transform::from_xyz(2.0, 9.5, 2.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Create a shared white material for all geometry
    // Reusing materials is more efficient than creating unique ones
    let white_material = standard_materials.add(Color::WHITE);

    // Spawn a floor plane for the decal to project onto
    commands.spawn((
        Name::new("Floor"),
        // Rectangle creates a square plane, from_length(10.0) makes it 10x10 units
        Mesh3d(meshes.add(Rectangle::from_length(10.0))),
        // Reuse the white material
        MeshMaterial3d(white_material.clone()),
        // Rotate 90 degrees around X axis to make the plane horizontal
        // Rectangles are created in the XY plane, we need it in the XZ plane
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // Spawn a grid of randomly rotated cubes to showcase how decals behave
    // with non-flat geometry. The decal will project onto these cubes,
    // wrapping around their surfaces.
    let num_obs = 10;
    // Use a seeded RNG for reproducible "randomness"
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467713);
    
    // Create a 10x10 grid of cubes
    //
    // ## Scene Complexity and GPU Performance
    //
    // This creates 100 cubes, each with:
    // - 8 vertices × 3 floats = 96 bytes vertex data
    // - 12 triangles × 3 indices = 36 indices
    // - Total per cube: ~200 bytes GPU memory
    //
    // The random rotations ensure varied surface normals,
    // which tests how decals handle different surface orientations.
    //
    // ## Instancing Optimization Note
    //
    // In a production game, these identical cubes would use GPU instancing:
    // - 1 mesh uploaded once
    // - 100 transform matrices in a buffer
    // - Single draw call instead of 100
    // Bevy automatically uses instancing for identical mesh/material combinations!
    for i in 0..num_obs {
        for j in 0..num_obs {
            // Generate a random rotation axis (3 floats between 0 and 1)
            let rotation_axis: [f32; 3] = rng.r#gen();
            let rotation_vec: Vec3 = rotation_axis.into();
            // Random rotation angle in degrees
            let rotation: u32 = rng.gen_range(0..360);
            
            // Position cubes in a grid centered around the origin
            // The formula (-num_obs + 1) / 2.0 + i centers the grid
            //
            // ## Spatial Layout for Decal Testing
            //
            // The grid layout tests different decal projection scenarios:
            // - Center cubes: Full decal coverage
            // - Edge cubes: Partial coverage, clipping
            // - Rotated faces: Oblique angle projection
            let transform = Transform::from_xyz(
                (-num_obs + 1) as f32 / 2.0 + i as f32,
                -0.2,  // Slightly below ground level so they intersect the floor
                (-num_obs + 1) as f32 / 2.0 + j as f32,
            )
            // Apply random rotation using axis-angle representation
            //
            // ## Quaternion vs Euler Angles
            //
            // We use quaternions (axis-angle converted to quat) because:
            // - No gimbal lock
            // - Smooth interpolation
            // - More efficient GPU representation (4 floats vs 9 for matrix)
            .with_rotation(Quat::from_axis_angle(
                rotation_vec.normalize_or_zero(), // Normalize the axis vector
                (rotation as f32).to_radians(),   // Convert degrees to radians
            ));

            commands.spawn((
                // Small cubes (0.6 units per side)
                Mesh3d(meshes.add(Cuboid::from_length(0.6))),
                MeshMaterial3d(white_material.clone()),
                transform,
            ));
        }
    }

    // Add a point light to illuminate the scene
    commands.spawn((
        Name::new("Light"),
        PointLight {
            // Enable shadows for more realistic lighting
            // This helps visualize how decals interact with lighting
            shadows_enabled: true,
            ..default()
        },
        // Position the light above and to the side of the scene
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
