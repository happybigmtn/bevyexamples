//! This example illustrates how to use [`States`] for high-level app control flow.
//! States are a powerful but intuitive tool for controlling which logic runs when.
//! You can have multiple independent states, and the [`OnEnter`] and [`OnExit`] schedules
//! can be used to great effect to ensure that you handle setup and teardown appropriately.
//!
//! In this case, we're transitioning from a `Menu` state to an `InGame` state.
//!
//! States are like different modes of your application - think of a DVD player with
//! Menu, Playing, and Paused modes. Each state has its own behavior, and transitions
//! between states trigger setup and cleanup automatically. It's like having different
//! apps within your app!

use bevy::{dev_tools::states::*, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Initialize our state machine - starts in default state (Menu)
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        
        // STATE TRANSITION SYSTEMS - Run once during state changes
        // OnEnter: Called when transitioning TO this state
        .add_systems(OnEnter(AppState::Menu), setup_menu)
        // OnExit: Called when transitioning FROM this state
        .add_systems(OnExit(AppState::Menu), cleanup_menu)
        .add_systems(OnEnter(AppState::InGame), setup_game)
        
        // STATE-SPECIFIC UPDATE SYSTEMS - Run every frame when in state
        // run_if ensures these only execute in the correct state
        .add_systems(Update, menu.run_if(in_state(AppState::Menu)))
        .add_systems(
            Update,
            (movement, change_color).run_if(in_state(AppState::InGame)),
        )
        
        // Debug system to log state transitions
        .add_systems(Update, log_transitions::<AppState>)
        .run();
}

// Define our application states
// States must derive specific traits to work with Bevy's state system
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]  // Starting state
    Menu,       // Main menu with "Play" button
    InGame,     // Gameplay with sprite movement
}

// Resource to track menu entities for cleanup
#[derive(Resource)]
struct MenuData {
    button_entity: Entity,
}

// Button color states - visual feedback for interaction
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);  // Dark gray
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25); // Lighter gray
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35); // Green!

// Runs once at app startup (regardless of state)
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// OnEnter(Menu) - Creates the menu UI
fn setup_menu(mut commands: Commands) {
    let button_entity = commands
        .spawn((
            Node {
                // Full-screen container to center the button
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            children![(
                Button,
                Node {
                    width: Val::Px(150.),
                    height: Val::Px(65.),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(NORMAL_BUTTON),
                children![(
                    Text::new("Play"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                )],
            )],
        ))
        .id();
    commands.insert_resource(MenuData { button_entity });
}

// Menu update system - handles button interaction
fn menu(
    mut next_state: ResMut<NextState<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                // TRIGGER STATE TRANSITION!
                // This queues a transition to InGame state
                next_state.set(AppState::InGame);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

// OnExit(Menu) - Clean up menu entities
// This prevents menu UI from persisting in game
fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.button_entity).despawn();
}

// OnEnter(InGame) - Spawn game entities
fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a sprite that can be moved with arrow keys
    commands.spawn(Sprite::from_image(asset_server.load("branding/icon.png")));
}

// InGame systems - only run when in InGame state
const SPEED: f32 = 100.0;
fn movement(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Sprite>>,
) {
    for mut transform in &mut query {
        // Build movement vector from input
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        if input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }

        // Apply movement if any input detected
        if direction != Vec3::ZERO {
            transform.translation += direction.normalize() * SPEED * time.delta_secs();
        }
    }
}

// Another InGame system - animates sprite color
fn change_color(time: Res<Time>, mut query: Query<&mut Sprite>) {
    for mut sprite in &mut query {
        // Oscillate blue channel using sine wave
        let new_color = LinearRgba {
            blue: ops::sin(time.elapsed_secs() * 0.5) + 2.0,
            ..LinearRgba::from(sprite.color)
        };

        sprite.color = new_color.into();
    }
}
