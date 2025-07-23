//! Loads and renders a glTF file as a scene, and list all the different `gltf_extras`.
//!
//! # What are glTF Extras?
//!
//! glTF files can contain custom application-specific data called "extras".
//! These are JSON objects that can be attached to various parts of a glTF file:
//! - **Scenes**: Custom data for entire scenes
//! - **Nodes**: Custom data for objects/transforms in the hierarchy
//! - **Meshes**: Custom data for geometry
//! - **Materials**: Custom data for surface properties
//! - **Primitives**: Custom data for mesh sub-parts
//!
//! # Why Use Extras?
//!
//! Extras are perfect for:
//! - **Game-specific data**: Health points, damage values, item descriptions
//! - **Editor metadata**: Layer info, tags, custom properties from Blender/Maya
//! - **Physics properties**: Mass, friction, collision groups
//! - **LOD settings**: Level-of-detail distances and quality settings
//! - **Anything else**: Any data you want to pass from your 3D editor to your game
//!
//! # How It Works
//!
//! 1. In your 3D editor (Blender, etc.), add custom properties to objects
//! 2. Export as glTF - the properties become "extras"
//! 3. Bevy automatically parses these extras into components
//! 4. You can query for these components and use the data in your game
//!
//! This example shows how to find and display all extras in a loaded glTF file.

use bevy::{
    gltf::{
        GltfExtras,         // Extras attached to nodes (objects)
        GltfMaterialExtras, // Extras attached to materials
        GltfMeshExtras,     // Extras attached to meshes
        GltfSceneExtras,    // Extras attached to scenes
    },
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // This system runs every frame to find and display extras
        // We run it in Update (not just once) because glTF files
        // load asynchronously - entities might not exist immediately
        .add_systems(Update, check_for_gltf_extras)
        .run();
}

// Marker component for the UI text that displays extras information
#[derive(Component)]
struct ExampleDisplay;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera positioned to see the loaded model
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Basic lighting
    commands.spawn(DirectionalLight {
        shadows_enabled: true,
        ..default()
    });

    // Load a glTF file that contains various extras
    // This example file has extras attached to:
    // - The scene itself
    // - Individual nodes (objects)
    // - Meshes
    // - Materials
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/extras/gltf_extras.glb"),
    )));

    // Create UI text to display the extras we find
    commands.spawn((
        Text::default(),
        TextFont {
            font_size: 15.,
            ..default()
        },
        // Position in top-left corner
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        // Mark this as our display text
        ExampleDisplay,
    ));
}

// System that searches for entities with glTF extras and displays them
fn check_for_gltf_extras(
    // Query for all possible types of extras
    // Using Option<&T> because not every entity will have every type of extra
    gltf_extras_per_entity: Query<(
        Entity,                          // The entity ID
        Option<&Name>,                   // Optional name component
        Option<&GltfSceneExtras>,        // Extras from the scene level
        Option<&GltfExtras>,             // Extras from nodes (objects)
        Option<&GltfMeshExtras>,         // Extras from mesh data
        Option<&GltfMaterialExtras>,     // Extras from materials
    )>,
    // Single ensures there's exactly one ExampleDisplay entity
    mut display: Single<&mut Text, With<ExampleDisplay>>,
) {
    let mut gltf_extra_infos_lines: Vec<String> = vec![];

    // Check every entity in the scene
    for (id, name, scene_extras, extras, mesh_extras, material_extras) in
        gltf_extras_per_entity.iter()
    {
        // Only process entities that have at least one type of extra
        if scene_extras.is_some()
            || extras.is_some()
            || mesh_extras.is_some()
            || material_extras.is_some()
        {
            // Format the information for display
            let formatted_extras = format!(
                "Extras per entity {} ('Name: {}'):
    - scene extras:     {:?}
    - primitive extras: {:?}
    - mesh extras:      {:?}
    - material extras:  {:?}
                ",
                id,
                name.unwrap_or(&Name::default()), // Use default name if none exists
                scene_extras,
                extras,
                mesh_extras,
                material_extras
            );
            gltf_extra_infos_lines.push(formatted_extras);
        }
        // Update the display text with all found extras
        display.0 = gltf_extra_infos_lines.join("\n");
    }
}
