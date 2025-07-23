//! This example showcases different blend modes.
//!
//! 🎨 The Art of Digital Mixing: Understanding Blend Modes
//!
//! Imagine you're painting with watercolors on wet paper - the colors blend and
//! mix in beautiful ways. In computer graphics, blend modes control exactly how
//! pixels combine when one object is drawn over another. It's like having different
//! types of magical paint that can add light, multiply shadows, or blend smoothly!
//!
//! 🎯 What You'll See:
//! - Five spheres demonstrating different blend modes
//! - A checkered floor showing through transparent spheres
//! - Interactive controls to adjust transparency and see the effects
//! - Labels that follow the spheres as you rotate the camera
//!
//! ## Controls
//!
//! | Key Binding        | Action                              |
//! |:-------------------|:------------------------------------|
//! | `Up` / `Down`      | Increase / Decrease Alpha           |
//! | `Left` / `Right`   | Rotate Camera                       |
//! | `H`                | Toggle HDR                          |
//! | `Spacebar`         | Toggle Unlit                        |
//! | `C`                | Randomize Colors                    |
//!
//! 🔑 Key Concepts:
//! - Alpha Blending: How transparency works
//! - Premultiplied Alpha: Avoiding color fringing
//! - Additive Blending: Creating glowing effects
//! - Multiplicative Blending: Creating shadows and filters

use bevy::{color::palettes::css::ORANGE, prelude::*, render::view::Hdr};
use rand::random;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, example_control_system);

    app.run();
}

