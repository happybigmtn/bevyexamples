//! A shader and a material that uses it.
//!
//! Shaders are like custom painting instructions for your GPU. Instead of using
//! Bevy's built-in rendering, you write your own program that runs on every pixel!
//! Think of it as teaching the GPU a new art technique - you provide the brushstrokes
//! (shader code) and the paint (material data), and it creates your masterpiece.

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

/// This example uses a shader source file from the assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/custom_material.wgsl";

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // MaterialPlugin teaches Bevy how to render our custom material
            // It's like registering a new art style with the rendering engine
            MaterialPlugin::<CustomMaterial>::default()
        ))
        .add_systems(Startup, setup)
        .run();
}

/// Set up a simple 3D scene with our custom material
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // CUBE WITH CUSTOM MATERIAL
    commands.spawn((
        // The geometry - what shape to draw
        Mesh3d(meshes.add(Cuboid::default())),
        // Our custom material - how to draw it
        MeshMaterial3d(materials.add(CustomMaterial {
            // Base color - mixed with texture in shader
            color: LinearRgba::BLUE,
            // Texture to sample - the Bevy icon!
            color_texture: Some(asset_server.load("branding/icon.png")),
            // Blend mode - allows transparency
            alpha_mode: AlphaMode::Blend,
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Camera to view our creation
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// CUSTOM MATERIAL DEFINITION
// This struct defines the data that will be passed to your shader
// Think of it as a recipe card that tells the GPU what ingredients to use
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    // UNIFORM: A single value sent to all pixels
    // Like a global setting that affects the entire surface
    #[uniform(0)]  // Binding 0 in shader
    color: LinearRgba,
    
    // TEXTURE: An image to sample colors from
    #[texture(1)]  // Binding 1 in shader
    #[sampler(2)]  // Binding 2 in shader (how to sample the texture)
    color_texture: Option<Handle<Image>>,
    
    // Not sent to shader - used by Bevy's rendering system
    alpha_mode: AlphaMode,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CustomMaterial {
    // Tell Bevy which shader to use for coloring pixels
    // Fragment shaders run once per pixel on screen
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    // Configure how transparency works
    // This affects render order and blending
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
    
    // Material trait provides many other methods:
    // - vertex_shader(): Transform vertices
    // - specialize(): Create shader variants
    // - prepass_fragment_shader(): Early depth/normal pass
    // We use defaults for everything else!
}
