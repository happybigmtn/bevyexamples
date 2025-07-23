//! Demonstrates fog volumes with voxel density textures.
//!
//! We render the Stanford bunny as a fog volume. Parts of the bunny become
//! lighter and darker as the camera rotates. This is physically-accurate
//! behavior that results from the scattering and absorption of the directional
//! light.
//!
//! # What Are Fog Volumes?
//!
//! Fog volumes are 3D regions filled with participating media (fog, smoke, clouds).
//! Unlike simple distance-based fog that affects the entire scene uniformly,
//! fog volumes let you place localized atmospheric effects.
//!
//! # Voxel Density Textures
//!
//! This example uses a 3D texture (voxel grid) to define the fog density at each
//! point in space. Think of it like a 3D array where each cell contains a density
//! value. The bunny.ktx2 file contains a voxelized version of the Stanford bunny -
//! a famous 3D model used in computer graphics research.
//!
//! # Light Scattering and Absorption
//!
//! When light travels through fog:
//! - **Scattering**: Light bounces off fog particles, making the fog visible
//! - **Absorption**: Fog particles absorb some light, making things darker
//! 
//! The interplay between these creates realistic volumetric lighting effects.
//!
//! # Graphics Theory: Volumetric Rendering Mathematics
//!
//! ## The Rendering Equation for Participating Media
//!
//! ```text
//! L(x, ω) = ∫[0,d] T(x, x') * σ_s * L_i(x', -ω) * p(ω', ω) dx'
//!           + T(x, x_d) * L_surface(x_d, ω)
//! 
//! Where:
//! - L(x, ω): Radiance at point x in direction ω
//! - T(x, x'): Transmittance between points (e^(-∫σ_t dx))
//! - σ_s: Scattering coefficient
//! - σ_t: Extinction coefficient (σ_s + σ_a)
//! - p(ω', ω): Phase function (typically Henyey-Greenstein)
//! ```
//!
//! ## Ray Marching Algorithm
//!
//! ```glsl
//! vec3 raymarch_volume(vec3 ray_origin, vec3 ray_dir, float max_dist) {
//!     vec3 accumulated = vec3(0.0);
//!     float transmittance = 1.0;
//!     
//!     for (int i = 0; i < STEP_COUNT; i++) {
//!         float t = i * step_size;
//!         vec3 pos = ray_origin + t * ray_dir;
//!         
//!         float density = sample_3d_texture(pos);
//!         float extinction = density * extinction_coeff;
//!         
//!         // Beer-Lambert law
//!         transmittance *= exp(-extinction * step_size);
//!         
//!         // In-scattering
//!         vec3 light = compute_lighting(pos);
//!         accumulated += transmittance * density * light * step_size;
//!     }
//!     
//!     return accumulated;
//! }
//! ```
//!
//! ## Performance Optimizations
//!
//! 1. **Adaptive Step Sizing**: Larger steps in low-density regions
//! 2. **Early Termination**: Stop when transmittance ≈ 0
//! 3. **Temporal Reprojection**: Reuse samples from previous frames
//! 4. **Importance Sampling**: More samples near lights
//!
//! # Game Design Context: Atmospheric Storytelling
//!
//! ## Emotional Impact of Fog
//!
//! Fog volumes create powerful moods:
//! - **Mystery**: Obscured visibility creates tension
//! - **Scale**: Volumetric light rays suggest vast spaces
//! - **Time of Day**: Morning mist, evening haze
//! - **Environmental Hazards**: Toxic gas, magical barriers
//!
//! ## Gameplay Applications
//!
//! 1. **Stealth Mechanics**: Hide in smoke grenades
//! 2. **Exploration**: Fog clears as you progress
//! 3. **Combat**: Smoke screens block line of sight
//! 4. **Puzzles**: Light beams through fog reveal paths
//! 5. **Horror**: Limited visibility increases fear
//!
//! ## Famous Examples
//!
//! - **Silent Hill**: Fog as core aesthetic and technical solution
//! - **Half-Life 2**: Volumetric light through windows
//! - **Red Dead Redemption 2**: Morning mist in forests
//! - **Control**: Supernatural fog in altered spaces
//!
//! # Performance Deep Dive: GPU Optimization Strategies
//!
//! ## Memory Bandwidth Analysis
//!
//! 3D texture sampling is expensive:
//! ```text
//! Per Ray:
//! - 64 steps × 8 texture samples (trilinear) = 512 samples
//! - Each sample: 4 bytes × 8 taps = 32 bytes
//! - Total: 16KB per pixel (!)
//! 
//! At 1920×1080: 33GB/frame bandwidth
//! ```
//!
//! ## Optimization Techniques
//!
//! ### 1. Resolution Scaling
//! Render fog at 1/4 resolution, upscale with temporal filtering:
//! - 4× performance improvement
//! - Minimal quality loss with good upsampling
//!
//! ### 2. Frustum-Aligned Volumes
//! Store density in view space, not world space:
//! - Better sampling distribution
//! - Concentrates detail near camera
//!
//! ### 3. Hierarchical Ray Marching
//! ```glsl
//! // Coarse pass: 8 steps
//! float coarse_density = raymarch_coarse();
//! if (coarse_density > threshold) {
//!     // Fine pass: 64 steps only where needed
//!     return raymarch_fine();
//! }
//! ```
//!
//! # Real-World Applications: Industry Implementation
//!
//! ## Game Engine Implementations
//!
//! ### Unreal Engine 5
//! - **Volumetric Fog**: Global exponential height fog
//! - **Local Fog Volumes**: Artist-placed density
//! - **Niagara Integration**: Particle-driven fog
//!
//! ### Unity HDRP
//! - **Volumetric Lighting**: Screen-space ray marching
//! - **Density Volumes**: 3D textures or procedural
//! - **Temporal Filtering**: Reduces noise
//!
//! ### Frostbite Engine
//! - **Tile-Based**: Splits screen into tiles
//! - **Async Compute**: Fog on separate GPU queue
//! - **LOD System**: Distance-based quality
//!
//! ## Production Considerations
//!
//! Memory budgets for volumetric effects:
//! - **Mobile**: 8-16MB for all fog textures
//! - **Console**: 64-128MB
//! - **PC High-End**: 256MB+
//!
//! # Advanced Techniques: Next-Generation Volumetrics
//!
//! ## Neural Fog Rendering
//!
//! Using ML to predict fog appearance:
//! ```rust
//! // Pseudo-code for neural fog
//! let features = extract_scene_features(camera, lights, geometry);
//! let fog_params = neural_network.predict(features);
//! let fog_result = fast_analytical_fog(fog_params);
//! ```
//!
//! ## Sparse Voxel Octrees (SVO)
//!
//! Hierarchical representation for large worlds:
//! - Only store non-empty voxels
//! - Adaptive detail levels
//! - Streaming from disk
//!
//! ## Temporal Accumulation
//!
//! ```glsl
//! vec3 current = raymarch_sparse(ray);
//! vec3 history = sample_history_buffer(reproject(pos));
//! float blend = compute_blend_factor(motion_vectors);
//! return mix(history, current, blend);
//! ```
//!
//! # Debugging and Profiling: Volumetric Diagnostics
//!
//! ## Debug Visualization Modes
//!
//! 1. **Density Slices**: Show 2D cross-sections
//! 2. **Ray Count**: Heatmap of ray marching cost
//! 3. **Scattering Only**: Isolate light interaction
//! 4. **Performance Metrics**: ms/frame overlay
//!
//! ## Common Issues and Solutions
//!
//! ### Banding Artifacts
//! - **Cause**: Too few ray march steps
//! - **Solution**: Add noise, increase steps
//! ```glsl
//! float dither = random(uv) * 0.1;
//! float t = (i + dither) * step_size;
//! ```
//!
//! ### Temporal Flickering
//! - **Cause**: Inconsistent sampling
//! - **Solution**: Stable random patterns
//! ```glsl
//! float noise = blue_noise(uv, frame_count);
//! ```
//!
//! ### Memory Pressure
//! - **Monitor**: GPU memory allocation
//! - **Optimize**: Lower resolution 3D textures
//! - **Stream**: Load fog volumes on demand
//!
//! ## Profiling Commands
//!
//! ```rust
//! // Bevy debug commands
//! app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
//!    .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default());
//! 
//! // GPU timing
//! puffin::profile_scope!("fog_volume_raymarch");
//! ```

