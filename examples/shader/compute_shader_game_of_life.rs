//! # GPU Compute Shaders: Conway's Game of Life GPGPU Implementation
//!
//! This example demonstrates **compute shaders** - specialized GPU programs that perform
//! general-purpose computation (GPGPU) rather than traditional graphics rendering.
//! Conway's Game of Life serves as an ideal demonstration of parallel cellular automata
//! computation, showcasing the GPU's massive parallel processing capabilities.
//!
//! ## Core Compute Shader Concepts:
//! - **General Purpose GPU Computing (GPGPU)**: Using graphics hardware for non-graphics tasks
//! - **Parallel Cellular Automata**: Massively parallel simulation of simple rules creating complex behavior
//! - **Memory Synchronization**: Coordinating read/write operations across thousands of threads
//! - **Workgroup Architecture**: Organizing computation into cache-coherent thread groups
//! - **Double Buffering**: Ping-pong technique for temporal state updates
//!
//! ## GPU Architecture Benefits for Compute:
//! - **Massive Parallelism**: 2000+ cores vs 8-16 CPU cores
//! - **High Memory Bandwidth**: 500-1000 GB/s vs 50-100 GB/s for CPU
//! - **SIMD Execution**: Single instruction operates on multiple data simultaneously
//! - **Specialized ALUs**: Optimized for floating-point and integer operations
//! - **Local Memory**: Fast shared memory for workgroup coordination
//!
//! ## Conway's Game of Life Rules:
//! Cellular automaton with simple rules creating emergent complexity:
//! 1. **Underpopulation**: Live cell with <2 neighbors dies
//! 2. **Survival**: Live cell with 2-3 neighbors survives
//! 3. **Overpopulation**: Live cell with >3 neighbors dies
//! 4. **Reproduction**: Dead cell with exactly 3 neighbors becomes alive
//!
//! ## Real-World Applications:
//! - **Fluid Simulation**: Water, smoke, and gas dynamics
//! - **Physics Simulation**: Particle systems and collision detection
//! - **Machine Learning**: Neural network training and inference
//! - **Scientific Computing**: Weather prediction and molecular dynamics
//! - **Image Processing**: Convolution, filtering, and computer vision

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::{RenderContext, RenderDevice},
        texture::GpuImage,
        Render, RenderApp, RenderSystems,
    },
};
use std::borrow::Cow;

/// Path to our compute shader written in WGSL (WebGPU Shading Language)
/// Compute shaders have different entry points and capabilities compared to graphics shaders
const SHADER_ASSET_PATH: &str = "shaders/game_of_life.wgsl";

/// **Performance Scaling Factor**: Reduces simulation resolution for better performance
/// Game of Life computation complexity: O(width * height * neighbors_per_cell)
/// Each cell checks 8 neighbors, so total operations = 320*180*8 = 460,800 per frame
const DISPLAY_FACTOR: u32 = 4;

/// **Simulation Grid Dimensions**: 320x180 cells for optimal GPU utilization
/// Chosen to balance visual detail with computational performance
/// Grid size affects both memory usage (2 * width * height * sizeof(f32)) and parallelism
const SIZE: (u32, u32) = (1280 / DISPLAY_FACTOR, 720 / DISPLAY_FACTOR);

/// **GPU Workgroup Size**: 8x8 threads per workgroup (64 threads total)
/// 
/// ## Workgroup Theory and Performance:
/// - **Hardware Alignment**: Most GPUs execute threads in groups of 32 (warps) or 64 (wavefronts)
/// - **Memory Coalescing**: 8x8 ensures neighboring threads access neighboring memory
/// - **Cache Efficiency**: Workgroup fits in GPU's local memory/cache for optimal performance
/// - **Occupancy**: 64 threads per workgroup allows multiple workgroups per SM (Streaming Multiprocessor)
/// 
/// ## Memory Access Pattern:
/// Each thread processes one cell, accessing its 8 neighbors:
/// ```
/// [N][N][N]
/// [N][C][N]  where C = current cell, N = neighbor
/// [N][N][N]
/// ```
/// The 8x8 workgroup ensures most neighbor accesses hit the same cache lines.
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
        // Black background emphasizes the bright cellular patterns
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // Scale up simulation resolution for display
                        // Simulation: 320x180, Display: 1280x720 (4x upscaling)
                        resolution: (
                            (SIZE.0 * DISPLAY_FACTOR) as f32,
                            (SIZE.1 * DISPLAY_FACTOR) as f32,
                        )
                            .into(),
                        // Enable for maximum performance testing (uncapped FPS)
                        // Useful for measuring compute shader performance limits
                        // present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                // Nearest neighbor sampling preserves crisp cellular boundaries
                // Prevents blurring artifacts when upscaling the simulation texture
                .set(ImagePlugin::default_nearest()),
            // Custom plugin encapsulating all compute shader logic
            GameOfLifeComputePlugin,
        ))
        .add_systems(Startup, setup)
        // Double-buffering system: alternates display between two textures
        // Required because compute shaders need separate read/write targets
        .add_systems(Update, switch_textures)
        .run();
}

