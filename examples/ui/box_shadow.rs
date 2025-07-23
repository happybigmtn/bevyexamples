//! # UI Box Shadow Example
//! 
//! This example demonstrates how to create and customize CSS-style box shadows in Bevy 0.16 UI.
//! 
//! ## What You'll Learn
//! - How to create box shadows using `BoxShadow` and `ShadowStyle`
//! - Understanding shadow properties: offset, blur, spread, color
//! - How to control shadow rendering quality with `BoxShadowSamples`
//! - Creating interactive UI controls with buttons and real-time updates
//! - Resource-driven UI systems for managing application state
//! - Button interaction handling with press, hold, and repeat behavior
//! - Dynamic UI updates using resource change detection
//! 
//! ## Key Concepts
//! - **Box Shadow**: A visual effect that creates the illusion of depth by casting shadows
//! - **ShadowStyle**: Defines individual shadow properties (color, position, blur, spread)
//! - **Shadow Offset**: How far the shadow is displaced from the element (x, y)
//! - **Shadow Blur**: How soft/diffused the shadow edges are
//! - **Shadow Spread**: How much the shadow expands beyond the element's size
//! - **Shadow Samples**: Quality setting for shadow rendering (higher = smoother but slower)
//! - **Interactive UI**: Buttons that respond to user input and update visual properties
//! 
//! ## Controls
//! - Use the control panel to adjust shadow properties in real-time
//! - Hold buttons for continuous value changes
//! - Try different shapes to see how shadows adapt to border radius
//! - Experiment with multiple shadow layers for creative effects

// Essential imports for creating an interactive shadow demo
use bevy::{
    color::palettes::css::*,    // Pre-defined color constants
    prelude::*,                 // Core Bevy types and traits
    time::Time,                 // For button repeat timing
    window::RequestRedraw,      // For triggering redraws during interactions
    winit::WinitSettings,       // For optimizing desktop app performance
};

// Button color constants for different interaction states
// These create a cohesive visual feedback system for user interactions
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);  // Dark gray when idle
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25); // Lighter gray when hovered
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35); // Green when pressed

// Default settings for the shape selector
// Using a struct here allows us to easily reset to defaults
const SHAPE_DEFAULT_SETTINGS: ShapeSettings = ShapeSettings { index: 0 };

// Default shadow configuration values
// These provide sensible starting values that demonstrate shadow effects clearly
const SHADOW_DEFAULT_SETTINGS: ShadowSettings = ShadowSettings {
    x_offset: 20.0,  // Shadow displaced 20px right
    y_offset: 20.0,  // Shadow displaced 20px down
    blur: 10.0,      // Moderate blur for soft edges
    spread: 15.0,    // Shadow extends 15px beyond element
    count: 1,        // Single shadow layer
    samples: 6,      // Good quality/performance balance
};

// Array of shape configurations to demonstrate how shadows work with different geometries
// Each entry contains a display name and a closure that configures the node and border radius
// This pattern allows us to define complex configurations in a data-driven way
const SHAPES: &[(&str, fn(&mut Node, &mut BorderRadius))] = &[
    // Shape 1: Square with sharp corners - shows basic shadow behavior
    ("1", |node, radius| {
        node.width = Val::Px(164.);
        node.height = Val::Px(164.);
        *radius = BorderRadius::ZERO;  // No rounding - sharp rectangular shadow
    }),
    
    // Shape 2: Square with moderate rounding - shows how shadows follow border radius
    ("2", |node, radius| {
        node.width = Val::Px(164.);
        node.height = Val::Px(164.);
        *radius = BorderRadius::all(Val::Px(41.));  // Rounded corners affect shadow shape
    }),
    
    // Shape 3: Perfect circle - demonstrates circular shadow casting
    ("3", |node, radius| {
        node.width = Val::Px(164.);
        node.height = Val::Px(164.);
        *radius = BorderRadius::MAX;  // Maximum radius creates perfect circle
    }),
    
    // Shape 4: Wide rectangle - shows shadow behavior on non-square elements
    ("4", |node, radius| {
        node.width = Val::Px(240.);   // Wide aspect ratio
        node.height = Val::Px(80.);
        *radius = BorderRadius::all(Val::Px(32.));  // Moderate rounding
    }),
    
    // Shape 5: Tall rectangle - demonstrates vertical shadow extension
    ("5", |node, radius| {
        node.width = Val::Px(80.);    // Narrow width
        node.height = Val::Px(240.);  // Tall height
        *radius = BorderRadius::all(Val::Px(32.));  // Consistent rounding
    }),
];

