//! This interactive example shows how to use distance fog,
//! and allows playing around with different fog settings.
//!
//! 🌫️ The Mystery of the Mist: Understanding Distance Fog
//!
//! Have you ever noticed how distant mountains appear hazy and blue? Or how
//! fog makes nearby objects fade into nothingness? That's atmospheric perspective!
//! In the real world, tiny particles in the air scatter light, making distant
//! objects appear lighter and less distinct. In games, we simulate this with
//! distance fog - a simple but powerful technique that adds depth, atmosphere,
//! and that cinematic quality to any scene!
//!
//! 🎯 What You'll See:
//! - An ancient pyramid with stone pillars and a mystical orb
//! - A camera that orbits around the scene automatically
//! - Real-time fog controls to experiment with different effects
//! - Three different fog falloff modes: Linear, Exponential, and Exponential²
//!
//! ## Controls
//!
//! | Key Binding        | Action                              |
//! |:-------------------|:------------------------------------|
//! | `1` / `2` / `3`    | Fog Falloff Mode                    |
//! | `A` / `S`          | Move Start Distance (Linear Fog)    |
//! |                    | Change Density (Exponential Fogs)   |
//! | `Z` / `X`          | Move End Distance (Linear Fog)      |
//! | `-` / `=`          | Adjust Fog Red Channel              |
//! | `[` / `]`          | Adjust Fog Green Channel            |
//! | `;` / `'`          | Adjust Fog Blue Channel             |
//! | `.` / `?`          | Adjust Fog Alpha Channel            |
//!
//! 🔑 Key Concepts:
//! - Linear Fog: Gradual transition from clear to foggy
//! - Exponential Fog: Natural-looking fog that thickens rapidly
//! - Fog Color: Not just gray! Use color for mood and atmosphere
//! - Falloff: How quickly visibility decreases with distance

use bevy::{
    math::ops,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
};

fn main() {
    App::new()
        // 🌑 No ambient light - makes fog more dramatic!
        .insert_resource(AmbientLight::NONE)
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (setup_camera_fog, setup_pyramid_scene, setup_instructions),
        )
        .add_systems(Update, update_system)
        .run();
}

// 📷 Camera & Fog Setup: Creating the Atmosphere
fn setup_camera_fog(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // 🌫️ Distance Fog Component - The Star of the Show!
        DistanceFog {
            // 🎨 Fog color: Dark gray for a mysterious atmosphere
            // Try different colors: blue for underwater, orange for sunset!
            color: Color::srgb(0.25, 0.25, 0.25),
            
            // 📏 Linear falloff: Like looking through increasingly thick glass
            // - start: Where fog begins to appear
            // - end: Where fog completely obscures vision
            falloff: FogFalloff::Linear {
                start: 5.0,   // Clear visibility up to 5 units
                end: 20.0,    // Complete fog at 20 units
            },
            ..default()
        },
    ));
}

