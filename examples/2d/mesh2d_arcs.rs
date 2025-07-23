//! Demonstrates UV mappings of the [`CircularSector`] and [`CircularSegment`] primitives.
//!
//! Also draws the bounding boxes and circles of the primitives.
//!
//! This example teaches an important concept in computer graphics: UV mapping.
//! When we apply a texture (like an image) to a 3D or 2D shape, we need to tell
//! the computer which part of the texture goes where on the shape. UV coordinates
//! are like a map that says "this corner of my shape corresponds to this pixel
//! in my image."
//!
//! U and V are just names for the X and Y axes in texture space (we use different
//! letters to avoid confusion with world space X,Y,Z). UV coordinates typically
//! range from 0 to 1, where (0,0) is the bottom-left of the texture and (1,1)
//! is the top-right.
//!
//! For circular shapes like sectors (pie slices) and segments (circle with chord
//! cut off), UV mapping is tricky because we're mapping a rectangular image onto
//! a curved surface. This example shows how to control that mapping.

// FRAC_PI_2 is π/2 radians, which equals 90 degrees. Rust provides these
// fractional constants for common angles to avoid computation and rounding errors.
use std::f32::consts::FRAC_PI_2;

use bevy::{
    // CSS color palette gives us web-standard named colors
    color::palettes::css::{BLUE, GRAY, RED},
    math::{
        // Bounding volumes are invisible shapes that fully contain our object.
        // They're used for collision detection, culling, and spatial queries.
        // Think of them like a cardboard box that perfectly fits around an oddly-shaped gift.
        bounding::{Bounded2d, BoundingVolume},
        // Isometry is a mathematical term for transformations that preserve distance.
        // In 2D, this means translation (moving) and rotation, but not scaling.
        // It's like sliding and spinning a paper on a table without stretching it.
        Isometry2d,
    },
    prelude::*,
    // These builders help us create meshes with specific UV mapping strategies
    render::mesh::{CircularMeshUvMode, CircularSectorMeshBuilder, CircularSegmentMeshBuilder},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // Generic functions in Rust! The ::<Type> syntax explicitly specifies
                // which type to use for the generic parameter. It's like telling
                // a recipe "make this dish with chicken" vs "make this dish with tofu".
                draw_bounds::<CircularSector>,
                draw_bounds::<CircularSegment>,
            ),
        )
        .run();
}

// Component is a trait that marks this struct as storable on entities.
// Debug allows us to print the struct for debugging.
// This is a generic component that can work with any shape that implements Bounded2d.
// The trait bounds ensure the shape can be:
// - Bounded2d: has bounding volume calculations
// - Send + Sync: can be safely shared between threads (required for Bevy systems)
// - 'static: has no non-static references (lives for entire program)
// The tuple struct syntax MyStruct(Type) creates a struct with one unnamed field.
#[derive(Component, Debug)]
struct DrawBounds<Shape: Bounded2d + Send + Sync + 'static>(Shape);

