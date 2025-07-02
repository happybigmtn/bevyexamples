// Casino War Part 9: Timed Tournament Mode - Winner Takes All
//
// This part adds the crucial time pressure element that transforms the game
// from casual play to intense competitive strategy. With only 2 minutes on the
// clock, every decision matters as players race for the highest chip count.
//
// Key concepts we'll explore:
// 1. Tournament Timer System - Real-time countdown with strategic implications
// 2. Time-Pressure Psychology - How urgency affects decision-making algorithms
// 3. Dynamic Strategy Adaptation - AI adjusts behavior based on time remaining
// 4. Winner-Take-All Mechanics - End-game scoring and prize distribution
// 5. Performance Analytics - Time-based metrics and efficiency calculations
// 6. Visual Urgency Design - UI that communicates time pressure effectively

use bevy::prelude::*;
use rand::prelude::*;

// PART 9: McLaren-inspired color palette with time-pressure variants
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
    
    // Leaderboard colors
    pub const FIRST_PLACE: Color = Color::srgb(1.0, 0.84, 0.0);          // Gold
    pub const SECOND_PLACE: Color = Color::srgb(0.75, 0.75, 0.75);       // Silver  
    pub const THIRD_PLACE: Color = Color::srgb(0.8, 0.5, 0.2);           // Bronze
    
    // PART 9: Time-pressure specific colors
    pub const TIME_NORMAL: Color = Color::srgb(0.0, 1.0, 0.4);          // Green (plenty of time)
    pub const TIME_WARNING: Color = Color::srgb(1.0, 0.8, 0.0);         // Yellow (30 seconds left)
    pub const TIME_CRITICAL: Color = Color::srgb(1.0, 0.2, 0.2);        // Red (10 seconds left)
    pub const TIME_OVERTIME: Color = Color::srgb(1.0, 0.0, 1.0);        // Magenta (overtime)
    
    // Urgency indicators
    pub const URGENT_PULSE: Color = Color::srgb(1.0, 0.0, 0.0);         // Pulsing red
    pub const RUSH_MODE: Color = Color::srgb(1.0, 0.5, 0.0);            // Orange rush
    pub const FINAL_SECONDS: Color = Color::srgb(1.0, 1.0, 1.0);        // White flash
}

use mclaren_colors::*;

// Card and basic game components (carried forward)
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
}

// AI Strategy System with time-pressure adaptations
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
    
    // PART 9: Time-pressure strategy modifications
    fn get_time_pressure_multiplier(&self, time_remaining_ratio: f32) -> f32 {
        // How much each strategy changes behavior under time pressure
        // time_remaining_ratio: 1.0 = full time, 0.0 = no time left
        
        match self {
            // Conservative becomes more aggressive when time is short
            AIStrategy::Conservative => {
                if time_remaining_ratio < 0.25 {
                    2.0 // Double aggression in final quarter
                } else if time_remaining_ratio < 0.5 {
                    1.5 // 50% more aggressive in second half
                } else {
                    1.0 // Normal behavior with plenty of time
                }
            },
            
            // Aggressive becomes even more reckless under pressure
            AIStrategy::Aggressive => {
                1.0 + (1.0 - time_remaining_ratio) * 2.0 // Up to 3x more aggressive
            },
            
            // Balanced adjusts proportionally to time pressure
            AIStrategy::Balanced => {
                1.0 + (1.0 - time_remaining_ratio) * 0.5 // Up to 1.5x more aggressive
            },
            
            // Adaptive changes strategy completely based on position and time
            AIStrategy::Adaptive => {
                // Complex adaptive behavior handled in decision logic
                1.0
            },
            
            // Chaos becomes even more unpredictable with time pressure
            AIStrategy::Chaos => {
                0.5 + (1.0 - time_remaining_ratio) * 2.0 // From 0.5x to 2.5x baseline
            },
        }
    }
}

// Player representation with time-pressure metrics
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerType {
    Human,
    AI(AIStrategy),
}