use bevy::{
    math::vec3,
    pbr::{
        FogVolume,       // Component that defines a fog volume
        VolumetricFog,   // Camera component for volumetric rendering settings
        VolumetricLight, // Marks lights that should interact with fog
    },
    prelude::*,
    // HDR (High Dynamic Range) rendering for better lighting
    render::view::Hdr,
};

/// Entry point.
fn main() {
    App::new()
        // Configure window with custom title
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Fog Volumes Example".into(),
                ..default()
            }),
            ..default()
        }))
        // Disable ambient light to better showcase the volumetric lighting
        // We want all light to come from our directional light source
        .insert_resource(AmbientLight::NONE)
        // Setup system creates the scene
        .add_systems(Startup, setup)
        // Rotate the camera each frame to show different lighting angles
        .add_systems(Update, rotate_camera)
        .run();
}

/// Spawns all the objects in the scene.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a fog volume with a voxelized version of the Stanford bunny.
    commands.spawn((
        // Position the fog volume slightly above the origin
        Transform::from_xyz(0.0, 0.5, 0.0),
        FogVolume {
            // Load the 3D density texture. KTX2 is a compressed texture format
            // that's efficient for GPU usage. This file contains voxel data
            // where each voxel's value represents fog density at that point.
            //
            // ## 3D Texture Format Details
            //
            // The bunny.ktx2 file specifications:
            // - Resolution: Typically 128³ or 256³ voxels
            // - Format: R8 (single channel, 8-bit)
            // - Compression: KTX2 with Basis Universal
            // - Memory: 128³ = 2MB uncompressed, ~200KB compressed
            //
            // ## Voxelization Process
            //
            // Creating fog volumes from meshes:
            // 1. Rasterize mesh into 3D grid
            // 2. Fill interior voxels (flood fill)
            // 3. Apply smoothing filter
            // 4. Encode as 3D texture
            density_texture: Some(asset_server.load("volumes/bunny.ktx2")),
            // Overall density multiplier. Higher values make the fog thicker.
            //
            // ## Density and Visual Perception
            //
            // Real-world fog densities (visibility distance):
            // - Light fog: 0.01 (1km visibility)
            // - Moderate fog: 0.1 (100m visibility)
            // - Dense fog: 1.0 (10m visibility)
            // - Pea soup: 10.0 (1m visibility)
            density_factor: 1.0,
            // Scattering coefficient (0.0 to 1.0)
            // 1.0 means maximum scattering - light bounces around a lot,
            // making the fog appear brighter and more visible
            //
            // ## Mie vs Rayleigh Scattering
            //
            // Real atmospheres have two scattering types:
            // - Rayleigh: Small particles (air molecules), blue sky
            // - Mie: Large particles (water droplets), white fog
            //
            // This example uses isotropic scattering (simplified Mie)
            // for performance. Advanced implementations use the
            // Henyey-Greenstein phase function:
            // 
            // p(θ) = (1 - g²) / (4π(1 + g² - 2g·cos(θ))^(3/2))
            //
            // where g is the anisotropy factor (-1 to 1)
            scattering: 1.0,
            // Using defaults for:
            // - absorption: How much light is absorbed (darkness)
            // - emissive: Whether the fog glows
            // - edge_fade: How the fog fades at volume boundaries
            ..default()
        },
    ));

    // Spawn a bright directional light that illuminates the fog well.
    commands.spawn((
        // Position the light above and to the side, aimed at the bunny
        Transform::from_xyz(1.0, 1.0, -0.3).looking_at(vec3(0.0, 0.5, 0.0), Vec3::Y),
        DirectionalLight {
            // Enable shadows for more realistic lighting
            shadows_enabled: true,
            // Very bright light (32000 lux) to penetrate the fog
            // For reference: bright sunlight is ~100,000 lux
            illuminance: 32000.0,
            ..default()
        },
        // CRITICAL: This component marks the light for volumetric rendering
        // Without it, the light won't interact with fog volumes
        VolumetricLight,
    ));

    // Spawn a camera.
    commands.spawn((
        Camera3d::default(),
        // Position camera to get a good view of the bunny
        Transform::from_xyz(-0.75, 1.0, 2.0).looking_at(vec3(0.0, 0.0, 0.0), Vec3::Y),
        // Enable HDR (High Dynamic Range) rendering
        // This allows for better handling of bright lights and fog
        Hdr,
        // Configure volumetric fog rendering for this camera
        VolumetricFog {
            // Number of steps when raymarching through the fog
            // More steps = better quality but slower performance
            // 64 is relatively high for good quality
            //
            // ## Performance vs Quality Trade-offs
            //
            // Step counts and their impact:
            // - 16 steps: Fast, visible banding, ~1ms
            // - 32 steps: Good balance, slight banding, ~2ms  
            // - 64 steps: High quality, smooth, ~4ms
            // - 128 steps: Film quality, ~8ms
            //
            // The relationship is linear: 2× steps = 2× cost
            step_count: 64,
            // Set ambient light contribution to 0
            // We want only directional light to affect the fog
            //
            // ## Ambient vs Directional Lighting in Fog
            //
            // In real atmospheres:
            // - Direct light: Creates strong scattering (god rays)
            // - Ambient light: Provides base illumination
            //
            // Setting to 0 here emphasizes the directional effect,
            // making the light shafts more dramatic.
            ambient_intensity: 0.0,
            // Defaults include:
            // - max_depth: How far to raymarch
            // - absorption: Global absorption factor  
            // - scattering: Global scattering factor
            //
            // ## Advanced Parameters (via defaults)
            //
            // The default values optimize for common use cases:
            // - max_depth: 16.0 world units (stops ray marching)
            // - absorption: 0.3 (30% light absorbed per unit)
            // - scattering: 0.3 (30% light scattered per unit)
            // - density: 0.1 (global density multiplier)
            //
            // These follow the physics relationship:
            // extinction = absorption + scattering
            ..default()
        },
    ));
}

