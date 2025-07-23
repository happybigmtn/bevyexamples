//! This example shows how to manually render 2d items using "mid level render apis" with a custom
//! pipeline for 2d meshes.
//! It doesn't use the [`Material2d`] abstraction, but changes the vertex buffer to include vertex color.
//! Check out the "mesh2d" example for simpler / higher level 2d meshes.
//!
//! This is an advanced example that demonstrates:
//! - Creating custom vertex attributes (per-vertex color)
//! - Writing a custom render pipeline from scratch
//! - Manual mesh construction with precise vertex placement
//! - GPU pipeline specialization for different rendering scenarios
//! - Integration with Bevy's rendering system
//!
//! Think of this like building your own custom paintbrush instead of using the
//! standard ones. It's more work, but gives you complete control over how your
//! graphics are rendered.
//!
//! [`Material2d`]: bevy::sprite::Material2d

use bevy::{
    // weak_handle! macro creates handles with specific UUIDs for assets
    asset::weak_handle,
    color::palettes::basic::YELLOW,
    // Transparent2d is the render phase for transparent 2D objects
    // CORE_2D_DEPTH_FORMAT defines the depth buffer format for 2D rendering
    core_pipeline::core_2d::{Transparent2d, CORE_2D_DEPTH_FORMAT},
    // FloatOrd wraps f32 to make it orderable (needed for sorting transparent objects)
    math::{ops, FloatOrd},
    prelude::*,
    render::{
        // Mesh construction types - Indices define triangle connectivity
        mesh::{Indices, MeshVertexAttribute, RenderMesh},
        // RenderAssetUsages controls where mesh data lives (CPU vs GPU)
        render_asset::{RenderAssetUsages, RenderAssets},
        // Render phases organize draw calls. Think of them as render queues.
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItemExtraIndex, SetItemPipeline,
            ViewSortedRenderPhases,
        },
        // Low-level GPU pipeline configuration. This is where we define exactly
        // how the GPU should process our vertices and pixels.
        render_resource::{
            BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
            DepthStencilState, Face, FragmentState, FrontFace, MultisampleState, PipelineCache,
            PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipelineDescriptor,
            SpecializedRenderPipeline, SpecializedRenderPipelines, StencilFaceState, StencilState,
            TextureFormat, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        // Synchronization between main world and render world
        sync_component::SyncComponentPlugin,
        sync_world::{MainEntityHashMap, RenderEntity},
        // View represents a camera's perspective
        view::{ExtractedView, RenderVisibleEntities, ViewTarget},
        // Extract moves data from main world to render world each frame
        Extract, Render, RenderApp, RenderSystems,
    },
    sprite::{
        // 2D mesh rendering infrastructure we're building upon
        extract_mesh2d, DrawMesh2d, Material2dBindGroupId, Mesh2dPipeline, Mesh2dPipelineKey,
        Mesh2dTransforms, MeshFlags, RenderMesh2dInstance, SetMesh2dBindGroup,
        SetMesh2dViewBindGroup,
    },
};
// PI constant for trigonometry when creating our star shape
use std::f32::consts::PI;

fn main() {
    App::new()
        // DefaultPlugins provides standard Bevy functionality
        // ColoredMesh2dPlugin is our custom plugin for rendering colored meshes
        .add_plugins((DefaultPlugins, ColoredMesh2dPlugin))
        // The star system creates our star mesh at startup
        .add_systems(Startup, star)
        .run();
}

