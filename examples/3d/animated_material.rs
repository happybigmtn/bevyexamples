//! Shows how to animate material properties
//!
//! 🎨 The Art of Living Materials
//! Imagine materials that breathe with life - colors that shift like a sunset,
//! surfaces that pulse with energy. This example demonstrates how to animate
//! material properties over time, creating dynamic, ever-changing visuals.
//!
//! 🌈 What You'll See:
//! Nine cubes arranged in a 3x3 grid, each starting with a different color
//! from the rainbow spectrum. As time passes, their colors continuously
//! rotate through the hue spectrum, creating a mesmerizing color dance.
//!
//! 🔑 Key Concepts:
//! - Material Animation: Changing material properties dynamically
//! - HSL Color Space: Perfect for smooth color transitions
//! - The Golden Angle: Nature's favorite spacing for optimal distribution

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_materials)
        .run();
}

// 🏗️ Scene Setup: Creating Our Color Laboratory
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 📷 Camera with Environment Lighting
    // Environment maps provide realistic lighting from all directions
    commands.spawn((
        Camera3d::default(),
        // 👁️ Position camera to see all cubes nicely
        Transform::from_xyz(3.0, 1.0, 3.0).looking_at(Vec3::new(0.0, -0.5, 0.0), Vec3::Y),
        // 🌍 Environment lighting for beautiful reflections
        EnvironmentMapLight {
            // These are pre-baked light maps that simulate sky lighting
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 2_000.0,
            ..default()
        },
    ));

    // 📦 Create a shared cube mesh - we'll reuse this for all cubes
    let cube = meshes.add(Cuboid::new(0.5, 0.5, 0.5));

    // 🌻 The Golden Angle: Nature's Magic Number
    //
    // This angle (≈137.5°) appears everywhere in nature - sunflower seeds,
    // pine cones, spiral galaxies. It provides optimal spacing because it's
    // related to the golden ratio. When we rotate hues by this angle, we get
    // colors that are maximally distinct from each other.
    //
    // Math: 360° × (1 - 1/φ) where φ is the golden ratio
    const GOLDEN_ANGLE: f32 = 137.507_77;

    // 🎨 Create HSL color for easy hue rotation
    // HSL = Hue (color), Saturation (intensity), Lightness (brightness)
    // Starting at red (hue = 0°) with full saturation and medium lightness
    let mut hsla = Hsla::hsl(0.0, 1.0, 0.5);
    
    // 🔄 Create a 3x3 Grid of Cubes
    for x in -1..2 {  // x positions: -1, 0, 1
        for z in -1..2 {  // z positions: -1, 0, 1
            // 🎭 Each cube gets its own material with a unique color
            commands.spawn((
                // Geometry: shared cube mesh
                Mesh3d(cube.clone()),
                // Material: unique color for each cube
                MeshMaterial3d(materials.add(Color::from(hsla))),
                // Position: arranged in a grid on the XZ plane
                Transform::from_translation(Vec3::new(x as f32, 0.0, z as f32)),
            ));
            
            // 🌈 Rotate hue by golden angle for next cube
            // This ensures each cube has a visually distinct color
            hsla = hsla.rotate_hue(GOLDEN_ANGLE);
        }
    }
}

// 🎬 The Animation System: Bringing Materials to Life
//
// This system runs every frame and smoothly animates the colors of all materials.
// It's like having a rainbow that slowly rotates through each cube.
fn animate_materials(
    // 🔍 Find all entities with material handles
    material_handles: Query<&MeshMaterial3d<StandardMaterial>>,
    // ⏰ Time gives us delta time for smooth animation
    time: Res<Time>,
    // 🎨 Material storage where we can modify materials
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 🔄 Iterate through all material handles
    for material_handle in material_handles.iter() {
        // 📦 Try to get mutable access to the actual material
        if let Some(material) = materials.get_mut(material_handle) {
            // 🎨 Check if the material's base color is in HSL format
            if let Color::Hsla(ref mut hsla) = material.base_color {
                // 🌈 Rotate the hue based on time
                // - time.delta_secs() ensures smooth, framerate-independent animation
                // - Multiply by 100.0 to rotate 100 degrees per second
                // - rotate_hue() automatically wraps around at 360°
                *hsla = hsla.rotate_hue(time.delta_secs() * 100.0);
            }
        }
    }
}

// 🎓 Deep Dive: Why HSL for Animation?
//
// RGB (Red, Green, Blue) is how screens display color, but it's terrible
// for animation. Interpolating between RGB values often passes through
// ugly muddy colors.
//
// HSL (Hue, Saturation, Lightness) separates:
// - Hue: The actual color (0-360°, like a color wheel)
// - Saturation: How vivid the color is (0-1, gray to pure color)
// - Lightness: How bright it is (0-1, black to white)
//
// By animating only the hue, we maintain consistent saturation and
// brightness while smoothly transitioning through the rainbow.

// 💡 Material Animation Techniques:
//
// 1. **Color Animation** (shown here): Smooth color transitions
// 2. **Metallic/Roughness**: Animate between shiny and matte
// 3. **Emissive**: Create pulsing lights or glowing effects
// 4. **Normal Maps**: Animate surface details for rippling effects
// 5. **Alpha**: Fade objects in/out with transparency
//
// Performance tip: Animating materials is very efficient because
// it only updates GPU uniforms, not mesh data!

// 🎮 Exercise Ideas:
// - Make colors pulse by animating lightness with sin(time)
// - Animate metallic property to create a "rust spreading" effect
// - Create a wave pattern by offsetting animation based on position
// - Synchronize color changes to music beats
// - Create a "selected" effect by animating emissive glow