// 🎬 Scene Setup: Creating Our Blend Mode Gallery
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 🎨 Base color for all spheres - a nice pink-red
    let base_color = Color::srgb(0.9, 0.2, 0.3);
    
    // 🌐 High-quality sphere mesh (ico level 7 = very smooth)
    let icosphere_mesh = meshes.add(Sphere::new(0.9).mesh().ico(7).unwrap());

    // 🚫 Opaque Mode: The Solid Wall
    // No transparency at all - this is your standard, everyday rendering
    let opaque = commands
        .spawn((
            Mesh3d(icosphere_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                // 🎯 AlphaMode::Opaque - ignores alpha channel completely
                alpha_mode: AlphaMode::Opaque,
                ..default()
            })),
            Transform::from_xyz(-4.0, 0.0, 0.0),
            ExampleControls {
                unlit: true,
                color: true,
            },
        ))
        .id();

    // 🌊 Blend Mode: The Classic Transparency
    // Standard alpha blending - what you expect from transparent objects
    let blend = commands
        .spawn((
            Mesh3d(icosphere_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                // 🎯 AlphaMode::Blend - classic transparency
                // Formula: output = source * alpha + dest * (1 - alpha)
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(-2.0, 0.0, 0.0),
            ExampleControls {
                unlit: true,
                color: true,
            },
        ))
        .id();

    // 🎭 Premultiplied Mode: The Professional's Choice
    // Prevents color fringing around transparent edges
    let premultiplied = commands
        .spawn((
            Mesh3d(icosphere_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                // 🎯 AlphaMode::Premultiplied - pre-multiplied alpha
                // Colors are already multiplied by alpha, preventing halos
                alpha_mode: AlphaMode::Premultiplied,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
            ExampleControls {
                unlit: true,
                color: true,
            },
        ))
        .id();

    // ✨ Add Mode: The Light Emitter
    // Perfect for glowing effects, fire, and energy
    let add = commands
        .spawn((
            Mesh3d(icosphere_mesh.clone()),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                // 🎯 AlphaMode::Add - adds color values
                // Formula: output = source + dest
                // Makes things brighter, never darker!
                alpha_mode: AlphaMode::Add,
                ..default()
            })),
            Transform::from_xyz(2.0, 0.0, 0.0),
            ExampleControls {
                unlit: true,
                color: true,
            },
        ))
        .id();

    // 🌑 Multiply Mode: The Shadow Caster
    // Darkens everything behind it - great for shadows and filters
    let multiply = commands
        .spawn((
            Mesh3d(icosphere_mesh),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                // 🎯 AlphaMode::Multiply - multiplies color values
                // Formula: output = source * dest
                // Makes things darker, never brighter!
                alpha_mode: AlphaMode::Multiply,
                ..default()
            })),
            Transform::from_xyz(4.0, 0.0, 0.0),
            ExampleControls {
                unlit: true,
                color: true,
            },
        ))
        .id();

    // ♟️ Chessboard Floor: Perfect for Seeing Transparency
    // The pattern makes it easy to see how each blend mode works
    let black_material = materials.add(Color::BLACK);
    let white_material = materials.add(Color::WHITE);

    let plane_mesh = meshes.add(Plane3d::default().mesh().size(2.0, 2.0));

    // 🏁 Create checkered pattern
    for x in -3..4 {
        for z in -3..4 {
            commands.spawn((
                Mesh3d(plane_mesh.clone()),
                MeshMaterial3d(if (x + z) % 2 == 0 {
                    black_material.clone()
                } else {
                    white_material.clone()
                }),
                Transform::from_xyz(x as f32 * 2.0, -1.0, z as f32 * 2.0),
                ExampleControls {
                    unlit: false,
                    color: true,
                },
            ));
        }
    }

    // 💡 Lighting
    commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));

    // 📷 Camera with HDR enabled for better blend mode visualization
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.5, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Hdr,  // High Dynamic Range for better color representation
        // Unfortunately, MSAA and HDR are not supported simultaneously under WebGL
        #[cfg(target_arch = "wasm32")]
        Msaa::Off,
    ));

    // 📝 UI Setup: Instructions and Labels

    let text_style = TextFont {
        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
        ..default()
    };

    let label_text_style = (text_style.clone(), TextColor(ORANGE.into()));

    // Instructions
    commands.spawn((Text::new("Up / Down — Increase / Decrease Alpha\nLeft / Right — Rotate Camera\nH - Toggle HDR\nSpacebar — Toggle Unlit\nC — Randomize Colors"),
            text_style.clone(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        })
    );

    // Status display
    commands.spawn((
        Text::default(),
        text_style,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            right: Val::Px(12.0),
            ..default()
        },
        ExampleDisplay,
    ));

    // 🏷️ Helper function to create labels for each sphere
    let mut label = |entity: Entity, label: &str| {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ExampleLabel { entity },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(label),
                    label_text_style.clone(),
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::ZERO,
                        ..default()
                    },
                    TextLayout::default().with_no_wrap(),
                ));
            });
    };

    // Create labels with box-drawing characters for visual connection
    label(opaque, "┌─ Opaque\n│\n│\n│\n│");
    label(blend, "┌─ Blend\n│\n│\n│");
    label(premultiplied, "┌─ Premultiplied\n│\n│");
    label(add, "┌─ Add\n│");
    label(multiply, "┌─ Multiply");
}

// 🎛️ Control components
#[derive(Component)]
struct ExampleControls {
    unlit: bool,
    color: bool,
}

#[derive(Component)]
struct ExampleLabel {
    entity: Entity,
}

// 🔧 Application state
struct ExampleState {
    alpha: f32,
    unlit: bool,
}

#[derive(Component)]
struct ExampleDisplay;

impl Default for ExampleState {
    fn default() -> Self {
        ExampleState {
            alpha: 0.9,
            unlit: false,
        }
    }
}