fn star(
    mut commands: Commands,
    // We will add a new Mesh for the star being created
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Let's define the mesh for the object we want to draw: a nice star.
    // We will specify here what kind of topology is used to define the mesh,
    // that is, how triangles are built from the vertices. We will use a
    // triangle list, meaning that each vertex of the triangle has to be
    // specified. We set `RenderAssetUsages::RENDER_WORLD`, meaning this mesh
    // will not be accessible in future frames from the `meshes` resource, in
    // order to save on memory once it has been uploaded to the GPU.
    //
    // PrimitiveTopology tells the GPU how to connect vertices:
    // - TriangleList: Every 3 vertices form a triangle
    // - TriangleStrip: Each vertex forms a triangle with the previous two
    // - LineList: Every 2 vertices form a line
    // - PointList: Each vertex is rendered as a point
    let mut star = Mesh::new(
        PrimitiveTopology::TriangleList,
        // RENDER_WORLD means the mesh data only exists on the GPU after upload.
        // This saves RAM but means we can't read the mesh data from the CPU later.
        // Use MAIN_WORLD if you need CPU access after creation.
        RenderAssetUsages::RENDER_WORLD,
    );

    // Vertices need to have a position attribute. We will use the following
    // vertices (I hope you can spot the star in the schema).
    //
    //        1
    //
    //     10   2
    // 9      0      3
    //     8     4
    //        6
    //   7        5
    //
    // These vertices are specified in 3D space.
    // Even though we're doing 2D rendering, positions are still Vec3 (z=0)
    let mut v_pos = vec![[0.0, 0.0, 0.0]];  // Center vertex at index 0
    for i in 0..10 {
        // The angle between each vertex is 1/10 of a full rotation.
        // We have 10 vertices around the star, so 2π/10 = π/5 radians between each
        let a = i as f32 * PI / 5.0;
        // The radius alternates: inner vertices (even i) at radius 100,
        // outer vertices (odd i) at radius 200. This creates the star points.
        // (1 - i % 2) equals 1 when i is even, 0 when i is odd
        let r = (1 - i % 2) as f32 * 100.0 + 100.0;
        // Add the vertex position using trigonometry to place it on the circle.
        // Note: sin/cos are swapped from typical usage to start from the top
        v_pos.push([r * ops::sin(a), r * ops::cos(a), 0.0]);
    }
    // Set the position attribute
    // Mesh::ATTRIBUTE_POSITION is a predefined attribute name that the shader expects
    star.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    
    // And a RGB color attribute as well. A built-in `Mesh::ATTRIBUTE_COLOR` exists, but we
    // use a custom vertex attribute here for demonstration purposes.
    // Colors are packed as u32 for efficiency: RGBA with 8 bits per channel
    let mut v_color: Vec<u32> = vec![LinearRgba::BLACK.as_u32()];  // Center is black
    v_color.extend_from_slice(&[LinearRgba::from(YELLOW).as_u32(); 10]);  // Points are yellow
    
    // Create a custom vertex attribute. Parameters:
    // - Name: Must match what the shader expects
    // - Location: Shader input location (must match shader)
    // - Format: Data type (Uint32 = 32-bit unsigned integer)
    star.insert_attribute(
        MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Uint32),
        v_color,
    );

    // Now, we specify the indices of the vertex that are going to compose the
    // triangles in our star. Vertices in triangles have to be specified in CCW
    // winding (that will be the front face, colored). Since we are using
    // triangle list, we will specify each triangle as 3 vertices
    //   First triangle: 0, 2, 1
    //   Second triangle: 0, 3, 2
    //   Third triangle: 0, 4, 3
    //   etc
    //   Last triangle: 0, 1, 10
    //
    // Winding order determines which side of a triangle is "front":
    // - Counter-clockwise (CCW): Front face (rendered)
    // - Clockwise (CW): Back face (culled/ignored)
    // This is important for performance - we don't render triangles facing away
    let mut indices = vec![0, 1, 10];  // Connect center to first and last vertices
    for i in 2..=10 {
        // Each triangle connects the center (0) to two consecutive perimeter vertices
        indices.extend_from_slice(&[0, i, i - 1]);
    }
    // U32 indices support up to 4 billion vertices. Use U16 for smaller meshes to save memory
    star.insert_indices(Indices::U32(indices));

    // We can now spawn the entities for the star and the camera
    commands.spawn((
        // We use a marker component to identify the custom colored meshes
        // This tells our custom rendering system to handle this entity
        ColoredMesh2d,
        // The `Handle<Mesh>` needs to be wrapped in a `Mesh2d` for 2D rendering
        // meshes.add() stores the mesh and returns a handle (like a library card)
        Mesh2d(meshes.add(star)),
    ));

    // Spawn a 2D camera to view our scene
    // Without this, we'd be rendering to nowhere!
    commands.spawn(Camera2d);
}

