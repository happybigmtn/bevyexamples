# Casino War Tutorial - Part 8: Real-time Leaderboard - The Competitive Edge

## What We're Building in Part 8

Transforming our tournament into a data-driven competitive arena with live analytics:

1. **Real-time Rankings**: Live leaderboard that updates with every hand
2. **Performance Metrics**: Multi-dimensional statistics (win rate, efficiency, momentum)
3. **Trend Analysis**: Visual indicators showing player trajectory (rising/falling)
4. **Strategic Intelligence**: Position-aware insights for competitive advantage
5. **Live Tournament Visualization**: Dynamic UI that brings the competition to life

## Understanding Real-time Data Systems

### The Live Leaderboard Problem

Imagine you're building a stock market ticker that updates in real-time. We need a leaderboard that:
- Updates instantly as game state changes
- Shows multiple performance dimensions simultaneously
- Provides trend analysis and momentum indicators
- Offers strategic insights for decision making
- Maintains readability during rapid updates

Let's think about this like building a sports scoreboard:
1. **Current Rankings**: Who's in what position right now
2. **Performance Metrics**: Not just chips, but efficiency and consistency
3. **Trend Indicators**: Is someone rising or falling?
4. **Predictive Elements**: Who has momentum? Who's struggling?
5. **Strategic Context**: What does this data mean for decision making?

In programming terms, this is a **reactive data visualization** system combined with **real-time analytics**. We need to efficiently compute complex metrics and present them in an intuitive, constantly-updating display.

## Section 1: Leaderboard Data Architecture

First, let's design our leaderboard data structures. Think of this like creating a sophisticated analytics dashboard:

```rust
// Comprehensive player performance tracking
#[derive(Component, Debug, Clone)]
struct PlayerPerformance {
    // Core metrics
    current_chips: u32,
    starting_chips: u32,
    hands_played: u32,
    hands_won: u32,
    
    // Advanced metrics
    win_rate: f32,               // hands_won / hands_played
    chip_efficiency: f32,        // chips_gained / hands_played
    momentum: f32,               // Recent performance trend (-1.0 to 1.0)
    consistency: f32,            // How stable is their performance
    pressure_resistance: f32,    // Performance under pressure
    
    // Temporal tracking
    recent_results: VecDeque<HandResult>,  // Last 10 hands
    performance_history: Vec<PerformanceSnapshot>,
    last_update_time: f32,
}

#[derive(Debug, Clone)]
struct HandResult {
    won: bool,
    chips_change: i32,
    was_war: bool,
    timestamp: f32,
}

#[derive(Debug, Clone)]
struct PerformanceSnapshot {
    timestamp: f32,
    chips: u32,
    win_rate: f32,
    momentum: f32,
}
```

**Advanced Metrics Explained:**

**Win Rate**: Simple percentage of hands won
**Chip Efficiency**: Average chip gain per hand played (can be negative)
**Momentum**: Weighted average of recent performance (recent results matter more)
**Consistency**: Standard deviation of chip changes (lower = more consistent)
**Pressure Resistance**: Performance when chip count is low relative to field

**VecDeque for Recent Results:**
- `VecDeque` (double-ended queue) allows efficient push/pop from both ends
- Perfect for sliding window of recent results
- `push_back()` adds new results, `pop_front()` removes old ones
- O(1) operations for maintaining recent history

```rust
// Leaderboard display state
#[derive(Resource)]
struct LeaderboardState {
    rankings: Vec<PlayerRanking>,
    update_timer: Timer,
    animation_state: AnimationState,
    sort_mode: SortMode,
}

#[derive(Debug, Clone)]
struct PlayerRanking {
    player_entity: Entity,
    name: String,
    current_position: usize,
    previous_position: usize,
    position_change: PositionChange,
    
    // Display metrics
    chips: u32,
    win_rate: f32,
    momentum: f32,
    trend: TrendIndicator,
    
    // Animation state
    display_position: f32,      // Smooth position animation
    highlight_timer: Timer,     // Flash on significant changes
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PositionChange {
    Rising(usize),    // Up N positions
    Falling(usize),   // Down N positions
    Stable,           // No change
    New,              // First appearance
}

#[derive(Debug, Clone, Copy)]
enum TrendIndicator {
    HotStreak,        // Multiple wins in a row
    ColdStreak,       // Multiple losses in a row
    Volatile,         // Big swings up and down
    Steady,           // Consistent performance
    Recovering,       // Coming back from losses
    Declining,        // Falling from previous highs
}

#[derive(Debug, Clone, Copy)]
enum SortMode {
    ChipCount,        // Traditional ranking by chips
    WinRate,          // Sort by win percentage
    Momentum,         // Sort by recent performance
    Efficiency,       // Sort by chip gain per hand
}
```

