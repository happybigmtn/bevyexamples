// Casino War Part 8: Real-time Leaderboard System - The Competitive Edge
//
// This part adds a live leaderboard that transforms the psychological aspect
// of the game. Players can see their position in real-time, creating strategic
// depth where betting decisions are influenced by tournament standings.
//
// Key concepts we'll explore:
// 1. Real-time Data Visualization - Live updating leaderboard displays
// 2. Competitive Psychology - How rankings affect decision-making
// 3. Performance Metrics - Multi-dimensional player statistics
// 4. Dynamic UI Layout - Responsive leaderboard that adapts to data
// 5. Trend Analysis - Momentum indicators and performance trajectories
// 6. Strategic Positioning - How leaderboard position influences AI behavior

use bevy::prelude::*;
use rand::prelude::*;

// PART 8: McLaren-inspired color palette (carried forward)
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
    
    // Bot identity colors
    pub const BOT_COLORS: [Color; 5] = [
        Color::srgb(1.0, 0.2, 0.2),    // Red - Aggressive Bot
        Color::srgb(0.2, 0.8, 0.2),    // Green - Conservative Bot  
        Color::srgb(0.2, 0.2, 1.0),    // Blue - Balanced Bot
        Color::srgb(1.0, 0.8, 0.2),    // Yellow - Adaptive Bot
        Color::srgb(0.8, 0.2, 1.0),    // Purple - Chaos Bot
    ];
    
    // PART 8: Leaderboard-specific colors
    pub const FIRST_PLACE: Color = Color::srgb(1.0, 0.84, 0.0);          // Gold
    pub const SECOND_PLACE: Color = Color::srgb(0.75, 0.75, 0.75);       // Silver  
    pub const THIRD_PLACE: Color = Color::srgb(0.8, 0.5, 0.2);           // Bronze
    pub const TRENDING_UP: Color = Color::srgb(0.0, 1.0, 0.4);           // Green arrow
    pub const TRENDING_DOWN: Color = Color::srgb(1.0, 0.2, 0.2);         // Red arrow
    pub const TRENDING_STABLE: Color = Color::srgb(0.7, 0.7, 0.7);       // Gray arrow
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

// AI Strategy System (carried forward from Part 7)
#[derive(Debug, Clone, Copy, PartialEq)]
enum AIStrategy {
    Conservative,
    Aggressive,
    Balanced,
    Adaptive,
    Chaos,
}

impl AIStrategy {
    fn get_name(&self) -> &'static str {
        match self {
            AIStrategy::Conservative => "CONSERVATIVE",
            AIStrategy::Aggressive => "AGGRESSIVE", 
            AIStrategy::Balanced => "BALANCED",
            AIStrategy::Adaptive => "ADAPTIVE",
            AIStrategy::Chaos => "CHAOS",
        }
    }
    
    fn get_description(&self) -> &'static str {
        match self {
            AIStrategy::Conservative => "Safe player",
            AIStrategy::Aggressive => "High risk",
            AIStrategy::Balanced => "Strategic",
            AIStrategy::Adaptive => "Adaptive", 
            AIStrategy::Chaos => "Unpredictable",
        }
    }
}

// Player representation with enhanced statistics for leaderboard
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerType {
    Human,
    AI(AIStrategy),
}

// PART 8: Enhanced Player Statistics for Leaderboard Analysis
//
// We track comprehensive statistics that feed into the leaderboard
// and provide rich data for strategic decision-making.

#[derive(Component, Debug)]
struct Player {
    id: usize,
    player_type: PlayerType,
    chips: u32,
    current_bet: u32,
    is_active: bool,
    
    // Basic performance metrics
    wins: u32,
    losses: u32,
    wars_won: u32,
    wars_lost: u32,
    
    // PART 8: Advanced leaderboard metrics
    total_winnings: i32,           // Net profit/loss (can be negative)
    biggest_win: u32,              // Largest single round win
    biggest_loss: u32,             // Largest single round loss
    win_streak: u32,               // Current consecutive wins
    loss_streak: u32,              // Current consecutive losses
    max_win_streak: u32,           // Best win streak this session
    max_loss_streak: u32,          // Worst loss streak this session
    
