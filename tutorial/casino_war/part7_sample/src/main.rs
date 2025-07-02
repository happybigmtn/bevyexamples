// Casino War Part 7: Bot AI Players - The Tournament Begins
//
// This part transforms our single-player Casino War into a competitive arena
// with 5 AI opponents, each with unique playing styles and strategies.
//
// Key concepts we'll explore:
// 1. AI Architecture - Different behavioral patterns for game opponents
// 2. Strategy Pattern - Implementing multiple AI personalities
// 3. Multi-player State Management - Handling 6 simultaneous players
// 4. Tournament Mechanics - Round-robin style gameplay
// 5. Performance Monitoring - Tracking AI decision-making speed
// 6. Emergent Gameplay - How different strategies interact competitively

use bevy::prelude::*;
use rand::prelude::*;

// PART 7: McLaren-inspired color palette (carried forward)
pub mod mclaren_colors {
    use bevy::prelude::*;
    
    // Primary colors
    pub const MCLAREN_ORANGE: Color = Color::srgb(1.0, 0.529, 0.0);      // #FF8700
    pub const CARBON_BLACK: Color = Color::srgb(0.08, 0.08, 0.1);        // #141416
    pub const ALUMINUM: Color = Color::srgb(0.7, 0.71, 0.72);            // #B3B5B8
    
    // Accent colors
    pub const ENERGY_BLUE: Color = Color::srgb(0.0, 0.749, 1.0);         // #00BFFF
    pub const VICTORY_GREEN: Color = Color::srgb(0.0, 1.0, 0.4);         // #00FF66
    pub const DANGER_RED: Color = Color::srgb(1.0, 0.2, 0.2);            // #FF3333
    
    // UI colors
    pub const PANEL_DARK: Color = Color::srgba(0.05, 0.05, 0.07, 0.95);  // Semi-transparent
    pub const PANEL_LIGHT: Color = Color::srgba(0.2, 0.2, 0.22, 0.8);
    pub const TEXT_PRIMARY: Color = Color::srgb(0.95, 0.95, 0.95);
    pub const TEXT_SECONDARY: Color = Color::srgb(0.7, 0.7, 0.7);
    
    // Bot identity colors - Each bot gets a unique color for visual identification
    pub const BOT_COLORS: [Color; 5] = [
        Color::srgb(1.0, 0.2, 0.2),    // Red - Aggressive Bot
        Color::srgb(0.2, 0.8, 0.2),    // Green - Conservative Bot  
        Color::srgb(0.2, 0.2, 1.0),    // Blue - Balanced Bot
        Color::srgb(1.0, 0.8, 0.2),    // Yellow - Adaptive Bot
        Color::srgb(0.8, 0.2, 1.0),    // Purple - Chaos Bot
    ];
}

use mclaren_colors::*;

// Card representation (unchanged from previous parts)
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
    Ace = 14,
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
    
    fn get_suit_symbol(&self) -> &'static str {
        match self.suit {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        }
    }
    
    fn get_rank_symbol(&self) -> &'static str {
        match self.rank {
            Rank::Ace => "A",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
        }
    }
    
    fn get_color(&self) -> Color {
        match self.suit {
            Suit::Hearts | Suit::Diamonds => DANGER_RED,
            Suit::Clubs | Suit::Spades => TEXT_PRIMARY,
        }
    }
}

// PART 7: AI Strategy System - The Brain of Each Bot
//
// Each bot has a unique strategy that determines how they play.
// This creates emergent gameplay where different strategies compete.

#[derive(Debug, Clone, Copy, PartialEq)]
enum AIStrategy {
    // Conservative: Plays it safe, makes minimum bets, avoids wars
    Conservative,
    
    // Aggressive: Goes all-in frequently, loves wars, high risk/reward
    Aggressive,
    
    // Balanced: Calculated risks, medium bets, strategic war decisions
    Balanced,
    
    // Adaptive: Changes strategy based on current position and opponents
    Adaptive,
    
