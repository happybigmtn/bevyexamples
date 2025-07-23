//! # UI Directional Navigation Example
//! 
//! This example demonstrates how to create accessible, keyboard and gamepad-navigable UI systems in Bevy 0.16.
//! 
//! ## What You'll Learn
//! - How to set up directional navigation with `DirectionalNavigationPlugin`
//! - Creating navigation graphs that connect UI elements spatially
//! - Handling keyboard and gamepad input for navigation
//! - Managing focus states and visual feedback for accessibility
//! - Converting navigation input into simulated click events
//! - Building input action mapping systems for flexible control schemes
//! - Creating grid-based UI layouts that support navigation
//! 
//! ## Key Concepts
//! - **Directional Navigation**: Moving between UI elements based on spatial relationships (up/down/left/right)
//! - **Navigation Graph**: A data structure that defines which elements can be reached from each other
//! - **Input Focus**: The currently selected UI element that will receive input
//! - **Focus Visibility**: Whether focus indicators should be displayed to the user
//! - **Action Mapping**: Converting raw input (keys/buttons) into semantic actions
//! - **Compass Octants**: 8-directional system (N, NE, E, SE, S, SW, W, NW) for navigation
//! - **Spatial UI**: UI design that considers the physical arrangement of elements
//! 
//! ## Navigation vs Tab Navigation
//! - **Directional Navigation**: Moves based on visual/spatial relationships (like a TV remote)
//! - **Tab Navigation**: Moves through elements in document order (like web pages)
//! - Directional navigation is more intuitive for grid layouts and game-like interfaces
//! 
//! ## Controls
//! - Arrow Keys or D-Pad: Navigate between buttons
//! - Enter or South gamepad button (A): Activate focused button
//! - Visual focus indicator shows which button is currently selected

use std::time::Duration;

// Essential imports for building a navigable UI system
use bevy::{
    input_focus::{
        directional_navigation::{
            DirectionalNavigation,     // System for moving focus between elements
            DirectionalNavigationMap, // Graph structure connecting navigable elements
            DirectionalNavigationPlugin, // Plugin that enables navigation functionality
        },
        InputDispatchPlugin,    // Plugin for managing input focus
        InputFocus,            // Resource tracking which element has focus
        InputFocusVisible,     // Resource controlling focus indicator visibility
    },
    math::{CompassOctant, FloatOrd}, // Direction system and float ordering
    picking::{
        backend::HitData,      // Data about what was clicked/interacted with
        pointer::{Location, PointerId}, // Pointer location and identification
    },
    platform::collections::{HashMap, HashSet}, // Cross-platform collection types
    prelude::*,                                 // Core Bevy types
    render::camera::NormalizedRenderTarget,     // Camera target for pointer events
};

fn main() {
    App::new()
        // Input focus and navigation are not enabled by default in Bevy
        // We need to explicitly add these plugins to enable the functionality
        .add_plugins((
            DefaultPlugins,                    // Standard Bevy systems
            InputDispatchPlugin,               // Manages input focus and dispatch
            DirectionalNavigationPlugin,       // Enables directional navigation
        ))
        
        // InputFocusVisible controls whether focus indicators are shown to users
        // It defaults to false (hidden), but we want visible feedback for this demo
        .insert_resource(InputFocusVisible(true))
        
        // Initialize our custom resource for tracking input actions
        // This creates an action mapping layer between raw input and game actions
        .init_resource::<ActionState>()
        
        // Set up the UI once when the app starts
        .add_systems(Startup, setup_ui)
        
        // Input processing happens during PreUpdate to ensure consistency
        // We chain these systems to guarantee processing order:
        // 1. Read input and convert to actions
        // 2. Use actions to navigate the UI
        .add_systems(PreUpdate, (process_inputs, navigate).chain())
        
        // Visual and interaction systems run during the main Update phase
        .add_systems(
            Update,
            (
                // Update visual focus indicators to show which element is selected
                highlight_focused_element,
                
                // Convert navigation "select" action into simulated click events
                interact_with_focused_button,
                
                // Handle button press animations and reset them after a timeout
                reset_button_after_interaction,
            ),
        )
        
        // Global observer that responds to button click events from any source
        // This demonstrates unified handling of mouse clicks and simulated clicks
        .add_observer(universal_button_click_behavior)
        .run(); // Start the application loop
}

