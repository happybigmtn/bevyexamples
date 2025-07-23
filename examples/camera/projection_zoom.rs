//! Shows how to zoom orthographic and perspective projection cameras.
//!
//! ## Key Concepts Demonstrated
//!
//! - **Orthographic vs Perspective Projections**: Understanding the fundamental differences
//! - **Camera Zooming Techniques**: Different approaches for different projection types
//! - **Mouse Wheel Input**: Processing scroll events for intuitive zoom controls
//! - **Dynamic Projection Switching**: Runtime switching between projection types
//! - **Field of View (FOV)**: How FOV affects perspective camera zoom
//! - **Orthographic Scale**: How scale affects orthographic camera zoom
//!
//! ## Projection Types Explained
//!
//! **Orthographic Projection:**
//! - No perspective distortion (parallel lines stay parallel)
//! - Objects don't get smaller with distance
//! - Zoom by changing the `scale` value
//! - Commonly used for 2D games, technical drawings, or top-down views
//!
//! **Perspective Projection:**
//! - Realistic perspective distortion (like human vision)
//! - Objects get smaller with distance
//! - Zoom by changing the Field of View (FOV)
//! - Commonly used for 3D games and realistic rendering
//!
//! ## Controls
//!
//! - **Mouse Wheel**: Zoom in/out
//! - **Space**: Switch between orthographic and perspective projections

use std::{f32::consts::PI, ops::Range};

use bevy::{
    // AccumulatedMouseScroll provides smooth mouse wheel input accumulation across frames
    input::mouse::AccumulatedMouseScroll,
    prelude::*,
    // ScalingMode determines how orthographic projections handle different aspect ratios
    render::camera::ScalingMode,
};

/// Configuration resource that stores all camera zoom settings
/// 
/// This resource centralizes all zoom-related parameters, making it easy to tweak
/// the camera behavior without diving into the system code.
#[derive(Debug, Resource)]
struct CameraSettings {
    /// The height of the viewport in world units when the orthographic camera's scale is 1.0
    /// 
    /// This defines the "default" zoom level for orthographic projection.
    /// A viewport height of 5.0 means that 5 world units will be visible vertically
    /// when the scale is 1.0.
    pub orthographic_viewport_height: f32,
    
    /// Valid range for orthographic camera scale values
    /// 
    /// Scale works inversely to zoom:
    /// - scale = 0.5 → zoomed in (shows less area)
    /// - scale = 2.0 → zoomed out (shows more area)
    pub orthographic_zoom_range: Range<f32>,
    
    /// Sensitivity multiplier for orthographic zoom with mouse wheel
    /// 
    /// Higher values = faster zoom response to mouse wheel movement
    pub orthographic_zoom_speed: f32,
    
    /// Valid range for perspective camera field of view (in radians)
    /// 
    /// FOV works directly with zoom:
    /// - smaller FOV = zoomed in (narrower view)
    /// - larger FOV = zoomed out (wider view)
    /// π radians = 180 degrees (typically the maximum useful FOV)
    pub perspective_zoom_range: Range<f32>,
    
    /// Sensitivity multiplier for perspective zoom with mouse wheel
    /// 
    /// FOV changes are more visually dramatic than scale changes, so this
    /// is typically smaller than orthographic_zoom_speed
    pub perspective_zoom_speed: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(CameraSettings {
            // ORTHOGRAPHIC SETTINGS
            orthographic_viewport_height: 5.,
            
            // Orthographic zoom range: scale values from 0.1 to 10.0
            // Remember: smaller scale = more zoomed in
            // 0.1 = very close up, 10.0 = very far away
            orthographic_zoom_range: 0.1..10.0,
            
            // Orthographic zoom speed: tuned for smooth but responsive zooming
            // This multiplies the mouse wheel delta to determine scale change
            orthographic_zoom_speed: 0.2,
            
            // PERSPECTIVE SETTINGS  
            // Perspective FOV range in radians:
            // PI/5 ≈ 36° (zoomed in), PI-0.2 ≈ 168° (zoomed out)
            // We avoid exactly PI (180°) to prevent rendering issues
            perspective_zoom_range: (PI / 5.)..(PI - 0.2),
            
            // Perspective zoom speed: slower than orthographic because
            // FOV changes are more visually dramatic due to the exponential
            // nature of perspective projection
            perspective_zoom_speed: 0.05,
        })
        .add_systems(Startup, (setup, instructions))
        .add_systems(Update, (switch_projection, zoom))
        .run();
}

