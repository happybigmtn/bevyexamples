//! Demonstrates different sub view effects.
//!
//! 🔍 The Magic Window: Understanding Camera Sub Views
//!
//! Imagine you have a massive painting, but you can only look at it through
//! different sized windows that slide around. That's what camera sub views do!
//! They let you show just a portion of what a camera sees, creating effects
//! like security camera monitors, magnifying glasses, or split-screen views.
//!
//! 🎯 What You'll See:
//! - 8 different camera views showing the same 3D scene
//! - Main views: Full perspective and orthographic cameras
//! - Stretched views: Showing how aspect ratios affect the image
//! - Moving views: Sliding windows that pan across the scene
//! - Control views: Properly aspect-ratio corrected partial views
//!
//! 🔑 Key Concepts:
//! - Sub Views: Showing only part of what a camera sees
//! - Viewport: Where on screen to display the camera's output
//! - Aspect Ratios: How stretching occurs when ratios don't match
//! - Perspective vs Orthographic: Two fundamental ways of seeing 3D
//!
//! A sub view is essentially a smaller section of a larger viewport. Some use
//! cases include:
//! - Split one image across multiple cameras, for use in a multimonitor setups
//! - Magnify a section of the image, by rendering a small sub view in another
//!   camera
//! - Rapidly change the sub view offset to get a screen shake effect

use bevy::{
    prelude::*,
    render::camera::{ScalingMode, SubCameraView, Viewport},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (move_camera_view, resize_viewports))
        .run();
}

// 🎬 Marker for cameras that move their sub view
#[derive(Debug, Component)]
struct MovingCameraMarker;

// 🏗️ Scene Setup: Creating Our Test Environment
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 📷 All cameras will share this viewpoint
    let transform = Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y);

    // 🌍 Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // 📦 Test Cube - Our subject
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // 💡 Lighting
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // 🖼️ Camera 1: Main Perspective View
    //
    // This is our reference image - shows the full scene as a normal camera would
    commands.spawn((
        Camera3d::default(),
        Camera::default(),
        ExampleViewports::PerspectiveMain,
        transform,
    ));

    // 🔲 Camera 2: Perspective Right Half (Stretched)
    //
    // Demonstrates what happens when aspect ratios don't match!
    // The sub view captures only the right half of the image (aspect 1:2)
    // but displays it in a square viewport (aspect 1:1), causing horizontal stretching
    commands.spawn((
        Camera3d::default(),
        Camera {
            sub_camera_view: Some(SubCameraView {
                // 📏 Think of these as "virtual pixels" - the actual values don't matter,
                // only their ratios do! Here we have a 10x10 "full image"
                full_size: UVec2::new(10, 10),
                // 📍 Offset of 5 units right = start from the middle
                offset: Vec2::new(5.0, 0.0),
                // 🔍 Size of 5x10 = right half of the image
                size: UVec2::new(5, 10),
            }),
            order: 1,  // Render order (lower = earlier)
            ..default()
        },
        ExampleViewports::PerspectiveStretched,
        transform,
    ));

    // 🎬 Camera 3: Perspective Moving View (Magnified)
    //
    // This creates a "magnifying glass" effect that slides across the scene
    commands.spawn((
        Camera3d::default(),
        Camera {
            sub_camera_view: Some(SubCameraView {
                // 🔍 Large full_size (500x500) with small size (100x100)
                // means we're zoomed in 5x!
                full_size: UVec2::new(500, 500),
                offset: Vec2::ZERO,  // Updated in move_camera_view
                size: UVec2::new(100, 100),
            }),
            order: 2,
            ..default()
        },
        transform,
        ExampleViewports::PerspectiveMoving,
        MovingCameraMarker,  // This camera's view will move!
    ));

    // 🎭 Camera 4: Perspective with Correct Aspect Ratio
    //
    // Shows how to properly display a portion of the image without stretching
    commands.spawn((
        Camera3d::default(),
        Camera {
            sub_camera_view: Some(SubCameraView {
                // 📐 The sub view (800x400 = 2:1 aspect) matches
                // the viewport aspect ratio, preventing distortion
                full_size: UVec2::new(800, 800),
                offset: Vec2::ZERO,
                size: UVec2::new(800, 400),  // Top half of image
            }),
            order: 3,
            ..default()
        },
        ExampleViewports::PerspectiveControl,
        transform,
    ));

    // 📐 Camera 5: Main Orthographic View
    //
    // Orthographic projection has no perspective - like architectural drawings
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            // 🔧 Fixed vertical height means consistent scale regardless of window size
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Camera {
            order: 4,
            ..default()
        },
        ExampleViewports::OrthographicMain,
        transform,
    ));

    // 🔲 Camera 6: Orthographic Left Half (Stretched)
    //
    // Same stretching concept as Camera 2, but with orthographic projection
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Camera {
            sub_camera_view: Some(SubCameraView {
                full_size: UVec2::new(2, 2),
                offset: Vec2::ZERO,
                size: UVec2::new(1, 2),  // Left half
            }),
            order: 5,
            ..default()
        },
        ExampleViewports::OrthographicStretched,
        transform,
    ));

    // 🎬 Camera 7: Orthographic Moving View
    //
    // Sliding window effect with orthographic projection
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Camera {
            sub_camera_view: Some(SubCameraView {
                full_size: UVec2::new(500, 500),
                offset: Vec2::ZERO,
                size: UVec2::new(100, 100),  // 5x zoom
            }),
            order: 6,
            ..default()
        },
        transform,
        ExampleViewports::OrthographicMoving,
        MovingCameraMarker,
    ));

    // 🎭 Camera 8: Orthographic with Correct Aspect Ratio
    commands.spawn((
        Camera3d::default(),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 6.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        Camera {
            sub_camera_view: Some(SubCameraView {
                full_size: UVec2::new(200, 200),
                offset: Vec2::ZERO,
                size: UVec2::new(200, 100),  // Top half, correct aspect
            }),
            order: 7,
            ..default()
        },
        ExampleViewports::OrthographicControl,
        transform,
    ));
}