/// Sets up the double-buffered texture system required for Game of Life simulation.
/// 
/// ## Double Buffering Theory:
/// Cellular automata require reading the current state while writing the next state.
/// Since compute shaders can't read from and write to the same texture simultaneously,
/// we use two textures and alternate between them each frame.
/// 
/// ## Memory Layout and Format:
/// - **TextureFormat::R32Float**: Single-channel 32-bit float per pixel
/// - **Memory Usage**: 320 * 180 * 4 bytes * 2 textures = 460KB total
/// - **Storage Binding**: Allows compute shader read/write access
/// - **Texture Binding**: Allows fragment shader sampling for display
fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // Create a texture initialized with all cells dead (0.0 = dead, 1.0 = alive)
    // The compute shader will initialize with a random pattern on first frame
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        // Fill with zeros (all cells start dead)
        &[0, 0, 0, 255],
        // R32Float: Single precision float for cellular state
        // More precision than needed for binary states, but enables future extensions
        // (like gradual death/birth, cellular strength, multi-species, etc.)
        TextureFormat::R32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    
    // CRITICAL: Configure texture for compute shader access
    // Multiple usage flags enable different GPU operations on the same texture
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST |        // CPU can write initial data
        TextureUsages::STORAGE_BINDING | // Compute shader read/write access  
        TextureUsages::TEXTURE_BINDING;  // Fragment shader sampling access
    
    // Create two identical textures for ping-pong buffering
    // Texture A: Read source for frame N, Write target for frame N+1
    // Texture B: Write target for frame N, Read source for frame N+1
    let image0 = images.add(image.clone());
    let image1 = images.add(image);

    commands.spawn((
        Sprite {
            image: image0.clone(),
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        Transform::from_scale(Vec3::splat(DISPLAY_FACTOR as f32)),
    ));
    commands.spawn(Camera2d);

    commands.insert_resource(GameOfLifeImages {
        texture_a: image0,
        texture_b: image1,
    });
}

// Switch texture to display every frame to show the one that was written to most recently.
fn switch_textures(images: Res<GameOfLifeImages>, mut sprite: Single<&mut Sprite>) {
    if sprite.image == images.texture_a {
        sprite.image = images.texture_b.clone_weak();
    } else {
        sprite.image = images.texture_a.clone_weak();
    }
}

struct GameOfLifeComputePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct GameOfLifeLabel;

impl Plugin for GameOfLifeComputePlugin {
    fn build(&self, app: &mut App) {
        // Extract the game of life image resource from the main world into the render world
        // for operation on by the compute shader and display on the sprite.
        app.add_plugins(ExtractResourcePlugin::<GameOfLifeImages>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
        );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(GameOfLifeLabel, GameOfLifeNode::default());
        render_graph.add_node_edge(GameOfLifeLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<GameOfLifePipeline>();
    }
}

#[derive(Resource, Clone, ExtractResource)]
struct GameOfLifeImages {
    texture_a: Handle<Image>,
    texture_b: Handle<Image>,
}

#[derive(Resource)]
struct GameOfLifeImageBindGroups([BindGroup; 2]);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<GameOfLifePipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    game_of_life_images: Res<GameOfLifeImages>,
    render_device: Res<RenderDevice>,
) {
    let view_a = gpu_images.get(&game_of_life_images.texture_a).unwrap();
    let view_b = gpu_images.get(&game_of_life_images.texture_b).unwrap();
    let bind_group_0 = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((&view_a.texture_view, &view_b.texture_view)),
    );
    let bind_group_1 = render_device.create_bind_group(
        None,
        &pipeline.texture_bind_group_layout,
        &BindGroupEntries::sequential((&view_b.texture_view, &view_a.texture_view)),
    );
    commands.insert_resource(GameOfLifeImageBindGroups([bind_group_0, bind_group_1]));
}