/// Sets up a simple 3D scene to demonstrate the different projection and zoom behaviors
fn setup(
    asset_server: Res<AssetServer>,
    camera_settings: Res<CameraSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // CAMERA SETUP: Start with orthographic projection
    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        
        // Start with orthographic projection to demonstrate the difference
        Projection::from(OrthographicProjection {
            // SCALING MODE: FixedVertical keeps the vertical viewport constant
            // as the window aspect ratio changes. This is ideal for many games
            // where you want consistent vertical coverage.
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: camera_settings.orthographic_viewport_height,
            },
            
            // INITIAL SCALE: Start at 1.0 (the "neutral" zoom level)
            // This means the viewport height will be exactly the value
            // specified in scaling_mode when scale = 1.0
            scale: 1.,
            
            // Use default values for all other orthographic settings
            ..OrthographicProjection::default_3d()
        }),
        
        // Position camera at an angle to show the 3D scene clearly
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Plane"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            // Turning off culling keeps the plane visible when viewed from beneath.
            cull_mode: None,
            ..default()
        })),
    ));

    commands.spawn((
        Name::new("Fox"),
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/animated/Fox.glb")),
        ),
        // Note: the scale adjustment is purely an accident of our fox model, which renders
        // HUGE unless mitigated!
        Transform::from_translation(Vec3::splat(0.0)).with_scale(Vec3::splat(0.025)),
    ));

    commands.spawn((
        Name::new("Light"),
        PointLight::default(),
        Transform::from_xyz(3.0, 8.0, 5.0),
    ));
}

fn instructions(mut commands: Commands) {
    commands.spawn((
        Name::new("Instructions"),
        Text::new(
            "Scroll mouse wheel to zoom in/out\n\
            Space: switch between orthographic and perspective projections",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));
}

fn switch_projection(
    mut camera: Single<&mut Projection, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Switch projection type
        **camera = match **camera {
            Projection::Orthographic(_) => Projection::Perspective(PerspectiveProjection {
                fov: camera_settings.perspective_zoom_range.start,
                ..default()
            }),
            Projection::Perspective(_) => Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: camera_settings.orthographic_viewport_height,
                },
                ..OrthographicProjection::default_3d()
            }),
            _ => return,
        }
    }
}

/// Handles zoom input for both orthographic and perspective cameras
/// 
/// This system demonstrates the key differences between zooming techniques:
/// - Orthographic: Change the scale (multiplicatively for smooth feel)
/// - Perspective: Change the field of view (additively)
fn zoom(
    camera: Single<&mut Projection, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
) {
    // Pattern match on the projection type to handle each differently
    // In a real game, you'd typically only support one projection type
    match *camera.into_inner() {
        Projection::Orthographic(ref mut orthographic) => {
            // ORTHOGRAPHIC ZOOM LOGIC
            
            // Negate the delta because we want:
            // - Scroll up (positive delta) → zoom in (decrease scale)
            // - Scroll down (negative delta) → zoom out (increase scale)
            let delta_zoom = -mouse_wheel_input.delta.y * camera_settings.orthographic_zoom_speed;
            
            // LOGARITHMIC SCALING for intuitive zoom feel
            // Instead of additive changes (scale += delta), we use multiplicative changes
            // This makes zoom feel consistent regardless of current zoom level
            // 
            // Math explanation:
            // - delta_zoom = 0.0 → multiplicative_zoom = 1.0 → no change
            // - delta_zoom = 0.1 → multiplicative_zoom = 1.1 → 10% larger scale
            // - delta_zoom = -0.1 → multiplicative_zoom = 0.9 → 10% smaller scale
            let multiplicative_zoom = 1. + delta_zoom;

            // Apply the zoom and clamp to valid range
            orthographic.scale = (orthographic.scale * multiplicative_zoom).clamp(
                camera_settings.orthographic_zoom_range.start,
                camera_settings.orthographic_zoom_range.end,
            );
        }
        Projection::Perspective(ref mut perspective) => {
            // PERSPECTIVE ZOOM LOGIC
            
            // Again, negate for intuitive scroll direction
            let delta_zoom = -mouse_wheel_input.delta.y * camera_settings.perspective_zoom_speed;

            // ADDITIVE FOV CHANGES work well for perspective
            // FOV has a smaller useful range (typically 30°-120°) so multiplicative
            // changes would be too dramatic. Linear changes feel more natural.
            // 
            // Remember: smaller FOV = more zoomed in, larger FOV = more zoomed out
            perspective.fov = (perspective.fov + delta_zoom).clamp(
                camera_settings.perspective_zoom_range.start,
                camera_settings.perspective_zoom_range.end,
            );
        }
        _ => {
            // Handle custom projections or unsupported types
            // In this example, we just ignore them
        }
    }
}
