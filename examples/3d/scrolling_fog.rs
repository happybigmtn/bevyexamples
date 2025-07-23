//! Showcases a `FogVolume`'s density texture being scrolled over time to create
//! the effect of fog moving in the wind.
//!
//! The density texture is a repeating 3d noise texture and the `density_texture_offset`
//! is moved every frame to achieve this.
//!
//! The example also utilizes the jitter option of `VolumetricFog` in tandem
//! with temporal anti-aliasing to improve the visual quality of the effect.
//!
//! The camera is looking at a pillar with the sun peaking behind it. The light
//! interactions change based on the density of the fog.
//!
//! # Animated Volumetric Fog: Bringing Atmosphere to Life
//!
//! Static fog is nice, but real fog moves! This example demonstrates how to
//! create dynamic, wind-blown fog using 3D noise textures. It's like having
//! a fog machine that follows realistic air currents.
//!
//! ## Key Techniques:
//!
//! 1. **3D Density Textures**: Instead of uniform fog, we use noise patterns
//! 2. **Texture Scrolling**: Moving the texture offset creates motion
//! 3. **Temporal Jittering**: Combined with TAA for smooth results
//! 4. **Light Scattering**: Fog interacts realistically with lights
//!
//! ## Real-World Applications:
//!
//! - Horror games: Creeping fog in graveyards
//! - Flight sims: Moving cloud layers
//! - Underwater scenes: Flowing murky water
//! - Atmospheric effects: Smoke, steam, mist

// Rust: External crate imports with specific items
use bevy::{
    // Rust: Deep module path for TAA
    anti_aliasing::taa::TemporalAntiAliasing,
    // Rust: Bloom post-processing effect
    core_pipeline::bloom::Bloom,
    // Rust: Multiple imports from image module
    image::{
        ImageAddressMode,      // How texture repeats at edges
        ImageFilterMode,       // Sampling interpolation
        ImageLoaderSettings,   // Asset loading configuration
        ImageSampler,         // Enum for sampler types
        ImageSamplerDescriptor, // Detailed sampler settings
    },
    // Rust: PBR (Physically Based Rendering) components
    pbr::{
        DirectionalLightShadowMap, // Shadow map configuration
        FogVolume,                 // 3D fog volume component
        VolumetricFog,            // Camera fog settings
        VolumetricLight          // Light scattering marker
    },
    // Rust: Common Bevy types
    prelude::*,
};

// Rust: Doc comment
/// Initializes the example.
// Rust: Entry point function
fn main() {
    // Rust: Builder pattern with method chaining
    App::new()
        // Rust: Complex plugin configuration
        .add_plugins(
            // Rust: Method call on struct
            DefaultPlugins.set(
                // Rust: Struct literal with nested Some
                WindowPlugin {
                    // Rust: Option<T> with Some variant
                    primary_window: Some(Window {
                        // Rust: String conversion with .into()
                        // &str -> String via Into trait
                        title: "Bevy Scrolling Fog".into(),
                        // Rust: Struct update syntax
                        ..default()
                    }),
                    ..default()
                }
            )
        )
        // Rust: Resource insertion
        // Resources are globally accessible data
        .insert_resource(DirectionalLightShadowMap { 
            size: 4096  // Shadow map resolution in pixels
        })
        // Rust: System scheduling
        .add_systems(Startup, setup)
        .add_systems(Update, scroll_fog)
        // Rust: Consumes App and runs
        .run();
}

