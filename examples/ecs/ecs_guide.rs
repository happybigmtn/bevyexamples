//! This is a guided introduction to Bevy's "Entity Component System" (ECS)
//! All Bevy app logic is built using the ECS pattern, so definitely pay attention!
//!
//! Think of ECS like this: Instead of objects that "have" behaviors (like traditional OOP),
//! we have data (Components) that systems operate on. It's like the difference between
//! saying "a ball bounces" vs "the physics system makes things with velocity and position move."
//!
//! Why ECS?
//! * Data oriented: Functionality is driven by data
//! * Clean Architecture: Loose coupling of functionality / prevents deeply nested inheritance
//! * High Performance: Massively parallel and cache friendly
//!
//! ECS Definitions - The Three Fundamental Particles of Game Development:
//!
//! Component: just a normal Rust data type. generally scoped to a single piece of functionality
//!     Examples: position, velocity, health, color, name
//!     Think of these as "properties" or "attributes" that things can have
//!
//! Entity: a collection of components with a unique id
//!     Examples: Entity1 { Name("Alice"), Position(0, 0) },
//!               Entity2 { Name("Bill"), Position(10, 5) }
//!     Think of these as "things" in your game world - each unique object
//!
//! Resource: a shared global piece of data
//!     Examples: asset storage, events, system state
//!     Think of these as "singletons" or "game-wide state"
//!
//! System: runs logic on entities, components, and resources
//!     Examples: move system, damage system
//!     Think of these as "behaviors" or "rules" that operate on the data
//!
//! The magic happens when you realize: Systems automatically process ALL entities
//! that have the components they care about. It's like having smart queries that
//! automatically find and update everything relevant!
//!
//! # COMPREHENSIVE ECS THEORY AND DATA-ORIENTED DESIGN
//!
//! ## The Paradigm Shift: From Objects to Data
//! Traditional Object-Oriented Programming (OOP) bundles data with behavior:
//! ```cpp
//! class Player {
//!     private:
//!         int health, x, y;
//!     public:
//!         void move() { ... }
//!         void takeDamage() { ... }
//! };
//! ```
//!
//! This creates several problems:
//! - **Cache Misses**: Objects scattered in memory, poor spatial locality
//! - **Inheritance Complexity**: Deep hierarchies become unmaintainable
//! - **Rigid Structure**: Hard to add new behaviors to existing objects
//! - **Parallel Processing**: Methods can't easily run concurrently
//!
//! ECS separates data from behavior completely:
//! ```rust
//! #[derive(Component)]
//! struct Health(i32);
//! 
//! #[derive(Component)]
//! struct Position { x: f32, y: f32 }
//! 
//! // Pure data - no methods, no inheritance
//! 
//! fn movement_system(mut query: Query<&mut Position, With<Velocity>>) {
//!     // Pure behavior - operates on all matching entities
//! }
//! ```
//!
//! ## Structure of Arrays (SoA) vs Array of Structures (AoS)
//! 
//! ### Array of Structures (Traditional OOP)
//! ```
//! Player[0]: {health: 100, x: 5.0, y: 3.0, inventory: [...]}
//! Player[1]: {health: 75,  x: 2.0, y: 8.0, inventory: [...]}
//! Player[2]: {health: 50,  x: 9.0, y: 1.0, inventory: [...]}
//! ```
//! Memory access pattern when updating positions:
//! - Load entire Player[0] (health, x, y, inventory) - 64+ bytes
//! - Use only x, y (8 bytes) - 87% waste!
//! - Cache miss for each player
//!
//! ### Structure of Arrays (ECS)
//! ```
//! Health:   [100, 75, 50, ...]
//! Position: [(5.0,3.0), (2.0,8.0), (9.0,1.0), ...]
//! Inventory: [[...], [...], [...], ...]
//! ```
//! Memory access pattern when updating positions:
//! - Load contiguous Position array
//! - Process 8+ positions per cache line
//! - 90%+ cache hit rate
//! - Perfect for SIMD optimization
//!
//! ## Cache-Friendly Memory Layouts in Practice
//! When Bevy processes a movement system:
//! 1. **Archetype Identification**: Find all entities with (Position, Velocity)
//! 2. **Dense Iteration**: Process packed arrays of position/velocity data
//! 3. **Cache Optimization**: 64-byte cache lines hold multiple components
//! 4. **SIMD Potential**: Vectorized operations on multiple entities
//!
//! Real-world performance difference:
//! - OOP approach: ~100 MB/s memory bandwidth utilization
//! - ECS approach: ~8000 MB/s memory bandwidth utilization
//! - 80x performance improvement on computation-heavy systems!
//!
//! ## Component Composition Over Inheritance
//! Traditional inheritance creates rigid hierarchies:
//! ```
//! GameObject
//! ├── Character
//! │   ├── Player
//! │   └── NPC
//! │       ├── Merchant
//! │       └── Enemy
//! └── Item
//!     ├── Weapon
//!     └── Consumable
//! ```
//!
//! Problems:
//! - What if an NPC becomes a player? (possession mechanic)
//! - What if a weapon has AI? (intelligent sword)
//! - What if an item can move? (floating pickup)
//!
//! ECS composition is flexible:
//! ```rust
//! // Basic entity types via component composition
//! commands.spawn((Name("Player"), Health(100), Position::default(), InputController));
//! commands.spawn((Name("Enemy"), Health(50), Position::default(), AIController));
//! commands.spawn((Name("Smart Sword"), Weapon::default(), AIController, Position::default()));
//! commands.spawn((Name("Floating Gem"), Item::default(), Position::default(), Velocity::default()));
//! ```
//!
//! Benefits:
//! - **Flexible**: Any entity can have any combination of components
//! - **Reusable**: Systems work on any entity with required components
//! - **Extensible**: Add new behaviors by adding components
//! - **Testable**: Each system has clear input/output dependencies
//!
//! ## Entity Archetypes and Memory Optimization
//! Bevy groups entities with identical component sets into "Archetypes":
//!
//! ```
//! Archetype 1: [Position, Velocity, Health]
//! Entities: [Player1, Enemy1, Enemy2, Projectile1]
//! Memory Layout:
//!   Position: [pos1, pos2, pos3, pos4]
//!   Velocity: [vel1, vel2, vel3, vel4]  
//!   Health:   [hp1,  hp2,  hp3,  hp4]
//! 
//! Archetype 2: [Position, Velocity, Sprite]
//! Entities: [Decoration1, Particle1, Particle2]
//! Memory Layout:
//!   Position: [pos5, pos6, pos7]
//!   Velocity: [vel5, vel6, vel7]
//!   Sprite:   [spr1, spr2, spr3]
//! ```
//!
//! ### Archetype Operations
//! - **Moving Between Archetypes**: Expensive! Requires copying all component data
//! - **Dense Iteration**: Systems process each archetype separately
//! - **Cache Efficiency**: Components of same type stored contiguously
//!
//! ### Optimization Strategies
//! 1. **Stable Component Sets**: Design entities with components that rarely change
//! 2. **Marker Components**: Use zero-sized types for state flags
//! 3. **Component Bundling**: Group related components added/removed together
//! 4. **Sparse Components**: Use for rarely-used or temporary data
//!
//! ## Query Optimization and Iteration Patterns
//! 
//! ### Basic Query Patterns
//! ```rust
//! // Immutable access - highest performance
//! fn render_system(query: Query<(&Position, &Sprite)>) { }
//! 
//! // Mutable access - triggers change detection
//! fn movement_system(mut query: Query<&mut Position>) { }
//! 
//! // Filtered queries - only matching entities
//! fn player_system(query: Query<&mut Health, With<Player>>) { }
//! 
//! // Multiple filters - precise entity selection  
//! fn ai_system(query: Query<&mut Position, (With<AI>, Without<Player>)>) { }
//! ```
//!
//! ### Advanced Query Optimization
//! ```rust
//! // Avoid expensive filters when possible
//! // BAD: String comparison every iteration
//! fn bad_system(query: Query<(&Name, &mut Health)>) {
//!     for (name, mut health) in &mut query {
//!         if name.0 == "Boss" {
//!             health.0 += 10;
//!         }
//!     }
//! }
//! 
//! // GOOD: Separate archetype with marker component
//! #[derive(Component)]
//! struct Boss;
//! 
//! fn good_system(mut query: Query<&mut Health, With<Boss>>) {
//!     for mut health in &mut query {
//!         health.0 += 10; // Only processes boss entities
//!     }
//! }
//! ```
//!
//! ## System Scheduling and Parallel Execution
//! Bevy's scheduler analyzes system dependencies automatically:
//!
//! ### Dependency Analysis
//! ```rust
//! fn system_a(mut query: Query<&mut Position>) { }    // Writes Position
//! fn system_b(query: Query<&Position>) { }            // Reads Position  
//! fn system_c(mut query: Query<&mut Velocity>) { }    // Writes Velocity
//! fn system_d(query: Query<(&Position, &Velocity)>) { } // Reads both
//! ```
//!
//! Automatic scheduling:
//! ```
//! Frame 1: [system_a] -> [system_b, system_d]
//!          [system_c] -> [system_d]
//! 
//! // system_a and system_c can run in parallel
//! // system_b and system_d wait for their dependencies
//! // system_d waits for both system_a and system_c
//! ```
//!
//! ### Performance Implications
//! - **Parallel Systems**: Systems with non-overlapping component access run simultaneously
//! - **Batching**: Similar systems grouped for better cache utilization
//! - **Pipelining**: CPU works on frame N while GPU renders frame N-1
//! - **Work Stealing**: Idle threads help with remaining systems
//!
//! ## Advanced ECS Patterns
//!
//! ### Sparse vs Dense Components
//! ```rust
//! // Dense: Frequently accessed, stored in archetype tables
//! #[derive(Component)]
//! struct Position(Vec3);
//! 
//! // Sparse: Rarely accessed, stored in separate hash map
//! #[derive(Component)]
//! #[component(storage = "SparseSet")]
//! struct DebugMarker;
//! ```
//!
//! ### Entity Relationships
//! ```rust
//! // Parent-child hierarchies
//! #[derive(Component)]
//! struct Parent(Entity);
//! 
//! #[derive(Component)]
//! struct Children(Vec<Entity>);
//! 
//! // Many-to-many relationships
//! #[derive(Component)]
//! struct Members(Vec<Entity>); // Guild members
//! 
//! #[derive(Component)]  
//! struct MemberOf(Entity);     // Member of guild
//! ```
//!
//! ### Component States and State Machines
//! ```rust
//! #[derive(Component)]
//! enum AIState {
//!     Idle,
//!     Patrolling { path: Vec<Vec3> },
//!     Chasing { target: Entity },
//!     Attacking { target: Entity, cooldown: Timer },
//! }
//! 
//! fn ai_system(mut query: Query<(&mut Position, &mut AIState)>) {
//!     for (mut pos, mut state) in &mut query {
//!         match &mut *state {
//!             AIState::Idle => { /* idle behavior */ }
//!             AIState::Patrolling { path } => { /* follow path */ }
//!             AIState::Chasing { target } => { /* chase target */ }
//!             AIState::Attacking { target, cooldown } => { /* attack target */ }
//!         }
//!     }
//! }
//! ```
//!
//! Now that you know a little bit about ECS, lets look at some Bevy code!
//! We will now make a simple "game" to illustrate what Bevy's ECS looks like in practice.

