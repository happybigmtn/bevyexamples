//! Illustrates how `Timer`s can be used both as resources and components.
//!
//! Timers are like digital stopwatches in your game - they count time and tell you
//! when they're done. You can use them for cooldowns, animations, spawning waves,
//! or any time-based game mechanic. This example shows two patterns: timers as
//! components (attached to entities) and timers as resources (global to the app).

use bevy::{log::info, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Countdown>()    // Global countdown timers
        .add_systems(Startup, setup)
        .add_systems(Update, (
            countdown,               // Updates resource timers
            print_when_completed     // Updates component timers
        ))
        .run();
}

// TIMER AS COMPONENT - Attached to individual entities
// Deref/DerefMut let us use the component like a Timer directly
#[derive(Component, Deref, DerefMut)]
struct PrintOnCompletionTimer(Timer);

// TIMER AS RESOURCE - Global to the entire app
#[derive(Resource)]
struct Countdown {
    percent_trigger: Timer,  // Reports progress every 4 seconds
    main_timer: Timer,       // Main 20-second countdown
}

impl Countdown {
    pub fn new() -> Self {
        Self {
            // Repeating timer - resets after finishing
            // Like a metronome that keeps ticking
            percent_trigger: Timer::from_seconds(4.0, TimerMode::Repeating),
            // Once timer - stops after finishing
            // Like a kitchen timer for your eggs
            main_timer: Timer::from_seconds(20.0, TimerMode::Once),
        }
    }
}

impl Default for Countdown {
    fn default() -> Self {
        Self::new()
    }
}

fn setup(mut commands: Commands) {
    // Spawn an entity with a timer component
    // After 5 seconds, it will print a message
    commands.spawn(PrintOnCompletionTimer(Timer::from_seconds(
        5.0,
        TimerMode::Once,
    )));
}

/// This system ticks the `Timer` on the entity with the `PrintOnCompletionTimer`
/// component using bevy's `Time` resource to get the delta between each update.
fn print_when_completed(time: Res<Time>, mut query: Query<&mut PrintOnCompletionTimer>) {
    for mut timer in &mut query {
        // tick() advances the timer by delta time and returns self
        // just_finished() returns true only on the frame it completes
        if timer.tick(time.delta()).just_finished() {
            info!("Entity timer just finished");
        }
    }
}

/// This system controls ticking the timer within the countdown resource and
/// handling its state.
fn countdown(time: Res<Time>, mut countdown: ResMut<Countdown>) {
    // Always tick the main timer
    countdown.main_timer.tick(time.delta());

    // Tick and check the percent trigger in one expression
    // Chain pattern: tick() returns &mut Timer, then we check just_finished()
    if countdown.percent_trigger.tick(time.delta()).just_finished() {
        if !countdown.main_timer.is_finished() {
            // Calculate and display progress
            // fraction() returns 0.0 to 1.0 based on elapsed/duration
            info!(
                "Timer is {:0.0}% complete!",
                countdown.main_timer.fraction() * 100.0
            );
        } else {
            // Main timer done - stop the progress reports
            countdown.percent_trigger.pause();
            info!("Paused percent trigger timer");
        }
    }
}
