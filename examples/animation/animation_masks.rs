//! Demonstrates how to use masks to limit the scope of animations.
//!
//! Animation masks are like stencils - they control which parts of a character move!
//! Imagine you want a character to wave while walking. Without masks, you'd need
//! separate animations for "walk", "wave", and "walk while waving". With masks,
//! you can play "walk" on the legs and "wave" on the arm simultaneously!
//! This example shows a fox where you can toggle animations for individual body parts.
//!
//! ## Animation Theory: Masking and Layering
//!
//! Animation masking solves the combinatorial explosion problem:
//! - Without masks: Need N×M animations for N actions × M variations
//! - With masks: Need only N+M animations that combine dynamically
//!
//! Masking concepts:
//! 1. **Bone Masks**: Include/exclude specific bones
//! 2. **Weight Masks**: Partial influence (0-100%)
//! 3. **Hierarchical Masks**: Affect bone and all children
//! 4. **Additive Masks**: Layer on top of base animation
//!
//! ## Game Feel Context: Expressive Characters
//!
//! Masks enable rich character expression:
//! - **Combat**: Upper body attacks while lower body moves
//! - **Dialogue**: Facial animation while body idles
//! - **Reactions**: Head tracking while performing actions
//! - **Procedural**: Breathing layered on any animation
//!
//! This example demonstrates:
//! - Independent control of 6 body part groups
//! - Mixing different animations on different parts
//! - Real-time mask switching
//!
//! ## Performance Optimization: Mask Evaluation
//!
//! Mask optimization strategies:
//! 1. **Bit Masks**: Pack bone inclusion into bit fields
//! 2. **Mask Caching**: Pre-compute mask combinations
//! 3. **Sparse Evaluation**: Skip masked-out bones
//! 4. **LOD Masks**: Simpler masks for distant characters
//! 5. **Mask Pooling**: Reuse common mask patterns
//!
//! Memory considerations:
//! - Masks add per-bone storage overhead
//! - Consider shared masks for similar characters
//! - Compress mask data when possible
//!
//! ## Real-World Applications: AAA Masking
//!
//! Games using advanced masking:
//! - **The Last of Us**: Contextual facial expressions
//! - **Red Dead Redemption 2**: Layered horse animations
//! - **Overwatch**: Hero-specific animation layers
//! - **Assassin's Creed**: Parkour with upper body variations
//!
//! Common mask patterns:
//! 1. **Upper/Lower Split**: Different animations above/below waist
//! 2. **Limb Isolation**: Individual arm/leg control
//! 3. **Face Masks**: Separate eye, mouth, brow regions
//! 4. **Prop Masks**: Weapon/tool specific bones
//! 5. **Physics Masks**: Blend animation with ragdoll
//!
//! ## Advanced Techniques: Mask Systems
//!
//! 1. **Dynamic Masks**: Change masks based on gameplay
//! 2. **Weighted Masks**: Gradual influence falloff
//! 3. **Mask Blending**: Smooth transitions between masks
//! 4. **Contextual Masks**: Auto-select based on state
//! 5. **Networked Masks**: Sync minimal data for multiplayer
//!
//! ## Common Issues and Solutions
//!
//! 1. **Mask Bleeding**: Animation affects wrong bones
//!    - Solution: Verify bone hierarchy paths
//!
//! 2. **Performance**: Too many mask groups
//!    - Solution: Merge similar groups, use LOD
//!
//! 3. **Blend Artifacts**: Weird poses at mask boundaries
//!    - Solution: Overlap masks or use blend weights
//!
//! 4. **Complexity**: Hard to manage many masks
//!    - Solution: Visual mask editor tools

use bevy::{
    animation::{AnimationTarget, AnimationTargetId},
    color::palettes::css::{LIGHT_GRAY, WHITE},
    prelude::*,
};
use std::collections::HashSet;