use bevy::{
    app::{AppExit, ScheduleRunnerPlugin},
    prelude::*,
};
use core::time::Duration;
use rand::random;
use std::fmt;

// COMPONENTS: Pieces of functionality we add to entities. These are just normal Rust data types
// The #[derive(Component)] macro is what makes a type usable as a component.
// Under the hood, this implements the Component trait for your type.

// Our game will have a number of "players". Each player has a name that identifies them
// Notice: Components are just data! No methods, no behaviors - pure information.
#[derive(Component)]
struct Player {
    name: String,
}

// Each player also has a score. This component holds on to that score
// By separating Player and Score, we achieve maximum flexibility:
// - We could have scoring entities that aren't players
// - We could have players without scores
// - Systems can query for just what they need
#[derive(Component)]
struct Score {
    value: usize,
}

// Enums can also be used as components.
// This component tracks how many consecutive rounds a player has/hasn't scored in.
// This shows that components can be as complex as you need - not just simple structs!
#[derive(Component)]
enum PlayerStreak {
    Hot(usize),  // On a roll! The number is how many rounds
    None,        // Just started or just broke a streak
    Cold(usize), // Having bad luck - number of rounds without scoring
}

impl fmt::Display for PlayerStreak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerStreak::Hot(n) => write!(f, "{n} round hot streak"),
            PlayerStreak::None => write!(f, "0 round streak"),
            PlayerStreak::Cold(n) => write!(f, "{n} round cold streak"),
        }
    }
}

