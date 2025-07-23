//! This example demonstrates the built-in 3d shapes in Bevy.
//! The scene includes a patterned texture and a rotation for visualizing the normals and UVs.
//!
//! You can toggle wireframes with the space bar except on wasm. Wasm does not support
//! `POLYGON_MODE_LINE` on the gpu.
//!
//! This is like a 3D geometry museum - showcasing all the primitive shapes Bevy provides.
//! We'll see how UV mapping works (texture coordinates) and how shapes are constructed.

use std::f32::consts::PI;

#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::{
    // Bevy includes color palettes! SILVER is a predefined color constant
    color::palettes::basic::SILVER,
    prelude::*,
    render::{
        // These imports are for creating textures programmatically
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

fn main() {
    App::new()
        .add_plugins((
            // We're overriding the default image settings to use "nearest" filtering
            // This makes our debug texture look pixelated/crisp instead of blurry
            // It's like the difference between pixel art and smooth graphics
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            #[cfg(not(target_arch = "wasm32"))]
            WireframePlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // Multiple systems in Update! They run in parallel when possible
                rotate,
                #[cfg(not(target_arch = "wasm32"))]
                toggle_wireframe,
            ),
        )
        .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
/// Marker components are like tags - they have no data, just presence
/// It's like putting a "SHAPE" sticker on certain entities
#[derive(Component)]
struct Shape;

// Layout constants - think of these as the dimensions of our display cases
const SHAPES_X_EXTENT: f32 = 14.0; // How wide to spread the basic shapes
const EXTRUSION_X_EXTENT: f32 = 16.0; // How wide to spread the extruded shapes
const Z_EXTENT: f32 = 5.0; // Front-to-back spacing

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // We need images to create textures programmatically
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a special material with a UV debug texture
    // UV coordinates map 2D texture positions to 3D surface points
    // Our debug texture will show how this mapping works on each shape
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    // Gallery Row 1: Basic 3D primitives
    // Each shape demonstrates different geometric properties
    let shapes = [
        meshes.add(Cuboid::default()), // 6 faces, 8 vertices - the classic box
        meshes.add(Tetrahedron::default()), // Simplest 3D shape - 4 triangular faces
        meshes.add(Capsule3d::default()), // Cylinder with hemisphere caps
        meshes.add(Torus::default()),  // Donut shape - demonstrates complex UV mapping
        meshes.add(Cylinder::default()), // Can with flat ends
        meshes.add(Cone::default()),   // Ice cream cone shape
        meshes.add(ConicalFrustum::default()), // Truncated cone - like a lampshade
        // Two ways to make spheres:
        // 1. Icosphere - subdivided icosahedron, uniform triangle distribution
        meshes.add(Sphere::default().mesh().ico(5).unwrap()), // 5 = subdivision level
        // 2. UV sphere - latitude/longitude grid, like Earth's coordinates
        meshes.add(Sphere::default().mesh().uv(32, 18)), // 32 segments, 18 stacks
    ];

    // Gallery Row 2: Extrusions - 2D shapes extended into 3D
    // This demonstrates how 2D shapes become 3D volumes
    let extrusions = [
        // Take any 2D shape and extrude it by 1 unit in the Z direction
        meshes.add(Extrusion::new(Rectangle::default(), 1.)), // Becomes a box
        meshes.add(Extrusion::new(Capsule2d::default(), 1.)), // Becomes a rounded box
        meshes.add(Extrusion::new(Annulus::default(), 1.)),   // Ring becomes a tube
        meshes.add(Extrusion::new(Circle::default(), 1.)),    // Becomes a cylinder
        meshes.add(Extrusion::new(Ellipse::default(), 1.)),   // Becomes oval cylinder
        meshes.add(Extrusion::new(RegularPolygon::default(), 1.)), // Becomes prism
        meshes.add(Extrusion::new(Triangle2d::default(), 1.)), // Becomes triangular prism
    ];

    let num_shapes = shapes.len();

    // Spawn the basic shapes in a row
    for (i, shape) in shapes.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            // Clone the material handle - multiple entities can share the same material
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                // Same distribution math as in 2d_shapes, but now in 3D!
                -SHAPES_X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * SHAPES_X_EXTENT,
                2.0,           // Lift them up a bit
                Z_EXTENT / 2., // Back row
            )
            // Tilt 45 degrees so we can see the top and front
            // This helps visualize the UV mapping on all faces
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape, // Tag them so our rotation system can find them
        ));
    }

    let num_extrusions = extrusions.len();

    // Spawn the extruded shapes in another row
    for (i, shape) in extrusions.into_iter().enumerate() {
        commands.spawn((
            Mesh3d(shape),
            MeshMaterial3d(debug_material.clone()),
            Transform::from_xyz(
                // Slightly wider spacing for extrusions
                -EXTRUSION_X_EXTENT / 2.
                    + i as f32 / (num_extrusions - 1) as f32 * EXTRUSION_X_EXTENT,
                2.0,
                -Z_EXTENT / 2., // Front row
            )
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
            Shape,
        ));
    }

    // Bright light to see our shapes clearly
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            // Very bright! Values are in lumens - this is like a powerful stadium light
            intensity: 10_000_000.,
            range: 100.0,
            // Shadow bias prevents "shadow acne" - artifacts from precision limits
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0),
    ));

    // Ground plane - notice how we can chain builder methods on primitives
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(50.0, 50.0) // Make it big
                    .subdivisions(10), // Add detail for better lighting
            ),
        ),
        MeshMaterial3d(materials.add(Color::from(SILVER))),
        // No transform needed - defaults to origin, which is perfect for a ground
    ));

    // Camera positioned to see both rows of shapes
    commands.spawn((
        Camera3d::default(),
        // Position above and back, looking slightly down at the shapes
        Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
    ));

    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn((
        Text::new("Press space to toggle wireframes"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// Simple rotation animation to show all sides of our shapes
fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        // Rotate around Y axis (vertical) at half speed
        // This lets us see how the UV mapping wraps around each shape
        transform.rotate_y(time.delta_secs() / 2.);
    }
}

/// Creates a colorful test pattern for visualizing UV coordinates
///
/// UV coordinates are like latitude/longitude for textures:
/// U = horizontal (0 to 1 from left to right)
/// V = vertical (0 to 1 from bottom to top)
///
/// This pattern helps us see how 2D texture space maps to 3D surfaces
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    // Create a colorful palette - each 4 bytes is one RGBA pixel
    // The pattern creates distinct colored squares so we can see
    // how the texture stretches and warps on different shapes
    let mut palette: [u8; 32] = [
        255, 102, 159, 255, // Pink
        255, 159, 102, 255, // Orange
        236, 255, 102, 255, // Yellow
        121, 255, 102, 255, // Green
        102, 255, 198, 255, // Cyan
        102, 198, 255, 255, // Light Blue
        121, 102, 255, 255, // Purple
        236, 102, 255, 255, // Magenta
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        // Copy one row of pixels
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        // Rotate the palette for the next row - creates a diagonal pattern
        palette.rotate_right(4); // Shift by one color
    }

    // Create the actual Image resource that Bevy can use
    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1, // 2D texture = 1 layer
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,   // Standard color format
        RenderAssetUsages::RENDER_WORLD, // Used for rendering only
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(
    mut wireframe_config: ResMut<WireframeConfig>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }
}
