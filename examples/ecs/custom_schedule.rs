//! # Custom Schedules in Bevy ECS
//!
//! This example demonstrates how to create and integrate custom schedules into Bevy's 
//! application execution flow. Schedules are containers for systems that define when 
//! and in what order those systems run.
//!
//! ## What is a Schedule?
//!
//! A schedule is a collection of systems organized into a dependency graph. Bevy has
//! several built-in schedules:
//! - `PreStartup`: Runs once before `Startup`
//! - `Startup`: Runs once when the app starts
//! - `First`: Runs at the beginning of each frame
//! - `PreUpdate`: Runs before `Update`
//! - `Update`: Main game logic runs here
//! - `PostUpdate`: Runs after `Update`
//! - `Last`: Runs at the end of each frame
//!
//! ## Why Create Custom Schedules?
//!
//! Custom schedules are useful when you need:
//! - Systems that run with specific execution constraints (e.g., single-threaded)
//! - Logical separation of different game phases (e.g., physics, AI, rendering)
//! - Fine-grained control over system execution order
//! - Systems that should run at specific points in the frame
//!
//! This example creates two custom schedules:
//! 1. `SingleThreadedUpdate`: Runs after `Update` with single-threaded execution
//! 2. `CustomStartup`: Runs after `PreStartup` during app initialization

use bevy::{
    // MainScheduleOrder controls the execution order of schedules within the Main schedule
    app::MainScheduleOrder,
    ecs::schedule::{
        // ExecutorKind determines how systems within a schedule are executed
        ExecutorKind,
        // ScheduleLabel is a trait for types that can identify schedules
        ScheduleLabel,
    },
    prelude::*,
};

/// A custom schedule that runs its systems sequentially on a single thread.
/// 
/// The `ScheduleLabel` derive macro generates the necessary trait implementation
/// to use this type as a schedule identifier. The required derives are:
/// - `Debug`: For debugging and error messages
/// - `Hash`: For storing in hash-based collections
/// - `PartialEq, Eq`: For equality comparisons
/// - `Clone`: For copying the label when needed
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct SingleThreadedUpdate;

/// A custom startup schedule that runs after PreStartup but before Startup.
/// This demonstrates how to insert custom initialization logic into the startup flow.
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct CustomStartup;

fn main() {
    // Create a new Bevy application
    // Note: We're not using DefaultPlugins to keep the output clean and focused
    let mut app = App::new();

    // Create a new Schedule with our custom label.
    // A Schedule is a container that holds systems and manages their execution.
    let mut custom_update_schedule = Schedule::new(SingleThreadedUpdate);
    
    // Configure the schedule to use single-threaded execution.
    // By default, Bevy uses ExecutorKind::MultiThreaded which runs systems
    // in parallel when possible. SingleThreaded ensures systems run one at a time
    // in the order they were added, which can be useful for:
    // - Debugging (predictable execution order)
    // - Systems that aren't thread-safe
    // - Reducing overhead for simple systems
    custom_update_schedule.set_executor_kind(ExecutorKind::SingleThreaded);

    // Register the schedule with the app. 
    // Important: This does NOT automatically run the schedule!
    // It only makes the schedule available in the app's Schedules resource.
    // To actually run the schedule, we need to tell Bevy WHEN to run it,
    // which we'll do next by modifying the MainScheduleOrder.
    app.add_schedule(custom_update_schedule);

    // ## Understanding Bevy's Schedule Hierarchy
    //
    // Bevy has a hierarchical schedule structure:
    // 1. The app's runner executes a single "main" schedule (by default called `Main`)
    // 2. The `Main` schedule is a "meta-schedule" that runs other schedules in order
    // 3. Built-in schedules like `Update`, `Startup`, etc. are run BY the `Main` schedule
    //
    // The `MainScheduleOrder` resource controls the execution order of schedules
    // within the `Main` schedule. We can modify it to insert our custom schedules.
    //
    // IMPORTANT: We must modify `MainScheduleOrder` here in `main()`, NOT in a system!
    // Systems that run as part of `Main` cannot modify the schedule they're running in
    // (this would be like trying to change the track while the train is running on it).
    let mut main_schedule_order = app.world_mut().resource_mut::<MainScheduleOrder>();
    
    // Insert our custom schedule to run after the built-in Update schedule.
    // This means every frame will run: First -> Update -> SingleThreadedUpdate -> Last
    main_schedule_order.insert_after(Update, SingleThreadedUpdate);

    // ## Custom Startup Schedules
    //
    // Startup schedules run only once when the app starts, unlike regular schedules
    // that run every frame. Bevy distinguishes between startup and regular schedules.
    
    // Create and register our custom startup schedule
    app.add_schedule(Schedule::new(CustomStartup));

    // Get the MainScheduleOrder resource again to add our startup schedule
    let mut main_schedule_order = app.world_mut().resource_mut::<MainScheduleOrder>();
    
    // Use `insert_startup_after` (not `insert_after`) for startup schedules!
    // This ensures our schedule only runs once during initialization.
    // The startup order will be: PreStartup -> CustomStartup -> Startup
    main_schedule_order.insert_startup_after(PreStartup, CustomStartup);

    // Add systems to various schedules to demonstrate execution order.
    // Each system simply prints its schedule name.
    app
        // Our custom schedules
        .add_systems(SingleThreadedUpdate, single_threaded_update_system)
        .add_systems(CustomStartup, custom_startup_system)
        
        // Built-in schedules (for comparison)
        .add_systems(PreStartup, pre_startup_system)
        .add_systems(Startup, startup_system)
        .add_systems(First, first_system)
        .add_systems(Update, update_system)
        .add_systems(Last, last_system)
        
        // Run the app. You'll see:
        // 1. Startup schedules run once: PreStartup -> CustomStartup -> Startup
        // 2. Then each frame: First -> Update -> SingleThreadedUpdate -> Last
        .run();
}

// === Startup Systems (run once) ===

/// Runs in the PreStartup schedule - the very first schedule to run.
/// Typically used for critical initialization that other systems depend on.
fn pre_startup_system() {
    println!("Pre Startup");
}

/// Runs in our custom startup schedule, after PreStartup but before Startup.
/// This demonstrates how custom schedules can be inserted into the startup flow.
fn custom_startup_system() {
    println!("Custom Startup");
}

/// Runs in the standard Startup schedule.
/// This is where most initialization happens in typical Bevy apps.
fn startup_system() {
    println!("Startup");
}

// === Frame Systems (run every frame) ===

/// Runs at the beginning of every frame in the First schedule.
/// Often used for input handling or frame setup.
fn first_system() {
    println!("First");
}

/// Runs in the main Update schedule where most game logic happens.
/// In a real game, this would update positions, handle collisions, etc.
fn update_system() {
    println!("Update");
}

/// Runs in our custom SingleThreadedUpdate schedule.
/// Because we configured it for single-threaded execution,
/// multiple systems in this schedule would run sequentially,
/// never in parallel.
fn single_threaded_update_system() {
    println!("Single Threaded Update");
}

/// Runs at the end of every frame in the Last schedule.
/// Often used for cleanup or preparing for the next frame.
fn last_system() {
    println!("Last");
}
