//! # Error Handling in Bevy Systems
//!
//! This example demonstrates how to handle errors in Bevy systems using Rust's Result type.
//! By default, systems that return errors will panic, but Bevy provides flexible error
//! handling mechanisms to gracefully handle failures.
//!
//! ## Key Concepts Covered
//!
//! - **Fallible Systems**: Systems that return `Result<(), BevyError>` instead of `()`
//! - **Error Handlers**: Global and per-system error handling strategies
//! - **Command Error Handling**: How to handle errors in deferred commands
//! - **Observer Error Handling**: Error handling in event-driven observers
//! - **Result Propagation**: Using Rust's `?` operator in systems
//!
//! ## Why Error Handling Matters
//!
//! In game development, many operations can fail:
//! - Loading assets that might not exist
//! - Accessing resources that haven't been initialized
//! - Performing calculations that might overflow
//! - Network operations that might timeout
//!
//! Proper error handling ensures your game can recover gracefully from these situations.

use bevy::ecs::{
    // Built-in error handler that logs warnings instead of panicking
    error::warn,
    // A world reference that allows deferred mutations (used in observers)
    world::DeferredWorld,
};
// For sampling random points on mesh surfaces
use bevy::math::sampling::UniformMeshSampler;
use bevy::prelude::*;

// Random number generation for demo purposes
use rand::distributions::Distribution;
use rand::SeedableRng;
// ChaCha8 is a cryptographically secure RNG that's deterministic with a seed
use rand_chacha::ChaCha8Rng;

fn main() {
    let mut app = App::new();
    
    // === Global Error Handler Configuration ===
    // 
    // By default, when a system returns an error, Bevy will panic and crash the app.
    // This is often too harsh for production games where you want to recover gracefully.
    //
    // We can set a global error handler that applies to all systems in the app.
    // Built-in handlers include:
    // - `panic`: Crash the app (default behavior)
    // - `error`: Log at error level and continue
    // - `warn`: Log at warning level and continue (used here)
    // - `info`, `debug`, `trace`: Log at respective levels
    // - `ignore`: Silently ignore errors (dangerous!)
    app.set_error_handler(warn);

    app.add_plugins(DefaultPlugins);

    // Optional mesh picking support for interactive examples
    #[cfg(feature = "bevy_mesh_picking_backend")]
    app.add_plugins(MeshPickingPlugin);

    // === Adding Fallible Systems ===
    //
    // Fallible systems look just like regular systems when adding them.
    // The key difference is their return type:
    // - Regular system: fn system() { }
    // - Fallible system: fn system() -> Result { }
    //
    // `Result` is a type alias for `Result<(), BevyError>`
    app.add_systems(Startup, setup);

    // This system demonstrates command error handling
    app.add_systems(Startup, failing_commands);

    // === Custom Per-System Error Handling ===
    //
    // Sometimes you want specific error handling for individual systems.
    // The `pipe` method lets you transform the output of one system into
    // the input of another. Here we pipe the Result to a closure that
    // handles the error with custom logic.
    app.add_systems(
        PostStartup,
        failing_system.pipe(|result: In<Result>| {
            // `In<T>` is a system parameter that receives piped input
            // `inspect_err` runs the closure only if the Result is Err
            // We use `let _ =` to explicitly ignore the Result
            let _ = result.0.inspect_err(|err| info!("captured error: {err}"));
        }),
    );

    // === Fallible Observers ===
    //
    // Observers (event handlers) can also be fallible.
    // They follow the same error handling rules as systems.
    app.add_observer(fallible_observer);

    // When we run the app, we'll see different error handling in action:
    //
    // 1. The global handler logs: WARN Encountered an error in system...
    // 2. The piped handler logs: INFO captured error: ...
    // 3. Command errors are handled according to their configuration
    //
    // The app continues running despite these errors!
    app.run();
}

