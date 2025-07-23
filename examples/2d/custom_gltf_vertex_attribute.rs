//! Renders a glTF mesh in 2D with a custom vertex attribute.
//!
//! # Custom GLTF Vertex Attributes: Beyond Standard Data
//!
//! While standard 3D models have position, normals, and UV coordinates,
//! sometimes you need custom data per vertex. This example shows how to
//! load and use custom vertex attributes from GLTF files.
//!
//! ## What Are Vertex Attributes?
//!
//! Vertex attributes are data attached to each vertex:
//! - **Position**: Where the vertex is in 3D space
//! - **Normal**: Surface direction for lighting
//! - **UV**: Texture coordinates
//! - **Color**: Per-vertex color data
//! - **Custom**: Your own data (barycentric coords, wind strength, etc.)
//!
//! ## Barycentric Coordinates:
//!
//! Each vertex in a triangle has barycentric coordinates [u, v, w]:
//! - Corner A: [1, 0, 0] - Red corner
//! - Corner B: [0, 1, 0] - Green corner  
//! - Corner C: [0, 0, 1] - Blue corner
//! - Center: [0.33, 0.33, 0.33] - Mixed
//!
//! ## Common Uses:
//!
//! - **Wireframe effects**: Show triangle edges
//! - **Debug visualization**: See mesh topology
//! - **Artistic effects**: Triangle-aware shading
//! - **Edge detection**: Find triangle boundaries
//!
//! ## This Example Shows:
//!
//! - Loading custom GLTF vertex attributes
//! - Creating custom 2D materials with shaders
//! - Using barycentric coordinates for wireframe
//! - Animated border thickness with time

// Rust: Complex nested imports from bevy modules
use bevy::{
    // Rust: GLTF file loading and processing
    gltf::GltfPlugin,
    // Rust: Common Bevy types
    prelude::*,
    // Rust: TypePath trait for asset system
    reflect::TypePath,
    // Rust: Low-level rendering types
    render::{
        // Rust: Mesh vertex data structures
        mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef},
        // Rust: GPU resource types (shaders, pipelines, etc.)
        render_resource::*,
    },
    // Rust: 2D material system
    sprite::{Material2d, Material2dKey, Material2dPlugin},
};

/// This example uses a shader source file from the assets subdirectory
// Rust: const for string literal with &str type
// Used for shader path - compile-time constant
const SHADER_ASSET_PATH: &str = "shaders/custom_gltf_2d.wgsl";

/// This vertex attribute supplies barycentric coordinates for each triangle.
///
/// Each component of the vector corresponds to one corner of a triangle. It's
/// equal to 1.0 in that corner and 0.0 in the other two. Hence, its value in
/// the fragment shader indicates proximity to a corner or the opposite edge.
// Rust: const with complex type - MeshVertexAttribute
// Creates compile-time constant for custom vertex data
const ATTRIBUTE_BARYCENTRIC: MeshVertexAttribute =
    // Rust: Associated function call with parameters
    // "Barycentric" = attribute name in shader
    // 2137464976 = unique ID (hash of name)
    // Float32x3 = 3 floats per vertex (u, v, w coordinates)
    MeshVertexAttribute::new("Barycentric", 2137464976, VertexFormat::Float32x3);

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Configure ambient lighting for the scene
        .insert_resource(AmbientLight {
            // Rust: White ambient light
            color: Color::WHITE,
            // Rust: f32 literal with division
            // 1.0 / 5.0 = 0.2 brightness (dimmed ambient)
            brightness: 1.0 / 5.0f32,
            // Rust: Default other fields
            ..default()
        })
        // Rust: Add multiple plugins as tuple
        .add_plugins((
            // Rust: Configure DefaultPlugins with custom GLTF plugin
            DefaultPlugins.set(
                // Rust: Method chaining on GltfPlugin
                GltfPlugin::default()
                    // Map a custom glTF attribute name to a `MeshVertexAttribute`.
                    // The glTF file used here has an attribute name with *two*
                    // underscores: __BARYCENTRIC
                    // One is stripped to do the comparison here.
                    // Rust: Register custom vertex attribute mapping
                    // "_BARYCENTRIC" = name in GLTF file (minus one underscore)
                    // ATTRIBUTE_BARYCENTRIC = our const attribute definition
                    .add_custom_vertex_attribute("_BARYCENTRIC", ATTRIBUTE_BARYCENTRIC),
            ),
            // Rust: Generic plugin with type parameter
            // Material2dPlugin::<CustomMaterial> enables our custom material
            Material2dPlugin::<CustomMaterial>::default(),
        ))
        // Rust: Register startup system
        .add_systems(Startup, setup)
        // Rust: Start game loop
        .run();
}