// 🎬 Animation System: Making the "Magnifying Glass" Move
fn move_camera_view(
    mut movable_camera_query: Query<&mut Camera, With<MovingCameraMarker>>,
    time: Res<Time>,
) {
    for mut camera in movable_camera_query.iter_mut() {
        if let Some(sub_view) = &mut camera.sub_camera_view {
            // 📐 Move diagonally across the image
            // Speed: 150 units/second, wrapping every 3.3 seconds
            sub_view.offset.x = (time.elapsed_secs() * 150.) % 450.0 - 50.0;
            sub_view.offset.y = sub_view.offset.x;
        }
    }
}

// 📐 Viewport Management: Responsive Layout
//
// This system ensures our camera grid maintains proper proportions
// regardless of window size
fn resize_viewports(
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
    mut viewports: Query<(&mut Camera, &ExampleViewports)>,
) {
    let window_size = window.physical_size();

    // 📏 Calculate sizes for our grid layout
    let small_height = window_size.y / 5;
    let small_width = window_size.x / 8;

    let large_height = small_height * 4;
    let large_width = small_width * 4;

    let large_size = UVec2::new(large_width, large_height);

    // 🔲 Force square viewports for the small views
    // This prevents additional distortion from the viewport itself
    let small_dim = small_height.min(small_width);
    let small_size = UVec2::new(small_dim, small_dim);

    // 📐 Wide viewport for aspect-ratio demonstrations
    let small_wide_size = UVec2::new(small_dim * 2, small_dim);

    // 🎯 Position each camera in our grid
    for (mut camera, example_viewport) in viewports.iter_mut() {
        if camera.viewport.is_none() {
            camera.viewport = Some(Viewport::default());
        };

        let Some(viewport) = &mut camera.viewport else {
            continue;
        };

        let (size, position) = match example_viewport {
            // Top row: Small examples
            ExampleViewports::PerspectiveStretched => (small_size, UVec2::ZERO),
            ExampleViewports::PerspectiveMoving => (small_size, UVec2::new(small_width, 0)),
            ExampleViewports::PerspectiveControl => {
                (small_wide_size, UVec2::new(small_width * 2, 0))
            }
            ExampleViewports::OrthographicStretched => (small_size, UVec2::new(small_width * 4, 0)),
            ExampleViewports::OrthographicMoving => (small_size, UVec2::new(small_width * 5, 0)),
            ExampleViewports::OrthographicControl => {
                (small_wide_size, UVec2::new(small_width * 6, 0))
            }
            // Bottom row: Main views
            ExampleViewports::PerspectiveMain => (large_size, UVec2::new(0, small_height)),
            ExampleViewports::OrthographicMain => {
                (large_size, UVec2::new(large_width, small_height))
            }
        };

        viewport.physical_size = size;
        viewport.physical_position = position;
    }
}

// 🏷️ Labels for our different viewport configurations
#[derive(Component)]
enum ExampleViewports {
    PerspectiveMain,
    PerspectiveStretched,
    PerspectiveMoving,
    PerspectiveControl,
    OrthographicMain,
    OrthographicStretched,
    OrthographicMoving,
    OrthographicControl,
}

// 🎓 Deep Dive: Understanding Sub Views
//
// Sub views solve the fundamental problem: "What if I don't want to show
// everything the camera sees?"
//
// The Math:
// 1. Camera renders full image at `full_size` resolution
// 2. We extract a rectangle: position = `offset`, dimensions = `size`
// 3. This rectangle is stretched to fill the viewport
//
// Common Patterns:
// - **Split Screen**: Multiple sub views of the same full image
// - **Picture-in-Picture**: Small sub view in corner
// - **Zoom Effect**: Small `size` relative to `full_size`
// - **Pan Effect**: Animate `offset` over time
// - **Screen Shake**: Rapidly change `offset` randomly

// 💡 Practical Applications:
//
// 1. **Security Cameras**: Multiple views on one monitor
// 2. **Racing Games**: Rear-view mirror as sub view
// 3. **Strategy Games**: Minimap showing full level
// 4. **Horror Games**: Restricted vision through keyhole
// 5. **Multi-Monitor**: Span one camera across displays
// 6. **VR/AR**: Different views for each eye
//
// Performance Tips:
// - Sub views are cheap - no extra rendering!
// - Multiple viewports cost more than sub views
// - Beware of overdraw with overlapping viewports