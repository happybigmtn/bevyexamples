//! Demonstrates color grading with an interactive adjustment UI.
//!
//! 🎨 The Digital Darkroom: Understanding Color Grading
//!
//! Remember the last time you used a photo filter on your phone? That's color
//! grading! It's the art of adjusting colors to create mood and atmosphere.
//! In films, it's why The Matrix has that green tint, or why Mexico always
//! looks orange in Hollywood movies. This example lets you play cinematographer,
//! adjusting every aspect of color in real-time!
//!
//! 🎯 What You'll See:
//! - A test scene with various colors and lighting
//! - Interactive buttons to adjust different color properties
//! - Real-time feedback showing how each setting affects the image
//! - Separate controls for highlights, midtones, and shadows
//!
//! 🎮 Controls:
//! - Click any button to select a color grading option
//! - Left/Right arrows to adjust the selected value
//! - Watch the scene transform with your adjustments!
//!
//! 🔑 Key Concepts:
//! - Global Controls: Affect the entire image
//! - Section Controls: Target specific brightness ranges
//! - Highlights: The brightest parts of the image
//! - Midtones: The middle brightness values
//! - Shadows: The darkest areas

use std::{
    f32::consts::PI,
    fmt::{self, Formatter},
};

use bevy::{
    ecs::system::EntityCommands,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::view::{ColorGrading, ColorGradingGlobal, ColorGradingSection, Hdr},
};
use std::fmt::Display;

static FONT_PATH: &str = "fonts/FiraMono-Medium.ttf";

/// How quickly the value changes per frame.
const OPTION_ADJUSTMENT_SPEED: f32 = 0.003;

/// The color grading section that the user has selected: highlights, midtones,
/// or shadows.
#[derive(Clone, Copy, PartialEq)]
enum SelectedColorGradingSection {
    Highlights,
    Midtones,
    Shadows,
}

/// The global option that the user has selected.
///
/// See the documentation of [`ColorGradingGlobal`] for more information about
/// each field here.
#[derive(Clone, Copy, PartialEq, Default)]
enum SelectedGlobalColorGradingOption {
    #[default]
    Exposure,
    Temperature,
    Tint,
    Hue,
}

/// The section-specific option that the user has selected.
///
/// See the documentation of [`ColorGradingSection`] for more information about
/// each field here.
#[derive(Clone, Copy, PartialEq)]
enum SelectedSectionColorGradingOption {
    Saturation,
    Contrast,
    Gamma,
    Gain,
    Lift,
}

/// The color grading option that the user has selected.
#[derive(Clone, Copy, PartialEq, Resource)]
enum SelectedColorGradingOption {
    /// The user has selected a global color grading option: one that applies to
    /// the whole image as opposed to specifically to highlights, midtones, or
    /// shadows.
    Global(SelectedGlobalColorGradingOption),

    /// The user has selected a color grading option that applies only to
    /// highlights, midtones, or shadows.
    Section(
        SelectedColorGradingSection,
        SelectedSectionColorGradingOption,
    ),
}

impl Default for SelectedColorGradingOption {
    fn default() -> Self {
        Self::Global(default())
    }
}

/// Buttons consist of three parts: the button itself, a label child, and a
/// value child. This specifies one of the three entities.
#[derive(Clone, Copy, PartialEq, Component)]
enum ColorGradingOptionWidgetType {
    /// The parent button.
    Button,
    /// The label of the button.
    Label,
    /// The numerical value that the button displays.
    Value,
}

// 🏷️ Widget component that connects UI elements to color grading options
#[derive(Clone, Copy, Component)]
struct ColorGradingOptionWidget {
    widget_type: ColorGradingOptionWidgetType,
    option: SelectedColorGradingOption,
}

/// A marker component for the help text at the top left of the screen.
#[derive(Clone, Copy, Component)]
struct HelpText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SelectedColorGradingOption>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                handle_button_presses,
                adjust_color_grading_option,
                update_ui_state,
            )
                .chain(),
        )
        .run();
}

