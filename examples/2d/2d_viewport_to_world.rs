//! This example demonstrates how to use the `Camera::viewport_to_world_2d` method with a dynamic viewport and camera.
//!
//! Viewport-to-world conversion is like translating between two coordinate systems - the screen's
//! pixel coordinates and the game world's spatial coordinates. Imagine you're looking through a
//! telescope (the viewport) at a vast landscape (the world). When you click somewhere on the
//! telescope's lens, you need to figure out exactly what point in the landscape you're pointing at.
//! This example shows that translation in action, with a moveable and resizable viewport!
//!
//! ## Game Design Context: Why Viewport-to-World Matters
//!
//! This conversion is fundamental to many game mechanics:
//! - **Click-to-Move**: Converting mouse clicks to world movement targets
//! - **Drag Selection**: Drawing selection boxes in RTS games
//! - **Aiming Systems**: Translating cursor position to projectile direction
//! - **UI Interaction**: Clicking on world objects through screen space
//! - **Split-Screen**: Multiple viewports showing different world areas
//!
//! The viewport system enables advanced rendering techniques:
//! - **Minimaps**: Small viewport showing zoomed-out world view
//! - **Picture-in-Picture**: Security cameras, rear-view mirrors
//! - **Screen Effects**: Rendering to texture for post-processing
//! - **UI Masking**: Limiting render area for scrollable content
//!
//! ## Rust Fundamentals: Pattern Matching and Error Handling
//!
//! This example showcases several Rust patterns:
//!
//! 1. **Early Returns with Pattern Matching**: The `let else` pattern
//!    ```rust
//!    let Some(value) = option else { return; };
//!    ```
//!    Combines pattern matching with control flow elegantly
//!
//! 2. **Saturating Arithmetic**: Prevents integer overflow/underflow
//!    ```rust
//!    value.saturating_sub(amount)  // Never goes below 0
//!    ```
//!    Safer than regular subtraction for user input
//!
//! 3. **Method Chaining**: Fluent interface pattern
//!    ```rust
//!    viewport.physical_size.min(window_size).max(MIN_SIZE)
//!    ```
//!    Reads like English, applies constraints in order
//!
//! ## Bevy Architecture: Camera and Transform Systems
//!
//! The camera system demonstrates key Bevy concepts:
//!
//! 1. **GlobalTransform vs Transform**: 
//!    - Transform: Local position/rotation/scale
//!    - GlobalTransform: Final world position after hierarchy
//!    - Camera needs GlobalTransform for accurate conversion
//!
//! 2. **System Scheduling**: 
//!    - `PostUpdate` runs after transform propagation
//!    - Ensures GlobalTransform is up-to-date
//!    - Critical for accurate viewport calculations
//!
//! 3. **Resource vs Component**:
//!    - Window is a Resource (one per app)
//!    - Camera is a Component (can have multiple)
//!
//! ## Real-World Applications
//!
//! 1. **Strategy Games**: Click to command units, drag to select
//! 2. **Puzzle Games**: Click and drag pieces in world space
//! 3. **Level Editors**: Place objects at cursor position
//! 4. **Tower Defense**: Place towers where player clicks
//! 5. **Drawing Apps**: Convert stylus input to canvas coordinates
//!
//! ## Performance Considerations
//!
//! 1. **Viewport Changes**: Changing viewport size reallocates framebuffers
//! 2. **Multiple Viewports**: Each viewport is a separate render pass
//! 3. **Coordinate Conversion**: Matrix math per conversion (cache if possible)
//! 4. **Gizmo Rendering**: Debug visuals have performance cost
//!
//! ## Common Pitfalls
//!
//! 1. **Wrong Transform**: Using Transform instead of GlobalTransform
//! 2. **Timing Issues**: Reading position before transform propagation
//! 3. **Viewport Bounds**: Not checking if cursor is inside viewport
//! 4. **Resolution Confusion**: Logical vs physical pixels on high-DPI displays
//! 5. **Z-Order**: Forgetting that 2D still uses Z for layering

use bevy::{
    color::palettes::{
        basic::WHITE,
        css::{GREEN, RED},
    },
    math::ops::powf,
    prelude::*,
    render::camera::Viewport,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, controls)
        .add_systems(PostUpdate, draw_cursor.after(TransformSystems::Propagate))
        .run();
}