    // Chaos: Completely unpredictable, adds randomness to the game
    Chaos,
}

impl AIStrategy {
    // Get a human-readable name for the strategy
    fn get_name(&self) -> &'static str {
        match self {
            AIStrategy::Conservative => "CONSERVATIVE",
            AIStrategy::Aggressive => "AGGRESSIVE", 
            AIStrategy::Balanced => "BALANCED",
            AIStrategy::Adaptive => "ADAPTIVE",
            AIStrategy::Chaos => "CHAOS",
        }
    }
    
    // Get a description of the strategy for UI display
    fn get_description(&self) -> &'static str {
        match self {
            AIStrategy::Conservative => "Plays it safe, minimum bets",
            AIStrategy::Aggressive => "High risk, high reward",
            AIStrategy::Balanced => "Calculated strategic play",
            AIStrategy::Adaptive => "Changes based on position", 
            AIStrategy::Chaos => "Completely unpredictable",
        }
    }
}

// Player representation - Both human and AI players
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerType {
    Human,
    AI(AIStrategy),
}

#[derive(Component, Debug)]
struct Player {
    id: usize,                    // Unique identifier (0 = human, 1-5 = bots)
    player_type: PlayerType,      // Human or AI with strategy
    chips: u32,                   // Current chip count
    current_bet: u32,             // Bet for current round
    is_active: bool,              // Still in the game (has chips)
    wins: u32,                    // Wins this session
    losses: u32,                  // Losses this session
    wars_won: u32,                // Wars won (high stakes victories)
    wars_lost: u32,               // Wars lost (expensive defeats)
    last_decision_time: f32,      // AI thinking time simulation
}

impl Player {
    // Create a new human player
    fn new_human() -> Self {
        Self {
            id: 0,
            player_type: PlayerType::Human,
            chips: 1000,              // Starting chips
            current_bet: 0,
            is_active: true,
            wins: 0,
            losses: 0,
            wars_won: 0,
            wars_lost: 0,
            last_decision_time: 0.0,
        }
    }
    
    // Create a new AI player with specific strategy
    fn new_ai(id: usize, strategy: AIStrategy) -> Self {
        Self {
            id,
            player_type: PlayerType::AI(strategy),
            chips: 1000,              // All players start equal
            current_bet: 0,
            is_active: true,
            wins: 0,
            losses: 0,
            wars_won: 0,
            wars_lost: 0,
            last_decision_time: 0.0,
        }
    }
    
    // Get player's name for display
    fn get_name(&self) -> String {
        match self.player_type {
            PlayerType::Human => "YOU".to_string(),
            PlayerType::AI(strategy) => format!("BOT {}", strategy.get_name()),
        }
    }
    
    // Get player's color for UI
    fn get_color(&self) -> Color {
        match self.player_type {
            PlayerType::Human => MCLAREN_ORANGE,
            PlayerType::AI(_) => BOT_COLORS[self.id - 1], // Bots are IDs 1-5
        }
    }
    
    // Check if player can afford a bet
    fn can_afford(&self, amount: u32) -> bool {
        self.chips >= amount
    }
    
    // Place a bet (deduct from chips)
    fn place_bet(&mut self, amount: u32) -> bool {
        if self.can_afford(amount) {
            self.chips -= amount;
            self.current_bet = amount;
            true
        } else {
            false
        }
    }
    
    // Win chips (add to chip count)
    fn win_chips(&mut self, amount: u32) {
        self.chips += amount;
        self.wins += 1;
    }
    
    // Lose current bet (already deducted when bet was placed)
    fn lose_bet(&mut self) {
        self.losses += 1;
        self.current_bet = 0;
    }
    
    // Win a war (high stakes victory)
    fn win_war(&mut self, amount: u32) {
        self.chips += amount;
        self.wars_won += 1;
    }
    
    // Lose a war (expensive defeat)
    fn lose_war(&mut self) {
        self.wars_lost += 1;
        self.current_bet = 0;
    }
}

