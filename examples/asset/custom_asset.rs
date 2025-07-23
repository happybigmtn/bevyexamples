//! Implements loader for a custom asset type.
//!
//! Custom assets are like teaching Bevy a new language - you define what the data
//! looks like, how to read it, and Bevy handles the rest! This example shows two
//! custom asset types: structured data (CustomAsset) and raw bytes (Blob).
//! It's like adding new file formats to your game engine's vocabulary.

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use serde::Deserialize;
use thiserror::Error;

// CUSTOM ASSET TYPE - Your own data format!
#[derive(Asset, TypePath, Debug, Deserialize)]
struct CustomAsset {
    #[expect(
        dead_code,
        reason = "Used to show how the data inside an asset file will be loaded into the struct"
    )]
    value: i32,  // Whatever data you need - could be game levels, dialog trees, etc.
}

// ASSET LOADER - Teaches Bevy how to load your custom format
#[derive(Default)]
struct CustomAssetLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
/// Good error handling helps debugging!
#[non_exhaustive]
#[derive(Debug, Error)]
enum CustomAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error - RON is Rusty Object Notation
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

// Implement the AssetLoader trait - the loading recipe
impl AssetLoader for CustomAssetLoader {
    type Asset = CustomAsset;  // What we're loading
    type Settings = ();        // No loader configuration needed
    type Error = CustomAssetLoaderError;
    
    // The async loading function - runs on background thread!
    async fn load(
        &self,
        reader: &mut dyn Reader,    // Abstract file reader
        _settings: &(),             // Loader settings (unused here)
        _load_context: &mut LoadContext<'_>,  // For loading dependencies
    ) -> Result<Self::Asset, Self::Error> {
        // Read entire file to memory
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        // Parse RON format to our struct
        let custom_asset = ron::de::from_bytes::<CustomAsset>(&bytes)?;
        Ok(custom_asset)
    }

    // Tell Bevy which file extensions we handle
    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

// SECOND ASSET TYPE - Raw binary data
// Sometimes you just need the raw bytes!
#[derive(Asset, TypePath, Debug)]
struct Blob {
    bytes: Vec<u8>,
}

#[derive(Default)]
struct BlobAssetLoader;

/// Possible errors that can be produced by [`BlobAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
enum BlobAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for BlobAssetLoader {
    type Asset = Blob;
    type Settings = ();
    type Error = BlobAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading Blob...");
        // Simple loader - just read raw bytes
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(Blob { bytes })
    }
    
    // No extensions specified - type inference will determine usage
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<State>()
        // Register our custom asset types
        .init_asset::<CustomAsset>()
        .init_asset::<Blob>()
        // Register the loaders that know how to load them
        .init_asset_loader::<CustomAssetLoader>()
        .init_asset_loader::<BlobAssetLoader>()
        .add_systems(Startup, setup)
        .add_systems(Update, print_on_load)
        .run();
}

// Track our loading assets
#[derive(Resource, Default)]
struct State {
    handle: Handle<CustomAsset>,       // Main custom asset
    other_handle: Handle<CustomAsset>, // Same type, different file
    blob: Handle<Blob>,                // Raw bytes asset
    printed: bool,                     // Avoid printing multiple times
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    // STANDARD LOADING - Extension determines loader
    state.handle = asset_server.load("data/asset.custom");

    // NO EXTENSION - Still works! Bevy tries to figure it out
    state.other_handle = asset_server.load("data/asset_no_extension");

    // TYPE INFERENCE MAGIC!
    // Same file, but loaded as Blob because of the handle type
    state.blob = asset_server.load("data/asset.custom");
}

fn print_on_load(
    mut state: ResMut<State>,
    custom_assets: Res<Assets<CustomAsset>>,
    blob_assets: Res<Assets<Blob>>,
) {
    // Try to get our assets - returns None if still loading
    let custom_asset = custom_assets.get(&state.handle);
    let other_custom_asset = custom_assets.get(&state.other_handle);
    let blob = blob_assets.get(&state.blob);

    // Guard against multiple prints
    if state.printed {
        return;
    }

    // Assets load asynchronously - check if ready
    if custom_asset.is_none() {
        info!("Custom Asset Not Ready");
        return;
    }

    if other_custom_asset.is_none() {
        info!("Other Custom Asset Not Ready");
        return;
    }

    if blob.is_none() {
        info!("Blob Not Ready");
        return;
    }

    // All assets loaded - print results!
    info!("Custom asset loaded: {:?}", custom_asset.unwrap());
    info!("Custom asset loaded: {:?}", other_custom_asset.unwrap());
    info!("Blob Size: {} Bytes", blob.unwrap().bytes.len());

    // Mark as printed
    state.printed = true;
}