#[derive(Component, Debug)]
struct Player {
    id: usize,
    player_type: PlayerType,
    chips: u32,
    current_bet: u32,
    is_active: bool,
    
    // Performance metrics
    wins: u32,
    losses: u32,
    wars_won: u32,
    wars_lost: u32,
    total_winnings: i32,
    
    // PART 9: Time-pressure specific metrics
    decisions_per_minute: f32,        // Speed of decision making
    time_efficiency_score: f32,       // How well they use available time
    pressure_performance: f32,        // Performance under time pressure (0.0-1.0)
    final_push_bonus: f32,           // Extra aggression in final moments
    time_based_errors: u32,          // Mistakes made due to time pressure
    
    // Risk and momentum
    risk_factor: f32,
    momentum_score: f32,
    performance_rating: f32,
    confidence_level: f32,
    
    // Decision timing
    last_decision_time: f32,
    decision_count: u32,
    total_decision_time: f32,
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
            decisions_per_minute: 0.0,
            time_efficiency_score: 1.0,
            pressure_performance: 0.7, // Humans typically struggle under pressure
            final_push_bonus: 1.2,     // Humans can rally in final moments
            time_based_errors: 0,
            risk_factor: 0.5,
            momentum_score: 0.0,
            performance_rating: 50.0,
            confidence_level: 0.5,
            last_decision_time: 0.0,
            decision_count: 0,
            total_decision_time: 0.0,
        }
    }
    
    fn new_ai(id: usize, strategy: AIStrategy) -> Self {
        let (base_efficiency, pressure_performance, final_push) = match strategy {
            AIStrategy::Conservative => (0.8, 0.9, 1.1),  // Efficient, handles pressure well, small final push
            AIStrategy::Aggressive => (1.2, 0.6, 1.5),    // Fast decisions, struggles under pressure, big final push
            AIStrategy::Balanced => (1.0, 0.8, 1.2),      // Balanced across all metrics
            AIStrategy::Adaptive => (1.1, 0.9, 1.3),      // Very efficient, excellent under pressure, good final push
            AIStrategy::Chaos => (0.9, 0.3, 2.0),         // Somewhat efficient, terrible under pressure, huge final push
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
            decisions_per_minute: 30.0, // AI makes fast decisions
            time_efficiency_score: base_efficiency,
            pressure_performance,
            final_push_bonus: final_push,
            time_based_errors: 0,
            risk_factor: 0.5,
            momentum_score: 0.0,
            performance_rating: 50.0,
            confidence_level: 0.5,
            last_decision_time: 0.0,
            decision_count: 0,
            total_decision_time: 0.0,
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
    
    // PART 9: Time-pressure decision making
    fn calculate_time_pressure_bet(&self, base_bet: u32, time_remaining: f32, tournament_duration: f32, current_position: usize) -> u32 {
        let time_ratio = time_remaining / tournament_duration;
        
        let mut bet_multiplier = 1.0;
        
        // Apply strategy-specific time pressure
        if let PlayerType::AI(strategy) = self.player_type {
            bet_multiplier *= strategy.get_time_pressure_multiplier(time_ratio);
        }
        
        // Position-based urgency
        if current_position > 3 && time_ratio < 0.5 {
            bet_multiplier *= 1.5; // Need to catch up
        }
        
        // Final push bonus
        if time_ratio < 0.1 {
            bet_multiplier *= self.final_push_bonus;
        }
        
        // Apply pressure performance factor
        let pressure_factor = 1.0 + (1.0 - time_ratio) * (1.0 - self.pressure_performance);
        bet_multiplier *= pressure_factor;
        
        let final_bet = (base_bet as f32 * bet_multiplier) as u32;
        final_bet.min(self.chips).max(10) // Minimum bet of 10
    }
}

// PART 9: Tournament Timer System
//
// The heart of the time-pressure system - manages the countdown and triggers
// behavioral changes as time runs out.

#[derive(Resource)]
struct TournamentTimer {
    total_duration: f32,          // Total tournament time in seconds
    time_remaining: f32,          // Seconds left
    is_active: bool,              // Whether tournament is running
    is_paused: bool,              // Pause state
    warning_triggered: bool,      // 30-second warning shown
    critical_triggered: bool,     // 10-second critical warning shown
    overtime_mode: bool,          // Extended time for final hand
    final_hand_started: bool,     // Last hand is in progress
}

impl TournamentTimer {
    fn new(duration_minutes: f32) -> Self {
        Self {
            total_duration: duration_minutes * 60.0,
            time_remaining: duration_minutes * 60.0,
            is_active: false,
            is_paused: false,
            warning_triggered: false,
            critical_triggered: false,
            overtime_mode: false,
            final_hand_started: false,
        }
    }
    
    fn start(&mut self) {
        self.is_active = true;
        self.is_paused = false;
    }
    
    fn pause(&mut self) {
        self.is_paused = true;
    }
    
    fn resume(&mut self) {
        self.is_paused = false;
    }
    
    fn update(&mut self, delta_time: f32) {
        if !self.is_active || self.is_paused {
            return;
        }
        
        self.time_remaining -= delta_time;
        
        // Trigger warnings
        if !self.warning_triggered && self.time_remaining <= 30.0 {
            self.warning_triggered = true;
        }
        
        if !self.critical_triggered && self.time_remaining <= 10.0 {
            self.critical_triggered = true;
        }
        
        // Handle overtime (when time runs out during a hand)
        if self.time_remaining <= 0.0 && !self.overtime_mode {
            self.overtime_mode = true;
            self.time_remaining = 0.0;
        }
    }
    
    fn get_time_remaining_ratio(&self) -> f32 {
        if self.total_duration <= 0.0 {
            return 0.0;
        }
        (self.time_remaining / self.total_duration).max(0.0)
    }
    
    fn get_urgency_level(&self) -> UrgencyLevel {
        if self.overtime_mode {
            UrgencyLevel::Overtime
        } else if self.time_remaining <= 10.0 {
            UrgencyLevel::Critical
        } else if self.time_remaining <= 30.0 {
            UrgencyLevel::Warning
        } else {
            UrgencyLevel::Normal
        }
    }
    
    fn get_time_color(&self) -> Color {
        match self.get_urgency_level() {
            UrgencyLevel::Normal => TIME_NORMAL,
            UrgencyLevel::Warning => TIME_WARNING,
            UrgencyLevel::Critical => TIME_CRITICAL,
            UrgencyLevel::Overtime => TIME_OVERTIME,
        }
    }
    
    fn format_time(&self) -> String {
        if self.overtime_mode {
            "OVERTIME".to_string()
        } else {
            let minutes = (self.time_remaining / 60.0) as u32;
            let seconds = (self.time_remaining % 60.0) as u32;
            format!("{:02}:{:02}", minutes, seconds)
        }
    }
    
    fn is_finished(&self) -> bool {
        self.overtime_mode && self.final_hand_started
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UrgencyLevel {
    Normal,
    Warning,
    Critical,
    Overtime,
}

// PART 9: Winner-Take-All Prize System

#[derive(Resource)]
struct PrizePool {
    total_entry_fees: u32,        // Entry fee from all players
    winner_percentage: f32,       // Percentage winner takes (0.0-1.0)
    second_place_percentage: f32, // Percentage for runner-up
    third_place_percentage: f32,  // Percentage for third place
    participation_bonus: u32,     // Fixed bonus for playing
}

impl PrizePool {
    fn new(players_count: usize, entry_fee: u32) -> Self {
        Self {
            total_entry_fees: (players_count as u32) * entry_fee,
            winner_percentage: 0.6,      // Winner takes 60%
            second_place_percentage: 0.25, // Second gets 25%
            third_place_percentage: 0.15,  // Third gets 15%
            participation_bonus: 50,     // Everyone gets 50 chips for playing
        }
    }
    
    fn calculate_winnings(&self, position: usize) -> u32 {
        let percentage = match position {
            1 => self.winner_percentage,
            2 => self.second_place_percentage,
            3 => self.third_place_percentage,
            _ => 0.0,
        };
        
        let prize = (self.total_entry_fees as f32 * percentage) as u32;
        prize + self.participation_bonus
    }
    
    fn get_total_pool(&self) -> u32 {
        self.total_entry_fees + (self.participation_bonus * 6) // 6 players
    }
}

// Game state and other resources
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GamePhase {
    #[default]
    MainMenu,
    PreTournament,    // Show rules and start countdown
    Tournament,       // Active tournament with timer
    Overtime,         // Final hand in progress
    Results,          // Show final standings and prizes
    GameOver,
}

#[derive(Resource)]
struct Tournament {
    current_round: u32,
    active_players: Vec<usize>,
    eliminated_players: Vec<usize>,
    final_standings: Vec<(usize, u32)>, // (player_id, final_chips)
}

impl Tournament {
    fn new() -> Self {
        Self {
            current_round: 1,
            active_players: vec![0, 1, 2, 3, 4, 5],
            eliminated_players: Vec::new(),
            final_standings: Vec::new(),
        }
    }
    
    fn finalize_standings(&mut self, players: &[Player]) {
        self.final_standings = players.iter()
            .map(|p| (p.id, p.chips))
            .collect();
        
        // Sort by chips (descending)
        self.final_standings.sort_by(|a, b| b.1.cmp(&a.1));
    }
}

// UI Components for timer and urgency display
#[derive(Component)]
struct TimerDisplay;

#[derive(Component)]
struct UrgencyIndicator;

#[derive(Component)]
struct TimeWarning;

#[derive(Component)]
struct FinalCountdown;

#[derive(Component)]
struct PrizePoolDisplay;

#[derive(Component)]
struct StandingsDisplay;

// Visual effects for time pressure
#[derive(Component)]
struct UrgencyPulse {
    frequency: f32,
    intensity: f32,
    phase: f32,
}

#[derive(Component)]
struct CountdownFlash {
    flash_timer: f32,
    flash_duration: f32,
}

// Events
#[derive(Event)]
struct TournamentStarted;

#[derive(Event)]
struct TimeWarningTriggered;

#[derive(Event)]
struct TimeCriticalTriggered;

#[derive(Event)]
struct OvertimeEntered;

#[derive(Event)]
struct TournamentFinished {
    final_standings: Vec<(usize, u32)>,
}

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

// Resources
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
}

// Main application
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CASINO WAR - 2-Minute Blitz Tournament".into(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GamePhase>()
        .insert_resource(ClearColor(CARBON_BLACK))
        .insert_resource(Tournament::new())
        .insert_resource(Deck::new())
        .insert_resource(TournamentTimer::new(2.0)) // 2-minute tournament
        .insert_resource(PrizePool::new(6, 100))     // 6 players, 100 chip entry fee
        .add_event::<TournamentStarted>()
        .add_event::<TimeWarningTriggered>()
        .add_event::<TimeCriticalTriggered>()
        .add_event::<OvertimeEntered>()
        .add_event::<TournamentFinished>()
        .add_event::<BetPlaced>()
        .add_event::<RoundResult>()
        .add_systems(Startup, setup_tournament)
        .add_systems(Update, (
            main_menu_system.run_if(in_state(GamePhase::MainMenu)),
            pre_tournament_system.run_if(in_state(GamePhase::PreTournament)),
            tournament_timer_system.run_if(in_state(GamePhase::Tournament).or_else(in_state(GamePhase::Overtime))),
            tournament_system.run_if(in_state(GamePhase::Tournament)),
            overtime_system.run_if(in_state(GamePhase::Overtime)),
            results_system.run_if(in_state(GamePhase::Results)),
            update_timer_display,
            handle_time_warnings,
            update_urgency_effects,
            simulate_fast_gameplay, // For demonstration
            handle_button_interactions,
        ))
        .run();
}

// PART 9: Setup and Systems

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
    
    setup_main_menu(&mut commands);
}

