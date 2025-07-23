//! # UI Display and Visibility Example
//! 
//! This example demonstrates the crucial differences between Display and Visibility properties in Bevy 0.16 UI.
//! 
//! ## What You'll Learn
//! - The difference between `Display` and `Visibility` and when to use each
//! - How `Display::None` removes elements from layout calculations
//! - How `Visibility::Hidden` hides elements but preserves layout space
//! - Understanding `Visibility::Inherited` for hierarchical visibility control
//! - Creating interactive UI demonstrations with buttons and real-time updates
//! - Building complex nested UI hierarchies with proper layout management
//! - Using trait systems for generic UI behavior across different component types
//! 
//! ## Key Concepts
//! - **Display**: Controls whether an element participates in layout (Flex, None, Block, Grid)
//! - **Visibility**: Controls whether an element is rendered while preserving layout space
//! - **Layout Space**: The area an element occupies in the UI layout calculation
//! - **Inheritance**: How child elements can inherit properties from their parents
//! - **Interactive Demonstration**: Using buttons to dynamically change properties
//! 
//! ## Display vs Visibility Comparison
//! | Property | Effect | Layout Space | Use Case |
//! |----------|--------|--------------|----------|
//! | `Display::Flex` | Visible and participates in layout | Occupies space | Normal UI elements |
//! | `Display::None` | Invisible and removed from layout | No space | Temporary removal |
//! | `Visibility::Visible` | Visible and participates in layout | Occupies space | Force visibility |
//! | `Visibility::Hidden` | Invisible but still in layout | Occupies space | Hide while maintaining layout |
//! | `Visibility::Inherited` | Inherits parent's visibility | Depends on parent | Default behavior |
//! 
//! ## Interactive Controls
//! Use the buttons in the right panel to toggle Display and Visibility properties for the corresponding
//! nested elements in the left panel. Notice how the layout changes differently for each property.

use bevy::{
    color::palettes::css::{DARK_CYAN, DARK_GRAY, YELLOW}, // Pre-defined color constants
    ecs::{component::Mutable, hierarchy::ChildSpawnerCommands}, // ECS component utilities
    prelude::*,        // Core Bevy types and traits
    winit::WinitSettings, // Window management settings for performance
};

// Color palette for the nested UI elements - provides visual distinction between hierarchy levels
// Using hex strings that will be converted to Color values for a cohesive blue gradient theme
const PALETTE: [&str; 4] = ["27496D", "466B7A", "669DB3", "ADCBE3"];

// Color used to indicate hidden/invisible elements in the UI
// Light red tint helps users identify when elements are in hidden states
const HIDDEN_COLOR: Color = Color::srgb(1.0, 0.7, 0.7);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        
        // Performance optimization: only run the app when there is user input
        // This significantly reduces CPU/GPU usage for demonstration applications
        .insert_resource(WinitSettings::desktop_app())
        
        // Initialize the demonstration UI once at startup
        .add_systems(Startup, setup)
        
        // Update systems that run every frame to handle interactions
        .add_systems(
            Update,
            (
                // Generic button handlers for different property types
                // These use Rust generics to handle both Display and Visibility buttons
                buttons_handler::<Display>,    // Handle Display property toggle buttons
                buttons_handler::<Visibility>, // Handle Visibility property toggle buttons
                
                // Visual feedback system for button hover states
                text_hover,
            ),
        )
        .run(); // Start the application main loop
}

// Generic component that links a button to a target entity for property manipulation
// The generic type T represents which property type this button controls (Display or Visibility)
// PhantomData is used because we need the type information but don't store actual T data
#[derive(Component)]
struct Target<T> {
    id: Entity,  // The entity whose properties this button will modify
    phantom: std::marker::PhantomData<T>, // Zero-size type marker for generic type safety
}

impl<T> Target<T> {
    fn new(id: Entity) -> Self {
        Self {
            id,
            phantom: std::marker::PhantomData, // Rust requires this for generic type parameters
        }
    }
}

