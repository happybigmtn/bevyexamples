//! This example illustrates the different ways you can employ component lifecycle hooks.
//!
//! Whenever possible, prefer using Bevy's change detection or Events for reacting to component changes.
//! Events generally offer better performance and more flexible integration into Bevy's systems.
//! Hooks are useful to enforce correctness but have limitations (only one hook per component,
//! less ergonomic than events).
//!
//! Here are some cases where components hooks might be necessary:
//!
//! - Maintaining indexes: If you need to keep custom data structures (like a spatial index) in
//!   sync with the addition/removal of components.
//!
//! - Enforcing structural rules: When you have systems that depend on specific relationships
//!   between components (like hierarchies or parent-child links) and need to maintain correctness.
//!
//! Component hooks are like automatic triggers in a database - they fire when components
//! are added, changed, or removed. Think of them as guardians that ensure your game's
//! data stays consistent. While powerful, they're the "nuclear option" - use Events
//! or change detection for most cases!
//!
//! # ADVANCED ECS CONCEPTS
//!
//! ## Component Storage Strategies
//! Bevy uses two main storage types for components:
//! 
//! ### Table Storage (Default)
//! - Components stored in "Structure of Arrays" (SoA) layout
//! - Excellent cache locality for iteration
//! - Fast bulk operations
//! - Ideal for components that are frequently queried together
//! 
//! ### SparseSet Storage  
//! - Components stored with entity ID as key
//! - Fast insertion/removal for sparse components
//! - Better for components added/removed frequently
//! - Less cache-friendly for iteration
//!
//! ```rust
//! #[derive(Component)]
//! #[component(storage = "SparseSet")]
//! struct RarelyUsedMarker;
//! ```
//!
//! ## Entity Archetypes and Memory Optimization
//! Bevy groups entities with identical component sets into "Archetypes":
//! - Entities with (Position, Velocity) form one archetype
//! - Entities with (Position, Health) form another archetype
//! - Moving components between entities can trigger archetype changes (expensive!)
//!
//! Archetype optimization strategies:
//! - Design stable component sets that rarely change
//! - Use sparse components for temporary markers
//! - Bundle related components to reduce archetype fragmentation
//!
//! ## Hierarchical Relationships and Parent-Child Components
//! Component hooks excel at maintaining hierarchical consistency:
//!
//! ```rust
//! // When parent is despawned, automatically remove children
//! fn on_remove_parent(trigger: Trigger<OnRemove, Parent>, mut commands: Commands) {
//!     let children = /* get children */;
//!     for child in children {
//!         commands.entity(child).despawn_recursive();
//!     }
//! }
//! ```
//!
//! This ensures referential integrity without manual bookkeeping.
//!
//! ## Resource Management and Singleton Patterns
//! Hooks can enforce singleton constraints:
//!
//! ```rust
//! fn enforce_singleton(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
//!     // Only allow one Player entity in the world
//!     let existing_players = /* query for existing players */;
//!     if existing_players.len() > 1 {
//!         commands.entity(trigger.target()).despawn();
//!     }
//! }
//! ```
//!
//! # DESIGN PATTERNS
//!
//! ## Observer Pattern Implementation
//! Hooks implement a perfect Observer pattern:
//! - Subject: Entity with component
//! - Observers: Hook functions
//! - Events: Component lifecycle changes
//! - Automatic subscription/unsubscription
//!
//! ## Dependency Injection Pattern
//! Use hooks to automatically inject dependent components:
//!
//! ```rust
//! fn auto_inject_renderer(trigger: Trigger<OnAdd, Mesh>, mut commands: Commands) {
//!     // When a Mesh is added, automatically add required rendering components
//!     commands.entity(trigger.target()).insert((
//!         MaterialInstance::default(),
//!         VisibilityState::Visible,
//!     ));
//! }
//! ```
//!
//! ## Command Pattern with Deferred Execution
//! Hooks use DeferredWorld - commands are queued and executed later:
//! - Avoids borrow checker conflicts
//! - Ensures consistent world state
//! - Enables complex multi-step operations
//!
//! ## Invariant Maintenance Pattern
//! Hooks excel at maintaining complex invariants:
//! - Network entity synchronization
//! - Physics constraint satisfaction
//! - UI layout consistency
//! - Asset dependency tracking
//!
//! # REAL-WORLD APPLICATIONS
//!
//! ## Spatial Indexing Systems
//! Automatically maintain spatial data structures:
//! - R-trees for 2D/3D spatial queries
//! - Quadtrees for efficient collision detection
//! - Grid-based indexing for chunk-based worlds
//! - Always consistent with entity positions
//!
//! ## Asset Reference Counting
//! Track asset usage automatically:
//! ```rust
//! fn on_add_texture(trigger: Trigger<OnAdd, TextureComponent>, mut assets: ResMut<AssetUsage>) {
//!     assets.increment_ref_count(trigger.event().texture_handle);
//! }
//! 
//! fn on_remove_texture(trigger: Trigger<OnRemove, TextureComponent>, mut assets: ResMut<AssetUsage>) {
//!     assets.decrement_ref_count(trigger.event().texture_handle);
//!     // Auto-unload when ref count reaches zero
//! }
//! ```
//!
//! ## Network State Synchronization
//! Automatically replicate component changes:
//! - Mark entities as "dirty" when components change
//! - Queue network updates for remote clients
//! - Handle ownership transfer between players
//! - Maintain consistent distributed state
//!
//! ## Memory Pool Management
//! Efficiently manage object pools:
//! - Return objects to pool when despawned
//! - Pre-allocate objects when spawned
//! - Manage memory fragmentation
//! - Optimize for specific allocation patterns
//!
//! # ADVANCED TECHNIQUES
//!
//! ## Hook Composition and Chaining
//! Hooks can trigger other hooks, creating dependency chains:
//! ```rust
//! fn on_add_position(trigger: Trigger<OnAdd, Position>) {
//!     // Adding Position might trigger Transform update
//!     // which might trigger Visibility update
//!     // which might trigger Renderer update
//! }
//! ```
//!
//! Design hook chains carefully to avoid infinite loops!
//!
//! ## Conditional Hook Execution
//! Use world state to make hooks conditional:
//! ```rust
//! fn conditional_hook(trigger: Trigger<OnAdd, Component>, world: &mut DeferredWorld) {
//!     if world.resource::<GameState>().is_paused {
//!         return; // Skip hook during pause
//!     }
//!     // Normal hook logic
//! }
//! ```
//!
//! ## Hook Performance Optimization
//! Optimize hook performance for high-frequency components:
//! - Batch operations when possible
//! - Use sparse sets for frequently added/removed components
//! - Minimize work in hot path hooks
//! - Consider using systems with change detection instead
//!
//! ## Meta-Programming with Hooks
//! Generate hooks programmatically:
//! ```rust
//! fn register_sync_hooks<T: Component + Clone + Send + Sync + 'static>(app: &mut App) {
//!     app.world_mut().register_component_hooks::<T>()
//!         .on_add(|trigger, world| {
//!             // Generic synchronization logic
//!         });
//! }
//! ```
//!
//! # COMMON PITFALLS
//!
//! ## Hook Recursion and Infinite Loops
//! Hooks can trigger themselves indirectly:
//! ```rust
//! // DANGEROUS: Can create infinite loop
//! fn bad_hook(trigger: Trigger<OnAdd, A>, mut commands: Commands) {
//!     commands.entity(trigger.target()).insert(B); // Might trigger another hook that adds A
//! }
//! ```
//!
//! Prevention strategies:
//! - Use guards to prevent recursive calls
//! - Design component hierarchies carefully
//! - Test edge cases thoroughly
//!
//! ## Performance Anti-Patterns
//! Don't use hooks for high-frequency operations:
//! ```rust
//! // BAD: Expensive operation in frequently triggered hook
//! fn expensive_hook(trigger: Trigger<OnAdd, Position>) {
//!     expensive_computation(); // This will hurt performance
//! }
//! 
//! // GOOD: Use change detection in systems instead
//! fn expensive_system(query: Query<&Position, Changed<Position>>) {
//!     for position in &query {
//!         expensive_computation(); // Only when actually changed
//!     }
//! }
//! ```
//!
//! ## Resource Contention
//! Hooks access world immutably by default. Mutable access requires DeferredWorld:
//! - Commands are queued, not executed immediately
//! - Can't read results of commands within same hook
//! - Multiple hooks can run simultaneously (if they don't conflict)
//!
//! ## Over-Engineering with Hooks
//! Hooks are powerful but complex. Often simpler solutions work better:
//! - Use events for loose coupling
//! - Use change detection for reactive updates
//! - Use systems for regular processing
//! - Reserve hooks for invariant maintenance only

