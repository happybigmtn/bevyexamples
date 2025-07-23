//! A simple 3D scene with a spinning cube with a normal map and depth map to demonstrate parallax mapping.
//! Press left mouse button to cycle through different views.
//!
//! # What is Parallax Mapping?
//!
//! Parallax mapping is a technique that creates the illusion of depth on flat surfaces.
//! It makes textures appear to have real 3D geometry without adding actual polygons!
//!
//! ## How It Works
//!
//! 1. **Height/Depth Map**: A grayscale texture where brightness = height
//!    - White pixels are "low" (recessed)
//!    - Black pixels are "high" (raised)
//!
//! 2. **View-Dependent Offset**: As you move the camera, the texture shifts
//!    - Creates illusion of depth and self-occlusion
//!    - Raised parts appear to stick out and hide lower parts
//!
//! ## Parallax Mapping Methods
//!
//! 1. **Parallax Occlusion Mapping**: Basic method, fast but less accurate
//! 2. **Relief Mapping**: More accurate, traces rays through the height field
//!
//! ## Real-World Uses
//!
//! - **Brick walls**: Individual bricks appear to stick out
//! - **Cobblestones**: Each stone looks rounded and separate
//! - **Terrain details**: Rocks, cracks, and bumps without extra geometry
//! - **Sci-fi panels**: Buttons and details that look truly 3D
//!
//! This example lets you adjust the effect strength and switch between methods!

use std::fmt;

use bevy::{
    image::ImageLoaderSettings, // For loading images with specific settings
    math::ops,                  // Mathematical operations
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                spin,
                move_camera,
                update_parallax_depth_scale,
                update_parallax_layers,
                switch_method,
            ),
        )
        .run();
}

#[derive(Component)]
struct Spin {
    speed: f32, // Rotation speed in radians per second
}

/// The camera, used to move camera on click.
#[derive(Component)]
struct CameraController;

// Constants for parallax depth adjustment
const DEPTH_CHANGE_RATE: f32 = 0.1;   // Smooth transition speed
const DEPTH_UPDATE_STEP: f32 = 0.03;  // How much depth changes per key press
const MAX_DEPTH: f32 = 0.3;           // Maximum depth to prevent artifacts

// Wrapper types for Local<T> system parameters
// These maintain state between system runs

struct TargetDepth(f32);
impl Default for TargetDepth {
    fn default() -> Self {
        TargetDepth(0.09) // Good starting depth for the effect
    }
}

struct TargetLayers(f32);
impl Default for TargetLayers {
    fn default() -> Self {
        TargetLayers(5.0) // 2^5 = 32 layers by default
    }
}

struct CurrentMethod(ParallaxMappingMethod);
impl Default for CurrentMethod {
    fn default() -> Self {
        // Start with Relief mapping, 4 steps - good balance of quality/performance
        CurrentMethod(ParallaxMappingMethod::Relief { max_steps: 4 })
    }
}

// Display implementation for showing method name in UI
impl fmt::Display for CurrentMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ParallaxMappingMethod::Occlusion => write!(f, "Parallax Occlusion Mapping"),
            ParallaxMappingMethod::Relief { max_steps } => {
                write!(f, "Relief Mapping with {max_steps} steps")
            }
        }
    }
}

impl CurrentMethod {
    fn next_method(&mut self) {
        use ParallaxMappingMethod::*;
        // Cycle through different methods and step counts
        self.0 = match self.0 {
            Occlusion => Relief { max_steps: 2 },                     // Fast but rough
            Relief { max_steps } if max_steps < 3 => Relief { max_steps: 4 },  // Balanced
            Relief { max_steps } if max_steps < 5 => Relief { max_steps: 8 },  // High quality
            Relief { .. } => Occlusion,                               // Back to start
        }
    }
}

/// Adjusts the parallax depth scale - how "deep" the effect appears
fn update_parallax_depth_scale(
    input: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut target_depth: Local<TargetDepth>,
    mut depth_update: Local<bool>, // Track if we're animating depth changes
    mut writer: TextUiWriter,
    text: Single<Entity, With<Text>>,
) {
    // Handle depth decrease (1 key)
    if input.just_pressed(KeyCode::Digit1) {
        target_depth.0 -= DEPTH_UPDATE_STEP;
        target_depth.0 = target_depth.0.max(0.0); // Clamp to minimum
        *depth_update = true;
    }
    // Handle depth increase (2 key)
    if input.just_pressed(KeyCode::Digit2) {
        target_depth.0 += DEPTH_UPDATE_STEP;
        target_depth.0 = target_depth.0.min(MAX_DEPTH); // Clamp to maximum
        *depth_update = true;
    }
    
    // Smoothly animate depth changes
    if *depth_update {
        for (_, mat) in materials.iter_mut() {
            let current_depth = mat.parallax_depth_scale;
            // Lerp (linear interpolation) for smooth transitions
            let new_depth = current_depth.lerp(target_depth.0, DEPTH_CHANGE_RATE);
            mat.parallax_depth_scale = new_depth;
            
            // Update UI text
            *writer.text(*text, 1) = format!("Parallax depth scale: {new_depth:.5}\n");
            
            // Stop animating when close enough to target
            if (new_depth - current_depth).abs() <= 0.000000001 {
                *depth_update = false;
            }
        }
    }
}

