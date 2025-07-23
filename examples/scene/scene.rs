//! This example demonstrates how to load scene data from files and then dynamically
//! apply that data to entities in your Bevy `World`. This includes spawning new
//! entities and applying updates to existing ones. Scenes in Bevy encapsulate
//! serialized and deserialized `Components` or `Resources` so that you can easily
//! store, load, and manipulate data outside of a purely code-driven context.
//!
//! This example also shows how to do the following:
//! * Register your custom types for reflection, which allows them to be serialized,
//!   deserialized, and manipulated dynamically.
//! * Skip serialization of fields you don't want stored in your scene files (like
//!   runtime values that should always be computed dynamically).
//! * Save a new scene to disk to show how it can be updated compared to the original
//!   scene file (and how that updated scene file might then be used later on).
//!
//! The example proceeds by creating components and resources, registering their types,
//! loading a scene from a file, logging when changes are detected, and finally saving
//! a new scene file to disk. This is useful for anyone wanting to see how to integrate
//! file-based scene workflows into their Bevy projects.
//!
//! # Note on working with files
//!
//! The saving behavior uses the standard filesystem APIs, which are blocking, so it
//! utilizes a thread pool (`IoTaskPool`) to avoid stalling the main thread. This
//! won't work on WASM because WASM typically doesn't have direct filesystem access.
//!
//! Scenes are like save files for your game world - they capture a snapshot of entities,
//! components, and resources that you can reload later. Think of them as freeze-dried
//! game states: just add Bevy and they spring back to life! Perfect for level editors,
//! save games, or any data-driven content.

use bevy::{asset::LoadState, prelude::*, tasks::IoTaskPool};
use core::time::Duration;
use std::{fs::File, io::Write};

/// The entry point of our Bevy app.
///
/// Sets up default plugins, registers all necessary component/resource types
/// for serialization/reflection, and runs the various systems in the correct schedule.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // REGISTER TYPES FOR REFLECTION
        // This tells Bevy how to serialize/deserialize our custom types
        .register_type::<ComponentA>()
        .register_type::<ComponentB>()
        .register_type::<ResourceA>()
        .add_systems(
            Startup,
            (
                save_scene_system,    // Creates and saves a new scene
                load_scene_system,    // Loads existing scene from file
                infotext_system,      // UI message
            ),
        )
        .add_systems(Update, (
            log_system,      // Monitor component changes
            panic_on_fail    // Error handling for CI
        ))
        .run();
}

/// # Components, Resources, and Reflection
///
/// Below are some simple examples of how to define your own Bevy `Component` types
/// and `Resource` types so that they can be properly reflected, serialized, and
/// deserialized. The `#[derive(Reflect)]` macro enables Bevy's reflection features,
/// and we add component-specific reflection by using `#[reflect(Component)]`.
/// We also illustrate how to skip serializing fields and how `FromWorld` can help
/// create runtime-initialized data.

// SIMPLE COMPONENT - Everything gets saved
#[derive(Component, Reflect, Default)]
#[reflect(Component)] // Magic attribute that makes components scene-compatible!
struct ComponentA {
    /// Position-like data that will be saved in scenes
    pub x: f32,
    pub y: f32,
}

// ADVANCED COMPONENT - Mix of saved and runtime data
#[derive(Component, Reflect)]
#[reflect(Component)]
struct ComponentB {
    /// This gets saved to the scene file
    pub value: String,
    
    /// This is runtime-only data - NOT saved!
    /// Perfect for timestamps, temp values, or computed data
    #[reflect(skip_serializing)]
    pub _time_since_startup: Duration,
}

// FromWorld - Initialize component using world data
// This runs when deserializing from scenes!
impl FromWorld for ComponentB {
    fn from_world(world: &mut World) -> Self {
        // Access resources during initialization
        let time = world.resource::<Time>();
        ComponentB {
            // Runtime field gets current time
            _time_since_startup: time.elapsed(),
            // Serialized field gets default
            value: "Default Value".to_string(),
        }
    }
}

// RESOURCES IN SCENES - Yes, you can save resources too!
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]  // Makes resources scene-compatible
struct ResourceA {
    /// Game state that persists across scene loads
    pub score: u32,
}

/// # Scene File Paths

/// Original scene we'll load - check this file to see the RON format!
const SCENE_FILE_PATH: &str = "scenes/load_scene_example.scn.ron";

/// Where we'll save our programmatically created scene
const NEW_SCENE_FILE_PATH: &str = "scenes/load_scene_example-new.scn.ron";

/// Loads a scene from an asset file and spawns it in the current world.
///
/// DynamicSceneRoot is like a scene player - it creates entities from the scene file
fn load_scene_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // One line to load and spawn an entire scene!
    // The scene file contains serialized entities and components
    commands.spawn(DynamicSceneRoot(asset_server.load(SCENE_FILE_PATH)));
}

/// Monitors scene loading - shows when components are added or changed
fn log_system(
    query: Query<(Entity, &ComponentA), Changed<ComponentA>>,
    res: Option<Res<ResourceA>>,
) {
    // Log any ComponentA changes (happens when scene loads!)
    for (entity, component_a) in &query {
        info!("  Entity({})", entity.index());
        info!(
            "    ComponentA: {{ x: {} y: {} }}\n",
            component_a.x, component_a.y
        );
    }
    
    // Check if ResourceA was just loaded from the scene
    if let Some(res) = res {
        if res.is_added() {
            info!("  New ResourceA: {{ score: {} }}\n", res.score);
        }
    }
}

/// Creates and saves a new scene programmatically
fn save_scene_system(world: &mut World) {
    // STEP 1: Create a mini-world for our scene
    // Scenes can capture part or all of a world
    let mut scene_world = World::new();

    // STEP 2: Copy type registry so scene knows our components
    // Without this, the scene can't serialize our custom types!
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    scene_world.insert_resource(type_registry);

    // STEP 3: Populate the scene world with entities
    let mut component_b = ComponentB::from_world(world);
    component_b.value = "hello".to_string();
    
    // Entity with multiple components
    scene_world.spawn((
        component_b,
        ComponentA { x: 1.0, y: 2.0 },
        Transform::IDENTITY,
        Name::new("joe"),  // Names help identify entities in scene files
    ));
    
    // Simple entity with just ComponentA
    scene_world.spawn(ComponentA { x: 3.0, y: 4.0 });
    
    // Resources get saved too!
    scene_world.insert_resource(ResourceA { score: 1 });

    // STEP 4: Create the actual scene
    let scene = DynamicScene::from_world(&scene_world);

    // STEP 5: Serialize to RON format (Rusty Object Notation)
    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    // Show the RON data in console - it's human readable!
    info!("{}", serialized_scene);

    // STEP 6: Save to file (async to avoid blocking)
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(format!("assets/{NEW_SCENE_FILE_PATH}"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();  // Fire and forget
}

/// UI setup - just tells user to check console
fn infotext_system(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Text::new("Nothing to see in this window! Check the console output!"),
        TextFont {
            font_size: 42.0,
            ..default()
        },
        Node {
            align_self: AlignSelf::FlexEnd,
            ..default()
        },
    ));
}

/// Error handling for CI testing - you probably don't need this in your game
fn panic_on_fail(scenes: Query<&DynamicSceneRoot>, asset_server: Res<AssetServer>) {
    for scene in &scenes {
        if let Some(LoadState::Failed(err)) = asset_server.get_load_state(&scene.0) {
            panic!("Failed to load scene. {}", err);
        }
    }
}
