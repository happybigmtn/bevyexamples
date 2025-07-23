//! Create a custom material to draw basic lines in 3D
//!
//! # Drawing Lines in 3D Graphics
//!
//! Unlike triangles (which fill an area), lines are 1D primitives that only have
//! length, no width. In modern graphics APIs, drawing lines requires special
//! handling because the GPU is optimized for triangle rendering.
//!
//! This example demonstrates:
//! 1. **Custom Line Material**: A material that renders meshes as lines
//! 2. **Line List**: Disconnected line segments (each pair of vertices = one line)
//! 3. **Line Strip**: Connected line segments (consecutive vertices form a continuous line)
//!
//! # Why Custom Materials?
//!
//! Bevy's standard materials are designed for surface rendering (triangles).
//! To render lines, we need to:
//! - Change the polygon mode from "Fill" to "Line"
//! - Use appropriate primitive topologies (LineList or LineStrip)
//! - Create meshes with the right vertex layout
//!
//! Note: These are "geometric" lines (1 pixel wide). For thick lines or stylized
//! lines, you'd typically use triangle strips or geometry shaders.

use bevy::{
    pbr::{
        MaterialPipeline,    // The rendering pipeline for materials
        MaterialPipelineKey, // Key for pipeline specialization
    },
    prelude::*,
    reflect::TypePath, // Required for asset type identification
    render::{
        mesh::{
            MeshVertexBufferLayoutRef, // Reference to vertex buffer layout
            PrimitiveTopology,         // How vertices connect (points/lines/triangles)
        },
        render_asset::RenderAssetUsages, // Where assets are stored (CPU/GPU)
        render_resource::{
            AsBindGroup,                   // Derive macro for GPU bindings
            PolygonMode,                   // Fill, Line, or Point rendering
            RenderPipelineDescriptor,      // GPU pipeline configuration
            ShaderRef,                     // Reference to shader code
            SpecializedMeshPipelineError,  // Error type for pipeline creation
        },
    },
};

/// This example uses a shader source file from the assets subdirectory
/// The shader defines how each pixel of the line gets colored
const SHADER_ASSET_PATH: &str = "shaders/line_material.wgsl";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Register our custom LineMaterial type
            // MaterialPlugin handles the rendering pipeline setup
            MaterialPlugin::<LineMaterial>::default()
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LineMaterial>>,
) {
    // Spawn a list of lines with start and end points for each line
    // LineList means each pair of vertices forms a separate line segment
    commands.spawn((
        Mesh3d(meshes.add(LineList {
            lines: vec![
                // First line: from origin to (1, 1, 0)
                (Vec3::ZERO, Vec3::new(1.0, 1.0, 0.0)),
                // Second line: from (1, 1, 0) to (1, 0, 0)
                (Vec3::new(1.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
            ],
        })),
        MeshMaterial3d(materials.add(LineMaterial {
            color: LinearRgba::GREEN,
        })),
        // Position to the left
        Transform::from_xyz(-1.5, 0.0, 0.0),
    ));

    // Spawn a line strip that goes from point to point
    // LineStrip means vertices are connected consecutively:
    // vertex 0 -> vertex 1 -> vertex 2 -> etc.
    commands.spawn((
        Mesh3d(meshes.add(LineStrip {
            points: vec![
                Vec3::ZERO,                  // Start point
                Vec3::new(1.0, 1.0, 0.0),   // Middle point
                Vec3::new(1.0, 0.0, 0.0),   // End point
            ],
        })),
        MeshMaterial3d(materials.add(LineMaterial {
            color: LinearRgba::BLUE,
        })),
        // Position to the right
        Transform::from_xyz(0.5, 0.0, 0.0),
    ));

    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        // Position camera to see both line examples
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// Custom material for rendering lines
// The derives provide necessary trait implementations:
// - Asset: Can be stored in Assets<T>
// - TypePath: Required for asset type identification
// - AsBindGroup: Generates GPU binding code for uniforms
// - Default, Debug, Clone: Standard traits
#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
struct LineMaterial {
    // Color uniform sent to the shader
    // uniform(0) means this is bound to binding location 0
    #[uniform(0)]
    color: LinearRgba,
}

// Implement the Material trait to define rendering behavior
impl Material for LineMaterial {
    // Specify which shader to use for fragment (pixel) shading
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    // Customize the render pipeline for line rendering
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the crucial part: change from filled triangles to lines
        // PolygonMode::Fill (default) - Fills triangles
        // PolygonMode::Line - Draws only edges
        // PolygonMode::Point - Draws only vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of lines with a start and end position
/// Each line is independent - they don't connect to each other
#[derive(Debug, Clone)]
struct LineList {
    lines: Vec<(Vec3, Vec3)>, // Each tuple is (start_point, end_point)
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        // Convert pairs of points into a flat list of vertices
        // [(A,B), (C,D)] becomes [A, B, C, D]
        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();

        Mesh::new(
            // LineList topology means:
            // - Vertices 0,1 form first line
            // - Vertices 2,3 form second line
            // - Vertices 4,5 form third line, etc.
            PrimitiveTopology::LineList,
            // Store mesh data only on GPU for rendering
            RenderAssetUsages::RENDER_WORLD,
        )
        // Add the vertices positions as an attribute
        // This tells the GPU where each vertex is located in 3D space
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}

/// A list of points that will have a line drawn between each consecutive points
/// Forms a continuous path through all points
#[derive(Debug, Clone)]
struct LineStrip {
    points: Vec<Vec3>, // Points to connect in order
}

impl From<LineStrip> for Mesh {
    fn from(line: LineStrip) -> Self {
        Mesh::new(
            // LineStrip topology means:
            // - Line from vertex 0 to vertex 1
            // - Line from vertex 1 to vertex 2
            // - Line from vertex 2 to vertex 3, etc.
            // Forms a continuous path
            PrimitiveTopology::LineStrip,
            RenderAssetUsages::RENDER_WORLD,
        )
        // Add the point positions as an attribute
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line.points)
    }
}
