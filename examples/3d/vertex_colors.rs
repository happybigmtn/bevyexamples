//! Illustrates the use of vertex colors.
//!
//! # Vertex Colors: Per-Vertex Color Data
//!
//! Vertex colors allow you to assign colors directly to mesh vertices,
//! creating gradients and effects without textures. Each vertex can have
//! its own color, and colors are interpolated across triangles.
//!
//! ## Common Uses:
//!
//! - **Terrain coloring**: Height-based colors
//! - **Heat maps**: Data visualization
//! - **Artistic effects**: Hand-painted models
//! - **Performance**: No texture lookups needed
//! - **Procedural colors**: Based on vertex positions
//!
//! ## How It Works:
//!
//! 1. Each vertex has position AND color data
//! 2. GPU interpolates colors between vertices
//! 3. Final color = vertex color × material color
//! 4. White material shows pure vertex colors
//!
//! ## This Example Shows:
//!
//! - Extracting vertex positions from a mesh
//! - Calculating colors based on positions
//! - Adding color attributes to vertices
//! - Rendering with vertex colors

// Rust: Multiple imports from different modules
use bevy::{
    // Rust: Glob import of common types
    prelude::*, 
    // Rust: Specific type from render module
    // :: is the path separator in Rust
    render::mesh::VertexAttributeValues,  // Enum for vertex data
};

// Rust: Program entry point
fn main() {
    // Rust: App builder pattern
    App::new()
        // Rust: Add standard plugins
        .add_plugins(DefaultPlugins)
        // Rust: Register setup system
        .add_systems(Startup, setup)
        // Rust: Start game loop
        .run();
}

/// set up a simple 3D scene
// Rust: System function with resource parameters
fn setup(
    // Rust: Mutable Commands for entity spawning
    mut commands: Commands,
    // Rust: Mutable access to mesh assets
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Mutable access to material assets
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn((
        // Rust: Create plane mesh
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        // Rust: Green material for ground
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));
    
    // cube
    // Assign vertex colors based on vertex positions
    // Rust: Create mutable mesh from primitive shape
    // Mesh::from() trait conversion from Cuboid to Mesh
    let mut colorful_cube = Mesh::from(Cuboid::default());
    
    // Rust: if-let pattern with enum matching
    // Extract position data if it exists and has the right format
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        colorful_cube.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        // Rust: Type annotation for Vec with array elements
        // Each color is [r, g, b, a] as f32 values
        let colors: Vec<[f32; 4]> = positions
            // Rust: Create iterator over position references
            .iter()
            // Rust: map() transforms each element
            // Closure with array pattern destructuring
            .map(|[r, g, b]| {
                // Rust: Calculate color from position
                // * dereferences the &f32 to f32
                [
                    (1. - *r) / 2.,  // Red from X position
                    (1. - *g) / 2.,  // Green from Y position
                    (1. - *b) / 2.,  // Blue from Z position
                    1.               // Alpha (fully opaque)
                ]
            })
            // Rust: collect() consumes iterator into Vec
            .collect();
        
        // Rust: Add color attribute to mesh
        // ATTRIBUTE_COLOR is a const string for vertex colors
        colorful_cube.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    // Rust: Spawn the colorful cube
    commands.spawn((
        // Rust: Add our modified mesh with vertex colors
        Mesh3d(meshes.add(colorful_cube)),
        // This is the default color, but note that vertex colors are
        // multiplied by the base color, so you'll likely want this to be
        // white if using vertex colors.
        // Rust: White material to show pure vertex colors
        // 1. is shorthand for 1.0 (f32 literal)
        MeshMaterial3d(materials.add(Color::srgb(1., 1., 1.))),
        // Rust: Position cube above ground
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Light
    commands.spawn((
        // Rust: PointLight with shadows
        PointLight {
            // Rust: Enable shadow casting
            shadows_enabled: true,
            // Rust: Default other fields
            ..default()
        },
        // Rust: Light position and orientation
        Transform::from_xyz(4.0, 5.0, 4.0)  // Above and to the side
            .looking_at(Vec3::ZERO, Vec3::Y),  // Point at origin
    ));

    // Camera
    commands.spawn((
        // Rust: Default camera settings
        Camera3d::default(),
        // Rust: Camera viewing angle
        Transform::from_xyz(-2.0, 2.5, 5.0)  // Back and elevated
            .looking_at(Vec3::ZERO, Vec3::Y),  // Look at origin
    ));
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Enum Pattern Matching**:
//    - `if let Some(VertexAttributeValues::Float32x3(data))`
//    - Extracts data from nested enum variants
//    - Handles missing attributes gracefully
//
// 2. **Vertex Attributes**:
//    - `ATTRIBUTE_POSITION` - Vertex positions
//    - `ATTRIBUTE_COLOR` - Vertex colors
//    - `ATTRIBUTE_NORMAL` - Surface normals
//    - `ATTRIBUTE_UV_0` - Texture coordinates
//
// 3. **Iterator Combinators**:
//    - `.iter()` - Create iterator over references
//    - `.map()` - Transform each element
//    - `.collect()` - Build collection from iterator
//
// 4. **Array Destructuring**:
//    - `|[r, g, b]|` extracts array elements
//    - Names them for use in closure body
//    - Works with fixed-size arrays
//
// 5. **Type Annotations**:
//    - `Vec<[f32; 4]>` - Vector of 4-element arrays
//    - Required when compiler can't infer
//    - Helps with complex generic types
//
// 6. **Dereference Operator**:
//    - `*r` dereferences `&f32` to `f32`
//    - Needed for arithmetic operations
//    - Common with iterator references
//
// 7. **Color Multiplication**:
//    - Vertex color × Material color = Final color
//    - White material (1,1,1) preserves vertex colors
//    - Allows tinting with non-white materials
