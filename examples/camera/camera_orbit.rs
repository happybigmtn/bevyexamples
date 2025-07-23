//! Shows how to orbit camera around a static scene using pitch, yaw, and roll.
//!
//! See also: `first_person_view_model` example, which does something similar but as a first-person
//! camera view.
//!
//! # Camera System Design and Control Theory
//!
//! Orbital camera controls are like being a satellite photographer - you stay at a fixed distance
//! from your subject but can rotate around it to get the perfect angle! Think of how you might
//! walk around a sculpture in a museum, always facing it but changing your perspective.
//! The three rotation axes (pitch, yaw, roll) are like the gimbal system on a professional
//! camera mount, allowing smooth movement in all directions without losing your target.
//!
//! ## Camera Projection Mathematics and View Frustums
//!
//! ### Perspective Projection Fundamentals
//! Orbital cameras typically use perspective projection to create natural 3D depth:
//! ```text
//! // Perspective projection matrix components
//! fov = field_of_view_in_radians
//! aspect = screen_width / screen_height
//! near = near_clipping_plane
//! far = far_clipping_plane
//!
//! // Projection matrix calculation
//! f = 1.0 / tan(fov / 2.0)
//! m00 = f / aspect
//! m11 = f
//! m22 = (far + near) / (near - far)
//! m23 = (2.0 * far * near) / (near - far)
//! ```
//!
//! ### View Frustum and Culling
//! The orbital camera's view frustum defines what's visible:
//! - **Near plane**: Closest visible distance (typically 0.1-1.0 units)
//! - **Far plane**: Furthest visible distance (typically 100-10000 units)
//! - **Field of view**: Angular width of vision (typically 45-90 degrees)
//! - **Aspect ratio**: Width/height ratio of the viewport
//!
//! ## Interpolation Algorithms for Smooth Camera Movement
//!
//! ### Linear Interpolation (LERP)
//! Basic smooth movement between positions:
//! ```text
//! lerp(start, end, t) = start + t * (end - start)
//! // Where t is typically: delta_time * interpolation_speed
//! ```
//!
//! ### Spherical Linear Interpolation (SLERP)
//! For smooth rotation interpolation:
//! ```text
//! slerp(q1, q2, t) = q1 * (q1⁻¹ * q2)^t
//! // Where q1, q2 are quaternions and t ∈ [0,1]
//! ```
//!
//! ### Exponential Smoothing
//! For natural-feeling camera lag:
//! ```text
//! smoothed_value = target * smoothing_factor + current * (1 - smoothing_factor)
//! // Where smoothing_factor = 1 - exp(-smoothing_rate * delta_time)
//! ```
//!
//! ## Constraint Systems and Collision Detection
//!
//! ### Gimbal Lock Prevention
//! Orbital cameras must prevent gimbal lock (loss of rotation axis):
//! - **Pitch constraints**: Limit vertical rotation to ±89°
//! - **Quaternion representation**: Use quaternions internally for stability
//! - **Euler angle clamping**: Clamp pitch in [-π/2 + ε, π/2 - ε]
//!
//! ### Camera Collision Detection
//! Advanced orbital cameras handle obstacles:
//! - **Ray casting**: Cast ray from target to desired camera position
//! - **Sphere collision**: Use camera radius for collision volume
//! - **Sliding**: Move camera along collision surface when blocked
//! - **Zoom adjustment**: Automatically reduce distance when blocked
//!
//! ### Distance Constraints
//! Limit how close/far the camera can orbit:
//! ```text
//! min_distance = 1.0;  // Prevent camera inside objects
//! max_distance = 100.0; // Prevent too-distant view
//! clamped_distance = clamp(desired_distance, min_distance, max_distance)
//! ```
//!
//! ## Multi-Camera Systems and Viewport Management
//!
//! ### Split-Screen Orbital Cameras
//! Multiple orbital cameras can share the scene:
//! - **Independent controls**: Each camera responds to different inputs
//! - **Synchronized targets**: Multiple cameras orbit same object
//! - **Viewport division**: Screen space divided between cameras
//!
//! ### Picture-in-Picture
//! Orbital camera as secondary view:
//! - **Main camera**: Player's primary perspective
//! - **Orbital overview**: Mini-map or tactical view
//! - **Render targets**: Each camera renders to different texture
//!
//! ### Camera Switching
//! Dynamic orbital camera management:
//! ```rust
//! enum CameraMode {
//!     FirstPerson,    // Direct control
//!     ThirdPerson,    // Follow behind player
//!     Orbital,        // Free orbit around target
//!     Cinematic,      // Scripted camera movements
//! }
//! ```
//!
//! ## Camera Shake and Procedural Animation
//!
//! ### Trauma-Based Shake
//! Realistic camera shake using trauma system:
//! ```text
//! trauma = clamp(trauma - recovery_rate * delta_time, 0.0, 1.0)
//! shake_power = trauma * trauma  // Quadratic falloff
//! shake_x = noise(time) * shake_power * max_shake_x
//! shake_y = noise(time + 100) * shake_power * max_shake_y
//! ```
//!
//! ### Procedural Orbit Animation
//! Automated orbital movement:
//! - **Sine wave orbits**: Smooth, predictable circular motion
//! - **Perlin noise**: Natural, organic camera movement
//! - **Scripted paths**: Keyframe-based camera animation
//!
//! ## Cinematic Camera Techniques
//!
//! ### Rule of Thirds
//! Position orbital camera for pleasing composition:
//! - **Target offset**: Don't always center the target
//! - **Leading space**: Leave room in direction of movement
//! - **Height variation**: Use different vertical angles for mood
//!
//! ### Camera Movements
//! Professional cinematography techniques:
//! - **Dolly**: Move camera toward/away from target (zoom distance)
//! - **Truck**: Move camera left/right while maintaining orientation
//! - **Pedestal**: Move camera up/down vertically
//! - **Pan**: Rotate camera horizontally (yaw)
//! - **Tilt**: Rotate camera vertically (pitch)
//! - **Roll**: Rotate camera around forward axis
//!
//! ### Focus Pulling
//! Depth of field control for orbital cameras:
//! ```text
//! focus_distance = distance_to_target
//! near_blur = focus_distance - depth_of_field_range / 2
//! far_blur = focus_distance + depth_of_field_range / 2
//! ```
//!
//! ## Human Factors in Camera Control
//!
//! ### Ergonomics and User Comfort
//! - **Motion sickness**: Avoid rapid rotations and sudden movements
//! - **Spatial orientation**: Maintain consistent "up" direction
//! - **Predictable controls**: Consistent input-to-movement mapping
//!
//! ### Accessibility Features
//! - **Reduced motion**: Options for users sensitive to camera movement
//! - **Alternative controls**: Multiple input methods for same actions
//! - **Visual indicators**: Show camera orientation and limits
//!
//! ### User Interface Integration
//! - **Context-sensitive cursors**: Different cursors for camera modes
//! - **Visual feedback**: Highlight active camera controls
//! - **Settings persistence**: Remember user preferences
//!
//! ## Performance Optimization for Camera Systems
//!
//! ### Frustum Culling
//! Only render objects visible to the orbital camera:
//! ```text
//! // Extract frustum planes from view-projection matrix
//! for each object:
//!     if object_bounds intersects camera_frustum:
//!         render_object()
//! ```
//!
//! ### Level-of-Detail (LOD)
//! Adjust detail based on distance from orbital camera:
//! ```text
//! distance_to_camera = length(object_position - camera_position)
//! lod_level = clamp(distance_to_camera / lod_distance, 0, max_lod)
//! ```
//!
//! ### Occlusion Culling
//! Skip rendering objects hidden behind others:
//! - **Hardware occlusion queries**: GPU-based visibility testing
//! - **Software occlusion**: CPU-based pre-culling
//! - **Hierarchical Z-buffer**: Efficient depth testing
//!
//! ## Real-World Applications
//!
//! ### Game Genre Applications
//!
//! #### Strategy Games
//! - **RTS cameras**: High-angle orbital view for battlefield overview
//! - **City builders**: Smooth zoom from street level to city-wide view
//! - **4X games**: Planetary and galactic scale orbital cameras
//!
//! #### Action Games
//! - **Boss fights**: Dramatic orbital cameras for large enemies
//! - **Vehicle games**: Orbital chase cameras for cars/planes
//! - **Sports games**: Broadcasting-style camera angles
//!
//! #### Simulation Games
//! - **Flight simulators**: External view for aircraft inspection
//! - **Architecture**: Walkthrough cameras for building visualization
//! - **Scientific visualization**: Data exploration with 3D navigation
//!
//! ### Industry Best Practices
//! - **Unreal Engine**: Uses spherical coordinates for orbital cameras
//! - **Unity**: Cinemachine provides advanced orbital camera tools
//! - **Film industry**: Maya/3ds Max orbital camera workflows
//! - **CAD software**: Professional 3D navigation patterns