// Resource to track which shape is currently selected
// Resources in Bevy are global application state that can be accessed by any system
// The Default trait allows us to easily create instances with default values
#[derive(Resource, Default)]
struct ShapeSettings {
    index: usize,  // Index into the SHAPES array
}

// Resource containing all shadow configuration parameters
// This centralizes shadow state and makes it easy to save/load configurations
#[derive(Resource, Default)]
struct ShadowSettings {
    x_offset: f32,  // Horizontal shadow displacement (-∞ to +∞)
    y_offset: f32,  // Vertical shadow displacement (-∞ to +∞)
    blur: f32,      // Shadow blur radius (0 = sharp, higher = softer)
    spread: f32,    // Shadow size expansion (-∞ to +∞)
    count: usize,   // Number of shadow layers (1-3 in this example)
    samples: u32,   // Rendering quality (higher = smoother but slower)
}

// Component marker to identify the node that displays shadows
// This allows systems to find and update the specific node that needs shadow changes
#[derive(Component)]
struct ShadowNode;

// Component enum to identify different types of interactive buttons
// Using an enum allows us to have one button system that handles all button types
// PartialEq and Clone are needed for comparison and copying in our systems
#[derive(Component, PartialEq, Clone, Copy)]
enum SettingsButton {
    // Shadow offset controls - move shadow position
    XOffsetInc,    // Increase horizontal offset (move shadow right)
    XOffsetDec,    // Decrease horizontal offset (move shadow left)
    YOffsetInc,    // Increase vertical offset (move shadow down)
    YOffsetDec,    // Decrease vertical offset (move shadow up)
    
    // Shadow blur controls - change edge softness
    BlurInc,       // Increase blur (softer edges)
    BlurDec,       // Decrease blur (sharper edges)
    
    // Shadow spread controls - change shadow size
    SpreadInc,     // Increase spread (larger shadow)
    SpreadDec,     // Decrease spread (smaller shadow)
    
    // Shadow count controls - number of shadow layers
    CountInc,      // Add another shadow layer
    CountDec,      // Remove a shadow layer
    
    // Shape selection controls
    ShapePrev,     // Previous shape in the array
    ShapeNext,     // Next shape in the array
    
    // Utility controls
    Reset,         // Reset all settings to defaults
    
    // Quality controls
    SamplesInc,    // Increase rendering quality
    SamplesDec,    // Decrease rendering quality
}

// Component enum to categorize different types of settings for UI display
// This helps us organize the control panel and update the correct value displays
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum SettingType {
    XOffset,  // Horizontal shadow displacement
    YOffset,  // Vertical shadow displacement
    Blur,     // Shadow edge softness
    Spread,   // Shadow size expansion
    Count,    // Number of shadow layers
    Shape,    // Selected shape configuration
    Samples,  // Rendering quality level
}

impl SettingType {
    // Helper method to get human-readable labels for the UI
    // This creates consistent labeling throughout the interface
    fn label(&self) -> &str {
        match self {
            SettingType::XOffset => "X Offset",   // Horizontal positioning
            SettingType::YOffset => "Y Offset",   // Vertical positioning  
            SettingType::Blur => "Blur",          // Edge softness
            SettingType::Spread => "Spread",      // Size expansion
            SettingType::Count => "Count",        // Layer quantity
            SettingType::Shape => "Shape",        // Geometry selection
            SettingType::Samples => "Samples",    // Quality setting
        }
    }
}

// Resource to track button press-and-hold behavior for continuous value changes
// This enables users to hold buttons for rapid adjustments instead of clicking repeatedly
#[derive(Resource, Default)]
struct HeldButton {
    button: Option<SettingsButton>,  // Which button is currently being held (if any)
    pressed_at: Option<f64>,         // When the button was first pressed (for initial delay)
    last_repeat: Option<f64>,        // When the last repeat action occurred (for repeat rate)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)  // Essential Bevy systems for windowing, rendering, input, etc.
        
        // Configure for desktop app performance - only redraws when needed
        .insert_resource(WinitSettings::desktop_app())
        
        // Initialize our application state resources with default values
        .insert_resource(SHADOW_DEFAULT_SETTINGS)  // Shadow configuration
        .insert_resource(SHAPE_DEFAULT_SETTINGS)   // Shape selection
        .insert_resource(HeldButton::default())    // Button hold tracking
        
        // Run setup once when the application starts
        .add_systems(Startup, setup)
        
