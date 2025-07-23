//! This example demonstrates how to hot patch systems.
//!
//! It needs to be run with the dioxus CLI:
//! ```sh
//! dx serve --hot-patch --example hotpatching_systems --features hotpatching
//! ```
//!
//! All systems are automatically hot patchable.
//!
//! You can change the text in the `update_text` system, or the color in the
//! `on_click` system, and those changes will be hotpatched into the running
//! application.
//!
//! It's also possible to make any function hot patchable by wrapping it with
//! `bevy::dev_tools::hotpatch::call`.
//!
//! Hot patching is like changing the engine of a car while it's driving - you can
//! modify your game's behavior without stopping or restarting! This supercharges
//! development by letting you tweak values, fix bugs, and experiment in real-time.
//! It's like having a time machine for debugging!

use std::time::Duration;

use bevy::{color::palettes, prelude::*};

fn main() {
    // Channel for communicating with background thread
    let (sender, receiver) = crossbeam_channel::unbounded::<()>();

    // Start a thread to demonstrate hot patching outside systems
    // Could be any background task - asset loading, networking, etc.
    start_thread(receiver);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(TaskSender(sender))
        .add_systems(Startup, setup)
        .add_systems(Update, update_text)  // This system is hot patchable!
        .run();
}

// HOT PATCHABLE SYSTEM - Try changing the text while running!
fn update_text(mut text: Single<&mut Text>) {
    // 🔥 CHANGE THIS TEXT AND SAVE THE FILE!
    // The running app will update immediately without restart
    text.0 = "before".to_string();
    
    // Try changing to:
    // text.0 = "HOT PATCHED!".to_string();
}

// HOT PATCHABLE OBSERVER - Click handler
fn on_click(
    _click: Trigger<Pointer<Click>>,
    mut color: Single<&mut TextColor>,
    task_sender: Res<TaskSender>,
) {
    // 🔥 CHANGE THIS COLOR AND SAVE!
    // Next click will use the new color
    color.0 = palettes::tailwind::RED_600.into();
    
    // Try: GREEN_600, BLUE_600, PURPLE_600, etc.

    // Notify background thread
    let _ = task_sender.0.send(());
}

// Channel for triggering background tasks
#[derive(Resource)]
struct TaskSender(crossbeam_channel::Sender<()>);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Create clickable text UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            children![(
                Text::default(),  // Text content set by update_text
                TextFont {
                    font_size: 100.0,
                    ..default()
                },
            )],
        ))
        .observe(on_click);  // Attach click observer
}

// Background thread demonstrating non-system hot patching
fn start_thread(receiver: crossbeam_channel::Receiver<()>) {
    std::thread::spawn(move || {
        while receiver.recv().is_ok() {
            let start = bevy::platform::time::Instant::now();

            // HOT PATCHABLE CLOSURE - Even non-systems can be patched!
            // 🔥 Try changing the duration while running
            let duration = bevy::app::hotpatch::call(|| Duration::from_secs(2));
            // Try: from_secs(1) or from_millis(500)

            std::thread::sleep(duration);
            info!("done after {:?}", start.elapsed());
        }
    });
}