// 🏗️ Initial Setup: Creating Our Color Laboratory
fn setup(
    mut commands: Commands,
    currently_selected_option: Res<SelectedColorGradingOption>,
    asset_server: Res<AssetServer>,
) {
    // Create the scene.
    add_basic_scene(&mut commands, &asset_server);

    // Create the root UI element.
    let font = asset_server.load(FONT_PATH);
    let color_grading = ColorGrading::default();
    add_buttons(&mut commands, &font, &color_grading);

    // Spawn help text.
    add_help_text(&mut commands, &font, &currently_selected_option);

    // Spawn the camera.
    add_camera(&mut commands, &asset_server, color_grading);
}

// 🎛️ UI Construction: Building the Control Panel
//
// Creates a grid of buttons for adjusting color grading parameters.
// Think of it as a mixing board for colors!
fn add_buttons(commands: &mut Commands, font: &Handle<Font>, color_grading: &ColorGrading) {
    // 📦 Parent container for all controls
    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            row_gap: Val::Px(6.0),
            left: Val::Px(12.0),
            bottom: Val::Px(12.0),
            ..default()
        })
        .with_children(|parent| {
            // 🌍 Global controls row
            add_buttons_for_global_controls(parent, color_grading, font);

            // 🎨 Section-specific controls
            for section in [
                SelectedColorGradingSection::Highlights,
                SelectedColorGradingSection::Midtones,
                SelectedColorGradingSection::Shadows,
            ] {
                add_buttons_for_section(parent, section, color_grading, font);
            }
        });
}

// 🌐 Global Controls: The Master Adjustments
//
// These affect the entire image uniformly, like adjusting the overall
// brightness or warmth of a photo.
fn add_buttons_for_global_controls(
    parent: &mut ChildSpawnerCommands,
    color_grading: &ColorGrading,
    font: &Handle<Font>,
) {
    parent.spawn(Node::default()).with_children(|parent| {
        // Placeholder for alignment
        parent.spawn(Node {
            width: Val::Px(125.0),
            ..default()
        });

        // 🎨 Global adjustment buttons
        for option in [
            SelectedGlobalColorGradingOption::Exposure,     // Overall brightness
            SelectedGlobalColorGradingOption::Temperature,  // Warm/cool balance
            SelectedGlobalColorGradingOption::Tint,        // Green/magenta balance
            SelectedGlobalColorGradingOption::Hue,         // Color wheel rotation
        ] {
            add_button_for_value(
                parent,
                SelectedColorGradingOption::Global(option),
                color_grading,
                font,
            );
        }
    });
}

// 🎭 Section Controls: Targeted Adjustments
//
// These let you adjust specific brightness ranges independently.
// It's like having separate controls for the bright sky, the mid-tone skin,
// and the dark shadows in a portrait!
fn add_buttons_for_section(
    parent: &mut ChildSpawnerCommands,
    section: SelectedColorGradingSection,
    color_grading: &ColorGrading,
    font: &Handle<Font>,
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // 🏷️ Section label
            add_text(parent, &section.to_string(), font, Color::WHITE).insert(Node {
                width: Val::Px(125.0),
                ..default()
            });

            // 🎚️ Section-specific controls
            for option in [
                SelectedSectionColorGradingOption::Saturation,  // Color intensity
                SelectedSectionColorGradingOption::Contrast,    // Light/dark difference
                SelectedSectionColorGradingOption::Gamma,       // Midtone brightness
                SelectedSectionColorGradingOption::Gain,        // Multiply brightness
                SelectedSectionColorGradingOption::Lift,        // Add brightness
            ] {
                add_button_for_value(
                    parent,
                    SelectedColorGradingOption::Section(section, option),
                    color_grading,
                    font,
                );
            }
        });
}

// 🔘 Individual Button Creation
//
// Each button shows both the parameter name and its current value
fn add_button_for_value(
    parent: &mut ChildSpawnerCommands,
    option: SelectedColorGradingOption,
    color_grading: &ColorGrading,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                border: UiRect::all(Val::Px(1.0)),
                width: Val::Px(200.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                margin: UiRect::right(Val::Px(12.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
            BorderRadius::MAX,
            BackgroundColor(Color::BLACK),
        ))
        .insert(ColorGradingOptionWidget {
            widget_type: ColorGradingOptionWidgetType::Button,
            option,
        })
        .with_children(|parent| {
            // 📝 Button label
            let label = match option {
                SelectedColorGradingOption::Global(option) => option.to_string(),
                SelectedColorGradingOption::Section(_, option) => option.to_string(),
            };
            add_text(parent, &label, font, Color::WHITE).insert(ColorGradingOptionWidget {
                widget_type: ColorGradingOptionWidgetType::Label,
                option,
            });

            // Spacer
            parent.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });

            // 🔢 Current value display
            add_text(
                parent,
                &format!("{:.3}", option.get(color_grading)),
                font,
                Color::WHITE,
            )
            .insert(ColorGradingOptionWidget {
                widget_type: ColorGradingOptionWidgetType::Value,
                option,
            });
        });
}