    // Risk and aggression metrics
    total_wagered: u32,            // Total amount bet across all rounds
    average_bet: f32,              // Running average bet size
    war_participation_rate: f32,   // Percentage of wars participated in
    risk_factor: f32,              // Calculated risk level (0.0 = safe, 1.0 = risky)
    
    // Performance trends
    last_5_results: [i32; 5],      // Last 5 round results (positive = win, negative = loss)
    momentum_score: f32,           // Trending up/down indicator (-1.0 to 1.0)
    performance_rating: f32,       // Overall performance score (0.0 to 100.0)
    
    // Psychological factors
    confidence_level: f32,         // AI confidence based on recent performance
    pressure_resistance: f32,      // How well player handles being behind
    adaptability: f32,             // How quickly player adjusts strategy
    
    // Timing and AI behavior
    last_decision_time: f32,
    decision_count: u32,           // Total decisions made
    average_decision_time: f32,    // Average time per decision
}

impl Player {
    fn new_human() -> Self {
        Self {
            id: 0,
            player_type: PlayerType::Human,
            chips: 1000,
            current_bet: 0,
            is_active: true,
            wins: 0,
            losses: 0,
            wars_won: 0,
            wars_lost: 0,
            total_winnings: 0,
            biggest_win: 0,
            biggest_loss: 0,
            win_streak: 0,
            loss_streak: 0,
            max_win_streak: 0,
            max_loss_streak: 0,
            total_wagered: 0,
            average_bet: 0.0,
            war_participation_rate: 0.0,
            risk_factor: 0.5,
            last_5_results: [0; 5],
            momentum_score: 0.0,
            performance_rating: 50.0,
            confidence_level: 0.5,
            pressure_resistance: 0.5,
            adaptability: 0.5,
            last_decision_time: 0.0,
            decision_count: 0,
            average_decision_time: 0.0,
        }
    }
    
    fn new_ai(id: usize, strategy: AIStrategy) -> Self {
        // AI personalities have different baseline psychological attributes
        let (confidence, pressure_resistance, adaptability) = match strategy {
            AIStrategy::Conservative => (0.3, 0.8, 0.2),   // Low confidence, high pressure resistance, low adaptability
            AIStrategy::Aggressive => (0.9, 0.2, 0.3),     // High confidence, low pressure resistance, low adaptability  
            AIStrategy::Balanced => (0.6, 0.6, 0.6),       // Balanced across all metrics
            AIStrategy::Adaptive => (0.5, 0.7, 0.9),       // Medium confidence, good pressure resistance, high adaptability
            AIStrategy::Chaos => (0.8, 0.1, 0.1),          // High confidence, terrible under pressure, no adaptability
        };
        
        Self {
            id,
            player_type: PlayerType::AI(strategy),
            chips: 1000,
            current_bet: 0,
            is_active: true,
            wins: 0,
            losses: 0,
            wars_won: 0,
            wars_lost: 0,
            total_winnings: 0,
            biggest_win: 0,
            biggest_loss: 0,
            win_streak: 0,
            loss_streak: 0,
            max_win_streak: 0,
            max_loss_streak: 0,
            total_wagered: 0,
            average_bet: 0.0,
            war_participation_rate: 0.0,
            risk_factor: 0.5,
            last_5_results: [0; 5],
            momentum_score: 0.0,
            performance_rating: 50.0,
            confidence_level: confidence,
            pressure_resistance,
            adaptability,
            last_decision_time: 0.0,
            decision_count: 0,
            average_decision_time: 0.0,
        }
    }
    
    fn get_name(&self) -> String {
        match self.player_type {
            PlayerType::Human => "YOU".to_string(),
            PlayerType::AI(strategy) => format!("{}", strategy.get_name()),
        }
    }
    
    fn get_color(&self) -> Color {
        match self.player_type {
            PlayerType::Human => MCLAREN_ORANGE,
            PlayerType::AI(_) => BOT_COLORS[self.id - 1],
        }
    }
    
    // PART 8: Enhanced statistics methods for leaderboard calculations
    
