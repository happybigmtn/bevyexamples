//! A simple scene to demonstrate picking events for UI and mesh entities.
//!
//! Picking is like giving your game objects a sense of touch - they can feel when the mouse
//! hovers over them, clicks them, or drags them! Think of it as adding invisible sensors to
//! every object that can detect mouse interactions. This is fundamental for any interactive
//! UI, object selection, or drag-and-drop mechanics. Instead of manually checking if the
//! mouse is over an object, the picking system handles all the ray casting and hit detection
//! automatically!

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin))  // Enable 3D mesh picking
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // INTERACTIVE UI TEXT - Click to spawn, hover to highlight
    commands
        .spawn((
            Text::new("Click Me to get a box\nDrag cubes to rotate"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(12.0),
                left: Val::Percent(12.0),
                ..default()
            },
        ))
        // CLICK OBSERVER - Spawn a cube when text is clicked
        .observe(on_click_spawn_cube)
        // MOUSE LEAVE OBSERVER - Return to white when mouse leaves
        .observe(
            |out: Trigger<Pointer<Out>>, mut texts: Query<&mut TextColor>| {
                let mut text_color = texts.get_mut(out.target()).unwrap();
                text_color.0 = Color::WHITE;  // Back to normal
            },
        )
        // MOUSE HOVER OBSERVER - Highlight cyan when mouse enters
        .observe(
            |over: Trigger<Pointer<Over>>, mut texts: Query<&mut TextColor>| {
                let mut color = texts.get_mut(over.target()).unwrap();
                color.0 = bevy::color::palettes::tailwind::CYAN_400.into();  // Hover highlight
            },
        );

    // GROUND PLANE - White circle as a base
    // Base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),  // Rotate to horizontal
    ));

    // SCENE LIGHTING - Cast shadows for depth perception
    // Light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // VIEWER POSITION - Angled view to see the 3D scene
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn on_click_spawn_cube(
    _click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut num: Local<usize>,  // Persistent counter for stacking
) {
    // SPAWN INTERACTIVE CUBE - Each one gets its own drag handler
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),  // Half-meter cube
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),  // Light blue
            // STACKING FORMULA - Each cube spawns higher than the last
            Transform::from_xyz(0.0, 0.25 + 0.55 * *num as f32, 0.0),
        ))
        // ATTACH DRAG BEHAVIOR - Make the cube rotatable
        // With the MeshPickingPlugin added, you can add pointer event observers to meshes:
        .observe(on_drag_rotate);
    *num += 1;  // Increment for next spawn position
}

// DRAG HANDLER - Convert mouse movement to object rotation
fn on_drag_rotate(drag: Trigger<Pointer<Drag>>, mut transforms: Query<&mut Transform>) {
    if let Ok(mut transform) = transforms.get_mut(drag.target()) {
        // ROTATION MAPPING - Mouse movement to 3D rotation
        transform.rotate_y(drag.delta.x * 0.02);  // Horizontal drag = Y-axis rotation
        transform.rotate_x(drag.delta.y * 0.02);  // Vertical drag = X-axis rotation
        // The 0.02 factor controls sensitivity - like a volume knob for rotation speed
    }
}
