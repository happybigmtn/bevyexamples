//! By default Bevy loads images to textures that clamps the image to the edges
//! This example shows how to configure it to repeat the image instead.

use bevy::{
    audio::AudioPlugin,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
};

/// How much to move some rectangles away from the center
const RECTANGLE_OFFSET: f32 = 250.0;
/// Length of the sides of the rectangle
const RECTANGLE_SIDE: f32 = 200.;
/// How much to move the label away from the rectangle
const LABEL_OFFSET: f32 = (RECTANGLE_SIDE / 2.) + 25.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<AudioPlugin>())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // #11111: We use a duplicated image so that it can be load with and without
    // settings
    let image_with_default_sampler =
        asset_server.load("textures/fantasy_ui_borders/panel-border-010.png");
    let image_with_repeated_sampler = asset_server.load_with_settings(
        "textures/fantasy_ui_borders/panel-border-010-repeated.png",
        |s: &mut _| {
            *s = ImageLoaderSettings {
                sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
                    // rewriting mode to repeat image,
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                }),
                ..default()
            }
        },
    );

    // Central rectangle - shows the texture with default clamping behavior
    // This serves as our control/reference to see the difference
    commands.spawn((
        // Create a square mesh
        Mesh2d(meshes.add(Rectangle::new(RECTANGLE_SIDE, RECTANGLE_SIDE))),
        // Apply a material with our texture
        MeshMaterial2d(materials.add(ColorMaterial {
            texture: Some(image_with_default_sampler.clone()),
            ..default()
        })),
        // Position at world origin
        Transform::from_translation(Vec3::ZERO),
        // The children! macro is a convenient way to spawn child entities
        // This creates a text label as a child of the rectangle
        children![(
            Text2d::new("Control"),
            // Position the text below the rectangle
            Transform::from_xyz(0., LABEL_OFFSET, 0.),
        )],
    ));

    // Left rectangle - demonstrates texture repeating
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(RECTANGLE_SIDE, RECTANGLE_SIDE))),
        MeshMaterial2d(materials.add(ColorMaterial {
            texture: Some(image_with_repeated_sampler),
            // UV transform modifies how texture coordinates map to the mesh.
            // Scaling UV coordinates is like zooming out on the texture:
            // - Scale 2.0 means we see 2x as much texture (it repeats twice)
            // - Scale 0.5 means we see half the texture (it's zoomed in)
            // Affine2 can also handle rotation and translation of textures!
            uv_transform: Affine2::from_scale(Vec2::new(2., 3.)),
            ..default()
        })),
        // Position to the left of center
        Transform::from_xyz(-RECTANGLE_OFFSET, 0.0, 0.0),
        children![(
            Text2d::new("Repeat On"),
            Transform::from_xyz(0., LABEL_OFFSET, 0.),
        )],
    ));

    // Right rectangle - shows what happens with clamping when UV coords exceed [0,1]
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(RECTANGLE_SIDE, RECTANGLE_SIDE))),
        MeshMaterial2d(materials.add(ColorMaterial {
            // Using the default sampler (clamp mode)
            // When UV coordinates go beyond [0,1], the edge pixels are stretched.
            // This creates a "smearing" effect where the border colors extend
            // to fill the rest of the space. This is often undesirable for
            // repeating patterns but useful for UI elements.
            texture: Some(image_with_default_sampler),

            // Same UV scaling as the left rectangle for comparison
            // The UV scale makes the texture coordinates go from 0 to 2 horizontally
            // and 0 to 3 vertically, but clamping prevents actual repetition
            uv_transform: Affine2::from_scale(Vec2::new(2., 3.)),
            ..default()
        })),
        // Position to the right of center
        Transform::from_xyz(RECTANGLE_OFFSET, 0.0, 0.0),
        children![(
            Text2d::new("Repeat Off"),
            Transform::from_xyz(0., LABEL_OFFSET, 0.),
        )],
    ));

    // camera
    commands.spawn((
        Camera2d,
        Transform::default().looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
