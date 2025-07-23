//! This example demonstrates how to create a custom mesh,
//! assign a custom UV mapping for a custom texture,
//! and how to change the UV mapping at run-time.
//!
//! # What You'll Learn
//!
//! 1. **Mesh Construction**: How to build a 3D mesh from scratch by defining:
//!    - Vertex positions (the corners of your shape)
//!    - UV coordinates (how textures map onto surfaces)
//!    - Normals (which direction surfaces face, for lighting)
//!    - Indices (which vertices form triangles)
//!
//! 2. **UV Mapping**: How to control which part of a texture appears on each face
//!    of your mesh. UV coordinates are 2D coordinates that map points on a 3D
//!    surface to points on a 2D texture.
//!
//! 3. **Dynamic Mesh Modification**: How to change mesh attributes at runtime,
//!    allowing for effects like texture swapping or animation.
//!
//! This example creates a cube that can switch between two different textures
//! (dirt+grass vs sand+water) by modifying its UV coordinates.

use bevy::{
    prelude::*,
    render::{
        mesh::{
            Indices,               // Defines which vertices form triangles
            VertexAttributeValues, // Enum for different vertex attribute types
        },
        // Controls where mesh data is stored (CPU and/or GPU)
        render_asset::RenderAssetUsages,
        // Defines how vertices connect (triangles, lines, points, etc.)
        render_resource::PrimitiveTopology,
    },
};

// Define a "marker" component to mark the custom mesh. Marker components are often used in Bevy for
// filtering entities in queries with `With`, they're usually not queried directly since they don't
// contain information within them.
//
// This is a zero-sized type (ZST) - it takes up no memory but lets us identify
// which entities have our custom mesh. It's like a tag or label.
#[derive(Component)]
struct CustomUV;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Create the mesh and scene at startup
        .add_systems(Startup, setup)
        // Handle keyboard input every frame
        .add_systems(Update, input_handler)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Import the custom texture.
    // This texture has two sections:
    // - Top half: dirt (bottom) transitioning to grass (top)
    // - Bottom half: sand (bottom) transitioning to water (top)
    let custom_texture_handle: Handle<Image> = asset_server.load("textures/array_texture.png");
    
    // Create and save a handle to the mesh.
    // The handle is like a reference or ID that points to the mesh data
    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_cube_mesh());

    // Render the mesh with the custom texture, and add the marker.
    commands.spawn((
        // Mesh3d component references our custom mesh
        Mesh3d(cube_mesh_handle),
        // StandardMaterial with our texture applied
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(custom_texture_handle),
            ..default()
        })),
        // Our marker component to identify this entity
        CustomUV,
    ));

    // Transform for the camera and lighting, looking at (0,0,0) (the position of the mesh).
    // Position at (1.8, 1.8, 1.8) gives us a nice 3/4 view of the cube
    let camera_and_light_transform =
        Transform::from_xyz(1.8, 1.8, 1.8).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera in 3D space.
    commands.spawn((Camera3d::default(), camera_and_light_transform));

    // Light up the scene.
    // Placing the light at the same position as the camera creates
    // a "headlight" effect where shadows aren't visible
    commands.spawn((PointLight::default(), camera_and_light_transform));

    // Text to describe the controls.
    commands.spawn((
        Text::new("Controls:\nSpace: Change UVs\nX/Y/Z: Rotate\nR: Reset orientation"),
        // Position the text in the top-left corner
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// System to receive input from the user,
// check out examples/input/ for more examples about user input.
fn input_handler(
    // Current state of keyboard keys
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Query to get the mesh handle from our entity
    mesh_query: Query<&Mesh3d, With<CustomUV>>,
    // Access to mesh assets so we can modify them
    mut meshes: ResMut<Assets<Mesh>>,
    // Query to get transform for rotation
    mut query: Query<&mut Transform, With<CustomUV>>,
    // Time resource for frame-independent rotation speed
    time: Res<Time>,
) {
    // Toggle texture when space is pressed
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Get the mesh handle from our entity
        let mesh_handle = mesh_query.single().expect("Query not successful");
        // Get mutable access to the actual mesh data
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        // Modify the UV coordinates to show different part of texture
        toggle_texture(mesh);
    }
    // Rotate around X axis while X is held
    if keyboard_input.pressed(KeyCode::KeyX) {
        for mut transform in &mut query {
            // Rotate by delta_secs to ensure consistent speed regardless of framerate
            // Dividing by 1.2 slows it down for comfortable viewing
            transform.rotate_x(time.delta_secs() / 1.2);
        }
    }
    // Rotate around Y axis while Y is held
    if keyboard_input.pressed(KeyCode::KeyY) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_secs() / 1.2);
        }
    }
    // Rotate around Z axis while Z is held
    if keyboard_input.pressed(KeyCode::KeyZ) {
        for mut transform in &mut query {
            transform.rotate_z(time.delta_secs() / 1.2);
        }
    }
    // Reset orientation when R is pressed
    if keyboard_input.pressed(KeyCode::KeyR) {
        for mut transform in &mut query {
            // Reset to looking forward (-Z is forward in Bevy's coordinate system)
            transform.look_to(Vec3::NEG_Z, Vec3::Y);
        }
    }
}