/// A marker component for colored 2d meshes
/// Marker components are zero-sized types that act like tags.
/// They let us query for specific entities without storing data.
#[derive(Component, Default)]
pub struct ColoredMesh2d;

/// Custom pipeline for 2d meshes with vertex colors
/// A pipeline defines how the GPU processes vertices and pixels.
/// Think of it as a factory assembly line for turning mesh data into pixels.
#[derive(Resource)]
pub struct ColoredMesh2dPipeline {
    /// This pipeline wraps the standard [`Mesh2dPipeline`]
    /// We reuse Bevy's 2D infrastructure and just customize the parts we need
    mesh2d_pipeline: Mesh2dPipeline,
}

// FromWorld allows creating resources that need access to the ECS World.
// This is necessary when the resource depends on other resources or systems.
impl FromWorld for ColoredMesh2dPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            // Create the standard 2D pipeline and wrap it
            mesh2d_pipeline: Mesh2dPipeline::from_world(world),
        }
    }
}

// We implement `SpecializedRenderPipeline` to customize the default rendering from `Mesh2dPipeline`
// Pipeline specialization allows creating variants of a pipeline for different scenarios
// (e.g., with/without HDR, different MSAA levels, different vertex formats)
impl SpecializedRenderPipeline for ColoredMesh2dPipeline {
    // The key contains flags that determine which pipeline variant to create
    type Key = Mesh2dPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        // Customize how to store the meshes' vertex attributes in the vertex buffer
        // Our meshes only have position and color
        let formats = vec![
            // Position: 3 floats (x, y, z)
            VertexFormat::Float32x3,
            // Color: 1 unsigned 32-bit int (packed RGBA)
            VertexFormat::Uint32,
        ];

        // Create the vertex buffer layout. VertexStepMode::Vertex means
        // we advance to the next vertex after reading all attributes.
        // (The alternative is Instance for instanced rendering)
        let vertex_layout =
            VertexBufferLayout::from_vertex_formats(VertexStepMode::Vertex, formats);

        // Choose render target format based on whether HDR is enabled
        // HDR allows colors brighter than white (>1.0) for advanced lighting
        let format = match key.contains(Mesh2dPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        RenderPipelineDescriptor {
            vertex: VertexState {
                // Use our custom shader
                shader: COLORED_MESH2D_SHADER_HANDLE,
                // Entry point is the function name in the shader to call
                entry_point: "vertex".into(),
                // Shader defs are compile-time flags for conditional compilation
                shader_defs: vec![],
                // Use our custom vertex buffer layout
                buffers: vec![vertex_layout],
            },
            fragment: Some(FragmentState {
                // Fragment shader runs once per pixel
                shader: COLORED_MESH2D_SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format,
                    // Alpha blending: new_color = src * src_alpha + dst * (1 - src_alpha)
                    // This allows transparency
                    blend: Some(BlendState::ALPHA_BLENDING),
                    // Write all color channels (Red, Green, Blue, Alpha)
                    write_mask: ColorWrites::ALL,
                })],
            }),
            // Use the two standard uniforms for 2d meshes
            layout: vec![
                // Bind group 0 is the view uniform (camera matrices)
                self.mesh2d_pipeline.view_layout.clone(),
                // Bind group 1 is the mesh uniform (model transform)
                self.mesh2d_pipeline.mesh_layout.clone(),
            ],
            // Push constants are small amounts of data passed directly to shaders
            push_constant_ranges: vec![],
            primitive: PrimitiveState {
                // Counter-clockwise vertices are front-facing
                front_face: FrontFace::Ccw,
                // Don't render back-facing triangles (performance optimization)
                cull_mode: Some(Face::Back),
                // Whether to clip primitives to the viewport
                unclipped_depth: false,
                // Fill triangles (vs wireframe or points)
                polygon_mode: PolygonMode::Fill,
                // Conservative rasterization ensures all touched pixels are drawn
                conservative: false,
                // Triangle list, strip, or other topology from the key
                topology: key.primitive_topology(),
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_2D_DEPTH_FORMAT,
                // 2D typically doesn't write depth (sprites are sorted by z manually)
                depth_write_enabled: false,
                // GreaterEqual allows equal depth values (common in 2D)
                depth_compare: CompareFunction::GreaterEqual,
                // Stencil test is disabled
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                // Depth bias prevents z-fighting by slightly offsetting depth values
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                // MSAA sample count from the key (1, 2, 4, 8, etc.)
                count: key.msaa_samples(),
                // Sample mask (all bits set = use all samples)
                mask: !0,
                // Alpha to coverage converts alpha to a coverage mask
                alpha_to_coverage_enabled: false,
            },
            // Debug label for GPU debugging tools
            label: Some("colored_mesh2d_pipeline".into()),
            // WGSL-specific flag for workgroup memory
            zero_initialize_workgroup_memory: false,
        }
    }
}

