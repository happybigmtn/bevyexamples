//! Rendering a scene with baked lightmaps.
//!
//! # What Are Lightmaps?
//!
//! Lightmaps are pre-calculated textures that store lighting information for static
//! geometry. Instead of calculating complex lighting in real-time, we "bake" the
//! lighting offline and store it in textures that are applied to surfaces.
//!
//! Think of it like this: Instead of calculating how light bounces around a room
//! every frame (expensive!), we do it once ahead of time and save the results as
//! images (cheap to render!).
//!
//! # Advantages of Lightmaps:
//! - Beautiful global illumination with multiple light bounces
//! - Soft shadows and ambient occlusion
//! - Very fast rendering - just texture lookups
//! - Perfect for static scenes (architecture, environments)
//!
//! # Limitations:
//! - Only works for static geometry and lights
//! - Requires UV unwrapping (each surface needs unique texture coordinates)
//! - Uses additional texture memory
//! - Dynamic objects won't affect the baked lighting
//!
//! This example shows the famous Cornell Box - a test scene used in computer
//! graphics research to validate rendering algorithms.

// argh is a command-line argument parser
use argh::FromArgs;
use bevy::{
    core_pipeline::prepass::{
        DeferredPrepass,      // Enables deferred rendering
        DepthPrepass,         // Pre-renders depth buffer
        MotionVectorPrepass,  // For motion blur/temporal effects
    },
    // Component that stores the name of a mesh from a GLTF file
    gltf::GltfMeshName,
    pbr::{
        DefaultOpaqueRendererMethod, // Controls forward vs deferred rendering
        Lightmap,                   // Component that applies a lightmap to a mesh
    },
    prelude::*,
};

/// Demonstrates lightmaps
/// 
/// Command line arguments for the example.
/// Run with --deferred for deferred rendering, --bicubic for smoother filtering
#[derive(FromArgs, Resource)]
struct Args {
    /// enables deferred shading
    /// Deferred rendering can be more efficient for scenes with many lights,
    /// though this example has no dynamic lights (only lightmaps)
    #[argh(switch)]
    deferred: bool,
    
    /// enables bicubic filtering
    /// Bicubic filtering provides smoother interpolation of lightmap textures
    /// at the cost of slightly more GPU work. Good for avoiding pixelated lighting.
    #[argh(switch)]
    bicubic: bool,
}

fn main() {
    // Parse command line arguments on native platforms
    #[cfg(not(target_arch = "wasm32"))]
    let args: Args = argh::from_env();
    // On web, we can't access command line args, so use defaults
    #[cfg(target_arch = "wasm32")]
    let args: Args = Args::from_args(&[], &[]).unwrap();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        // Turn off ambient light - we want all lighting from lightmaps
        .insert_resource(AmbientLight::NONE);

    // Configure deferred rendering if requested
    if args.deferred {
        app.insert_resource(DefaultOpaqueRendererMethod::deferred());
    }

    app.insert_resource(args)
        .add_systems(Startup, setup)
        // This system runs every frame to check for newly loaded meshes
        // and attach lightmaps to them
        .add_systems(Update, add_lightmaps_to_meshes)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, args: Res<Args>) {
    // Load the Cornell Box scene
    // The Cornell Box is a classic test scene with:
    // - A white box room
    // - Red wall on the left, green wall on the right
    // - Two boxes inside (one tall, one short)
    // - A light source at the top
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/CornellBox/CornellBox.glb"),
    )));

    // Spawn camera positioned to view into the box
    let mut camera = commands.spawn((
        Camera3d::default(),
        // Position outside the box looking in
        Transform::from_xyz(-278.0, 273.0, 800.0),
    ));

    // Configure camera for deferred rendering if enabled
    if args.deferred {
        camera.insert((
            DepthPrepass,         // Required for deferred
            MotionVectorPrepass,  // For temporal effects
            DeferredPrepass,      // Enable deferred rendering
            Msaa::Off,           // MSAA doesn't work with deferred
        ));
    }
}

// This system runs every frame and attaches lightmaps to meshes that don't have them yet
// This is necessary because GLTF scenes load asynchronously
fn add_lightmaps_to_meshes(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Query for meshes that:
    // - Have a Mesh3d component (they're 3D meshes)
    // - Have a name from the GLTF file
    // - Have a material
    // - Don't already have a lightmap
    meshes: Query<
        (Entity, &GltfMeshName, &MeshMaterial3d<StandardMaterial>),
        (With<Mesh3d>, Without<Lightmap>),
    >,
    args: Res<Args>,
) {
    // Lightmap exposure controls brightness
    // Higher values make the lightmap brighter
    let exposure = 250.0;
    
    for (entity, name, material) in meshes.iter() {
        // Match mesh names to their corresponding lightmap textures
        
        // The large box gets its own lightmap
        if &**name == "large_box" {
            materials.get_mut(material).unwrap().lightmap_exposure = exposure;
            commands.entity(entity).insert(Lightmap {
                // Load compressed lightmap texture
                // .zstd.ktx2 = KTX2 format with Zstandard compression
                image: asset_server.load("lightmaps/CornellBox-Large.zstd.ktx2"),
                // Use bicubic filtering if enabled (smoother but slower)
                bicubic_sampling: args.bicubic,
                ..default()
            });
            continue;
        }

        // The small box gets its own lightmap
        if &**name == "small_box" {
            materials.get_mut(material).unwrap().lightmap_exposure = exposure;
            commands.entity(entity).insert(Lightmap {
                image: asset_server.load("lightmaps/CornellBox-Small.zstd.ktx2"),
                bicubic_sampling: args.bicubic,
                ..default()
            });
            continue;
        }

        // The room itself (walls, floor, ceiling) shares one lightmap
        if name.starts_with("cornell_box") {
            materials.get_mut(material).unwrap().lightmap_exposure = exposure;
            commands.entity(entity).insert(Lightmap {
                image: asset_server.load("lightmaps/CornellBox-Box.zstd.ktx2"),
                bicubic_sampling: args.bicubic,
                ..default()
            });
            continue;
        }
    }
}