fn setup_main_menu(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(CARBON_BLACK),
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("CASINO WAR\n2-MINUTE BLITZ TOURNAMENT"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(MCLAREN_ORANGE),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));
        
        // Rules
        parent.spawn((
            Text::new("🏆 WINNER TAKES ALL\n⏱️ 2 MINUTES ON THE CLOCK\n🚀 HIGHEST CHIPS WINS THE PRIZE"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(TEXT_PRIMARY),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        // Prize breakdown
        parent.spawn((
            Text::new("PRIZE POOL: 600 CHIPS + BONUSES\n1ST PLACE: 60% | 2ND PLACE: 25% | 3RD PLACE: 15%"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(FIRST_PLACE),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        // Start button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(400.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1)),
            BorderColor(MCLAREN_ORANGE),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("START 2-MINUTE BLITZ"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
        });
    });
}

fn main_menu_system(
    mut next_state: ResMut<NextState<GamePhase>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GamePhase::PreTournament);
        }
    }
}

fn pre_tournament_system(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut timer: ResMut<TournamentTimer>,
    menu_entities: Query<Entity, With<Node>>,
) {
    // Clear main menu
    for entity in &menu_entities {
        commands.entity(entity).despawn_recursive();
    }
    
    // Setup tournament UI
    setup_tournament_ui(&mut commands);
    
    // Start timer
    timer.start();
    
    next_state.set(GamePhase::Tournament);
}