**Position Change Tracking:**
- Tracks movement between leaderboard updates
- `Rising(3)` means moved up 3 positions
- Used for visual animations and trend analysis

**Trend Indicators:**
- **HotStreak**: 3+ wins in a row
- **ColdStreak**: 3+ losses in a row  
- **Volatile**: Alternating big wins/losses
- **Steady**: Consistent small gains/losses
- **Recovering**: Winning after a losing streak
- **Declining**: Losing after a winning streak

**Multiple Sort Modes:**
- Players can switch between different ranking criteria
- Each mode highlights different aspects of performance
- **ChipCount**: Traditional tournament ranking
- **WinRate**: Pure skill measurement
- **Momentum**: Who's hot right now
- **Efficiency**: Best chip-per-hand performance

### Understanding the Design

**Why separate PlayerPerformance from PlayerRanking?**
- `PlayerPerformance` is the **data layer** (attached to player entities)
- `PlayerRanking` is the **presentation layer** (managed by leaderboard system)
- This separation allows complex analytics without cluttering entity components
- Rankings can be sorted, filtered, and animated independently of core data

**Real-time vs Batch Updates:**
- Player performance updates immediately on every hand
- Leaderboard rankings update on a timer (every 0.5 seconds)
- This prevents visual chaos while maintaining responsiveness

## Section 2: Performance Calculation Engine

Now let's implement the analytics engine that computes all these metrics:

```rust
impl PlayerPerformance {
    fn new(starting_chips: u32) -> Self {
        Self {
            current_chips: starting_chips,
            starting_chips,
            hands_played: 0,
            hands_won: 0,
            win_rate: 0.0,
            chip_efficiency: 0.0,
            momentum: 0.0,
            consistency: 0.0,
            pressure_resistance: 0.0,
            recent_results: VecDeque::with_capacity(10),
            performance_history: Vec::new(),
            last_update_time: 0.0,
        }
    }

    fn update_with_result(&mut self, result: HandResult, current_time: f32) {
        // Update basic counters
        self.hands_played += 1;
        if result.won {
            self.hands_won += 1;
        }
        
        // Update chip count
        self.current_chips = (self.current_chips as i32 + result.chips_change) 
            .max(0) as u32;
        
        // Add to recent results (maintain sliding window)
        self.recent_results.push_back(result.clone());
        if self.recent_results.len() > 10 {
            self.recent_results.pop_front();
        }
        
        // Recalculate all derived metrics
        self.recalculate_metrics(current_time);
        
        // Take performance snapshot every 5 hands
        if self.hands_played % 5 == 0 {
            self.performance_history.push(PerformanceSnapshot {
                timestamp: current_time,
                chips: self.current_chips,
                win_rate: self.win_rate,
                momentum: self.momentum,
            });
        }
        
        self.last_update_time = current_time;
    }

    fn recalculate_metrics(&mut self, current_time: f32) {
        // Basic win rate
        self.win_rate = if self.hands_played > 0 {
            self.hands_won as f32 / self.hands_played as f32
        } else {
            0.0
        };
        
        // Chip efficiency (average gain per hand)
        let total_change = self.current_chips as i32 - self.starting_chips as i32;
        self.chip_efficiency = if self.hands_played > 0 {
            total_change as f32 / self.hands_played as f32
        } else {
            0.0
        };
        
        // Momentum calculation (weighted recent performance)
        self.momentum = self.calculate_momentum();
        
        // Consistency (stability of performance)
        self.consistency = self.calculate_consistency();
        
        // Pressure resistance (performance when behind)
        self.pressure_resistance = self.calculate_pressure_resistance();
    }

    fn calculate_momentum(&self) -> f32 {
        if self.recent_results.is_empty() {
            return 0.0;
        }
        
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        
        // Weight recent results more heavily
        for (i, result) in self.recent_results.iter().enumerate() {
            let weight = (i + 1) as f32; // More recent = higher weight
            let value = if result.won { 1.0 } else { -1.0 };
            
            // Boost weight for wars (more significant outcomes)
            let final_weight = if result.was_war { weight * 1.5 } else { weight };
            
            weighted_sum += value * final_weight;
            weight_sum += final_weight;
        }
        
        if weight_sum > 0.0 {
            (weighted_sum / weight_sum).clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }
```