use std::{f32::consts::FRAC_PI_2, ops::Range};

use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

// CAMERA CONFIGURATION - The control panel for our orbital camera
#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,    // How far from target (zoom level)
    pub pitch_speed: f32,       // Up/down rotation sensitivity
    // Clamp pitch to this range (prevents camera flipping upside down)
    pub pitch_range: Range<f32>, // Vertical rotation limits
    pub roll_speed: f32,        // Tilt rotation speed
    pub yaw_speed: f32,         // Left/right rotation sensitivity
}

impl Default for CameraSettings {
    fn default() -> Self {
        // GIMBAL LOCK PREVENTION - Keep camera from flipping upside down
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        // Like preventing a drone from doing barrel rolls when you just want to look up!
        let pitch_limit = FRAC_PI_2 - 0.01;  // Just under 90 degrees
        
        Self {
            // TUNED VALUES - Calibrated for smooth, natural camera movement
            // These values are completely arbitrary, chosen because they seem to produce
            // "sensible" results for this example. Adjust as required.
            orbit_distance: 20.0,               // 20 units from target
            pitch_speed: 0.003,                 // Gentle up/down movement
            pitch_range: -pitch_limit..pitch_limit,  // ±89.99° vertical range
            roll_speed: 1.0,                    // 1 radian per second tilt
            yaw_speed: 0.004,                   // Gentle left/right movement
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<CameraSettings>()     // Load our camera configuration
        .add_systems(Startup, (setup, instructions))
        .add_systems(Update, orbit)            // Continuous camera movement
        .run();
}

/// THE PHOTOGRAPHY STUDIO - Set up a simple 3D scene to orbit around
/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // THE ORBITAL CAMERA - Our satellite photographer
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        // Start at distance, looking at origin (our orbit target)
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // THE SUBJECT - Objects to orbit around
    // Ground plane
    commands.spawn((
        Name::new("Plane"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),  // Green ground
            // Turning off culling keeps the plane visible when viewed from beneath.
            // Important for orbital cameras that might look up from below!
            cull_mode: None,
            ..default()
        })),
    ));

    // Subject cube to examine from all angles
    commands.spawn((
        Name::new("Cube"),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),  // Beige cube
        Transform::from_xyz(1.5, 0.51, 1.5),  // Slightly off-center
    ));

    // STUDIO LIGHTING - Illuminate our subjects
    commands.spawn((
        Name::new("Light"),
        PointLight::default(),
        Transform::from_xyz(3.0, 8.0, 5.0),  // Above and to the side
    ));
}