        // Update systems that run every frame
        .add_systems(
            Update,
            (
                // Handle button press events and update resources
                button_system,
                
                // Update button colors based on interaction state
                button_color_system,
                
                // Update shape only when shape settings change
                // run_if() is a performance optimization - only runs when needed
                update_shape.run_if(resource_changed::<ShapeSettings>),
                
                // Update shadow styling when shadow settings change
                update_shadow.run_if(resource_changed::<ShadowSettings>),
                
                // Update shadow rendering quality when settings change
                update_shadow_samples.run_if(resource_changed::<ShadowSettings>),
                
                // Handle continuous button presses (hold-to-repeat)
                button_repeat_system,
            ),
        )
        .run(); // Start the main application loop
}

// --- UI Setup ---
// This function creates the entire user interface including the shadow demo and control panel
fn setup(
    mut commands: Commands,    // For spawning entities
    asset_server: Res<AssetServer>,  // For loading fonts
    shadow: Res<ShadowSettings>,     // Current shadow configuration
    shape: Res<ShapeSettings>,       // Current shape selection
) {
    // Spawn camera with shadow quality setting
    // BoxShadowSamples controls the rendering quality of all shadows in the scene
    commands.spawn((Camera2d, BoxShadowSamples(shadow.samples)));
    
    // Create the main demonstration area showing the shadowed shape
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(GRAY.into()),
        ))
        .insert(children![{
            let mut node = Node {
                width: Val::Px(164.),
                height: Val::Px(164.),
                border: UiRect::all(Val::Px(1.)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            };
            let mut radius = BorderRadius::ZERO;
            SHAPES[shape.index % SHAPES.len()].1(&mut node, &mut radius);

            (
                node,
                BorderColor::all(WHITE.into()),
                radius,
                BackgroundColor(Color::srgb(0.21, 0.21, 0.21)),
                // BoxShadow component applies shadow effects to this UI node
                // It takes a Vec<ShadowStyle> allowing multiple shadow layers
                BoxShadow(vec![ShadowStyle {
                    color: Color::BLACK.with_alpha(0.8),      // Semi-transparent black
                    x_offset: Val::Px(shadow.x_offset),       // Horizontal displacement
                    y_offset: Val::Px(shadow.y_offset),       // Vertical displacement
                    spread_radius: Val::Px(shadow.spread),    // Size expansion
                    blur_radius: Val::Px(shadow.blur),        // Edge softness
                }]),
                ShadowNode,
            )
        }]);

    // Settings Panel
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                left: Val::Px(24.0),
                bottom: Val::Px(24.0),
                width: Val::Px(270.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12).with_alpha(0.85)),
            BorderColor::all(Color::WHITE.with_alpha(0.15)),
            BorderRadius::all(Val::Px(12.0)),
            ZIndex(10),
        ))
        .insert(children![
            build_setting_row(
                SettingType::Shape,
                SettingsButton::ShapePrev,
                SettingsButton::ShapeNext,
                shape.index as f32,
                &asset_server,
            ),
            build_setting_row(
                SettingType::XOffset,
                SettingsButton::XOffsetDec,
                SettingsButton::XOffsetInc,
                shadow.x_offset,
                &asset_server,
            ),
            build_setting_row(
                SettingType::YOffset,
                SettingsButton::YOffsetDec,
                SettingsButton::YOffsetInc,
                shadow.y_offset,
                &asset_server,
            ),
            build_setting_row(
                SettingType::Blur,
                SettingsButton::BlurDec,
                SettingsButton::BlurInc,
                shadow.blur,
                &asset_server,
            ),
            build_setting_row(
                SettingType::Spread,
                SettingsButton::SpreadDec,
                SettingsButton::SpreadInc,
                shadow.spread,
                &asset_server,
            ),
            build_setting_row(
                SettingType::Count,
                SettingsButton::CountDec,
                SettingsButton::CountInc,
                shadow.count as f32,
                &asset_server,
            ),
            // Add BoxShadowSamples as a setting row
            build_setting_row(
                SettingType::Samples,
                SettingsButton::SamplesDec,
                SettingsButton::SamplesInc,
                shadow.samples as f32,
                &asset_server,
            ),
            // Reset button
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    height: Val::Px(36.0),
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
                children![(
                    Button,
                    Node {
                        width: Val::Px(90.),
                        height: Val::Px(32.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(NORMAL_BUTTON),
                    BorderRadius::all(Val::Px(8.)),
                    SettingsButton::Reset,
                    children![(
                        Text::new("Reset"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                    )],
                )],
            ),
        ]);
}

// --- UI Helper Functions ---

// Helper to return an input to the children! macro for a setting row
fn build_setting_row(
    setting_type: SettingType,
    dec: SettingsButton,
    inc: SettingsButton,
    value: f32,
    asset_server: &Res<AssetServer>,
) -> impl Bundle {
    let value_text = match setting_type {
        SettingType::Shape => SHAPES[value as usize % SHAPES.len()].0.to_string(),
        SettingType::Count => format!("{}", value as usize),
        _ => format!("{:.1}", value),
    };

    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            height: Val::Px(32.0),
            ..default()
        },
        children![
            (
                Node {
                    width: Val::Px(80.0),
                    justify_content: JustifyContent::FlexEnd,
                    align_items: AlignItems::Center,
                    ..default()
                },
                // Attach SettingType to the value label node, not the parent row
                children![(
                    Text::new(setting_type.label()),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 16.0,
                        ..default()
                    },
                )],
            ),
            (
                Button,
                Node {
                    width: Val::Px(28.),
                    height: Val::Px(28.),
                    margin: UiRect::left(Val::Px(8.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderRadius::all(Val::Px(6.)),
                dec,
                children![(
                    Text::new(if setting_type == SettingType::Shape {
                        "<"
                    } else {
                        "-"
                    }),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 18.0,
                        ..default()
                    },
                )],
            ),
            (
                Node {
                    width: Val::Px(48.),
                    height: Val::Px(28.),
                    margin: UiRect::horizontal(Val::Px(8.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderRadius::all(Val::Px(6.)),
                children![{
                    (
                        Text::new(value_text),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 16.0,
                            ..default()
                        },
                        setting_type,
                    )
                }],
            ),
            (
                Button,
                Node {
                    width: Val::Px(28.),
                    height: Val::Px(28.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderRadius::all(Val::Px(6.)),
                inc,
                children![(
                    Text::new(if setting_type == SettingType::Shape {
                        ">"
                    } else {
                        "+"
                    }),
                    TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 18.0,
                        ..default()
                    },
                )],
            ),
        ],
    )
}

// --- SYSTEMS ---

// Update the shadow node's BoxShadow on resource changes
fn update_shadow(
    shadow: Res<ShadowSettings>,
    mut query: Query<&mut BoxShadow, With<ShadowNode>>,
    mut label_query: Query<(&mut Text, &SettingType)>,
) {
    for mut box_shadow in &mut query {
        *box_shadow = BoxShadow(generate_shadows(&shadow));
    }
    // Update value labels for shadow settings
    for (mut text, setting) in &mut label_query {
        let value = match setting {
            SettingType::XOffset => format!("{:.1}", shadow.x_offset),
            SettingType::YOffset => format!("{:.1}", shadow.y_offset),
            SettingType::Blur => format!("{:.1}", shadow.blur),
            SettingType::Spread => format!("{:.1}", shadow.spread),
            SettingType::Count => format!("{}", shadow.count),
            SettingType::Shape => continue,
            SettingType::Samples => format!("{}", shadow.samples),
        };
        *text = Text::new(value);
    }
}

fn update_shadow_samples(
    shadow: Res<ShadowSettings>,
    mut query: Query<&mut BoxShadowSamples, With<Camera2d>>,
) {
    for mut samples in &mut query {
        samples.0 = shadow.samples;
    }
}

fn generate_shadows(shadow: &ShadowSettings) -> Vec<ShadowStyle> {
    match shadow.count {
        1 => vec![make_shadow(
            BLACK.into(),
            shadow.x_offset,
            shadow.y_offset,
            shadow.spread,
            shadow.blur,
        )],
        2 => vec![
            make_shadow(
                BLUE.into(),
                shadow.x_offset,
                shadow.y_offset,
                shadow.spread,
                shadow.blur,
            ),
            make_shadow(
                YELLOW.into(),
                -shadow.x_offset,
                -shadow.y_offset,
                shadow.spread,
                shadow.blur,
            ),
        ],
        3 => vec![
            make_shadow(
                BLUE.into(),
                shadow.x_offset,
                shadow.y_offset,
                shadow.spread,
                shadow.blur,
            ),
            make_shadow(
                YELLOW.into(),
                -shadow.x_offset,
                -shadow.y_offset,
                shadow.spread,
                shadow.blur,
            ),
            make_shadow(
                RED.into(),
                shadow.y_offset,
                -shadow.x_offset,
                shadow.spread,
                shadow.blur,
            ),
        ],
        _ => vec![],
    }
}

fn make_shadow(color: Color, x_offset: f32, y_offset: f32, spread: f32, blur: f32) -> ShadowStyle {
    ShadowStyle {
        color: color.with_alpha(0.8),
        x_offset: Val::Px(x_offset),
        y_offset: Val::Px(y_offset),
        spread_radius: Val::Px(spread),
        blur_radius: Val::Px(blur),
    }
}

// Update shape of ShadowNode if shape selection changed
fn update_shape(
    shape: Res<ShapeSettings>,
    mut query: Query<(&mut Node, &mut BorderRadius), With<ShadowNode>>,
    mut label_query: Query<(&mut Text, &SettingType)>,
) {
    for (mut node, mut radius) in &mut query {
        SHAPES[shape.index % SHAPES.len()].1(&mut node, &mut radius);
    }
    for (mut text, kind) in &mut label_query {
        if *kind == SettingType::Shape {
            *text = Text::new(SHAPES[shape.index % SHAPES.len()].0);
        }
    }
}

// Handles button interactions for all settings
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &SettingsButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut shadow: ResMut<ShadowSettings>,
    mut shape: ResMut<ShapeSettings>,
    mut held: ResMut<HeldButton>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs_f64();
    for (interaction, btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                trigger_button_action(btn, &mut shadow, &mut shape);
                held.button = Some(*btn);
                held.pressed_at = Some(now);
                held.last_repeat = Some(now);
            }
            Interaction::None | Interaction::Hovered => {
                if held.button == Some(*btn) {
                    held.button = None;
                    held.pressed_at = None;
                    held.last_repeat = None;
                }
            }
        }
    }
}