    fn update_round_result(&mut self, amount_won: i32, was_war: bool) {
        // Update basic stats
        if amount_won > 0 {
            self.wins += 1;
            self.win_streak += 1;
            self.loss_streak = 0;
            self.max_win_streak = self.max_win_streak.max(self.win_streak);
            self.biggest_win = self.biggest_win.max(amount_won as u32);
            if was_war { self.wars_won += 1; }
        } else {
            self.losses += 1;
            self.loss_streak += 1;
            self.win_streak = 0;
            self.max_loss_streak = self.max_loss_streak.max(self.loss_streak);
            self.biggest_loss = self.biggest_loss.max((-amount_won) as u32);
            if was_war { self.wars_lost += 1; }
        }
        
        // Update financial tracking
        self.total_winnings += amount_won;
        self.chips = ((self.chips as i32) + amount_won).max(0) as u32;
        
        // Update last 5 results (rolling window)
        for i in 0..4 {
            self.last_5_results[i] = self.last_5_results[i + 1];
        }
        self.last_5_results[4] = amount_won;
        
        // Recalculate derived metrics
        self.update_momentum_score();
        self.update_performance_rating();
        self.update_confidence_level();
    }
    
    fn update_bet_placed(&mut self, amount: u32) {
        self.total_wagered += amount;
        self.decision_count += 1;
        
        // Update average bet (running average)
        let total_bets = self.wins + self.losses;
        if total_bets > 0 {
            self.average_bet = (self.total_wagered as f32) / (total_bets as f32);
        }
        
        // Update risk factor based on bet size relative to chips
        let bet_ratio = (amount as f32) / (self.chips as f32);
        self.risk_factor = (self.risk_factor * 0.9) + (bet_ratio * 0.1); // Smoothed update
    }
    
    fn update_war_decision(&mut self, participated: bool) {
        let total_war_opportunities = self.wars_won + self.wars_lost + if participated { 0 } else { 1 };
        if total_war_opportunities > 0 {
            let total_participated = self.wars_won + self.wars_lost + if participated { 1 } else { 0 };
            self.war_participation_rate = (total_participated as f32) / (total_war_opportunities as f32);
        }
    }
    
    fn update_momentum_score(&mut self) {
        // Calculate momentum based on last 5 results
        let mut momentum: f32 = 0.0;
        let mut weight = 1.0;
        
        for &result in self.last_5_results.iter().rev() {
            if result > 0 {
                momentum += weight;
            } else if result < 0 {
                momentum -= weight;
            }
            weight *= 0.8; // Decay weight for older results
        }
        
        // Normalize to -1.0 to 1.0 range
        self.momentum_score = momentum.tanh();
    }
    
    fn update_performance_rating(&mut self) {
        // Multi-factor performance rating
        let total_games = self.wins + self.losses;
        if total_games == 0 {
            self.performance_rating = 50.0;
            return;
        }
        
        // Win rate component (0-40 points)
        let win_rate = (self.wins as f32) / (total_games as f32);
        let win_score = win_rate * 40.0;
        
        // Profit component (0-30 points)
        let profit_ratio = (self.total_winnings as f32) / 1000.0; // Starting chips
        let profit_score = (profit_ratio.tanh() + 1.0) * 15.0; // -15 to +15, shifted to 0-30
        
        // Risk efficiency component (0-20 points)
        let risk_efficiency = if self.risk_factor > 0.0 {
            win_rate / self.risk_factor.min(1.0)
        } else {
            win_rate
        };
        let efficiency_score = risk_efficiency * 20.0;
        
        // Consistency component (0-10 points)
        let consistency = 1.0 - (self.max_loss_streak as f32 / total_games as f32);
        let consistency_score = consistency * 10.0;
        
        self.performance_rating = (win_score + profit_score + efficiency_score + consistency_score)
            .min(100.0)
            .max(0.0);
    }
    
    fn update_confidence_level(&mut self) {
        // Confidence is affected by recent performance and player type
        let base_confidence = match self.player_type {
            PlayerType::Human => 0.5,
            PlayerType::AI(strategy) => match strategy {
                AIStrategy::Conservative => 0.3,
                AIStrategy::Aggressive => 0.9,
                AIStrategy::Balanced => 0.6,
                AIStrategy::Adaptive => 0.5,
                AIStrategy::Chaos => 0.8,
            }
        };
        
        // Adjust based on momentum and performance
        let momentum_adjustment = self.momentum_score * 0.2;
        let performance_adjustment = (self.performance_rating - 50.0) / 500.0; // -0.1 to +0.1
        
        self.confidence_level = (base_confidence + momentum_adjustment + performance_adjustment)
            .min(1.0)
            .max(0.0);
    }
    
