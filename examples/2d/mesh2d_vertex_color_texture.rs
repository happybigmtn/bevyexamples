//! Shows how to render a polygonal [`Mesh`], generated from a [`Rectangle`] primitive, in a 2D scene.
//! Adds a texture and colored vertices, giving per-vertex tinting.
//!
//! This example demonstrates vertex coloring, a powerful technique where each vertex
//! of a mesh can have its own color. When rendered, the GPU automatically interpolates
//! these colors across the surface, creating smooth gradients.
//!
//! When you combine vertex colors with a texture, the colors are multiplied together:
//! - White vertices (1,1,1) leave the texture unchanged
//! - Colored vertices tint the texture (like looking through colored glass)
//! - Black vertices (0,0,0) make that part of the texture black
//!
//! This is useful for effects like:
//! - Gradients without needing gradient textures
//! - Dynamic coloring of sprites (damage flash, team colors)
//! - Atmospheric effects (fog, lighting)

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Load the Bevy logo as a texture
    // The asset_server returns a handle immediately, even before the image is loaded
    let texture_handle = asset_server.load("branding/banner.png");
    
    // Build a default quad mesh (a rectangle with 4 vertices)
    // Rectangle::default() creates a 1x1 unit rectangle centered at origin
    let mut mesh = Mesh::from(Rectangle::default());
    
    // Build vertex colors for the quad. One entry per vertex (the corners of the quad)
    // The order matters! It must match the order vertices are defined in the mesh.
    // For a default Rectangle, vertices are typically: bottom-left, bottom-right, top-right, top-left
    let vertex_colors: Vec<[f32; 4]> = vec![
        LinearRgba::RED.to_f32_array(),    // Bottom-left corner will be red
        LinearRgba::GREEN.to_f32_array(),  // Bottom-right corner will be green
        LinearRgba::BLUE.to_f32_array(),   // Top-right corner will be blue
        LinearRgba::WHITE.to_f32_array(),  // Top-left corner will be white
    ];
    
    // Insert the vertex colors as an attribute
    // ATTRIBUTE_COLOR is a predefined attribute name that shaders expect
    // The mesh now has both position data (from Rectangle) and color data
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors);

    // Store the mesh and get a handle we can use multiple times
    let mesh_handle = meshes.add(mesh);

    // Spawn a 2D camera to view our scene
    commands.spawn(Camera2d);

    // Left quad: vertex colors only (no texture)
    // This shows pure vertex color interpolation creating a gradient
    commands.spawn((
        // clone() just clones the handle (cheap), not the mesh data
        Mesh2d(mesh_handle.clone()),
        // Default material has no texture, so only vertex colors are visible
        MeshMaterial2d(materials.add(ColorMaterial::default())),
        // Position to the left and scale up to 128x128 pixels
        Transform::from_translation(Vec3::new(-96., 0., 0.)).with_scale(Vec3::splat(128.)),
    ));

    // Right quad: vertex colors AND texture
    // The texture color is multiplied by the vertex color at each pixel
    // This creates a tinted version of the texture
    commands.spawn((
        Mesh2d(mesh_handle),
        // Create a material from the texture - it will be tinted by vertex colors
        MeshMaterial2d(materials.add(texture_handle)),
        // Position to the right
        Transform::from_translation(Vec3::new(96., 0., 0.)).with_scale(Vec3::splat(128.)),
    ));
}