fn trigger_button_action(
    btn: &SettingsButton,
    shadow: &mut ShadowSettings,
    shape: &mut ShapeSettings,
) {
    match btn {
        SettingsButton::XOffsetInc => shadow.x_offset += 1.0,
        SettingsButton::XOffsetDec => shadow.x_offset -= 1.0,
        SettingsButton::YOffsetInc => shadow.y_offset += 1.0,
        SettingsButton::YOffsetDec => shadow.y_offset -= 1.0,
        SettingsButton::BlurInc => shadow.blur = (shadow.blur + 1.0).max(0.0),
        SettingsButton::BlurDec => shadow.blur = (shadow.blur - 1.0).max(0.0),
        SettingsButton::SpreadInc => shadow.spread += 1.0,
        SettingsButton::SpreadDec => shadow.spread -= 1.0,
        SettingsButton::CountInc => {
            if shadow.count < 3 {
                shadow.count += 1;
            }
        }
        SettingsButton::CountDec => {
            if shadow.count > 1 {
                shadow.count -= 1;
            }
        }
        SettingsButton::ShapePrev => {
            if shape.index == 0 {
                shape.index = SHAPES.len() - 1;
            } else {
                shape.index -= 1;
            }
        }
        SettingsButton::ShapeNext => {
            shape.index = (shape.index + 1) % SHAPES.len();
        }
        SettingsButton::Reset => {
            *shape = SHAPE_DEFAULT_SETTINGS;
            *shadow = SHADOW_DEFAULT_SETTINGS;
        }
        SettingsButton::SamplesInc => shadow.samples += 1,
        SettingsButton::SamplesDec => {
            if shadow.samples > 1 {
                shadow.samples -= 1;
            }
        }
    }
}

