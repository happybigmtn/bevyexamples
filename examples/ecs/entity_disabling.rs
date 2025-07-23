//! Disabling entities is a powerful feature that allows you to hide entities from the ECS without deleting them.
//!
//! This can be useful for implementing features like "sleeping" objects that are offscreen
//! or managing networked entities.
//!
//! While disabling entities *will* make them invisible,
//! that's not its primary purpose!
//! [`Visibility`](bevy::prelude::Visibility) should be used to hide entities;
//! disabled entities are skipped entirely, which can lead to subtle bugs.
//!
//! # Default query filters
//!
//! Under the hood, Bevy uses a "default query filter" that skips entities with the
//! the [`Disabled`] component.
//! These filters act as a by-default exclusion list for all queries,
//! and can be bypassed by explicitly including these components in your queries.
//! For example, `Query<&A, With<Disabled>`, `Query<(Entity, Has<Disabled>>)` or
//! `Query<&A, Or<(With<Disabled>, With<B>)>>` will include disabled entities.
//!
//! Entity disabling is like putting objects in a special "invisible box" - they still exist
//! in your game world, but systems can't see them unless they specifically look for things
//! in that box! Think of it as the difference between moving furniture to the basement
//! (disabling) versus throwing it away (despawning). The furniture still exists and can
//! be brought back upstairs anytime, but house guests won't trip over it.

use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))  // Mouse picking support
        // EVENT-DRIVEN DISABLING - Click to hide entities
        .add_observer(disable_entities_on_click)
        .add_systems(
            Update,
            (
                list_all_named_entities,        // Shows which entities are visible
                reenable_entities_on_space,     // Space key brings everything back
            ),
        )
        .add_systems(Startup, (setup_scene, display_instructions))
        .run();
}

// MARKER COMPONENT - Tags entities that can be disabled by clicking
// Like putting a "clickable" sticker on objects
#[derive(Component)]
struct DisableOnClick;

fn disable_entities_on_click(
    trigger: Trigger<Pointer<Click>>,
    valid_query: Query<&DisableOnClick>,
    mut commands: Commands,
) {
    let clicked_entity = trigger.target();
    // SAFETY CHECK - Only disable entities we want to disable
    // Windows and text are entities and can be clicked!
    // We definitely don't want to disable the window itself,
    // because that would cause the app to close!
    if valid_query.contains(clicked_entity) {
        // THE MAGIC VANISHING ACT - Add Disabled component
        // Just add the `Disabled` component to the entity to disable it.
        // Note that the `Disabled` component is *only* added to the entity,
        // its children are not affected.
        // Like putting an invisibility cloak on just one person in a group!
        commands.entity(clicked_entity).insert(Disabled);
    }
}

// UI MARKER - Tags the text that shows entity names
#[derive(Component)]
struct EntityNameText;

// THE VISIBILITY DETECTOR - Shows which entities can be "seen" by normal queries
// The query here will not find entities with the `Disabled` component,
// because it does not explicitly include it.
// Like a security camera that only sees people not wearing invisibility cloaks!
fn list_all_named_entities(
    query: Query<&Name>,  // This query automatically skips disabled entities
    mut name_text_query: Query<&mut Text, With<EntityNameText>>,
    mut commands: Commands,
) {
    let mut text_string = String::from("Named entities found:\n");
    // CONSISTENT ORDERING - Sort for predictable output
    // Query iteration order is not guaranteed, so we sort the names
    // to ensure the output is consistent.
    for name in query.iter().sort::<&Name>() {
        text_string.push_str(&format!("{:?}\n", name));
    }

    // UPDATE OR CREATE UI TEXT
    if let Ok(mut text) = name_text_query.single_mut() {
        *text = Text::new(text_string);  // Update existing text
    } else {
        // Create the UI text for the first time
        commands.spawn((
            EntityNameText,
            Text::default(),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),  // Top-right corner
                ..default()
            },
        ));
    }
}

fn reenable_entities_on_space(
    mut commands: Commands,
    // THE SPECIAL DETECTOR - This query CAN see disabled entities!
    // This query can find disabled entities,
    // because it explicitly includes the `Disabled` component.
    // Like infrared goggles that can see people with invisibility cloaks!
    disabled_entities: Query<Entity, With<Disabled>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) {
        // MASS RESTORATION - Bring everything back from the invisible box
        for entity in disabled_entities.iter() {
            // THE REVEALING SPELL - Remove the invisibility cloak
            // To re-enable an entity, just remove the `Disabled` component.
            commands.entity(entity).remove::<Disabled>();
        }
    }
}

const X_EXTENT: f32 = 900.;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);

    // SHAPE GALLERY - Create different geometric shapes with names
    let named_shapes = [
        (Name::new("Annulus"), meshes.add(Annulus::new(25.0, 50.0))),      // Ring
        (
            Name::new("Bestagon"),                                          // Hexagon
            meshes.add(RegularPolygon::new(50.0, 6)),
        ),
        (Name::new("Rhombus"), meshes.add(Rhombus::new(75.0, 100.0))),      // Diamond
    ];
    let num_shapes = named_shapes.len();

    for (i, (name, shape)) in named_shapes.into_iter().enumerate() {
        // RAINBOW DISTRIBUTION - Each shape gets a different hue
        // Distribute colors evenly across the rainbow.
        let color = Color::hsl(360. * i as f32 / num_shapes as f32, 0.95, 0.7);

        // SPAWN CLICKABLE SHAPES - Each one can be disabled
        commands.spawn((
            name,                    // Human-readable identifier
            DisableOnClick,          // Makes it clickable for disabling
            Mesh2d(shape),           // Visual appearance
            MeshMaterial2d(materials.add(color)),
            Transform::from_xyz(
                // HORIZONTAL LAYOUT - Spread shapes across the screen
                // Distribute shapes from -X_EXTENT/2 to +X_EXTENT/2.
                -X_EXTENT / 2. + i as f32 / (num_shapes - 1) as f32 * X_EXTENT,
                0.0,
                0.0,
            ),
        ));
    }
}

fn display_instructions(mut commands: Commands) {
    // USER GUIDE - Instructions for the demonstration
    commands.spawn((
        Text::new(
            "Click an entity to disable it.\n\nPress Space to re-enable all disabled entities.",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),   // Top-left corner
            ..default()
        },
    ));
}
