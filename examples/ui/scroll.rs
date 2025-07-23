//! This example illustrates scrolling in Bevy UI.
//!
//! Scrolling is like having a window into a larger world - you can only see
//! part of the content at once, but you can move the window to see different parts.
//! Think of it like reading a long scroll through a magnifying glass - you move
//! the glass (viewport) over the content, not the content itself!

use accesskit::{Node as Accessible, Role};
use bevy::{
    a11y::AccessibilityNode,
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
    winit::WinitSettings,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        // Desktop app settings for smooth scrolling
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        // Custom scroll handling system
        .add_systems(Update, update_scroll_position);

    app.run();
}

// Constants for consistent text sizing
const FONT_SIZE: f32 = 20.;
const LINE_HEIGHT: f32 = 21.; // Slightly larger than font for readability

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera with UI rendering
    commands.spawn((Camera2d, IsDefaultUiCamera));

    // ROOT CONTAINER - Full window
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            flex_direction: FlexDirection::Column,
            ..default()
        })
        // Prevent root from intercepting pointer events
        .insert(Pickable::IGNORE)
        .with_children(|parent| {
            // HORIZONTAL SCROLL EXAMPLE
            parent
                .spawn(Node {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    // Header with instructions
                    parent.spawn((
                        Text::new("Horizontally Scrolling list (Ctrl + MouseWheel)"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: FONT_SIZE,
                            ..default()
                        },
                        Label, // Accessibility label
                    ));

                    // HORIZONTAL SCROLL CONTAINER
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(80.), // 80% of parent width
                                margin: UiRect::all(Val::Px(10.)),
                                flex_direction: FlexDirection::Row, // Items flow left-to-right
                                // THE KEY: overflow scroll_x enables horizontal scrolling!
                                overflow: Overflow::scroll_x(),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.10, 0.10, 0.10)),
                        ))
                        .with_children(|parent| {
                            // Create 100 items - more than can fit!
                            for i in 0..100 {
                                parent.spawn((Text(format!("Item {i}")),
                                        TextFont {
                                            font: asset_server
                                                .load("fonts/FiraSans-Bold.ttf"),
                                            ..default()
                                        },
                                    Label,
                                    // Accessibility: Mark as list item for screen readers
                                    AccessibilityNode(Accessible::new(Role::ListItem)),
                                ))
                                .insert(Node {
                                    // Fixed width ensures items don't shrink
                                    min_width: Val::Px(200.),
                                    align_content: AlignContent::Center,
                                    ..default()
                                })
                                .insert(Pickable {
                                    // Allow clicks to pass through to scrollable area
                                    should_block_lower: false,
                                    ..default()
                                })
                                // Click to remove items - demonstrates interactivity!
                                .observe(|
                                    trigger: Trigger<Pointer<Pressed>>,
                                    mut commands: Commands
                                | {
                                    if trigger.event().button == PointerButton::Primary {
                                        commands.entity(trigger.target()).despawn();
                                    }
                                });
                            }
                        });
                });

            // CONTAINER FOR VERTICAL EXAMPLES
            // Three columns showing different scroll patterns
            parent
                .spawn(Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    // VERTICAL SCROLL EXAMPLE (Most Common!)
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Px(200.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Title
                            parent.spawn((
                                Text::new("Vertically Scrolling List"),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: FONT_SIZE,
                                    ..default()
                                },
                                Label,
                            ));
                            // VERTICAL SCROLL CONTAINER
                            parent
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Column, // Stack items vertically
                                        align_self: AlignSelf::Stretch, // Fill parent width
                                        height: Val::Percent(50.), // Limited height forces scrolling
                                        // THE KEY: overflow scroll_y for vertical scrolling
                                        overflow: Overflow::scroll_y(),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.10, 0.10, 0.10)),
                                ))
                                .with_children(|parent| {
                                    // List items
                                    for i in 0..25 {
                                        parent
                                            .spawn(Node {
                                                min_height: Val::Px(LINE_HEIGHT),
                                                max_height: Val::Px(LINE_HEIGHT),
                                                ..default()
                                            })
                                            .insert(Pickable {
                                                should_block_lower: false,
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                parent
                                                    .spawn((
                                                        Text(format!("Item {i}")),
                                                        TextFont {
                                                            font: asset_server
                                                                .load("fonts/FiraSans-Bold.ttf"),
                                                            ..default()
                                                        },
                                                        Label,
                                                        AccessibilityNode(Accessible::new(
                                                            Role::ListItem,
                                                        )),
                                                    ))
                                                    .insert(Pickable {
                                                        should_block_lower: false,
                                                        ..default()
                                                    });
                                            });
                                    }
                                });
                        });

                    // BIDIRECTIONAL SCROLL EXAMPLE
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Px(200.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Title
                            parent.spawn((
                                Text::new("Bidirectionally Scrolling List"),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: FONT_SIZE,
                                    ..default()
                                },
                                Label,
                            ));
                            // BOTH-DIRECTION SCROLL CONTAINER
                            parent
                                .spawn((
                                    Node {
                                        flex_direction: FlexDirection::Column,
                                        align_self: AlignSelf::Stretch,
                                        height: Val::Percent(50.),
                                        // THE KEY: Overflow::scroll() enables BOTH axes!
                                        // Content can overflow in any direction
                                        overflow: Overflow::scroll(),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.10, 0.10, 0.10)),
                                ))
                                .with_children(|parent| {
                                    // Rows in each column
                                    for oi in 0..10 {
                                        parent
                                            .spawn(Node {
                                                flex_direction: FlexDirection::Row,
                                                ..default()
                                            })
                                            .insert(Pickable::IGNORE)
                                            .with_children(|parent| {
                                                // Elements in each row
                                                for i in 0..25 {
                                                    parent
                                                        .spawn((
                                                            Text(format!("Item {}", (oi * 25) + i)),
                                                            TextFont {
                                                                font: asset_server.load(
                                                                    "fonts/FiraSans-Bold.ttf",
                                                                ),
                                                                ..default()
                                                            },
                                                            Label,
                                                            AccessibilityNode(Accessible::new(
                                                                Role::ListItem,
                                                            )),
                                                        ))
                                                        .insert(Pickable {
                                                            should_block_lower: false,
                                                            ..default()
                                                        });
                                                }
                                            });
                                    }
                                });
                        });

                    // NESTED SCROLLS EXAMPLE - Scroll inception!
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Px(200.),
                            ..default()
                        })
                        .with_children(|parent| {
                            // Title
                            parent.spawn((
                                Text::new("Nested Scrolling Lists"),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: FONT_SIZE,
                                    ..default()
                                },
                                Label,
                            ));
                            // OUTER CONTAINER - Scrolls horizontally
                            parent
                                .spawn((
                                    Node {
                                        column_gap: Val::Px(20.), // Space between columns
                                        flex_direction: FlexDirection::Row,
                                        align_self: AlignSelf::Stretch,
                                        height: Val::Percent(50.),
                                        // Horizontal scroll for the outer container
                                        overflow: Overflow::scroll_x(),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.10, 0.10, 0.10)),
                                ))
                                .with_children(|parent| {
                                    // INNER CONTAINERS - Each scrolls vertically!
                                    for oi in 0..30 {
                                        parent
                                            .spawn((
                                                Node {
                                                    flex_direction: FlexDirection::Column,
                                                    align_self: AlignSelf::Stretch,
                                                    // Each column scrolls independently!
                                                    overflow: Overflow::scroll_y(),
                                                    ..default()
                                                },
                                                // Darker background to distinguish from outer
                                                BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                                            ))
                                            .insert(Pickable {
                                                // Allow scroll events to propagate
                                                should_block_lower: false,
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                for i in 0..25 {
                                                    parent
                                                        .spawn((
                                                            Text(format!("Item {}", (oi * 25) + i)),
                                                            TextFont {
                                                                font: asset_server.load(
                                                                    "fonts/FiraSans-Bold.ttf",
                                                                ),
                                                                ..default()
                                                            },
                                                            Label,
                                                            AccessibilityNode(Accessible::new(
                                                                Role::ListItem,
                                                            )),
                                                        ))
                                                        .insert(Pickable {
                                                            should_block_lower: false,
                                                            ..default()
                                                        });
                                                }
                                            });
                                    }
                                });
                        });
                });
        });
}

/// Updates the scroll position of scrollable nodes in response to mouse input
/// This system demonstrates manual scroll handling - Bevy also has automatic scrolling!
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>, // Tracks which entities are under the cursor
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        // Convert scroll units to pixels
        // Different platforms report scroll differently!
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            // Line-based scrolling (common on Windows)
            MouseScrollUnit::Line => (
                mouse_wheel_event.x * LINE_HEIGHT,
                mouse_wheel_event.y * LINE_HEIGHT,
            ),
            // Pixel-based scrolling (common on touchpads)
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        // MODIFIER KEY: Ctrl swaps scroll axes!
        // This allows horizontal scrolling with vertical wheel
        if keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        // Apply scroll to all hovered scrollable elements
        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                // Check if this entity has a ScrollPosition component
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    // Update scroll offsets
                    // Note: negative because scrolling "down" moves content "up"
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}
