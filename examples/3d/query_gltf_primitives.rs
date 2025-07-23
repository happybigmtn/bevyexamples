//! This example demonstrates how to query a [`StandardMaterial`] within a glTF scene.
//! It is particularly useful for glTF scenes with a mesh that consists of multiple primitives.
//!
//! # What are glTF Primitives?
//!
//! In glTF (GL Transmission Format), a single mesh can contain multiple "primitives".
//! Each primitive is essentially a sub-mesh with its own material. Think of it like:
//! - A car model where wheels, body, and windows are separate primitives
//! - Each primitive can have different materials (rubber, metal, glass)
//! - All primitives together form the complete mesh
//!
//! ## This Example Shows:
//! - Loading a glTF scene with multiple primitives
//! - Finding specific materials by name (using GltfMaterialName)
//! - Modifying materials at runtime (color animation)
//! - Directly manipulating mesh vertex data (position animation)
//!
//! ## Real-World Use Cases:
//! - Highlighting parts of a model (e.g., selected car parts)
//! - Animating specific mesh components
//! - Creating material variations without duplicating models
//! - Interactive 3D configurators

// Rust: Importing from the standard library
// `std::f32::consts` contains mathematical constants
// PI is approximately 3.14159... (ratio of circle's circumference to diameter)
use std::f32::consts::PI;

// Rust: Importing from external crates with path syntax
use bevy::{
    // Rust: Nested module path - gltf is a submodule of bevy
    // GltfMaterialName is a component that stores material names from glTF files
    gltf::GltfMaterialName, 
    // Rust: Glob import with * - brings all public items into scope
    // Includes common types like App, Commands, Query, etc.
    prelude::*, 
    // Rust: Deep module path with :: separators
    // VertexAttributeValues is an enum for different vertex data formats
    render::mesh::VertexAttributeValues
};

// Rust: Entry point function - every executable needs exactly one
// No parameters, no explicit return type (returns unit type ())
fn main() {
    // Rust: Method chaining pattern
    // Each method returns Self allowing .method1().method2() chains
    App::new()
        // Rust: Method that takes a value implementing a trait
        // DefaultPlugins implements the PluginGroup trait
        .add_plugins(DefaultPlugins)
        // Rust: Passing function pointers as arguments
        // Functions are first-class values in Rust
        .add_systems(Startup, setup)
        .add_systems(Update, find_top_material_and_mesh)
        // Rust: Consumes the App and starts the game loop
        // This never returns - runs until window closed
        .run();
}

// Rust: System function with multiple parameters
// Bevy automatically provides these based on type signatures
fn find_top_material_and_mesh(
    // Rust: ResMut<T> - mutable access to a resource
    // Assets<T> is a collection of assets indexed by handles
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // Rust: Res<T> - immutable access to a resource
    // Time provides frame timing information
    time: Res<Time>,
    // Rust: Query<T> - efficient way to iterate entities with components
    // Tuple syntax (A, B, C) queries entities that have ALL listed components
    mat_query: Query<(
        // Rust: &T means immutable reference/borrow
        // MeshMaterial3d<T> is a component containing a Handle<T>
        &MeshMaterial3d<StandardMaterial>,
        // Rust: Mesh3d contains Handle<Mesh>
        &Mesh3d,
        // Rust: Component added by glTF loader with material names
        &GltfMaterialName,
    )>,
) {
    // Rust: Iterator pattern with tuple destructuring
    // .iter() returns an iterator over query results
    for (mat_handle, mesh_handle, name) in mat_query.iter() {
        // Rust: String comparison
        // name.0 accesses the first field of the tuple struct GltfMaterialName
        // "Top" is a &str (string slice) literal
        if name.0 == "Top" {
            // Rust: if let pattern - combines pattern matching with conditional
            // Only executes block if pattern matches (Some variant)
            if let Some(material) = materials.get_mut(mat_handle) {
                // Rust: Nested pattern matching on enum variants
                // ref mut creates a mutable reference to the matched value
                // This avoids moving the value out of the enum
                if let Color::Hsla(ref mut hsla) = material.base_color {
                    // Rust: Dereferencing and method call
                    // *hsla dereferences the mutable reference
                    // rotate_hue returns a new Hsla with rotated hue
                    *hsla = hsla.rotate_hue(time.delta_secs() * 100.0);
                } else {
                    // Rust: Type conversion with From trait
                    // Color::from(Hsla) converts Hsla to Color enum
                    // Hsla::hsl creates HSL color (Hue, Saturation, Lightness)
                    material.base_color = Color::from(Hsla::hsl(0.0, 0.9, 0.7));
                }
            }

            // Rust: Another if let pattern for optional mesh access
            if let Some(mesh) = meshes.get_mut(mesh_handle) {
                // Rust: Complex nested pattern matching
                // Matches specific enum variant with data extraction
                if let Some(VertexAttributeValues::Float32x3(positions)) =
                    // Rust: Method call returning Option<&mut VertexAttributeValues>
                    // Mesh::ATTRIBUTE_POSITION is an associated constant
                    mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
                {
                    // Rust: Iterating over mutable slice
                    // positions is &mut Vec<[f32; 3]>
                    for position in positions {
                        // Rust: Tuple creation and conversion
                        // Creates tuple then converts to [f32; 3] array
                        *position = (
                            position[0],  // X remains unchanged
                            // Rust: Arithmetic expression
                            // Creates wave motion on Y axis
                            1.5 + 0.5 * ops::sin(time.elapsed_secs() / 2.0),
                            position[2],  // Z remains unchanged
                        )
                            // Rust: .into() calls Into trait implementation
                            // Converts (f32, f32, f32) tuple to [f32; 3] array
                            .into();
                    }
                }
            }
        }
    }
}

