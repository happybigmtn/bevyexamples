//! Animates a sprite in response to a keyboard event.
//!
//! See `sprite_sheet.rs` for an example where the sprite animation loops indefinitely.
//!
//! Sprite animation is like a digital flipbook - you rapidly show different frames
//! to create the illusion of movement! Just as early animators drew each frame by hand
//! and flipped through them quickly, we cycle through sprite atlas indices at controlled
//! intervals. The magic happens when our brains perceive discrete images as continuous motion.

use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

fn main() {
    App::new()
        // PIXEL ART PRESERVATION - Keep sprites crisp!
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_systems(Startup, setup)
        // CONTINUOUS ANIMATION PROCESSING - Runs every frame
        .add_systems(Update, execute_animations)
        .add_systems(
            Update,
            (
                // EVENT-DRIVEN ANIMATION TRIGGERS - Only run when keys are pressed
                // Press the right arrow key to animate the right sprite
                trigger_animation::<RightSprite>.run_if(input_just_pressed(KeyCode::ArrowRight)),
                // Press the left arrow key to animate the left sprite
                trigger_animation::<LeftSprite>.run_if(input_just_pressed(KeyCode::ArrowLeft)),
            ),
        )
        .run();
}

// ANIMATION TRIGGER - Like pressing "play" on a flipbook
// This system runs when the user clicks the left arrow key or right arrow key
// Generic function works for any sprite type (LeftSprite or RightSprite)
fn trigger_animation<S: Component>(mut animation: Single<&mut AnimationConfig, With<S>>) {
    // RESET THE ANIMATION TIMER - Start fresh!
    // We create a new timer when the animation is triggered
    // This restarts the animation from the beginning
    animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
}

// ANIMATION BLUEPRINT - Defines how a sprite should animate
#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,  // Starting frame in the sprite sheet
    last_sprite_index: usize,   // Ending frame (animation loops back after this)
    fps: u8,                    // Frames per second - how fast to flip
    frame_timer: Timer,         // Countdown timer for frame transitions
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            // Initialize with a timer that immediately triggers
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    // FPS TO DURATION CONVERSION - Math for timing!
    // Convert frames-per-second to time-per-frame
    fn timer_from_fps(fps: u8) -> Timer {
        // If fps=10, then each frame lasts 1/10 = 0.1 seconds
        // TimerMode::Once means timer doesn't auto-repeat
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}

// THE ANIMATION ENGINE - Where the flipbook magic happens!
// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut config, mut sprite) in &mut query {
        // FRAME TIMING - Count down to the next frame
        // We track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // TIME TO FLIP THE PAGE?
        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                if atlas.index == config.last_sprite_index {
                    // END OF ANIMATION - Loop back to start but STOP
                    // ...and it IS the last frame, then we move back to the first frame and stop.
                    atlas.index = config.first_sprite_index;
                    // Note: Timer doesn't reset, so animation stays paused
                } else {
                    // ADVANCE TO NEXT FRAME - Continue the flipbook
                    // ...and it is NOT the last frame, then we move to the next frame...
                    atlas.index += 1;
                    // ...and reset the frame timer to start counting all over again
                    config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
                }
            }
        }
    }
}

// MARKER COMPONENTS - Tags to distinguish different sprites
// Like name tags at a party - help us identify which sprite is which
#[derive(Component)]
struct LeftSprite;

#[derive(Component)]
struct RightSprite;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d);

    // USER INSTRUCTIONS - Always help your players!
    // Create a minimal UI explaining how to interact with the example
    commands.spawn((
        Text::new("Left Arrow: Animate Left Sprite\nRight Arrow: Animate Right Sprite"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    // SPRITE SHEET LOADING - Get our flipbook pages
    // Load the sprite sheet using the `AssetServer`
    let texture = asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png");

    // ATLAS LAYOUT - Define how the sheet is organized
    // The sprite sheet has 7 sprites arranged in a row, and they are all 24px x 24px
    // from_grid creates a uniform grid layout for the sprites
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    // LEFT SPRITE SETUP - Slower animation (10 FPS)
    // The first (left-hand) sprite runs at 10 FPS
    // Skip frame 0 (idle), animate frames 1-6 (running)
    let animation_config_1 = AnimationConfig::new(1, 6, 10);

    // Create the first (left-hand) sprite
    commands.spawn((
        Sprite {
            image: texture.clone(),              // Share the same texture
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config_1.first_sprite_index,  // Start at frame 1
            }),
            ..default()
        },
        // Scale up 6x and position on the left
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(-70.0, 0.0, 0.0)),
        LeftSprite,           // Marker component
        animation_config_1,   // Animation settings
    ));

    // RIGHT SPRITE SETUP - Faster animation (20 FPS)
    // The second (right-hand) sprite runs at 20 FPS
    // Same frames (1-6) but twice as fast - compare the speeds!
    let animation_config_2 = AnimationConfig::new(1, 6, 20);

    // Create the second (right-hand) sprite
    commands.spawn((
        Sprite {
            image: texture.clone(),              // Same texture, different instance
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: animation_config_2.first_sprite_index,  // Also starts at frame 1
            }),
            ..Default::default()
        },
        // Same scale, but positioned on the right
        Transform::from_scale(Vec3::splat(6.0)).with_translation(Vec3::new(70.0, 0.0, 0.0)),
        RightSprite,          // Different marker component
        animation_config_2,   // Faster animation settings
    ));
}