// Color scheme constants for consistent visual feedback
// Using Tailwind color palette provides professional, accessible color choices
const NORMAL_BUTTON: Srgba = bevy::color::palettes::tailwind::BLUE_400;  // Default button color
const PRESSED_BUTTON: Srgba = bevy::color::palettes::tailwind::BLUE_500; // Darker when pressed
const FOCUSED_BORDER: Srgba = bevy::color::palettes::tailwind::BLUE_50;  // Light border for focus

// Observer function that responds to button click events from any source
// This creates unified behavior for both mouse clicks and keyboard/gamepad interactions
// In a real application, each button would have additional unique behaviors
fn universal_button_click_behavior(
    mut trigger: Trigger<Pointer<Click>>,  // Event data including which entity was clicked
    mut button_query: Query<(&mut BackgroundColor, &mut ResetTimer)>,
) {
    let button_entity = trigger.target(); // Get the entity that was clicked
    
    // Check if the clicked entity is a button with the required components
    if let Ok((mut color, mut reset_timer)) = button_query.get_mut(button_entity) {
        // Visual feedback: change button color to indicate it was pressed
        color.0 = PRESSED_BUTTON.into();
        
        // Start a timer to reset the button color after a short delay
        // This creates a brief "flash" animation to confirm the interaction
        reset_timer.0 = Timer::from_seconds(0.3, TimerMode::Once);

        // Stop event propagation to prevent parent elements from also responding
        // This is important in hierarchical UI structures
        trigger.propagate(false);
        
        // TODO: In a real game, this would be an excellent place to:
        // - Play button click sound effects
        // - Trigger haptic feedback on gamepads
        // - Log user interaction analytics
    }
}

// Component that automatically resets UI elements after a timed delay
// Deref and DerefMut allow us to use this like a Timer directly
// This pattern is useful for temporary visual effects
#[derive(Component, Default, Deref, DerefMut)]
struct ResetTimer(Timer);

fn reset_button_after_interaction(
    time: Res<Time>,
    mut query: Query<(&mut ResetTimer, &mut BackgroundColor)>,
) {
    for (mut reset_timer, mut color) in query.iter_mut() {
        reset_timer.tick(time.delta());
        if reset_timer.just_finished() {
            color.0 = NORMAL_BUTTON.into();
        }
    }
}