// PART 7: AI Decision Making Engine
//
// This is where the magic happens - each AI strategy makes decisions
// based on their personality and current game state.

struct AIDecisionContext {
    current_chips: u32,           // Bot's current chip count
    opponent_chips: Vec<u32>,     // All other players' chips
    round_number: u32,            // Current round (for adaptive strategies)
    time_remaining: f32,          // Time left in tournament (future use)
    last_card_value: Option<u8>,  // Previous card for pattern recognition
    consecutive_losses: u32,      // Recent losing streak
    position_in_standings: usize, // 1st, 2nd, 3rd, etc.
}

impl AIStrategy {
    // Calculate bet amount based on strategy and context
    fn calculate_bet(&self, context: &AIDecisionContext) -> u32 {
        let base_bet = 10u32; // Minimum bet
        let max_bet = context.current_chips.min(100); // Cap at 100 or all chips
        
        match self {
            // Conservative: Always minimum bet, exception for very strong position
            AIStrategy::Conservative => {
                if context.current_chips > 2000 && context.position_in_standings <= 2 {
                    base_bet * 2 // Slightly more when winning big
                } else {
                    base_bet
                }
            },
            
            // Aggressive: High bets, goes for broke
            AIStrategy::Aggressive => {
                let aggressive_bet = context.current_chips / 10; // 10% of chips
                aggressive_bet.max(base_bet * 3).min(max_bet)
            },
            
            // Balanced: Calculated based on chip count and position
            AIStrategy::Balanced => {
                let multiplier = match context.position_in_standings {
                    1 | 2 => 2,  // Leading: moderate bets
                    3 | 4 => 3,  // Middle: slightly higher
                    _ => 4,      // Behind: catch-up bets
                };
                (base_bet * multiplier).min(max_bet)
            },
            
            // Adaptive: Complex strategy that changes based on game state
            AIStrategy::Adaptive => {
                let mut bet = base_bet * 2;
                
                // Adapt based on position
                if context.position_in_standings > 3 {
                    bet *= 2; // More aggressive when behind
                }
                
                // Adapt based on recent performance
                if context.consecutive_losses > 2 {
                    bet /= 2; // Conservative after losing streak
                }
                
                // Adapt based on chip count relative to others
                let avg_opponent_chips: u32 = context.opponent_chips.iter().sum::<u32>() / context.opponent_chips.len() as u32;
                if context.current_chips < avg_opponent_chips / 2 {
                    bet = base_bet; // Desperate preservation mode
                }
                
                bet.min(max_bet)
            },
            
            // Chaos: Truly random, but with some bounds
            AIStrategy::Chaos => {
                let mut rng = thread_rng();
                let random_multiplier = rng.gen_range(1..=8);
                (base_bet * random_multiplier).min(max_bet)
            },
        }
    }
    
    // Decide whether to go to war when tied
    fn should_go_to_war(&self, context: &AIDecisionContext) -> bool {
        match self {
            // Conservative: Almost never goes to war
            AIStrategy::Conservative => {
                // Only if way ahead and can afford it
                context.position_in_standings == 1 && context.current_chips > 500
            },
            
            // Aggressive: Always goes to war
            AIStrategy::Aggressive => {
                context.current_chips >= 20 // Need minimum chips for war
            },
            
            // Balanced: Strategic war decisions
            AIStrategy::Balanced => {
                // Go to war if in good position or need to catch up
                (context.position_in_standings <= 2 && context.current_chips > 200) ||
                (context.position_in_standings > 4 && context.current_chips > 100)
            },
            
            // Adaptive: Context-dependent war decisions
            AIStrategy::Adaptive => {
                let mut should_war = false;
                
                // More likely to war if behind
                if context.position_in_standings > 3 {
                    should_war = true;
                }
                
                // Less likely after consecutive losses
                if context.consecutive_losses > 1 {
                    should_war = false;
                }
                
                // Must have enough chips
                should_war && context.current_chips >= 50
            },
            
            // Chaos: Random war decisions
            AIStrategy::Chaos => {
                let mut rng = thread_rng();
                rng.gen_bool(0.6) && context.current_chips >= 20 // 60% chance if affordable
            },
        }
    }
}

