//! # Custom Render Phase Items: Deep GPU Pipeline Integration
//!
//! This example demonstrates the most advanced level of GPU programming integration with Bevy's
//! render system: **custom render phase items**. This technique allows you to inject completely
//! custom GPU draw commands directly into Bevy's optimized rendering pipeline while maintaining
//! compatibility with Bevy's culling, batching, and sorting systems.
//!
//! ## Advanced GPU Programming Concepts:
//! - **Render Phases**: Ordered stages of GPU commands (e.g., opaque, transparent, UI)
//! - **Draw Functions**: Parameterized GPU command execution functions
//! - **Render Pipelines**: Complete GPU state descriptions (shaders, blending, depth testing)
//! - **Specialized Pipelines**: Runtime compilation of pipelines based on rendering conditions
//! - **GPU Buffer Management**: Direct control over vertex/index buffer creation and binding
//! - **Render Command Composition**: Building complex draw operations from smaller components
//!
//! ## Why Use Custom Phase Items:
//! - **Performance**: Integrate with Bevy's optimized culling and sorting systems
//! - **Flexibility**: Complete control over GPU state while leveraging Bevy infrastructure
//! - **Composability**: Reuse Bevy's built-in rendering components (lighting, shadows, etc.)
//! - **Debugging**: Benefit from Bevy's render graph visualization and profiling tools
//!
//! ## Alternative Approaches:
//! - **Custom Materials**: Simpler, for shader-only customizations
//! - **Render Nodes**: Lower-level, for compute shaders or complete pipeline control
//! - **Post-Processing**: For screen-space effects

use bevy::{
    // Core 3D pipeline integration - accessing Bevy's built-in opaque render phase
    core_pipeline::core_3d::{Opaque3d, Opaque3dBatchSetKey, Opaque3dBinKey, CORE_3D_DEPTH_FORMAT},
    ecs::{
        // Tick: Change detection system for forcing render pipeline updates
        component::Tick,
        // ROQueryItem: Read-only query items for render commands
        query::ROQueryItem,
        // System parameter types for render commands
        system::{lifetimeless::SRes, SystemParamItem},
    },
    prelude::*,
    render::{
        // Component extraction system - moves data from main world to render world
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        // Axis-aligned bounding box for frustum culling
        primitives::Aabb,
        render_phase::{
            // Core render phase types for managing GPU draw commands
            AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, InputUniformIndex, PhaseItem,
            RenderCommand, RenderCommandResult, SetItemPipeline, TrackedRenderPass,
            ViewBinnedRenderPhases,
        },
        render_resource::{
            // GPU resource management - buffers, pipelines, and render state
            BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState,
            FragmentState, IndexFormat, MultisampleState, PipelineCache, PrimitiveState,
            RawBufferVec, RenderPipelineDescriptor, SpecializedRenderPipeline,
            SpecializedRenderPipelines, TextureFormat, VertexAttribute, VertexBufferLayout,
            VertexFormat, VertexState, VertexStepMode,
        },
        // GPU device and command queue interfaces
        renderer::{RenderDevice, RenderQueue},
        // View and visibility systems for camera culling
        view::{self, ExtractedView, RenderVisibleEntities, VisibilityClass},
        // Render app scheduling and systems
        Render, RenderApp, RenderSystems,
    },
};
// Bytemuck: Zero-copy casting between Rust types and raw bytes for GPU upload
use bytemuck::{Pod, Zeroable};

/// Marker component identifying entities that use our custom rendering pipeline.
/// This demonstrates the **render world extraction pattern** - how data flows from
/// the main ECS world to the specialized render world that runs on a separate thread.
///
/// ## Key Concepts:
/// - **ExtractComponent**: Automatically copies this component from main world to render world
/// - **VisibilityClass**: Required for Bevy's frustum culling system to work correctly
/// - **Component Hook**: The `on_add` function registers this entity for visibility testing
/// 
/// ## Render World Architecture:
/// The render world is a parallel ECS world that contains only rendering-related data.
/// This separation allows the main world to continue game logic while rendering happens.
#[derive(Clone, Component, ExtractComponent)]
#[require(VisibilityClass)]
#[component(on_add = view::add_visibility_class::<CustomRenderedEntity>)]
struct CustomRenderedEntity;

