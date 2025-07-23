//! This example shows how to send, mutate, and receive, events. As well as showing
//! how to you might control system ordering so that events are processed in a specific order.
//! It does this by simulating a damage over time effect that you might find in a game.
//!
//! Events are like messages passed between systems - think of them as a postal service
//! for your game. One system sends a letter (event), others receive and act on it.
//! This decouples systems beautifully - the damage dealer doesn't need to know about
//! sound effects or particle systems!

use bevy::prelude::*;

// In order to send or receive events first you must define them
// Events are just structs with the #[derive(Event)] macro
// Think of each event type as a different kind of message

// This event should be sent when something attempts to deal damage to another entity.
#[derive(Event, Debug)]
struct DealDamage {
    pub amount: i32, // How much damage to attempt
}

// This event should be sent when an entity receives damage.
// Default trait means we can create it without specifying fields
#[derive(Event, Debug, Default)]
struct DamageReceived; // Empty event - just signals that damage happened

// This event should be sent when an entity blocks damage with armor.
#[derive(Event, Debug, Default)]
struct ArmorBlockedDamage; // Another signal event

// This resource represents a timer used to determine when to deal damage
// By default it repeats once per second
// Deref/DerefMut let us use this like it's just a Timer directly
#[derive(Resource, Deref, DerefMut)]
struct DamageTimer(pub Timer); // Newtype pattern - wrapping Timer to give it meaning

impl Default for DamageTimer {
    fn default() -> Self {
        // Timer that fires every second, forever
        // Like a poison effect that ticks every second
        DamageTimer(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

// Next we define systems that send, mutate, and receive events
// This system reads 'DamageTimer', updates it, then sends a 'DealDamage' event
// if the timer has finished.
//
// EventWriter<T> is your "outbox" for sending events of type T
fn deal_damage_over_time(
    time: Res<Time>,
    mut state: ResMut<DamageTimer>,
    mut events: EventWriter<DealDamage>, // Our event "pen" for writing damage events
) {
    // Tick the timer forward by the time that passed this frame
    // is_finished() returns true when the timer completes (every 1 second)
    if state.tick(time.delta()).is_finished() {
        // Send an event! This goes into a queue that other systems can read
        // It's like dropping a letter in the mailbox
        events.write(DealDamage { amount: 10 });
    }
}

// This system mutates the 'DealDamage' events to apply some armor value
// It also sends an 'ArmorBlockedDamage' event if the value of 'DealDamage' is zero
//
// EventMutator<T> is special - it can MODIFY events before others read them!
// This is like being able to edit letters while they're in transit
fn apply_armor_to_damage(
    mut dmg_events: EventMutator<DealDamage>, // Can read AND modify damage events
    mut armor_events: EventWriter<ArmorBlockedDamage>, // Can send armor block events
) {
    // Process each damage event that hasn't been handled yet
    for event in dmg_events.read() {
        // Armor reduces damage by 1 (simple armor system)
        event.amount -= 1;
        if event.amount <= 0 {
            // Damage was completely blocked! Send a notification event
            // This is an empty event - it just signals that something happened
            armor_events.write(ArmorBlockedDamage);
        }
    }
}

// This system reads 'DealDamage' events and sends 'DamageReceived' if the amount is non-zero
//
// EventReader<T> is your "inbox" for receiving events of type T
// The reader tracks which events you've already seen, so you only process new ones
fn apply_damage_to_health(
    mut dmg_events: EventReader<DealDamage>,     // Read damage events
    mut rcvd_events: EventWriter<DamageReceived>, // Send "we got hit" events
) {
    // Only iterate over NEW events since last time this system ran
    // This is the key to event systems - automatic tracking of what's been processed
    for event in dmg_events.read() {
        info!("Applying {} damage", event.amount);
        if event.amount > 0 {
            // We took damage! Notify other systems (sound, visuals, etc.)
            // write_default() works because DamageReceived implements Default
            rcvd_events.write_default();
        }
    }
}

// Finally these two systems read 'DamageReceived' events.
//
// This demonstrates a KEY PRINCIPLE: Multiple systems can listen to the same events!
// It's like a radio broadcast - any number of receivers can tune in.
//
// The first system will play a sound.
fn play_damage_received_sound(mut dmg_events: EventReader<DamageReceived>) {
    // We don't care about the event data (it's empty anyway)
    // We just care that the event happened
    for _ in dmg_events.read() {
        info!("Playing a sound.");
    }
}

// The second system will spawn a particle effect.
// Note that both systems receive the SAME 'DamageReceived' events. 
// This is the power of events - complete decoupling!
// The damage system doesn't need to know about sounds OR particles
fn play_damage_received_particle_effect(mut dmg_events: EventReader<DamageReceived>) {
    for _ in dmg_events.read() {
        info!("Playing particle effect.");
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Events must be registered before use - this sets up the event queues
        // Think of it as installing mailboxes for each event type
        .add_event::<DealDamage>()
        .add_event::<ArmorBlockedDamage>()
        .add_event::<DamageReceived>()
        .init_resource::<DamageTimer>()
        // As always we must add our systems to the apps schedule.
        // Here we add our systems to the schedule using 'chain()' so that they run in order
        // This ensures that 'apply_armor_to_damage' runs before 'apply_damage_to_health'
        // 
        // ORDER MATTERS HERE! We want:
        // 1. Deal damage (send event)
        // 2. Apply armor (modify event)  
        // 3. Apply to health (read modified event)
        .add_systems(
            Update,
            (
                deal_damage_over_time,
                apply_armor_to_damage,
                apply_damage_to_health,
            )
                .chain(), // Forces sequential execution
        )
        // These two systems are not guaranteed to run in order, nor are they guaranteed to run
        // after the above chain. They may even run in parallel with each other.
        // This means they may have a one frame delay in processing events compared to the above chain
        //
        // This is actually GOOD for performance - sound and particles can run in parallel!
        // The tradeoff: they might process events one frame late. For cosmetic effects, that's fine.
        .add_systems(
            Update,
            (
                play_damage_received_sound,
                play_damage_received_particle_effect,
            ),
        )
        .run();
}