fn draw_cursor(
    // BEVY PATTERN: Single<T> query for exactly one result
    // Panics if zero or multiple cameras exist - perfect for examples
    // In production, use Query<T> with .single() for error handling
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    mut gizmos: Gizmos,
) {
    // RUST FUNDAMENTALS: Destructuring and dereferencing
    // Single<T> implements Deref, so * gives us the inner tuple
    // This avoids camera_query.0 and camera_query.1 syntax
    let (camera, camera_transform) = *camera_query;
    
    // RUST PATTERN: Early return with pattern matching
    // If we can't get exactly one window, bail out gracefully
    let Ok(window) = window.single() else {
        return;
    };

    // Get mouse position in screen pixels
    // GAME DESIGN: cursor_position() returns None when cursor is outside window
    // This is important for fullscreen games where cursor can leave bounds
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // THE MAGIC TRANSLATION - Screen pixels to world coordinates!
    // Calculate a world position based on the cursor's position.
    // This accounts for camera position, zoom, and viewport settings
    //
    // MATH BREAKDOWN: What happens inside viewport_to_world_2d?
    // 1. Adjust cursor position relative to viewport origin
    // 2. Convert to normalized device coordinates (NDC): -1 to 1
    // 3. Apply inverse projection matrix (undo camera zoom)
    // 4. Apply inverse view matrix (undo camera position/rotation)
    // 5. Result: world space position where ray intersects Z=0 plane
    //
    // BEVY ARCHITECTURE: Why can this fail (return Err)?
    // - Cursor outside viewport bounds
    // - Invalid camera projection settings
    // - Numerical instability in matrix inversion
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // ROUND-TRIP TEST - Like measuring twice to cut once!
    // To test Camera::world_to_viewport, convert result back to viewport space and then back to world space.
    // world -> viewport -> world should give us the same position (within floating point precision)
    let Ok(viewport_check) = camera.world_to_viewport(camera_transform, world_pos.extend(0.0))
    else {
        return;
    };
    let Ok(world_check) = camera.viewport_to_world_2d(camera_transform, viewport_check.xy()) else {
        return;
    };

    // White circle shows where the cursor is in world space
    gizmos.circle_2d(world_pos, 10., WHITE);
    // Red circle shows the round-trip result - should overlap perfectly!
    // If they don't match, we have a conversion bug
    gizmos.circle_2d(world_check, 8., RED);
}