// PART 7: Multi-Player Game State Management
//
// Managing 6 players simultaneously requires careful state tracking
// and coordination between all the moving parts.

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GamePhase {
    #[default]
    MainMenu,
    Tournament,         // New: Tournament mode with multiple players
    Betting,           // All players place bets
    Dealing,           // Cards dealt to all players
    Revealing,         // Cards revealed simultaneously
    Resolving,         // Determine winners/losers
    War,              // War phase for tied players
    RoundEnd,         // Clean up and prepare next round
    GameOver,         // Tournament finished
}

// Tournament state - tracks the overall competition
#[derive(Resource)]
struct Tournament {
    current_round: u32,
    max_rounds: u32,
    active_players: Vec<usize>,    // Player IDs still in the game
    eliminated_players: Vec<usize>, // Player IDs that ran out of chips
    tournament_winner: Option<usize>, // Final winner
    pot: u32,                      // Total chips bet this round
}

impl Tournament {
    fn new() -> Self {
        Self {
            current_round: 1,
            max_rounds: 50,            // Tournament ends after 50 rounds max
            active_players: vec![0, 1, 2, 3, 4, 5], // All 6 players start active
            eliminated_players: Vec::new(),
            tournament_winner: None,
            pot: 0,
        }
    }
    
    // Check if tournament should end
    fn is_finished(&self) -> bool {
        self.active_players.len() <= 1 || self.current_round >= self.max_rounds
    }
    
    // Get tournament winner
    fn get_winner(&self, players: &[Player]) -> Option<usize> {
        if self.active_players.len() == 1 {
            Some(self.active_players[0])
        } else if self.current_round >= self.max_rounds {
            // Tournament time limit reached - highest chips wins
            players.iter()
                .filter(|p| p.is_active)
                .max_by_key(|p| p.chips)
                .map(|p| p.id)
        } else {
            None
        }
    }
    
    // Start next round
    fn next_round(&mut self) {
        self.current_round += 1;
        self.pot = 0;
    }
    
    // Eliminate a player
    fn eliminate_player(&mut self, player_id: usize) {
        if let Some(pos) = self.active_players.iter().position(|&id| id == player_id) {
            let eliminated_id = self.active_players.remove(pos);
            self.eliminated_players.push(eliminated_id);
        }
    }
}

// Round state - tracks current round progress
#[derive(Resource)]
struct RoundState {
    betting_complete: bool,
    dealing_complete: bool,
    revealing_complete: bool,
    war_participants: Vec<usize>,  // Players involved in current war
    round_winners: Vec<usize>,     // Winners of current round
    round_losers: Vec<usize>,      // Losers of current round
}

impl RoundState {
    fn new() -> Self {
        Self {
            betting_complete: false,
            dealing_complete: false,
            revealing_complete: false,
            war_participants: Vec::new(),
            round_winners: Vec::new(),
            round_losers: Vec::new(),
        }
    }
    
    fn reset(&mut self) {
        self.betting_complete = false;
        self.dealing_complete = false;
        self.revealing_complete = false;
        self.war_participants.clear();
        self.round_winners.clear();
        self.round_losers.clear();
    }
}

// PART 7: Enhanced Visual Components for Multi-Player Display

// Visual card components (enhanced for multiplayer)
#[derive(Component)]
struct CardVisual {
    face_up: bool,
    target_position: Vec3,
    player_id: usize,         // Which player owns this card
}

// Card position states (enhanced)
#[derive(Component, Debug, Clone, Copy, PartialEq)]
enum CardPosition {
    Deck,
    PlayerHand(usize),        // Which player's hand (0-5)
    Discard,
}

// McLaren-style visual effects (carried forward from Part 6)
#[derive(Component)]
struct HologramEffect {
    scan_speed: f32,
    glow_intensity: f32,
    flicker_rate: f32,
}

