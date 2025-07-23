// Casino War Part 10: Advanced Betting Analytics - The Data-Driven Edge
//
// This final part completes our Casino War journey by adding sophisticated
// analytics that transform the game into a data-rich competitive environment.
// Players can analyze patterns, optimize strategies, and gain insights into
// their performance across multiple dimensions.
//
// Key concepts we'll explore:
// 1. Advanced Analytics Engine - Multi-dimensional performance analysis
// 2. Pattern Recognition - Identifying trends and behavioral signatures
// 3. Strategy Optimization - Real-time strategy effectiveness scoring
// 4. Predictive Modeling - Forecasting outcomes based on historical data
// 5. Data Visualization - Interactive charts and performance dashboards
// 6. Machine Learning Concepts - Adaptive algorithms that learn from play

use bevy::prelude::*;
use rand::prelude::*;
use std::collections::{HashMap, VecDeque};

// PART 10: McLaren-inspired color palette with analytics visualization colors
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
    
    // Time-pressure colors
    pub const TIME_NORMAL: Color = Color::srgb(0.0, 1.0, 0.4);          // Green
    pub const TIME_WARNING: Color = Color::srgb(1.0, 0.8, 0.0);         // Yellow
    pub const TIME_CRITICAL: Color = Color::srgb(1.0, 0.2, 0.2);        // Red
    
    // PART 10: Analytics and data visualization colors
    pub const CHART_POSITIVE: Color = Color::srgb(0.0, 0.8, 0.4);       // Success green
    pub const CHART_NEGATIVE: Color = Color::srgb(0.9, 0.3, 0.3);       // Loss red
    pub const CHART_NEUTRAL: Color = Color::srgb(0.6, 0.6, 0.6);        // Neutral gray
    pub const CHART_TREND_UP: Color = Color::srgb(0.2, 1.0, 0.2);       // Bright green
    pub const CHART_TREND_DOWN: Color = Color::srgb(1.0, 0.2, 0.2);     // Bright red
    
    // Analytics panel colors
    pub const DATA_EXCELLENT: Color = Color::srgb(0.0, 1.0, 0.8);       // Cyan-green
    pub const DATA_GOOD: Color = Color::srgb(0.4, 0.8, 0.4);            // Good green
    pub const DATA_AVERAGE: Color = Color::srgb(0.8, 0.8, 0.4);         // Average yellow
    pub const DATA_POOR: Color = Color::srgb(0.8, 0.4, 0.4);            // Poor orange-red
    pub const DATA_TERRIBLE: Color = Color::srgb(1.0, 0.2, 0.2);        // Terrible red
    
    // Machine learning indicators
    pub const ML_LEARNING: Color = Color::srgb(0.6, 0.8, 1.0);          // Learning blue
    pub const ML_ADAPTING: Color = Color::srgb(1.0, 0.6, 0.8);          // Adapting pink
    pub const ML_OPTIMIZED: Color = Color::srgb(0.8, 1.0, 0.6);         // Optimized green
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
}

// AI Strategy System with analytics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

// Player representation with comprehensive analytics
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerType {
    Human,
    AI(AIStrategy),
}

// PART 10: Advanced Analytics Data Structures
//
// These structures capture every aspect of player behavior for deep analysis

#[derive(Debug, Clone)]
struct BettingPattern {
    bet_size_distribution: [u32; 10],  // Distribution across bet size ranges
    war_acceptance_rate: f32,          // Percentage of wars accepted
    risk_escalation_rate: f32,         // How quickly bets increase under pressure
    consistency_score: f32,            // How consistent betting patterns are
    adaptability_index: f32,           // How much strategy changes over time
}