fn controls(
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
    window: Query<&Window>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Fixed>>,  // Fixed timestep for consistent movement
) {
    let Ok(window) = window.single() else {
        return;
    };
    let Ok((mut camera, mut transform, mut projection)) = camera_query.single_mut() else {
        return;
    };
    // Speed calculations - frame-rate independent movement!
    // GAME DESIGN: Why use Fixed time?
    // - Consistent physics and movement regardless of framerate
    // - Prevents speed variations on different hardware
    // - Makes networked games deterministic
    //
    // RUST FUNDAMENTALS: Type casting with 'as'
    // - fspeed as u32 truncates decimal (floor operation)
    // - Important: can overflow if fspeed > u32::MAX (unlikely here)
    // - Alternative: fspeed.round() as u32 for nearest integer
    let fspeed = 600.0 * time.delta_secs();  // Float speed for smooth camera movement
    let uspeed = fspeed as u32;              // Integer speed for pixel-perfect viewport movement
    let window_size = window.resolution.physical_size();

    // CAMERA MOVEMENT - Moving the observer through the world
    // Like panning a telescope across the sky
    if input.pressed(KeyCode::ArrowUp) {
        transform.translation.y += fspeed;  // Positive Y is up in Bevy 2D
    }
    if input.pressed(KeyCode::ArrowDown) {
        transform.translation.y -= fspeed;
    }
    if input.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= fspeed;  // Negative X is left
    }
    if input.pressed(KeyCode::ArrowRight) {
        transform.translation.x += fspeed;
    }

    // CAMERA ZOOM - Changing the magnification level
    // Orthographic projection = no perspective distortion (perfect for 2D)
    //
    // RUST FUNDAMENTALS: Pattern matching with mutable references
    // - &mut *projection dereferences then re-borrows mutably
    // - if let only executes block if pattern matches
    // - projection2d is a mutable borrow of the inner OrthographicProjection
    //
    // GAME DESIGN: Exponential vs Linear Zoom
    // - Linear: scale += 0.1 feels fast when zoomed in, slow when zoomed out
    // - Exponential: scale *= 1.1 feels consistent at any zoom level
    // - This is why camera controls often use log scale!
    if let Projection::Orthographic(projection2d) = &mut *projection {
        // Exponential zoom for smooth scaling at any zoom level
        // powf ensures zoom speed feels consistent whether zoomed in or out
        //
        // MATH: Why these specific numbers?
        // - 4.0 means 4x zoom per second (holding key)
        // - 0.25 is reciprocal (1/4) for opposite direction
        // - delta_secs makes it frame-rate independent
        // - powf(base, exponent) = base^exponent
        if input.pressed(KeyCode::Comma) {
            projection2d.scale *= powf(4.0f32, time.delta_secs());  // Zoom out
        }

        if input.pressed(KeyCode::Period) {
            projection2d.scale *= powf(0.25f32, time.delta_secs()); // Zoom in (1/4 = inverse of 4)
        }
    }

    if let Some(viewport) = camera.viewport.as_mut() {
        // VIEWPORT MOVEMENT - Moving the render target within the window
        // Like sliding a picture frame around on a wall
        if input.pressed(KeyCode::KeyW) {
            // saturating_sub prevents underflow (going below 0)
            viewport.physical_position.y = viewport.physical_position.y.saturating_sub(uspeed);
        }
        if input.pressed(KeyCode::KeyS) {
            viewport.physical_position.y += uspeed;
        }
        if input.pressed(KeyCode::KeyA) {
            viewport.physical_position.x = viewport.physical_position.x.saturating_sub(uspeed);
        }
        if input.pressed(KeyCode::KeyD) {
            viewport.physical_position.x += uspeed;
        }

        // BOUNDARY CHECKING - Keep viewport inside window
        // Viewport can't extend past window edges
        viewport.physical_position = viewport
            .physical_position
            .min(window_size - viewport.physical_size);

        // VIEWPORT RESIZING - Changing the render area size
        // Like adjusting the aperture of a camera lens
        if input.pressed(KeyCode::KeyI) {
            viewport.physical_size.y = viewport.physical_size.y.saturating_sub(uspeed);
        }
        if input.pressed(KeyCode::KeyK) {
            viewport.physical_size.y += uspeed;
        }
        if input.pressed(KeyCode::KeyJ) {
            viewport.physical_size.x = viewport.physical_size.x.saturating_sub(uspeed);
        }
        if input.pressed(KeyCode::KeyL) {
            viewport.physical_size.x += uspeed;
        }

        // SIZE CONSTRAINTS - Viewport must fit in window and have minimum size
        viewport.physical_size = viewport
            .physical_size
            .min(window_size - viewport.physical_position)  // Can't extend past window edge
            .max(UVec2::new(20, 20));                      // Minimum 20x20 pixels
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let window_size = window.resolution.physical_size().as_vec2();

    // CUSTOM VIEWPORT SETUP - Not using the full window!
    // Initialize centered, non-window-filling viewport
    // This creates a "window within a window" effect
    //
    // GAME DESIGN: Why custom viewports?
    // - Split-screen multiplayer (2-4 viewports)
    // - Minimap in corner (small viewport)
    // - Cinematic letterboxing (black bars)
    // - Picture-in-picture (security cameras)
    // - UI safe zones (avoiding screen edges)
    //
    // BEVY ARCHITECTURE: Camera component composition
    // - Camera2d: Marker component + bundles common 2D settings
    // - Camera: The actual rendering configuration
    // - Transform: Position/rotation (added by Camera2d bundle)
    // - Projection: How 3D world maps to 2D screen (added by bundle)
    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(Viewport {
                // Start at 12.5% from top-left corner
                // MATH: 0.125 = 1/8, so 1/8 margin on each side
                // This centers a 3/4 sized viewport (1/8 + 3/4 + 1/8 = 1)
                physical_position: (window_size * 0.125).as_uvec2(),
                // Take up 75% of window width and height
                // RUST: as_uvec2() converts Vec2 (f32) to UVec2 (u32)
                // Truncates decimal parts - be careful with fractional pixels!
                physical_size: (window_size * 0.75).as_uvec2(),
                ..default()
            }),
            ..default()
        },
    ));

    // Create a minimal UI explaining how to interact with the example
    commands.spawn((
        Text::new(
            "Move the mouse to see the circle follow your cursor.\n\
                    Use the arrow keys to move the camera.\n\
                    Use the comma and period keys to zoom in and out.\n\
                    Use the WASD keys to move the viewport.\n\
                    Use the IJKL keys to resize the viewport.",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    // REFERENCE OBJECT - Green rectangle at world origin
    // Add mesh to make camera movement visible
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(40.0, 20.0))),
        MeshMaterial2d(materials.add(Color::from(GREEN))),
    ));

    // BACKGROUND PLANE - Huge dark rectangle
    // Add background to visualize viewport bounds
    // The dark area shows what's outside the viewport
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(50000.0, 50000.0))),  // Massive size
        MeshMaterial2d(materials.add(Color::linear_rgb(0.01, 0.01, 0.01))),  // Almost black
        Transform::from_translation(Vec3::new(0.0, 0.0, -200.0)),  // Far behind other objects
    ));
}
