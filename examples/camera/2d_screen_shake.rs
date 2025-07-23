//! This example showcases a 2D screen shake using concept in this video: `<https://www.youtube.com/watch?v=tu-Qe66AvtY>`
//!
//! ## Key Concepts Demonstrated
//!
//! - **Trauma-based Screen Shake**: Using a trauma value that decays over time
//! - **SubCameraView**: Manipulating the camera's viewport offset for shake effects
//! - **Smooth Interpolation**: Using mathematical smoothing for natural-feeling motion
//! - **Perlin Noise Simulation**: Using random values to create organic shake patterns
//!
//! ## The Algorithm
//!
//! The screen shake uses a "trauma" system where:
//! 1. Trauma increases when the player triggers a shake (space key)
//! 2. The actual shake intensity = trauma²
//! 3. Random offsets and rotations are applied based on this intensity
//! 4. Both trauma and camera position smoothly decay back to normal
//!
//! ## Controls
//!
//! | Key Binding  | Action               |
//! |:-------------|:---------------------|
//! | Space        | Trigger screen shake |

use bevy::{
    prelude::*,
    // SubCameraView allows us to manipulate the camera's viewport region
    // This is useful for effects like screen shake without moving the actual camera entity
    render::camera::SubCameraView,
    // MeshMaterial2d is used for 2D mesh rendering with materials
    sprite::MeshMaterial2d,
};
// Random number generation for shake offsets
use rand::{Rng, SeedableRng};
// ChaCha8Rng is a cryptographically secure random number generator
// We use it for consistent, high-quality randomness in our shake effect
use rand_chacha::ChaCha8Rng;

// SCREEN SHAKE CONFIGURATION CONSTANTS
// =====================================

// How quickly the camera returns to its target position (0.0 = instant, 1.0 = never)
// 0.9 creates a smooth, slightly elastic feeling
const CAMERA_DECAY_RATE: f32 = 0.9;

// How many units of trauma are lost per second
// Higher values make the shake stop more quickly
const TRAUMA_DECAY_SPEED: f32 = 0.5;

// How much trauma is added per second while holding the trigger key
const TRAUMA_INCREMENT: f32 = 1.0;

// Maximum shake values (applied when trauma = 1.0)
// These are multiplied by trauma² to get the actual shake amount
const MAX_ANGLE: f32 = 0.5;      // Maximum rotation in radians
const MAX_OFFSET: f32 = 500.0;   // Maximum position offset in pixels

/// Marker component to identify the player entity
#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_scene, setup_instructions, setup_camera))
        .add_systems(Update, (screen_shake, trigger_shake_on_space))
        .run();
}

/// Sets up the game scene with background, player, and static objects
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // BACKGROUND: Large rectangle to serve as the game world backdrop
    // This helps visualize the screen shake effect since it provides reference points
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1000., 700.))),
        MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.3))), // Dark blue-gray
    ));

    // PLAYER: The main character (cyan rectangle)
    // In a real game, the screen shake might be triggered by player actions like
    // taking damage, explosions, or powerful attacks
    commands.spawn((
        Player,  // Marker component for identification
        Mesh2d(meshes.add(Rectangle::new(50.0, 100.0))),  // Tall rectangle
        MeshMaterial2d(materials.add(Color::srgb(0.25, 0.94, 0.91))),  // Bright cyan
        // Place at origin with Z=2 to appear in front of background
        Transform::from_xyz(0., 0., 2.),
    ));

    // STATIC OBJECT 1: Red square in upper-left area
    // These static objects help demonstrate that the shake affects the entire viewport
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(50.0, 50.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.85, 0.0, 0.2))),  // Bright red
        Transform::from_xyz(-450.0, 200.0, 2.),
    ));

    // STATIC OBJECT 2: Green rectangle in lower-right area
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(70.0, 50.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.5, 0.8, 0.2))),  // Lime green
        Transform::from_xyz(450.0, -150.0, 2.),
    ));
    
    // Initialize the ScreenShake resource with default values
    // This must be done before any systems try to use it
    commands.init_resource::<ScreenShake>();
}

/// Creates UI instructions for the user
fn setup_instructions(mut commands: Commands) {
    commands.spawn((
        Text::new("Hold space to trigger a screen shake"),
        Node {
            // Absolute positioning allows us to place UI elements precisely
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),  // 12 pixels from bottom of screen
            left: Val::Px(12.0),    // 12 pixels from left of screen
            ..default()
        },
    ));
}

/// Sets up the camera with SubCameraView for screen shake effects
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            // SubCameraView is the key to our screen shake implementation
            // It allows us to manipulate the camera's viewport without affecting the camera entity itself
            sub_camera_view: Some(SubCameraView {
                // full_size: The size of the virtual "world" we're looking at
                full_size: UVec2::new(1000, 700),
                
                // offset: Where in the world we're currently looking (0,0 = center)
                // This is what we'll modify to create the shake effect
                offset: Vec2::new(0.0, 0.0),
                
                // size: The size of the actual viewport on screen
                // This determines how much of the full_size world we can see at once
                size: UVec2::new(1000, 700),
            }),
            
            // Higher order means this camera renders after others
            // This ensures our shake effect appears on top of any UI elements
            order: 1,
            ..default()
        },
    ));
}