// Trait that defines how different property types can be updated by buttons
// This demonstrates advanced Rust patterns: associated types, const generics, and trait bounds
trait TargetUpdate {
    // Associated type: which component type this target updates
    // Mutability bound ensures we can modify the component
    type TargetComponent: Component<Mutability = Mutable>;
    
    // Associated constant: human-readable name for UI display
    const NAME: &'static str;
    
    // Method to toggle the property and return a description of the new state
    fn update_target(&self, target: &mut Self::TargetComponent) -> String;
}

// Implementation for Display property buttons
// Display controls whether an element participates in layout calculations
impl TargetUpdate for Target<Display> {
    type TargetComponent = Node;  // Display is a field of the Node component
    const NAME: &'static str = "Display";
    
    fn update_target(&self, node: &mut Self::TargetComponent) -> String {
        // Toggle between the two main display states used in this example
        node.display = match node.display {
            Display::Flex => Display::None,  // Hide and remove from layout
            Display::None => Display::Flex,  // Show and participate in layout
            
            // This example only uses Flex and None, other variants are not used
            Display::Block | Display::Grid => unreachable!(),
        };
        
        // Return a description of the new state for the button text
        format!("{}::{:?} ", Self::NAME, node.display)
    }
}

// Implementation for Visibility property buttons  
// Visibility controls rendering while preserving layout space
impl TargetUpdate for Target<Visibility> {
    type TargetComponent = Visibility;  // Visibility is its own component
    const NAME: &'static str = "Visibility";
    
    fn update_target(&self, visibility: &mut Self::TargetComponent) -> String {
        // Cycle through all three visibility states to demonstrate differences
        *visibility = match *visibility {
            Visibility::Inherited => Visibility::Visible,  // Force visible
            Visibility::Visible => Visibility::Hidden,     // Hide but keep layout space
            Visibility::Hidden => Visibility::Inherited,   // Use parent's visibility
        };
        
        // Return a description of the new state for the button text
        format!("{}::{visibility:?}", Self::NAME)
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let palette: [Color; 4] = PALETTE.map(|hex| Srgba::hex(hex).unwrap().into());

    let text_font = TextFont {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        ..default()
    };

    commands.spawn(Camera2d);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                ..Default::default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Use the panel on the right to change the Display and Visibility properties for the respective nodes of the panel on the left"),
                text_font.clone(),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.)),
                    ..Default::default()
                },
            ));

            parent
                .spawn(Node {
                    width: Val::Percent(100.),
                    ..default()
                })
                .with_children(|parent| {
                    let mut target_ids = vec![];
                    parent
                        .spawn(Node {
                            width: Val::Percent(50.),
                            height: Val::Px(520.),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            target_ids = spawn_left_panel(parent, &palette);
                        });

                    parent
                        .spawn(Node {
                            width: Val::Percent(50.),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_right_panel(parent, text_font, &palette, target_ids);
                        });
                });

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    column_gap: Val::Px(10.),
                    ..default()
                })
                .with_children(|builder| {
                    let text_font = TextFont {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        ..default()
                    };

                    builder.spawn((
                        Text::new("Display::None\nVisibility::Hidden\nVisibility::Inherited"),
                        text_font.clone(),
                        TextColor(HIDDEN_COLOR),
                        TextLayout::new_with_justify(JustifyText::Center),
                    ));
                    builder.spawn((
                        Text::new("-\n-\n-"),
                        text_font.clone(),
                        TextColor(DARK_GRAY.into()),
                        TextLayout::new_with_justify(JustifyText::Center),
                    ));
                    builder.spawn((Text::new("The UI Node and its descendants will not be visible and will not be allotted any space in the UI layout.\nThe UI Node will not be visible but will still occupy space in the UI layout.\nThe UI node will inherit the visibility property of its parent. If it has no parent it will be visible."), text_font));
                });
        });
}