/// Resource containing our custom shader handle for pipeline specialization.
/// This demonstrates **lazy shader loading** - shaders are loaded asynchronously
/// and compiled by the GPU driver when first needed.
///
/// ## Pipeline Specialization Pattern:
/// Modern graphics APIs require different pipeline objects for different rendering
/// conditions (MSAA levels, HDR vs SDR, etc.). This resource provides the base
/// shader that gets specialized at runtime.
#[derive(Resource)]
struct CustomPhasePipeline {
    shader: Handle<Shader>,
}

/// **Core RenderCommand**: Executes the actual GPU draw call for our custom geometry.
/// This is where CPU-side preparation meets GPU-side execution. RenderCommands are
/// designed to be composable, stateless, and fast - they run during the critical
/// render loop where performance matters most.
///
/// ## GPU Command Generation:
/// RenderCommands translate high-level rendering intent into low-level GPU API calls
/// like `set_vertex_buffer()`, `set_index_buffer()`, and `draw_indexed()`.
struct DrawCustomPhaseItem;

impl<P> RenderCommand<P> for DrawCustomPhaseItem
where
    P: PhaseItem,
{
    // System parameters this render command needs access to
    // SRes = System Resource (render world resource access)
    type Param = SRes<CustomPhaseItemBuffers>;

    // Per-view data this command needs (none in this simple example)
    type ViewQuery = ();

    // Per-entity data this command needs (none in this simple example)  
    type ItemQuery = ();

    /// The critical render function: translates rendering intent to GPU commands.
    /// This runs during the hot path of the render loop, so performance is crucial.
    ///
    /// ## GPU State Machine:
    /// Modern GPUs are state machines - you configure rendering state (buffers, textures,
    /// shaders) then issue draw commands. This function demonstrates the typical pattern:
    /// 1. Bind vertex data (positions, colors, normals, etc.)
    /// 2. Bind index data (triangle connectivity)  
    /// 3. Issue indexed draw call (GPU processes triangles)
    fn render<'w>(
        _: &P,
        _: ROQueryItem<'w, Self::ViewQuery>,
        _: Option<ROQueryItem<'w, Self::ItemQuery>>,
        custom_phase_item_buffers: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // Extract the buffer resource (Rust borrow checker requirement)
        let custom_phase_item_buffers = custom_phase_item_buffers.into_inner();

        // CRITICAL: Bind vertex buffer to GPU pipeline slot 0
        // The GPU will read vertex data from this buffer during draw call execution
        pass.set_vertex_buffer(
            0,  // Vertex buffer slot (must match shader layout)
            custom_phase_item_buffers
                .vertices
                .buffer()
                .unwrap()
                .slice(..),  // Use entire buffer
        );

        // CRITICAL: Bind index buffer for indexed drawing
        // Indices specify which vertices form triangles, enabling vertex reuse
        pass.set_index_buffer(
            custom_phase_item_buffers
                .indices
                .buffer()
                .unwrap()
                .slice(..),
            0,  // Byte offset into index buffer
            IndexFormat::Uint32,  // Each index is a 32-bit unsigned integer
        );

        // CRITICAL: Issue the actual draw call to the GPU
        // draw_indexed(indices, base_vertex, instances)
        // - 0..3: Draw indices 0, 1, 2 (one triangle)
        // - 0: Base vertex offset
        // - 0..1: Draw one instance of this geometry
        pass.draw_indexed(0..3, 0, 0..1);

        RenderCommandResult::Success
    }
}

/// **GPU Buffer Management**: Contains the actual geometry data uploaded to GPU memory.
/// This demonstrates **static geometry** - data uploaded once and reused many times.
/// 
/// ## GPU Memory Architecture:
/// - **GPU Memory**: Separate from CPU RAM, optimized for parallel access
/// - **Buffer Types**: Different usage patterns (vertex data, index data, uniform data)
/// - **Memory Layout**: Data must be properly aligned for GPU hardware requirements
#[derive(Resource)]
struct CustomPhaseItemBuffers {
    /// Vertex buffer containing position and color data for our triangle.
    /// RawBufferVec provides efficient GPU buffer management with minimal overhead.
    /// It handles the complex details of GPU memory allocation and synchronization.
    vertices: RawBufferVec<Vertex>,

    /// Index buffer defining triangle connectivity (which vertices form triangles).
    /// Using indices allows vertex reuse - multiple triangles can share vertices,
    /// reducing memory usage and improving cache performance.
    indices: RawBufferVec<u32>,
}

