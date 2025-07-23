//! This example illustrates how to react to component and resource changes.
//!
//! Change detection is like having a smart security system that notices when things move!
//! Just as motion detectors only trigger when something actually changes position,
//! Bevy's change detection lets your systems respond only when data is modified.
//! This prevents wasted computation and enables efficient reactive programming patterns.
//! Think of it as the difference between constantly checking your phone vs getting
//! notifications only when you receive a new message!
//!
//! # ECS ARCHITECTURE PATTERNS
//!
//! ## Data-Oriented Design vs Object-Oriented Design
//! Traditional OOP: Objects contain data AND behavior, leading to cache misses
//! when iterating over collections. Each object might be scattered in memory.
//!
//! ECS Design: Components are pure data, systems are pure behavior. This creates
//! "Structure of Arrays" (SoA) memory layouts where all components of the same
//! type are stored contiguously. When a system processes Position components,
//! it reads a tight array of position data - maximum cache efficiency!
//!
//! ## Cache-Friendly Memory Layouts
//! Bevy stores components in "Archetypes" - collections of entities with identical
//! component sets. Entities with (Position, Velocity) are stored separately from
//! entities with (Position, Health). This enables:
//! - Cache-friendly iteration (no pointer chasing)
//! - SIMD optimization opportunities  
//! - Predictable memory access patterns
//!
//! ## System Scheduling and Parallel Execution
//! Change detection enables smart scheduling. Systems only run when their data
//! has actually changed, and Bevy's scheduler can:
//! - Skip unchanged systems entirely
//! - Run independent systems in parallel
//! - Batch similar operations for better cache utilization
//!
//! # PERFORMANCE ANALYSIS
//!
//! ## Change Detection Overhead
//! Each component/resource stores:
//! - Current value
//! - "Changed" tick (when last modified)
//! - "Added" tick (when first added)
//! 
//! Memory overhead: ~16 bytes per component for tick tracking
//! Performance cost: One integer comparison per query
//! 
//! ## Query Optimization Strategies
//! 1. Use `Changed<T>` filters to process only modified entities
//! 2. Combine multiple change filters: `Query<&T, (Changed<A>, Changed<B>)>`
//! 3. Use `set_if_neq()` to avoid false-positive change detection
//! 4. Structure systems to minimize unnecessary change triggers
//!
//! ## Memory Access Patterns
//! - Dense iteration over archetypes (optimal)
//! - Sparse access patterns when mixing many component types (slower)
//! - Change detection enables "reactive" architectures that scale well
//!
//! # DESIGN PATTERNS
//!
//! ## Reactive Programming Pattern
//! Instead of polling for changes every frame:
//! ```rust
//! // Anti-pattern: Check everything every frame
//! fn update_ui(query: Query<&Score>) { /* always runs */ }
//! 
//! // Pattern: React only to changes
//! fn update_ui(query: Query<&Score, Changed<Score>>) { /* runs when Score changes */ }
//! ```
//!
//! ## Event-Driven State Machines
//! Change detection pairs beautifully with state machines:
//! - State changes trigger specific reactions
//! - Only relevant systems activate
//! - Clear separation of concerns
//!
//! ## Incremental Computation Pattern
//! Build systems that only recalculate when inputs change:
//! - Physics: only update transform when position/rotation changes
//! - Rendering: only rebuild meshes when geometry changes
//! - AI: only recalculate paths when environment changes
//!
//! # REAL-WORLD APPLICATIONS
//!
//! ## UI Updates
//! Only regenerate UI when underlying data changes, not every frame.
//! Massively improves performance in complex UIs.
//!
//! ## Physics Optimization
//! Only run expensive collision detection on entities that moved.
//! Static environment objects can be completely skipped.
//!
//! ## Asset Hot-Reloading
//! Detect when assets change on disk and automatically reload only
//! the affected systems and entities.
//!
//! ## Network Synchronization
//! Only send network updates for entities that actually changed,
//! dramatically reducing bandwidth usage.
//!
//! # ADVANCED TECHNIQUES
//!
//! ## Hierarchical Change Propagation
//! Changes to parent entities can trigger child updates efficiently:
//! ```rust
//! // When parent transform changes, mark all children as "derived dirty"
//! fn propagate_transform_changes(
//!     changed_parents: Query<&Children, Changed<Transform>>,
//!     mut child_transforms: Query<&mut Transform>,
//! ) { /* propagation logic */ }
//! ```
//!
//! ## Batched Change Processing
//! Group similar changes together for maximum efficiency:
//! - Collect all transform changes
//! - Process them in a single batch
//! - Update derived systems once per batch
//!
//! ## Change Detection Scoping
//! Use system sets to control when change detection resets:
//! - Process all changes in one frame
//! - Clear "changed" flags before next frame
//! - Ensure no changes are missed or double-processed
//!
//! # COMMON PITFALLS
//!
//! ## False Positive Changes
//! Bevy triggers change detection on mutable access, NOT value changes:
//! ```rust
//! // BAD: Always triggers change detection
//! *position = *position; // Assignment = change detected
//! 
//! // GOOD: Only triggers when value actually differs
//! position.set_if_neq(new_position);
//! ```
//!
//! ## Over-Granular Change Detection
//! Don't split components too finely just for change detection.
//! One component with multiple fields is often better than many
//! single-field components.
//!
//! ## Ignoring Change Detection in Performance-Critical Code
//! Always profile! Change detection can eliminate 90%+ of unnecessary
//! computation in reactive systems. The small overhead pays for itself
//! many times over.