/// Spawns all entities into the scene.
// Rust: System function with resource parameters
fn setup(
    // Rust: Mutable access for spawning entities
    mut commands: Commands,
    // Rust: Mutable asset storage access
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Rust: Immutable AssetServer access
    // Note: renamed from asset_server to assets
    assets: Res<AssetServer>,
) {
    // Spawn camera with temporal anti-aliasing and a VolumetricFog configuration.
    // Rust: Tuple of components for entity
    commands.spawn((
        // Rust: Default camera settings
        Camera3d::default(),
        // Rust: Transform builder methods
        Transform::from_xyz(0.0, 2.0, 0.0)  // Camera position
            // Rust: Method chaining with Vec3 construction
            .looking_at(Vec3::new(-5.0, 3.5, -6.0), Vec3::Y),
        // Rust: Enum variant for MSAA settings
        Msaa::Off,  // Required for TAA to work properly
        // Rust: Default TAA settings
        TemporalAntiAliasing::default(),
        // Rust: Bloom for glowing effects
        Bloom::default(),
        // Rust: Struct literal with specific fields
        VolumetricFog {
            ambient_intensity: 0.0,  // No ambient fog light
            // Rust: f32 literal for jitter amount
            jitter: 0.5,  // Randomizes sampling for quality
            // Rust: Default other fields
            ..default()
        },
    ));

    // Spawn a directional light shining at the camera with the VolumetricLight component.
    commands.spawn((
        // Rust: Struct literal with partial initialization
        DirectionalLight {
            // Rust: bool literal
            shadows_enabled: true,  // Shadows interact with fog
            ..default()
        },
        // Rust: Transform positioning and orientation
        Transform::from_xyz(-5.0, 5.0, -7.0)  // Sun position
            // Rust: Looking at world origin
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        // Rust: Marker component (zero-size type)
        VolumetricLight,  // Enables light scattering in fog
    ));

    // Spawn ground mesh.
    commands.spawn((
        // Rust: Nested function calls
        // Cuboid::new creates shape, meshes.add stores it
        Mesh3d(meshes.add(Cuboid::new(64.0, 1.0, 64.0))),  // Large flat ground
        // Rust: Creating and storing material
        MeshMaterial3d(materials.add(StandardMaterial {
            // Rust: Color constant
            base_color: Color::BLACK,
            // Rust: f32 for roughness (0.0 = mirror, 1.0 = diffuse)
            perceptual_roughness: 1.0,
            ..default()
        })),
        // Rust: Position just below origin
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    // Spawn pillar standing between the camera and the sun.
    commands.spawn((
        // Rust: Tall thin cuboid for pillar
        Mesh3d(meshes.add(Cuboid::new(2.0, 9.0, 2.0))),
        // Rust: Implicit conversion
        // Color implements Into<StandardMaterial>
        MeshMaterial3d(materials.add(Color::BLACK)),
        // Rust: Positioned to cast shadows through fog
        Transform::from_xyz(-10.0, 4.5, -11.0),
    ));

    // Load a repeating 3d noise texture. Make sure to set ImageAddressMode to Repeat
    // so that the texture wraps around as the density texture offset is moved along.
    // Also set ImageFilterMode to Linear so that the fog isn't pixelated.
    // Rust: Variable binding with complex initialization
    let noise_texture = assets.load_with_settings(
        "volumes/fog_noise.ktx2",  // 3D texture file
        // Rust: Closure with type inference
        // &mut _ means compiler infers the type
        |settings: &mut _| {
            // Rust: Dereferencing and assignment
            // *settings replaces entire struct
            *settings = ImageLoaderSettings {
                // Rust: Enum variant with nested struct
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // Rust: Texture wrapping modes
                    // Repeat makes texture tile infinitely
                    address_mode_u: ImageAddressMode::Repeat,  // X axis
                    address_mode_v: ImageAddressMode::Repeat,  // Y axis  
                    address_mode_w: ImageAddressMode::Repeat,  // Z axis
                    // Rust: Filtering for smooth interpolation
                    mag_filter: ImageFilterMode::Linear,     // Magnification
                    min_filter: ImageFilterMode::Linear,     // Minification
                    mipmap_filter: ImageFilterMode::Linear,  // Between mip levels
                    ..default()
                }),
                ..default()
            }
        }
    );

    // Spawn a FogVolume and use the repeating noise texture as its density texture.
    commands.spawn((
        // Rust: Transform with position and scale
        Transform::from_xyz(0.0, 32.0, 0.0)  // Centered, elevated
            // Rust: Vec3::splat creates uniform scale
            .with_scale(Vec3::splat(64.0)),  // 64x64x64 volume
        // Rust: FogVolume configuration
        FogVolume {
            // Rust: Option<Handle<Image>> with Some
            density_texture: Some(noise_texture),
            // Rust: f32 multiplier for density
            density_factor: 0.05,  // Low density for subtle effect
            ..default()
        },
    ));
}

/// Moves fog density texture offset every frame.
// Rust: Update system that runs every frame
fn scroll_fog(
    // Rust: Time resource for frame timing
    time: Res<Time>, 
    // Rust: Query for mutable FogVolume components
    mut query: Query<&mut FogVolume>
) {
    // Rust: Iterate over all fog volumes
    for mut fog_volume in query.iter_mut() {
        // Rust: Compound assignment operator +=
        // Moves texture offset over time
        fog_volume.density_texture_offset += 
            // Rust: Vector math
            Vec3::new(0.0, 0.0, 0.04)  // Movement direction (Z axis)
            * time.delta_secs();       // Scale by frame time
        // This creates smooth, framerate-independent motion
    }
}

// 🎯 Advanced Rust Concepts in This Example:
//
// 1. **Closure Type Inference**:
//    - `|settings: &mut _|` - underscore lets compiler infer type
//    - Useful when type is complex or obvious from context
//
// 2. **Dereferencing Patterns**:
//    - `*settings = ...` - Replace entire struct through reference
//    - Different from `settings.field = ...` which modifies fields
//
// 3. **Enum Variants as Types**:
//    - `ImageSampler::Descriptor(...)` - Enum with data
//    - Each variant can hold different types
//
// 4. **Compound Assignment**:
//    - `+=` is syntactic sugar for `x = x + y`
//    - Works with any type implementing AddAssign trait
//
// 5. **Method Chaining Returns**:
//    - `.with_scale()` returns modified Transform
//    - Allows `from_xyz().with_scale().with_rotation()`
//
// 6. **Resource vs Component**:
//    - Resources: Global data (DirectionalLightShadowMap)
//    - Components: Per-entity data (FogVolume)
//
// 7. **Marker Components**:
//    - `VolumetricLight` has no data
//    - Used purely for system filtering
//    - Zero runtime cost