// MASK GROUP IDENTIFIERS
// Each body part gets a unique ID for its mask group
// Think of these as channels - like having separate volume controls for different instruments
const MASK_GROUP_HEAD: u32 = 0;
const MASK_GROUP_LEFT_FRONT_LEG: u32 = 1;
const MASK_GROUP_RIGHT_FRONT_LEG: u32 = 2;
const MASK_GROUP_LEFT_HIND_LEG: u32 = 3;
const MASK_GROUP_RIGHT_HIND_LEG: u32 = 4;
const MASK_GROUP_TAIL: u32 = 5;

// UI layout constant - width of the control buttons
const MASK_GROUP_BUTTON_WIDTH: f32 = 250.0;

// BONE HIERARCHY PATHS
// Each mask group is defined by a bone chain - a path through the skeleton
// Format: (prefix_path, suffix_path)
// - The prefix is the common ancestor path
// - The suffix defines additional bones in the chain
// Example: ("A/B/C", "D/E") includes bones: "A/B/C", "A/B/C/D", "A/B/C/D/E"
//
// This hierarchical approach works because body parts are typically chains:
// shoulder -> upper arm -> forearm -> hand
// But you could define any arbitrary set of bones in a mask group!
const MASK_GROUP_PATHS: [(&str, &str); 6] = [
    // Head (neck and head bones)
    (
        "root/_rootJoint/b_Root_00/b_Hip_01/b_Spine01_02/b_Spine02_03",
        "b_Neck_04/b_Head_05",
    ),
    // Left front leg (shoulder to paw)
    (
        "root/_rootJoint/b_Root_00/b_Hip_01/b_Spine01_02/b_Spine02_03/b_LeftUpperArm_09",
        "b_LeftForeArm_010/b_LeftHand_011",
    ),
    // Right front leg (shoulder to paw)
    (
        "root/_rootJoint/b_Root_00/b_Hip_01/b_Spine01_02/b_Spine02_03/b_RightUpperArm_06",
        "b_RightForeArm_07/b_RightHand_08",
    ),
    // Left hind leg (hip to foot)
    (
        "root/_rootJoint/b_Root_00/b_Hip_01/b_LeftLeg01_015",
        "b_LeftLeg02_016/b_LeftFoot01_017/b_LeftFoot02_018",
    ),
    // Right hind leg (hip to foot)
    (
        "root/_rootJoint/b_Root_00/b_Hip_01/b_RightLeg01_019",
        "b_RightLeg02_020/b_RightFoot01_021/b_RightFoot02_022",
    ),
    // Tail (all tail segments)
    (
        "root/_rootJoint/b_Root_00/b_Hip_01/b_Tail01_012",
        "b_Tail02_013/b_Tail03_014",
    ),
];

// Component for UI buttons that control animations
#[derive(Clone, Copy, Component)]
struct AnimationControl {
    // Which body part this button controls (0-5)
    group_id: u32,
    // Which animation to play (Idle, Walk, Run, Off)
    label: AnimationLabel,
}

// The different animation states available
// The values (0,1,2,3) correspond to animation clip indices
#[derive(Clone, Copy, Component, PartialEq, Debug)]
enum AnimationLabel {
    Idle = 0,  // Standing still animation
    Walk = 1,  // Walking animation
    Run = 2,   // Running animation
    Off = 3,   // No animation (freeze frame)
}

// Stores the animation graph node indices for our three animation clips
#[derive(Clone, Debug, Resource)]
struct AnimationNodes([AnimationNodeIndex; 3]);

// Global state tracking which animation plays on each body part
#[derive(Clone, Copy, Debug, Resource)]
struct AppState([MaskGroupState; 6]); // One state per mask group

// State for a single mask group
#[derive(Clone, Copy, Debug)]
struct MaskGroupState {
    clip: u8, // Which animation clip is active (0=Idle, 1=Walk, 2=Run, 3=Off)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Animation Masks Example".into(),
                ..default()
            }),
            ..default()
        }))
        // Startup systems
        .add_systems(Startup, (setup_scene, setup_ui))
        // Update systems
        .add_systems(Update, setup_animation_graph_once_loaded) // Runs once when fox loads
        .add_systems(Update, handle_button_toggles)            // Handles UI interactions
        .add_systems(Update, update_ui)                        // Updates button visuals
        // Global lighting
        .insert_resource(AmbientLight {
            color: WHITE.into(),
            brightness: 100.0,
            ..default()
        })
        // Initialize app state with default values
        .init_resource::<AppState>()
        .run();
}