// 🏛️ Scene Setup: Building an Ancient Mystery
fn setup_pyramid_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 🪨 Stone Material: Dark, rough, ancient
    let stone = materials.add(StandardMaterial {
        base_color: Srgba::hex("28221B").unwrap().into(),  // Dark brown-gray
        perceptual_roughness: 1.0,  // Completely rough (no shine)
        ..default()
    });

    // 🏛️ Four Pillars: Guardians of the Pyramid
    // Positioned at the corners of a square
    for (x, z) in &[(-1.5, -1.5), (1.5, -1.5), (1.5, 1.5), (-1.5, 1.5)] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 3.0, 1.0))),  // Tall, thin pillars
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(*x, 1.5, *z),  // Raised off the ground
        ));
    }

    // 🔮 Mystical Orb: The Heart of the Pyramid
    // A glowing, translucent sphere floating above everything
    commands.spawn((
        Mesh3d(meshes.add(Sphere::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            // 💚 Translucent green with hex color including alpha (CC = 80% opacity)
            base_color: Srgba::hex("126212CC").unwrap().into(),
            reflectance: 1.0,            // Maximum reflectivity
            perceptual_roughness: 0.0,   // Mirror-smooth
            metallic: 0.5,               // Half-metallic for alien look
            alpha_mode: AlphaMode::Blend,  // Enable transparency
            ..default()
        })),
        Transform::from_scale(Vec3::splat(1.75))  // Make it bigger
            .with_translation(Vec3::new(0.0, 4.0, 0.0)),  // Float above pyramid
        NotShadowCaster,    // Orb doesn't cast shadows (it glows!)
        NotShadowReceiver,  // Orb ignores shadows (always luminous)
    ));

    // 🏔️ Pyramid Steps: Descending into the Depths
    // Each step is wider than the last, creating the pyramid shape
    for i in 0..50 {
        let half_size = i as f32 / 2.0 + 3.0;  // Each step gets wider
        let y = -i as f32 / 2.0;               // Each step goes lower
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(2.0 * half_size, 0.5, 2.0 * half_size))),
            MeshMaterial3d(stone.clone()),
            Transform::from_xyz(0.0, y + 0.25, 0.0),  // Center each step
        ));
    }

    // 🌌 Sky Box: The Foggy Void Beyond
    // A massive inverted cube that surrounds everything
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Srgba::hex("888888").unwrap().into(),  // Medium gray
            unlit: true,      // Sky doesn't need lighting
            cull_mode: None,  // Render both sides (we're inside!)
            ..default()
        })),
        Transform::from_scale(Vec3::splat(1_000_000.0)),  // ENORMOUS!
    ));

    // 💡 Central Light Source
    // Positioned at the center of the pyramid to cast dramatic shadows
    commands.spawn((
        PointLight {
            shadows_enabled: true,  // Essential for atmosphere!
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
}

// 📝 UI Setup: Instructions Display
fn setup_instructions(mut commands: Commands) {
    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

// 🎮 Main Update System: Camera Movement & Fog Control
fn update_system(
    camera: Single<(&mut DistanceFog, &mut Transform)>,
    mut text: Single<&mut Text>,
    time: Res<Time>,
    keycode: Res<ButtonInput<KeyCode>>,
) {
    let now = time.elapsed_secs();
    let delta = time.delta_secs();

    let (mut fog, mut transform) = camera.into_inner();

    // 🎥 Cinematic Camera Movement
    // The camera orbits the pyramid, moving closer and farther
    // This creates a dynamic view that showcases how fog changes with distance!
    let orbit_scale = 8.0 + ops::sin(now / 10.0) * 7.0;  // Varies from 1 to 15
    *transform = Transform::from_xyz(
        ops::cos(now / 5.0) * orbit_scale,    // X: Circular motion
        12.0 - orbit_scale / 2.0,              // Y: Higher when closer
        ops::sin(now / 5.0) * orbit_scale,     // Z: Circular motion
    )
    .looking_at(Vec3::ZERO, Vec3::Y);  // Always look at pyramid center

    // 📊 Display Current Fog Settings
    text.0 = format!("Fog Falloff: {:?}\nFog Color: {:?}", fog.falloff, fog.color);

    // 🔄 Fog Falloff Mode Switching
    text.push_str("\n\n1 / 2 / 3 - Fog Falloff Mode");

    // 📏 Mode 1: Linear Fog
    // Like walking into a fog bank - gradual transition
    if keycode.pressed(KeyCode::Digit1) {
        if let FogFalloff::Linear { .. } = fog.falloff {
            // Already in linear mode
        } else {
            fog.falloff = FogFalloff::Linear {
                start: 5.0,   // Fog begins here
                end: 20.0,    // Total fog here
            };
        };
    }

    // 📈 Mode 2: Exponential Fog
    // More realistic - fog thickens exponentially with distance
    if keycode.pressed(KeyCode::Digit2) {
        if let FogFalloff::Exponential { .. } = fog.falloff {
            // Already exponential
        } else if let FogFalloff::ExponentialSquared { density } = fog.falloff {
            // Convert from squared to regular exponential
            fog.falloff = FogFalloff::Exponential { density };
        } else {
            fog.falloff = FogFalloff::Exponential { density: 0.07 };
        };
    }

    // 📈² Mode 3: Exponential Squared Fog
    // Even more dramatic - fog thickens VERY quickly!
    if keycode.pressed(KeyCode::Digit3) {
        if let FogFalloff::Exponential { density } = fog.falloff {
            // Convert from regular to squared exponential
            fog.falloff = FogFalloff::ExponentialSquared { density };
        } else if let FogFalloff::ExponentialSquared { .. } = fog.falloff {
            // Already squared
        } else {
            fog.falloff = FogFalloff::ExponentialSquared { density: 0.07 };
        };
    }

    // 🎛️ Linear Fog Controls
    // Adjust the start and end distances for fine control
    if let FogFalloff::Linear { start, end } = &mut fog.falloff {
        text.push_str("\nA / S - Move Start Distance\nZ / X - Move End Distance");

        if keycode.pressed(KeyCode::KeyA) {
            *start -= delta * 3.0;  // Move start closer
        }
        if keycode.pressed(KeyCode::KeyS) {
            *start += delta * 3.0;  // Move start farther
        }
        if keycode.pressed(KeyCode::KeyZ) {
            *end -= delta * 3.0;    // Move end closer
        }
        if keycode.pressed(KeyCode::KeyX) {
            *end += delta * 3.0;    // Move end farther
        }
    }

    // 🌊 Exponential Fog Controls
    // Density controls how quickly fog thickens
    if let FogFalloff::Exponential { density } = &mut fog.falloff {
        text.push_str("\nA / S - Change Density");

        if keycode.pressed(KeyCode::KeyA) {
            // Decrease density (less fog) - multiplicative for smooth control
            *density -= delta * 0.5 * *density;
            if *density < 0.0 {
                *density = 0.0;
            }
        }
        if keycode.pressed(KeyCode::KeyS) {
            // Increase density (more fog)
            *density += delta * 0.5 * *density;
        }
    }

    // 🌊² Exponential Squared Fog Controls
    // Same as above but for squared exponential
    if let FogFalloff::ExponentialSquared { density } = &mut fog.falloff {
        text.push_str("\nA / S - Change Density");

        if keycode.pressed(KeyCode::KeyA) {
            *density -= delta * 0.5 * *density;
            if *density < 0.0 {
                *density = 0.0;
            }
        }
        if keycode.pressed(KeyCode::KeyS) {
            *density += delta * 0.5 * *density;
        }
    }

    // 🎨 Color Controls: Paint Your Fog!
    text.push_str("\n\n- / = - Red\n[ / ] - Green\n; / ' - Blue\n. / ? - Alpha");

    // 🎨 We work in sRGB color space for intuitive control
    // sRGB is what your monitor displays, so adjustments feel natural
    let mut fog_color = Srgba::from(fog.color);
    
    // 🔴 Red Channel
    if keycode.pressed(KeyCode::Minus) {
        fog_color.red = (fog_color.red - 0.1 * delta).max(0.0);
    }
    if keycode.any_pressed([KeyCode::Equal, KeyCode::NumpadEqual]) {
        fog_color.red = (fog_color.red + 0.1 * delta).min(1.0);
    }

    // 🟢 Green Channel
    if keycode.pressed(KeyCode::BracketLeft) {
        fog_color.green = (fog_color.green - 0.1 * delta).max(0.0);
    }
    if keycode.pressed(KeyCode::BracketRight) {
        fog_color.green = (fog_color.green + 0.1 * delta).min(1.0);
    }

    // 🔵 Blue Channel
    if keycode.pressed(KeyCode::Semicolon) {
        fog_color.blue = (fog_color.blue - 0.1 * delta).max(0.0);
    }
    if keycode.pressed(KeyCode::Quote) {
        fog_color.blue = (fog_color.blue + 0.1 * delta).min(1.0);
    }

    // 👻 Alpha Channel (Transparency)
    // Lower alpha = more transparent fog = can see further
    if keycode.pressed(KeyCode::Period) {
        fog_color.alpha = (fog_color.alpha - 0.1 * delta).max(0.0);
    }
    if keycode.pressed(KeyCode::Slash) {
        fog_color.alpha = (fog_color.alpha + 0.1 * delta).min(1.0);
    }

    // 🔄 Convert back from sRGB to linear color space
    fog.color = Color::from(fog_color);
}

// 🎓 Deep Dive: The Mathematics of Fog
//
// **Linear Fog**:
// visibility = (end - distance) / (end - start)
// - Simple linear interpolation
// - Good for corridors or indoor scenes
// - Predictable and easy to control
//
// **Exponential Fog**:
// visibility = e^(-density * distance)
// - Natural exponential decay
// - More realistic for outdoor scenes
// - Matches real atmospheric scattering
//
// **Exponential Squared Fog**:
// visibility = e^(-(density * distance)²)
// - Even more dramatic falloff
// - Good for very thick fog or underwater
// - Can create "wall of fog" effects
//
// The final color is calculated as:
// final_color = fog_color + visibility * (object_color - fog_color)

// 💡 Artistic Tips:
//
// **Horror Games**: 
// - Use dark gray fog with short distances
// - Add slight green or red tint for atmosphere
// - Exponential squared for sudden reveals
//
// **Fantasy Worlds**:
// - Colored fog matching the environment
// - Blue for underwater, orange for lava caves
// - Lower alpha for magical mist effects
//
// **Open World**:
// - Light blue-gray fog for atmospheric perspective
// - Linear fog with large distances
// - Match fog color to sky color
//
// **Performance Note**:
// Fog is calculated per-pixel in the fragment shader,
// so it's very efficient - essentially free on modern GPUs!