fn spawn_left_panel(builder: &mut ChildSpawnerCommands, palette: &[Color; 4]) -> Vec<Entity> {
    let mut target_ids = vec![];
    builder
        .spawn((
            Node {
                padding: UiRect::all(Val::Px(10.)),
                ..default()
            },
            BackgroundColor(Color::WHITE),
        ))
        .with_children(|parent| {
            parent
                .spawn((Node::default(), BackgroundColor(Color::BLACK)))
                .with_children(|parent| {
                    let id = parent
                        .spawn((
                            Node {
                                align_items: AlignItems::FlexEnd,
                                justify_content: JustifyContent::FlexEnd,
                                ..default()
                            },
                            BackgroundColor(palette[0]),
                            Outline {
                                width: Val::Px(4.),
                                color: DARK_CYAN.into(),
                                offset: Val::Px(10.),
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(Node {
                                width: Val::Px(100.),
                                height: Val::Px(500.),
                                ..default()
                            });

                            let id = parent
                                .spawn((
                                    Node {
                                        height: Val::Px(400.),
                                        align_items: AlignItems::FlexEnd,
                                        justify_content: JustifyContent::FlexEnd,
                                        ..default()
                                    },
                                    BackgroundColor(palette[1]),
                                ))
                                .with_children(|parent| {
                                    parent.spawn(Node {
                                        width: Val::Px(100.),
                                        height: Val::Px(400.),
                                        ..default()
                                    });

                                    let id = parent
                                        .spawn((
                                            Node {
                                                height: Val::Px(300.),
                                                align_items: AlignItems::FlexEnd,
                                                justify_content: JustifyContent::FlexEnd,
                                                ..default()
                                            },
                                            BackgroundColor(palette[2]),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(Node {
                                                width: Val::Px(100.),
                                                height: Val::Px(300.),
                                                ..default()
                                            });

                                            let id = parent
                                                .spawn((
                                                    Node {
                                                        width: Val::Px(200.),
                                                        height: Val::Px(200.),
                                                        ..default()
                                                    },
                                                    BackgroundColor(palette[3]),
                                                ))
                                                .id();
                                            target_ids.push(id);
                                        })
                                        .id();
                                    target_ids.push(id);
                                })
                                .id();
                            target_ids.push(id);
                        })
                        .id();
                    target_ids.push(id);
                });
        });
    target_ids
}

fn spawn_right_panel(
    parent: &mut ChildSpawnerCommands,
    text_font: TextFont,
    palette: &[Color; 4],
    mut target_ids: Vec<Entity>,
) {
    let spawn_buttons = |parent: &mut ChildSpawnerCommands, target_id| {
        spawn_button::<Display>(parent, text_font.clone(), target_id);
        spawn_button::<Visibility>(parent, text_font.clone(), target_id);
    };
    parent
        .spawn((
            Node {
                padding: UiRect::all(Val::Px(10.)),
                ..default()
            },
            BackgroundColor(Color::WHITE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(500.),
                        height: Val::Px(500.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexEnd,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect {
                            left: Val::Px(5.),
                            top: Val::Px(5.),
                            ..default()
                        },
                        ..default()
                    },
                    BackgroundColor(palette[0]),
                    Outline {
                        width: Val::Px(4.),
                        color: DARK_CYAN.into(),
                        offset: Val::Px(10.),
                    },
                ))
                .with_children(|parent| {
                    spawn_buttons(parent, target_ids.pop().unwrap());

                    parent
                        .spawn((
                            Node {
                                width: Val::Px(400.),
                                height: Val::Px(400.),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexEnd,
                                justify_content: JustifyContent::SpaceBetween,
                                padding: UiRect {
                                    left: Val::Px(5.),
                                    top: Val::Px(5.),
                                    ..default()
                                },
                                ..default()
                            },
                            BackgroundColor(palette[1]),
                        ))
                        .with_children(|parent| {
                            spawn_buttons(parent, target_ids.pop().unwrap());

                            parent
                                .spawn((
                                    Node {
                                        width: Val::Px(300.),
                                        height: Val::Px(300.),
                                        flex_direction: FlexDirection::Column,
                                        align_items: AlignItems::FlexEnd,
                                        justify_content: JustifyContent::SpaceBetween,
                                        padding: UiRect {
                                            left: Val::Px(5.),
                                            top: Val::Px(5.),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    BackgroundColor(palette[2]),
                                ))
                                .with_children(|parent| {
                                    spawn_buttons(parent, target_ids.pop().unwrap());

                                    parent
                                        .spawn((
                                            Node {
                                                width: Val::Px(200.),
                                                height: Val::Px(200.),
                                                align_items: AlignItems::FlexStart,
                                                justify_content: JustifyContent::SpaceBetween,
                                                flex_direction: FlexDirection::Column,
                                                padding: UiRect {
                                                    left: Val::Px(5.),
                                                    top: Val::Px(5.),
                                                    ..default()
                                                },
                                                ..default()
                                            },
                                            BackgroundColor(palette[3]),
                                        ))
                                        .with_children(|parent| {
                                            spawn_buttons(parent, target_ids.pop().unwrap());

                                            parent.spawn(Node {
                                                width: Val::Px(100.),
                                                height: Val::Px(100.),
                                                ..default()
                                            });
                                        });
                                });
                        });
                });
        });
}

fn spawn_button<T>(parent: &mut ChildSpawnerCommands, text_font: TextFont, target: Entity)
where
    T: Default + std::fmt::Debug + Send + Sync + 'static,
    Target<T>: TargetUpdate,
{
    parent
        .spawn((
            Button,
            Node {
                align_self: AlignSelf::FlexStart,
                padding: UiRect::axes(Val::Px(5.), Val::Px(1.)),
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.5)),
            Target::<T>::new(target),
        ))
        .with_children(|builder| {
            builder.spawn((
                Text(format!("{}::{:?}", Target::<T>::NAME, T::default())),
                text_font,
                TextLayout::new_with_justify(JustifyText::Center),
            ));
        });
}

fn buttons_handler<T>(
    mut left_panel_query: Query<&mut <Target<T> as TargetUpdate>::TargetComponent>,
    mut visibility_button_query: Query<(&Target<T>, &Interaction, &Children), Changed<Interaction>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) where
    T: Send + Sync,
    Target<T>: TargetUpdate + Component,
{
    for (target, interaction, children) in visibility_button_query.iter_mut() {
        if matches!(interaction, Interaction::Pressed) {
            let mut target_value = left_panel_query.get_mut(target.id).unwrap();
            for &child in children {
                if let Ok((mut text, mut text_color)) = text_query.get_mut(child) {
                    **text = target.update_target(target_value.as_mut());
                    text_color.0 = if text.contains("None") || text.contains("Hidden") {
                        Color::srgb(1.0, 0.7, 0.7)
                    } else {
                        Color::WHITE
                    };
                }
            }
        }
    }
}

fn text_hover(
    mut button_query: Query<(&Interaction, &mut BackgroundColor, &Children), Changed<Interaction>>,
    mut text_query: Query<(&Text, &mut TextColor)>,
) {
    for (interaction, mut color, children) in button_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *color = Color::BLACK.with_alpha(0.6).into();
                for &child in children {
                    if let Ok((_, mut text_color)) = text_query.get_mut(child) {
                        // Bypass change detection to avoid recomputation of the text when only changing the color
                        text_color.bypass_change_detection().0 = YELLOW.into();
                    }
                }
            }
            _ => {
                *color = Color::BLACK.with_alpha(0.5).into();
                for &child in children {
                    if let Ok((text, mut text_color)) = text_query.get_mut(child) {
                        text_color.bypass_change_detection().0 =
                            if text.contains("None") || text.contains("Hidden") {
                                HIDDEN_COLOR
                            } else {
                                Color::WHITE
                            };
                    }
                }
            }
        }
    }
}
