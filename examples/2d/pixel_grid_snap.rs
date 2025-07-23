//! Shows how to create graphics that snap to the pixel grid by rendering to a texture in 2D
//!
//! Pixel-perfect rendering is like creating art with graph paper - every pixel must align
//! exactly to the grid! This is crucial for retro games, pixel art, and any time you want
//! that crisp, chunky aesthetic. The trick is to render your game at a low resolution
//! (like 160x90) onto a texture, then scale that texture up to fill the screen. Think of it
//! like painting a tiny masterpiece, then using a magnifying glass to make it bigger -
//! each pixel stays sharp and square instead of getting blurry!

use bevy::{
    color::palettes::css::GRAY,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::WindowResized,
};

/// In-game resolution width.
// THE TINY CANVAS - Our actual game resolution (10x smaller than HD!)
const RES_WIDTH: u32 = 160;  // Classic retro width

/// In-game resolution height.
const RES_HEIGHT: u32 = 90;   // 16:9 aspect ratio at low res

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
// THE PIXEL ART LAYER - Everything here gets the chunky treatment
const PIXEL_PERFECT_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
// THE SMOOTH LAYER - UI and other elements that should stay crisp
const HIGH_RES_LAYERS: RenderLayers = RenderLayers::layer(1);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup_camera, setup_sprite, setup_mesh))
        .add_systems(Update, (rotate, fit_canvas))
        .run();
}

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
// THE PIXEL PAINTING - Our tiny texture that holds the game world
#[derive(Component)]
struct Canvas;  // Like a tiny canvas that we'll magnify later

/// Camera that renders the pixel-perfect world to the [`Canvas`].
// THE MINIATURE PHOTOGRAPHER - Captures the game at low resolution
#[derive(Component)]
struct InGameCamera;  // Takes a 160x90 "photo" of the game world

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
// THE MAGNIFYING GLASS - Shows the tiny canvas at full screen size
#[derive(Component)]
struct OuterCamera;  // Scales up our pixel art without blur

#[derive(Component)]
struct Rotate;

fn setup_sprite(mut commands: Commands, asset_server: Res<AssetServer>) {
    // PIXEL ART SPRITE - This one gets the retro treatment
    // The sample sprite that will be rendered to the pixel-perfect canvas
    commands.spawn((
        Sprite::from_image(asset_server.load("pixel/bevy_pixel_dark.png")),
        Transform::from_xyz(-45., 20., 2.),
        Rotate,
        PIXEL_PERFECT_LAYERS,  // Goes on the low-res layer - will snap to pixels!
    ));

    // HIGH-RES SPRITE - This one stays smooth and crisp
    // The sample sprite that will be rendered to the high-res "outer world"
    commands.spawn((
        Sprite::from_image(asset_server.load("pixel/bevy_pixel_light.png")),
        Transform::from_xyz(-45., -20., 2.),
        Rotate,
        HIGH_RES_LAYERS,  // Goes on the high-res layer - no pixel snapping!
    ));
}

/// Spawns a capsule mesh on the pixel-perfect layer.
// GEOMETRIC PIXEL ART - Even smooth shapes become chunky!
fn setup_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Capsule2d::default())),
        MeshMaterial2d(materials.add(Color::BLACK)),
        Transform::from_xyz(25., 0., 2.).with_scale(Vec3::splat(32.)),
        Rotate,
        PIXEL_PERFECT_LAYERS,  // Watch the smooth capsule become pixelated!
    ));
}

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // CANVAS DIMENSIONS - Our tiny pixel art canvas
    let canvas_size = Extent3d {
        width: RES_WIDTH,   // 160 pixels wide
        height: RES_HEIGHT, // 90 pixels tall
        ..default()
    };

    // THE RENDER TARGET - Create a texture to draw our low-res game onto
    // This Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,    // No mipmaps - we want crisp pixels!
            sample_count: 1,       // No anti-aliasing - keep it chunky!
            usage: TextureUsages::TEXTURE_BINDING    // Can be used as a texture
                | TextureUsages::COPY_DST            // Can receive rendered data  
                | TextureUsages::RENDER_ATTACHMENT,  // Can be rendered to
            view_formats: &[],
        },
        ..default()
    };

    // INITIALIZE CANVAS - Start with a blank slate
    // Fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // THE LOW-RES CAMERA - Renders the game world to our tiny canvas
    // This camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Camera2d,
        Camera {
            // Render before the "main pass" camera
            order: -1,  // Render first (lower number = earlier)
            target: RenderTarget::Image(image_handle.clone().into()),  // Draw to texture, not screen!
            clear_color: ClearColorConfig::Custom(GRAY.into()),
            ..default()
        },
        Msaa::Off,  // No anti-aliasing - we want those crispy pixels!
        InGameCamera,
        PIXEL_PERFECT_LAYERS,  // Only sees entities on the pixel-perfect layer
    ));

    // THE CANVAS SPRITE - Turn our render texture into a visible sprite
    // Spawn the canvas
    commands.spawn((Sprite::from_image(image_handle), Canvas, HIGH_RES_LAYERS));
    // This sprite will be scaled up to fill the screen!

    // THE SCREEN CAMERA - Shows the magnified canvas to the player
    // The "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((Camera2d, Msaa::Off, OuterCamera, HIGH_RES_LAYERS));
    // This camera sees: 1) The canvas sprite (our pixel art), 2) Any high-res UI
}

/// Rotates entities to demonstrate grid snapping.
// THE ROTATION TEST - Watch pixels snap to the grid as things spin!
fn rotate(time: Res<Time>, mut transforms: Query<&mut Transform, With<Rotate>>) {
    for mut transform in &mut transforms {
        let dt = time.delta_secs();
        transform.rotate_z(dt);  // Smooth rotation, pixelated result!
        // The rotation is smooth, but pixels can only be on or off -
        // creating a "stepped" appearance as the sprite rotates
    }
}

/// Scales camera projection to fit the window (integer multiples only).
// THE PIXEL-PERFECT SCALER - Ensures pixels stay square when resizing!
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projection: Single<&mut Projection, With<OuterCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    for event in resize_events.read() {
        // CALCULATE SCALE OPTIONS - How many times can our canvas fit?
        let h_scale = event.width / RES_WIDTH as f32;   // Horizontal fit
        let v_scale = event.height / RES_HEIGHT as f32;  // Vertical fit
        
        // INTEGER SCALING - Round to whole numbers to keep pixels square!
        projection.scale = 1. / h_scale.min(v_scale).round();
        // We pick the smaller scale and round it - this ensures:
        // 1. The entire canvas fits on screen
        // 2. Pixels remain perfectly square (no stretching!)
        // 3. We might get black bars, but pixels stay crisp
    }
}
