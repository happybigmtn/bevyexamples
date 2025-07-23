//! Shows how to render a polygonal [`Mesh`], generated from a [`Rectangle`] primitive, in a 2D scene.
//!
//! This example demonstrates the fundamental concept of 2D mesh rendering in Bevy.
//! A mesh is a collection of vertices (points in space) connected by edges to form
//! triangles. Even in 2D, we're still working with meshes - they're just flat!
//!
//! Think of it like cutting shapes out of paper: the paper (mesh) has a shape
//! (defined by vertices) and a color (material). We position it in our 2D world
//! using transforms.
//!
//! Key concepts illustrated:
//! - Creating meshes from primitive shapes
//! - Applying materials (colors) to meshes
//! - Using transforms to position and scale objects
//! - The component-based architecture of Bevy

// We're importing from Bevy's color palette system. This gives us predefined colors
// that follow good design principles. It's like having a professional color swatch!
use bevy::{color::palettes::basic::PURPLE, prelude::*};

fn main() {
    // The App is the heart of any Bevy application. Think of it as the main control
    // panel where we wire everything together.
    App::new()
        // DefaultPlugins gives us the standard Bevy setup: rendering, input, audio, etc.
        // It's like getting a fully equipped laboratory instead of building from scratch.
        .add_plugins(DefaultPlugins)
        // Systems are functions that run at specific times. Startup systems run once
        // when the app starts - perfect for setting up our scene.
        .add_systems(Startup, setup)
        // This starts the game loop. From here, Bevy takes over and runs our systems,
        // renders frames, handles input, etc. It's like pressing "play" on our game.
        .run();
}

fn setup(
    // Commands let us spawn entities and add components. Think of it as giving
    // instructions to a assistant who will carry them out for us.
    mut commands: Commands,
    // ResMut = Resource Mutable. Resources are globally accessible data.
    // Assets<Mesh> stores all our mesh data. When we add a mesh, we get back
    // a Handle - like a library card that lets us reference the mesh later.
    mut meshes: ResMut<Assets<Mesh>>,
    // Same concept but for materials. ColorMaterial is the simplest material type -
    // it just applies a solid color to our mesh.
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Every 2D scene needs a camera to define what we're looking at.
    // Camera2d sets up an orthographic camera perfect for 2D games.
    // Without this, we'd see nothing - like trying to take a photo without a camera!
    commands.spawn(Camera2d);

    // Now we spawn our rectangle! The tuple syntax here is Bevy's way of
    // spawning an entity with multiple components at once.
    commands.spawn((
        // Mesh2d is a component that says "this entity should be rendered as a 2D mesh".
        // We create the mesh from a Rectangle primitive. Rectangle::default() creates
        // a 1x1 unit rectangle centered at the origin.
        // The add() method converts the shape to triangles and stores it, returning a Handle.
        Mesh2d(meshes.add(Rectangle::default())),
        
        // MeshMaterial2d tells the renderer what material to use for this mesh.
        // We're creating a simple color material from the PURPLE constant.
        // Color::from() is needed because PURPLE is a specific color type that needs
        // conversion to Bevy's general Color type.
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
        
        // Transform defines position, rotation, and scale in the world.
        // Transform::default() starts at origin (0,0) with no rotation and scale of 1.
        // with_scale() is a builder method that returns a modified transform.
        // Vec3::splat(128.) creates Vec3(128., 128., 128.) - all dimensions get the same value.
        // This scales our 1x1 rectangle to 128x128 pixels. Why 128? It's a nice visible size
        // and a power of 2, which computers handle efficiently.
        Transform::default().with_scale(Vec3::splat(128.)),
    ));
}