// Sets up the 3D scene with camera, lights, and the animated fox model
fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera - positioned to see the fox clearly
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-15.0, 10.0, 20.0)  // Back and to the side
            .looking_at(Vec3::new(0., 1., 0.), Vec3::Y), // Look at fox center
    ));

    // Point light for shadows and depth
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, 13.0),
    ));

    // Load the fox model
    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/animated/Fox.glb")),
        ),
        Transform::from_scale(Vec3::splat(0.07)), // Scale down the fox
    ));

    // Ground plane - a green circle
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(7.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        // Rotate to lay flat (circles spawn vertical by default)
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
}

// Creates the user interface with control buttons for each body part
fn setup_ui(mut commands: Commands) {
    // Help text at the top
    commands.spawn((
        Text::new("Click on a button to toggle animations for its associated bones"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
    ));

    // Container for all control buttons - positioned at bottom left
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column, // Stack vertically
            position_type: PositionType::Absolute,
            row_gap: Val::Px(6.0),                // Space between rows
            left: Val::Px(12.0),
            bottom: Val::Px(12.0),
            ..default()
        })
        .with_children(|parent| {
            // Template for horizontal button rows
            let row_node = Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(6.0), // Space between buttons
                ..default()
            };

            // Head control (full width)
            add_mask_group_control(parent, "Head", Val::Auto, MASK_GROUP_HEAD);

            // Front legs (side by side)
            parent.spawn(row_node.clone()).with_children(|parent| {
                add_mask_group_control(
                    parent,
                    "Left Front Leg",
                    Val::Px(MASK_GROUP_BUTTON_WIDTH),
                    MASK_GROUP_LEFT_FRONT_LEG,
                );
                add_mask_group_control(
                    parent,
                    "Right Front Leg",
                    Val::Px(MASK_GROUP_BUTTON_WIDTH),
                    MASK_GROUP_RIGHT_FRONT_LEG,
                );
            });

            // Hind legs (side by side)
            parent.spawn(row_node).with_children(|parent| {
                add_mask_group_control(
                    parent,
                    "Left Hind Leg",
                    Val::Px(MASK_GROUP_BUTTON_WIDTH),
                    MASK_GROUP_LEFT_HIND_LEG,
                );
                add_mask_group_control(
                    parent,
                    "Right Hind Leg",
                    Val::Px(MASK_GROUP_BUTTON_WIDTH),
                    MASK_GROUP_RIGHT_HIND_LEG,
                );
            });

            // Tail control (full width)
            add_mask_group_control(parent, "Tail", Val::Auto, MASK_GROUP_TAIL);
        });
}

