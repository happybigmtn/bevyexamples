//! Demonstrates how to observe life-cycle triggers as well as define custom ones.
//!
//! Observers are like security cameras watching for specific events - when they see
//! their target event happen, they spring into action! This example creates a minefield
//! where clicking triggers chain reactions. Each mine watches for its own explosion event,
//! and when it explodes, it triggers nearby mines - like dominoes falling!

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<SpatialIndex>()
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_shapes, handle_click))
        
        // GLOBAL OBSERVER - Watches for ExplodeMines events
        // This is like a central command center that coordinates explosions
        .add_observer(
            |trigger: Trigger<ExplodeMines>,
             mines: Query<&Mine>,
             index: Res<SpatialIndex>,
             mut commands: Commands| {
                // Get the explosion event data
                let event = trigger.event();
                
                // Use spatial index to find nearby mines efficiently
                // Like checking a grid map instead of measuring every distance
                for e in index.get_nearby(event.pos) {
                    let mine = mines.get(e).unwrap();
                    // Check if mine is within explosion radius
                    if mine.pos.distance(event.pos) < mine.size + event.radius {
                        // Trigger individual mine explosion!
                        // This creates a chain reaction
                        commands.trigger_targets(Explode, e);
                    }
                }
            },
        )
        
        // LIFECYCLE OBSERVERS - Automatically track component changes
        // OnAdd: When Mine component is added, update spatial index
        .add_observer(on_add_mine)
        // OnRemove: When Mine component is removed, clean up index
        .add_observer(on_remove_mine)
        .run();
}

#[derive(Component)]
struct Mine {
    pos: Vec2,    // Position in 2D space
    size: f32,    // Explosion radius when triggered
}

impl Mine {
    fn random(rand: &mut ChaCha8Rng) -> Self {
        Mine {
            // Random position within screen bounds
            pos: Vec2::new(
                (rand.r#gen::<f32>() - 0.5) * 1200.0,  // -600 to 600
                (rand.r#gen::<f32>() - 0.5) * 600.0,   // -300 to 300
            ),
            // Random size between 4 and 20
            size: 4.0 + rand.r#gen::<f32>() * 16.0,
        }
    }
}

// CUSTOM EVENTS - Define what can be observed

#[derive(Event)]
struct ExplodeMines {
    pos: Vec2,      // Explosion center
    radius: f32,    // Blast radius
}

#[derive(Event)]
struct Explode;     // Simple trigger for individual mines

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    // Instructions text
    commands.spawn((
        Text::new(
            "Click on a \"Mine\" to trigger it.\n\
            When it explodes it will trigger all overlapping mines.",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.),
            left: Val::Px(12.),
            ..default()
        },
    ));

    // Seeded RNG for consistent mine placement
    let mut rng = ChaCha8Rng::seed_from_u64(19878367467713);

    // ENTITY-SPECIFIC OBSERVER - Method 1
    // Each entity gets its own observer instance
    commands
        .spawn(Mine::random(&mut rng))
        // This mine watches for Explode events targeting it
        .observe(explode_mine);

    // SHARED OBSERVER - Method 2 (More Efficient)
    // Instead of 1000 observers, we create ONE observer watching 1000 entities
    // Like having one security system monitoring many cameras
    
    // Observers are entities too! The Observer component makes them special
    let mut observer = Observer::new(explode_mine);

    // Spawn mines and register them with our shared observer
    for _ in 0..1000 {
        let entity = commands.spawn(Mine::random(&mut rng)).id();
        // This observer will respond to Explode events for this entity
        observer.watch_entity(entity);
    }

    // Activate the observer by spawning it
    commands.spawn(observer);
}

// LIFECYCLE OBSERVER - Runs when Mine component is added
// Automatically maintains our spatial index
fn on_add_mine(
    trigger: Trigger<OnAdd, Mine>,
    query: Query<&Mine>,
    mut index: ResMut<SpatialIndex>,
) {
    // Get the mine that was just added
    let mine = query.get(trigger.target()).unwrap();
    
    // Calculate grid cell coordinates
    let tile = (
        (mine.pos.x / CELL_SIZE).floor() as i32,
        (mine.pos.y / CELL_SIZE).floor() as i32,
    );
    
    // Add entity to spatial index for efficient lookups
    index.map.entry(tile).or_default().insert(trigger.target());
}

// LIFECYCLE OBSERVER - Runs when Mine component is removed
// Keeps our spatial index clean
fn on_remove_mine(
    trigger: Trigger<OnRemove, Mine>,
    query: Query<&Mine>,
    mut index: ResMut<SpatialIndex>,
) {
    // Get the mine before it's removed
    let mine = query.get(trigger.target()).unwrap();
    
    // Find its grid cell
    let tile = (
        (mine.pos.x / CELL_SIZE).floor() as i32,
        (mine.pos.y / CELL_SIZE).floor() as i32,
    );
    
    // Remove from spatial index
    index.map.entry(tile).and_modify(|set| {
        set.remove(&trigger.target());
    });
}

// ENTITY OBSERVER - Responds to Explode events for specific mines
fn explode_mine(trigger: Trigger<Explode>, query: Query<&Mine>, mut commands: Commands) {
    // Get the entity that received the Explode event
    let id = trigger.target();
    
    // Try to get the entity (might already be despawned)
    let Ok(mut entity) = commands.get_entity(id) else {
        return;
    };
    
    info!("Boom! {} exploded.", id.index());
    
    // Get mine data before despawning
    let mine = query.get(id).unwrap();
    let pos = mine.pos;
    let size = mine.size;
    
    // Remove the mine
    entity.despawn();
    
    // CHAIN REACTION! Trigger area explosion
    commands.trigger(ExplodeMines {
        pos,
        radius: size,
    });
}

// Visualization system - draws mines as colored circles
fn draw_shapes(mut gizmos: Gizmos, mines: Query<&Mine>) {
    for mine in &mines {
        // Color based on size - small mines are red, large are violet
        // Like heat map: small = hot, large = cool
        gizmos.circle_2d(
            mine.pos,
            mine.size,
            Color::hsl((mine.size - 4.0) / 16.0 * 360.0, 1.0, 0.8),
        );
    }
}

// Input handler - converts mouse clicks to world explosions
fn handle_click(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut commands: Commands,
) {
    let Ok(windows) = windows.single() else {
        return;
    };

    let (camera, camera_transform) = *camera;
    
    // Convert screen position to world position
    if let Some(pos) = windows
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
    {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            // Start explosion at click position
            commands.trigger(ExplodeMines { pos, radius: 1.0 });
        }
    }
}

// SPATIAL INDEX - Optimization structure for finding nearby mines
// Instead of checking every mine, we divide space into a grid
#[derive(Resource, Default)]
struct SpatialIndex {
    // Grid cells -> entities in that cell
    map: HashMap<(i32, i32), HashSet<Entity>>,
}

/// Cell size has to be bigger than any mine radius
const CELL_SIZE: f32 = 64.0;

impl SpatialIndex {
    // Find all entities near a position efficiently
    // Checks only the 9 cells around the position
    fn get_nearby(&self, pos: Vec2) -> Vec<Entity> {
        // Calculate which cell we're in
        let tile = (
            (pos.x / CELL_SIZE).floor() as i32,
            (pos.y / CELL_SIZE).floor() as i32,
        );
        
        let mut nearby = Vec::new();
        // Check 3x3 grid around position
        for x in -1..2 {
            for y in -1..2 {
                if let Some(mines) = self.map.get(&(tile.0 + x, tile.1 + y)) {
                    nearby.extend(mines.iter());
                }
            }
        }
        nearby
    }
}