fn setup_tournament_ui(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(CARBON_BLACK),
    ))
    .with_children(|parent| {
        // Top bar with timer and urgency indicators
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::bottom(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(PANEL_DARK),
            BorderColor(MCLAREN_ORANGE),
        ))
        .with_children(|top_bar| {
            // Tournament title
            top_bar.spawn((
                Text::new("2-MINUTE BLITZ TOURNAMENT"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(MCLAREN_ORANGE),
            ));
            
            // Timer display
            top_bar.spawn((
                Text::new("02:00"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(TIME_NORMAL),
                TimerDisplay,
                UrgencyPulse {
                    frequency: 1.0,
                    intensity: 0.2,
                    phase: 0.0,
                },
            ));
            
            // Prize pool
            top_bar.spawn((
                Text::new("PRIZE POOL: 900 CHIPS"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(FIRST_PLACE),
                PrizePoolDisplay,
            ));
        });
        
        // Main game area
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
        ))
        .with_children(|main_area| {
            // Game visualization (60%)
            main_area.spawn((
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|game_area| {
                game_area.spawn((
                    Text::new("HIGH-SPEED TOURNAMENT IN PROGRESS\nEvery second counts!"),
                    TextFont {
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                ));
            });
            
            // Leaderboard (40%)
            setup_live_leaderboard(main_area);
        });
    });
}

fn setup_live_leaderboard(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Percent(40.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            border: UiRect::left(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(PANEL_DARK),
        BorderColor(MCLAREN_ORANGE),
    ))
    .with_children(|leaderboard| {
        // Header
        leaderboard.spawn((
            Text::new("🏁 LIVE STANDINGS"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(FIRST_PLACE),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ));
        
        // Time pressure indicator
        leaderboard.spawn((
            Text::new("⚡ TIME PRESSURE: NORMAL"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(TIME_NORMAL),
            UrgencyIndicator,
            Node {
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
        ));
        
        // Standings list
        leaderboard.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(70.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            StandingsDisplay,
        ));
    });
}

// PART 9: Core Tournament Systems

fn tournament_timer_system(
    mut timer: ResMut<TournamentTimer>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut warning_events: EventWriter<TimeWarningTriggered>,
    mut critical_events: EventWriter<TimeCriticalTriggered>,
    mut overtime_events: EventWriter<OvertimeEntered>,
    mut finished_events: EventWriter<TournamentFinished>,
    time: Res<Time>,
    tournament: Res<Tournament>,
) {
    let old_urgency = timer.get_urgency_level();
    timer.update(time.delta_secs());
    let new_urgency = timer.get_urgency_level();
    
    // Trigger events on urgency level changes
    if old_urgency != new_urgency {
        match new_urgency {
            UrgencyLevel::Warning => warning_events.write(TimeWarningTriggered),
            UrgencyLevel::Critical => critical_events.write(TimeCriticalTriggered),
            UrgencyLevel::Overtime => {
                overtime_events.write(OvertimeEntered);
                next_state.set(GamePhase::Overtime);
            },
            _ => {},
        }
    }
    
    // Check if tournament should end
    if timer.is_finished() {
        finished_events.write(TournamentFinished {
            final_standings: tournament.final_standings.clone(),
        });
        next_state.set(GamePhase::Results);
    }
}

fn tournament_system(
    timer: Res<TournamentTimer>,
    mut players: Query<&mut Player>,
    mut bet_events: EventWriter<BetPlaced>,
    mut result_events: EventWriter<RoundResult>,
) {
    // Main tournament logic runs here
    // For demonstration, we'll use the fast gameplay simulation
}

fn overtime_system(
    mut timer: ResMut<TournamentTimer>,
    mut tournament: ResMut<Tournament>,
    players: Query<&Player>,
) {
    if !timer.final_hand_started {
        timer.final_hand_started = true;
        
        // Finalize standings when overtime starts
        let players_vec: Vec<_> = players.iter().collect();
        tournament.finalize_standings(&players_vec);
    }
}

fn results_system(
    mut commands: Commands,
    tournament_entities: Query<Entity, With<Node>>,
    tournament: Res<Tournament>,
    prize_pool: Res<PrizePool>,
) {
    // Clear tournament UI
    for entity in &tournament_entities {
        commands.entity(entity).despawn_recursive();
    }
    
    // Show results
    setup_results_ui(&mut commands, &tournament, &prize_pool);
}

fn setup_results_ui(commands: &mut Commands, tournament: &Tournament, prize_pool: &PrizePool) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(CARBON_BLACK),
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("🏆 TOURNAMENT COMPLETE!"),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(FIRST_PLACE),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        // Final standings and prizes
        for (position, &(player_id, chips)) in tournament.final_standings.iter().enumerate() {
            let position_num = position + 1;
            let prize = prize_pool.calculate_winnings(position_num);
            let player_name = match player_id {
                0 => "YOU".to_string(),
                id => format!("BOT {}", BOT_COLORS.get(id-1).map_or("?", |_| 
                    match id {
                        1 => "CONSERVATIVE",
                        2 => "AGGRESSIVE", 
                        3 => "BALANCED",
                        4 => "ADAPTIVE",
                        5 => "CHAOS",
                        _ => "?"
                    }
                )),
            };
            
            let color = match position_num {
                1 => FIRST_PLACE,
                2 => SECOND_PLACE,
                3 => THIRD_PLACE,
                _ => TEXT_SECONDARY,
            };
            
            parent.spawn((
                Text::new(format!("{}. {} - {} chips - Prize: {} chips", 
                    position_num, player_name, chips, prize)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(color),
                Node {
                    margin: UiRect::bottom(Val::Px(15.0)),
                    ..default()
                },
            ));
        }
    });
}

// PART 9: UI Update Systems

fn update_timer_display(
    timer: Res<TournamentTimer>,
    mut timer_query: Query<(&mut Text, &mut TextColor), With<TimerDisplay>>,
    mut urgency_query: Query<&mut Text, (With<UrgencyIndicator>, Without<TimerDisplay>)>,
) {
    // Update timer display
    for (mut text, mut color) in &mut timer_query {
        **text = timer.format_time();
        color.0 = timer.get_time_color();
    }
    
    // Update urgency indicator
    for mut text in &mut urgency_query {
        let urgency_text = match timer.get_urgency_level() {
            UrgencyLevel::Normal => "⚡ TIME PRESSURE: NORMAL",
            UrgencyLevel::Warning => "⚠️ TIME PRESSURE: WARNING",
            UrgencyLevel::Critical => "🚨 TIME PRESSURE: CRITICAL",
            UrgencyLevel::Overtime => "💥 OVERTIME MODE",
        };
        **text = urgency_text.to_string();
    }
}

fn handle_time_warnings(
    mut warning_events: EventReader<TimeWarningTriggered>,
    mut critical_events: EventReader<TimeCriticalTriggered>,
    mut overtime_events: EventReader<OvertimeEntered>,
) {
    for _ in warning_events.read() {
        info!("⚠️ 30 seconds remaining!");
    }
    
    for _ in critical_events.read() {
        info!("🚨 10 seconds remaining!");
    }
    
    for _ in overtime_events.read() {
        info!("💥 Overtime! Final hand in progress!");
    }
}

fn update_urgency_effects(
    mut urgency_query: Query<(&mut UrgencyPulse, &mut TextColor)>,
    timer: Res<TournamentTimer>,
    time: Res<Time>,
) {
    let urgency_level = timer.get_urgency_level();
    
    for (mut pulse, mut color) in &mut urgency_query {
        pulse.phase += time.delta_secs() * pulse.frequency;
        
        let pulse_intensity = match urgency_level {
            UrgencyLevel::Normal => 0.0,
            UrgencyLevel::Warning => 0.3,
            UrgencyLevel::Critical => 0.6,
            UrgencyLevel::Overtime => 1.0,
        };
        
        let pulse_factor = (pulse.phase.sin() * 0.5 + 0.5) * pulse_intensity;
        let base_color = timer.get_time_color();
        
        // Pulse between base color and white
        color.0 = Color::srgb(
            base_color.to_srgba().red + pulse_factor * (1.0 - base_color.to_srgba().red),
            base_color.to_srgba().green + pulse_factor * (1.0 - base_color.to_srgba().green),
            base_color.to_srgba().blue + pulse_factor * (1.0 - base_color.to_srgba().blue),
        );
    }
}

// Fast-paced gameplay simulation for demonstration
fn simulate_fast_gameplay(
    timer: Res<TournamentTimer>,
    mut players: Query<&mut Player>,
    mut bet_events: EventWriter<BetPlaced>,
    mut result_events: EventWriter<RoundResult>,
    time: Res<Time>,
) {
    if !timer.is_active || timer.is_paused {
        return;
    }
    
    static mut LAST_ACTION: f32 = 0.0;
    let current_time = time.elapsed_secs();
    
    unsafe {
        // Much faster action in timed tournament (every 0.5 seconds)
        if current_time - LAST_ACTION > 0.5 {
            LAST_ACTION = current_time;
            
            let mut rng = thread_rng();
            for mut player in &mut players {
                if player.is_active && player.chips > 10 {
                    // Time-pressure modified betting
                    let base_bet = rng.gen_range(10..=50).min(player.chips);
                    let time_pressure_bet = player.calculate_time_pressure_bet(
                        base_bet,
                        timer.time_remaining,
                        timer.total_duration,
                        1 // Position placeholder
                    );
                    
                    bet_events.write(BetPlaced {
                        player_id: player.id,
                        amount: time_pressure_bet,
                    });
                    
                    // Fast results
                    let won = rng.gen_bool(0.5);
                    let amount_won = if won {
                        time_pressure_bet as i32
                    } else {
                        -(time_pressure_bet as i32)
                    };
                    
                    result_events.write(RoundResult {
                        player_id: player.id,
                        amount_won,
                        was_war: rng.gen_bool(0.1),
                    });
                    
                    // Update player
                    if won {
                        player.chips += time_pressure_bet;
                        player.wins += 1;
                    } else {
                        player.chips = player.chips.saturating_sub(time_pressure_bet);
                        player.losses += 1;
                    }
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
            if let Ok(mut text_color) = text_query.get_mut(*child) {
                *text_color = TextColor(color);
            }
        }
    }
}