// Helper function to create a control widget for one body part
// Each widget has a label and four buttons: Run, Walk, Idle, Off
fn add_mask_group_control(
    parent: &mut ChildSpawnerCommands,
    label: &str,          // Display name (e.g., "Head")
    width: Val,           // Widget width
    mask_group_id: u32,   // Which body part this controls
) {
    // Text styles for different UI states
    let button_text_style = (
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor::WHITE, // Default: white text
    );
    let selected_button_text_style = (button_text_style.0.clone(), TextColor::BLACK); // Selected: black text
    let label_text_style = (
        button_text_style.0.clone(),
        TextColor(Color::Srgba(LIGHT_GRAY)), // Labels: gray text
    );

    // Main container for this control widget
    parent
        .spawn((
            Node {
                border: UiRect::all(Val::Px(1.0)), // 1px white border
                width,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::ZERO,
                margin: UiRect::ZERO,
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BorderRadius::all(Val::Px(3.0)), // Rounded corners
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|builder| {
            // Label section (e.g., "Head")
            builder
                .spawn((
                    Node {
                        border: UiRect::ZERO,
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::ZERO,
                        margin: UiRect::ZERO,
                        ..default()
                    },
                    BackgroundColor(Color::BLACK),
                ))
                .with_child((
                    Text::new(label),
                    label_text_style.clone(),
                    Node {
                        margin: UiRect::vertical(Val::Px(3.0)), // Vertical padding
                        ..default()
                    },
                ));

            // Button row container
            builder
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::top(Val::Px(1.0)), // Separator line
                        ..default()
                    },
                    BorderColor::all(Color::WHITE),
                ))
                .with_children(|builder| {
                    // Create four buttons: Run, Walk, Idle, Off
                    for (index, label) in [
                        AnimationLabel::Run,
                        AnimationLabel::Walk,
                        AnimationLabel::Idle,
                        AnimationLabel::Off,
                    ]
                    .iter()
                    .enumerate()
                    {
                        builder
                            .spawn((
                                Button, // Makes this clickable
                                // First button (Run) starts selected (white bg)
                                BackgroundColor(if index > 0 {
                                    Color::BLACK
                                } else {
                                    Color::WHITE
                                }),
                                Node {
                                    flex_grow: 1.0, // Equal width buttons
                                    // Add separator between buttons (except first)
                                    border: if index > 0 {
                                        UiRect::left(Val::Px(1.0))
                                    } else {
                                        UiRect::ZERO
                                    },
                                    ..default()
                                },
                                BorderColor::all(Color::WHITE),
                                // Attach control data to button
                                AnimationControl {
                                    group_id: mask_group_id,
                                    label: *label,
                                },
                            ))
                            .with_child((
                                Text(format!("{:?}", label)), // "Run", "Walk", etc.
                                // Selected button has inverted colors
                                if index > 0 {
                                    button_text_style.clone()
                                } else {
                                    selected_button_text_style.clone()
                                },
                                TextLayout::new_with_justify(JustifyText::Center),
                                Node {
                                    flex_grow: 1.0,
                                    margin: UiRect::vertical(Val::Px(3.0)),
                                    ..default()
                                },
                            ));
                    }
                });
        });
}

// Sets up the animation graph with mask groups once the fox model is loaded
// This is where the magic happens - we define which bones belong to which mask group
fn setup_animation_graph_once_loaded(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    targets: Query<(Entity, &AnimationTarget)>,
) {
    for (entity, mut player) in &mut players {
        // CREATE THE ANIMATION GRAPH
        let mut animation_graph = AnimationGraph::new();
        // Add an additive blend node - this allows multiple animations to combine
        let blend_node = animation_graph.add_additive_blend(1.0, animation_graph.root);

        // Load all three animation clips and add them to the graph
        let animation_graph_nodes: [AnimationNodeIndex; 3] =
            std::array::from_fn(|animation_index| {
                let handle = asset_server.load(
                    GltfAssetLabel::Animation(animation_index)
                        .from_asset("models/animated/Fox.glb"),
                );
                // Initial mask: 0 = no body parts, 0x3f = all 6 body parts (binary: 111111)
                let mask = if animation_index == 0 { 0 } else { 0x3f };
                animation_graph.add_clip_with_mask(handle, mask, 1.0, blend_node)
            });

        // CONFIGURE MASK GROUPS
        // This is where we tell Bevy which bones belong to which mask group
        let mut all_animation_target_ids = HashSet::new();
        for (mask_group_index, (mask_group_prefix, mask_group_suffix)) in
            MASK_GROUP_PATHS.iter().enumerate()
        {
            // Convert bone paths to Name components
            let prefix: Vec<_> = mask_group_prefix.split('/').map(Name::new).collect();
            let suffix: Vec<_> = mask_group_suffix.split('/').map(Name::new).collect();

            // Add each bone in the chain to this mask group
            // Example: If suffix is ["A", "B", "C"], we add:
            // - prefix (chain_length=0)
            // - prefix/A (chain_length=1)
            // - prefix/A/B (chain_length=2)
            // - prefix/A/B/C (chain_length=3)
            for chain_length in 0..=suffix.len() {
                let animation_target_id = AnimationTargetId::from_names(
                    prefix.iter().chain(suffix[0..chain_length].iter()),
                );
                // Register this bone with its mask group
                animation_graph
                    .add_target_to_mask_group(animation_target_id, mask_group_index as u32);
                all_animation_target_ids.insert(animation_target_id);
            }
        }

        // Finalize and store the animation graph
        let animation_graph = animation_graphs.add(animation_graph);
        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animation_graph));

        // CLEANUP: Remove bones that aren't in any mask group
        // Without this, unmasked bones would play ALL animations simultaneously!
        for (target_entity, target) in &targets {
            if !all_animation_target_ids.contains(&target.id) {
                commands.entity(target_entity).remove::<AnimationTarget>();
            }
        }

        // Start playing all animations (they'll be masked appropriately)
        for animation_graph_node in animation_graph_nodes {
            player.play(animation_graph_node).repeat();
        }

        // Store node indices for later reference
        commands.insert_resource(AnimationNodes(animation_graph_nodes));
    }
}

