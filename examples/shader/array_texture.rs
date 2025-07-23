//! # GPU Texture Arrays: Efficient Multi-Texture Sampling
//!
//! This example demonstrates **texture arrays** - a GPU optimization technique that allows 
//! multiple related textures to be stored in a single GPU resource and sampled efficiently
//! with dynamic indexing. This is crucial for performance in scenarios like:
//! - Tile-based games (different ground types, wall textures)
//! - Animation frames stored as texture layers
//! - Material variations (different wood types, metal finishes)
//! - Terrain texturing with multiple detail textures
//!
//! ## Key GPU Programming Concepts:
//! - **Texture Arrays**: 3D texture resources where each "layer" is a 2D texture
//! - **Dynamic Indexing**: Using runtime values to select which texture layer to sample
//! - **Shader Uniforms**: How data flows from CPU application to GPU shaders
//! - **Memory Efficiency**: Reducing GPU memory bandwidth by batching related textures
//!
//! ## How It Works:
//! 1. Load a stacked 2D texture (4 textures arranged in a 2x2 grid)
//! 2. Reinterpret it as a texture array with 4 layers
//! 3. In the shader, use the cube's world position to select which layer to sample
//! 4. Sample from the selected layer using `textureSample()` with layer index

// Core Bevy functionality - ECS, math, transforms, etc.
use bevy::{
    prelude::*,
    // TypePath: Required for asset types to be identified in Bevy's type system
    // This enables the asset to be loaded, stored, and referenced properly
    reflect::TypePath,
    render::render_resource::{
        // AsBindGroup: Derives how this material's data gets bound to GPU shader resources
        // Think of this as the "bridge" between Rust structs and GPU shader uniforms
        AsBindGroup, 
        // ShaderRef: A reference to a shader file that can be an asset path or inline code
        ShaderRef
    },
};

// Path to our custom fragment shader that handles texture array sampling
// WGSL (WebGPU Shading Language) is the shader language used by Bevy
const SHADER_ASSET_PATH: &str = "shaders/array_texture.wgsl";

fn main() {
    App::new()
        .add_plugins((
            // Standard Bevy plugins (window, renderer, input, etc.)
            DefaultPlugins,
            // Register our custom material type with Bevy's material system
            // This creates the render pipeline, manages GPU resources, and handles
            // the binding of our material data to shader uniforms
            MaterialPlugin::<ArrayTextureMaterial>::default(),
        ))
        // Setup system runs once at startup to initialize the scene
        .add_systems(Startup, setup)
        // Update system runs every frame to check if texture is loaded
        // This demonstrates async asset loading patterns in Bevy
        .add_systems(Update, create_array_texture)
        .run();
}

// Resource: A global singleton that can be accessed by any system
// Used here to track the asynchronous loading state of our texture
#[derive(Resource)]
struct LoadingTexture {
    // Flag to prevent duplicate processing once the texture is loaded
    is_loaded: bool,
    // Handle: A lightweight reference to an asset that may not be loaded yet
    // Bevy's asset system works asynchronously - files load in the background
    handle: Handle<Image>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Begin asynchronous loading of our texture array image
    // The image file contains 4 textures arranged in a 2x2 grid that we'll later
    // reinterpret as a texture array for efficient GPU sampling
    commands.insert_resource(LoadingTexture {
        is_loaded: false,
        // asset_server.load() returns immediately with a handle, actual loading happens in background
        handle: asset_server.load("textures/array_texture.png"),
    });