    // Get leaderboard display metrics
    fn get_leaderboard_score(&self) -> f32 {
        // Primary score is chip count, but we weight by performance rating
        let chip_score = self.chips as f32;
        let performance_multiplier = 0.8 + (self.performance_rating / 500.0); // 0.8 to 1.2
        chip_score * performance_multiplier
    }
    
    fn get_trend_direction(&self) -> TrendDirection {
        if self.momentum_score > 0.3 {
            TrendDirection::Up
        } else if self.momentum_score < -0.3 {
            TrendDirection::Down
        } else {
            TrendDirection::Stable
        }
    }
}

// PART 8: Leaderboard System Components and Resources

#[derive(Debug, Clone, Copy, PartialEq)]
enum TrendDirection {
    Up,
    Down,
    Stable,
}

impl TrendDirection {
    fn get_color(&self) -> Color {
        match self {
            TrendDirection::Up => TRENDING_UP,
            TrendDirection::Down => TRENDING_DOWN,
            TrendDirection::Stable => TRENDING_STABLE,
        }
    }
    
    fn get_symbol(&self) -> &'static str {
        match self {
            TrendDirection::Up => "▲",
            TrendDirection::Down => "▼",
            TrendDirection::Stable => "■",
        }
    }
}

// Leaderboard entry represents a player's current standing
#[derive(Debug, Clone, Component)]
struct LeaderboardEntry {
    player_id: usize,
    position: usize,              // 1st, 2nd, 3rd, etc.
    previous_position: usize,     // Position last update (for change tracking)
    name: String,
    color: Color,
    score: f32,                   // Calculated leaderboard score
    chips: u32,
    performance_rating: f32,
    momentum_score: f32,
    trend_direction: TrendDirection,
    win_rate: f32,
    total_winnings: i32,
    win_streak: u32,
    risk_factor: f32,
}

impl LeaderboardEntry {
    fn from_player(player: &Player, position: usize, previous_position: usize) -> Self {
        let total_games = player.wins + player.losses;
        let win_rate = if total_games > 0 {
            (player.wins as f32) / (total_games as f32)
        } else {
            0.0
        };
        
        Self {
            player_id: player.id,
            position,
            previous_position,
            name: player.get_name(),
            color: player.get_color(),
            score: player.get_leaderboard_score(),
            chips: player.chips,
            performance_rating: player.performance_rating,
            momentum_score: player.momentum_score,
            trend_direction: player.get_trend_direction(),
            win_rate,
            total_winnings: player.total_winnings,
            win_streak: player.win_streak,
            risk_factor: player.risk_factor,
        }
    }
    
    fn get_position_color(&self) -> Color {
        match self.position {
            1 => FIRST_PLACE,
            2 => SECOND_PLACE,
            3 => THIRD_PLACE,
            _ => self.color,
        }
    }
    
    fn get_position_change(&self) -> i32 {
        self.previous_position as i32 - self.position as i32
    }
}

// Leaderboard resource manages the complete standings
#[derive(Resource)]
struct Leaderboard {
    entries: Vec<LeaderboardEntry>,
    last_update: f32,
    update_frequency: f32,        // How often to recalculate (in seconds)
    animation_timer: f32,         // For smooth position changes
    highlight_player: Option<usize>, // Which player to highlight
}