// RESOURCES: "Global" state accessible by systems. These are also just normal Rust data types!
// Resources are like components, but instead of being attached to entities, there's only ONE
// instance that's shared across the whole app. Perfect for game-wide settings and state.

// This resource holds information about the game:
// #[derive(Default)] lets Bevy create this with default values (0, 0, None)
#[derive(Resource, Default)]
struct GameState {
    current_round: usize,
    total_players: usize,
    winning_player: Option<String>, // Option = might have a winner, might not yet
}

// This resource provides rules for our "game".
// No Default here - we'll set these explicitly because game rules shouldn't be zero!
#[derive(Resource)]
struct GameRules {
    winning_score: usize,
    max_rounds: usize,
    max_players: usize,
}

// SYSTEMS: Logic that runs on entities, components, and resources. These generally run once each
// time the app updates.
//
// Systems are where the magic happens! They're just functions, but Bevy calls them
// with exactly the data they request. It's like having a smart assistant that knows
// what tools you need before you ask.

// This is the simplest type of system. It just prints "This game is fun!" on each run:
// No parameters = no data needed. Still runs every frame though!
fn print_message_system() {
    println!("This game is fun!");
}

// Systems can also read and modify resources. This system starts a new "round" on each update:
// The function parameters ARE the system's interface to the game world!
fn new_round_system(
    // Res<T> = "I want read-only access to resource T"
    game_rules: Res<GameRules>,
    // ResMut<T> = "I want read-write access to resource T"
    mut game_state: ResMut<GameState>,
) {
    // We can read game_rules...
    // We can read AND write game_state...
    game_state.current_round += 1;
    println!(
        "Begin round {} of {}",
        game_state.current_round, game_rules.max_rounds
    );
}

