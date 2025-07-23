//! # GPU Automatic Instancing: Rendering Thousands of Objects Efficiently
//!
//! This example demonstrates **automatic instancing** - a critical GPU optimization technique
//! that allows rendering thousands of identical objects in a single draw call. Instead of 
//! submitting 65,536 separate render commands (one per cube), the GPU processes all cubes
//! together, dramatically improving performance.
//!
//! ## Key GPU Programming Concepts:
//! - **Instancing**: Rendering multiple copies of the same mesh with different transforms/data
//! - **Instance Index**: Built-in shader variable that identifies which instance is being processed
//! - **Draw Call Batching**: Combining multiple objects into single GPU commands
//! - **Vertex Shader Data Lookup**: Using instance indices to fetch per-instance data
//! - **GPU Parallelism**: Thousands of vertices processed simultaneously across GPU cores
//!
//! ## How Automatic Instancing Works:
//! 1. **CPU Side**: Multiple entities share the same mesh handle and material handle
//! 2. **Bevy's Render System**: Automatically detects identical mesh/material combinations
//! 3. **GPU Side**: Single draw call processes all instances, using `instance_index` to differentiate
//! 4. **Per-Instance Data**: Each instance can have unique transforms, tags, and other properties
//!
//! ## Performance Impact:
//! - **Without Instancing**: 65,536 draw calls = significant CPU/GPU overhead
//! - **With Instancing**: 1 draw call = minimal overhead, maximum throughput
//! - Use graphics profilers (RenderDoc, PIX, etc.) to verify single draw call execution

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        // MeshTag: Optional component that associates arbitrary data with mesh instances
        // This allows shaders to access per-instance data beyond just transforms
        mesh::MeshTag,
        render_resource::{AsBindGroup, ShaderRef},
    },
};

// Custom shader that demonstrates vertex-stage instance data lookup and coloring
const SHADER_ASSET_PATH: &str = "shaders/automatic_instancing.wgsl";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins, 
            // Register our custom material with instancing support
            // The MaterialPlugin automatically handles instance data upload to GPU
            MaterialPlugin::<CustomMaterial>::default()
        ))
        .add_systems(Startup, setup)
        // Update system demonstrates that instanced objects can still be individually animated
        .add_systems(Update, update)
        .run();
}

/// Creates a massive grid of cubes (65,536 total) demonstrating GPU instancing efficiency.
/// Each cube is colored by sampling a corresponding pixel from an image texture, creating
/// a pixelated 3D representation of the 2D image.
///
/// ## CPU-GPU Data Flow:
/// 1. **CPU**: Creates entities with MeshTag containing pixel indices (0-65535)
/// 2. **Bevy Render System**: Automatically batches identical mesh/material combinations
/// 3. **GPU Upload**: Instance data (transforms, tags) uploaded to GPU buffers
/// 4. **Vertex Shader**: Uses instance_index to look up per-instance tag and sample texture
/// 5. **Fragment Shader**: Outputs the color sampled in the vertex stage
///
/// This demonstrates how **MeshTag enables custom per-instance data** beyond transforms.
fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // Load texture that will be sampled in the vertex shader for per-instance coloring
    // Each cube will sample the pixel at its corresponding position in this image
    let image = assets.load("branding/icon.png");

    // CRITICAL FOR INSTANCING: Single mesh handle shared by all instances
    // The GPU recognizes identical mesh handles and automatically batches them
    let mesh_handle = meshes.add(Cuboid::from_size(Vec3::splat(0.01)));

    // CRITICAL FOR INSTANCING: Single material handle shared by all instances  
    // Automatic instancing requires both mesh AND material handles to be identical
    // This custom material demonstrates how to access MeshTag data in shaders
    let material_handle = materials.add(CustomMaterial {
        image: image.clone(),
    });

    // Hardcoded dimensions matching the loaded image (256x256 = 65,536 total pixels)
    // Each pixel will become one cube instance, creating a 3D voxelization of the image
    let image_dims = UVec2::new(256, 256);
    let total_pixels = image_dims.x * image_dims.y;

    // Create 65,536 cube instances in a grid layout matching the image dimensions
    for index in 0..total_pixels {
        // Convert 1D pixel index to 2D texture coordinates
        // Modulo gives us the column (x), division gives us the row (y)
        let x = index % image_dims.x;
        let y = index / image_dims.x;

        // Transform texture coordinates to centered world space
        // Divide by 50.0 to scale down the massive grid to visible size
        let world_x = (x as f32 - image_dims.x as f32 / 2.0) / 50.0;
        let world_y = -((y as f32 - image_dims.y as f32 / 2.0) / 50.0); // Negative for correct orientation

        commands.spawn((
            // INSTANCING REQUIREMENTS: Identical handles enable automatic batching
            // All 65,536 entities share these same two handles
            Mesh3d(mesh_handle.clone()),
            MeshMaterial3d(material_handle.clone()),
            
            // MeshTag: Per-instance data accessible in shaders via mesh_functions::get_tag()
            // This index will be used to determine which pixel to sample from the texture
            MeshTag(index),
            
            // Each instance has a unique transform (position in the grid)
            Transform::from_xyz(world_x, world_y, 0.0),
        ));
    }

    // Position camera to view the massive cube grid from a distance
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Animates all 65,536 cubes individually while maintaining instancing efficiency.
/// This demonstrates that instanced objects can still have dynamic, per-instance properties.
/// 
/// ## Performance Note:
/// Even though we're animating 65,536 individual transforms, the rendering remains
/// a single draw call because the mesh and material handles are still identical.
/// The GPU efficiently processes different transforms for each instance.
fn update(time: Res<Time>, mut transforms: Query<(&mut Transform, &MeshTag)>) {
    for (mut transform, index) in transforms.iter_mut() {
        // Create a wave/spiral animation using each cube's index as a phase offset
        // This creates a rippling effect across the entire grid
        // ops::sin provides a smooth oscillation between -1.0 and 1.0
        transform.translation.z = ops::sin(time.elapsed_secs() + index.0 as f32 * 0.01);
    }
}

/// Custom material that provides texture data to the instancing shader.
/// The shader will sample from this texture using each instance's MeshTag as a pixel index.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    // Bind texture to shader binding group 2, slot 0
    // This texture contains the pixel data that each cube instance will sample
    #[texture(0)]
    #[sampler(1)]
    image: Handle<Image>,
}

impl Material for CustomMaterial {
    // Override vertex shader to implement custom instancing logic
    // The vertex shader uses instance_index to look up MeshTag data and sample textures
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    // Override fragment shader to output the color computed in vertex stage
    // Simple pass-through shader that displays the per-instance color
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