use bevy::{
    ecs::component::{ComponentHook, HookContext, Mutable, StorageType},
    prelude::*,
};
use std::collections::HashMap;

#[derive(Debug)]
/// Hooks can also be registered during component initialization by
/// using [`Component`] derive macro:
/// ```no_run
/// #[derive(Component)]
/// #[component(on_add = ..., on_insert = ..., on_replace = ..., on_remove = ...)]
/// ```
struct MyComponent(KeyCode);

// Manual Component implementation to show hook registration
impl Component for MyComponent {
    const STORAGE_TYPE: StorageType = StorageType::Table;
    type Mutability = Mutable;

    /// Hooks can also be registered during component initialization by
    /// implementing the associated method
    fn on_add() -> Option<ComponentHook> {
        // We don't have an `on_add` hook here - we'll register it later
        // This shows you CAN define hooks here, but we'll use the runtime API instead
        None
    }
}

// Custom index tracking which keys are pressed by which entities
// This demonstrates maintaining external data structures in sync
#[derive(Resource, Default, Debug, Deref, DerefMut)]
struct MyComponentIndex(HashMap<KeyCode, Entity>);

// Event sent from hooks to demonstrate hook-system communication
#[derive(Event)]
struct MyEvent;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)           // Register hooks
        .add_systems(Update, trigger_hooks)    // Test hook behavior
        .init_resource::<MyComponentIndex>()   // Our custom index
        .add_event::<MyEvent>()               // Hook-triggered events
        .run();
}