impl Leaderboard {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            last_update: 0.0,
            update_frequency: 0.5,   // Update twice per second
            animation_timer: 0.0,
            highlight_player: Some(0), // Highlight human player by default
        }
    }
    
    fn update_from_players(&mut self, players: &[Player], current_time: f32) {
        if current_time - self.last_update < self.update_frequency {
            return;
        }
        
        // Store previous positions
        let previous_positions: std::collections::HashMap<usize, usize> = 
            self.entries.iter().map(|e| (e.player_id, e.position)).collect();
        
        // Create new entries sorted by leaderboard score
        let mut new_entries: Vec<_> = players.iter()
            .filter(|p| p.is_active)
            .map(|p| {
                let previous_pos = previous_positions.get(&p.id).copied().unwrap_or(99);
                LeaderboardEntry::from_player(p, 0, previous_pos)
            })
            .collect();
        
        // Sort by score (descending)
        new_entries.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Assign positions
        for (index, entry) in new_entries.iter_mut().enumerate() {
            entry.position = index + 1;
        }
        
        self.entries = new_entries;
        self.last_update = current_time;
        self.animation_timer = 0.0; // Reset animation
    }
    
    fn get_player_position(&self, player_id: usize) -> Option<usize> {
        self.entries.iter().find(|e| e.player_id == player_id).map(|e| e.position)
    }
    
    fn get_player_entry(&self, player_id: usize) -> Option<&LeaderboardEntry> {
        self.entries.iter().find(|e| e.player_id == player_id)
    }
    
    fn set_highlight_player(&mut self, player_id: Option<usize>) {
        self.highlight_player = player_id;
    }
}

// Game state and other components (carried forward from Part 7)
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GamePhase {
    #[default]
    MainMenu,
    Tournament,
    Betting,
    Dealing,
    Revealing,
    Resolving,
    War,
    RoundEnd,
    GameOver,
}

#[derive(Resource)]
struct Tournament {
    current_round: u32,
    max_rounds: u32,
    active_players: Vec<usize>,
    eliminated_players: Vec<usize>,
    tournament_winner: Option<usize>,
    pot: u32,
}

impl Tournament {
    fn new() -> Self {
        Self {
            current_round: 1,
            max_rounds: 50,
            active_players: vec![0, 1, 2, 3, 4, 5],
            eliminated_players: Vec::new(),
            tournament_winner: None,
            pot: 0,
        }
    }
}

#[derive(Resource)]
struct RoundState {
    betting_complete: bool,
    dealing_complete: bool,
    revealing_complete: bool,
    war_participants: Vec<usize>,
    round_winners: Vec<usize>,
    round_losers: Vec<usize>,
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

// PART 8: Leaderboard UI Components

#[derive(Component)]
struct LeaderboardPanel;

#[derive(Component)]
struct LeaderboardEntryComponent {
    player_id: usize,
}

#[derive(Component)]
struct PositionDisplay {
    player_id: usize,
}

#[derive(Component)]
struct PlayerNameDisplay {
    player_id: usize,
}

#[derive(Component)]
struct ChipCountDisplay {
    player_id: usize,
}

#[derive(Component)]
struct TrendDisplay {
    player_id: usize,
}

#[derive(Component)]
struct PerformanceDisplay {
    player_id: usize,
}

#[derive(Component)]
struct PositionChangeIndicator {
    player_id: usize,
}

// Visual effects for leaderboard
#[derive(Component)]
struct LeaderboardGlow {
    intensity: f32,
    color: Color,
}

#[derive(Component)]
struct AnimatedPosition {
    target_y: f32,
    current_y: f32,
    speed: f32,
}

// Events
#[derive(Event)]
struct LeaderboardUpdated;

#[derive(Event)]
struct PlayerHighlighted {
    player_id: usize,
}

// Input and other events (carried forward)
#[derive(Event)]
struct BetPlaced {
    player_id: usize,
    amount: u32,
}

#[derive(Event)]
struct RoundResult {
    player_id: usize,
    amount_won: i32,
    was_war: bool,
}

// Resources (carried forward)
#[derive(Resource)]
struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    fn new() -> Self {
        let mut cards = Vec::new();
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
    
    fn reset(&mut self) {
        *self = Self::new();
        self.shuffle();
    }
}

// Main application
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CASINO WAR - Leaderboard Edition".into(),
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
        .insert_resource(Leaderboard::new())
        .add_event::<BetPlaced>()
        .add_event::<RoundResult>()
        .add_event::<LeaderboardUpdated>()
        .add_event::<PlayerHighlighted>()
        .add_systems(Startup, setup_tournament)
        .add_systems(Update, (
            main_menu_system.run_if(in_state(GamePhase::MainMenu)),
            tournament_system.run_if(in_state(GamePhase::Tournament)),
            leaderboard_update_system,
            leaderboard_ui_system,
            leaderboard_animation_system,
            simulate_gameplay, // For demonstration
        ))
        .add_systems(Update, handle_button_interactions)
        .run();
}