// This system updates the score for each entity with the `Player`, `Score` and `PlayerStreak` components.
// HERE'S THE MAGIC: Query<T> finds ALL entities that have the components you specify!
fn score_system(
    // Query<(ComponentA, ComponentB, ...)> = "Find all entities with ALL these components"
    // The &mut on components means we want write access
    mut query: Query<(&Player, &mut Score, &mut PlayerStreak)>,
) {
    // The query iterator gives us the components for each matching entity
    // If we had 100 players, this loop runs 100 times automatically!
    for (player, mut score, mut streak) in &mut query {
        // Simulate scoring with randomness (not a very fair game!)
        let scored_a_point = random::<bool>();
        if scored_a_point {
            // Accessing components immutably is done via a regular reference - `player`
            // has type `&Player`. We can read but not modify.
            //
            // Accessing components mutably is performed via type `Mut<T>` - `score`
            // has type `Mut<Score>` and `streak` has type `Mut<PlayerStreak>`.
            // Mut<T> is a smart pointer that tracks if you actually changed the data!
            //
            // `Mut<T>` implements `Deref<T>`, so struct fields can be updated using
            // standard field update syntax ...
            score.value += 1;
            // ... and matching against enums requires dereferencing them
            // Pattern matching to update our streak state machine
            *streak = match *streak {
                PlayerStreak::Hot(n) => PlayerStreak::Hot(n + 1),     // Keep the streak going!
                PlayerStreak::Cold(_) | PlayerStreak::None => PlayerStreak::Hot(1), // Start a new hot streak
            };
            println!(
                "{} scored a point! Their score is: {} ({})",
                player.name, score.value, *streak
            );
        } else {
            // Same pattern matching for failures
            *streak = match *streak {
                PlayerStreak::Hot(_) | PlayerStreak::None => PlayerStreak::Cold(1), // Start a cold streak
                PlayerStreak::Cold(n) => PlayerStreak::Cold(n + 1),   // Continue the cold streak
            };

            println!(
                "{} did not score a point! Their score is: {} ({})",
                player.name, score.value, *streak
            );
        }
    }

    // this game isn't very fun is it :)
    // But it perfectly demonstrates how systems process sets of entities!
}

// This system runs on all entities with the `Player` and `Score` components, but it also
// accesses the `GameRules` resource to determine if a player has won.
fn score_check_system(
    game_rules: Res<GameRules>,
    mut game_state: ResMut<GameState>,
    query: Query<(&Player, &Score)>,
) {
    for (player, score) in &query {
        if score.value == game_rules.winning_score {
            game_state.winning_player = Some(player.name.clone());
        }
    }
}

// This system ends the game if we meet the right conditions. This fires an AppExit event, which
// tells our App to quit. Check out the "event.rs" example if you want to learn more about using
// events.
fn game_over_system(
    game_rules: Res<GameRules>,
    game_state: Res<GameState>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if let Some(ref player) = game_state.winning_player {
        println!("{player} won the game!");
        app_exit_events.write(AppExit::Success);
    } else if game_state.current_round == game_rules.max_rounds {
        println!("Ran out of rounds. Nobody wins!");
        app_exit_events.write(AppExit::Success);
    }
}