    // Directional light simulates sunlight - parallel rays from infinite distance
    // Essential for 3D scenes to see surface details and depth
    commands.spawn((
        DirectionalLight::default(),
        // Position light above and to the side, looking toward the origin
        Transform::from_xyz(3.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 3D camera positioned to view our array of cubes
    commands.spawn((
        Camera3d::default(),
        // Position camera at (5,5,5) looking toward our cube array at x=1.5
        // Vec3::Y is the up direction (positive Y axis)
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::new(1.5, 0.0, 0.0), Vec3::Y),
    ));
}

fn create_array_texture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_texture: ResMut<LoadingTexture>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ArrayTextureMaterial>>,
) {
    // This system runs every frame until the texture is loaded and processed
    // Early return if already processed OR if texture isn't loaded yet
    if loading_texture.is_loaded
        || !asset_server
            .load_state(loading_texture.handle.id())
            .is_loaded()
    {
        return;
    }
    
    // Mark as processed to prevent running again
    loading_texture.is_loaded = true;
    
    // Get mutable access to the loaded image data
    let image = images.get_mut(&loading_texture.handle).unwrap();

    // CRITICAL GPU OPTIMIZATION: Convert stacked 2D texture to texture array
    // A "stacked" texture has multiple images arranged in a grid (2x2 in this case)
    // reinterpret_stacked_2d_as_array() tells the GPU to treat this as 4 separate layers
    // instead of one large 2D texture. This enables efficient dynamic layer selection.
    let array_layers = 4;
    image.reinterpret_stacked_2d_as_array(array_layers);

    // Create a row of cubes, each sampling from a different texture array layer
    let mesh_handle = meshes.add(Cuboid::default());
    let material_handle = materials.add(ArrayTextureMaterial {
        array_texture: loading_texture.handle.clone(),
    });
    
    // Spawn 11 cubes (-5 to +5) positioned along the X axis
    // Each cube's world position will determine which texture layer it samples
    for x in -5..=5 {
        commands.spawn((
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material_handle.clone()),
            // Offset by 0.5 to center the array around the origin
            Transform::from_xyz(x as f32 + 0.5, 0.0, 0.0),
        ));
    }
}

// This struct defines our custom material type for texture array sampling
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct ArrayTextureMaterial {
    // AsBindGroup attribute annotations tell Bevy how to bind this data to GPU resources:
    
    // #[texture(0, dimension = "2d_array")]: 
    // - Binds to shader binding 0 in group 2 (materials use group 2)
    // - dimension = "2d_array" tells GPU this is a texture array, not a regular 2D texture
    // - This corresponds to "texture_2d_array<f32>" in the WGSL shader
    #[texture(0, dimension = "2d_array")]
    
    // #[sampler(1)]: 
    // - Binds to shader binding 1 in group 2
    // - Samplers control how textures are filtered and wrapped
    // - This corresponds to "sampler" in the WGSL shader
    #[sampler(1)]
    array_texture: Handle<Image>,
}