fn setup(world: &mut World) {
    // HOOK REGISTRATION - Must be done before any entities use the component!
    // Hooks can't be changed once entities start using the component
    // This ensures consistency and performance
    world
        .register_component_hooks::<MyComponent>()
        // THE 4 LIFECYCLE HOOKS:
        // 1. on_add: Component added to entity that didn't have it
        // 2. on_insert: Component added (regardless of prior state)  
        // 3. on_replace: Component value being replaced
        // 4. on_remove: Component being removed
        //
        // Hook parameters:
        // - DeferredWorld: Like World but some operations are deferred
        // - HookContext: Info about what triggered the hook
        //
        // ON_ADD - First time this component is added to an entity
        .on_add(
            |mut world,
             HookContext {
                 entity,         // Which entity got the component
                 component_id,   // Type info for dynamic components
                 caller,         // Where in code this happened
                 ..
             }| {
                // Access the component data
                let value = world.get::<MyComponent>(entity).unwrap().0;
                println!(
                    "{component_id:?} added to {entity} with value {value:?}{}",
                    caller
                        .map(|location| format!("due to {location}"))
                        .unwrap_or_default()
                );
                
                // Update our index - this keeps it in sync automatically!
                world
                    .resource_mut::<MyComponentIndex>()
                    .insert(value, entity);
                    
                // Hooks can trigger events for systems to handle
                world.send_event(MyEvent);
            },
        )
        // ON_INSERT - Runs for ANY insertion (new or replacement)
        // Runs AFTER on_add (if applicable)
        .on_insert(|world, _| {
            println!("Current Index: {:?}", world.resource::<MyComponentIndex>());
        })
        
        // ON_REPLACE - Component value is being replaced
        // ALSO runs before on_remove when component is removed!
        // Gets the OLD value before it's replaced
        .on_replace(|mut world, context| {
            let value = world.get::<MyComponent>(context.entity).unwrap().0;
            // Clean up old value from index before it's replaced
            world.resource_mut::<MyComponentIndex>().remove(&value);
        })
        
        // ON_REMOVE - Component is being removed entirely
        // Runs AFTER on_replace, component data still accessible
        .on_remove(
            |mut world,
             HookContext {
                 entity,
                 component_id,
                 caller,
                 ..
             }| {
                let value = world.get::<MyComponent>(entity).unwrap().0;
                println!(
                    "{component_id:?} removed from {entity} with value {value:?}{}",
                    caller
                        .map(|location| format!("due to {location}"))
                        .unwrap_or_default()
                );
                // Hooks can use Commands too!
                // Here we despawn the entity when component is removed
                world.commands().entity(entity).despawn();
            },
        );
}

// System to test our hooks - press keys to add components, release to remove
fn trigger_hooks(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    index: Res<MyComponentIndex>,
) {
    // Remove components for released keys
    for (key, entity) in index.iter() {
        if !keys.pressed(*key) {
            // This triggers: on_replace -> on_remove -> entity despawn
            commands.entity(*entity).remove::<MyComponent>();
        }
    }
    
    // Add components for newly pressed keys
    for key in keys.get_just_pressed() {
        // This triggers: on_add -> on_insert
        commands.spawn(MyComponent(*key));
    }
}