#[derive(Component)]
struct GlowEffect {
    color: Color,
    intensity: f32,
    radius: f32,
}

#[derive(Component)]
struct CarbonFiberAnimation {
    scroll_speed: Vec2,
    scale: f32,
}

// UI components for tournament display
#[derive(Component)]
struct PlayerNameDisplay {
    player_id: usize,
}

#[derive(Component)]
struct ChipCountDisplay {
    player_id: usize,
}

#[derive(Component)]
struct BetDisplay {
    player_id: usize,
}

#[derive(Component)]
struct StatusDisplay {
    player_id: usize,
}

#[derive(Component)]
struct TournamentInfoDisplay;

#[derive(Component)]
struct RoundDisplay;

// Input components
#[derive(Component)]
struct BetButton {
    amount: u32,
}

#[derive(Component)]
struct WarDecisionButton {
    go_to_war: bool,
}

#[derive(Component)]
struct NextRoundButton;

// Events for multi-player coordination
#[derive(Event)]
struct BetPlaced {
    player_id: usize,
    amount: u32,
}

#[derive(Event)]
struct AllBetsPlaced;

#[derive(Event)]
struct DealCards;

#[derive(Event)]
struct CardsDealt;

#[derive(Event)]
struct RevealCards;

#[derive(Event)]
struct CardsRevealed;

#[derive(Event)]
struct RoundResolved {
    winners: Vec<usize>,
    losers: Vec<usize>,
    ties: Vec<usize>,
}

#[derive(Event)]
struct WarDecision {
    player_id: usize,
    go_to_war: bool,
}

#[derive(Event)]
struct WarResolved {
    winners: Vec<usize>,
    losers: Vec<usize>,
}

#[derive(Event)]
struct RoundComplete;

#[derive(Event)]
struct TournamentComplete {
    winner_id: usize,
}

#[derive(Event)]
struct PlayerEliminated {
    player_id: usize,
}

// Game resources
#[derive(Resource)]
struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new() -> Self {
        let mut cards = Vec::new();
        
        // Create standard 52-card deck
        for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
            for rank in [Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six, 
                        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack, 
                        Rank::Queen, Rank::King, Rank::Ace] {
                cards.push(Card { suit, rank });
            }
        }
        
        Self { cards }
    }
    
    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }
    
    fn deal(&mut self) -> Option<Card> {
        self.cards.pop()
    }
    
    fn reset(&mut self) {
        *self = Self::new();
        self.shuffle();
    }
}

// AI thinking simulation - makes AI decisions feel more realistic
#[derive(Resource)]
struct AIThinkingTimer {
    timer: Timer,
    current_player: Option<usize>,
}

impl AIThinkingTimer {
    fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            current_player: None,
        }
    }
    
    fn start_thinking(&mut self, player_id: usize, thinking_time: f32) {
        self.current_player = Some(player_id);
        self.timer = Timer::from_seconds(thinking_time, TimerMode::Once);
    }
    
    fn is_thinking(&self) -> bool {
        self.current_player.is_some() && !self.timer.finished()
    }
    
    fn finish_thinking(&mut self) -> Option<usize> {
        if self.timer.finished() {
            let player = self.current_player;
            self.current_player = None;
            player
        } else {
            None
        }
    }
}