// We're spawning a simple grid of buttons and some instructions
// The buttons are just colored rectangles with text displaying the button's name
fn setup_ui(
    mut commands: Commands,
    mut directional_nav_map: ResMut<DirectionalNavigationMap>,
    mut input_focus: ResMut<InputFocus>,
) {
    const N_ROWS: u16 = 5;
    const N_COLS: u16 = 3;

    // Rendering UI elements requires a camera
    commands.spawn(Camera2d);

    // Create a full-screen background node
    let root_node = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .id();

    // Add instruction to the left of the grid
    let instructions = commands
        .spawn((
            Text::new("Use arrow keys or D-pad to navigate. \
            Click the buttons, or press Enter / the South gamepad button to interact with the focused button."),
            Node {
                width: Val::Px(300.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(12.0)),
                ..default()
            },
        ))
        .id();

    // Set up the root entity to hold the grid
    let grid_root_entity = commands
        .spawn(Node {
            display: Display::Grid,
            // Allow the grid to take up the full height and the rest of the width of the window
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            // Set the number of rows and columns in the grid
            // allowing the grid to automatically size the cells
            grid_template_columns: RepeatedGridTrack::auto(N_COLS),
            grid_template_rows: RepeatedGridTrack::auto(N_ROWS),
            ..default()
        })
        .id();

    // Add the instructions and grid to the root node
    commands
        .entity(root_node)
        .add_children(&[instructions, grid_root_entity]);

    let mut button_entities: HashMap<(u16, u16), Entity> = HashMap::default();
    for row in 0..N_ROWS {
        for col in 0..N_COLS {
            let button_name = format!("Button {}-{}", row, col);

            let button_entity = commands
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(120.0),
                        // Add a border so we can show which element is focused
                        border: UiRect::all(Val::Px(4.0)),
                        // Center the button's text label
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        // Center the button within the grid cell
                        align_self: AlignSelf::Center,
                        justify_self: JustifySelf::Center,
                        ..default()
                    },
                    ResetTimer::default(),
                    BorderRadius::all(Val::Px(16.0)),
                    BackgroundColor::from(NORMAL_BUTTON),
                    Name::new(button_name.clone()),
                ))
                // Add a text element to the button
                .with_child((
                    Text::new(button_name),
                    // And center the text if it flows onto multiple lines
                    TextLayout {
                        justify: JustifyText::Center,
                        ..default()
                    },
                ))
                .id();

            // Add the button to the grid
            commands.entity(grid_root_entity).add_child(button_entity);

            // Keep track of the button entities so we can set up our navigation graph
            button_entities.insert((row, col), button_entity);
        }
    }

    // Connect all of the buttons in the same row to each other,
    // looping around when the edge is reached.
    for row in 0..N_ROWS {
        let entities_in_row: Vec<Entity> = (0..N_COLS)
            .map(|col| button_entities.get(&(row, col)).unwrap())
            .copied()
            .collect();
        directional_nav_map.add_looping_edges(&entities_in_row, CompassOctant::East);
    }

    // Connect all of the buttons in the same column to each other,
    // but don't loop around when the edge is reached.
    // While looping is a very reasonable choice, we're not doing it here to demonstrate the different options.
    for col in 0..N_COLS {
        let entities_in_column: Vec<Entity> = (0..N_ROWS)
            .map(|row| button_entities.get(&(row, col)).unwrap())
            .copied()
            .collect();

        directional_nav_map.add_edges(&entities_in_column, CompassOctant::South);
    }

    // When changing scenes, remember to set an initial focus!
    let top_left_entity = *button_entities.get(&(0, 0)).unwrap();
    input_focus.set(top_left_entity);
}

// Action mapping enum - the key to flexible input handling
// This creates a layer of abstraction between physical inputs and game actions
// Benefits:
// - Easy remapping of controls without changing game logic
// - Support for multiple input devices (keyboard + gamepad)
// - Cleaner, more readable code than checking raw input everywhere
// - Consistent behavior across different input methods
#[derive(Debug, PartialEq, Eq, Hash)]
enum DirectionalNavigationAction {
    Up,     // Move focus to element above current
    Down,   // Move focus to element below current  
    Left,   // Move focus to element left of current
    Right,  // Move focus to element right of current
    Select, // Activate/click the currently focused element
}

impl DirectionalNavigationAction {
    fn variants() -> Vec<Self> {
        vec![
            DirectionalNavigationAction::Up,
            DirectionalNavigationAction::Down,
            DirectionalNavigationAction::Left,
            DirectionalNavigationAction::Right,
            DirectionalNavigationAction::Select,
        ]
    }

    fn keycode(&self) -> KeyCode {
        match self {
            DirectionalNavigationAction::Up => KeyCode::ArrowUp,
            DirectionalNavigationAction::Down => KeyCode::ArrowDown,
            DirectionalNavigationAction::Left => KeyCode::ArrowLeft,
            DirectionalNavigationAction::Right => KeyCode::ArrowRight,
            DirectionalNavigationAction::Select => KeyCode::Enter,
        }
    }

    fn gamepad_button(&self) -> GamepadButton {
        match self {
            DirectionalNavigationAction::Up => GamepadButton::DPadUp,
            DirectionalNavigationAction::Down => GamepadButton::DPadDown,
            DirectionalNavigationAction::Left => GamepadButton::DPadLeft,
            DirectionalNavigationAction::Right => GamepadButton::DPadRight,
            // This is the "A" button on an Xbox controller,
            // and is conventionally used as the "Select" / "Interact" button in many games
            DirectionalNavigationAction::Select => GamepadButton::South,
        }
    }
}

// This keeps track of the inputs that are currently being pressed
#[derive(Default, Resource)]
struct ActionState {
    pressed_actions: HashSet<DirectionalNavigationAction>,
}