#[derive(Resource)]
struct GameOfLifePipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for GameOfLifePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            "GameOfLifeImages",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::ReadOnly),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );
        let shader = world.load_asset(SHADER_ASSET_PATH);
        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
            zero_initialize_workgroup_memory: false,
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
            zero_initialize_workgroup_memory: false,
        });

        GameOfLifePipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

enum GameOfLifeState {
    Loading,
    Init,
    Update(usize),
}

struct GameOfLifeNode {
    state: GameOfLifeState,
}

impl Default for GameOfLifeNode {
    fn default() -> Self {
        Self {
            state: GameOfLifeState::Loading,
        }
    }
}

impl render_graph::Node for GameOfLifeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<GameOfLifePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            GameOfLifeState::Loading => {
                match pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline) {
                    CachedPipelineState::Ok(_) => {
                        self.state = GameOfLifeState::Init;
                    }
                    // If the shader hasn't loaded yet, just wait.
                    CachedPipelineState::Err(PipelineCacheError::ShaderNotLoaded(_)) => {}
                    CachedPipelineState::Err(err) => {
                        panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}")
                    }
                    _ => {}
                }
            }
            GameOfLifeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = GameOfLifeState::Update(1);
                }
            }
            GameOfLifeState::Update(0) => {
                self.state = GameOfLifeState::Update(1);
            }
            GameOfLifeState::Update(1) => {
                self.state = GameOfLifeState::Update(0);
            }
            GameOfLifeState::Update(_) => unreachable!(),
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let bind_groups = &world.resource::<GameOfLifeImageBindGroups>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<GameOfLifePipeline>();

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        // Execute the appropriate compute shader based on simulation state
        match self.state {
            GameOfLifeState::Loading => {} // Waiting for shader compilation
            
            GameOfLifeState::Init => {
                // **INITIALIZATION PHASE**: Generate random starting pattern
                // This runs only once to seed the simulation with interesting patterns
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_bind_group(0, &bind_groups[0], &[]);
                pass.set_pipeline(init_pipeline);
                
                // **CRITICAL COMPUTE DISPATCH**: Launch parallel computation
                // dispatch_workgroups(x, y, z) creates a 3D grid of workgroups
                // Total threads = (SIZE.0/8) * (SIZE.1/8) * 64 threads_per_workgroup
                // = 40 * 22.5 * 64 = 57,600 parallel threads
                // Each thread initializes one cell with pseudo-random state
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                
                // ## GPU Execution Model:
                // 1. GPU schedules 900 workgroups (40x22.5, rounded up)
                // 2. Each workgroup runs 64 threads in parallel (8x8)
                // 3. Threads execute the init compute shader in lockstep
                // 4. Each thread writes to one pixel in the output texture
                // 5. GPU ensures all writes complete before continuing
            }
            
            GameOfLifeState::Update(index) => {
                // **SIMULATION PHASE**: Apply Conway's Game of Life rules
                // This runs every frame after initialization
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                
                // Bind appropriate texture pair for ping-pong buffering
                // index alternates between 0 and 1 each frame
                // bind_groups[0]: texture_a (read) -> texture_b (write)  
                // bind_groups[1]: texture_b (read) -> texture_a (write)
                pass.set_bind_group(0, &bind_groups[index], &[]);
                pass.set_pipeline(update_pipeline);
                
                // **PARALLEL GAME OF LIFE COMPUTATION**
                // Each thread processes one cell and its 8 neighbors:
                // 1. Read current cell state and 8 neighbor states (9 reads total)
                // 2. Count living neighbors
                // 3. Apply Conway's rules (survive/die/birth)
                // 4. Write new state to output texture
                // 
                // Total memory operations per frame:
                // - Reads: 320*180*9 = 518,400 texture samples
                // - Writes: 320*180*1 = 57,600 texture writes
                // - Memory bandwidth: ~2.3MB/frame at 60fps = 138MB/s
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
                
                // ## Performance Analysis:
                // At 60fps, this system processes:
                // - 57,600 cells * 60fps = 3.456 million cellular updates/second
                // - Each update involves 8 neighbor checks + rule evaluation
                // - Total operations: ~27.6 million comparisons/second
                // - Modern GPUs handle this easily due to massive parallelism
            }
        }

        Ok(())
    }
}

// ============================================================================
// ADVANCED COMPUTE SHADER CONCEPTS AND GPGPU THEORY
// ============================================================================