/// **Vertex Data Structure**: Represents a single vertex with GPU-compatible memory layout.
/// This demonstrates critical concepts for CPU-GPU data transfer.
///
/// ## Memory Layout Requirements:
/// - **#[repr(C)]**: Forces C-style memory layout for predictable GPU compatibility
/// - **Pod + Zeroable**: Bytemuck traits allowing safe zero-copy transfer to GPU
/// - **Explicit Padding**: GPU hardware often requires 16-byte alignment for vector types
/// - **Field Ordering**: Matches the vertex shader's expected input layout
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    /// 3D world-space position (x, y, z coordinates)
    position: Vec3,
    /// Manual padding to ensure proper GPU memory alignment (Vec3 + u32 = 16 bytes)
    pad0: u32,
    /// RGB color values (red, green, blue) - each component 0.0 to 1.0
    color: Vec3,
    /// Additional padding for consistent 32-byte vertex size
    pad1: u32,
}

impl Vertex {
    /// Constructs a new vertex with proper padding for GPU memory layout.
    /// The const fn allows compile-time initialization of static vertex data.
    const fn new(position: Vec3, color: Vec3) -> Vertex {
        Vertex {
            position,
            color,
            // Zero-initialize padding fields - critical for deterministic GPU behavior
            pad0: 0,
            pad1: 0,
        }
    }
}

/// **Composable Render Commands**: Defines the sequence of GPU operations for our custom item.
/// This demonstrates Bevy's **command composition pattern** - complex rendering operations
/// built from smaller, reusable components.
///
/// ## Command Sequence:
/// 1. **SetItemPipeline**: Binds the render pipeline (shaders, blend state, etc.)
/// 2. **DrawCustomPhaseItem**: Binds buffers and issues the draw call
///
/// Commands execute in order, building up GPU state before the final draw operation.
type DrawCustomPhaseItemCommands = (SetItemPipeline, DrawCustomPhaseItem);

/// **Static Geometry Data**: An equilateral triangle with per-vertex colors.
/// This demonstrates **compile-time vertex data** and basic 3D coordinate math.
///
/// ## Triangle Geometry:
/// - **Equilateral Triangle**: All sides equal length, centered at origin
/// - **Color Gradient**: Red (bottom-left) → Green (bottom-right) → Blue (top)
/// - **Z-Position**: 0.5 units forward from origin for visibility
static VERTICES: [Vertex; 3] = [
    // Bottom-left vertex: Red color
    Vertex::new(vec3(-0.866, -0.5, 0.5), vec3(1.0, 0.0, 0.0)),
    // Bottom-right vertex: Green color  
    Vertex::new(vec3(0.866, -0.5, 0.5), vec3(0.0, 1.0, 0.0)),
    // Top vertex: Blue color
    Vertex::new(vec3(0.0, 1.0, 0.5), vec3(0.0, 0.0, 1.0)),
];

/// Application entry point demonstrating the **dual-world architecture** of Bevy rendering.
/// This shows how to properly set up systems in both the main world and render world.
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        // CRITICAL: Register component extraction from main world to render world
        .add_plugins(ExtractComponentPlugin::<CustomRenderedEntity>::default())
        .add_systems(Startup, setup);

    // CRITICAL: Render-specific setup must happen in the RenderApp, not the main App
    // The RenderApp is a separate ECS world that runs rendering systems on a different thread
    app.get_sub_app_mut(RenderApp)
        .unwrap()
        // Initialize render pipeline resources
        .init_resource::<CustomPhasePipeline>()
        .init_resource::<SpecializedRenderPipelines<CustomPhasePipeline>>()
        // Register our custom draw commands with the opaque render phase
        .add_render_command::<Opaque3d, DrawCustomPhaseItemCommands>()
        // Prepare phase: Set up GPU buffers before rendering
        .add_systems(
            Render,
            prepare_custom_phase_item_buffers.in_set(RenderSystems::Prepare),
        )
        // Queue phase: Enqueue render items into render phases  
        .add_systems(Render, queue_custom_phase_item.in_set(RenderSystems::Queue));

    app.run();
}

