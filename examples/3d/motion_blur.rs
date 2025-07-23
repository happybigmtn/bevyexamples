//! Demonstrates how to enable per-object motion blur. This rendering feature can be configured per
//! camera using the [`MotionBlur`] component.
//!
//! # What is Motion Blur?
//!
//! Motion blur simulates the way cameras capture fast-moving objects. When a camera's
//! shutter is open, moving objects leave a "trail" or "streak" in the image.
//!
//! In real cameras:
//! - **Shutter angle**: How long the shutter stays open (0-360 degrees)
//! - **Motion trail**: Object positions are averaged over the exposure time
//! - **Natural look**: Makes motion feel smooth and cinematic
//!
//! # Why Use Motion Blur?
//!
//! 1. **Cinematic quality**: Makes games look like movies
//! 2. **Smooth motion**: Reduces perceived "jerkiness" at lower framerates
//! 3. **Speed perception**: Helps players sense how fast things are moving
//! 4. **Realism**: Matches how we see fast motion in films/photos
//!
//! # Performance Considerations
//!
//! Motion blur has a cost:
//! - Requires motion vectors (previous frame positions)
//! - Multiple samples = multiple texture reads
//! - More samples = better quality but slower
//!
//! This example shows a racing scene where cars move in a loop,
//! demonstrating how motion blur enhances the sense of speed.

use bevy::{
    core_pipeline::motion_blur::MotionBlur, // Component that enables motion blur
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    math::ops, // Bevy's optimized math operations (sin, cos, etc.)
    prelude::*,
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_scene, setup_ui))
        // chain() ensures systems run in order - important for camera following cars
        .add_systems(Update, (keyboard_inputs, move_cars, move_camera).chain())
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Add the `MotionBlur` component to a camera to enable motion blur.
        // Motion blur requires the depth and motion vector prepass, which this bundle adds.
        // Configure the amount and quality of motion blur per-camera using this component.
        MotionBlur {
            // Shutter angle in range [0.0, 1.0]
            // 0.0 = No motion blur (instant shutter)
            // 0.5 = 180° shutter (film standard)
            // 1.0 = 360° shutter (maximum blur)
            shutter_angle: 1.0,
            // Number of samples for quality
            // More samples = smoother blur but more expensive
            // 1 = Rough/fast, 8 = Good quality, 32+ = Film quality
            samples: 2,
        },
        // MSAA and Motion Blur together are not compatible on WebGL
        // This conditional compilation ensures WebGL builds work correctly
        #[cfg(all(feature = "webgl2", target_arch = "wasm32", not(feature = "webgpu")))]
        Msaa::Off,
    ));
}

// Everything past this point is used to build the example, but isn't required to use motion blur.

/// Camera can either track the car from a distance or chase behind it
#[derive(Resource)]
enum CameraMode {
    Track, // Fixed position, rotates to follow
    Chase, // Follows behind the car
}

/// Component for objects that move along the race track
/// The f32 value is a time offset to space out multiple cars
#[derive(Component)]
struct Moves(f32);

/// Marks which car the camera should follow
#[derive(Component)]
struct CameraTracked;

/// Component for wheels that should rotate as the car moves
#[derive(Component)]
struct Rotates;

fn setup_scene(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Bright ambient light so we can see everything clearly
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        ..default()
    });
    // Start in chase mode for dramatic effect
    commands.insert_resource(CameraMode::Chase);
    
    // Sun light casting shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 3_000.0,
            shadows_enabled: true,
            ..default()
        },
        // Angle the light for interesting shadows
        Transform::default().looking_to(Vec3::new(-1.0, -0.7, -1.0), Vec3::X),
    ));
    
    // Sky sphere (inside-out sphere for skybox effect)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            unlit: true, // Sky doesn't need lighting
            base_color: Color::linear_rgb(0.1, 0.6, 1.0), // Sky blue
            ..default()
        })),
        // Negative scale flips the sphere inside-out
        Transform::default().with_scale(Vec3::splat(-4000.0)),
    ));
    
    // Ground plane with custom UV mapping for tiled texture
    let mut plane: Mesh = Plane3d::default().into();
    let uv_size = 4000.0; // Large UV coords = many texture repeats
    // Custom UV coordinates for each vertex of the plane
    let uvs = vec![[uv_size, 0.0], [0.0, 0.0], [0.0, uv_size], [uv_size; 2]];
    plane.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    commands.spawn((
        Mesh3d(meshes.add(plane)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0, // Rough surface (no reflections)
            base_color_texture: Some(images.add(uv_debug_texture())),
            ..default()
        })),
        Transform::from_xyz(0.0, -0.65, 0.0).with_scale(Vec3::splat(80.)),
    ));

    // Spawn all scene elements
    spawn_cars(&asset_server, &mut meshes, &mut materials, &mut commands);
    spawn_trees(&mut meshes, &mut materials, &mut commands);
    spawn_barriers(&mut meshes, &mut materials, &mut commands);
}