// ## GPU Architecture Deep Dive: Compute vs Graphics Pipeline
//
// ### Compute Shaders vs Traditional Graphics Shaders:
// 
// #### Graphics Pipeline (Vertex/Fragment):
// - **Fixed Function**: Rigid pipeline stages (vertex -> rasterization -> fragment)
// - **Data Flow**: Linear progression through pipeline stages
// - **Output**: Rendered pixels to framebuffer
// - **Synchronization**: Hardware-managed between pipeline stages
// - **Memory Access**: Limited to uniforms, textures, and vertex buffers
//
// #### Compute Pipeline:
// - **Flexible Function**: Arbitrary computation with no fixed stages
// - **Data Flow**: User-defined, can be complex and non-linear
// - **Output**: Can write to textures, buffers, or atomic counters
// - **Synchronization**: Manual barriers and memory fences required
// - **Memory Access**: Full read/write access to storage resources
//
// ### GPU Memory Hierarchy for Compute:
//
// #### Memory Types and Performance:
// - **Registers**: ~1 cycle latency, 64KB per SM, automatic allocation
// - **Shared Memory**: ~1-5 cycles, 48-96KB per SM, programmer-controlled
// - **L1 Cache**: ~10-20 cycles, 16-128KB per SM, automatic caching
// - **L2 Cache**: ~30-50 cycles, 512KB-6MB total, shared across GPU
// - **Global Memory**: ~400-800 cycles, 8-24GB total, high bandwidth
//
// #### Memory Coalescing:
// ```
// // GOOD: Coalesced access (consecutive threads access consecutive memory)
// workgroup_id.x * 8 + local_id.x  // Thread pattern: 0,1,2,3,4,5,6,7,8,9...
// 
// // BAD: Strided access (threads access scattered memory)
// workgroup_id.x * 64 + local_id.x // Thread pattern: 0,64,128,192...
// ```
//
// ## Workgroup Theory and Optimization:
//
// ### Thread Hierarchy:
// ```
// GPU Device
// ├── Streaming Multiprocessors (SMs) - 20-100+ units
// │   ├── Warps/Wavefronts - 32-64 threads executing in lockstep
// │   │   └── Individual Threads - SIMD execution
// │   └── Shared Memory - Fast inter-thread communication
// └── Global Memory - Large capacity, high latency
// ```
//
// ### Optimal Workgroup Sizing:
// #### Occupancy Calculation:
// - **Threads per SM**: 1024-2048 (hardware dependent)
// - **Workgroups per SM**: Limited by shared memory and register usage
// - **Target Occupancy**: 50-100% for best performance
// - **Our Choice**: 64 threads (8x8) allows 16-32 workgroups per SM
//
// #### Memory Access Patterns:
// ```rust
// // Game of Life neighbor access pattern per thread:
// let neighbors = [
//     (-1, -1), ( 0, -1), ( 1, -1),  // Top row
//     (-1,  0),           ( 1,  0),  // Middle row (excluding center)
//     (-1,  1), ( 0,  1), ( 1,  1),  // Bottom row
// ];
// // 8x8 workgroup ensures neighbors often in same cache line
// ```
//
// ## Performance Optimization Strategies:
//
// ### Memory Bandwidth Optimization:
// #### Texture Format Analysis:
// - **R32Float**: 4 bytes per cell, more than needed for binary states
// - **R8Uint**: 1 byte per cell, 4x memory savings
// - **Packed Bits**: 32 cells per u32, 32x memory savings (complex indexing)
// - **Trade-off**: Memory vs computation complexity
//
// #### Cache Optimization:
// ```rust
// // Shared memory optimization for large neighborhoods:
// // Load 10x10 block into shared memory for 8x8 workgroup
// // Each thread loads 1.56 cells on average (10*10/64)
// // All 8 neighbors guaranteed to be in shared memory
// ```
//
// ### Algorithmic Optimizations:
//
// #### Sparse Computation:
// - **Problem**: Many cells remain dead for long periods
// - **Solution**: Track "active regions" and only compute those areas
// - **Implementation**: Use atomic counters to build active cell lists
// - **Performance**: 10-100x speedup for sparse patterns
//
// #### Hierarchical Simulation:
// - **Multi-Resolution**: Low-res simulation for distant areas
// - **Temporal Coherence**: Skip computation for stable regions
// - **Adaptive Refinement**: Increase resolution near interesting patterns
//
// ## Real-World GPGPU Applications:
//
// ### Scientific Computing:
// #### Fluid Dynamics (Navier-Stokes):
// ```glsl
// // Lattice Boltzmann Method on GPU
// // Each cell stores 9 velocity vectors (D2Q9 model)
// // Collision step: Local computation per cell
// // Streaming step: Global memory access pattern
// ```
//
// #### N-Body Simulation:
// - **Gravitational Physics**: O(N²) force calculations
// - **GPU Advantage**: Massive parallelism for particle interactions
// - **Optimization**: Spatial partitioning, tree algorithms
// - **Applications**: Galaxy formation, protein folding
//
// ### Machine Learning:
// #### Neural Network Training:
// - **Matrix Multiplication**: Core operation benefits from GPU SIMD
// - **Gradient Descent**: Parallel computation across training samples
// - **Memory Pattern**: Dense matrix operations with high reuse
// - **Performance**: 10-100x speedup over CPU for large networks
//
// #### Computer Vision:
// - **Convolution**: Perfect fit for GPU parallel architecture
// - **Image Filtering**: Independent pixel operations
// - **Feature Detection**: SIMD operations on image regions
//
// ## Advanced Synchronization and Communication:
//
// ### Memory Barriers and Synchronization:
// ```wgsl
// // Ensure all threads complete writes before next phase
// workgroupBarrier();
// storageBarrier(); // Synchronize storage buffer access
// textureBarrier(); // Synchronize texture access
// ```
//
// ### Atomic Operations:
// ```wgsl
// // Atomic increment for counting live neighbors
// atomicAdd(&neighbor_count, 1u);
// 
// // Atomic compare-and-swap for lock-free algorithms
// atomicCompareExchangeWeak(&cell_state, old_val, new_val);
// ```
//
// ### Inter-Workgroup Communication:
// - **Global Memory**: Shared between all workgroups
// - **Atomic Counters**: Coordination primitives
// - **Prefix Sums**: Parallel algorithm building block
// - **Reduction Operations**: Combining results across threads
//
// ## Debugging and Profiling GPGPU Code:
//
// ### Performance Metrics:
// #### Key Performance Indicators:
// - **Occupancy**: Percentage of maximum theoretical threads active
// - **Memory Throughput**: GB/s achieved vs theoretical maximum
// - **ALU Utilization**: Arithmetic intensity vs memory bandwidth
// - **Cache Hit Rate**: L1/L2 cache effectiveness
//
// #### Profiling Tools:
// - **NVIDIA Nsight Compute**: Detailed compute shader analysis
// - **AMD GPU Profiler**: Wavefront occupancy and memory analysis
// - **PIX**: Microsoft GPU debugging for DirectX compute
// - **RenderDoc**: Compute shader debugging and resource inspection
//
// ### Common Performance Pitfalls:
// #### Memory Issues:
// - **Bank Conflicts**: Shared memory access patterns
// - **Uncoalesced Access**: Scattered global memory reads
// - **Cache Thrashing**: Poor temporal/spatial locality
// - **Memory Divergence**: Threads accessing different memory types
//
// #### Execution Issues:
// - **Branch Divergence**: Threads in same warp taking different paths
// - **Workgroup Imbalance**: Uneven work distribution
// - **Register Pressure**: Too many variables reducing occupancy
// - **Synchronization Overhead**: Excessive barriers
//
// ## Extending the Game of Life Example:
//
// ### Advanced Features:
// #### Multi-Species Simulation:
// ```rust
// // Use RGB channels for different species
// struct MultiCell {
//     species_a: f32,  // Red channel
//     species_b: f32,  // Green channel  
//     species_c: f32,  // Blue channel
//     // Each species has different survival rules
// }
// ```
//
// #### Continuous States:
// ```rust
// // Cells have health values between 0.0 and 1.0
// // Gradual birth/death instead of binary states
// // Enables smoother visual transitions
// let health = mix(current_health, target_health, decay_rate);
// ```
//
// #### 3D Cellular Automata:
// - **Volume Textures**: 3D grid for volumetric simulation
// - **26 Neighbors**: Each cell checks surrounding 3x3x3 cube
// - **Applications**: Crystal growth, tissue simulation, 3D Game of Life
//
// ### Performance Scaling:
// #### Large-Scale Simulations:
// - **Multiple GPUs**: Distribute grid across multiple devices
// - **Streaming**: Process larger-than-memory simulations
// - **Hierarchical LOD**: Different resolutions for different regions
// - **Temporal Compression**: Store keyframes, interpolate between
//
// This example demonstrates the power of GPU compute for parallel algorithms
// and serves as a foundation for understanding more complex GPGPU applications
// in scientific computing, machine learning, and real-time simulation.
