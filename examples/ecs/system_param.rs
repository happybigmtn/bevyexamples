//! This example creates a custom [`SystemParam`] struct that counts the number of players.
//!
//! SystemParam is like creating your own custom tool that bundles together
//! multiple system parameters. Instead of having systems with many parameters,
//! you can group related ones into a single, meaningful unit.
//! It's like the difference between carrying loose tools vs having a toolbox!
//!
//! # ZERO-COST ABSTRACTIONS AND TRAIT SYSTEM
//!
//! ## Zero-Cost Abstractions in ECS Context
//! SystemParam demonstrates Rust's zero-cost abstractions principle:
//! - High-level interface (custom SystemParam types)
//! - Compile-time code generation (derive macros)
//! - Runtime performance identical to manual parameter listing
//! - No virtual dispatch or dynamic allocation overhead
//!
//! At compile time, your custom SystemParam gets "flattened" into the
//! individual parameters it contains. There's no wrapper cost!
//!
//! ## Trait System for Component Behavior
//! SystemParam uses advanced trait techniques:
//!
//! ```rust
//! pub trait SystemParam {
//!     type State: Send + Sync + 'static;  // Associated type for state
//!     type Item<'w, 's>;                  // Generic associated type (GAT)
//!     
//!     fn init_state(world: &mut World, system_meta: &mut SystemMeta) -> Self::State;
//!     unsafe fn get_param<'w, 's>(
//!         state: &'s mut Self::State,
//!         system_meta: &SystemMeta,
//!         world: UnsafeWorldCell<'w>,
//!         change_tick: Tick,
//!     ) -> Self::Item<'w, 's>;
//! }
//! ```
//!
//! Key concepts:
//! - **Associated Types**: Each SystemParam defines its own State and Item types
//! - **Generic Associated Types (GATs)**: Enable lifetime-parameterized associated types
//! - **Unsafe Code**: Low-level world access for maximum performance
//! - **State Management**: Persistent state between system runs
//!
//! ## Generic Programming with Associated Types
//! SystemParam enables powerful generic programming:
//!
//! ```rust
//! // Generic system that works with any SystemParam
//! fn generic_system<P: SystemParam>(param: P) {
//!     // Work with any system parameter type
//! }
//! 
//! // Constraint-based generic programming
//! fn query_system<T: Component>(query: Query<&T>) {
//!     // T can be any component type
//! }
//! ```
//!
//! This enables:
//! - Code reuse across different component types
//! - Type-safe generic algorithms
//! - Compile-time specialization for optimal performance
//!
//! ## Lifetime Management in Query Systems
//! SystemParam carefully manages Rust lifetimes:
//!
//! ### World Lifetime ('w)
//! - References to world data (components, resources)
//! - Must not outlive the world access
//! - Prevents use-after-free of world data
//!
//! ### System State Lifetime ('s)
//! - References to system's internal state
//! - Persists between system runs
//! - Enables caching and optimization
//!
//! ```rust
//! // Lifetime annotations ensure safety
//! struct MyParam<'w, 's> {
//!     query: Query<'w, 's, &'static MyComponent>,  // References world data
//!     state: ResMut<'w, MyResource>,               // Mutable world access
//!     local: Local<'s, MyLocalState>,              // System-local state
//! }
//! ```
//!
//! ## Type-Level Programming with Const Generics
//! Advanced SystemParam patterns use const generics:
//!
//! ```rust
//! // Fixed-size array parameter
//! #[derive(SystemParam)]
//! struct FixedEntitySet<'w, 's, const N: usize> {
//!     entities: Query<'w, 's, Entity>,
//!     buffer: Local<'s, [Option<Entity>; N]>,
//! }
//! 
//! // Usage
//! fn system(entities: FixedEntitySet<128>) {
//!     // Always allocates exactly 128 entity slots
//! }
//! ```
//!
//! Benefits:
//! - Compile-time size guarantees
//! - No runtime allocation
//! - Type-safe array bounds
//! - Optimized machine code generation
//!
//! # DESIGN PATTERNS
//!
//! ## Facade Pattern
//! SystemParam implements the Facade pattern:
//! - Simple interface hiding complex subsystem interactions
//! - Multiple system parameters presented as unified API
//! - Reduced coupling between systems and ECS internals
//!
//! ## Builder Pattern for Complex Parameters
//! Build complex SystemParam types incrementally:
//!
//! ```rust
//! #[derive(SystemParam)]
//! struct GameWorldAccess<'w, 's> {
//!     players: Query<'w, 's, &'static Player>,
//!     enemies: Query<'w, 's, &'static Enemy>,
//!     physics: Res<'w, PhysicsWorld>,
//!     events: EventWriter<'w, GameEvent>,
//!     time: Res<'w, Time>,
//! }
//! 
//! impl<'w, 's> GameWorldAccess<'w, 's> {
//!     fn player_count(&self) -> usize { self.players.iter().count() }
//!     fn enemy_count(&self) -> usize { self.enemies.iter().count() }
//!     fn send_victory_event(&mut self) { self.events.send(GameEvent::Victory); }
//! }
//! ```
//!
//! ## Strategy Pattern with Generic SystemParam
//! Different strategies via different SystemParam implementations:
//!
//! ```rust
//! // Different AI strategies with same interface
//! trait AIStrategy: SystemParam {
//!     fn update_ai(&mut self, entity: Entity);
//! }
//! 
//! #[derive(SystemParam)]
//! struct AggressiveAI<'w, 's> {
//!     targets: Query<'w, 's, &'static Player>,
//!     // ... aggressive AI data
//! }
//! 
//! #[derive(SystemParam)]
//! struct DefensiveAI<'w, 's> {
//!     allies: Query<'w, 's, &'static Ally>,
//!     // ... defensive AI data
//! }
//! ```
//!
//! # REAL-WORLD APPLICATIONS
//!
//! ## Game System Abstraction
//! Create high-level interfaces for game systems:
//!
//! ```rust
//! #[derive(SystemParam)]
//! struct CombatSystem<'w, 's> {
//!     combatants: Query<'w, 's, (&'static Health, &'static mut Damage)>,
//!     effects: EventWriter<'w, EffectEvent>,
//!     audio: Res<'w, AudioManager>,
//!     rng: ResMut<'w, GameRng>,
//! }
//! 
//! impl<'w, 's> CombatSystem<'w, 's> {
//!     fn resolve_attack(&mut self, attacker: Entity, target: Entity) -> AttackResult {
//!         // High-level combat resolution with all needed access
//!     }
//! }
//! ```
//!
//! ## Plugin Interface Standardization
//! Define standard interfaces for plugin systems:
//!
//! ```rust
//! // Standard physics interface
//! #[derive(SystemParam)]
//! struct PhysicsAccess<'w, 's> {
//!     bodies: Query<'w, 's, (&'static Transform, &'static mut Velocity)>,
//!     collisions: EventReader<'w, CollisionEvent>,
//!     physics_world: ResMut<'w, PhysicsWorld>,
//! }
//! 
//! // Any physics plugin can implement this interface
//! ```
//!
//! ## Performance Monitoring and Profiling
//! Bundle profiling and monitoring tools:
//!
//! ```rust
//! #[derive(SystemParam)]
//! struct ProfiledAccess<'w, 's, T: SystemParam> {
//!     inner: T,
//!     profiler: ResMut<'w, SystemProfiler>,
//!     #[system_param(ignore)]
//!     system_name: &'static str,
//! }
//! 
//! impl<'w, 's, T: SystemParam> ProfiledAccess<'w, 's, T> {
//!     fn with_timing<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
//!         let start = Instant::now();
//!         let result = f(&mut self.inner);
//!         self.profiler.record_time(self.system_name, start.elapsed());
//!         result
//!     }
//! }
//! ```
//!
//! # ADVANCED TECHNIQUES
//!
//! ## Conditional SystemParam with PhantomData
//! Create parameters that exist only under certain conditions:
//!
//! ```rust
//! #[derive(SystemParam)]
//! struct ConditionalParam<'w, 's, T: Resource> {
//!     #[system_param(ignore)]
//!     phantom: PhantomData<T>,
//!     query: Query<'w, 's, Entity>,
//! }
//! 
//! impl<'w, 's, T: Resource> ConditionalParam<'w, 's, T> {
//!     fn try_get_resource(&self, world: &World) -> Option<&T> {
//!         world.get_resource::<T>()
//!     }
//! }
//! ```
//!
//! ## SystemParam Composition and Nesting
//! Compose SystemParams hierarchically:
//!
//! ```rust
//! #[derive(SystemParam)]
//! struct InputHandler<'w, 's> {
//!     keyboard: Res<'w, ButtonInput<KeyCode>>,
//!     mouse: Res<'w, ButtonInput<MouseButton>>,
//! }
//! 
//! #[derive(SystemParam)]
//! struct GameplayInput<'w, 's> {
//!     input: InputHandler<'w, 's>,           // Composed SystemParam
//!     camera: Query<'w, 's, &'static Camera>,
//!     ui_events: EventWriter<'w, UIEvent>,
//! }
//! ```
//!
//! ## Macro-Generated SystemParam
//! Generate SystemParam types programmatically:
//!
//! ```rust
//! macro_rules! generate_component_access {
//!     ($name:ident, $($component:ty),*) => {
//!         #[derive(SystemParam)]
//!         struct $name<'w, 's> {
//!             $(
//!                 [<$component:snake>]: Query<'w, 's, &'static $component>,
//!             )*
//!         }
//!     };
//! }
//! 
//! // Generate: PlayerAccess with health, position, inventory queries
//! generate_component_access!(PlayerAccess, Health, Position, Inventory);
//! ```
//!
//! # COMMON PITFALLS
//!
//! ## Lifetime Parameter Confusion
//! Don't confuse the two lifetime parameters:
//!
//! ```rust
//! // WRONG: Using 's for world data
//! struct BadParam<'w, 's> {
//!     query: Query<'s, 's, &'static Component>,  // Should use 'w
//! }
//! 
//! // CORRECT: Proper lifetime usage
//! struct GoodParam<'w, 's> {
//!     query: Query<'w, 's, &'static Component>,  // 'w for world, 's for state
//! }
//! ```
//!
//! ## Over-Abstraction
//! Don't create SystemParam types for everything:
//!
//! ```rust
//! // UNNECESSARY: Simple single-parameter wrapper
//! #[derive(SystemParam)]
//! struct JustAQuery<'w, 's> {
//!     query: Query<'w, 's, &'static Position>,
//! }
//! 
//! // BETTER: Use Query directly for simple cases
//! fn simple_system(query: Query<&Position>) { }
//! ```
//!
//! ## Ignoring Performance Implications
//! SystemParam isn't free for complex types:
//! - Each field adds to system initialization cost
//! - Complex nested queries can impact scheduling
//! - Consider using Events or Resources for heavy data
//!
//! ## Forgetting #[system_param(ignore)]
//! Fields that don't implement SystemParam need the ignore attribute:
//!
//! ```rust
//! #[derive(SystemParam)]
//! struct MyParam<'w, 's> {
//!     query: Query<'w, 's, &'static Component>,
//!     #[system_param(ignore)]
//!     debug_name: &'static str,  // Doesn't implement SystemParam
//! }
//! ```