// 💬 Help Text Creation
fn add_help_text(
    commands: &mut Commands,
    font: &Handle<Font>,
    currently_selected_option: &SelectedColorGradingOption,
) {
    commands.spawn((
        Text::new(create_help_text(currently_selected_option)),
        TextFont {
            font: font.clone(),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
        HelpText,
    ));
}

// 📝 Text Helper Function
fn add_text<'a>(
    parent: &'a mut ChildSpawnerCommands,
    label: &str,
    font: &Handle<Font>,
    color: Color,
) -> EntityCommands<'a> {
    parent.spawn((
        Text::new(label),
        TextFont {
            font: font.clone(),
            font_size: 15.0,
            ..default()
        },
        TextColor(color),
    ))
}

// 📷 Camera Setup with Color Grading
fn add_camera(commands: &mut Commands, asset_server: &AssetServer, color_grading: ColorGrading) {
    commands.spawn((
        Camera3d::default(),
        Hdr,  // High Dynamic Range for better color grading
        Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
        color_grading,  // 🎨 The magic happens here!
        // 🌫️ Fog adds depth
        DistanceFog {
            color: Color::srgb_u8(43, 44, 47),
            falloff: FogFalloff::Linear {
                start: 1.0,
                end: 8.0,
            },
            ..default()
        },
        // 🌍 Environment lighting
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2000.0,
            ..default()
        },
    ));
}

// 🎬 Scene Setup: Our Test Environment
fn add_basic_scene(commands: &mut Commands, asset_server: &AssetServer) {
    // 🎯 Tone mapping test scene - perfect for color grading experiments
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/TonemappingTest/TonemappingTest.gltf"),
    )));

    // 🚁 Flight helmet for detail testing
    commands.spawn((
        SceneRoot(
            asset_server
                .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
        Transform::from_xyz(0.5, 0.0, -0.5).with_rotation(Quat::from_rotation_y(-0.15 * PI)),
    ));

    // ☀️ Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI * -0.15, PI * -0.15)),
        CascadeShadowConfigBuilder {
            maximum_distance: 3.0,
            first_cascade_far_bound: 0.9,
            ..default()
        }
        .build(),
    ));
}

// 🎨 Display implementations for UI labels
impl Display for SelectedGlobalColorGradingOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match *self {
            SelectedGlobalColorGradingOption::Exposure => "Exposure",
            SelectedGlobalColorGradingOption::Temperature => "Temperature",
            SelectedGlobalColorGradingOption::Tint => "Tint",
            SelectedGlobalColorGradingOption::Hue => "Hue",
        };
        f.write_str(name)
    }
}

impl Display for SelectedColorGradingSection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match *self {
            SelectedColorGradingSection::Highlights => "Highlights",
            SelectedColorGradingSection::Midtones => "Midtones",
            SelectedColorGradingSection::Shadows => "Shadows",
        };
        f.write_str(name)
    }
}

impl Display for SelectedSectionColorGradingOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match *self {
            SelectedSectionColorGradingOption::Saturation => "Saturation",
            SelectedSectionColorGradingOption::Contrast => "Contrast",
            SelectedSectionColorGradingOption::Gamma => "Gamma",
            SelectedSectionColorGradingOption::Gain => "Gain",
            SelectedSectionColorGradingOption::Lift => "Lift",
        };
        f.write_str(name)
    }
}

impl Display for SelectedColorGradingOption {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SelectedColorGradingOption::Global(option) => write!(f, "\"{option}\""),
            SelectedColorGradingOption::Section(section, option) => {
                write!(f, "\"{option}\" for \"{section}\"")
            }
        }
    }
}