/// Spawns the objects in the scene.
fn setup(mut commands: Commands) {
    // Spawn a single entity that has custom rendering. It'll be extracted into
    // the render world via [`ExtractComponent`].
    commands.spawn((
        Visibility::default(),
        Transform::default(),
        // This `Aabb` is necessary for the visibility checks to work.
        Aabb {
            center: Vec3A::ZERO,
            half_extents: Vec3A::splat(0.5),
        },
        CustomRenderedEntity,
    ));

    // Spawn the camera.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Creates the [`CustomPhaseItemBuffers`] resource.
///
/// This must be done in a startup system because it needs the [`RenderDevice`]
/// and [`RenderQueue`] to exist, and they don't until [`App::run`] is called.
fn prepare_custom_phase_item_buffers(mut commands: Commands) {
    commands.init_resource::<CustomPhaseItemBuffers>();
}

/// A render-world system that enqueues the entity with custom rendering into
/// the opaque render phases of each view.
fn queue_custom_phase_item(
    pipeline_cache: Res<PipelineCache>,
    custom_phase_pipeline: Res<CustomPhasePipeline>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<CustomPhasePipeline>>,
    views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    mut next_tick: Local<Tick>,
) {
    let draw_custom_phase_item = opaque_draw_functions
        .read()
        .id::<DrawCustomPhaseItemCommands>();

    // Render phases are per-view, so we need to iterate over all views so that
    // the entity appears in them. (In this example, we have only one view, but
    // it's good practice to loop over all views anyway.)
    for (view, view_visible_entities, msaa) in views.iter() {
        let Some(opaque_phase) = opaque_render_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        // Find all the custom rendered entities that are visible from this
        // view.
        for &entity in view_visible_entities.get::<CustomRenderedEntity>().iter() {
            // Ordinarily, the [`SpecializedRenderPipeline::Key`] would contain
            // some per-view settings, such as whether the view is HDR, but for
            // simplicity's sake we simply hard-code the view's characteristics,
            // with the exception of number of MSAA samples.
            let pipeline_id = specialized_render_pipelines.specialize(
                &pipeline_cache,
                &custom_phase_pipeline,
                *msaa,
            );

            // Bump the change tick in order to force Bevy to rebuild the bin.
            let this_tick = next_tick.get() + 1;
            next_tick.set(this_tick);

            // Add the custom render item. We use the
            // [`BinnedRenderPhaseType::NonMesh`] type to skip the special
            // handling that Bevy has for meshes (preprocessing, indirect
            // draws, etc.)
            //
            // The asset ID is arbitrary; we simply use [`AssetId::invalid`],
            // but you can use anything you like. Note that the asset ID need
            // not be the ID of a [`Mesh`].
            opaque_phase.add(
                Opaque3dBatchSetKey {
                    draw_function: draw_custom_phase_item,
                    pipeline: pipeline_id,
                    material_bind_group_index: None,
                    lightmap_slab: None,
                    vertex_slab: default(),
                    index_slab: None,
                },
                Opaque3dBinKey {
                    asset_id: AssetId::<Mesh>::invalid().untyped(),
                },
                entity,
                InputUniformIndex::default(),
                BinnedRenderPhaseType::NonMesh,
                *next_tick,
            );
        }
    }
}

impl SpecializedRenderPipeline for CustomPhasePipeline {
    type Key = Msaa;

    fn specialize(&self, msaa: Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("custom render pipeline".into()),
            layout: vec![],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    // This needs to match the layout of [`Vertex`].
                    attributes: vec![
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x3,
                            offset: 16,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    // Ordinarily, you'd want to check whether the view has the
                    // HDR format and substitute the appropriate texture format
                    // here, but we omit that for simplicity.
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            // Note that if your view has no depth buffer this will need to be
            // changed.
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Always,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: msaa.samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        }
    }
}

impl FromWorld for CustomPhaseItemBuffers {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();

        // Create the vertex and index buffers.
        let mut vbo = RawBufferVec::new(BufferUsages::VERTEX);
        let mut ibo = RawBufferVec::new(BufferUsages::INDEX);

        for vertex in &VERTICES {
            vbo.push(*vertex);
        }
        for index in 0..3 {
            ibo.push(index);
        }

        // These two lines are required in order to trigger the upload to GPU.
        vbo.write_buffer(render_device, render_queue);
        ibo.write_buffer(render_device, render_queue);

        CustomPhaseItemBuffers {
            vertices: vbo,
            indices: ibo,
        }
    }
}

impl FromWorld for CustomPhasePipeline {
    fn from_world(world: &mut World) -> Self {
        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();

        CustomPhasePipeline {
            shader: asset_server.load("shaders/custom_phase_item.wgsl"),
        }
    }
}