/// Switches between different parallax mapping algorithms
fn switch_method(
    input: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
    mut current: Local<CurrentMethod>,
) {
    if input.just_pressed(KeyCode::Space) {
        current.next_method();
    } else {
        return;
    }
    
    // Update UI to show current method
    let text_entity = *text;
    *writer.text(text_entity, 3) = format!("Method: {}\n", *current);

    // Apply new method to all materials
    for (_, mat) in materials.iter_mut() {
        mat.parallax_mapping_method = current.0;
    }
}

/// Updates the number of layers used in parallax calculations
/// More layers = better quality but worse performance
fn update_parallax_layers(
    input: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut target_layers: Local<TargetLayers>,
    text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
) {
    if input.just_pressed(KeyCode::Digit3) {
        target_layers.0 -= 1.0;
        target_layers.0 = target_layers.0.max(0.0); // Minimum 2^0 = 1 layer
    } else if input.just_pressed(KeyCode::Digit4) {
        target_layers.0 += 1.0;
    } else {
        return;
    }
    
    // Convert from exponent to actual layer count (2^n)
    // e.g., target_layers = 5 means 2^5 = 32 layers
    let layer_count = ops::exp2(target_layers.0);
    
    // Update UI
    let text_entity = *text;
    *writer.text(text_entity, 2) = format!("Layers: {layer_count:.0}\n");

    // Apply to all materials
    for (_, mat) in materials.iter_mut() {
        mat.max_parallax_layer_count = layer_count;
    }
}

/// Continuously rotates objects with the Spin component
fn spin(time: Res<Time>, mut query: Query<(&mut Transform, &Spin)>) {
    for (mut transform, spin) in query.iter_mut() {
        // Rotate on all axes for interesting motion
        transform.rotate_local_y(spin.speed * time.delta_secs());
        transform.rotate_local_x(spin.speed * time.delta_secs());
        transform.rotate_local_z(-spin.speed * time.delta_secs());
    }
}

// Camera positions to cycle through when left-clicking.
// Each position shows the parallax effect from different angles
const CAMERA_POSITIONS: &[Transform] = &[
    // Close-up diagonal view - good for seeing depth details
    Transform {
        translation: Vec3::new(1.5, 1.5, 1.5),
        rotation: Quat::from_xyzw(-0.279, 0.364, 0.115, 0.880),
        scale: Vec3::ONE,
    },
    // Side view - shows how texture shifts with viewing angle
    Transform {
        translation: Vec3::new(2.4, 0.0, 0.2),
        rotation: Quat::from_xyzw(0.094, 0.676, 0.116, 0.721),
        scale: Vec3::ONE,
    },
    // Far view - see the overall effect
    Transform {
        translation: Vec3::new(2.4, 2.6, -4.3),
        rotation: Quat::from_xyzw(0.170, 0.908, 0.308, 0.225),
        scale: Vec3::ONE,
    },
    // Another angle - demonstrates view-dependent nature
    Transform {
        translation: Vec3::new(-1.0, 0.8, -1.2),
        rotation: Quat::from_xyzw(-0.004, 0.909, 0.247, -0.335),
        scale: Vec3::ONE,
    },
];