use bevy::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                change_component,     // Randomly modifies components
                change_component_2,   // Another modifier (shows where changes come from)
                change_resource,      // Randomly modifies resources
                change_detection,     // Reacts to any changes
            ),
        )
        .run();
}

// TEST DATA TYPES - Simple wrappers for demonstration
// PartialEq enables smart change detection (only trigger when values actually differ)
#[derive(Component, PartialEq, Debug)]
struct MyComponent(f32);

#[derive(Resource, PartialEq, Debug)]
struct MyResource(f32);

fn setup(mut commands: Commands) {
    // INITIAL SETUP - Create our test subjects
    // Note the first change detection log correctly points to this line because the component is
    // added. Although commands are deferred, they are able to track the original calling location.
    commands.spawn(MyComponent(0.0));     // This will trigger "Added" detection
    commands.insert_resource(MyResource(0.0));  // This will trigger "Added" detection
}

fn change_component(time: Res<Time>, mut query: Query<(Entity, &mut MyComponent)>) {
    for (entity, mut component) in &mut query {
        // RANDOM MODIFICATION - 10% chance per frame
        if rand::thread_rng().gen_bool(0.1) {
            let new_component = MyComponent(time.elapsed_secs().round());
            info!("New value: {new_component:?} {entity}");
            
            // SMART CHANGE DETECTION - Only trigger if value actually changes!
            // Change detection occurs on mutable dereference, and does not consider whether or not
            // a value is actually equal. To avoid triggering change detection when nothing has
            // actually changed, you can use the `set_if_neq` method on any component or resource
            // that implements PartialEq.
            //
            // This is like a smart thermostat that only turns on when temperature actually changes,
            // not just when you touch the controls!
            component.set_if_neq(new_component);
        }
    }
}

/// DEBUGGING MULTIPLE MODIFIERS - Identifying the culprit!
/// This is a duplicate of the `change_component` system, added to show that change tracking can
/// help you find *where* your component is being changed, when there are multiple possible
/// locations.
/// 
/// In real games, you might have physics, AI, input handling, and animation all modifying
/// the same Transform component. Change tracking helps you debug which system caused an issue!
fn change_component_2(time: Res<Time>, mut query: Query<(Entity, &mut MyComponent)>) {
    for (entity, mut component) in &mut query {
        // Another 10% chance - sometimes both systems will try to change the same frame!
        if rand::thread_rng().gen_bool(0.1) {
            let new_component = MyComponent(time.elapsed_secs().round());
            info!("New value: {new_component:?} {entity}");
            component.set_if_neq(new_component);
        }
    }
}

/// RESOURCE CHANGE DETECTION - Same principles, different target
/// Change detection concepts for components apply similarly to resources.
/// Resources are like global variables, but with change tracking superpowers!
fn change_resource(time: Res<Time>, mut my_resource: ResMut<MyResource>) {
    if rand::thread_rng().gen_bool(0.1) {
        let new_resource = MyResource(time.elapsed_secs().round());
        info!("New value: {new_resource:?}");
        // Same smart detection for resources
        my_resource.set_if_neq(new_resource);
    }
}

/// THE DETECTIVE SYSTEM - Investigate what changed and where!
/// Query filters like [`Changed<T>`] and [`Added<T>`] ensure only entities matching these filters
/// will be returned by the query.
///
/// Using the [`Ref<T>`] system param allows you to access change detection information, but does
/// not filter the query.
fn change_detection(
    changed_components: Query<Ref<MyComponent>, Changed<MyComponent>>,  // Only changed entities
    my_resource: Res<MyResource>,
) {
    // COMPONENT INVESTIGATION - Only runs when components actually change!
    for component in &changed_components {
        // THE FORENSICS REPORT - What happened and where?
        // By default, you can only tell that a component was changed.
        //
        // This is useful, but what if you have multiple systems modifying the same component, how
        // will you know which system is causing the component to change?
        warn!(
            "Change detected!\n\t-> value: {:?}\n\t-> added: {}\n\t-> changed: {}\n\t-> changed by: {}",
            component,
            component.is_added(),    // Was this component just created?
            component.is_changed(),  // Was this component modified?
            // DEBUGGING SUPERPOWER - Find the exact line that changed it!
            // If you enable the `track_location` feature, you can unlock the `changed_by()`
            // method. It returns the file and line number that the component or resource was
            // changed in. It's not recommended for released games, but great for debugging!
            component.changed_by()
        );
    }

    // RESOURCE INVESTIGATION - Check if our global state changed
    if my_resource.is_changed() {
        warn!(
            "Change detected!\n\t-> value: {:?}\n\t-> added: {}\n\t-> changed: {}\n\t-> changed by: {}",
            my_resource,
            my_resource.is_added(),
            my_resource.is_changed(),
            my_resource.changed_by() // Like components, requires `track_location` feature.
        );
    }
}
