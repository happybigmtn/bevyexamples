//! In this example we generate four texture atlases (sprite sheets) from a folder containing
//! individual sprites.
//!
//! The texture atlases are generated with different padding and sampling to demonstrate the
//! effect of these settings, and how bleeding issues can be resolved by padding the sprites.
//!
//! Only one padded and one unpadded texture atlas are rendered to the screen.
//! An upscaled sprite from each of the four atlases are rendered to the screen.
//!
//! Texture atlases are like creating a photo collage - you take many individual images and
//! arrange them efficiently on a single large sheet! This reduces draw calls (like having
//! one big poster instead of hundreds of sticky notes). The "bleeding" problem is like ink
//! from one photo smearing into another - padding creates safe borders between images.
//! Linear vs nearest sampling is like the difference between smooth gradients vs sharp pixels.

use bevy::{asset::LoadedFolder, image::ImageSampler, prelude::*};

fn main() {
    App::new()
        // PIXEL-PERFECT RENDERING - Default to crisp, pixelated look
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // fallback to nearest sampling
        // STATE MACHINE - Organize loading vs display phases
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::Setup), load_textures)  // Start loading
        .add_systems(Update, check_textures.run_if(in_state(AppState::Setup)))  // Wait for loading
        .add_systems(OnEnter(AppState::Finished), setup)  // Create atlases when ready
        .run();
}

// STATE MACHINE - Two-phase process for atlas creation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,    // Loading individual sprites from disk
    Finished, // Sprites loaded, ready to build atlases
}

// FOLDER HANDLE - Reference to a collection of sprite files
#[derive(Resource, Default)]
struct RpgSpriteFolder(Handle<LoadedFolder>);

fn load_textures(mut commands: Commands, asset_server: Res<AssetServer>) {
    // BATCH LOADING - Load an entire folder of sprites at once
    // Load multiple, individual sprites from a folder
    // This is asynchronous - the function returns immediately while loading continues
    commands.insert_resource(RpgSpriteFolder(asset_server.load_folder("textures/rpg")));
}

fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    rpg_sprite_folder: Res<RpgSpriteFolder>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    // LOADING COMPLETION DETECTION - Wait for all sprites to finish loading
    // Advance the `AppState` once all sprite handles have been loaded by the `AssetServer`
    for event in events.read() {
        // Check if OUR folder (and all dependencies) finished loading
        if event.is_loaded_with_dependencies(&rpg_sprite_folder.0) {
            next_state.set(AppState::Finished);  // Trigger atlas creation
        }
    }
}

fn setup(
    mut commands: Commands,
    rpg_sprite_handles: Res<RpgSpriteFolder>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    mut textures: ResMut<Assets<Image>>,
) {
    let loaded_folder = loaded_folders.get(&rpg_sprite_handles.0).unwrap();

    // ATLAS GENERATION - Create 4 different atlas variants
    // Create texture atlases with different padding and sampling

    // LINEAR SAMPLING, NO PADDING - Smooth but may have bleeding
    let (texture_atlas_linear, linear_sources, linear_texture) = create_texture_atlas(
        loaded_folder,
        None,                               // No padding between sprites
        Some(ImageSampler::linear()),       // Smooth interpolation
        &mut textures,
    );
    let atlas_linear_handle = texture_atlases.add(texture_atlas_linear);

    // NEAREST SAMPLING, NO PADDING - Pixel-perfect but may have bleeding
    let (texture_atlas_nearest, nearest_sources, nearest_texture) = create_texture_atlas(
        loaded_folder,
        None,                               // No padding
        Some(ImageSampler::nearest()),      // Sharp, pixelated look
        &mut textures,
    );
    let atlas_nearest_handle = texture_atlases.add(texture_atlas_nearest);

    // LINEAR SAMPLING, WITH PADDING - Smooth and bleeding-resistant
    let (texture_atlas_linear_padded, linear_padded_sources, linear_padded_texture) =
        create_texture_atlas(
            loaded_folder,
            Some(UVec2::new(6, 6)),         // 6 pixels of padding on all sides
            Some(ImageSampler::linear()),   // Smooth interpolation
            &mut textures,
        );
    let atlas_linear_padded_handle = texture_atlases.add(texture_atlas_linear_padded.clone());

    // NEAREST SAMPLING, WITH PADDING - Perfect pixels and bleeding-resistant
    let (texture_atlas_nearest_padded, nearest_padded_sources, nearest_padded_texture) =
        create_texture_atlas(
            loaded_folder,
            Some(UVec2::new(6, 6)),         // 6 pixels of safe space
            Some(ImageSampler::nearest()),  // Crisp pixels
            &mut textures,
        );
    let atlas_nearest_padded_handle = texture_atlases.add(texture_atlas_nearest_padded);

    commands.spawn(Camera2d);

    // ATLAS VISUALIZATION - Show the actual atlas textures side by side
    // Padded textures are to the right, unpadded to the left

    // Draw unpadded texture atlas - you can see sprites touching each other
    commands.spawn((
        Sprite::from_image(linear_texture.clone()),
        Transform {
            translation: Vec3::new(-250.0, -160.0, 0.0),  // Left side
            scale: Vec3::splat(0.5),                       // Shrink to fit
            ..default()
        },
    ));

    // Draw padded texture atlas - notice the gaps between sprites
    commands.spawn((
        Sprite::from_image(linear_padded_texture.clone()),
        Transform {
            translation: Vec3::new(250.0, -160.0, 0.0),   // Right side
            scale: Vec3::splat(0.5),                       // Same size
            ..default()
        },
    ));

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    // Padding label text style
    let text_style: TextFont = TextFont {
        font: font.clone(),
        font_size: 42.0,
        ..default()
    };

    // Labels to indicate padding

    // No padding
    create_label(
        &mut commands,
        (-250.0, 250.0, 0.0),
        "No padding",
        text_style.clone(),
    );

    // Padding
    create_label(&mut commands, (250.0, 250.0, 0.0), "Padding", text_style);

    // Get handle to a sprite to render
    let vendor_handle: Handle<Image> = asset_server
        .get_handle("textures/rpg/chars/vendor/generic-rpg-vendor.png")
        .unwrap();

    // Configuration array to render sprites through iteration
    let configurations: [(
        &str,
        Handle<TextureAtlasLayout>,
        TextureAtlasSources,
        Handle<Image>,
        f32,
    ); 4] = [
        (
            "Linear",
            atlas_linear_handle,
            linear_sources,
            linear_texture,
            -350.0,
        ),
        (
            "Nearest",
            atlas_nearest_handle,
            nearest_sources,
            nearest_texture,
            -150.0,
        ),
        (
            "Linear",
            atlas_linear_padded_handle,
            linear_padded_sources,
            linear_padded_texture,
            150.0,
        ),
        (
            "Nearest",
            atlas_nearest_padded_handle,
            nearest_padded_sources,
            nearest_padded_texture,
            350.0,
        ),
    ];

    // Label text style
    let sampling_label_style = TextFont {
        font,
        font_size: 25.0,
        ..default()
    };

    let base_y = 80.0; // y position of the sprites

    for (sampling, atlas_handle, atlas_sources, atlas_texture, x) in configurations {
        // Render a sprite from the texture_atlas
        create_sprite_from_atlas(
            &mut commands,
            (x, base_y, 0.0),
            atlas_texture,
            atlas_sources,
            atlas_handle,
            &vendor_handle,
        );

        // Render a label to indicate the sampling setting
        create_label(
            &mut commands,
            (x, base_y + 110.0, 0.0), // Offset to y position of the sprite
            sampling,
            sampling_label_style.clone(),
        );
    }
}