/// Resource that manages screen shake state and parameters
/// 
/// The ScreenShake resource uses a "trauma" system where trauma represents
/// the intensity of the shake effect. The actual shake magnitude is trauma²,
/// which creates a more natural feeling decay curve.
#[derive(Resource, Clone)]
struct ScreenShake {
    /// Maximum rotation angle (in radians) when trauma = 1.0
    max_angle: f32,
    /// Maximum position offset (in pixels) when trauma = 1.0
    max_offset: f32,
    /// Current trauma level (0.0 = no shake, 1.0 = maximum shake)
    /// The actual shake intensity is trauma² for more natural falloff
    trauma: f32,
    /// The position the camera should return to when shake ends
    /// In a following camera system, this would be the player's position
    latest_position: Option<Vec2>,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            max_angle: 0.0,
            max_offset: 0.0,
            trauma: 0.0,
            // Start with camera centered at origin
            latest_position: Some(Vec2::default()),
        }
    }
}

impl ScreenShake {
    /// Initiates or intensifies a screen shake effect
    /// 
    /// # Parameters
    /// - `max_angle`: Maximum rotation in radians
    /// - `max_offset`: Maximum position offset in pixels
    /// - `trauma`: Trauma level to set (will be clamped to 0.0-1.0)
    /// - `final_position`: Where the camera should return to when shake ends
    fn start_shake(&mut self, max_angle: f32, max_offset: f32, trauma: f32, final_position: Vec2) {
        self.max_angle = max_angle;
        self.max_offset = max_offset;
        // clamp() ensures trauma stays within valid bounds
        // Values outside 0.0-1.0 could cause unpredictable behavior
        self.trauma = trauma.clamp(0.0, 1.0);
        self.latest_position = Some(final_position);
    }
}

/// System that triggers screen shake when the space key is held down
fn trigger_shake_on_space(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        // We need to clone the current trauma value before modifying the resource
        // This is necessary because Rust's borrow checker prevents us from reading
        // from screen_shake while also holding a mutable reference to it
        let current_trauma = screen_shake.trauma;
        
        screen_shake.start_shake(
            MAX_ANGLE,
            MAX_OFFSET,
            // Gradually increase trauma over time while the key is held
            // Using delta_secs() ensures the effect is frame-rate independent
            current_trauma + TRAUMA_INCREMENT * time.delta_secs(),
            Vec2::ZERO,  // Return to center position (Vec2::ZERO is equivalent to Vec2::new(0.0, 0.0))
        );
        
        // In a real game, final_position would typically be the player's current position
        // so the camera returns to following the player after the shake ends
    }
}

/// The main screen shake system that applies shake effects to the camera
/// 
/// This system runs every frame and:
/// 1. Calculates random offsets based on current trauma level
/// 2. Applies position and rotation changes to the camera
/// 3. Smoothly interpolates toward the target position
/// 4. Decays trauma over time
fn screen_shake(
    time: Res<Time>,
    mut screen_shake: ResMut<ScreenShake>,
    mut query: Query<(&mut Camera, &mut Transform)>,
) {
    // Initialize random number generator with entropy from the system
    // ChaCha8Rng provides high-quality randomness for smooth shake effects
    let mut rng = ChaCha8Rng::from_entropy();
    
    // CALCULATE SHAKE INTENSITY
    // The key insight: shake intensity = trauma²
    // This creates a more natural falloff curve than linear trauma
    let shake = screen_shake.trauma * screen_shake.trauma;
    
    // GENERATE RANDOM OFFSETS
    // Each component (angle, x, y) gets its own random value in the range [-1.0, 1.0]
    let angle = screen_shake.max_angle * shake * rng.gen_range(-1.0..1.0);
    let offset_x = screen_shake.max_offset * shake * rng.gen_range(-1.0..1.0);
    let offset_y = screen_shake.max_offset * shake * rng.gen_range(-1.0..1.0);

    if shake > 0.0 {
        // ACTIVE SHAKE: Apply random offsets with smooth interpolation
        for (mut camera, mut transform) in query.iter_mut() {
            // POSITION SHAKE: Modify the SubCameraView offset
            let sub_view = camera.sub_camera_view.as_mut().unwrap();
            
            // Calculate target position: current position + random offset
            let target = sub_view.offset + Vec2::new(offset_x, offset_y);
            
            // smooth_nudge() gradually moves toward the target instead of jumping instantly
            // This creates a more organic, less jarring shake effect
            sub_view.offset.smooth_nudge(&target, CAMERA_DECAY_RATE, time.delta_secs());

            // ROTATION SHAKE: Apply random rotation to the camera transform
            let rotation = Quat::from_rotation_z(angle);
            
            // interpolate_stable() smoothly blends between current and target rotation
            // mul_quat() combines the existing rotation with the shake rotation
            transform.rotation = transform.rotation.interpolate_stable(
                &(transform.rotation.mul_quat(rotation)), 
                CAMERA_DECAY_RATE
            );
        }
    } else {
        // RECOVERY PHASE: Smoothly return camera to rest position
        if let Ok((mut camera, mut transform)) = query.single_mut() {
            let sub_view = camera.sub_camera_view.as_mut().unwrap();
            let target = screen_shake.latest_position.unwrap();
            
            // Use rate 1.0 for faster return to rest position
            sub_view.offset.smooth_nudge(&target, 1.0, time.delta_secs());
            
            // Return rotation to identity (no rotation)
            // Quat::IDENTITY represents zero rotation
            transform.rotation = transform.rotation.interpolate_stable(&Quat::IDENTITY, 0.1);
        }
    }
    
    // TRAUMA DECAY: Gradually reduce trauma over time
    // This ensures the shake effect naturally fades out
    screen_shake.trauma -= TRAUMA_DECAY_SPEED * time.delta_secs();
    screen_shake.trauma = screen_shake.trauma.clamp(0.0, 1.0);
}