**Momentum Calculation Deep Dive:**

The momentum formula uses **weighted average** of recent results:
1. Iterate through recent results (last 10 hands)
2. Weight more recent results higher (hand 10 has 10x weight of hand 1)
3. Win = +1.0, Loss = -1.0
4. Wars get 1.5x weight (more significant outcomes)
5. Normalize to [-1.0, 1.0] range

**Why Weighted Average?**
- Recent performance is more indicative of current "momentum"
- A single recent win after 9 losses shouldn't show positive momentum
- But 3 recent wins after 7 old losses might indicate a turning point

```rust
    fn calculate_consistency(&self) -> f32 {
        if self.recent_results.len() < 3 {
            return 0.5; // Not enough data, assume average consistency
        }
        
        // Calculate standard deviation of chip changes
        let chip_changes: Vec<f32> = self.recent_results.iter()
            .map(|r| r.chips_change as f32)
            .collect();
            
        let mean = chip_changes.iter().sum::<f32>() / chip_changes.len() as f32;
        
        let variance = chip_changes.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f32>() / chip_changes.len() as f32;
            
        let std_dev = variance.sqrt();
        
        // Convert standard deviation to consistency score (0.0 = chaotic, 1.0 = very consistent)
        // Typical std_dev ranges from 0 (all results identical) to ~50 (very volatile)
        let normalized_std_dev = (std_dev / 50.0).min(1.0);
        1.0 - normalized_std_dev  // Invert so high consistency = high score
    }

    fn calculate_pressure_resistance(&self) -> f32 {
        if self.performance_history.len() < 3 {
            return 0.5; // Not enough data
        }
        
        // Find periods where player was under pressure (below average chips)
        let average_chips: f32 = self.performance_history.iter()
            .map(|s| s.chips as f32)
            .sum::<f32>() / self.performance_history.len() as f32;
        
        let mut pressure_performances = Vec::new();
        
        for snapshot in &self.performance_history {
            if snapshot.chips < average_chips as u32 {
                // Player was under pressure at this point
                pressure_performances.push(snapshot.win_rate);
            }
        }
        
        if pressure_performances.is_empty() {
            return 1.0; // Never under pressure = perfect resistance
        }
        
        // Average win rate during pressure situations
        pressure_performances.iter().sum::<f32>() / pressure_performances.len() as f32
    }
}
```

**Standard Deviation for Consistency:**
```rust
let variance = chip_changes.iter()
    .map(|x| (x - mean).powi(2))
    .sum::<f32>() / chip_changes.len() as f32;
```
- Calculate how much each result deviates from the average
- Square the deviations (removes negative values, emphasizes outliers)
- Average the squared deviations = variance
- Square root of variance = standard deviation

**Pressure Resistance Logic:**
- Find all historical periods where player was below average chip count
- Calculate win rate during those "pressure" periods
- Players who maintain high win rates when behind have high pressure resistance
- Never being under pressure gives perfect score (1.0)

## Section 3: Real-time Leaderboard System

Now let's implement the live-updating leaderboard display:

```rust
fn update_leaderboard_system(
    mut leaderboard_state: ResMut<LeaderboardState>,
    tournament_state: Res<TournamentState>,
    performance_query: Query<&PlayerPerformance>,
    time: Res<Time>,
) {
    leaderboard_state.update_timer.tick(time.delta());
    
    if leaderboard_state.update_timer.finished() {
        // Rebuild rankings from current tournament state
        let mut new_rankings = Vec::new();
        
        for (index, player) in tournament_state.players.iter().enumerate() {
            if let Ok(performance) = performance_query.get(player.entity) {
                let current_position = index + 1;
                
                // Find previous position
                let previous_position = leaderboard_state.rankings.iter()
                    .find(|r| r.player_entity == player.entity)
                    .map(|r| r.current_position)
                    .unwrap_or(current_position);
                
                // Calculate position change
                let position_change = if previous_position == current_position {
                    PositionChange::Stable
                } else if previous_position > current_position {
                    PositionChange::Rising(previous_position - current_position)
                } else {
                    PositionChange::Falling(current_position - previous_position)
                };
                
                // Calculate trend indicator
                let trend = calculate_trend_indicator(performance);
                
                new_rankings.push(PlayerRanking {
                    player_entity: player.entity,
                    name: player.name.clone(),
                    current_position,
                    previous_position,
                    position_change,
                    chips: performance.current_chips,
                    win_rate: performance.win_rate,
                    momentum: performance.momentum,
                    trend,
                    display_position: current_position as f32,
                    highlight_timer: Timer::from_seconds(0.0, TimerMode::Once),
                });
            }
        }
        
        // Sort by current sort mode
        sort_rankings(&mut new_rankings, leaderboard_state.sort_mode);
        
        // Update positions after sorting
        for (index, ranking) in new_rankings.iter_mut().enumerate() {
            ranking.current_position = index + 1;
        }
        
        // Highlight significant changes
        highlight_significant_changes(&mut new_rankings, &leaderboard_state.rankings);
        
        leaderboard_state.rankings = new_rankings;
        leaderboard_state.update_timer = Timer::from_seconds(0.5, TimerMode::Once);
    }
}

fn calculate_trend_indicator(performance: &PlayerPerformance) -> TrendIndicator {
    if performance.recent_results.len() < 3 {
        return TrendIndicator::Steady;
    }
    
    // Count recent wins/losses
    let recent_wins = performance.recent_results.iter()
        .rev()  // Most recent first
        .take(3)
        .filter(|r| r.won)
        .count();
    
    let recent_losses = 3 - recent_wins;
    
    // Analyze patterns
    if recent_wins >= 3 {
        TrendIndicator::HotStreak
    } else if recent_losses >= 3 {
        TrendIndicator::ColdStreak
    } else if performance.momentum > 0.5 {
        TrendIndicator::Recovering
    } else if performance.momentum < -0.5 {
        TrendIndicator::Declining
    } else if performance.consistency < 0.3 {
        TrendIndicator::Volatile
    } else {
        TrendIndicator::Steady
    }
}

fn sort_rankings(rankings: &mut Vec<PlayerRanking>, sort_mode: SortMode) {
    match sort_mode {
        SortMode::ChipCount => {
            rankings.sort_by(|a, b| b.chips.cmp(&a.chips));
        },
        SortMode::WinRate => {
            rankings.sort_by(|a, b| b.win_rate.partial_cmp(&a.win_rate)
                .unwrap_or(std::cmp::Ordering::Equal));
        },
        SortMode::Momentum => {
            rankings.sort_by(|a, b| b.momentum.partial_cmp(&a.momentum)
                .unwrap_or(std::cmp::Ordering::Equal));
        },
        SortMode::Efficiency => {
            // Would need to calculate efficiency here or store it in PlayerRanking
            rankings.sort_by(|a, b| b.chips.cmp(&a.chips)); // Fallback to chips
        },
    }
}

fn highlight_significant_changes(
    new_rankings: &mut Vec<PlayerRanking>,
    old_rankings: &Vec<PlayerRanking>,
) {
    for new_ranking in new_rankings.iter_mut() {
        if let Some(old_ranking) = old_rankings.iter()
            .find(|r| r.player_entity == new_ranking.player_entity)
        {
            let position_change = old_ranking.current_position as i32 - 
                                new_ranking.current_position as i32;
            
            // Highlight significant moves (2+ positions)
            if position_change.abs() >= 2 {
                new_ranking.highlight_timer = Timer::from_seconds(2.0, TimerMode::Once);
            }
        }
    }
}
```