/// THE ATLAS FACTORY - Combine individual sprites into one mega-texture!
/// Create a texture atlas with the given padding and sampling settings
/// from the individual sprites in the given folder.
fn create_texture_atlas(
    folder: &LoadedFolder,
    padding: Option<UVec2>,
    sampling: Option<ImageSampler>,
    textures: &mut ResMut<Assets<Image>>,
) -> (TextureAtlasLayout, TextureAtlasSources, Handle<Image>) {
    // THE PUZZLE ASSEMBLER - TextureAtlasBuilder arranges sprites optimally
    // Build a texture atlas using the individual sprites
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    texture_atlas_builder.padding(padding.unwrap_or_default());
    
    // COLLECT ALL THE PIECES - Add each sprite to the atlas
    for handle in folder.handles.iter() {
        let id = handle.id().typed_unchecked::<Image>();
        let Some(texture) = textures.get(id) else {
            warn!(
                "{} did not resolve to an `Image` asset.",
                handle.path().unwrap()
            );
            continue;
        };

        // Add this sprite to our collage
        texture_atlas_builder.add_texture(Some(id), texture);
    }

    // THE BIG REVEAL - Build the final atlas!
    let (texture_atlas_layout, texture_atlas_sources, texture) =
        texture_atlas_builder.build().unwrap();
    let texture = textures.add(texture);

    // SAMPLING CONFIGURATION - How to interpolate between pixels
    // Update the sampling settings of the texture atlas
    let image = textures.get_mut(&texture).unwrap();
    image.sampler = sampling.unwrap_or_default();

    // Return the atlas layout (where sprites are), sources (name mappings), and the texture
    (texture_atlas_layout, texture_atlas_sources, texture)
}

/// SPRITE EXTRACTION - Pull one sprite out of the atlas and display it
/// Create and spawn a sprite from a texture atlas
fn create_sprite_from_atlas(
    commands: &mut Commands,
    translation: (f32, f32, f32),
    atlas_texture: Handle<Image>,     // The big combined texture
    atlas_sources: TextureAtlasSources,  // Maps original handles to atlas positions
    atlas_handle: Handle<TextureAtlasLayout>,  // Describes where each sprite is
    vendor_handle: &Handle<Image>,    // Which specific sprite we want
) {
    commands.spawn((
        Transform {
            translation: Vec3::new(translation.0, translation.1, translation.2),
            scale: Vec3::splat(3.0),  // Make it 3x larger so we can see sampling differences
            ..default()
        },
        // FROM_ATLAS_IMAGE - Use just a portion of the big texture
        Sprite::from_atlas_image(
            atlas_texture,
            // Look up where our vendor sprite ended up in the atlas
            atlas_sources.handle(atlas_handle, vendor_handle).unwrap(),
        ),
    ));
}

/// TEXT LABEL HELPER - Create descriptive text for the demo
/// Create and spawn a label (text)
fn create_label(
    commands: &mut Commands,
    translation: (f32, f32, f32),
    text: &str,
    text_style: TextFont,
) {
    commands.spawn((
        Text2d::new(text),
        text_style,
        TextLayout::new_with_justify(JustifyText::Center),  // Center-align the text
        Transform {
            translation: Vec3::new(translation.0, translation.1, translation.2),
            ..default()
        },
    ));
}
