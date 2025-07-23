//! This example shows how to use the ECS and the [`AsyncComputeTaskPool`]
//! to spawn, poll, and complete tasks across systems and system ticks.
//!
//! Async tasks are like hiring contractors - you give them work, they go off
//! and do it in the background, and you check back later to see if they're done.
//! This example spawns 216 cubes (6x6x6), each with a random "processing time".
//! Instead of freezing the game, we let them appear as they're ready!

use bevy::{
    ecs::{system::SystemState, world::CommandQueue},
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use rand::Rng;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (
            setup_env,     // Camera and lights
            add_assets,    // Shared mesh and material
            spawn_tasks    // Start async work
        ))
        .add_systems(Update, handle_tasks) // Poll for completion
        .run();
}

// Number of cubes to spawn across the x, y, and z axis
const NUM_CUBES: u32 = 6; // 6³ = 216 total cubes

// Shared resources - all cubes use the same mesh and material
// Like a factory using the same mold for all products
#[derive(Resource, Deref)]
struct BoxMeshHandle(Handle<Mesh>);

#[derive(Resource, Deref)]
struct BoxMaterialHandle(Handle<StandardMaterial>);

/// Startup system which runs only once and generates our Box Mesh
/// and Box Material assets, adds them to their respective Asset
/// Resources, and stores their handles as resources so we can access
/// them later when we're ready to render our Boxes
fn add_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create one mesh for all cubes - efficient memory usage!
    let box_mesh_handle = meshes.add(Cuboid::new(0.25, 0.25, 0.25));
    commands.insert_resource(BoxMeshHandle(box_mesh_handle));

    // Pink material for visibility
    let box_material_handle = materials.add(Color::srgb(1.0, 0.2, 0.3));
    commands.insert_resource(BoxMaterialHandle(box_material_handle));
}

// Component that holds an async task
// The task returns a CommandQueue when complete
#[derive(Component)]
struct ComputeTransform(Task<CommandQueue>);

/// This system generates tasks simulating computationally intensive
/// work that potentially spans multiple frames/ticks. A separate
/// system, [`handle_tasks`], will poll the spawned tasks on subsequent
/// frames/ticks, and use the results to spawn cubes
fn spawn_tasks(mut commands: Commands) {
    // Get the global thread pool for compute tasks
    let thread_pool = AsyncComputeTaskPool::get();
    
    // Create a 6x6x6 grid of tasks
    for x in 0..NUM_CUBES {
        for y in 0..NUM_CUBES {
            for z in 0..NUM_CUBES {
                // Pre-spawn entity that will become a cube later
                let entity = commands.spawn_empty().id();
                
                // Spawn async task on background thread
                let task = thread_pool.spawn(async move {
                    // Random processing time (50ms to 5s)
                    // Simulates varying computational complexity
                    let duration = Duration::from_secs_f32(rand::thread_rng().gen_range(0.05..5.0));

                    // Simulate heavy computation
                    async_std::task::sleep(duration).await;

                    // Calculate final position in grid
                    let transform = Transform::from_xyz(x as f32, y as f32, z as f32);
                    
                    // CommandQueue lets us defer World access
                    // We can't access World directly from async context!
                    let mut command_queue = CommandQueue::default();

                    // Push a closure that will run on the main thread
                    command_queue.push(move |world: &mut World| {
                        // SystemState gives us safe World access
                        // Like a temporary system just for this closure
                        let (box_mesh_handle, box_material_handle) = {
                            let mut system_state = SystemState::<(
                                Res<BoxMeshHandle>,
                                Res<BoxMaterialHandle>,
                            )>::new(world);
                            let (box_mesh_handle, box_material_handle) =
                                system_state.get_mut(world);

                            (box_mesh_handle.clone(), box_material_handle.clone())
                        };

                        // Transform empty entity into visible cube
                        world
                            .entity_mut(entity)
                            // Add rendering components
                            .insert((
                                Mesh3d(box_mesh_handle),
                                MeshMaterial3d(box_material_handle),
                                transform,
                            ))
                            // Clean up - remove the task component
                            .remove::<ComputeTransform>();
                    });

                    command_queue
                });

                // Attach task to entity - it's now "processing"
                commands.entity(entity).insert(ComputeTransform(task));
            }
        }
    }
}

/// This system queries for entities that have our Task<Transform> component. It polls the
/// tasks to see if they're complete. If the task is complete it takes the result, adds a
/// new [`Mesh3d`] and [`MeshMaterial3d`] to the entity using the result from the task's work, and
/// removes the task component from the entity.
fn handle_tasks(mut commands: Commands, mut transform_tasks: Query<&mut ComputeTransform>) {
    for mut task in &mut transform_tasks {
        // Poll the task once without blocking
        // block_on + poll_once = check if ready, don't wait
        if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
            // Task complete! Apply the deferred commands
            commands.append(&mut commands_queue);
        }
        // If not ready, we'll check again next frame
    }
}

/// This system is only used to setup light and camera for the environment
fn setup_env(mut commands: Commands) {
    // Center camera on the cube grid
    // Even number of cubes: offset by 0.5 to center between cubes
    // Odd number: center on middle cube
    let offset = if NUM_CUBES % 2 == 0 {
        (NUM_CUBES / 2) as f32 - 0.5
    } else {
        (NUM_CUBES / 2) as f32
    };

    // Light positioned above and to the side
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 12.0, 15.0)));

    // Camera looks at center of cube formation
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(offset, offset, 15.0)
            .looking_at(Vec3::new(offset, offset, 0.0), Vec3::Y),
    ));
}