impl Material for ArrayTextureMaterial {
    /// Override the default fragment shader with our custom texture array shader
    /// 
    /// ## Advanced Texture Array Concepts:
    /// The Material trait handles vertex processing automatically, but we need
    /// custom fragment logic to sample from texture arrays with dynamic indexing.
    /// This demonstrates several advanced GPU programming techniques:
    /// 
    /// ### Dynamic Indexing and GPU Architecture:
    /// - **Texture Units**: Modern GPUs have dedicated texture sampling units (TMUs)
    /// - **Cache Hierarchy**: L1 texture cache for recently accessed texels
    /// - **Memory Coalescing**: Efficient access patterns when neighboring fragments sample similar locations
    /// - **SIMD Execution**: Multiple fragments processed simultaneously can benefit from coherent texture access
    /// 
    /// ### Performance Considerations:
    /// - **Dynamic Branching**: Layer selection based on runtime values can cause divergence
    /// - **Texture Bandwidth**: Multiple layer access can saturate memory bandwidth
    /// - **Cache Efficiency**: Sequential layer access patterns optimize cache usage
    /// - **Filtering**: Hardware trilinear filtering works across texture array layers
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

// ============================================================================
// ADVANCED GRAPHICS THEORY AND GPU ARCHITECTURE CONCEPTS
// ============================================================================

// ## Texture Array Deep Dive: Memory Layout and Access Patterns
//
// ### GPU Memory Hierarchy for Textures:
// Modern GPUs implement a sophisticated memory hierarchy optimized for texture access:
//
// #### Memory Levels (from fastest to slowest):
// - **Texture Cache (L1)**: 16-64KB per SM, ~1 cycle latency for hits
// - **Texture Cache (L2)**: 512KB-6MB shared, ~30-50 cycle latency  
// - **VRAM**: 8-24GB, ~400-800 cycle latency, high bandwidth (500-1000 GB/s)
// - **System RAM**: Accessed via PCIe, ~1000+ cycle latency (fallback only)
//
// #### Texture Array Memory Layout:
// Texture arrays store multiple 2D textures in a single contiguous 3D block:
// ```
// Memory Layout: [Layer0][Layer1][Layer2][Layer3]...
// Each Layer:    [Mip0][Mip1][Mip2]... (if mipmapping enabled)
// Each Mip:      [Row0][Row1][Row2]... (linear or swizzled)
// ```
//
// ### Advanced Sampling and Filtering Theory:
//
// #### Trilinear Filtering Across Array Layers:
// Modern hardware supports filtering between texture array layers, enabling:
// - **Temporal Interpolation**: Smooth animation frame blending
// - **Level-of-Detail**: Distance-based layer selection with smooth transitions
// - **Procedural Blending**: Combining multiple material layers
//
// #### Anisotropic Filtering:
// Texture arrays support anisotropic filtering within each layer:
// - **Sampling Pattern**: Elliptical footprint based on screen-space derivatives
// - **Performance Impact**: Higher anisotropy levels (16x) can reduce performance
// - **Quality Benefit**: Dramatically improves texture clarity at glancing angles
//
// ## Performance Optimization Strategies:
//
// ### Memory Bandwidth Optimization:
// #### Texture Compression:
// - **BC7 (DXT5)**: Best quality for diffuse textures, 8:1 compression ratio
// - **BC4 (ATI1)**: Optimal for normal maps and single-channel data
// - **ASTC**: Advanced format with variable bit rates (4x4 to 12x12 blocks)
// - **ETC2**: Mobile-optimized compression with good quality/size balance
//
// #### Cache-Friendly Access Patterns:
// ```rust
// // GOOD: Sequential layer access
// for layer in 0..num_layers {
//     let color = sample_texture_array(uv, layer);
//     // Process layer...
// }
//
// // BAD: Random layer access (causes cache thrashing)
// let random_layers = [3, 0, 7, 1, 5, 2, 6, 4];
// for layer in random_layers {
//     let color = sample_texture_array(uv, layer);
// }
// ```
//
// ### Advanced Batching Techniques:
//
// #### GPU-Driven Rendering with Texture Arrays:
// - **Indirect Drawing**: GPU determines which objects to draw based on texture availability
// - **Bindless Textures**: Access texture arrays without CPU bind calls
// - **Descriptor Indexing**: Dynamic texture selection in shaders
//
// #### Material System Integration:
// - **Material Atlasing**: Combine related materials into texture array layers
// - **Streaming**: Dynamically load/unload texture array layers based on distance
// - **LOD Systems**: Use different array layers for different detail levels
//
// ## Real-World Applications and Case Studies:
//
// ### Terrain Rendering:
// Modern open-world games use texture arrays for terrain systems:
// - **Layer 0**: Grass texture for low-elevation areas
// - **Layer 1**: Rock texture for mountain regions  
// - **Layer 2**: Sand texture for desert biomes
// - **Layer 3**: Snow texture for high-altitude areas
// - **Blending**: Use heightmaps and biome masks to blend between layers
//
// ### Animation Systems:
// #### Sprite Animation with Texture Arrays:
// ```glsl
// // Fragment shader pseudocode for sprite animation
// float time_fraction = fract(time * animation_speed);
// int frame_index = int(time_fraction * num_frames);
// vec4 color = texture(sprite_array, vec3(uv, frame_index));
// ```
//
// #### Benefits over Traditional Sprite Sheets:
// - **No UV Calculation**: Direct layer indexing instead of complex UV math
// - **Hardware Filtering**: Automatic bilinear filtering within frames
// - **Memory Efficiency**: Better cache locality compared to large sprite sheets
//
// ### Particle Systems:
// Modern particle systems use texture arrays for variation:
// - **Different Particle Types**: Smoke, fire, sparks, debris in separate layers
// - **Random Variation**: Pseudo-random layer selection for visual diversity
// - **Performance**: Single draw call for entire particle system
//
// ## Advanced Shader Techniques:
//
// ### Procedural Layer Selection:
// ```glsl
// // Advanced layer selection based on world position and noise
// float noise_value = noise(world_position.xy * 0.01);
// float height_factor = world_position.y * 0.1;
// float combined = noise_value + height_factor;
// int layer = int(combined * num_layers) % num_layers;
// vec4 color = texture(texture_array, vec3(uv, layer));
// ```
//
// ### Multi-Layer Blending:
// ```glsl
// // Blend multiple layers based on material masks
// vec4 layer0 = texture(texture_array, vec3(uv, 0));
// vec4 layer1 = texture(texture_array, vec3(uv, 1));
// vec4 layer2 = texture(texture_array, vec3(uv, 2));
//
// vec3 blend_weights = texture(blend_mask, uv).rgb;
// vec4 final_color = layer0 * blend_weights.r + 
//                    layer1 * blend_weights.g + 
//                    layer2 * blend_weights.b;
// ```
//
// ## Debugging and Profiling:
//
// ### Texture Array Debugging:
// #### Visual Debugging Techniques:
// - **Layer Visualization**: Render each layer separately to verify content
// - **Index Debugging**: Color-code fragments by their selected layer index
// - **UV Debugging**: Visualize texture coordinates to ensure correct mapping
//
// #### Common Issues and Solutions:
// - **Layer Bleeding**: Ensure proper texture padding between array layers
// - **Compression Artifacts**: Test different compression formats for quality
// - **Memory Usage**: Monitor VRAM consumption with multiple large arrays
//
// ### Performance Profiling:
// #### Key Metrics to Monitor:
// - **Texture Cache Hit Rate**: Should be >90% for optimal performance
// - **Memory Bandwidth Utilization**: Modern GPUs support 500-1000 GB/s
// - **TMU (Texture Mapping Unit) Utilization**: Monitor for texture bottlenecks
// - **Fragment Shader ALU**: Balance texture sampling with arithmetic operations
//
// #### Tools for Analysis:
// - **NVIDIA Nsight Graphics**: Comprehensive GPU profiling and debugging
// - **RenderDoc**: Cross-platform graphics debugging with texture inspection
// - **AMD GPU Profiler**: AMD-specific performance analysis tools
// - **Intel GPA**: Intel GPU performance analysis for integrated graphics
//
// ## Memory Management and Streaming:
//
// ### Texture Array Streaming:
// For large-scale applications with many texture arrays:
//
// #### Distance-Based Streaming:
// ```rust
// struct TextureArrayLOD {
//     near_distance: f32,   // Use high-res array
//     mid_distance: f32,    // Use medium-res array  
//     far_distance: f32,    // Use low-res array or disable
// }
// ```
//
// #### Benefits:
// - **Memory Efficiency**: Only load required detail levels
// - **Performance**: Reduce memory bandwidth for distant objects
// - **Scalability**: Support larger worlds with limited VRAM
//
// ### Mobile Optimization:
// #### Considerations for Mobile GPUs:
// - **Memory Constraints**: Limit texture array size to avoid OOM
// - **Bandwidth Limits**: Mobile GPUs have lower memory bandwidth (~50-100 GB/s)
// - **Power Efficiency**: Reduce texture sampling frequency to save battery
// - **Format Support**: Verify texture array support across mobile GPU vendors
//
// This example demonstrates fundamental concepts that scale to the most demanding
// real-time graphics applications, from mobile games to AAA PC titles. Understanding
// texture arrays is essential for modern GPU programming and performance optimization.