// 📊 Value Getters and Setters
impl SelectedSectionColorGradingOption {
    /// Returns the appropriate value in the given color grading section.
    fn get(&self, section: &ColorGradingSection) -> f32 {
        match *self {
            SelectedSectionColorGradingOption::Saturation => section.saturation,
            SelectedSectionColorGradingOption::Contrast => section.contrast,
            SelectedSectionColorGradingOption::Gamma => section.gamma,
            SelectedSectionColorGradingOption::Gain => section.gain,
            SelectedSectionColorGradingOption::Lift => section.lift,
        }
    }

    fn set(&self, section: &mut ColorGradingSection, value: f32) {
        match *self {
            SelectedSectionColorGradingOption::Saturation => section.saturation = value,
            SelectedSectionColorGradingOption::Contrast => section.contrast = value,
            SelectedSectionColorGradingOption::Gamma => section.gamma = value,
            SelectedSectionColorGradingOption::Gain => section.gain = value,
            SelectedSectionColorGradingOption::Lift => section.lift = value,
        }
    }
}

impl SelectedGlobalColorGradingOption {
    fn get(&self, global: &ColorGradingGlobal) -> f32 {
        match *self {
            SelectedGlobalColorGradingOption::Exposure => global.exposure,
            SelectedGlobalColorGradingOption::Temperature => global.temperature,
            SelectedGlobalColorGradingOption::Tint => global.tint,
            SelectedGlobalColorGradingOption::Hue => global.hue,
        }
    }

    fn set(&self, global: &mut ColorGradingGlobal, value: f32) {
        match *self {
            SelectedGlobalColorGradingOption::Exposure => global.exposure = value,
            SelectedGlobalColorGradingOption::Temperature => global.temperature = value,
            SelectedGlobalColorGradingOption::Tint => global.tint = value,
            SelectedGlobalColorGradingOption::Hue => global.hue = value,
        }
    }
}

impl SelectedColorGradingOption {
    fn get(&self, color_grading: &ColorGrading) -> f32 {
        match self {
            SelectedColorGradingOption::Global(option) => option.get(&color_grading.global),
            SelectedColorGradingOption::Section(
                SelectedColorGradingSection::Highlights,
                option,
            ) => option.get(&color_grading.highlights),
            SelectedColorGradingOption::Section(SelectedColorGradingSection::Midtones, option) => {
                option.get(&color_grading.midtones)
            }
            SelectedColorGradingOption::Section(SelectedColorGradingSection::Shadows, option) => {
                option.get(&color_grading.shadows)
            }
        }
    }

    fn set(&self, color_grading: &mut ColorGrading, value: f32) {
        match self {
            SelectedColorGradingOption::Global(option) => {
                option.set(&mut color_grading.global, value);
            }
            SelectedColorGradingOption::Section(
                SelectedColorGradingSection::Highlights,
                option,
            ) => option.set(&mut color_grading.highlights, value),
            SelectedColorGradingOption::Section(SelectedColorGradingSection::Midtones, option) => {
                option.set(&mut color_grading.midtones, value);
            }
            SelectedColorGradingOption::Section(SelectedColorGradingSection::Shadows, option) => {
                option.set(&mut color_grading.shadows, value);
            }
        }
    }
}

// 🖱️ Button Click Handler
fn handle_button_presses(
    mut interactions: Query<(&Interaction, &ColorGradingOptionWidget), Changed<Interaction>>,
    mut currently_selected_option: ResMut<SelectedColorGradingOption>,
) {
    for (interaction, widget) in interactions.iter_mut() {
        if widget.widget_type == ColorGradingOptionWidgetType::Button
            && *interaction == Interaction::Pressed
        {
            *currently_selected_option = widget.option;
        }
    }
}