// Main application setup
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CASINO WAR - Tournament Edition".into(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GamePhase>()
        .insert_resource(ClearColor(CARBON_BLACK))
        .insert_resource(Tournament::new())
        .insert_resource(RoundState::new())
        .insert_resource(Deck::new())
        .insert_resource(AIThinkingTimer::new())
        .add_event::<BetPlaced>()
        .add_event::<AllBetsPlaced>()
        .add_event::<DealCards>()
        .add_event::<CardsDealt>()
        .add_event::<RevealCards>()
        .add_event::<CardsRevealed>()
        .add_event::<RoundResolved>()
        .add_event::<WarDecision>()
        .add_event::<WarResolved>()
        .add_event::<RoundComplete>()
        .add_event::<TournamentComplete>()
        .add_event::<PlayerEliminated>()
        .add_systems(Startup, setup_tournament)
        .add_systems(Update, (
            main_menu_system.run_if(in_state(GamePhase::MainMenu)),
            tournament_betting_system.run_if(in_state(GamePhase::Betting)),
            ai_betting_system.run_if(in_state(GamePhase::Betting)),
            tournament_dealing_system.run_if(in_state(GamePhase::Dealing)),
            tournament_revealing_system.run_if(in_state(GamePhase::Revealing)),
            tournament_resolving_system.run_if(in_state(GamePhase::Resolving)),
            tournament_war_system.run_if(in_state(GamePhase::War)),
            tournament_round_end_system.run_if(in_state(GamePhase::RoundEnd)),
            tournament_game_over_system.run_if(in_state(GamePhase::GameOver)),
            update_displays,
            handle_ai_thinking,
            animate_hologram_effects,
            update_carbon_fiber_animation,
            handle_button_interactions,
        ))
        .run();
}

// PART 7: Tournament Setup - Initialize All Players and UI

fn setup_tournament(
    mut commands: Commands,
    mut tournament: ResMut<Tournament>,
    mut deck: ResMut<Deck>,
) {
    // Initialize deck
    deck.reset();
    
    // Create camera
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
    
    // Create all 6 players (1 human + 5 AI bots)
    let players = vec![
        Player::new_human(),
        Player::new_ai(1, AIStrategy::Conservative),
        Player::new_ai(2, AIStrategy::Aggressive),
        Player::new_ai(3, AIStrategy::Balanced),
        Player::new_ai(4, AIStrategy::Adaptive),
        Player::new_ai(5, AIStrategy::Chaos),
    ];
    
    for player in players {
        commands.spawn(player);
    }
    
    // Setup main menu UI
    setup_main_menu(&mut commands);
}

// Main menu with tournament introduction
fn setup_main_menu(commands: &mut Commands) {
    // Background
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            color: CARBON_BLACK,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        CarbonFiberAnimation {
            scroll_speed: Vec2::new(10.0, 5.0),
            scale: 2.0,
        },
    ));
    
    // Main title
    commands.spawn((
        Text::new("CASINO WAR\nTOURNAMENT EDITION"),
        TextFont {
            font_size: 72.0,
            ..default()
        },
        TextColor(MCLAREN_ORANGE),
        Transform::from_xyz(0.0, 300.0, 1.0),
        GlowEffect {
            color: MCLAREN_ORANGE,
            intensity: 0.8,
            radius: 30.0,
        },
    ));
    
    // Tournament description
    commands.spawn((
        Text::new("COMPETE AGAINST 5 AI OPPONENTS\nEACH WITH UNIQUE STRATEGIES"),
        TextFont {
            font_size: 32.0,
            ..default()
        },
        TextColor(TEXT_PRIMARY),
        Transform::from_xyz(0.0, 150.0, 1.0),
    ));
    
    // Bot lineup description
    let bot_descriptions = [
        "CONSERVATIVE BOT - Plays it safe, minimum risks",
        "AGGRESSIVE BOT - High stakes, high rewards", 
        "BALANCED BOT - Calculated strategic play",
        "ADAPTIVE BOT - Changes tactics based on position",
        "CHAOS BOT - Completely unpredictable",
    ];
    
    for (i, description) in bot_descriptions.iter().enumerate() {
        commands.spawn((
            Text::new(*description),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(BOT_COLORS[i]),
            Transform::from_xyz(0.0, 50.0 - (i as f32 * 30.0), 1.0),
        ));
    }
    
    // Start button
    commands.spawn((
        Button,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(80.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1)),
        BorderColor(MCLAREN_ORANGE),
        Transform::from_xyz(0.0, -200.0, 1.0),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("START TOURNAMENT"),
            TextFont {
                font_size: 36.0,
                ..default()
            },
            TextColor(TEXT_PRIMARY),
        ));
    });
}