fn instructions(mut commands: Commands) {
    // USER GUIDE - Explain the control scheme
    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "Mouse up or down: pitch\n\
            Mouse left or right: yaw\n\
            Mouse buttons: roll",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.),
            left: Val::Px(12.),   // Top-left corner
            ..default()
        },
    ));
}

fn orbit(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    // INPUT PROCESSING - Convert mouse movement to rotation
    let delta = mouse_motion.delta;  // Raw mouse movement this frame
    let mut delta_roll = 0.0;

    // ROLL CONTROL - Mouse buttons tilt the camera left/right
    // Like tilting your head when looking at something
    if mouse_buttons.pressed(MouseButton::Left) {
        delta_roll -= 1.0;  // Roll left
    }
    if mouse_buttons.pressed(MouseButton::Right) {
        delta_roll += 1.0;  // Roll right
    }

    // TIMING CONSIDERATIONS - Different inputs need different time handling
    // Mouse motion is one of the few inputs that should not be multiplied by delta time,
    // as we are already receiving the full movement since the last frame was rendered. Multiplying
    // by delta time here would make the movement slower that it should be.
    // Mouse movement = immediate response (like moving your eyes)
    let delta_pitch = delta.y * camera_settings.pitch_speed;  // Up/down mouse = pitch
    let delta_yaw = delta.x * camera_settings.yaw_speed;      // Left/right mouse = yaw

    // Conversely, we DO need to factor in delta time for mouse button inputs.
    // Button presses = gradual response (like slowly tilting your head)
    delta_roll *= camera_settings.roll_speed * time.delta_secs();

    // ROTATION MATH - Convert quaternion to angles, modify, then convert back
    // Obtain the existing pitch, yaw, and roll values from the transform.
    // Like reading the current position of a tripod head
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    // APPLY MOVEMENT WITH LIMITS - Update angles but prevent camera flipping
    // Establish the new yaw and pitch, preventing the pitch value from exceeding our limits.
    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,  // Don't look too far down
        camera_settings.pitch_range.end,    // Don't look too far up
    );
    let roll = roll + delta_roll;    // Tilt has no limits (could do barrel rolls!)
    let yaw = yaw + delta_yaw;       // Spin has no limits (can rotate 360°)
    
    // REBUILD ROTATION - Convert angles back to quaternion
    camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

    // ORBITAL POSITIONING - Keep camera at fixed distance from target
    // Adjust the translation to maintain the correct orientation toward the orbit target.
    // In our example it's a static target, but this could easily be customized.
    let target = Vec3::ZERO;  // The point we're orbiting around
    
    // DISTANCE MAINTENANCE - Always stay the same distance from target
    // Like being on an invisible tether that keeps you the right distance away
    camera.translation = target - camera.forward() * camera_settings.orbit_distance;
    
    // ADVANCED TECHNIQUES: Production-ready orbital camera enhancements
    // 
    // 1. COLLISION DETECTION - Prevent camera from going through walls
    // let desired_position = target - camera.forward() * camera_settings.orbit_distance;
    // if let Some(collision) = raycast_camera_collision(target, desired_position) {
    //     camera.translation = collision.point + collision.normal * camera_radius;
    // } else {
    //     camera.translation = desired_position;
    // }
    //
    // 2. SMOOTH INTERPOLATION - Reduce jarring camera movements
    // let target_position = target - camera.forward() * camera_settings.orbit_distance;
    // let smoothing_factor = 1.0 - (-8.0 * time.delta_secs()).exp();
    // camera.translation = camera.translation.lerp(target_position, smoothing_factor);
    //
    // 3. DYNAMIC TARGET TRACKING - Follow moving objects
    // if let Ok(target_transform) = target_query.get_single() {
    //     let moving_target = target_transform.translation;
    //     camera.translation = moving_target - camera.forward() * camera_settings.orbit_distance;
    // }
    //
    // 4. ZOOM CONSTRAINTS - Limit minimum/maximum distances
    // let zoom_input = scroll_events.iter().fold(0.0, |acc, e| acc + e.y);
    // camera_settings.orbit_distance = (camera_settings.orbit_distance - zoom_input * 2.0)
    //     .clamp(camera_settings.min_distance, camera_settings.max_distance);
    //
    // 5. CAMERA SHAKE INTEGRATION - Add trauma-based shake
    // let shake_offset = Vec3::new(
    //     noise(time.elapsed_secs()) * trauma * trauma,
    //     noise(time.elapsed_secs() + 100.0) * trauma * trauma,
    //     0.0
    // ) * shake_intensity;
    // camera.translation += shake_offset;
    //
    // PERFORMANCE OPTIMIZATIONS:
    // - Update only when input detected (early return if no movement)
    // - Use dirty flags to avoid unnecessary matrix recalculations
    // - Cache trigonometric calculations when possible
    // - Implement frame rate-independent smoothing
    //
    // INDUSTRY PATTERNS:
    // - Unreal Engine: Uses FRotator for pitch/yaw/roll representation
    // - Unity: Cinemachine vcam with orbital transposer
    // - Maya: Tumble/track/dolly navigation (rotate/pan/zoom)
    // - Blender: Orbit around 3D cursor or selected object
    //
    // MATHEMATICAL FOUNDATIONS:
    // This orbital camera implementation uses:
    // - Euler angle composition (YXZ order to prevent gimbal lock)
    // - Spherical coordinate system (radius, azimuth, elevation)
    // - Quaternion rotation for stable interpolation
    // - Vector math for position calculation
    //
    // COORDINATE SYSTEM NOTES:
    // Bevy uses right-handed coordinate system:
    // - X points right
    // - Y points up  
    // - Z points toward viewer (out of screen)
    // - Rotations follow right-hand rule
    //
    // This affects how we interpret pitch, yaw, and roll:
    // - Pitch: Rotation around X-axis (look up/down)
    // - Yaw: Rotation around Y-axis (look left/right)
    // - Roll: Rotation around Z-axis (tilt left/right)
}
