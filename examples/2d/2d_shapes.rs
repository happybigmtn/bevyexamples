//! Here we use shape primitives to build meshes in a 2D rendering context, making each mesh a certain color by giving that mesh's entity a material based off a [`Color`].
//!
//! Meshes are better known for their use in 3D rendering, but we can use them in a 2D context too. Without a third dimension, the meshes we're building are flat – like paper on a table. These are still very useful for "vector-style" graphics, picking behavior, or as a foundation to build off of for where to apply a shader.
//!
//! A "shape definition" is not a mesh on its own. A circle can be defined with a radius, i.e. [`Circle::new(50.0)`][Circle::new], but rendering tends to happen with meshes built out of triangles. So we need to turn shape descriptions into meshes.
//!
//! Thankfully, we can add shape primitives directly to [`Assets<Mesh>`] because [`Mesh`] implements [`From`] for shape primitives and [`Assets<T>::add`] can be given any value that can be "turned into" `T`!
//!
//! We apply a material to the shape by first making a [`Color`] then calling [`Assets<ColorMaterial>::add`] with that color as its argument, which will create a material from that color through the same process [`Assets<Mesh>::add`] can take a shape primitive.
//!
//! Both the mesh and material need to be wrapped in their own "newtypes". The mesh and material are currently [`Handle<Mesh>`] and [`Handle<ColorMaterial>`] at the moment, which are not components. Handles are put behind "newtypes" to prevent ambiguity, as some entities might want to have handles to meshes (or images, or materials etc.) for different purposes! All we need to do to make them rendering-relevant components is wrap the mesh handle and the material handle in [`Mesh2d`] and [`MeshMaterial2d`] respectively.
//!
//! You can toggle wireframes with the space bar except on wasm. Wasm does not support
//! `POLYGON_MODE_LINE` on the gpu.
//!
//! This example teaches a fundamental principle: in computer graphics, EVERYTHING is triangles.
//! Even a "circle" is really a polygon with many sides. It's like how calculus approximates
//! curves with infinitesimal straight lines - we approximate shapes with triangles.
//!
//! ## Game Design Context: The Power of Primitive Shapes
//!
//! Primitive shapes are the building blocks of 2D game art. Think of classic games:
//! - Pong: rectangles
//! - Asteroids: triangles and lines
//! - Pac-Man: circles and circular sectors
//!
//! Why primitives matter in game design:
//! 1. **Performance**: Mathematical shapes render faster than textured sprites
//! 2. **Scalability**: Vector graphics scale perfectly at any resolution
//! 3. **Style**: Minimalist aesthetics can be more engaging than realistic graphics
//! 4. **Prototyping**: Quick iteration without needing an artist
//! 5. **Visual Clarity**: Clean shapes communicate gameplay mechanics clearly
//!
//! ## Rust Fundamentals: Type Safety and Zero-Cost Abstractions
//!
//! This example demonstrates several key Rust concepts:
//!
//! 1. **Newtype Pattern**: `Mesh2d(Handle<Mesh>)` wraps a handle to add semantic meaning
//!    - Prevents mixing 2D and 3D meshes at compile time
//!    - Zero runtime cost - the wrapper disappears after compilation
//!
//! 2. **Builder Pattern with Defaults**: `..default()` struct update syntax
//!    - Explicitly set what you care about, implicitly use sensible defaults
//!    - Prevents bugs from forgetting to initialize fields
//!
//! 3. **Conditional Compilation**: `#[cfg(not(target_arch = "wasm32"))]`
//!    - Different code paths for different platforms
//!    - Checked at compile time, no runtime overhead
//!
//! ## Bevy Architecture: The Asset System
//!
//! The `Assets<T>` system is Bevy's centralized resource management:
//!
//! 1. **Handle-based Architecture**: Instead of storing mesh data in components, we store handles
//!    - Enables asset sharing (multiple entities can use the same mesh)
//!    - Allows hot-reloading and async loading
//!    - Reduces memory usage through deduplication
//!
//! 2. **Type-driven Storage**: `Assets<Mesh>` and `Assets<ColorMaterial>` are separate
//!    - Each asset type gets its own optimized storage
//!    - Type safety prevents mixing incompatible assets
//!
//! ## Real-World Applications
//!
//! Use primitive shapes for:
//! - **UI Elements**: Buttons, progress bars, health indicators
//! - **Particle Systems**: Sparks, explosions, magic effects
//! - **Debug Visualization**: Collision boxes, pathfinding grids
//! - **Procedural Generation**: Dungeons, landscapes, abstract patterns
//! - **Mobile Games**: Lower memory footprint and battery usage
//!
//! ## Performance Considerations
//!
//! 1. **Mesh Complexity**: More vertices = more GPU work
//!    - Circle with 32 segments vs 256 segments
//!    - Balance visual quality with performance needs
//!
//! 2. **Batching**: Shapes with the same material can be batched
//!    - Using different colors prevents batching
//!    - Consider using vertex colors for variation
//!
//! 3. **Memory Layout**: Bevy stores all instances of an asset type contiguously
//!    - Better cache locality when iterating over entities
//!    - Handle indirection has small cost vs storing data directly
//!
//! ## Common Pitfalls
//!
//! 1. **Forgetting Mesh2d/MeshMaterial2d wrappers**: Raw handles aren't components!
//! 2. **Z-fighting**: When shapes overlap at Z=0, flickering occurs
//! 3. **Tessellation assumptions**: Different shapes have different vertex counts
//! 4. **Material leaks**: Creating new materials every frame instead of reusing
//! 5. **Coordinate confusion**: 2D uses screen space, not world space by default