/// Smoothly moves camera between preset positions
fn move_camera(
    mut camera: Single<&mut Transform, With<CameraController>>,
    mut current_view: Local<usize>,
    button: Res<ButtonInput<MouseButton>>,
) {
    if button.just_pressed(MouseButton::Left) {
        // Cycle to next camera position
        *current_view = (*current_view + 1) % CAMERA_POSITIONS.len();
    }
    
    let target = CAMERA_POSITIONS[*current_view];
    // Smooth interpolation to target position
    camera.translation = camera.translation.lerp(target.translation, 0.2);
    // Spherical interpolation for smooth rotation
    camera.rotation = camera.rotation.slerp(target.rotation, 0.2);
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    // Load the normal map with special settings
    // Normal maps store surface direction data, not colors!
    // 
    // To create normal maps:
    // 1. Start with a height/depth map (grayscale image)
    // 2. Use image editor (GIMP: Filters → Generic → Normal Map)
    // 3. Enable "flip X" for correct handedness
    let normal_handle = asset_server.load_with_settings(
        "textures/parallax_example/cube_normal.png",
        // IMPORTANT: Normal maps must be in linear color space!
        // is_srgb = false prevents gamma correction that would break lighting
        |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
    );

    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.5, 1.5, 1.5).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController, // Allows camera movement on click
    ));

    // Light with visual representation
    commands
        .spawn((
            PointLight {
                shadows_enabled: true, // Shadows enhance the 3D effect
                ..default()
            },
            Transform::from_xyz(2.0, 1.0, -1.1),
        ))
        .with_children(|commands| {
            // Small white sphere to show light position
            let mesh = meshes.add(Sphere::new(0.05).mesh().ico(3).unwrap());
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(materials.add(Color::WHITE))));
        });

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            // Dark green ground with specific material properties
            perceptual_roughness: 0.45, // Somewhat rough surface
            reflectance: 0.18,          // Low reflectance
            ..Color::srgb_u8(0, 80, 0).into() // Dark green base
        })),
        Transform::from_xyz(0.0, -1.0, 0.0),
    ));

    // Set up initial parallax parameters
    let parallax_depth_scale = TargetDepth::default().0;
    let max_parallax_layer_count = ops::exp2(TargetLayers::default().0);
    let parallax_mapping_method = CurrentMethod::default();
    
    // Create the parallax material with all required textures
    let parallax_material = materials.add(StandardMaterial {
        perceptual_roughness: 0.4,
        // Base color texture - what you see
        base_color_texture: Some(asset_server.load("textures/parallax_example/cube_color.png")),
        // Normal map - surface detail directions
        normal_map_texture: Some(normal_handle),
        // Depth map - the key to parallax effect!
        // Grayscale texture where:
        // - Black pixels = highest points (stick out)
        // - White pixels = lowest points (recessed)
        depth_map: Some(asset_server.load("textures/parallax_example/cube_depth.png")),
        // How deep the effect appears (0.0 = flat, higher = deeper)
        parallax_depth_scale,
        // Which algorithm to use
        parallax_mapping_method: parallax_mapping_method.0,
        // Quality setting - more layers = smoother effect
        max_parallax_layer_count,
        ..default()
    });
    // Main demo cube
    commands.spawn((
        Mesh3d(
            meshes.add(
                // IMPORTANT: Tangents are required for normal mapping!
                // Tangents define the "surface space" for the normal map
                // Without them, the lighting would be wrong
                Mesh::from(Cuboid::default())
                    .with_generated_tangents()
                    .unwrap(),
            ),
        ),
        MeshMaterial3d(parallax_material.clone()),
        Spin { speed: 0.3 }, // Rotate to see effect from all angles
    ));

    // Large background cubes to fill the scene
    let background_cube = meshes.add(
        Mesh::from(Cuboid::new(40.0, 40.0, 40.0))
            .with_generated_tangents()
            .unwrap(),
    );

    // Helper closure to create background cubes
    let background_cube_bundle = |translation| {
        (
            Mesh3d(background_cube.clone()),
            MeshMaterial3d(parallax_material.clone()),
            Transform::from_translation(translation),
            Spin { speed: -0.1 }, // Slow counter-rotation
        )
    };
    
    // Place cubes around the scene for visual interest
    commands.spawn(background_cube_bundle(Vec3::new(45., 0., 0.)));   // Right
    commands.spawn(background_cube_bundle(Vec3::new(-45., 0., 0.)));  // Left
    commands.spawn(background_cube_bundle(Vec3::new(0., 0., 45.)));   // Back
    commands.spawn(background_cube_bundle(Vec3::new(0., 0., -45.)));  // Front

    // UI instructions and status display
    commands
        .spawn((
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
        ))
        .with_children(|p| {
            // Status display - will be updated by systems
            p.spawn(TextSpan(format!(
                "Parallax depth scale: {parallax_depth_scale:.5}\n"
            )));
            p.spawn(TextSpan(format!("Layers: {max_parallax_layer_count:.0}\n")));
            p.spawn(TextSpan(format!("{parallax_mapping_method}\n")));
            p.spawn(TextSpan::new("\n\n"));
            
            // Control instructions
            p.spawn(TextSpan::new("Controls:\n"));
            p.spawn(TextSpan::new("Left click - Change view angle\n"));
            p.spawn(TextSpan::new(
                "1/2 - Decrease/Increase parallax depth scale\n",
            ));
            p.spawn(TextSpan::new("3/4 - Decrease/Increase layer count\n"));
            p.spawn(TextSpan::new("Space - Switch parallaxing algorithm\n"));
        });
}