// PART 8: Setup and Main Systems

fn setup_tournament(mut commands: Commands) {
    // Create camera
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
    
    // Create all 6 players
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
    
    // Setup main menu
    setup_main_menu(&mut commands);
}

fn setup_main_menu(commands: &mut Commands) {
    // Background
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            color: CARBON_BLACK,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    
    // Title
    commands.spawn((
        Text::new("CASINO WAR\nLEADERBOARD EDITION"),
        TextFont {
            font_size: 72.0,
            ..default()
        },
        TextColor(MCLAREN_ORANGE),
        Transform::from_xyz(0.0, 300.0, 1.0),
    ));
    
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

fn main_menu_system(
    mut next_state: ResMut<NextState<GamePhase>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GamePhase::Tournament);
        }
    }
}

fn tournament_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GamePhase>>,
    tournament_query: Query<Entity, With<Node>>, // Clear main menu
) {
    // Clear main menu UI
    for entity in &tournament_query {
        commands.entity(entity).despawn();
    }
    
    // Setup tournament UI with leaderboard
    setup_tournament_ui(&mut commands);
    
    next_state.set(GamePhase::Betting);
}

// PART 8: Leaderboard UI Setup and Management

fn setup_tournament_ui(commands: &mut Commands) {
    // Main tournament container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BackgroundColor(CARBON_BLACK),
    ))
    .with_children(|parent| {
        // Left side: Game area (70%)
        parent.spawn((
            Node {
                width: Val::Percent(70.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|game_area| {
            // Game title
            game_area.spawn((
                Text::new("TOURNAMENT IN PROGRESS"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(MCLAREN_ORANGE),
            ));
            
            // Game status
            game_area.spawn((
                Text::new("Watching the leaderboard battle unfold..."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
            ));
        });
        
        // Right side: Leaderboard panel (30%)
        parent.spawn((
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(PANEL_DARK),
        BorderColor(MCLAREN_ORANGE),
        LeaderboardPanel,
    ))
    .with_children(|leaderboard| {
        // Leaderboard header
        leaderboard.spawn((
            Text::new("🏆 LIVE STANDINGS"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(FIRST_PLACE),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));
        
        // Header columns
        leaderboard.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(PANEL_LIGHT),
        ))
        .with_children(|header| {
            header.spawn((
                Text::new("POS"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
                Node {
                    width: Val::Px(40.0),
                    ..default()
                },
            ));
            
            header.spawn((
                Text::new("PLAYER"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));
            
            header.spawn((
                Text::new("CHIPS"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
                Node {
                    width: Val::Px(60.0),
                    ..default()
                },
            ));
            
            header.spawn((
                Text::new("TREND"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
                Node {
                    width: Val::Px(40.0),
                    ..default()
                },
            ));
        });
        
        // Player entries container (will be populated by system)
        leaderboard.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(70.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ));
    });
    });
}

// PART 8: Leaderboard Update and Display Systems

fn leaderboard_update_system(
    mut leaderboard: ResMut<Leaderboard>,
    players: Query<&Player>,
    time: Res<Time>,
    mut leaderboard_events: EventWriter<LeaderboardUpdated>,
) {
    let players_vec: Vec<_> = players.iter().collect();
    let current_time = time.elapsed_secs();
    
    let entries_before = leaderboard.entries.len();
    let players_data: Vec<Player> = players_vec.into_iter().cloned().collect();
    leaderboard.update_from_players(&players_data, current_time);
    
    // Fire event if leaderboard changed
    if leaderboard.entries.len() != entries_before || 
       current_time - leaderboard.last_update < 0.1 {
        leaderboard_events.write(LeaderboardUpdated);
    }
}

fn leaderboard_ui_system(
    mut commands: Commands,
    leaderboard: Res<Leaderboard>,
    leaderboard_events: EventReader<LeaderboardUpdated>,
    leaderboard_panel: Query<Entity, With<LeaderboardPanel>>,
    existing_entries: Query<Entity, With<LeaderboardEntry>>,
) {
    // Only update when leaderboard changes
    if leaderboard_events.is_empty() {
        return;
    }
    
    // Find the leaderboard panel and its entries container
    let Ok(panel_entity) = leaderboard_panel.single() else { return };
    
    // Clear existing entries
    for entity in &existing_entries {
        commands.entity(entity).despawn();
    }
    
    // Find the entries container (last child of leaderboard panel)
    // Create new entries directly on the panel
    commands.entity(panel_entity).with_children(|parent| {
        for (index, entry) in leaderboard.entries.iter().enumerate() {
            let is_top = index == 0;
            let highlight_color = if entry.player_id == 0 { // Human player
                MCLAREN_ORANGE.with_alpha(0.2)
            } else {
                Color::NONE
            };
            
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: if is_top { UiRect::all(Val::Px(2.0)) } else { UiRect::ZERO },
                    ..default()
                },
                BackgroundColor(highlight_color),
                BorderColor(if is_top { FIRST_PLACE } else { Color::NONE }),
                entry.clone(),
                AnimatedPosition {
                    target_y: -(index as f32 * 52.0),
                    current_y: -(index as f32 * 52.0),
                    speed: 300.0,
                },
            ))
            .with_children(|row| {
                // Position
                row.spawn((
            Text::new(format!("{}", entry.position)),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(entry.get_position_color()),
            Node {
                width: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            PositionDisplay { player_id: entry.player_id },
        ));
        
        // Player name
        row.spawn((
            Text::new(entry.name.clone()),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(entry.color),
            Node {
                width: Val::Px(100.0),
                ..default()
            },
            PlayerNameDisplay { player_id: entry.player_id },
        ));
        
        // Chips
        row.spawn((
            Text::new(format!("{}", entry.chips)),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(TEXT_PRIMARY),
            Node {
                width: Val::Px(60.0),
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            ChipCountDisplay { player_id: entry.player_id },
        ));
        
        // Trend indicator
        row.spawn((
            Text::new(entry.trend_direction.get_symbol()),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(entry.trend_direction.get_color()),
            Node {
                width: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            TrendDisplay { player_id: entry.player_id },
        ));
        
        // Position change indicator
        let position_change = entry.get_position_change();
        if position_change != 0 {
            let change_text = if position_change > 0 {
                format!("+{}", position_change)
            } else {
                format!("{}", position_change)
            };
            
            let change_color = if position_change > 0 {
                TRENDING_UP
            } else {
                TRENDING_DOWN
            };
            
            row.spawn((
                Text::new(change_text),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(change_color),
                Node {
                    width: Val::Px(30.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                PositionChangeIndicator { player_id: entry.player_id },
                ));
            });
        }
    });
}

fn leaderboard_animation_system(
    mut animated_query: Query<(&mut AnimatedPosition, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut animated_pos, mut transform) in &mut animated_query {
        let diff = animated_pos.target_y - animated_pos.current_y;
        if diff.abs() > 1.0 {
            let movement = diff * animated_pos.speed * time.delta_secs();
            animated_pos.current_y += movement;
            transform.translation.y = animated_pos.current_y;
        }
    }
}

// PART 8: Gameplay Simulation (for demonstration)
fn simulate_gameplay(
    mut players: Query<&mut Player>,
    mut bet_events: EventWriter<BetPlaced>,
    mut result_events: EventWriter<RoundResult>,
    time: Res<Time>,
) {
    // Simple simulation to show leaderboard in action
    static mut LAST_ACTION: f32 = 0.0;
    let current_time = time.elapsed_secs();
    
    unsafe {
        if current_time - LAST_ACTION > 2.0 {
            LAST_ACTION = current_time;
            
            // Simulate random game results
            let mut rng = thread_rng();
            for mut player in &mut players {
                if player.is_active && player.chips > 10 {
                    // Random bet
                    let bet_amount = rng.gen_range(10..=50).min(player.chips);
                    bet_events.write(BetPlaced {
                        player_id: player.id,
                        amount: bet_amount,
                    });
                    
                    // Random result
                    let won = rng.gen_bool(0.5);
                    let amount_won = if won {
                        bet_amount as i32
                    } else {
                        -(bet_amount as i32)
                    };
                    
                    result_events.write(RoundResult {
                        player_id: player.id,
                        amount_won,
                        was_war: rng.gen_bool(0.1),
                    });
                    
                    player.update_round_result(amount_won, false);
                }
            }
        }
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