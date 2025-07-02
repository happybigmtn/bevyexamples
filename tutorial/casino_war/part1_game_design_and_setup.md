# Casino War Tutorial - Part 1: Game Design and Setup

## What We're Building

Casino War is one of the simplest casino card games. Each player gets one card, highest card wins. In case of a tie, players can either surrender (lose half their bet) or go to "war" (double the bet and play again). 

This tutorial will teach you:
- Bevy 0.16 fundamentals through a complete game
- Game state management 
- Card rendering and animations
- UI systems for betting and scoring
- Event-driven architecture
- Asset management

## Game Rules

1. **Initial Bet**: Player places a bet
2. **Deal**: One card each to player and dealer
3. **Compare**: Higher card wins (Ace is high)
4. **Tie Handling**:
   - Surrender: Lose half the bet
   - Go to War: Match the original bet, deal 3 burn cards, then one card each
5. **War Resolution**: If player wins war, they win 2x the war bet. If dealer wins, player loses all bets

## Project Setup

First, create a new Rust project:

```bash
cargo new casino_war
cd casino_war
```

Add Bevy 0.16 to your `Cargo.toml`:

```toml
[package]
name = "casino_war"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = "0.16"
rand = "0.8"

# Optimize for faster compile times in development
[profile.dev]
opt-level = 1

# Enable optimizations for dependencies but not our code
[profile.dev.package."*"]
opt-level = 3
```

## Core Data Structures

Create `src/main.rs` and define our core game types:

```rust
use bevy::prelude::*;
use rand::prelude::*;

// Card representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rank {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,  // Ace is high in Casino War
}

#[derive(Debug, Clone, Copy, Component)]
struct Card {
    suit: Suit,
    rank: Rank,
}

impl Card {
    fn value(&self) -> u8 {
        self.rank as u8
    }
}

// Game phases
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GamePhase {
    #[default]
    MainMenu,
    Betting,
    Dealing,
    Comparing,
    TieDecision,  // Player chooses surrender or war
    War,
    RoundComplete,
}

// Resources for game state
#[derive(Resource)]
struct GameState {
    player_chips: u32,
    current_bet: u32,
    war_bet: u32,  // Additional bet during war
    deck: Vec<Card>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            player_chips: 1000,  // Starting chips
            current_bet: 0,
            war_bet: 0,
            deck: Self::create_deck(),
        }
    }
}

impl GameState {
    fn create_deck() -> Vec<Card> {
        let mut deck = Vec::with_capacity(52);
        use Suit::*;
        use Rank::*;
        
        for &suit in &[Hearts, Diamonds, Clubs, Spades] {
            for &rank in &[Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace] {
                deck.push(Card { suit, rank });
            }
        }
        
        // Shuffle the deck
        let mut rng = thread_rng();
        deck.shuffle(&mut rng);
        
        deck
    }
    
    fn draw_card(&mut self) -> Option<Card> {
        if self.deck.is_empty() {
            self.deck = Self::create_deck();
        }
        self.deck.pop()
    }
}

// Component markers for entities
#[derive(Component)]
struct PlayerCard;

#[derive(Component)]
struct DealerCard;

#[derive(Component)]
struct BurnCard;  // Cards burned during war

// Events for game flow
#[derive(Event)]
struct BetPlaced(u32);

#[derive(Event)]
struct DealCards;

#[derive(Event)]
struct PlayerDecision {
    go_to_war: bool,
}

#[derive(Event)]
struct RoundResult {
    player_won: bool,
    winnings: i32,  // Can be negative for losses
}
```

## Basic App Structure

Now let's set up the basic Bevy app with our game states:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Casino War".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GamePhase>()
        .init_resource::<GameState>()
        .add_event::<BetPlaced>()
        .add_event::<DealCards>()
        .add_event::<PlayerDecision>()
        .add_event::<RoundResult>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GamePhase::MainMenu), setup_main_menu)
        .add_systems(OnExit(GamePhase::MainMenu), cleanup_menu)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn 2D camera
    commands.spawn(Camera2d);
}

fn setup_main_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root UI container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        StateScoped(GamePhase::MainMenu),  // Auto-cleanup when leaving state
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Casino War"),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.8, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(50.0)),
                ..default()
            },
        ));
        
        // Play button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::srgb(0.8, 0.7, 0.0)),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Play"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<StateScoped>>) {
    // This is handled automatically by StateScoped component in 0.16
}
```

## Key Concepts Explained

### 1. **State Management**
We use Bevy's state system to control game flow. Each phase has distinct behavior:
- `MainMenu`: Show title screen
- `Betting`: Player chooses bet amount
- `Dealing`: Animate card dealing
- `Comparing`: Show results
- `TieDecision`: Handle tie scenario
- `War`: Special war round
- `RoundComplete`: Clean up and prepare for next round

### 2. **Component Architecture**
Instead of inheritance, we use components to mark entities:
- `PlayerCard`, `DealerCard`, `BurnCard` are marker components
- `Card` holds the actual card data
- This allows us to query specific cards easily

### 3. **Event-Driven Design**
Events decouple systems:
- `BetPlaced`: Triggers dealing phase
- `DealCards`: Initiates card animation
- `PlayerDecision`: Handles tie choices
- `RoundResult`: Updates chips and UI

### 4. **Resource Pattern**
`GameState` is a global resource holding:
- Player's chip count
- Current bets
- The deck of cards

This follows Bevy's pattern of using resources for singleton game data.

## Visual Layout Planning

Our game screen will have these areas:

```
+------------------------+
|     Dealer Cards       |
|                        |
|    [?]  or  [K♠]      |
|                        |
|    Betting Area        |
|   Current Bet: $50     |
|                        |
|     Player Cards       |
|    [A♥]  or  [?]      |
|                        |
|  Chips: $950  [BET]    |
+------------------------+
```

## Next Steps

In Part 2, we'll implement:
- Card rendering system
- Betting UI with chip selection
- State transitions
- Basic animations

The modular design we've established makes it easy to add features incrementally. Each system handles one concern, making the code maintainable and testable.

## Exercises

1. **Add a Settings Resource**: Create a resource to store game settings like minimum bet, maximum bet, and animation speed

2. **Implement Deck Validation**: Add a method to `GameState` that verifies the deck has exactly 52 unique cards

3. **Create Card Display**: Think about how you'd implement a `display()` method for `Card` that returns strings like "A♠" or "K♥"

4. **Plan the Animation**: Consider what components you'd need to animate cards moving from deck to table positions

## Key Takeaways

- Bevy 0.16 uses explicit component spawning (no bundles for basic types)
- States control game flow with `OnEnter`/`OnExit` systems
- Events enable loose coupling between systems
- Resources hold global game data
- Components should be small and focused
- Marker components are useful for entity identification