**Leaderboard Update Flow:**
1. **Timer-Based Updates**: Every 0.5 seconds to prevent visual chaos
2. **Position Tracking**: Compare current vs previous position for each player
3. **Trend Analysis**: Analyze recent performance patterns
4. **Dynamic Sorting**: Support multiple ranking criteria
5. **Change Highlighting**: Flash significant position changes

**Position Change Calculation:**
```rust
let position_change = if previous_position == current_position {
    PositionChange::Stable
} else if previous_position > current_position {
    PositionChange::Rising(previous_position - current_position)
} else {
    PositionChange::Falling(current_position - previous_position)
};
```
- Higher position number = worse rank (position 1 = 1st place)
- Moving from position 5 to 3 = Rising(2)
- Moving from position 2 to 4 = Falling(2)

## Section 4: Leaderboard Visualization System

Let's create the visual leaderboard that makes all this data compelling:

```rust
// Leaderboard UI components
#[derive(Component)]
struct LeaderboardPanel;

#[derive(Component)]
struct PlayerRow {
    player_entity: Entity,
}

#[derive(Component)]
struct PositionIndicator;

#[derive(Component)]
struct TrendIcon;

#[derive(Component)]
struct AnimatedValue {
    target_value: f32,
    current_value: f32,
    animation_speed: f32,
}

fn setup_leaderboard_ui(mut commands: Commands) {
    // Main leaderboard panel
    commands.spawn((
        LeaderboardPanel,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            top: Val::Px(20.0),
            width: Val::Px(350.0),
            height: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
    ))
    .with_children(|parent| {
        // Header
        parent.spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("LEADERBOARD"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.6, 0.0)),
            ));
            
            // Sort mode indicator
            parent.spawn((
                Text::new("CHIPS"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
        
        // Column headers
        spawn_column_headers(parent);
        
        // Player rows container
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                gap: Val::Px(5.0),
                ..default()
            },
        ));
    });
}

fn spawn_column_headers(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::vertical(Val::Px(5.0)),
            border_bottom: UiRect::all(Val::Px(1.0)),
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
        BorderColor(Color::srgb(0.4, 0.4, 0.4)),
    ))
    .with_children(|parent| {
        let headers = [
            ("POS", 40.0),
            ("PLAYER", 120.0),
            ("CHIPS", 60.0),
            ("TREND", 30.0),
            ("WIN%", 40.0),
        ];
        
        for (header, width) in headers {
            parent.spawn((
                Node {
                    width: Val::Px(width),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_child((
                Text::new(header),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        }
    });
}
```

**UI Layout Architecture:**
- **Fixed Position**: Right side of screen, always visible
- **Column-Based**: Structured data presentation
- **Dark Theme**: High contrast for readability during gameplay
- **Responsive**: Adapts to different player counts

**Flexbox Layout Strategy:**
- `FlexDirection::Column` for vertical player stacking
- `JustifyContent::SpaceBetween` for even spacing
- Fixed column widths for consistent alignment
- `flex_grow: 1.0` for dynamic content area