// PART 7: Main Menu System - Handle Tournament Start

fn main_menu_system(
    mut next_state: ResMut<NextState<GamePhase>>,
    mut interaction_query: Query<
        &Interaction,
        (Changed<Interaction>, With<Button>)
    >,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GamePhase::Tournament);
        }
    }
}

// PART 7: Tournament Betting System - Coordinate All Player Bets

fn tournament_betting_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut round_state: ResMut<RoundState>,
    mut players: Query<&mut Player>,
    mut bet_events: EventReader<BetPlaced>,
    mut all_bets_events: EventWriter<AllBetsPlaced>,
    tournament: Res<Tournament>,
) {
    // Handle bet placed events
    for bet_event in bet_events.read() {
        if let Some(mut player) = players.iter_mut().find(|p| p.id == bet_event.player_id) {
            if player.place_bet(bet_event.amount) {
                info!("Player {} bet {} chips", player.get_name(), bet_event.amount);
            }
        }
    }
    
    // Check if all active players have placed bets
    let active_players: Vec<_> = players.iter()
        .filter(|p| tournament.active_players.contains(&p.id))
        .collect();
    
    let bets_placed = active_players.iter()
        .all(|p| p.current_bet > 0);
    
    if bets_placed && !round_state.betting_complete {
        round_state.betting_complete = true;
        all_bets_events.write(AllBetsPlaced);
        next_state.set(GamePhase::Dealing);
    }
}

// PART 7: AI Betting System - Each Bot Makes Strategic Decisions

fn ai_betting_system(
    mut bet_events: EventWriter<BetPlaced>,
    mut ai_timer: ResMut<AIThinkingTimer>,
    players: Query<&Player>,
    tournament: Res<Tournament>,
    time: Res<Time>,
) {
    // Update AI thinking timer
    ai_timer.timer.tick(time.delta());
    
    // If an AI finished thinking, process their bet
    if let Some(player_id) = ai_timer.finish_thinking() {
        if let Some(player) = players.iter().find(|p| p.id == player_id) {
            if let PlayerType::AI(strategy) = player.player_type {
                // Create decision context for AI
                let other_players: Vec<_> = players.iter()
                    .filter(|p| p.id != player_id && tournament.active_players.contains(&p.id))
                    .collect();
                
                let opponent_chips: Vec<u32> = other_players.iter()
                    .map(|p| p.chips)
                    .collect();
                
                // Calculate position in standings (1 = best, 6 = worst)
                let mut all_chips: Vec<_> = players.iter()
                    .filter(|p| tournament.active_players.contains(&p.id))
                    .map(|p| p.chips)
                    .collect();
                all_chips.sort_by(|a, b| b.cmp(a)); // Sort descending
                
                let position = all_chips.iter()
                    .position(|&chips| chips == player.chips)
                    .unwrap_or(0) + 1;
                
                let context = AIDecisionContext {
                    current_chips: player.chips,
                    opponent_chips,
                    round_number: tournament.current_round,
                    time_remaining: 0.0, // TODO: Implement tournament timer
                    last_card_value: None, // TODO: Implement card memory
                    consecutive_losses: player.losses.saturating_sub(player.wins),
                    position_in_standings: position,
                };
                
                // AI makes betting decision
                let bet_amount = strategy.calculate_bet(&context);
                
                // Place the bet
                bet_events.write(BetPlaced {
                    player_id,
                    amount: bet_amount,
                });
                
                info!("AI {} ({:?}) bets {} chips (Position: {}, Chips: {})",
                      player.get_name(), strategy, bet_amount, position, player.chips);
            }
        }
    }
    
    // Start thinking for next AI that hasn't bet yet
    if !ai_timer.is_thinking() {
        for player in players.iter() {
            if let PlayerType::AI(strategy) = player.player_type {
                if tournament.active_players.contains(&player.id) && player.current_bet == 0 {
                    // Different strategies take different amounts of time to "think"
                    let thinking_time = match strategy {
                        AIStrategy::Conservative => 0.5,
                        AIStrategy::Aggressive => 0.2,
                        AIStrategy::Balanced => 1.0,
                        AIStrategy::Adaptive => 1.5,
                        AIStrategy::Chaos => 0.1,
                    };
                    
                    ai_timer.start_thinking(player.id, thinking_time);
                    break;
                }
            }
        }
    }
}