fn spawn_cars(
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    const N_CARS: usize = 20; // Total cars in the race
    
    // Car body mesh - elongated box
    let box_mesh = meshes.add(Cuboid::new(0.3, 0.15, 0.55));
    // Wheel mesh - cylinder on its side
    let cylinder = meshes.add(Cylinder::default());
    
    // Load Bevy logo for wheel texture
    let logo = asset_server.load("branding/icon.png");
    let wheel_matl = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(logo.clone()),
        ..default()
    });

    // Helper closure to create materials with different colors
    let mut matl = |color| {
        materials.add(StandardMaterial {
            base_color: color,
            ..default()
        })
    };

    // Rainbow of car colors for variety
    let colors = [
        matl(Color::linear_rgb(1.0, 0.0, 0.0)),   // Red
        matl(Color::linear_rgb(1.0, 1.0, 0.0)),   // Yellow
        matl(Color::BLACK),                       // Black
        matl(Color::linear_rgb(0.0, 0.0, 1.0)),   // Blue
        matl(Color::linear_rgb(0.0, 1.0, 0.0)),   // Green
        matl(Color::linear_rgb(1.0, 0.0, 1.0)),   // Magenta
        matl(Color::linear_rgb(0.5, 0.5, 0.0)),   // Olive
        matl(Color::linear_rgb(1.0, 0.5, 0.0)),   // Orange
    ];

    for i in 0..N_CARS {
        let color = colors[i % colors.len()].clone();
        commands
            .spawn((
                Mesh3d(box_mesh.clone()),
                MeshMaterial3d(color.clone()),
                Transform::from_scale(Vec3::splat(0.5)),
                // Each car gets a time offset so they're spread around the track
                Moves(i as f32 * 2.0),
            ))
            // First car (i==0) is tracked by the camera
            .insert_if(CameraTracked, || i == 0)
            .with_children(|parent| {
                // Car roof/cockpit
                parent.spawn((
                    Mesh3d(box_mesh.clone()),
                    MeshMaterial3d(color),
                    Transform::from_xyz(0.0, 0.08, 0.03)
                        .with_scale(Vec3::new(1.0, 1.0, 0.5)), // Flatter than body
                ));
                
                // Helper to spawn wheels at corners
                let mut spawn_wheel = |x: f32, z: f32| {
                    parent.spawn((
                        Mesh3d(cylinder.clone()),
                        MeshMaterial3d(wheel_matl.clone()),
                        Transform::from_xyz(0.14 * x, -0.045, 0.15 * z)
                            .with_scale(Vec3::new(0.15, 0.04, 0.15)) // Thin cylinders
                            // Rotate 90° to make cylinder horizontal
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                        Rotates, // Wheels will spin!
                    ));
                };
                // Four wheels at corners
                spawn_wheel(1.0, 1.0);   // Front right
                spawn_wheel(1.0, -1.0);  // Back right
                spawn_wheel(-1.0, 1.0);  // Front left
                spawn_wheel(-1.0, -1.0); // Back left
            });
    }
}

/// Creates orange traffic cones along the track edges
fn spawn_barriers(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    const N_CONES: usize = 100; // Cones per side
    
    // Capsule shape approximates a traffic cone
    let capsule = meshes.add(Capsule3d::default());
    let matl = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(255, 87, 51), // Safety orange
        reflectance: 1.0, // Shiny plastic look
        ..default()
    });
    
    // Helper to spawn a line of cones at given offset from track center
    let mut spawn_with_offset = |offset: f32| {
        for i in 0..N_CONES {
            // Distribute cones evenly around the track
            let t = (i as f32) / (N_CONES as f32) * std::f32::consts::PI * 2.0;
            let pos = race_track_pos(offset, t);
            commands.spawn((
                Mesh3d(capsule.clone()),
                MeshMaterial3d(matl.clone()),
                Transform::from_xyz(pos.x, -0.65, pos.y)
                    .with_scale(Vec3::splat(0.07)), // Small cones
            ));
        }
    };
    spawn_with_offset(0.04);  // Inner track edge
    spawn_with_offset(-0.04); // Outer track edge
}