// This is a "startup" system that runs exactly once when the app starts up. Startup systems are
// generally used to create the initial "state" of our game. The only thing that distinguishes a
// "startup" system from a "normal" system is how it is registered:
//      Startup: app.add_systems(Startup, startup_system)
//      Normal:  app.add_systems(Update, normal_system)
fn startup_system(mut commands: Commands, mut game_state: ResMut<GameState>) {
    // Create our game rules resource
    commands.insert_resource(GameRules {
        max_rounds: 10,
        winning_score: 4,
        max_players: 4,
    });

    // Add some players to our world. Players start with a score of 0 ... we want our game to be
    // fair!
    commands.spawn_batch(vec![
        (
            Player {
                name: "Alice".to_string(),
            },
            Score { value: 0 },
            PlayerStreak::None,
        ),
        (
            Player {
                name: "Bob".to_string(),
            },
            Score { value: 0 },
            PlayerStreak::None,
        ),
    ]);

    // set the total players to "2"
    game_state.total_players = 2;
}

// This system uses a command buffer to (potentially) add a new player to our game on each
// iteration. Normal systems cannot safely access the World instance directly because they run in
// parallel. Our World contains all of our components, so mutating arbitrary parts of it in parallel
// is not thread safe. Command buffers give us the ability to queue up changes to our World without
// directly accessing it
//
// Think of Commands like a "to-do list" for the engine. You write down what you want
// to happen, and Bevy does it safely between system runs.
fn new_player_system(
    mut commands: Commands,
    game_rules: Res<GameRules>,
    mut game_state: ResMut<GameState>,
) {
    // Randomly add a new player
    let add_new_player = random::<bool>();
    if add_new_player && game_state.total_players < game_rules.max_players {
        game_state.total_players += 1;
        // commands.spawn() creates a new entity with the components you specify
        // This is THE way to create new things in your game world!
        commands.spawn((
            Player {
                name: format!("Player {}", game_state.total_players),
            },
            Score { value: 0 },
            PlayerStreak::None,
        ));

        println!("Player {} joined the game!", game_state.total_players);
    }
}

// If you really need full, immediate read/write access to the world or resources, you can use an
// "exclusive system".
// WARNING: These will block all parallel execution of other systems until they finish, so they
// should generally be avoided if you want to maximize parallelism.
//
// Think of exclusive systems like having exclusive access to a laboratory -
// nobody else can work while you're using it. Powerful but inefficient!
fn exclusive_player_system(world: &mut World) {
    // With exclusive access, we manually get resources and spawn entities
    // It's more cumbersome than normal systems - this is intentional!
    // Bevy wants to encourage you to use the parallel-friendly approach.
    
    // this does the same thing as "new_player_system"
    let total_players = world.resource_mut::<GameState>().total_players;
    let should_add_player = {
        let game_rules = world.resource::<GameRules>();
        let add_new_player = random::<bool>();
        add_new_player && total_players < game_rules.max_players
    };
    // Randomly add a new player
    if should_add_player {
        println!("Player {} has joined the game!", total_players + 1);
        world.spawn((
            Player {
                name: format!("Player {}", total_players + 1),
            },
            Score { value: 0 },
            PlayerStreak::None,
        ));

        let mut game_state = world.resource_mut::<GameState>();
        game_state.total_players += 1;
    }
}

// Sometimes systems need to be stateful. Bevy's ECS provides the `Local` system parameter
// for this case. A `Local<T>` refers to a value of type `T` that is owned by the system.
// This value is automatically initialized using `T`'s `FromWorld`* implementation upon the system's initialization.
// In this system's `Local` (`counter`), `T` is `u32`.
// Therefore, on the first turn, `counter` has a value of 0.
//
// Think of Local<T> like a system having its own personal notebook - 
// it remembers things between runs, but only this system can see it.
//
// *: `FromWorld` is a trait which creates a value using the contents of the `World`.
// For any type which is `Default`, like `u32` in this example, `FromWorld` creates the default value.
fn print_at_end_round(mut counter: Local<u32>) {
    // This counter persists between system runs!
    // First run: 0 -> 1
    // Second run: 1 -> 2
    // And so on...
    *counter += 1;
    println!("In set 'Last' for the {}th time", *counter);
    // Print an empty line between rounds
    println!();
}