// Rust: Function with mutable and immutable parameters
// Setup systems run once at startup
fn setup(
    // Rust: Mutable binding for Commands
    // Commands queues entity/component operations
    mut commands: Commands, 
    // Rust: Immutable access to AssetServer resource
    // AssetServer loads assets from files
    asset_server: Res<AssetServer>
) {
    // Rust: Spawning entity with tuple of components
    commands.spawn((
        // Rust: Default trait implementation
        // Creates camera with default settings
        Camera3d::default(),
        // Rust: Builder pattern with method chaining
        // Transform describes position, rotation, scale
        Transform::from_xyz(4.0, 4.0, 12.0)
            // Rust: Method with multiple parameters
            // looking_at sets rotation to face target point
            .looking_at(
                // Rust: Associated function on Vec3
                // Creates 3D vector from components
                Vec3::new(0.0, 0.0, 0.5), 
                // Rust: Associated constant
                // Vec3::Y is (0, 1, 0) - the up direction
                Vec3::Y
            ),
    ));

    // Rust: Spawning light entity
    commands.spawn((
        // Rust: Complex transform creation
        // Quat represents rotation as quaternion
        Transform::from_rotation(
            // Rust: Quaternion from Euler angles
            // EulerRot::ZYX specifies rotation order
            Quat::from_euler(
                EulerRot::ZYX,  // Rotation order: Z, then Y, then X
                0.0,            // Z rotation in radians
                1.0,            // Y rotation in radians  
                -PI / 4.        // X rotation: -45 degrees
            )
        ),
        // Rust: Component with default values
        DirectionalLight::default(),
    ));

    // Rust: Loading a glTF scene
    commands.spawn(
        // Rust: Newtype pattern - SceneRoot wraps Handle<Scene>
        SceneRoot(
            // Rust: Asset loading with label
            asset_server.load(
                // Rust: Complex type construction
                // GltfAssetLabel specifies which part of glTF to load
                GltfAssetLabel::Scene(0)  // Load scene at index 0
                    // Rust: Builder method to specify the asset path
                    // Creates full asset path with label
                    .from_asset("models/GltfPrimitives/gltf_primitives.glb"),
            )
        )
    );
}

// 🎯 Key Rust Concepts in This Example:
//
// 1. **Pattern Matching (`if let`)**:
//    - Safely handles Option types
//    - Extracts values from enums
//    - More idiomatic than .is_some() + .unwrap()
//
// 2. **Reference Types**:
//    - `&T`: Immutable borrow (read-only)
//    - `&mut T`: Mutable borrow (read-write)
//    - `ref mut`: Create mutable reference in pattern
//
// 3. **Smart Pointers**:
//    - `Res<T>`: Read-only access to resources
//    - `ResMut<T>`: Read-write access to resources
//    - Automatically dereference with . operator
//
// 4. **Type Conversions**:
//    - `.into()`: Uses Into trait (caller decides type)
//    - `Type::from()`: Uses From trait (explicit type)
//    - `as`: Primitive type casting
//
// 5. **Iterators**:
//    - `.iter()`: Iterate by reference
//    - `.iter_mut()`: Iterate by mutable reference
//    - `for x in iter`: Consumes iterator
//
// 6. **Tuple Structs**:
//    - `GltfMaterialName(String)`: Newtype pattern
//    - Access with .0, .1, etc.
//    - Provides type safety for primitives
//
// 7. **Builder Pattern**:
//    - Method chaining with self-returning methods
//    - Common in Rust for ergonomic APIs
//    - Each method configures and returns self
