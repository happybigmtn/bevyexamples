//! Demonstrates how to define and use custom camera projections.
//!
//! ## Key Concepts Demonstrated
//!
//! - **Custom Projection Implementation**: Creating new camera projection types
//! - **CameraProjection Trait**: Understanding the core interface for camera projections
//! - **Matrix Mathematics**: How projection matrices transform 3D to 2D coordinates
//! - **Oblique Projections**: Creating perspective projections with offset vanishing points
//! - **Frustum Calculations**: Understanding camera view volumes
//!
//! ## What is a Camera Projection?
//!
//! A camera projection defines how 3D world coordinates are transformed into 2D screen coordinates.
//! This transformation involves two main steps:
//! 1. View transformation: Convert world space to camera space
//! 2. Projection transformation: Convert camera space to clip space (this example focuses on step 2)
//!
//! ## Oblique Perspective Projection
//!
//! This example implements an oblique perspective projection, which is like a normal perspective
//! projection but with the vanishing point offset from the center. This creates a skewed effect
//! that can be useful for architectural visualization or artistic effects.

use bevy::prelude::*;
// CameraProjection is the trait that defines how cameras transform coordinates
use bevy::render::camera::CameraProjection;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

/// A custom camera projection that creates an oblique perspective effect
/// 
/// Unlike standard perspective projection where the vanishing point is centered,
/// oblique perspective shifts the vanishing point, creating a skewed appearance.
/// This is useful for architectural drawings, isometric-style effects, or
/// simulating viewing through a tilted window.
#[derive(Debug, Clone)]
struct ObliquePerspectiveProjection {
    /// How much to skew the projection horizontally (-1.0 to 1.0)
    /// - 0.0 = no horizontal skew (normal perspective)
    /// - Positive values skew to the right
    /// - Negative values skew to the left
    horizontal_obliqueness: f32,
    
    /// How much to skew the projection vertically (-1.0 to 1.0)
    /// - 0.0 = no vertical skew (normal perspective)
    /// - Positive values skew upward
    /// - Negative values skew downward
    vertical_obliqueness: f32,
    
    /// The underlying perspective projection that provides the base transformation
    /// We modify this rather than implementing perspective math from scratch
    perspective: PerspectiveProjection,
}

/// Implementation of the CameraProjection trait for our custom oblique projection
/// 
/// The CameraProjection trait defines the interface that all camera projections must implement.
/// This allows Bevy's rendering system to work with any projection type, from standard
/// perspective and orthographic projections to completely custom ones like this.
impl CameraProjection for ObliquePerspectiveProjection {
    /// Returns the projection matrix that transforms view-space coordinates to clip-space
    /// 
    /// This is the core method that defines how our projection works. We start with a
    /// standard perspective projection matrix and then modify it to add oblique effects.
    fn get_clip_from_view(&self) -> Mat4 {
        // Start with the standard perspective projection matrix
        let mut mat = self.perspective.get_clip_from_view();
        
        // MODIFY THE PROJECTION MATRIX FOR OBLIQUE EFFECT
        // The projection matrix is a 4x4 matrix where:
        // - Column 0: Controls X scaling and shearing
        // - Column 1: Controls Y scaling and shearing  
        // - Column 2: Controls Z depth and oblique effects (what we're modifying)
        // - Column 3: Controls perspective division and translation
        
        // mat.col_mut(2) gets a mutable reference to the third column (index 2)
        // This column controls how Z coordinates affect X and Y positions
        
        // [0] = X component: how much Z depth affects horizontal position
        mat.col_mut(2)[0] = self.horizontal_obliqueness;
        
        // [1] = Y component: how much Z depth affects vertical position  
        mat.col_mut(2)[1] = self.vertical_obliqueness;
        
        mat
    }

    /// Returns the projection matrix for sub-camera views (like split-screen or picture-in-picture)
    /// 
    /// This is similar to get_clip_from_view but handles cases where the camera only renders
    /// to part of the screen. We apply the same oblique modifications.
    fn get_clip_from_view_for_sub(&self, sub_view: &bevy::render::camera::SubCameraView) -> Mat4 {
        let mut mat = self.perspective.get_clip_from_view_for_sub(sub_view);
        
        // Apply the same oblique modifications as in the main projection
        mat.col_mut(2)[0] = self.horizontal_obliqueness;
        mat.col_mut(2)[1] = self.vertical_obliqueness;
        
        mat
    }

    /// Updates the projection when the window size changes
    /// 
    /// When the user resizes the window, we need to update our projection to maintain
    /// the correct aspect ratio. We delegate this to the underlying perspective projection.
    fn update(&mut self, width: f32, height: f32) {
        self.perspective.update(width, height);
    }

    /// Returns the far clipping plane distance
    /// 
    /// Objects farther than this distance from the camera won't be rendered.
    /// We delegate this to the underlying perspective projection.
    fn far(&self) -> f32 {
        self.perspective.far
    }

    /// Returns the eight corners of the view frustum (the pyramid-shaped volume the camera can see)
    /// 
    /// This is used for frustum culling - determining which objects are within the camera's
    /// view so we don't waste time rendering invisible objects.
    /// 
    /// Parameters:
    /// - z_near: Distance to the near clipping plane
    /// - z_far: Distance to the far clipping plane
    /// 
    /// Returns: Array of 8 Vec3A points representing the frustum corners
    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        // For simplicity, we use the underlying perspective projection's frustum
        // In a more sophisticated implementation, you might modify these corners
        // to account for the oblique effect
        self.perspective.get_frustum_corners(z_near, z_far)
    }
}

/// Sets up the scene with a custom oblique perspective camera and some test geometry
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // CAMERA SETUP WITH CUSTOM PROJECTION
    commands.spawn((
        Camera3d::default(),
        
        // Use our custom oblique perspective projection instead of the default
        // Projection::custom() tells Bevy to use our custom implementation
        Projection::custom(ObliquePerspectiveProjection {
            // These values create a noticeable oblique effect
            // Try changing these values to see different effects:
            horizontal_obliqueness: 0.2,  // Slight rightward skew
            vertical_obliqueness: 0.6,    // More pronounced upward skew
            
            // Use default perspective projection settings for everything else
            perspective: PerspectiveProjection::default(),
        }),
        
        // Position the camera to get a good view of our test scene
        // looking_at() points the camera at the origin (0,0,0) with Y as the up direction
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // SCENE GEOMETRY: Simple objects to demonstrate the projection effect
    
    // Ground plane: A horizontal circle that shows the oblique distortion
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        
        // Rotate 90 degrees around X-axis to make it horizontal (ground plane)
        // FRAC_PI_2 is π/2 radians = 90 degrees
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    
    // Test cube: A cube that will show the oblique effect clearly
    // The cube's parallel edges will appear to converge to different vanishing points
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),  // Light blue
        
        // Position it above the ground plane
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    
    // Lighting: A point light to illuminate the scene and cast shadows
    commands.spawn((
        PointLight {
            shadows_enabled: true,  // Enable shadow casting for more visual depth
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),  // Position above and to the side
    ));
}