use bevy::prelude::*;
// Platform-specific code! WebAssembly (wasm) has limitations - it's like trying to do
// certain physics experiments in space vs on Earth. Some things just work differently.
#[cfg(not(target_arch = "wasm32"))]
use bevy::sprite::{Wireframe2dConfig, Wireframe2dPlugin};

fn main() {
    // Sometimes we need to build our app step by step, like assembling a complex machine.
    // We store it in a variable first, then conditionally add parts based on the platform.
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        // Conditional compilation! This plugin only exists on native platforms.
        // It's like having different equipment available in different laboratories.
        #[cfg(not(target_arch = "wasm32"))]
        Wireframe2dPlugin::default(),
    ))
    .add_systems(Startup, setup);
    
    // Only add the wireframe toggle system on platforms that support it.
    // This is defensive programming - like checking if your equipment can handle
    // a certain experiment before attempting it.
    #[cfg(not(target_arch = "wasm32"))]
    app.add_systems(Update, toggle_wireframe);
    
    app.run();
}

// Constants are like physical constants - they don't change during runtime.
// This defines how wide our "stage" is for displaying shapes.
const X_EXTENT: f32 = 900.;

fn setup(
    mut commands: Commands,
    // ResMut means "mutable resource" - we need write access to add new meshes.
    // Assets<Mesh> is like a library catalog - it stores all our mesh data.
    // Think of it as a warehouse where we store geometric blueprints.
    //
    // RUST FUNDAMENTALS: ResMut<T> demonstrates Rust's borrowing rules in action:
    // - Only ONE system can have ResMut<T> at a time (exclusive access)
    // - Multiple systems can have Res<T> (shared read access)
    // - This prevents data races at compile time - no mutex needed!
    //
    // BEVY ARCHITECTURE: Resources vs Components
    // - Resources (Res/ResMut) are global singletons - one instance in the whole app
    // - Components are per-entity data - each entity has its own copy
    // - Assets are resources because we want to share mesh data between entities
    mut meshes: ResMut<Assets<Mesh>>,
    // Same idea but for materials - this is our "paint storage".
    // ColorMaterial is the simplest material - just a solid color.
    //
    // GAME DESIGN: Why separate meshes and materials?
    // - One mesh (shape) can have many materials (colors/textures)
    // - Example: Same circle mesh for coins (gold), health (red), mana (blue)
    // - Saves memory and allows dynamic visual changes
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Always need our observer (camera) to see the experiment!
    commands.spawn(Camera2d);

    // Here's our shape gallery! Each shape is a different geometric primitive.
    // When we call meshes.add(), we're doing two things:
    // 1. Converting the shape description into actual triangles (tessellation)
    // 2. Storing it in the Assets system and getting back a Handle (like a library card number)
    //
    // RUST FUNDAMENTALS: Array initialization and type inference
    // - Rust infers this is [Handle<Mesh>; 10] from the first element
    // - All elements must be the same type (homogeneous array)
    // - The array size is known at compile time (stack allocated)
    let shapes = [
        // Circle: defined by radius. More sides = smoother circle (Bevy chooses automatically)
        // GAME DESIGN: Circles are perfect for: coins, bullets, shields, detection ranges
        // PERFORMANCE: Default is ~32 vertices. For tiny circles, this might be overkill!
        meshes.add(Circle::new(50.0)),
        
        // CircularSector: like a pie slice. Second parameter is the angle in radians.
        // GAME DESIGN: Perfect for: pie charts, radar sweeps, vision cones, ability cooldowns
        // MATH: 1.0 radian ≈ 57.3 degrees. Full circle = 2π radians ≈ 6.28
        meshes.add(CircularSector::new(50.0, 1.0)),
        
        // CircularSegment: circle with a chord cut off. Second parameter is the chord length.
        // GAME DESIGN: Useful for: health bars, loading indicators, partial shields
        // GEOMETRY: Creates a "lens" shape - two circular arcs meeting at points
        meshes.add(CircularSegment::new(50.0, 1.25)),
        
        // Ellipse: stretched circle. Two radii for width and height.
        // GAME DESIGN: Great for: shadows, portals, stretched UI elements, oval tracks
        // VISUAL DESIGN: Ellipses feel more organic and less mechanical than circles
        meshes.add(Ellipse::new(25.0, 50.0)),
        
        // Annulus: donut shape. Inner radius and outer radius.
        // GAME DESIGN: Perfect for: ring effects, orbits, selection indicators, shockwaves
        // RUST PATTERN: Constructor validates inner < outer radius (type safety!)
        meshes.add(Annulus::new(25.0, 50.0)),
        
        // Capsule2d: pill shape. Radius and length.
        // GAME DESIGN: Ideal for: characters, pills, rounded buttons, speech bubbles
        // COLLISION: Capsules are great for character physics - no sharp corners!
        meshes.add(Capsule2d::new(25.0, 50.0)),
        
        // Rhombus: diamond shape. Half-width and half-height.
        // GAME DESIGN: Use for: gems, diamonds, directional indicators, kites
        // VISUAL HIERARCHY: Points create visual tension and draw the eye
        meshes.add(Rhombus::new(75.0, 100.0)),
        
        // Rectangle: the classic. Width and height.
        // GAME DESIGN: The workhorse - platforms, walls, buttons, text boxes
        // PERFORMANCE: Only 4 vertices! Most efficient shape after triangles
        meshes.add(Rectangle::new(50.0, 100.0)),
        
        // RegularPolygon: N-sided shape. Radius and number of sides (hexagon here).
        // GAME DESIGN: Hexagons are trendy! Perfect for: tiles, shields, sci-fi UI
        // MATH: Interior angle = (n-2) × 180° / n. Hexagon = 120° per angle
        meshes.add(RegularPolygon::new(50.0, 6)),
        
        // Triangle2d: defined by three points. Vec2::Y * 50.0 means (0, 50).
        // This creates an isoceles triangle pointing up.
        // RUST FUNDAMENTALS: Vec2::Y is a const unit vector (0, 1)
        // Multiplying by scalar scales the vector: (0, 1) × 50 = (0, 50)
        // GAME DESIGN: Triangles suggest: direction, danger, mountains, arrows
        meshes.add(Triangle2d::new(
            Vec2::Y * 50.0,          // Top vertex - using const multiplication
            Vec2::new(-50.0, -50.0), // Bottom left - explicit construction
            Vec2::new(50.0, -50.0),  // Bottom right - forms base of triangle
        )),
    ];
    let num_shapes = shapes.len();

    for (i, shape) in shapes.into_iter().enumerate() {
        // Color theory in action! HSL = Hue, Saturation, Lightness
        // Hue goes from 0-360 degrees (red -> yellow -> green -> blue -> purple -> red)
        // We distribute our shapes evenly across this spectrum.
        // It's like spreading light through a prism to see all colors!
        //
        // GAME DESIGN: Color Psychology in Games
        // - Red: danger, health, aggression, urgency
        // - Orange: energy, enthusiasm, caution
        // - Yellow: happiness, coins, alerts, electricity
        // - Green: nature, healing, safety, poison
        // - Blue: calm, water, ice, technology
        // - Purple: magic, mystery, royalty, corruption
        //
        // RUST FUNDAMENTALS: into_iter() consumes the array
        // - We can't use 'shapes' after this (ownership transferred)
        // - enumerate() adds index to each item: (0, item0), (1, item1)...
        // - This is zero-cost - no runtime overhead vs manual indexing
        let color = Color::hsl(
            360. * i as f32 / num_shapes as f32, // Hue: position in rainbow
            0.95,                                 // Saturation: how pure (0=gray, 1=pure)
            0.7,                                  // Lightness: how bright (0=black, 1=white)
        );

        // Time to spawn our entity! Notice the tuple syntax - we're adding multiple
        // components at once. It's like describing an object's properties all at once.
        //
        // BEVY ARCHITECTURE: Entity-Component-System (ECS)
        // - Entity: just an ID, like a primary key in a database
        // - Components: data attached to entities (position, mesh, color)
        // - Systems: functions that process entities with specific components
        //
        // RUST PATTERN: Tuple components bundle
        // - More efficient than multiple spawn().insert() calls
        // - All components inserted atomically (all or nothing)
        commands.spawn((
            // Mesh2d is a "marker component" that says "render this mesh in 2D"
            // BEVY PATTERN: Newtype wrapper for type safety
            // Without this, how would Bevy know if Handle<Mesh> is for 2D or 3D?
            Mesh2d(shape),
            
            // MeshMaterial2d wraps our material handle for 2D rendering
            // PERFORMANCE: materials.add() returns existing handle if color already exists
            // This deduplication happens automatically!
            MeshMaterial2d(materials.add(color)),
            
            // Transform: position, rotation, scale. The holy trinity of spatial existence!
            // GAME DESIGN: Transform hierarchy enables:
            // - Rotating groups of objects together
            // - UI that follows characters
            // - Weapons attached to hands
            // - Particle effects relative to emitters
            Transform::from_xyz(
                // Here's the math for even distribution:
                // Start at -X_EXTENT/2, end at +X_EXTENT/2
                // Each shape gets position: start + (progress * total_distance)
                // where progress = i / (num_shapes - 1)
                //
                // MATH DETAIL: Why (num_shapes - 1)?
                // With 10 shapes, we want positions at 0%, 11%, 22%... 100%
                // That's 9 gaps between 10 shapes, not 10 gaps!
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                0.0, // Y position (0 = center)
                0.0, // Z position (doesn't matter in 2D, but Transform is 3D)
            ),
        ));
    }

    // UI text - but only on platforms that support wireframes
    #[cfg(not(target_arch = "wasm32"))]
    commands.spawn((
        // Text component - just a string!
        Text::new("Press space to toggle wireframes"),
        // Node component defines UI layout properties
        // This is Bevy's flexbox-inspired layout system
        Node {
            // Absolute positioning - like position:absolute in CSS
            // This takes it out of normal layout flow
            position_type: PositionType::Absolute,
            // Distance from top and left edges
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            // ..default() fills in the rest with sensible defaults
            // It's like saying "I only care about these fields, use defaults for the rest"
            ..default()
        },
    ));
}

// This system only exists on non-wasm platforms
// It's our interactive element - responding to user input!
#[cfg(not(target_arch = "wasm32"))]
fn toggle_wireframe(
    // Mutable access to the wireframe configuration
    mut wireframe_config: ResMut<Wireframe2dConfig>,
    // Read access to keyboard state - this tracks all key states
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // just_pressed() is true for exactly one frame when key is pressed
    // This prevents toggle-spam if user holds the key
    // It's like detecting the moment a switch flips, not whether it's on
    if keyboard.just_pressed(KeyCode::Space) {
        // Toggle the boolean - classic XOR with true, or just use !
        // When true, shapes render as wireframes (you see the triangles!)
        wireframe_config.global = !wireframe_config.global;
    }
}