// System to repeat button action while held
fn button_repeat_system(
    time: Res<Time>,
    mut held: ResMut<HeldButton>,
    mut shadow: ResMut<ShadowSettings>,
    mut shape: ResMut<ShapeSettings>,
    mut redraw_events: EventWriter<RequestRedraw>,
) {
    if held.button.is_some() {
        redraw_events.write(RequestRedraw);
    }
    const INITIAL_DELAY: f64 = 0.15;
    const REPEAT_RATE: f64 = 0.08;
    if let (Some(btn), Some(pressed_at)) = (held.button, held.pressed_at) {
        let now = time.elapsed_secs_f64();
        let since_pressed = now - pressed_at;
        let last_repeat = held.last_repeat.unwrap_or(pressed_at);
        let since_last = now - last_repeat;
        if since_pressed > INITIAL_DELAY && since_last > REPEAT_RATE {
            trigger_button_action(&btn, &mut shadow, &mut shape);
            held.last_repeat = Some(now);
        }
    }
}

// Changes color of button on hover and on pressed
fn button_color_system(
    mut query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<SettingsButton>),
    >,
) {
    for (interaction, mut color) in &mut query {
        match *interaction {
            Interaction::Pressed => *color = PRESSED_BUTTON.into(),
            Interaction::Hovered => *color = HOVERED_BUTTON.into(),
            Interaction::None => *color = NORMAL_BUTTON.into(),
        }
    }
}