// Handles button clicks to change which animation plays on each body part
fn handle_button_toggles(
    mut interactions: Query<(&Interaction, &mut AnimationControl), Changed<Interaction>>,
    mut animation_players: Query<&AnimationGraphHandle, With<AnimationPlayer>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut animation_nodes: Option<ResMut<AnimationNodes>>,
    mut app_state: ResMut<AppState>,
) {
    let Some(ref mut animation_nodes) = animation_nodes else {
        return; // Animation not loaded yet
    };

    for (interaction, animation_control) in interactions.iter_mut() {
        // Only respond to button presses (not hovers or releases)
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Update app state to track which animation is selected
        app_state.0[animation_control.group_id as usize].clip = animation_control.label as u8;

        // UPDATE THE ANIMATION MASKS
        for animation_graph_handle in animation_players.iter_mut() {
            let Some(animation_graph) = animation_graphs.get_mut(animation_graph_handle) else {
                continue;
            };

            // Update mask bits for each animation clip
            for (clip_index, &animation_node_index) in animation_nodes.0.iter().enumerate() {
                let Some(animation_node) = animation_graph.get_mut(animation_node_index) else {
                    continue;
                };

                // BITWISE MASK OPERATIONS
                // Each body part is represented by a bit in the mask:
                // Bit 0: Head, Bit 1: Left Front Leg, etc.
                // Example: mask = 0b000101 means Head and Left Hind Leg are active
                
                if animation_control.label as usize == clip_index {
                    // Enable this body part for the selected animation
                    // Clear the bit: mask &= ~(1 << group_id)
                    // Example: If group_id=2, ~(1<<2) = ~0b000100 = 0b111011
                    animation_node.mask &= !(1 << animation_control.group_id);
                } else {
                    // Disable this body part for other animations
                    // Set the bit: mask |= (1 << group_id)
                    // Example: If group_id=2, (1<<2) = 0b000100
                    animation_node.mask |= 1 << animation_control.group_id;
                }
            }
        }
    }
}

// Updates button appearance to show which animation is selected
fn update_ui(
    mut animation_controls: Query<(&AnimationControl, &mut BackgroundColor, &Children)>,
    texts: Query<Entity, With<Text>>,
    mut writer: TextUiWriter,
    app_state: Res<AppState>,
) {
    for (animation_control, mut background_color, kids) in animation_controls.iter_mut() {
        // Check if this button represents the currently selected animation
        let enabled =
            app_state.0[animation_control.group_id as usize].clip == animation_control.label as u8;

        // Update button background (white = selected, black = not selected)
        *background_color = if enabled {
            BackgroundColor(Color::WHITE)
        } else {
            BackgroundColor(Color::BLACK)
        };

        // Update text color for all child text entities (inverse of background)
        for &kid in kids {
            let Ok(text) = texts.get(kid) else {
                continue;
            };

            writer.for_each_color(text, |mut color| {
                color.0 = if enabled { Color::BLACK } else { Color::WHITE };
            });
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        // All body parts start with animation 0 (Idle)
        AppState([MaskGroupState { clip: 0 }; 6])
    }
}