use bevy::{ecs::system::SystemParam, prelude::*};

fn main() {
    App::new()
        .insert_resource(PlayerCount(0))
        .add_systems(Startup, spawn)
        .add_systems(Update, count_players)
        .run();
}

// Simple marker component for players
#[derive(Component)]
struct Player;

// Resource to store the current player count
#[derive(Resource)]
struct PlayerCount(usize); // Tuple struct with one field

/// The [`SystemParam`] struct can contain any types that can also be included in a
/// system function signature.
///
/// In this example, it includes a query and a mutable resource.
/// The lifetime parameters 'w and 's are required - they represent:
/// - 'w: lifetime of the World access
/// - 's: lifetime of the system's state
/// Don't worry too much about these - they're usually just boilerplate
#[derive(SystemParam)]
struct PlayerCounter<'w, 's> {
    players: Query<'w, 's, &'static Player>, // Query to find all players
    count: ResMut<'w, PlayerCount>,          // Mutable access to the count resource
}

// We can add methods to our SystemParam!
// This encapsulates the counting logic with the data it needs
impl<'w, 's> PlayerCounter<'w, 's> {
    fn count(&mut self) {
        // Count all entities with the Player component
        self.count.0 = self.players.iter().len();
    }
}

/// Spawn some players to count
fn spawn(mut commands: Commands) {
    // Create three player entities
    // They have no data other than the Player marker component
    commands.spawn(Player);
    commands.spawn(Player);
    commands.spawn(Player);
}

/// The [`SystemParam`] can be used directly in a system argument.
/// Instead of: fn count_players(players: Query<&Player>, mut count: ResMut<PlayerCount>)
/// We have: fn count_players(mut counter: PlayerCounter)
/// Much cleaner, especially when you have many related parameters!
fn count_players(mut counter: PlayerCounter) {
    // Use our custom method
    counter.count();

    println!("{} players in the game", counter.count.0);
    
    // The beauty: All the complexity is hidden inside PlayerCounter
    // The system just uses high-level operations
}