// 🎮 Interactive Control System
fn example_control_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    controllable: Query<(&MeshMaterial3d<StandardMaterial>, &ExampleControls)>,
    camera: Single<
        (
            Entity,
            &mut Camera,
            &mut Transform,
            &GlobalTransform,
            Has<Hdr>,
        ),
        With<Camera3d>,
    >,
    mut labels: Query<(&mut Node, &ExampleLabel)>,
    mut display: Single<&mut Text, With<ExampleDisplay>>,
    labeled: Query<&GlobalTransform>,
    mut state: Local<ExampleState>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    // 🎚️ Alpha control - smoothly adjust transparency
    if input.pressed(KeyCode::ArrowUp) {
        state.alpha = (state.alpha + time.delta_secs()).min(1.0);
    } else if input.pressed(KeyCode::ArrowDown) {
        state.alpha = (state.alpha - time.delta_secs()).max(0.0);
    }

    // 💡 Toggle lighting calculation
    if input.just_pressed(KeyCode::Space) {
        state.unlit = !state.unlit;
    }

    // 🎨 Randomize colors for fun!
    let randomize_colors = input.just_pressed(KeyCode::KeyC);

    // 📦 Update all materials
    for (material_handle, controls) in &controllable {
        let material = materials.get_mut(material_handle).unwrap();

        if controls.color && randomize_colors {
            // 🎲 Generate random color with current alpha
            material.base_color = Srgba {
                red: random(),
                green: random(),
                blue: random(),
                alpha: state.alpha,
            }
            .into();
        } else {
            // 🔄 Just update alpha
            material.base_color.set_alpha(state.alpha);
        }

        if controls.unlit {
            material.unlit = state.unlit;
        }
    }

    let (entity, camera, mut camera_transform, camera_global_transform, hdr) = camera.into_inner();

    // 🎬 Toggle HDR
    if input.just_pressed(KeyCode::KeyH) {
        if hdr {
            commands.entity(entity).remove::<Hdr>();
        } else {
            commands.entity(entity).insert(Hdr);
        }
    }

    // 🔄 Camera rotation
    let rotation = if input.pressed(KeyCode::ArrowLeft) {
        time.delta_secs()
    } else if input.pressed(KeyCode::ArrowRight) {
        -time.delta_secs()
    } else {
        0.0
    };

    camera_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(rotation));

    // 📍 Update label positions to follow spheres
    for (mut node, label) in &mut labels {
        let world_position = labeled.get(label.entity).unwrap().translation() + Vec3::Y;

        let viewport_position = camera
            .world_to_viewport(camera_global_transform, world_position)
            .unwrap();

        node.top = Val::Px(viewport_position.y);
        node.left = Val::Px(viewport_position.x);
    }

    // 📊 Update status display
    display.0 = format!(
        "  HDR: {}\nAlpha: {:.2}",
        if hdr { "ON " } else { "OFF" },
        state.alpha
    );
}

// 🎓 Deep Dive: Understanding Blend Modes
//
// When rendering transparent objects, the GPU needs to combine the new pixel
// color with what's already in the framebuffer. The blend equation is:
//
// output = source_color * source_factor + dest_color * dest_factor
//
// Different blend modes use different factors:
//
// 1. **Opaque**: No blending - just overwrites
//    - Use when: Object is fully solid
//    - Performance: Fastest (no blending math)
//
// 2. **Blend** (Traditional Alpha):
//    - source_factor = source_alpha
//    - dest_factor = 1 - source_alpha
//    - Result: Linear interpolation based on alpha
//    - Use when: General transparency (glass, water)
//
// 3. **Premultiplied**:
//    - source_factor = 1
//    - dest_factor = 1 - source_alpha
//    - Colors pre-multiplied by alpha
//    - Use when: Avoiding color fringing, particle systems
//
// 4. **Add**:
//    - source_factor = source_alpha
//    - dest_factor = 1
//    - Always makes things brighter
//    - Use when: Fire, lasers, magic effects
//
// 5. **Multiply**:
//    - source_factor = dest_color
//    - dest_factor = 0
//    - Always makes things darker
//    - Use when: Shadows, tinted glass

// 💡 Pro Tips:
//
// - **Sorting Matters**: Transparent objects must be drawn back-to-front
// - **HDR Benefits**: Better color accuracy with additive blending
// - **Performance**: Opaque > Premultiplied > Blend > Add/Multiply
// - **Artifacts**: Watch for sorting issues with overlapping transparencies
//
// Common Gotchas:
// - Forgetting to sort transparent objects
// - Using Blend mode with pre-multiplied textures
// - Not considering HDR for additive effects
// - Depth buffer issues with transparency