// 🎨 UI State Update System
//
// Updates button appearance and values based on current selection
fn update_ui_state(
    mut buttons: Query<(
        &mut BackgroundColor,
        &mut BorderColor,
        &ColorGradingOptionWidget,
    )>,
    button_text: Query<(Entity, &ColorGradingOptionWidget), (With<Text>, Without<HelpText>)>,
    help_text: Single<Entity, With<HelpText>>,
    mut writer: TextUiWriter,
    cameras: Single<Ref<ColorGrading>>,
    currently_selected_option: Res<SelectedColorGradingOption>,
) {
    // Exit early if the UI didn't change
    if !currently_selected_option.is_changed() && !cameras.is_changed() {
        return;
    }

    // 🎨 Update button colors - selected button gets inverted colors
    for (mut background, mut border_color, widget) in buttons.iter_mut() {
        if *currently_selected_option == widget.option {
            *background = Color::WHITE.into();
            *border_color = Color::BLACK.into();
        } else {
            *background = Color::BLACK.into();
            *border_color = Color::WHITE.into();
        }
    }

    let value_label = format!("{:.3}", currently_selected_option.get(cameras.as_ref()));

    // 📝 Update button text
    for (entity, widget) in button_text.iter() {
        // Set text color based on selection
        let color = if *currently_selected_option == widget.option {
            Color::BLACK
        } else {
            Color::WHITE
        };

        writer.for_each_color(entity, |mut text_color| {
            text_color.0 = color;
        });

        // Update value display for selected option
        if widget.widget_type == ColorGradingOptionWidgetType::Value
            && *currently_selected_option == widget.option
        {
            writer.for_each_text(entity, |mut text| {
                text.clone_from(&value_label);
            });
        }
    }

    // Update help text
    *writer.text(*help_text, 0) = create_help_text(&currently_selected_option);
}

fn create_help_text(currently_selected_option: &SelectedColorGradingOption) -> String {
    format!("Press Left/Right to adjust {currently_selected_option}")
}

// ⌨️ Keyboard Input Handler
//
// Adjusts the selected color grading value based on arrow key input
fn adjust_color_grading_option(
    mut color_grading: Single<&mut ColorGrading>,
    input: Res<ButtonInput<KeyCode>>,
    currently_selected_option: Res<SelectedColorGradingOption>,
) {
    let mut delta = 0.0;
    if input.pressed(KeyCode::ArrowLeft) {
        delta -= OPTION_ADJUSTMENT_SPEED;
    }
    if input.pressed(KeyCode::ArrowRight) {
        delta += OPTION_ADJUSTMENT_SPEED;
    }

    if delta != 0.0 {
        let new_value = currently_selected_option.get(color_grading.as_ref()) + delta;
        currently_selected_option.set(&mut color_grading, new_value);
    }
}

// 🎓 Deep Dive: Understanding Color Grading Parameters
//
// **Global Parameters** (affect entire image):
// - Exposure: Overall brightness (-10 to 10, 0 = neutral)
// - Temperature: Blue/orange balance (-1 to 1, 0 = neutral)
// - Tint: Green/magenta balance (-1 to 1, 0 = neutral)
// - Hue: Rotates all colors on the color wheel (-PI to PI)
//
// **Section Parameters** (affect brightness ranges):
// - Saturation: Color intensity (0 = grayscale, 1 = normal, >1 = vivid)
// - Contrast: Difference between lights and darks (1 = normal)
// - Gamma: Midtone brightness (1 = normal, <1 = darker, >1 = lighter)
// - Gain: Multiplies brightness (1 = normal)
// - Lift: Adds brightness (0 = normal)
//
// **The Three-Way Split**:
// - Shadows: Darkest 1/3 of the image
// - Midtones: Middle 1/3 of the image
// - Highlights: Brightest 1/3 of the image
//
// This separation allows targeted adjustments - you can warm up
// skin tones (midtones) while keeping shadows cool and moody!

// 💡 Artistic Tips:
//
// **Cinematic Looks**:
// - Teal & Orange: Cool shadows (negative temperature), warm highlights
// - Film Noir: Low saturation, high contrast, lifted shadows
// - Vintage: Warm temperature, lifted shadows, reduced contrast
//
// **Common Techniques**:
// - Day for Night: Reduce exposure, add blue temperature, increase contrast
// - Golden Hour: Warm temperature, slight magenta tint, soft contrast
// - Bleach Bypass: Desaturate, increase contrast, cool temperature
//
// **Professional Workflow**:
// 1. Set exposure first (overall brightness)
// 2. Adjust temperature/tint for white balance
// 3. Fine-tune with section controls
// 4. Add creative color shifts with hue