// This specifies how to render a colored 2d mesh
// Type aliases make complex types more readable.
// This is a tuple of render commands that execute in sequence:
type DrawColoredMesh2d = (
    // Set the pipeline (shader program + GPU state)
    SetItemPipeline,
    // Set the view uniform as bind group 0 (camera matrices)
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1 (model transform)
    SetMesh2dBindGroup<1>,
    // Draw the mesh using the bound pipeline and data
    DrawMesh2d,
);

// The custom shader can be inline like here, included from another file at build time
// using `include_str!()`, or loaded like any other asset with `asset_server.load()`.
// WGSL (WebGPU Shading Language) is the shader language used by Bevy.
const COLORED_MESH2D_SHADER: &str = r"
// Import the standard 2d mesh uniforms and set their bind groups
// This gives us access to view and mesh transformation matrices
#import bevy_sprite::mesh2d_functions

// The structure of the vertex buffer is as specified in `specialize()`
struct Vertex {
    // Instance index is used for instanced rendering (drawing many copies efficiently)
    @builtin(instance_index) instance_index: u32,
    // Location 0: vertex position in local space
    @location(0) position: vec3<f32>,
    // Location 1: packed RGBA color as a single u32
    @location(1) color: u32,
};

struct VertexOutput {
    // The vertex shader must set the on-screen position of the vertex
    // clip_position is in 'clip space' (-1 to 1 on each axis)
    @builtin(position) clip_position: vec4<f32>,
    // We pass the vertex color to the fragment shader in location 0
    // Colors between vertices will be smoothly interpolated (gradient effect)
    @location(0) color: vec4<f32>,
};