/// Creates simple trees around the track for scenery
fn spawn_trees(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    const N_TREES: usize = 30; // Trees per side
    
    // Simple geometry for trees
    let capsule = meshes.add(Capsule3d::default()); // Tree trunk
    let sphere = meshes.add(Sphere::default());     // Tree foliage
    
    // Tree materials
    let leaves = materials.add(Color::linear_rgb(0.0, 1.0, 0.0)); // Bright green
    let trunk = materials.add(Color::linear_rgb(0.4, 0.2, 0.2));  // Brown

    // Helper to spawn a line of trees at given offset
    let mut spawn_with_offset = |offset: f32| {
        for i in 0..N_TREES {
            // Distribute trees around track
            let t = (i as f32) / (N_TREES as f32) * std::f32::consts::PI * 2.0;
            let pos = race_track_pos(offset, t);
            let [x, z] = pos.into();
            
            // Spawn foliage (sphere)
            commands.spawn((
                Mesh3d(sphere.clone()),
                MeshMaterial3d(leaves.clone()),
                Transform::from_xyz(x, -0.3, z).with_scale(Vec3::splat(0.3)),
            ));
            
            // Spawn trunk (capsule)
            commands.spawn((
                Mesh3d(capsule.clone()),
                MeshMaterial3d(trunk.clone()),
                Transform::from_xyz(x, -0.5, z)
                    .with_scale(Vec3::new(0.05, 0.3, 0.05)), // Tall and thin
            ));
        }
    };
    spawn_with_offset(0.07);  // Trees on outside of track
    spawn_with_offset(-0.07); // Trees on inside of track
}

/// Creates UI text showing controls and current settings
fn setup_ui(mut commands: Commands) {
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
            // First two spans will show current settings (updated in keyboard_inputs)
            p.spawn(TextSpan::default());
            p.spawn(TextSpan::default());
            // Control instructions
            p.spawn(TextSpan::new("1/2: -/+ shutter angle (blur amount)\n"));
            p.spawn(TextSpan::new("3/4: -/+ sample count (blur quality)\n"));
            p.spawn(TextSpan::new("Spacebar: cycle camera\n"));
        });
}

/// Handles keyboard controls for adjusting motion blur settings
fn keyboard_inputs(
    mut motion_blur: Single<&mut MotionBlur>,
    presses: Res<ButtonInput<KeyCode>>,
    text: Single<Entity, With<Text>>,
    mut writer: TextUiWriter,
    mut camera: ResMut<CameraMode>,
) {
    // Adjust shutter angle (blur amount)
    if presses.just_pressed(KeyCode::Digit1) {
        motion_blur.shutter_angle -= 0.25;
    } else if presses.just_pressed(KeyCode::Digit2) {
        motion_blur.shutter_angle += 0.25;
    } 
    // Adjust sample count (blur quality)
    else if presses.just_pressed(KeyCode::Digit3) {
        // saturating_sub prevents underflow below 0
        motion_blur.samples = motion_blur.samples.saturating_sub(1);
    } else if presses.just_pressed(KeyCode::Digit4) {
        motion_blur.samples += 1;
    } 
    // Toggle camera mode
    else if presses.just_pressed(KeyCode::Space) {
        *camera = match *camera {
            CameraMode::Track => CameraMode::Chase,
            CameraMode::Chase => CameraMode::Track,
        };
    }
    
    // Clamp values to valid ranges
    motion_blur.shutter_angle = motion_blur.shutter_angle.clamp(0.0, 1.0);
    motion_blur.samples = motion_blur.samples.clamp(0, 64);
    
    // Update UI text to show current values
    let entity = *text;
    *writer.text(entity, 1) = format!("Shutter angle: {:.2}\n", motion_blur.shutter_angle);
    *writer.text(entity, 2) = format!("Samples: {:.5}\n", motion_blur.samples);
}

/// Parametric function for a looping race track. `offset` will return the point offset
/// perpendicular to the track at the given point.
///
/// This creates a figure-8 style track using Lissajous curves:
/// - x follows sin(2t) pattern
/// - y follows cos(3t) pattern
/// The different frequencies create the crossing pattern
fn race_track_pos(offset: f32, t: f32) -> Vec2 {
    let x_tweak = 2.0; // X frequency multiplier
    let y_tweak = 3.0; // Y frequency multiplier (3/2 ratio makes figure-8)
    let scale = 8.0;   // Overall track size
    
    // Base track position
    let x0 = ops::sin(x_tweak * t);
    let y0 = ops::cos(y_tweak * t);
    
    // Track tangent (derivative) for finding perpendicular
    let dx = x_tweak * ops::cos(x_tweak * t);
    let dy = y_tweak * -ops::sin(y_tweak * t);
    
    // Normalize the perpendicular vector
    let dl = ops::hypot(dx, dy); // Length of tangent
    
    // Offset perpendicular to track
    // Perpendicular to (dx,dy) is (-dy,dx)
    let x = x0 + offset * dy / dl;
    let y = y0 - offset * dx / dl;
    
    Vec2::new(x, y) * scale
}

