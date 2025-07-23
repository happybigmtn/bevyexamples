//! This example shows how to render 2D objects on top of Bevy UI, by using a second camera with a higher `order` than the UI camera.
//! 
//! ## Key Concepts Demonstrated
//! 
//! - **Camera Ordering**: Using the `order` field to control which camera renders first/last
//! - **Render Layers**: Separating what each camera sees using `RenderLayers`
//! - **Clear Color Config**: Making cameras transparent to composite multiple views
//! - **UI Camera Setup**: Understanding the default UI camera and custom camera targeting
//! 
//! ## Visual Result
//! 
//! You'll see a rose-colored UI background with a white rounded border box in the center.
//! A rotating sprite character appears on top of the UI, demonstrating proper layering.

use bevy::{
    // The `tailwind` module provides pre-defined colors based on the Tailwind CSS color palette
    color::palettes::tailwind,
    prelude::*,
    // RenderLayers is used to control which entities are visible to which cameras
    render::view::RenderLayers,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_sprite)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // CAMERA 1: The Default UI Camera
    // ================================
    // This camera is responsible for rendering UI elements. By adding `IsDefaultUiCamera`,
    // we tell Bevy that all UI nodes should render to this camera by default.
    // 
    // Camera Order: 0 (implicit default)
    // Render Layers: All layers (default behavior when RenderLayers is not specified)
    // Clear Color: Uses the default clear color (typically black or as configured)
    commands.spawn((Camera2d, IsDefaultUiCamera));

    // CAMERA 2: The Overlay Camera for 2D Sprites
    // ===========================================
    // This camera renders AFTER the UI camera due to its higher order value.
    // It's used to draw 2D sprites on top of the UI elements.
    commands.spawn((
        Camera2d,
        Camera {
            // The `order` field determines the rendering sequence:
            // - Lower values render first (background)
            // - Higher values render last (foreground)
            // Since the default camera has order 0, this camera with order 1
            // will render after it, making its content appear on top
            order: 1,
            
            // ClearColorConfig::None means "don't clear the screen before rendering"
            // This is crucial! If we cleared the screen, we'd erase everything
            // the first camera drew (including our UI). Instead, we render
            // on top of the existing framebuffer content.
            clear_color: ClearColorConfig::None,
            
            // The `..default()` syntax in Rust means "use default values for
            // all other fields I haven't explicitly set"
            ..default()
        },
        // RenderLayers::layer(1) restricts this camera to ONLY render entities
        // that also have RenderLayers::layer(1). This prevents the camera from
        // rendering the UI elements (which are on the default layer 0).
        // 
        // Think of render layers like transparent sheets of paper:
        // - Layer 0 (default): Contains UI elements
        // - Layer 1: Contains our 2D sprites
        // Each camera can choose which layers to "see"
        RenderLayers::layer(1),
    ));

    // UI SETUP: Creating the User Interface
    // ======================================
    // This creates a full-screen UI with a centered white border box on a rose background.
    // Since we didn't specify a render layer, this UI will be on the default layer 0,
    // which means only the first camera (with IsDefaultUiCamera) will render it.
    commands.spawn((
        // The root UI node that fills the entire screen
        Node {
            // Val::Percent(100.) means "take up 100% of the parent's size"
            // Since this is a root node, the parent is the window itself
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            
            // Flexbox layout properties:
            display: Display::Flex,                      // Use flexbox layout
            justify_content: JustifyContent::Center,     // Center children horizontally
            align_items: AlignItems::Center,             // Center children vertically
            ..default()
        },
        // BackgroundColor sets the fill color for this UI node
        // The `.into()` converts the tailwind color to Bevy's Color type
        BackgroundColor(tailwind::ROSE_400.into()),
        
        // The children! macro is a convenient way to define child entities
        // This creates a single child node with a white rounded border
        children![(
            Node {
                // Size as percentage of parent, with minimum pixel constraints
                height: Val::Percent(30.),    // 30% of parent height
                width: Val::Percent(20.),     // 20% of parent width
                min_height: Val::Px(150.),    // But at least 150 pixels tall
                min_width: Val::Px(150.),     // And at least 150 pixels wide
                
                // UiRect::all() applies the same value to all four sides
                border: UiRect::all(Val::Px(2.)),  // 2-pixel border on all sides
                ..default()
            },
            // BorderRadius makes the corners rounded
            // Using percentage means the radius is relative to the node's size
            BorderRadius::all(Val::Percent(25.0)),
            
            // BorderColor defines the color of the border we specified above
            BorderColor::all(Color::WHITE),
        )],
    ));

    // 2D SPRITE: The Overlay Content
    // ==============================
    // This sprite will appear on top of the UI because:
    // 1. It's on render layer 1 (matching the second camera)
    // 2. The second camera has a higher order value
    // 3. The second camera doesn't clear the screen
    commands.spawn((
        Sprite {
            // Load a texture from the assets folder
            // The asset_server handles loading files asynchronously
            image: asset_server.load("textures/rpg/chars/sensei/sensei.png"),
            
            // custom_size overrides the sprite's natural size from the texture
            // Some() wraps the value in Rust's Option type (can be Some(value) or None)
            custom_size: Some(Vec2::new(100., 100.)),
            ..default()
        },
        // CRITICAL: This sprite has RenderLayers::layer(1), which means:
        // - The first camera (UI camera) will NOT render this sprite
        // - The second camera (overlay camera) WILL render this sprite
        // Without this, the sprite would appear behind the UI!
        RenderLayers::layer(1),
    ));
}

/// Continuously rotates the sprite to demonstrate that 2D rendering features work normally
/// even when rendering on top of UI elements.
fn rotate_sprite(
    time: Res<Time>,
    // Single<T> is a system parameter that expects exactly one entity matching the query
    // It will panic if there are 0 or more than 1 entities with both Transform and Sprite
    mut sprite: Single<&mut Transform, With<Sprite>>
) {
    // Create rotation quaternions for two different axes
    // Quaternions are a mathematical way to represent rotations without gimbal lock
    
    // Rotation around Z-axis (spinning like a wheel)
    // delta_secs() gives us the time since last frame in seconds
    // Multiplying by 0.5 makes it rotate at half a radian per second
    let z_rotation = Quat::from_rotation_z(time.delta_secs() * 0.5);
    
    // Rotation around Y-axis (spinning like a coin)
    let y_rotation = Quat::from_rotation_y(time.delta_secs());
    
    // Combine rotations by multiplying quaternions
    // Order matters! z_rotation * y_rotation would give a different result
    // The *= operator modifies the rotation in place
    sprite.rotation *= z_rotation * y_rotation;
    
    // Why this works on top of UI:
    // The sprite is being rendered by a completely separate camera pass.
    // Even though it's rotating in 3D space, it still appears on top of
    // the UI because camera order determines the final composition.
}