/// Entry point for the vertex shader
/// Runs once per vertex, transforming positions and preparing data for fragments
@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    // Project the world position of the mesh into screen position
    // This involves: local space -> world space -> view space -> clip space
    let model = mesh2d_functions::get_world_from_local(vertex.instance_index);
    out.clip_position = mesh2d_functions::mesh2d_position_local_to_clip(model, vec4<f32>(vertex.position, 1.0));
    
    // Unpack the `u32` from the vertex buffer into the `vec4<f32>` used by the fragment shader
    // The color is packed as RGBA with 8 bits per channel: 0xAABBGGRR
    // We extract each byte by shifting and masking:
    // - Shift right by 0, 8, 16, 24 bits for R, G, B, A respectively
    // - Mask with 255 (0xFF) to get just the bottom 8 bits
    // - Divide by 255.0 to convert from 0-255 range to 0.0-1.0 range
    out.color = vec4<f32>((vec4<u32>(vertex.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    return out;
}

// The input of the fragment shader must correspond to the output of the vertex shader for all `location`s
struct FragmentInput {
    // The color is interpolated between vertices by default
    // This creates smooth gradients between differently colored vertices
    @location(0) color: vec4<f32>,
};

/// Entry point for the fragment shader
/// Runs once per pixel, determining the final color
@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // Simply output the interpolated color
    // @location(0) means this goes to the first render target (usually the screen)
    return in.color;
}
";

/// Plugin that renders [`ColoredMesh2d`]s
/// Plugins are the main way to extend Bevy. They encapsulate related functionality.
pub struct ColoredMesh2dPlugin;

/// Handle to the custom shader with a unique random ID
/// weak_handle! creates a handle without loading the asset.
/// The UUID must be unique across all assets in the project.
pub const COLORED_MESH2D_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("f48b148f-7373-4638-9900-392b3b3ccc66");

/// Our custom pipeline needs its own instance storage
/// This maps entities to their render data in the render world.
/// Deref/DerefMut allow treating this like the inner HashMap.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct RenderColoredMesh2dInstances(MainEntityHashMap<RenderMesh2dInstance>);

impl Plugin for ColoredMesh2dPlugin {
    fn build(&self, app: &mut App) {
        // Load our custom shader into the asset system
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();
        shaders.insert(
            &COLORED_MESH2D_SHADER_HANDLE,
            // file!() macro provides the current file path for error messages
            Shader::from_wgsl(COLORED_MESH2D_SHADER, file!()),
        );
        // Sync our marker component between main and render worlds
        app.add_plugins(SyncComponentPlugin::<ColoredMesh2d>::default());

        // The render app is a separate ECS world that runs in parallel
        // for better performance. We need to set up our rendering logic there.
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            // Register our custom draw function for the transparent phase
            .add_render_command::<Transparent2d, DrawColoredMesh2d>()
            // Storage for specialized pipeline variants
            .init_resource::<SpecializedRenderPipelines<ColoredMesh2dPipeline>>()
            // Storage for per-entity render data
            .init_resource::<RenderColoredMesh2dInstances>()
            .add_systems(
                // Extract runs between main world update and render world update
                ExtractSchedule,
                // Run after standard mesh extraction to access its data
                extract_colored_mesh2d.after(extract_mesh2d),
            )
            .add_systems(
                // Render schedule runs the actual rendering
                Render,
                // Queue phase decides what to draw and in what order
                queue_colored_mesh2d.in_set(RenderSystems::QueueMeshes),
            );
    }

    fn finish(&self, app: &mut App) {
        // finish() runs after all plugins are built, ensuring dependencies are ready
        // Register our custom pipeline
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<ColoredMesh2dPipeline>();
    }
}

/// Extract the [`ColoredMesh2d`] marker component into the render app
/// Extract systems copy data from the main world to the render world each frame.
/// This separation allows the main world to start simulating the next frame
/// while the render world is still rendering the previous frame.
pub fn extract_colored_mesh2d(
    mut commands: Commands,
    // Local<T> is a system-local resource that persists between calls
    // We use it to track the previous query length for efficient memory allocation
    mut previous_len: Local<usize>,
    // When extracting, you must use `Extract` to mark the `SystemParam`s
    // which should be taken from the main world.
    query: Extract<
        Query<
            (
                Entity,
                // RenderEntity maps main world entities to render world entities
                RenderEntity,
                // ViewVisibility combines Visibility and InheritedVisibility
                &ViewVisibility,
                // GlobalTransform is the final world-space transform
                &GlobalTransform,
                // The mesh handle
                &Mesh2d,
            ),
            // Only query entities with our marker component
            With<ColoredMesh2d>,
        >,
    >,
    mut render_mesh_instances: ResMut<RenderColoredMesh2dInstances>,
) {
    // Pre-allocate based on previous frame for performance
    let mut values = Vec::with_capacity(*previous_len);
    for (entity, render_entity, view_visibility, transform, handle) in &query {
        // Skip invisible entities
        if !view_visibility.get() {
            continue;
        }

        // Convert transform to the format needed by the GPU
        let transforms = Mesh2dTransforms {
            // affine() gets the 3x4 matrix (rotation, scale, translation)
            world_from_local: (&transform.affine()).into(),
            // Flags can indicate special states (like negative scale)
            flags: MeshFlags::empty().bits(),
        };

        // Add the marker component to the render world entity
        values.push((render_entity, ColoredMesh2d));
        // Store the instance data for rendering
        render_mesh_instances.insert(
            entity.into(),
            RenderMesh2dInstance {
                // Extract the asset ID from the handle
                mesh_asset_id: handle.0.id(),
                transforms,
                // We don't use materials, so use default ID
                material_bind_group_id: Material2dBindGroupId::default(),
                // Disable automatic batching for this example
                automatic_batching: false,
                tag: 0,
            },
        );
    }
    // Remember the count for next frame
    *previous_len = values.len();
    // Batch insert for efficiency
    commands.try_insert_batch(values);
}

/// Queue the 2d meshes marked with [`ColoredMesh2d`] using our custom pipeline and draw function
/// Queue systems prepare draw calls for the GPU. They determine what to draw,
/// in what order, and with which pipeline.
pub fn queue_colored_mesh2d(
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    colored_mesh2d_pipeline: Res<ColoredMesh2dPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<ColoredMesh2dPipeline>>,
    // PipelineCache compiles and caches GPU pipelines
    pipeline_cache: Res<PipelineCache>,
    // GPU-ready mesh data
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderColoredMesh2dInstances>,
    // Render phases store draw commands sorted by draw order
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    // Each camera is a "view" with its own render settings
    views: Query<(&RenderVisibleEntities, &ExtractedView, &Msaa)>,
) {
    if render_mesh_instances.is_empty() {
        return;
    }
    // Iterate each view (a camera is a view)
    for (visible_entities, view, msaa) in &views {
        // Get the render phase for this view
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        // Get the draw function ID for our custom draw command
        let draw_colored_mesh2d = transparent_draw_functions.read().id::<DrawColoredMesh2d>();

        // Build the pipeline key based on view settings
        // The | operator combines bit flags
        let mesh_key = Mesh2dPipelineKey::from_msaa_samples(msaa.samples())
            | Mesh2dPipelineKey::from_hdr(view.hdr);

        // Queue all entities visible to that view
        for (render_entity, visible_entity) in visible_entities.iter::<Mesh2d>() {
            if let Some(mesh_instance) = render_mesh_instances.get(visible_entity) {
                let mesh2d_handle = mesh_instance.mesh_asset_id;
                let mesh2d_transforms = &mesh_instance.transforms;
                // Get our specialized pipeline
                let mut mesh2d_key = mesh_key;
                // Ensure the mesh has been uploaded to the GPU
                let Some(mesh) = render_meshes.get(mesh2d_handle) else {
                    continue;
                };
                // Add topology to the key (triangles vs lines vs points)
                mesh2d_key |= Mesh2dPipelineKey::from_primitive_topology(mesh.primitive_topology());

                // Get or create the specialized pipeline for these specific settings
                let pipeline_id =
                    pipelines.specialize(&pipeline_cache, &colored_mesh2d_pipeline, mesh2d_key);

                // Extract Z coordinate for depth sorting
                let mesh_z = mesh2d_transforms.world_from_local.translation.z;
                // Add a draw command to the transparent phase
                transparent_phase.add(Transparent2d {
                    // Both render entity and main entity for tracking
                    entity: (*render_entity, *visible_entity),
                    draw_function: draw_colored_mesh2d,
                    pipeline: pipeline_id,
                    // The 2d render items are sorted according to their z value before rendering,
                    // in order to get correct transparency (back to front)
                    sort_key: FloatOrd(mesh_z),
                    // Batch range 0..1 means this item can't be batched with others
                    batch_range: 0..1,
                    extra_index: PhaseItemExtraIndex::None,
                    extracted_index: usize::MAX,
                    // Whether the mesh uses an index buffer
                    indexed: mesh.indexed(),
                });
            }
        }
    }
}