// Rust: Setup system with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Read-only access to asset server
    asset_server: Res<AssetServer>,
    // Rust: Mutable access to custom material assets
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // Add a mesh loaded from a glTF file. This mesh has data for `ATTRIBUTE_BARYCENTRIC`.
    // Rust: Load specific primitive from GLTF file
    let mesh = asset_server.load(
        // Rust: Struct literal with named fields
        GltfAssetLabel::Primitive {
            mesh: 0,       // First mesh in the file
            primitive: 0,  // First primitive in that mesh
        }
        // Rust: Method call to specify the GLTF file
        .from_asset("models/barycentric/barycentric.gltf"),
    );
    // Rust: Spawn entity with multiple components
    commands.spawn((
        // Rust: 2D mesh component
        Mesh2d(mesh),
        // Rust: Add our custom material to assets and get handle
        // materials.add() returns Handle<CustomMaterial>
        MeshMaterial2d(materials.add(CustomMaterial {})),
        // Rust: Scale the mesh to make it visible
        // Vec3::ONE is shorthand for Vec3::new(1.0, 1.0, 1.0)
        // 150.0 * Vec3::ONE scales uniformly to 150x size
        Transform::from_scale(150.0 * Vec3::ONE),
    ));

    // Rust: Spawn 2D camera
    commands.spawn(Camera2d);
}

/// This custom material uses barycentric coordinates from
/// `ATTRIBUTE_BARYCENTRIC` to shade a white border around each triangle. The
/// thickness of the border is animated using the global time shader uniform.
// Rust: Multiple derive macros for custom material
#[derive(
    Asset,      // Can be stored in asset system
    TypePath,   // Required for reflection system
    AsBindGroup, // Automatically creates GPU binding layout
    Debug,      // Enables {:?} formatting
    Clone       // Can be duplicated
)]
// Rust: Empty struct - all data comes from shader uniforms
struct CustomMaterial {}

// Rust: Trait implementation for Material2d
impl Material2d for CustomMaterial {
    // Rust: Associated function returning shader reference
    fn vertex_shader() -> ShaderRef {
        // Rust: .into() converts &str to ShaderRef
        // Uses Into trait for type conversion
        SHADER_ASSET_PATH.into()
    }
    // Rust: Fragment shader path (same file, different entry point)
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    // Rust: Configure the GPU rendering pipeline
    fn specialize(
        // Rust: Mutable reference to pipeline descriptor
        descriptor: &mut RenderPipelineDescriptor,
        // Rust: Reference to vertex buffer layout
        layout: &MeshVertexBufferLayoutRef,
        // Rust: Underscore prefix indicates unused parameter
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Rust: Create vertex layout with our custom attributes
        let vertex_layout = layout.0.get_layout(&[
            // Rust: Standard position attribute at shader location 0
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            // Rust: Standard color attribute at shader location 1
            Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
            // Rust: Our custom barycentric attribute at shader location 2
            ATTRIBUTE_BARYCENTRIC.at_shader_location(2),
        ])?;  // ? operator for error propagation
        // Rust: Assign vertex buffer layout to pipeline
        // vec![] macro creates Vec with initial contents
        descriptor.vertex.buffers = vec![vertex_layout];
        // Rust: Return Ok(()) for success
        Ok(())
    }
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Custom Vertex Attributes**:
//    - MeshVertexAttribute defines custom data per vertex
//    - Unique ID prevents conflicts with other attributes
//    - VertexFormat specifies data type (Float32x3 = 3 floats)
//
// 2. **GLTF Integration**:
//    - GltfPlugin.add_custom_vertex_attribute() maps names
//    - GLTF files can contain custom vertex data
//    - Must match naming conventions ("_BARYCENTRIC")
//
// 3. **Material2d Trait**:
//    - vertex_shader() and fragment_shader() specify shaders
//    - specialize() configures GPU pipeline
//    - Custom vertex layouts for additional attributes
//
// 4. **AsBindGroup Derive**:
//    - Automatically creates GPU binding layout
//    - Handles uniform buffers and textures
//    - Required for Material2d trait
//
// 5. **Error Handling**:
//    - Result<T, E> for functions that can fail
//    - ? operator propagates errors up the call stack
//    - SpecializedMeshPipelineError for rendering errors
//
// 6. **Type Conversions**:
//    - .into() converts types via Into trait
//    - &str -> ShaderRef for shader paths
//    - Automatic conversions reduce boilerplate
//
// 7. **Shader Locations**:
//    - at_shader_location() maps attributes to shader inputs
//    - Numbers (0, 1, 2) correspond to shader layout locations
//    - Must match shader attribute declarations
//
// 8. **Barycentric Coordinates**:
//    - Each triangle vertex has [u, v, w] coordinates
//    - Sum always equals 1.0 (u + v + w = 1)
//    - Used for wireframe effects and triangle visualization