fn setup(
    mut commands: Commands,
    // AssetServer loads files from disk. It returns handles immediately,
    // even before the file is loaded, allowing async loading.
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Load the Bevy logo image and create a material from it.
    // The material is what gets applied to our meshes to give them appearance.
    let material = materials.add(asset_server.load("branding/icon.png"));

    // Spawn a camera with a custom gray background
    commands.spawn((
        Camera2d,
        Camera {
            // Instead of default black, we use a custom clear color
            clear_color: ClearColorConfig::Custom(GRAY.into()),
            // ..default() fills in remaining fields with their Default values
            ..default()
        },
    ));

    // Constants for layout. Using constants makes it easy to adjust the visualization.
    const NUM_SLICES: i32 = 8;  // How many shapes to draw
    const SPACING_X: f32 = 100.0;  // Pixels between each shape
    // Calculate offset to center the row of shapes around x=0
    const OFFSET_X: f32 = SPACING_X * (NUM_SLICES - 1) as f32 / 2.0;

    // This draws NUM_SLICES copies of the Bevy logo as circular sectors and segments,
    // with successively larger angles up to a complete circle.
    for i in 0..NUM_SLICES {
        // Calculate what fraction of a full circle this iteration represents
        // i+1 because we want fractions from 1/8 to 8/8, not 0/8 to 7/8
        let fraction = (i + 1) as f32 / NUM_SLICES as f32;

        // Create a circular sector (pie slice) with radius 40 and the calculated angle.
        // from_turns() takes the fraction of a full rotation (1.0 = 360°)
        let sector = CircularSector::from_turns(40.0, fraction);
        // We want to rotate the circular sector so that the sectors appear clockwise from north.
        // We must rotate it both in the Transform and in the mesh's UV mappings.
        // half_angle() returns half the sector's arc angle - we negate to rotate clockwise
        let sector_angle = -sector.half_angle();
        // Build the mesh with UV mapping. The Mask mode means we're showing only
        // part of the texture, rotated by the given angle.
        let sector_mesh =
            CircularSectorMeshBuilder::new(sector).uv_mode(CircularMeshUvMode::Mask {
                angle: sector_angle,
            });
        commands.spawn((
            Mesh2d(meshes.add(sector_mesh)),
            MeshMaterial2d(material.clone()),
            Transform {
                // Position shapes in a horizontal row, centered at x=0
                translation: Vec3::new(SPACING_X * i as f32 - OFFSET_X, 50.0, 0.0),
                // Rotate the shape to match the UV rotation
                rotation: Quat::from_rotation_z(sector_angle),
                ..default()
            },
            // Attach our debug component to draw bounds later
            DrawBounds(sector),
        ));

        // Similar process for circular segments (circle with a chord cut off)
        let segment = CircularSegment::from_turns(40.0, fraction);
        // For the circular segment, we will draw Bevy charging forward, which requires rotating the
        // shape and texture by 90 degrees.
        //
        // Note that this may be unintuitive; it may feel like we should rotate the texture by the
        // opposite angle to preserve the orientation of Bevy. But the angle is not the angle of the
        // texture itself, rather it is the angle at which the vertices are mapped onto the texture.
        // so it is the negative of what you might otherwise expect.
        let segment_angle = -FRAC_PI_2;  // -90 degrees
        let segment_mesh =
            CircularSegmentMeshBuilder::new(segment).uv_mode(CircularMeshUvMode::Mask {
                angle: -segment_angle,
            });
        commands.spawn((
            Mesh2d(meshes.add(segment_mesh)),
            MeshMaterial2d(material.clone()),
            Transform {
                // Place segments in a row below the sectors
                translation: Vec3::new(SPACING_X * i as f32 - OFFSET_X, -50.0, 0.0),
                rotation: Quat::from_rotation_z(segment_angle),
                ..default()
            },
            DrawBounds(segment),
        ));
    }
}

// Generic system that draws bounding volumes for any shape type.
// This demonstrates Rust's powerful generics - we write this once and it works
// for both CircularSector and CircularSegment (or any other Bounded2d shape).
fn draw_bounds<Shape: Bounded2d + Send + Sync + 'static>(
    // Query for entities with our DrawBounds component and their global position
    q: Query<(&DrawBounds<Shape>, &GlobalTransform)>,
    // Gizmos are Bevy's immediate-mode drawing API for debug visualizations
    mut gizmos: Gizmos,
) {
    for (shape, transform) in &q {
        // Decompose the transform into its components
        // We ignore scale (first return value) since Isometry2d doesn't support it
        let (_, rotation, translation) = transform.to_scale_rotation_translation();
        // Convert 3D translation to 2D by dropping the Z component
        let translation = translation.truncate();
        // Extract just the Z-axis rotation (2D rotation in 3D space)
        // to_euler returns (x_rot, y_rot, z_rot), we want the third
        let rotation = rotation.to_euler(EulerRot::XYZ).2;
        // Create an isometry (position + rotation) for the bounding calculations
        let isometry = Isometry2d::new(translation, Rot2::radians(rotation));

        // Calculate and draw the axis-aligned bounding box (AABB)
        // An AABB is always aligned with the world axes, not rotated with the shape
        let aabb = shape.0.aabb_2d(isometry);
        // Draw a red rectangle. Note: half_size * 2.0 because rect_2d wants full size
        gizmos.rect_2d(aabb.center(), aabb.half_size() * 2.0, RED);

        // Calculate and draw the bounding circle
        // This is the smallest circle that completely contains the shape
        let bounding_circle = shape.0.bounding_circle(isometry);
        gizmos.circle_2d(bounding_circle.center, bounding_circle.radius(), BLUE);
    }
}