// PART 7: Tournament Systems - Placeholder implementations

fn tournament_dealing_system(
    mut next_state: ResMut<NextState<GamePhase>>,
    mut deal_events: EventWriter<DealCards>,
    mut round_state: ResMut<RoundState>,
) {
    if !round_state.dealing_complete {
        deal_events.write(DealCards);
        round_state.dealing_complete = true;
        next_state.set(GamePhase::Revealing);
    }
}

fn tournament_revealing_system(
    mut next_state: ResMut<NextState<GamePhase>>,
    mut reveal_events: EventWriter<RevealCards>,
    mut round_state: ResMut<RoundState>,
) {
    if !round_state.revealing_complete {
        reveal_events.write(RevealCards);
        round_state.revealing_complete = true;
        next_state.set(GamePhase::Resolving);
    }
}

fn tournament_resolving_system(
    mut next_state: ResMut<NextState<GamePhase>>,
    mut resolve_events: EventWriter<RoundResolved>,
    mut round_state: ResMut<RoundState>,
) {
    // TODO: Implement actual card comparison logic
    resolve_events.write(RoundResolved {
        winners: vec![0], // Placeholder
        losers: vec![1, 2, 3, 4, 5], // Placeholder
        ties: vec![], // Placeholder
    });
    next_state.set(GamePhase::RoundEnd);
}

fn tournament_war_system(
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    // TODO: Implement war logic
    next_state.set(GamePhase::RoundEnd);
}

fn tournament_round_end_system(
    mut next_state: ResMut<NextState<GamePhase>>,
    mut round_state: ResMut<RoundState>,
    mut tournament: ResMut<Tournament>,
) {
    round_state.reset();
    tournament.next_round();
    
    if tournament.is_finished() {
        next_state.set(GamePhase::GameOver);
    } else {
        next_state.set(GamePhase::Betting);
    }
}

fn tournament_game_over_system(
    // TODO: Implement game over logic
) {
}

// PART 7: UI Update Systems

fn update_displays(
    // TODO: Update all UI displays
) {
}

fn handle_ai_thinking(
    // TODO: Show AI thinking indicators
) {
}

fn animate_hologram_effects(
    mut query: Query<(&mut Transform, &HologramEffect)>,
    time: Res<Time>,
) {
    for (mut transform, effect) in &mut query {
        // Simple hologram flicker effect
        let flicker = (time.elapsed_secs() * effect.flicker_rate).sin() * 0.1;
        transform.scale = Vec3::splat(1.0 + flicker);
    }
}

fn update_carbon_fiber_animation(
    mut query: Query<(&mut Transform, &CarbonFiberAnimation)>,
    time: Res<Time>,
) {
    for (mut transform, animation) in &mut query {
        // Animate carbon fiber background
        let offset = animation.scroll_speed * time.elapsed_secs();
        transform.translation.x = (offset.x % 100.0) - 50.0;
        transform.translation.y = (offset.y % 100.0) - 50.0;
    }
}

fn handle_button_interactions(
    mut interaction_query: Query<
        (&Interaction, &Children),
        (Changed<Interaction>, With<Button>)
    >,
    mut text_query: Query<&mut TextColor>,
) {
    for (interaction, children) in &mut interaction_query {
        let color = match *interaction {
            Interaction::Pressed => MCLAREN_ORANGE,
            Interaction::Hovered => ENERGY_BLUE,
            Interaction::None => TEXT_PRIMARY,
        };
        
        for child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                *text_color = TextColor(color);
            }
        }
    }
}