/// A group of related system sets, used for controlling the order of systems. Systems can be
/// added to any number of sets.
///
/// SystemSets are like labels or categories for organizing your systems.
/// Instead of specifying "A before B, B before C" for every system,
/// you can say "all systems in SetA run before SetB"
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum MySystems {
    BeforeRound,  // Setup phase
    Round,        // Main gameplay  
    AfterRound,   // Cleanup/checking phase
}

// Our Bevy app's entry point
fn main() {
    // Bevy apps are created using the builder pattern. We use the builder to add systems,
    // resources, and plugins to our app
    // 
    // The builder pattern is like assembling a machine piece by piece -
    // each method call adds another component to your app.
    App::new()
        // Resources that implement the Default or FromWorld trait can be added like this:
        .init_resource::<GameState>()
        // Plugins are just a grouped set of app builder calls (just like we're doing here).
        // We could easily turn our game into a plugin, but you can check out the plugin example for
        // that :) The plugin below runs our app's "system schedule" once every 5 seconds.
        // This is unusual - most games run continuously. We're doing it for demonstration.
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs(5)))
        // `Startup` systems run exactly once BEFORE all other systems. These are generally used for
        // app initialization code (ex: adding entities and resources)
        .add_systems(Startup, startup_system)
        // `Update` systems run once every update. These are generally used for "real-time app logic"
        .add_systems(Update, print_message_system)
        // SYSTEM EXECUTION ORDER
        //
        // This is crucial to understand! By default, Bevy runs systems in parallel
        // whenever possible. It's like having multiple scientists working on different
        // experiments at the same time - much more efficient!
        //
        // Each system belongs to a `Schedule`, which controls the execution strategy and broad order
        // of the systems within each tick. The `Startup` schedule holds
        // startup systems, which are run a single time before `Update` runs. `Update` runs once per app update,
        // which is generally one "frame" or one "tick".
        //
        // By default, all systems in a `Schedule` run in parallel, except when they require mutable access to a
        // piece of data. This is efficient, but sometimes order matters.
        // For example, we want our "game over" system to execute after all other systems to ensure
        // we don't accidentally run the game for an extra round.
        //
        // Bevy automatically figures out when systems CAN'T run in parallel
        // (e.g., two systems both need mut access to the same resource).
        // But sometimes you need explicit ordering for logical reasons.
        //
        // You can force an explicit ordering between systems using the `.before` or `.after` methods.
        // Systems will not be scheduled until all of the systems that they have an "ordering dependency" on have
        // completed.
        // There are other schedules, such as `Last` which runs at the very end of each run.
        .add_systems(Last, print_at_end_round)
        // We can also create new system sets, and order them relative to other system sets.
        // Here is what our games execution order will look like:
        // "before_round": new_player_system, new_round_system
        // "round": print_message_system, score_system
        // "after_round": score_check_system, game_over_system
        .configure_sets(
            Update,
            // chain() will ensure sets run in the order they are listed
            // It's like saying "BeforeRound THEN Round THEN AfterRound"
            (
                MySystems::BeforeRound,
                MySystems::Round,
                MySystems::AfterRound,
            )
                .chain(),
        )
        // The add_systems function is powerful. You can define complex system configurations with ease!
        // This is where we actually assign systems to our sets
        .add_systems(
            Update,
            (
                // These `BeforeRound` systems will run before `Round` systems, thanks to the chained set configuration
                (
                    // You can also chain systems! new_round_system will run first, followed by new_player_system
                    // chain() means "strict ordering" - the second waits for the first
                    (new_round_system, new_player_system).chain(),
                    // This runs in parallel with the chain above (both are in BeforeRound)
                    exclusive_player_system,
                )
                    // All of the systems in the tuple above will be added to this set
                    .in_set(MySystems::BeforeRound),
                    
                // This `Round` system will run after the `BeforeRound` systems thanks to the chained set configuration
                score_system.in_set(MySystems::Round),
                
                // These `AfterRound` systems will run after the `Round` systems thanks to the chained set configuration
                (
                    score_check_system,
                    // In addition to chain(), you can also use `before(system)` and `after(system)`. This also works
                    // with sets! Here we ensure game_over runs after score_check within the same set
                    game_over_system.after(score_check_system),
                )
                    .in_set(MySystems::AfterRound),
            ),
        )
        // This call to run() starts the app we just built!
        // From here, the engine takes over and runs your systems according to the rules you defined
        .run();
}