#[rustfmt::skip] // Disable auto-formatting to keep our manual layout
fn create_cube_mesh() -> Mesh {
    // Create a new mesh with:
    // - TriangleList: Every 3 vertices form a triangle
    // - RenderAssetUsages flags:
    //   - MAIN_WORLD: Keep mesh data on CPU (needed for runtime modification)
    //   - RENDER_WORLD: Also send to GPU for rendering
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        // Each array is an [x, y, z] coordinate in local space.
        // Bevy uses a right-handed coordinate system:
        // - X: Right (positive) / Left (negative)
        // - Y: Up (positive) / Down (negative)
        // - Z: Back (positive) / Forward (negative) - Note: "forward" is -Z!
        // 
        // We center the cube at origin (0,0,0) so rotations look natural.
        // Each face needs 4 vertices (corners), total 24 vertices for 6 faces.
        vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0 - top-left-front
            [0.5, 0.5, -0.5],  // vertex with index 1 - top-right-front
            [0.5, 0.5, 0.5],   // vertex with index 2 - top-right-back
            [-0.5, 0.5, 0.5],  // vertex with index 3 - top-left-back
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2,
                             // but needs different UV coords and normal for the right face
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ],
    )
    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense.
    //
    // UV coordinates work like this:
    // - U (horizontal): 0.0 = left edge, 1.0 = right edge
    // - V (vertical): 0.0 = top edge, 1.0 = bottom edge
    // 
    // Our texture has two horizontal bands:
    // - V = 0.0 to 0.5: dirt+grass texture
    // - V = 0.5 to 1.0: sand+water texture
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![
            // Assigning the UV coords for the top side.
            // Using V from 0.0 to 0.2 shows the "grass" part of the texture
            [0.0, 0.2], [0.0, 0.0], [1.0, 0.0], [1.0, 0.2],
            // Assigning the UV coords for the bottom side.
            // Using V from 0.25 to 0.45 shows the "dirt" part of the texture
            [0.0, 0.45], [0.0, 0.25], [1.0, 0.25], [1.0, 0.45],
            // Assigning the UV coords for the right side.
            [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
            // Assigning the UV coords for the left side.
            [1.0, 0.45], [0.0, 0.45], [0.0, 0.2], [1.0, 0.2],
            // Assigning the UV coords for the back side.
            [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
            // Assigning the UV coords for the forward side.
            [0.0, 0.45], [0.0, 0.2], [1.0, 0.2], [1.0, 0.45],
        ],
    )
    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    //
    // Normals determine:
    // - How light reflects off the surface
    // - Which direction the surface "faces"
    // - Whether a surface is visible (back-face culling)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ],
    )
    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    //
    // Why counter-clockwise? This is called the "winding order" and determines:
    // - Which side of the triangle is the front (visible) side
    // - Which side gets culled (not rendered) for performance
    //
    // Indices let us reuse vertex data efficiently. Instead of duplicating vertices,
    // we just reference them by their position in the vertex arrays.
    //
    // The first two defined triangles look like this (marked with the vertex indices,
    // and the axis), when looking down at the top (+y) of the cube:
    //   -Z
    //   ^
    // 0---1
    // |  /|
    // | / | -> +X
    // |/  |
    // 3---2
    //
    // The right face's (+x) triangles look like this, seen from the outside of the cube.
    //   +Y
    //   ^
    // 10--11
    // |  /|
    // | / | -> -Z
    // |/  |
    // 9---8
    //
    // The back face's (+z) triangles look like this, seen from the outside of the cube.
    //   +Y
    //   ^
    // 17--18
    // |\  |
    // | \ | -> +X
    // |  \|
    // 16--19
    .with_inserted_indices(Indices::U32(vec![
        0,3,1 , 1,3,2,       // triangles making up the top (+y) facing side
        4,5,7 , 5,6,7,       // bottom (-y) - note the different winding order!
        8,11,9 , 9,11,10,    // right (+x)
        12,13,15 , 13,14,15, // left (-x)
        16,19,17 , 17,19,18, // back (+z)
        20,21,23 , 21,22,23, // forward (-z)
    ]))
}

// Function that changes the UV mapping of the mesh, to apply the other texture.
// This demonstrates runtime mesh modification - a powerful feature for dynamic effects.
fn toggle_texture(mesh_to_change: &mut Mesh) {
    // Get a mutable reference to the values of the UV attribute, so we can iterate over it.
    let uv_attribute = mesh_to_change.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap();
    
    // UV coordinates are stored as Float32x2 (two f32 values: U and V)
    // We need to pattern match to ensure we have the right type
    let VertexAttributeValues::Float32x2(uv_attribute) = uv_attribute else {
        panic!("Unexpected vertex format, expected Float32x2.");
    };

    // Iterate over the UV coordinates, and change them as we want.
    for uv_coord in uv_attribute.iter_mut() {
        // uv_coord[0] is U (horizontal), uv_coord[1] is V (vertical)
        
        // If the UV coordinate points to the upper, "dirt+grass" part of the texture...
        if (uv_coord[1] + 0.5) < 1.0 {
            // ... point to the equivalent lower, "sand+water" part instead,
            // by shifting V coordinate down by 0.5
            uv_coord[1] += 0.5;
        } else {
            // else, point back to the upper, "dirt+grass" part
            // by shifting V coordinate up by 0.5
            uv_coord[1] -= 0.5;
        }
    }
}