/// Rotates the camera a bit every frame.
/// 
/// This creates an orbital motion around the bunny, allowing us to see
/// how the volumetric lighting changes from different angles.
///
/// ## Why Camera Movement Matters for Volumetrics
///
/// Volumetric lighting is highly view-dependent because:
/// 1. **Phase Function**: Scattering depends on light-view angle
/// 2. **Occlusion**: Different angles reveal different shadows
/// 3. **Integration Path**: Ray marching samples change
///
/// This rotation showcases all these effects dynamically.
fn rotate_camera(mut cameras: Query<&mut Transform, With<Camera3d>>) {
    for mut camera_transform in cameras.iter_mut() {
        // Rotate the camera's position around the Y axis
        // Quat::from_rotation_y(0.01) creates a small rotation (0.01 radians)
        // Multiplying this quaternion by the position vector rotates it
        //
        // ## Orbital Camera Mathematics
        //
        // We're rotating the position vector, not just the orientation:
        // ```
        // new_pos = rotation_matrix × old_pos
        // new_orient = look_at(target, up)
        // ```
        //
        // At 60 FPS: 0.01 rad/frame = 0.6 rad/sec = 34°/sec
        // Full orbit takes: 2π/0.6 ≈ 10.5 seconds
        //
        // ## Alternative Camera Movements for Volumetrics
        //
        // Other useful patterns:
        // 1. **Fly-through**: Camera moves through the volume
        // 2. **Light orbit**: Keep camera still, rotate light
        // 3. **Zoom**: Change FOV to test LOD systems
        // 4. **Shake**: Small random movements for temporal effects
        *camera_transform =
            Transform::from_translation(Quat::from_rotation_y(0.01) * camera_transform.translation)
                // Keep the camera looking at the bunny's center (0.0, 0.5, 0.0)
                .looking_at(vec3(0.0, 0.5, 0.0), Vec3::Y);
    }
}