fn process_inputs(
    mut action_state: ResMut<ActionState>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepad_input: Query<&Gamepad>,
) {
    // Reset the set of pressed actions each frame
    // to ensure that we only process each action once
    action_state.pressed_actions.clear();

    for action in DirectionalNavigationAction::variants() {
        // Use just_pressed to ensure that we only process each action once
        // for each time it is pressed
        if keyboard_input.just_pressed(action.keycode()) {
            action_state.pressed_actions.insert(action);
        }
    }

    // We're treating this like a single-player game:
    // if multiple gamepads are connected, we don't care which one is being used
    for gamepad in gamepad_input.iter() {
        for action in DirectionalNavigationAction::variants() {
            // Unlike keyboard input, gamepads are bound to a specific controller
            if gamepad.just_pressed(action.gamepad_button()) {
                action_state.pressed_actions.insert(action);
            }
        }
    }
}

fn navigate(action_state: Res<ActionState>, mut directional_navigation: DirectionalNavigation) {
    // If the user is pressing both left and right, or up and down,
    // we should not move in either direction.
    let net_east_west = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Right) as i8
        - action_state
            .pressed_actions
            .contains(&DirectionalNavigationAction::Left) as i8;

    let net_north_south = action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Up) as i8
        - action_state
            .pressed_actions
            .contains(&DirectionalNavigationAction::Down) as i8;

    // Compute the direction that the user is trying to navigate in
    let maybe_direction = match (net_east_west, net_north_south) {
        (0, 0) => None,
        (0, 1) => Some(CompassOctant::North),
        (1, 1) => Some(CompassOctant::NorthEast),
        (1, 0) => Some(CompassOctant::East),
        (1, -1) => Some(CompassOctant::SouthEast),
        (0, -1) => Some(CompassOctant::South),
        (-1, -1) => Some(CompassOctant::SouthWest),
        (-1, 0) => Some(CompassOctant::West),
        (-1, 1) => Some(CompassOctant::NorthWest),
        _ => None,
    };

    if let Some(direction) = maybe_direction {
        match directional_navigation.navigate(direction) {
            // In a real game, you would likely want to play a sound or show a visual effect
            // on both successful and unsuccessful navigation attempts
            Ok(entity) => {
                println!("Navigated {direction:?} successfully. {entity} is now focused.");
            }
            Err(e) => println!("Navigation failed: {e}"),
        }
    }
}

fn highlight_focused_element(
    input_focus: Res<InputFocus>,
    // While this isn't strictly needed for the example,
    // we're demonstrating how to be a good citizen by respecting the `InputFocusVisible` resource.
    input_focus_visible: Res<InputFocusVisible>,
    mut query: Query<(Entity, &mut BorderColor)>,
) {
    for (entity, mut border_color) in query.iter_mut() {
        if input_focus.0 == Some(entity) && input_focus_visible.0 {
            // Don't change the border size / radius here,
            // as it would result in wiggling buttons when they are focused
            *border_color = BorderColor::all(FOCUSED_BORDER.into());
        } else {
            *border_color = BorderColor::DEFAULT;
        }
    }
}

// By sending a Pointer<Click> trigger rather than directly handling button-like interactions,
// we can unify our handling of pointer and keyboard/gamepad interactions
fn interact_with_focused_button(
    action_state: Res<ActionState>,
    input_focus: Res<InputFocus>,
    mut commands: Commands,
) {
    if action_state
        .pressed_actions
        .contains(&DirectionalNavigationAction::Select)
    {
        if let Some(focused_entity) = input_focus.0 {
            commands.trigger_targets(
                Pointer::<Click> {
                    target: focused_entity,
                    // We're pretending that we're a mouse
                    pointer_id: PointerId::Mouse,
                    // This field isn't used, so we're just setting it to a placeholder value
                    pointer_location: Location {
                        target: NormalizedRenderTarget::Image(
                            bevy::render::camera::ImageRenderTarget {
                                handle: Handle::default(),
                                scale_factor: FloatOrd(1.0),
                            },
                        ),
                        position: Vec2::ZERO,
                    },
                    event: Click {
                        button: PointerButton::Primary,
                        // This field isn't used, so we're just setting it to a placeholder value
                        hit: HitData {
                            camera: Entity::PLACEHOLDER,
                            depth: 0.0,
                            position: None,
                            normal: None,
                        },
                        duration: Duration::from_secs_f32(0.1),
                    },
                },
                focused_entity,
            );
        }
    }
}