impl BettingPattern {
    fn new() -> Self {
        Self {
            bet_size_distribution: [0; 10],
            war_acceptance_rate: 0.0,
            risk_escalation_rate: 0.0,
            consistency_score: 1.0,
            adaptability_index: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
struct PerformanceMetrics {
    roi: f32,                          // Return on investment
    sharpe_ratio: f32,                 // Risk-adjusted return
    maximum_drawdown: f32,             // Largest peak-to-trough decline
    win_loss_ratio: f32,               // Average win / average loss
    profit_factor: f32,                // Gross profit / gross loss
    expectancy: f32,                   // Expected value per bet
    kelly_criterion: f32,              // Optimal bet size percentage
    var_95: f32,                       // Value at Risk (95% confidence)
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            roi: 0.0,
            sharpe_ratio: 0.0,
            maximum_drawdown: 0.0,
            win_loss_ratio: 1.0,
            profit_factor: 1.0,
            expectancy: 0.0,
            kelly_criterion: 0.0,
            var_95: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
struct PsychologicalProfile {
    tilt_resistance: f32,              // Resistance to emotional decisions
    pressure_response: f32,            // Performance under time pressure
    risk_tolerance: f32,               // Willingness to take risks
    confidence_level: f32,             // Current confidence state
    emotional_stability: f32,          // Consistency across emotional states
    learning_rate: f32,                // How quickly they adapt
    pattern_recognition: f32,          // Ability to recognize game patterns
}

impl PsychologicalProfile {
    fn new(player_type: PlayerType) -> Self {
        match player_type {
            PlayerType::Human => Self {
                tilt_resistance: 0.6,
                pressure_response: 0.5,
                risk_tolerance: 0.5,
                confidence_level: 0.5,
                emotional_stability: 0.7,
                learning_rate: 0.8,
                pattern_recognition: 0.6,
            },
            PlayerType::AI(strategy) => {
                let (tilt, pressure, risk, confidence, stability, learning, pattern) = match strategy {
                    AIStrategy::Conservative => (0.9, 0.8, 0.2, 0.6, 0.9, 0.3, 0.7),
                    AIStrategy::Aggressive => (0.3, 0.4, 0.9, 0.8, 0.4, 0.2, 0.5),
                    AIStrategy::Balanced => (0.7, 0.7, 0.5, 0.6, 0.8, 0.6, 0.8),
                    AIStrategy::Adaptive => (0.8, 0.9, 0.6, 0.7, 0.7, 0.9, 0.9),
                    AIStrategy::Chaos => (0.1, 0.2, 0.8, 0.9, 0.2, 0.1, 0.3),
                };
                Self { tilt_resistance: tilt, pressure_response: pressure, risk_tolerance: risk,
                      confidence_level: confidence, emotional_stability: stability, 
                      learning_rate: learning, pattern_recognition: pattern }
            }
        }
    }
}

#[derive(Component, Debug, Clone)]
struct Player {
    id: usize,
    player_type: PlayerType,
    chips: u32,
    current_bet: u32,
    is_active: bool,
    
    // Basic performance
    wins: u32,
    losses: u32,
    wars_won: u32,
    wars_lost: u32,
    total_winnings: i32,
    
    // PART 10: Advanced analytics components
    betting_pattern: BettingPattern,
    performance_metrics: PerformanceMetrics,
    psychological_profile: PsychologicalProfile,
    
    // Historical data for analysis
    bet_history: VecDeque<u32>,            // Last 100 bets
    result_history: VecDeque<i32>,         // Last 100 results
    decision_times: VecDeque<f32>,         // Last 100 decision times
    chip_history: VecDeque<u32>,           // Chip count over time
    
    // Real-time analytics
    current_streak: i32,                   // Current win/loss streak
    peak_chips: u32,                       // Highest chip count achieved
    valley_chips: u32,                     // Lowest chip count reached
    volatility: f32,                       // Standard deviation of results
    momentum: f32,                         // Current momentum indicator
    efficiency_rating: f32,                // Overall efficiency score
    
    // Strategy adaptation
    strategy_confidence: f32,              // Confidence in current strategy
    adaptation_trigger: f32,               // Threshold for changing strategy
    learning_progress: f32,                // How much the player has learned
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
            betting_pattern: BettingPattern::new(),
            performance_metrics: PerformanceMetrics::new(),
            psychological_profile: PsychologicalProfile::new(PlayerType::Human),
            bet_history: VecDeque::with_capacity(100),
            result_history: VecDeque::with_capacity(100),
            decision_times: VecDeque::with_capacity(100),
            chip_history: VecDeque::with_capacity(100),
            current_streak: 0,
            peak_chips: 1000,
            valley_chips: 1000,
            volatility: 0.0,
            momentum: 0.0,
            efficiency_rating: 50.0,
            strategy_confidence: 0.5,
            adaptation_trigger: 0.3,
            learning_progress: 0.0,
        }
    }
    
    fn new_ai(id: usize, strategy: AIStrategy) -> Self {
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
            betting_pattern: BettingPattern::new(),
            performance_metrics: PerformanceMetrics::new(),
            psychological_profile: PsychologicalProfile::new(PlayerType::AI(strategy)),
            bet_history: VecDeque::with_capacity(100),
            result_history: VecDeque::with_capacity(100),
            decision_times: VecDeque::with_capacity(100),
            chip_history: VecDeque::with_capacity(100),
            current_streak: 0,
            peak_chips: 1000,
            valley_chips: 1000,
            volatility: 0.0,
            momentum: 0.0,
            efficiency_rating: 50.0,
            strategy_confidence: 0.8, // AIs start more confident
            adaptation_trigger: 0.2,  // AIs adapt more readily
            learning_progress: 0.0,
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
    
    // PART 10: Advanced Analytics Methods
    
    fn record_bet(&mut self, amount: u32, decision_time: f32) {
        // Record bet in history
        if self.bet_history.len() >= 100 {
            self.bet_history.pop_front();
        }
        self.bet_history.push_back(amount);
        
        // Record decision time
        if self.decision_times.len() >= 100 {
            self.decision_times.pop_front();
        }
        self.decision_times.push_back(decision_time);
        
        // Update betting pattern
        self.update_betting_pattern(amount);
    }
    
    fn record_result(&mut self, result: i32, was_war: bool) {
        // Update basic stats
        if result > 0 {
            self.wins += 1;
            self.current_streak = if self.current_streak >= 0 { self.current_streak + 1 } else { 1 };
            if was_war { self.wars_won += 1; }
        } else {
            self.losses += 1;
            self.current_streak = if self.current_streak <= 0 { self.current_streak - 1 } else { -1 };
            if was_war { self.wars_lost += 1; }
        }
        
        // Update chip count
        self.chips = ((self.chips as i32) + result).max(0) as u32;
        self.total_winnings += result;
        
        // Track peaks and valleys
        self.peak_chips = self.peak_chips.max(self.chips);
        self.valley_chips = self.valley_chips.min(self.chips);
        
        // Record in history
        if self.result_history.len() >= 100 {
            self.result_history.pop_front();
        }
        self.result_history.push_back(result);
        
        if self.chip_history.len() >= 100 {
            self.chip_history.pop_front();
        }
        self.chip_history.push_back(self.chips);
        
        // Update analytics
        self.update_performance_metrics();
        self.update_psychological_state(result);
        self.update_volatility();
        self.update_momentum();
        self.update_efficiency_rating();
    }
    
    fn update_betting_pattern(&mut self, bet_amount: u32) {
        // Update bet size distribution
        let bet_range = (bet_amount / 10).min(9) as usize;
        self.betting_pattern.bet_size_distribution[bet_range] += 1;
        
        // Calculate consistency score
        if self.bet_history.len() > 10 {
            let mean_bet: f32 = self.bet_history.iter().map(|&b| b as f32).sum::<f32>() / self.bet_history.len() as f32;
            let variance: f32 = self.bet_history.iter()
                .map(|&b| (b as f32 - mean_bet).powi(2))
                .sum::<f32>() / self.bet_history.len() as f32;
            let std_dev = variance.sqrt();
            self.betting_pattern.consistency_score = 1.0 - (std_dev / mean_bet).min(1.0);
        }
    }
    
    fn update_performance_metrics(&mut self) {
        if self.bet_history.is_empty() || self.result_history.is_empty() {
            return;
        }
        
        // Calculate ROI
        let total_invested: u32 = self.bet_history.iter().sum();
        if total_invested > 0 {
            self.performance_metrics.roi = (self.total_winnings as f32) / (total_invested as f32);
        }
        
        // Calculate win/loss ratio
        let wins: i32 = self.result_history.iter().filter(|&&r| r > 0).map(|&r| r).sum();
        let losses: i32 = self.result_history.iter().filter(|&&r| r < 0).map(|&r| -r).sum();
        if losses > 0 {
            self.performance_metrics.win_loss_ratio = (wins as f32) / (losses as f32);
        }
        
        // Calculate expectancy
        let total_results = self.result_history.len();
        if total_results > 0 {
            let sum_results: i32 = self.result_history.iter().sum();
            self.performance_metrics.expectancy = (sum_results as f32) / (total_results as f32);
        }
        
        // Calculate maximum drawdown
        self.performance_metrics.maximum_drawdown = 
            1.0 - (self.valley_chips as f32 / self.peak_chips as f32);
    }
    
    fn update_psychological_state(&mut self, result: i32) {
        // Update confidence based on recent results
        let confidence_adjustment = if result > 0 { 0.02 } else { -0.03 };
        self.psychological_profile.confidence_level = 
            (self.psychological_profile.confidence_level + confidence_adjustment).clamp(0.0, 1.0);
        
        // Update strategy confidence
        if result > 0 {
            self.strategy_confidence = (self.strategy_confidence + 0.01).min(1.0);
        } else {
            self.strategy_confidence = (self.strategy_confidence - 0.02).max(0.0);
        }
        
        // Check for adaptation trigger
        if self.strategy_confidence < self.adaptation_trigger {
            self.trigger_adaptation();
        }
    }
    
    fn update_volatility(&mut self) {
        if self.result_history.len() < 10 {
            return;
        }
        
        let mean: f32 = self.result_history.iter().map(|&r| r as f32).sum::<f32>() / self.result_history.len() as f32;
        let variance: f32 = self.result_history.iter()
            .map(|&r| (r as f32 - mean).powi(2))
            .sum::<f32>() / self.result_history.len() as f32;
        
        self.volatility = variance.sqrt();
    }
    
    fn update_momentum(&mut self) {
        if self.result_history.len() < 10 {
            return;
        }
        
        // Calculate momentum based on recent results trend
        let recent_results: Vec<f32> = self.result_history.iter()
            .rev()
            .take(10)
            .map(|&r| r as f32)
            .collect();
        
        let mut momentum_sum = 0.0;
        let mut weight = 1.0;
        
        for result in recent_results {
            momentum_sum += result * weight;
            weight *= 0.9; // Decay weight for older results
        }
        
        self.momentum = momentum_sum.tanh(); // Normalize to -1.0 to 1.0
    }
    
    fn update_efficiency_rating(&mut self) {
        let total_games = self.wins + self.losses;
        if total_games == 0 {
            return;
        }
        
        // Multi-factor efficiency calculation
        let win_rate = (self.wins as f32) / (total_games as f32);
        let profit_efficiency = if self.total_winnings > 0 { 1.0 } else { 0.0 };
        let risk_efficiency = 1.0 - self.volatility / 100.0; // Normalize volatility
        let consistency_bonus = self.betting_pattern.consistency_score;
        
        self.efficiency_rating = ((win_rate * 30.0) + 
                                 (profit_efficiency * 25.0) + 
                                 (risk_efficiency * 25.0) + 
                                 (consistency_bonus * 20.0)).min(100.0);
    }
    
    fn trigger_adaptation(&mut self) {
        // Increase learning progress
        self.learning_progress += 0.1;
        
        // Reset strategy confidence
        self.strategy_confidence = 0.6;
        
        // Adjust psychological profile based on learning
        let learning_factor = self.learning_progress.min(1.0);
        self.psychological_profile.pattern_recognition = 
            (self.psychological_profile.pattern_recognition + learning_factor * 0.1).min(1.0);
    }
    
    // Analytics query methods
    fn get_performance_grade(&self) -> PerformanceGrade {
        match self.efficiency_rating {
            90.0..=100.0 => PerformanceGrade::Excellent,
            75.0..=89.9 => PerformanceGrade::Good,
            50.0..=74.9 => PerformanceGrade::Average,
            25.0..=49.9 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Terrible,
        }
    }
    
    fn get_risk_profile(&self) -> RiskProfile {
        match self.psychological_profile.risk_tolerance {
            0.8..=1.0 => RiskProfile::VeryAggressive,
            0.6..=0.79 => RiskProfile::Aggressive,
            0.4..=0.59 => RiskProfile::Moderate,
            0.2..=0.39 => RiskProfile::Conservative,
            _ => RiskProfile::VeryConservative,
        }
    }
    
    fn get_adaptation_status(&self) -> AdaptationStatus {
        if self.learning_progress > 0.8 {
            AdaptationStatus::HighlyAdapted
        } else if self.learning_progress > 0.5 {
            AdaptationStatus::WellAdapted
        } else if self.learning_progress > 0.2 {
            AdaptationStatus::Adapting
        } else {
            AdaptationStatus::Learning
        }
    }
}

// PART 10: Analytics Classification Systems

#[derive(Debug, Clone, Copy, PartialEq)]
enum PerformanceGrade {
    Excellent,
    Good,
    Average,
    Poor,
    Terrible,
}

impl PerformanceGrade {
    fn get_color(&self) -> Color {
        match self {
            PerformanceGrade::Excellent => DATA_EXCELLENT,
            PerformanceGrade::Good => DATA_GOOD,
            PerformanceGrade::Average => DATA_AVERAGE,
            PerformanceGrade::Poor => DATA_POOR,
            PerformanceGrade::Terrible => DATA_TERRIBLE,
        }
    }
    
    fn get_label(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "EXCELLENT",
            PerformanceGrade::Good => "GOOD",
            PerformanceGrade::Average => "AVERAGE",
            PerformanceGrade::Poor => "POOR",
            PerformanceGrade::Terrible => "TERRIBLE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RiskProfile {
    VeryAggressive,
    Aggressive,
    Moderate,
    Conservative,
    VeryConservative,
}

impl RiskProfile {
    fn get_label(&self) -> &'static str {
        match self {
            RiskProfile::VeryAggressive => "VERY AGGRESSIVE",
            RiskProfile::Aggressive => "AGGRESSIVE",
            RiskProfile::Moderate => "MODERATE",
            RiskProfile::Conservative => "CONSERVATIVE",
            RiskProfile::VeryConservative => "VERY CONSERVATIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AdaptationStatus {
    Learning,
    Adapting,
    WellAdapted,
    HighlyAdapted,
}

impl AdaptationStatus {
    fn get_color(&self) -> Color {
        match self {
            AdaptationStatus::Learning => ML_LEARNING,
            AdaptationStatus::Adapting => ML_ADAPTING,
            AdaptationStatus::WellAdapted => ML_OPTIMIZED,
            AdaptationStatus::HighlyAdapted => DATA_EXCELLENT,
        }
    }
    
    fn get_label(&self) -> &'static str {
        match self {
            AdaptationStatus::Learning => "LEARNING",
            AdaptationStatus::Adapting => "ADAPTING",
            AdaptationStatus::WellAdapted => "WELL ADAPTED",
            AdaptationStatus::HighlyAdapted => "HIGHLY ADAPTED",
        }
    }
}

// PART 10: Analytics Engine Resource
//
// Central analytics processor that runs machine learning algorithms
// and generates insights across all players.

#[derive(Resource)]
struct AnalyticsEngine {
    global_patterns: HashMap<String, f32>,      // Discovered patterns across all players
    market_trends: VecDeque<f32>,               // Overall market trend data
    prediction_models: HashMap<usize, f32>,     // Predictive models per player
    anomaly_detection: HashMap<usize, f32>,     // Anomaly scores per player
    strategy_effectiveness: HashMap<AIStrategy, f32>, // Strategy performance rankings
    learning_curves: HashMap<usize, VecDeque<f32>>,   // Learning progress per player
    optimization_suggestions: HashMap<usize, String>, // AI-generated suggestions
}

impl AnalyticsEngine {
    fn new() -> Self {
        Self {
            global_patterns: HashMap::new(),
            market_trends: VecDeque::with_capacity(1000),
            prediction_models: HashMap::new(),
            anomaly_detection: HashMap::new(),
            strategy_effectiveness: HashMap::new(),
            learning_curves: HashMap::new(),
            optimization_suggestions: HashMap::new(),
        }
    }
    
    fn update_global_patterns(&mut self, players: &[Player]) {
        // Analyze cross-player patterns
        let total_players = players.len();
        if total_players == 0 { return; }
        
        // Calculate market volatility
        let avg_volatility: f32 = players.iter().map(|p| p.volatility).sum::<f32>() / total_players as f32;
        self.global_patterns.insert("market_volatility".to_string(), avg_volatility);
        
        // Calculate market momentum
        let avg_momentum: f32 = players.iter().map(|p| p.momentum).sum::<f32>() / total_players as f32;
        self.global_patterns.insert("market_momentum".to_string(), avg_momentum);
        
        // Record market trend
        if self.market_trends.len() >= 1000 {
            self.market_trends.pop_front();
        }
        self.market_trends.push_back(avg_momentum);
    }
    
    fn update_strategy_effectiveness(&mut self, players: &[Player]) {
        // Clear previous data
        self.strategy_effectiveness.clear();
        
        // Group players by strategy and calculate average performance
        for strategy in [AIStrategy::Conservative, AIStrategy::Aggressive, AIStrategy::Balanced, 
                        AIStrategy::Adaptive, AIStrategy::Chaos] {
            let strategy_players: Vec<_> = players.iter()
                .filter(|p| matches!(p.player_type, PlayerType::AI(s) if s == strategy))
                .collect();
            
            if !strategy_players.is_empty() {
                let avg_efficiency: f32 = strategy_players.iter()
                    .map(|p| p.efficiency_rating)
                    .sum::<f32>() / strategy_players.len() as f32;
                
                self.strategy_effectiveness.insert(strategy, avg_efficiency);
            }
        }
    }
    
    fn generate_predictions(&mut self, players: &[Player]) {
        for player in players {
            if player.result_history.len() < 20 {
                continue;
            }
            
            // Simple linear regression for next result prediction
            let results: Vec<f32> = player.result_history.iter().map(|&r| r as f32).collect();
            let n = results.len() as f32;
            let x_sum: f32 = (0..results.len()).map(|i| i as f32).sum();
            let y_sum: f32 = results.iter().sum();
            let xy_sum: f32 = results.iter().enumerate().map(|(i, &y)| i as f32 * y).sum();
            let x2_sum: f32 = (0..results.len()).map(|i| (i as f32).powi(2)).sum();
            
            let slope = (n * xy_sum - x_sum * y_sum) / (n * x2_sum - x_sum.powi(2));
            let prediction = slope * n + (y_sum - slope * x_sum) / n;
            
            self.prediction_models.insert(player.id, prediction);
        }
    }
    
    fn detect_anomalies(&mut self, players: &[Player]) {
        for player in players {
            let mut anomaly_score = 0.0;
            
            // Check for unusual betting patterns
            if player.betting_pattern.consistency_score < 0.3 {
                anomaly_score += 0.3;
            }
            
            // Check for extreme volatility
            if player.volatility > 50.0 {
                anomaly_score += 0.4;
            }
            
            // Check for unusual win/loss streaks
            if player.current_streak.abs() > 10 {
                anomaly_score += 0.3;
            }
            
            self.anomaly_detection.insert(player.id, anomaly_score);
        }
    }
    
    fn generate_optimization_suggestions(&mut self, players: &[Player]) {
        for player in players {
            let suggestion = match player.get_performance_grade() {
                PerformanceGrade::Excellent => "Maintain current strategy - excellent performance!".to_string(),
                PerformanceGrade::Good => "Consider slight risk adjustment to maximize gains".to_string(),
                PerformanceGrade::Average => "Analyze betting patterns for optimization opportunities".to_string(),
                PerformanceGrade::Poor => "Reduce risk and focus on consistency".to_string(),
                PerformanceGrade::Terrible => "Major strategy revision needed - consider conservative approach".to_string(),
            };
            
            self.optimization_suggestions.insert(player.id, suggestion);
        }
    }
    
    fn run_full_analysis(&mut self, players: &[Player]) {
        self.update_global_patterns(players);
        self.update_strategy_effectiveness(players);
        self.generate_predictions(players);
        self.detect_anomalies(players);
        self.generate_optimization_suggestions(players);
    }
}

// Game state and resources (simplified for analytics focus)
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GamePhase {
    #[default]
    MainMenu,
    AnalyticsDashboard,
    Tournament,
    GameOver,
}

#[derive(Resource)]
struct Tournament {
    current_round: u32,
    active_players: Vec<usize>,
}

impl Tournament {
    fn new() -> Self {
        Self {
            current_round: 1,
            active_players: vec![0, 1, 2, 3, 4, 5],
        }
    }
}

// UI Components for analytics dashboard
#[derive(Component)]
struct AnalyticsDashboard;

#[derive(Component)]
struct PerformanceChart {
    player_id: usize,
}

#[derive(Component)]
struct StrategyEffectivenessDisplay;

#[derive(Component)]
struct PredictionDisplay {
    player_id: usize,
}

#[derive(Component)]
struct AnomalyAlert {
    player_id: usize,
}

#[derive(Component)]
struct OptimizationSuggestion {
    player_id: usize,
}

// Events
#[derive(Event)]
struct AnalyticsUpdated;

#[derive(Event)]
struct BetPlaced {
    player_id: usize,
    amount: u32,
    decision_time: f32,
}

#[derive(Event)]
struct RoundResult {
    player_id: usize,
    amount_won: i32,
    was_war: bool,
}

// Main application
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CASINO WAR - Advanced Analytics Suite".into(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GamePhase>()
        .insert_resource(ClearColor(CARBON_BLACK))
        .insert_resource(Tournament::new())
        .insert_resource(AnalyticsEngine::new())
        .add_event::<AnalyticsUpdated>()
        .add_event::<BetPlaced>()
        .add_event::<RoundResult>()
        .add_systems(Startup, setup_analytics_suite)
        .add_systems(Update, (
            main_menu_system.run_if(in_state(GamePhase::MainMenu)),
            analytics_dashboard_system.run_if(in_state(GamePhase::AnalyticsDashboard)),
            analytics_update_system,
            simulate_analytics_data, // For demonstration
            handle_button_interactions,
        ))
        .run();
}

// PART 10: Setup and Systems

fn setup_analytics_suite(mut commands: Commands) {
    // Create camera
    commands.spawn((
        Camera2d,
        Transform::from_xyz(0.0, 0.0, 1000.0),
    ));
    
    // Create players with analytics
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
            Text::new("CASINO WAR\nADVANCED ANALYTICS SUITE"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(MCLAREN_ORANGE),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        // Features
        parent.spawn((
            Text::new("📊 REAL-TIME PERFORMANCE ANALYTICS\n🤖 MACHINE LEARNING INSIGHTS\n📈 PREDICTIVE MODELING\n🎯 STRATEGY OPTIMIZATION"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(DATA_EXCELLENT),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        // Launch button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(400.0),
                height: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(DATA_EXCELLENT.with_alpha(0.1)),
            BorderColor(DATA_EXCELLENT),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("LAUNCH ANALYTICS DASHBOARD"),
                TextFont {
                    font_size: 28.0,
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
            next_state.set(GamePhase::AnalyticsDashboard);
        }
    }
}

fn analytics_dashboard_system(
    mut commands: Commands,
    dashboard_query: Query<Entity, With<AnalyticsDashboard>>,
    menu_entities: Query<Entity, With<Node>>,
) {
    // Clear main menu if dashboard doesn't exist
    if dashboard_query.is_empty() {
        for entity in &menu_entities {
            commands.entity(entity).despawn();
        }
        setup_analytics_dashboard(&mut commands);
    }
}

fn setup_analytics_dashboard(commands: &mut Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(CARBON_BLACK),
        AnalyticsDashboard,
    ))
    .with_children(|parent| {
        // Header
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(PANEL_DARK),
            BorderColor(DATA_EXCELLENT),
        ))
        .with_children(|header| {
            header.spawn((
                Text::new("🧠 ADVANCED ANALYTICS DASHBOARD"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(DATA_EXCELLENT),
            ));
            
            header.spawn((
                Text::new("MACHINE LEARNING ACTIVE"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(ML_LEARNING),
            ));
        });
        
        // Main dashboard area
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
        ))
        .with_children(|main_area| {
            // Left panel: Player analytics
            main_area.spawn((
                Node {
                    width: Val::Percent(33.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(15.0)),
                    border: UiRect::right(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(PANEL_DARK),
                BorderColor(DATA_GOOD),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("👤 PLAYER PERFORMANCE"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(DATA_GOOD),
                    Node {
                        margin: UiRect::bottom(Val::Px(15.0)),
                        ..default()
                    },
                ));
                
                // Player performance summary
                panel.spawn((
                    Text::new("Real-time performance analytics\nshowing efficiency ratings,\nrisk profiles, and adaptation\nstatus for all players."),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(TEXT_SECONDARY),
                ));
            });
            
            // Center panel: Global analytics
            main_area.spawn((
                Node {
                    width: Val::Percent(34.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(15.0)),
                    border: UiRect::right(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(PANEL_DARK),
                BorderColor(DATA_AVERAGE),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("🌐 GLOBAL PATTERNS"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(DATA_AVERAGE),
                    Node {
                        margin: UiRect::bottom(Val::Px(15.0)),
                        ..default()
                    },
                ));
                
                // Global analytics display
                panel.spawn((
                    Text::new("Market volatility analysis,\nstrategy effectiveness rankings,\nand cross-player pattern\nrecognition running in real-time."),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(TEXT_SECONDARY),
                ));
            });
            
            // Right panel: Predictions and suggestions
            main_area.spawn((
                Node {
                    width: Val::Percent(33.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(15.0)),
                    ..default()
                },
                BackgroundColor(PANEL_DARK),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("🔮 PREDICTIONS & AI"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(ML_OPTIMIZED),
                    Node {
                        margin: UiRect::bottom(Val::Px(15.0)),
                        ..default()
                    },
                ));
                
                // AI predictions display
                panel.spawn((
                    Text::new("Machine learning predictions,\nstrategy suggestions, and\nprobabilistic modeling\nwith real-time updates."),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(TEXT_SECONDARY),
                ));
            });
        });
    });
}


fn analytics_update_system(
    mut analytics: ResMut<AnalyticsEngine>,
    players: Query<&Player>,
    mut analytics_events: EventWriter<AnalyticsUpdated>,
    time: Res<Time>,
) {
    static mut LAST_UPDATE: f32 = 0.0;
    let current_time = time.elapsed_secs();
    
    unsafe {
        if current_time - LAST_UPDATE > 2.0 { // Update every 2 seconds
            LAST_UPDATE = current_time;
            
            let players_vec: Vec<Player> = players.iter().cloned().collect();
            analytics.run_full_analysis(&players_vec);
            analytics_events.write(AnalyticsUpdated);
        }
    }
}

fn simulate_analytics_data(
    mut players: Query<&mut Player>,
    mut bet_events: EventWriter<BetPlaced>,
    mut result_events: EventWriter<RoundResult>,
    time: Res<Time>,
) {
    static mut LAST_SIMULATION: f32 = 0.0;
    let current_time = time.elapsed_secs();
    
    unsafe {
        if current_time - LAST_SIMULATION > 1.0 {
            LAST_SIMULATION = current_time;
            
            let mut rng = thread_rng();
            for mut player in &mut players {
                if player.chips > 10 {
                    // Generate analytics data
                    let bet_amount = rng.gen_range(10..=100).min(player.chips);
                    let decision_time = rng.gen_range(0.1..2.0);
                    
                    bet_events.write(BetPlaced {
                        player_id: player.id,
                        amount: bet_amount,
                        decision_time,
                    });
                    
                    let won = rng.gen_bool(0.5);
                    let result = if won { bet_amount as i32 } else { -(bet_amount as i32) };
                    
                    result_events.write(RoundResult {
                        player_id: player.id,
                        amount_won: result,
                        was_war: rng.gen_bool(0.1),
                    });
                    
                    // Update player with analytics
                    player.record_bet(bet_amount, decision_time);
                    player.record_result(result, false);
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
            Interaction::Pressed => DATA_EXCELLENT,
            Interaction::Hovered => ML_LEARNING,
            Interaction::None => TEXT_PRIMARY,
        };
        
        for child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                *text_color = TextColor(color);
            }
        }
    }
}