```rust
fn update_leaderboard_display(
    mut commands: Commands,
    leaderboard_state: Res<LeaderboardState>,
    leaderboard_query: Query<Entity, With<LeaderboardPanel>>,
    mut row_query: Query<(Entity, &PlayerRow)>,
) {
    if !leaderboard_state.is_changed() {
        return;
    }
    
    // Find the player rows container
    let leaderboard_entity = match leaderboard_query.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };
    
    // Clear existing player rows
    for (entity, _) in &row_query {
        commands.entity(entity).despawn_recursive();
    }
    
    // Find the rows container (3rd child of leaderboard panel)
    commands.entity(leaderboard_entity).with_children(|parent| {
        let children: Vec<_> = parent.parent_entity().iter().collect();
        if children.len() >= 3 {
            commands.entity(children[2]).with_children(|rows_parent| {
                // Spawn new player rows
                for (index, ranking) in leaderboard_state.rankings.iter().enumerate() {
                    spawn_player_row(rows_parent, ranking, index);
                }
            });
        }
    });
}

fn spawn_player_row(
    parent: &mut ChildBuilder,
    ranking: &PlayerRanking,
    index: usize,
) {
    let row_color = if index % 2 == 0 {
        Color::srgba(0.1, 0.1, 0.1, 0.3)
    } else {
        Color::srgba(0.2, 0.2, 0.2, 0.3)
    };
    
    // Highlight based on position change
    let highlight_color = match ranking.position_change {
        PositionChange::Rising(_) => Color::srgba(0.0, 0.8, 0.0, 0.4),
        PositionChange::Falling(_) => Color::srgba(0.8, 0.0, 0.0, 0.4),
        _ => row_color,
    };
    
    parent.spawn((
        PlayerRow {
            player_entity: ranking.player_entity,
        },
        Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(if ranking.highlight_timer.finished() { row_color } else { highlight_color }),
    ))
    .with_children(|parent| {
        // Position with change indicator
        spawn_position_cell(parent, ranking);
        
        // Player name
        spawn_name_cell(parent, ranking);
        
        // Chips with animated value
        spawn_chips_cell(parent, ranking);
        
        // Trend indicator
        spawn_trend_cell(parent, ranking);
        
        // Win rate
        spawn_winrate_cell(parent, ranking);
    });
}

fn spawn_position_cell(parent: &mut ChildBuilder, ranking: &PlayerRanking) {
    parent.spawn((
        Node {
            width: Val::Px(40.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            gap: Val::Px(3.0),
            ..default()
        },
    ))
    .with_children(|parent| {
        // Position number
        parent.spawn((
            Text::new(format!("{}", ranking.current_position)),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        // Change indicator
        let (symbol, color) = match ranking.position_change {
            PositionChange::Rising(n) => (format!("▲{}", n), Color::srgb(0.0, 0.8, 0.0)),
            PositionChange::Falling(n) => (format!("▼{}", n), Color::srgb(0.8, 0.0, 0.0)),
            PositionChange::Stable => ("=".to_string(), Color::srgb(0.5, 0.5, 0.5)),
            PositionChange::New => ("●".to_string(), Color::srgb(0.0, 0.6, 1.0)),
        };
        
        parent.spawn((
            Text::new(symbol),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(color),
        ));
    });
}

fn spawn_trend_cell(parent: &mut ChildBuilder, ranking: &PlayerRanking) {
    let (symbol, color) = match ranking.trend {
        TrendIndicator::HotStreak => ("🔥", Color::srgb(1.0, 0.3, 0.0)),
        TrendIndicator::ColdStreak => ("❄️", Color::srgb(0.0, 0.6, 1.0)),
        TrendIndicator::Volatile => ("⚡", Color::srgb(1.0, 1.0, 0.0)),
        TrendIndicator::Steady => ("📈", Color::srgb(0.0, 0.8, 0.0)),
        TrendIndicator::Recovering => ("↗️", Color::srgb(0.0, 0.8, 0.0)),
        TrendIndicator::Declining => ("↘️", Color::srgb(0.8, 0.0, 0.0)),
    };
    
    parent.spawn((
        Node {
            width: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
    ))
    .with_child((
        Text::new(symbol),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(color),
    ));
}
```

**Visual Design Principles:**

**Alternating Row Colors:**
- Even rows: Dark gray background
- Odd rows: Slightly lighter gray
- Creates visual separation without heavy borders

**Position Change Indicators:**
- **▲n**: Green up arrow with position count
- **▼n**: Red down arrow with position count  
- **=**: Gray equals sign for stable
- **●**: Blue dot for new players

**Trend Indicators:**
- **🔥**: Hot streak (multiple wins)
- **❄️**: Cold streak (multiple losses)
- **⚡**: Volatile performance
- **📈**: Steady improvement
- **↗️**: Recovering from losses
- **↘️**: Declining from highs

## Section 5: Animated Value Updates

Let's add smooth animations for changing values to make updates feel polished:

```rust
fn animate_leaderboard_values(
    mut animated_query: Query<(&mut AnimatedValue, &mut Text)>,
    time: Res<Time>,
) {
    for (mut animated, mut text) in &mut animated_query {
        let delta = time.delta_seconds();
        let difference = animated.target_value - animated.current_value;
        
        if difference.abs() > 0.1 {
            // Smoothly animate toward target
            let step = difference * animated.animation_speed * delta;
            animated.current_value += step;
            
            // Update display text
            text.0 = format!("{:.0}", animated.current_value);
        } else {
            // Snap to target when close enough
            animated.current_value = animated.target_value;
            text.0 = format!("{:.0}", animated.target_value);
        }
    }
}

fn spawn_chips_cell(parent: &mut ChildBuilder, ranking: &PlayerRanking) {
    parent.spawn((
        Node {
            width: Val::Px(60.0),
            justify_content: JustifyContent::End,
            ..default()
        },
    ))
    .with_child((
        AnimatedValue {
            target_value: ranking.chips as f32,
            current_value: ranking.chips as f32,
            animation_speed: 3.0,  // 3x speed for responsive feel
        },
        Text::new(format!("{}", ranking.chips)),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.8, 0.0)),
    ));
}

fn spawn_winrate_cell(parent: &mut ChildBuilder, ranking: &PlayerRanking) {
    let color = if ranking.win_rate > 0.6 {
        Color::srgb(0.0, 0.8, 0.0)  // Green for high win rate
    } else if ranking.win_rate < 0.4 {
        Color::srgb(0.8, 0.0, 0.0)  // Red for low win rate
    } else {
        Color::WHITE  // White for average
    };
    
    parent.spawn((
        Node {
            width: Val::Px(40.0),
            justify_content: JustifyContent::End,
            ..default()
        },
    ))
    .with_child((
        AnimatedValue {
            target_value: ranking.win_rate * 100.0,
            current_value: ranking.win_rate * 100.0,
            animation_speed: 2.0,
        },
        Text::new(format!("{:.0}%", ranking.win_rate * 100.0)),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(color),
    ));
}
```

**Animation System Design:**

**Smooth Value Interpolation:**
```rust
let step = difference * animated.animation_speed * delta;
animated.current_value += step;
```
- Exponential decay animation (fast at start, slows as it approaches target)
- `animation_speed` controls how quickly values converge
- Higher speed = more responsive, lower speed = smoother

**Snap Threshold:**
```rust
if difference.abs() > 0.1 {
    // Animate
} else {
    // Snap to target
}
```
- Prevents infinite tiny movements
- Ensures final value is exactly correct
- 0.1 threshold is barely perceptible

**Color-Coded Win Rates:**
- **Green**: 60%+ win rate (excellent)
- **Red**: <40% win rate (struggling)
- **White**: 40-60% win rate (average)

## Section 6: Interactive Leaderboard Features

Let's add interactive features that enhance the competitive experience:

```rust
// Leaderboard interaction components
#[derive(Component)]
struct SortModeButton {
    sort_mode: SortMode,
}

#[derive(Component)]
struct PlayerDetailButton {
    player_entity: Entity,
}

fn handle_sort_mode_buttons(
    mut leaderboard_state: ResMut<LeaderboardState>,
    mut button_query: Query<
        (&Interaction, &SortModeButton, &mut BackgroundColor),
        Changed<Interaction>
    >,
) {
    for (interaction, button, mut background) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                leaderboard_state.sort_mode = button.sort_mode;
                *background = BackgroundColor(Color::srgb(0.0, 0.6, 0.0));
                info!("Switched to {:?} sorting", button.sort_mode);
            },
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            },
            Interaction::None => {
                let is_active = leaderboard_state.sort_mode == button.sort_mode;
                *background = BackgroundColor(if is_active {
                    Color::srgb(0.0, 0.4, 0.0)
                } else {
                    Color::srgb(0.2, 0.2, 0.2)
                });
            },
        }
    }
}

fn setup_sort_mode_buttons(parent: &mut ChildBuilder) {
    let sort_modes = [
        (SortMode::ChipCount, "CHIPS"),
        (SortMode::WinRate, "WIN%"),
        (SortMode::Momentum, "MOMENTUM"),
        (SortMode::Efficiency, "EFFICIENCY"),
    ];
    
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            gap: Val::Px(5.0),
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        for (sort_mode, label) in sort_modes {
            parent.spawn((
                Button,
                SortModeButton { sort_mode },
                Node {
                    padding: UiRect::all(Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_child((
                Text::new(label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        }
    });
}

// Detailed player statistics popup
fn spawn_player_detail_popup(
    commands: &mut Commands,
    player_entity: Entity,
    performance: &PlayerPerformance,
    position: usize,
) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(400.0),
            top: Val::Px(100.0),
            width: Val::Px(300.0),
            height: Val::Px(250.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        BorderColor(Color::srgb(0.4, 0.4, 0.4)),
    ))
    .with_children(|parent| {
        // Header
        parent.spawn((
            Text::new(format!("Player #{} Details", position)),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.6, 0.0)),
        ));
        
        // Statistics
        let stats = [
            ("Current Chips", format!("{}", performance.current_chips)),
            ("Hands Played", format!("{}", performance.hands_played)),
            ("Win Rate", format!("{:.1}%", performance.win_rate * 100.0)),
            ("Chip Efficiency", format!("{:.1}", performance.chip_efficiency)),
            ("Momentum", format!("{:.2}", performance.momentum)),
            ("Consistency", format!("{:.1}%", performance.consistency * 100.0)),
            ("Pressure Resistance", format!("{:.1}%", performance.pressure_resistance * 100.0)),
        ];
        
        for (label, value) in stats {
            parent.spawn((
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::vertical(Val::Px(2.0)),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(label),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                ));
                
                parent.spawn((
                    Text::new(value),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        }
    });
}
```

**Interactive Features:**

**Sort Mode Switching:**
- Buttons for each ranking criterion
- Active button highlighted in green
- Hover effects for responsiveness
- Immediate resort when clicked

**Player Detail Popups:**
- Click any player row to see detailed stats
- Shows all calculated metrics
- Positioned to not obscure leaderboard
- Comprehensive performance breakdown

**Visual State Management:**
- Active sort button stays highlighted
- Hover states provide immediate feedback
- Color coding indicates current mode
- Smooth transitions between states

## Testing Your Leaderboard System

At this point, you should be able to:

1. **View Live Rankings**: See all 6 players ranked by chips in real-time
2. **Switch Sort Modes**: Click buttons to rank by win rate, momentum, etc.
3. **See Position Changes**: Watch arrows indicate who's rising/falling
4. **Observe Trend Indicators**: Different emojis show performance patterns
5. **Click for Details**: See comprehensive stats for any player

## Key Concepts Mastered

1. **Real-time Data Visualization**: Efficient updates without overwhelming the UI
   - Timer-based refresh rates
   - Smooth value animations
   - Change highlighting and indicators

2. **Multi-dimensional Analytics**: Complex performance metrics beyond simple scores
   - Win rate, momentum, consistency calculations
   - Pressure resistance and trend analysis
   - Historical performance tracking

3. **Interactive Data Presentation**: User-controlled views of the same data
   - Multiple sorting criteria
   - Detail-on-demand popups
   - Visual state management

4. **Performance Optimization**: Efficient calculations and minimal UI updates
   - Sliding window for recent results
   - Batch calculations during updates
   - Component-based UI architecture

5. **Statistical Analysis**: Mathematical models for game performance
   - Weighted averages for momentum
   - Standard deviation for consistency
   - Conditional analysis for pressure resistance

## Exercises

1. **Add Performance Charts**: Create mini line graphs showing each player's chip progression over time

2. **Implement Player Comparison**: Side-by-side detailed comparison of any two players

3. **Create Achievement System**: Award badges for specific accomplishments (hot streaks, comebacks, etc.)

4. **Add Prediction Engine**: Use historical data to predict likely tournament winner

5. **Implement Export Functionality**: Allow saving tournament statistics to JSON file

## Next Steps

In Part 9, we'll add:
- Timed tournament mode with countdown pressure
- Winner-take-all prize distribution
- Time-pressure psychology for AI players
- Overtime mechanics for dramatic finishes

The real-time leaderboard will become even more exciting as players race against the clock!