/// Moves cars along the race track and rotates their wheels
fn move_cars(
    time: Res<Time>,
    mut movables: Query<(&mut Transform, &Moves, &Children)>,
    mut spins: Query<&mut Transform, (Without<Moves>, With<Rotates>)>,
) {
    for (mut transform, moves, children) in &mut movables {
        // Base time scaled down for reasonable speed
        let time = time.elapsed_secs() * 0.25;
        // Add car's time offset to spread them around track
        let t = time + 0.5 * moves.0;
        
        // Calculate speed variation based on track curvature
        // This makes cars slow down in tight turns
        let dx = ops::cos(t);
        let dz = -ops::sin(3.0 * t);
        let speed_variation = (dx * dx + dz * dz).sqrt() * 0.15;
        let t = t + speed_variation;
        
        // Store previous position to calculate movement delta
        let prev = transform.translation;
        
        // Update car position along track
        let track_pos = race_track_pos(0.0, t);
        transform.translation.x = track_pos.x;
        transform.translation.z = track_pos.y; // 2D track pos -> 3D world
        transform.translation.y = -0.59; // Fixed height above ground
        
        // Calculate movement for this frame
        let delta = transform.translation - prev;
        
        // Point car in direction of movement
        transform.look_to(delta, Vec3::Y);
        
        // Rotate wheels based on distance traveled
        for child in children.iter() {
            let Ok(mut wheel) = spins.get_mut(child) else {
                continue;
            };
            // Calculate wheel rotation from distance
            let radius = wheel.scale.x;
            let circumference = 2.0 * std::f32::consts::PI * radius;
            let angle = delta.length() / circumference * std::f32::consts::PI * 2.0;
            wheel.rotate_local_y(angle);
        }
    }
}

/// Updates camera position based on selected mode
fn move_camera(
    camera: Single<(&mut Transform, &mut Projection), Without<CameraTracked>>,
    tracked: Single<&Transform, With<CameraTracked>>,
    mode: Res<CameraMode>,
) {
    let (mut transform, mut projection) = camera.into_inner();
    match *mode {
        CameraMode::Track => {
            // Fixed position, rotates to track car
            transform.look_at(tracked.translation, Vec3::Y);
            transform.translation = Vec3::new(15.0, -0.5, 0.0);
            if let Projection::Perspective(perspective) = &mut *projection {
                // Very narrow FOV = telephoto lens effect
                perspective.fov = 0.05;
            }
        }
        CameraMode::Chase => {
            // Follow behind the car
            transform.translation =
                tracked.translation 
                + Vec3::new(0.0, 0.15, 0.0)  // Slightly above car
                + tracked.back() * 0.6;      // Behind car
            transform.look_to(tracked.forward(), Vec3::Y);
            if let Projection::Perspective(perspective) = &mut *projection {
                // Wide FOV for dramatic chase view
                perspective.fov = 1.0;
            }
        }
    }
}

/// Creates a simple checkerboard texture for the ground
/// This helps visualize motion by providing a reference pattern
fn uv_debug_texture() -> Image {
    use bevy::render::{render_asset::RenderAssetUsages, render_resource::*};
    const TEXTURE_SIZE: usize = 7;

    // Gray values for checkerboard pattern (RGBA format)
    let mut palette = [
        164, 164, 164, 255, // Light gray
        168, 168, 168, 255, // Slightly lighter
        153, 153, 153, 255, // Medium gray
        139, 139, 139, 255, // Darker gray
        153, 153, 153, 255, // Medium gray
        177, 177, 177, 255, // Light gray
        159, 159, 159, 255, // Medium gray
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        // Rotate palette to create checkerboard effect
        palette.rotate_right(12); // 12 = 3 pixels * 4 bytes
    }

    let mut img = Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    
    // Configure texture sampling
    img.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,      // Tile horizontally
        address_mode_v: ImageAddressMode::MirrorRepeat, // Mirror vertically
        mag_filter: ImageFilterMode::Nearest,           // Pixelated look
        ..ImageSamplerDescriptor::linear()
    });
    img
}