/// A fallible setup system that demonstrates error propagation with the `?` operator.
/// 
/// The `?` operator is Rust's way of propagating errors up the call stack.
/// If any operation returns an Err, the ? immediately returns from this function
/// with that error, which Bevy's error handler will then process.
///
/// Common patterns:
/// - `operation()?` - Propagate error if operation fails
/// - `option.ok_or("error")?` - Convert None to an error and propagate
/// - `result.map_err(|e| format!("context: {e}"))?` - Add context to errors
///
/// See: <https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator>
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> Result {
    // Create a deterministic RNG for reproducible random sampling
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(19878367467712);

    // === Scene Setup (No Errors Expected) ===
    
    // Create a ground plane - this operation is infallible
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, -2.5, 0.0),
    ));

    // Add lighting to the scene
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // Position camera to view the scene
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // === Fallible Operations with ? Operator ===
    
    // Create a sphere mesh with subdivision level 7
    // The `ico()` method can fail if the subdivision level is invalid
    // The ? operator will return early with the error if it fails
    let mut sphere_mesh = Sphere::new(1.0).mesh().ico(7)?;
    
    // Generate tangent vectors for proper lighting calculations
    // This can fail if the mesh topology is invalid
    sphere_mesh.generate_tangents()?;

    // Spawn the sphere entity
    let mut sphere = commands.spawn((
        Mesh3d(meshes.add(sphere_mesh.clone())),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
        Transform::from_xyz(-1.0, 1.0, 0.0),
    ));

    // === More Fallible Operations ===
    
    // Extract triangle data from the mesh
    // This can fail if the mesh doesn't have the expected attributes
    let triangles = sphere_mesh.triangles()?;
    
    // Create a distribution for uniform random sampling on the mesh surface
    // `try_new` can fail if the triangles are degenerate or invalid
    let distribution = UniformMeshSampler::try_new(triangles)?;

    // Create small sphere mesh for visualization points
    // Even this simple operation can fail with invalid parameters
    let point_mesh = meshes.add(Sphere::new(0.01).mesh().ico(3)?);
    
    // Red emissive material for the sample points
    let point_material = materials.add(StandardMaterial {
        base_color: Srgba::RED.into(),
        emissive: LinearRgba::rgb(1.0, 0.0, 0.0),  // Makes points glow red
        ..default()
    });

    // Spawn 10,000 points uniformly distributed on the sphere surface
    for point in distribution.sample_iter(&mut seeded_rng).take(10000) {
        sphere.with_child((
            Mesh3d(point_mesh.clone()),
            MeshMaterial3d(point_material.clone()),
            Transform::from_translation(point),
        ));
    }

    // === Success Case ===
    // If we reach here, all fallible operations succeeded
    // Return Ok(()) to indicate successful completion
    Ok(())
}

/// A fallible observer that responds to pointer movement events.
/// 
/// Observers are Bevy's event handling system. Like regular systems,
/// they can be fallible by returning a Result.
/// 
/// This observer makes an entity move back and forth when the pointer moves over it.
fn fallible_observer(
    // The event that triggered this observer
    trigger: Trigger<Pointer<Move>>,
    // DeferredWorld allows mutations that are applied after the observer runs
    mut world: DeferredWorld,
    // Local state persists between observer invocations
    mut step: Local<f32>,
) -> Result {
    // Try to get the Transform component of the entity that was moused over
    // This can fail if the entity doesn't have a Transform
    let mut transform = world
        .get_mut::<Transform>(trigger.target)
        .ok_or("No transform found.")?;  // Convert Option to Result

    // Simple ping-pong movement logic
    // Reverse direction when reaching boundaries at x = ±3
    *step = if transform.translation.x > 3. {
        -0.1  // Move left
    } else if transform.translation.x < -3. || *step == 0. {
        0.1   // Move right
    } else {
        *step // Continue in current direction
    };

    // Apply the movement
    transform.translation.x += *step;

    // Return success
    Ok(())
}

/// A resource that is never actually inserted into the world.
/// Used to demonstrate error handling when resources are missing.
#[derive(Resource)]
struct UninitializedResource;

/// A system that intentionally fails by trying to access a missing resource.
/// 
/// This demonstrates a common error scenario: accessing resources that
/// haven't been initialized yet. In a real game, this might happen with:
/// - Settings that haven't loaded yet
/// - Network connections that aren't established
/// - Game state that depends on user actions
fn failing_system(world: &mut World) -> Result {
    world
        // `get_resource` returns `Option<&T>`:
        // - Some(&resource) if it exists
        // - None if it doesn't exist
        .get_resource::<UninitializedResource>()
        // `ok_or` converts Option to Result:
        // - Some(T) becomes Ok(T)
        // - None becomes Err(error_value)
        // BevyError implements From<&str>, so we can use a string literal
        .ok_or("Resource not initialized")?;

    // This line is never reached because the resource doesn't exist
    Ok(())
}

/// Demonstrates error handling for deferred commands.
/// 
/// Commands are deferred operations that run after systems complete.
/// They can fail in various ways, such as referencing non-existent entities.
fn failing_commands(mut commands: Commands) {
    // === Command Error with Global Handler ===
    
    commands
        // Create an entity ID that doesn't correspond to any spawned entity
        // In a real game, this might happen with:
        // - Entities that were despawned by another system
        // - Entity IDs received over the network
        // - Saved game data with outdated entity references
        .entity(Entity::from_raw_u32(12345678).unwrap())
        // Try to add a component to the non-existent entity
        // Without our error handler, this would panic!
        // With `warn` handler, it logs a warning and continues
        .insert(Transform::default());

    // === Command with Custom Error Handler ===
    
    // `queue_handled` lets us provide a custom error handler for specific commands
    commands.queue_handled(
        // The command: a closure that returns a Result
        |world: &mut World| -> Result {
            world
                .get_resource::<UninitializedResource>()
                .ok_or("Resource not initialized when accessed in a command")?;

            Ok(())
        },
        // The error handler: called if the command returns Err
        // Receives the error and an EntityCommandInfo context
        |error, context| {
            // Log at error level with both error message and context
            // Context includes information about where the command was queued
            error!("{error}, {context}");
        },
    );
}
