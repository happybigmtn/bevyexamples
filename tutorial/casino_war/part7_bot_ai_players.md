# Casino War Tutorial - Part 7: Bot AI Players - The Tournament Begins

## What We're Building in Part 7

Transforming our single-player Casino War into a competitive 6-player tournament arena:

1. **5 AI Opponents**: Conservative, Aggressive, Balanced, Adaptive, and Chaos Bot
2. **AI Decision Engine**: Strategic betting and war decisions based on game state
3. **Multi-player Architecture**: Simultaneous tournament management for 6 players
4. **Psychological Modeling**: Different AI personalities with unique behavioral patterns
5. **Tournament Progression**: Multi-round elimination with persistent statistics

## Understanding AI Decision Systems

### The AI Strategy Problem

Imagine you're designing poker-playing robots, each with a different personality. We need AI players that:
- Make consistent decisions based on their strategy
- Adapt to changing game conditions
- Feel authentically different from each other
- Provide meaningful competition for the human player

Let's think about this like building actual personalities:
1. **Conservative Bot**: Risk-averse, consistent betting, rarely goes to war
2. **Aggressive Bot**: High-risk, high-reward, always looking for big wins
3. **Balanced Bot**: Mathematical approach, considers odds and position
4. **Adaptive Bot**: Learns from opponents, adjusts strategy dynamically
5. **Chaos Bot**: Unpredictable, creates exciting variance in gameplay

In programming terms, this is a **strategy pattern** combined with **state machines**. Each AI has a core personality that filters through different decision contexts.

## Section 1: AI Player Architecture

First, let's define our AI player system. Think of this like creating character classes in an RPG:

```rust
// AI Player personality types
#[derive(Component, Debug, Clone, Copy, PartialEq)]
enum AiPersonality {
    Conservative,  // Low risk, consistent play
    Aggressive,   // High risk, go big or go home
    Balanced,     // Mathematical, considers position
    Adaptive,     // Learns and adjusts strategy
    Chaos,        // Unpredictable variance
}

// AI decision state tracking
#[derive(Component)]
struct AiPlayer {
    personality: AiPersonality,
    risk_tolerance: f32,      // 0.0 = very conservative, 1.0 = very aggressive
    learning_rate: f32,       // How quickly they adapt (Adaptive bot)
    recent_wins: u32,         // Performance tracking
    recent_losses: u32,       // Performance tracking
    decision_timer: Timer,    // Thinking time for realism
    current_strategy: AiStrategy,
}
```

**Rust Concept: Enums as Types**
- `AiPersonality` enum creates distinct, compile-time-checked personality types
- Each bot MUST be exactly one personality - no ambiguity
- `#[derive(Clone, Copy)]` allows easy personality copying
- This is **type safety** - prevents impossible states like "half-aggressive, half-conservative"

**Component Design Philosophy:**
- `AiPlayer` holds all AI-specific state
- Separate from human player components
- Each field serves a specific decision-making purpose

```rust
// AI strategic approaches within personalities
#[derive(Debug, Clone)]
enum AiStrategy {
    // Conservative strategies
    MinimumBets,           // Always bet minimum to conserve chips
    WaitForGoodCards,      // Only increase bets with high cards
    
    // Aggressive strategies  
    MaximumPressure,       // Large bets to intimidate opponents
    AllInOnFaceCards,      // Big bets when holding face cards
    
    // Balanced strategies
    PositionAware,         // Bet based on tournament position
    ChipStackManagement,   // Adjust betting based on chip count
    
    // Adaptive strategies
    OpponentModeling,      // Mirror successful opponent strategies
    CounterStrategy,       // Exploit opponent weaknesses
    
    // Chaos strategies
    RandomWalk,            // Completely unpredictable
    MoodSwings,            // Alternate between extreme strategies
}
```

**Strategy Pattern in Action:**
- Each personality can switch between multiple strategies
- Strategies are **behaviors**, personalities are **temperaments**
- Adaptive bot changes strategies based on game state
- Chaos bot switches randomly for unpredictability

```rust
// AI decision context - what information do they consider?
#[derive(Debug)]
struct DecisionContext {
    current_card: Option<Card>,
    opponent_visible_cards: Vec<Card>,
    chip_position: f32,           // Percentage of total chips in tournament
    tournament_position: usize,    // Current ranking (1st, 2nd, etc.)
    recent_war_outcomes: Vec<bool>, // Win history for war decisions
    pressure_level: f32,          // How "pressured" the AI feels
}
```

**Context-Driven Decisions:**
This struct contains ALL the information an AI needs to make smart decisions:
- `current_card` - what they're holding (if visible)
- `opponent_visible_cards` - what they can see others holding
- `chip_position` - are they ahead or behind?
- `tournament_position` - ranking affects risk tolerance
- `recent_war_outcomes` - do wars tend to favor them?
- `pressure_level` - psychological state affects decision making

### Understanding the Design

**Why separate personality from strategy?**
- Personality is CORE temperament (doesn't change much)
- Strategy is TACTICAL approach (adapts to situation)
- Same personality can use different strategies in different contexts
- Allows for complex, realistic AI behavior

**Why include psychological state?**
Real players make decisions based on:
- Current performance (winning streak vs losing streak)
- Tournament pressure (ahead vs behind)
- Recent experiences (just lost a big war vs just won one)

Our AI simulates these human factors for authenticity.

## Section 2: AI Decision Engine

Now let's implement the core decision-making logic. This is the "brain" of our AI players:

```rust
impl AiPlayer {
    fn new(personality: AiPersonality) -> Self {
        let (risk_tolerance, learning_rate, initial_strategy) = match personality {
            AiPersonality::Conservative => (0.2, 0.1, AiStrategy::MinimumBets),
            AiPersonality::Aggressive => (0.8, 0.3, AiStrategy::MaximumPressure),
            AiPersonality::Balanced => (0.5, 0.2, AiStrategy::PositionAware),
            AiPersonality::Adaptive => (0.5, 0.7, AiStrategy::OpponentModeling),
            AiPersonality::Chaos => (0.6, 0.9, AiStrategy::RandomWalk),
        };

        Self {
            personality,
            risk_tolerance,
            learning_rate,
            recent_wins: 0,
            recent_losses: 0,
            decision_timer: Timer::from_seconds(
                fastrand::f32() * 2.0 + 1.0, // 1-3 second "thinking" time
                TimerMode::Once
            ),
            current_strategy: initial_strategy,
        }
    }
```

**Personality Calibration:**
Each personality starts with carefully tuned parameters:
- **Conservative**: Low risk (0.2), slow learning (0.1), minimal betting
- **Aggressive**: High risk (0.8), moderate learning (0.3), pressure tactics
- **Balanced**: Middle risk (0.5), moderate learning (0.2), position-aware
- **Adaptive**: Variable risk (0.5), high learning (0.7), models opponents
- **Chaos**: High risk (0.6), very high learning (0.9), random strategies

**Randomized Thinking Time:**
`fastrand::f32() * 2.0 + 1.0` creates 1-3 second delays:
- Makes AI feel more human-like
- Prevents instant decision making
- Adds suspense to tournament play

```rust
    fn make_betting_decision(&mut self, context: &DecisionContext) -> u32 {
        // Update strategy based on recent performance
        self.update_strategy(context);
        
        let base_bet = match self.current_strategy {
            AiStrategy::MinimumBets => MIN_BET,
            
            AiStrategy::WaitForGoodCards => {
                if let Some(card) = context.current_card {
                    if card.rank.value() >= 10 {
                        MIN_BET * 3  // Bet more on face cards
                    } else {
                        MIN_BET
                    }
                } else {
                    MIN_BET
                }
            },
            
            AiStrategy::MaximumPressure => {
                // Bet based on chip advantage
                if context.chip_position > 0.6 {
                    MIN_BET * 5  // Pressure opponents when ahead
                } else {
                    MIN_BET * 2  // Conservative when behind
                }
            },
            
            AiStrategy::PositionAware => {
                match context.tournament_position {
                    1 => MIN_BET,           // Leader plays safe
                    2..=3 => MIN_BET * 2,   // Contenders stay aggressive
                    _ => MIN_BET * 3,       // Desperate times call for big bets
                }
            },
            
            AiStrategy::ChipStackManagement => {
                let chip_factor = (context.chip_position * 10.0) as u32;
                MIN_BET * chip_factor.max(1).min(5)
            },
            
            _ => self.calculate_adaptive_bet(context),
        };
        
        // Apply personality risk adjustment
        let risk_multiplier = 1.0 + (self.risk_tolerance - 0.5) * 0.5;
        let adjusted_bet = (base_bet as f32 * risk_multiplier) as u32;
        
        // Apply pressure modifier
        let pressure_adjustment = if context.pressure_level > 0.7 {
            if self.personality == AiPersonality::Aggressive {
                1.5  // Aggressive bots bet MORE under pressure
            } else {
                0.7  // Others bet LESS under pressure
            }
        } else {
            1.0
        };
        
        (adjusted_bet as f32 * pressure_adjustment) as u32
    }
```

**Strategy Implementation Deep Dive:**

**MinimumBets**: Always bets the minimum - simple but consistent
**WaitForGoodCards**: Only increases bets with strong cards (value >= 10)
**MaximumPressure**: Uses chip advantage to intimidate - bets big when ahead
**PositionAware**: Tournament rank determines aggression level
**ChipStackManagement**: Bet size scales with chip percentage

**Risk Tolerance Mathematics:**
```rust
let risk_multiplier = 1.0 + (self.risk_tolerance - 0.5) * 0.5;
```
- If risk_tolerance = 0.2 (Conservative): multiplier = 0.85x (bet 15% less)
- If risk_tolerance = 0.5 (Balanced): multiplier = 1.0x (no change)
- If risk_tolerance = 0.8 (Aggressive): multiplier = 1.15x (bet 15% more)

**Pressure Response System:**
Different personalities react differently to pressure:
- **Aggressive**: Bets 50% MORE under pressure (1.5x)
- **Others**: Bet 30% LESS under pressure (0.7x)
- This creates authentic psychological responses

```rust
    fn should_go_to_war(&mut self, context: &DecisionContext) -> bool {
        let base_probability = match self.current_strategy {
            AiStrategy::MinimumBets => 0.1,        // Almost never war
            AiStrategy::MaximumPressure => 0.8,    // Almost always war
            AiStrategy::PositionAware => {
                if context.tournament_position <= 2 {
                    0.3  // Leaders avoid wars
                } else {
                    0.7  // Desperate players take wars
                }
            },
            AiStrategy::ChipStackManagement => {
                if context.chip_position > 0.6 {
                    0.4  // Rich players can afford wars
                } else {
                    0.2  // Poor players avoid wars
                }
            },
            _ => 0.5,  // Default 50/50 for other strategies
        };
        
        // Factor in recent war performance
        let war_success_rate = if (self.recent_wins + self.recent_losses) > 0 {
            self.recent_wins as f32 / (self.recent_wins + self.recent_losses) as f32
        } else {
            0.5
        };
        
        // Adjust probability based on recent success
        let adjusted_probability = base_probability * (0.5 + war_success_rate);
        
        // Apply personality and pressure modifiers
        let final_probability = adjusted_probability 
            * (0.5 + self.risk_tolerance)  // Risk-tolerant players war more
            * (1.0 + context.pressure_level * 0.3);  // Pressure increases war likelihood
            
        fastrand::f32() < final_probability
    }
```

**War Decision Matrix:**
Each strategy has a baseline war probability:
- **MinimumBets**: 10% (very conservative)
- **MaximumPressure**: 80% (loves conflict)
- **PositionAware**: Varies by rank (30% for leaders, 70% for trailers)
- **ChipStackManagement**: Varies by wealth (40% rich, 20% poor)

**Performance Learning:**
```rust
let war_success_rate = self.recent_wins as f32 / (self.recent_wins + self.recent_losses) as f32
```
- Tracks recent war outcomes
- Successful warriors become more likely to war
- Failed warriors become more conservative
- Creates adaptive behavior over time

**Pressure Amplification:**
High pressure increases war likelihood by up to 30%:
- Simulates "desperate times" decision making
- Trailing players take bigger risks
- Leading players also feel pressure to maintain lead

```rust
    fn update_strategy(&mut self, context: &DecisionContext) {
        // Only Adaptive and Chaos personalities change strategies frequently
        match self.personality {
            AiPersonality::Adaptive => {
                // Switch strategy based on performance
                if self.recent_losses > self.recent_wins + 2 {
                    self.current_strategy = match context.tournament_position {
                        1..=2 => AiStrategy::PositionAware,
                        _ => AiStrategy::ChipStackManagement,
                    };
                }
            },
            
            AiPersonality::Chaos => {
                // Randomly switch strategies for unpredictability
                if fastrand::f32() < 0.1 {  // 10% chance each decision
                    self.current_strategy = match fastrand::u32(1..=6) {
                        1 => AiStrategy::MinimumBets,
                        2 => AiStrategy::MaximumPressure,
                        3 => AiStrategy::PositionAware,
                        4 => AiStrategy::ChipStackManagement,
                        5 => AiStrategy::RandomWalk,
                        _ => AiStrategy::MoodSwings,
                    };
                }
            },
            
            _ => {
                // Other personalities stick to their core strategies
                // But may adjust risk tolerance slightly based on performance
                if self.recent_wins > self.recent_losses {
                    self.risk_tolerance = (self.risk_tolerance * 1.05).min(1.0);
                } else {
                    self.risk_tolerance = (self.risk_tolerance * 0.95).max(0.0);
                }
            }
        }
    }
```

**Adaptive Learning:**
- **Adaptive Bot**: Changes entire strategy when losing badly
- **Chaos Bot**: Randomly switches strategies 10% of the time
- **Other Bots**: Slightly adjust risk tolerance based on performance

**Risk Tolerance Evolution:**
```rust
self.risk_tolerance = (self.risk_tolerance * 1.05).min(1.0);  // Winning increases confidence
self.risk_tolerance = (self.risk_tolerance * 0.95).max(0.0);  // Losing decreases confidence
```
- Winners become 5% more aggressive
- Losers become 5% more conservative
- Clamped to [0.0, 1.0] range
- Creates realistic confidence/doubt cycles

## Section 3: Multi-Player Tournament System

Now let's implement the tournament management. This coordinates all 6 players simultaneously:

```rust
// Tournament state tracking
#[derive(Resource)]
struct TournamentState {
    players: Vec<TournamentPlayer>,
    current_round: u32,
    elimination_threshold: u32,  // Chip count for elimination
    prize_pool: u32,
    tournament_timer: Timer,
}

#[derive(Debug, Clone)]
struct TournamentPlayer {
    entity: Entity,
    name: String,
    chips: u32,
    is_human: bool,
    current_bet: u32,
    is_eliminated: bool,
    tournament_stats: PlayerStats,
}

#[derive(Debug, Clone, Default)]
struct PlayerStats {
    hands_won: u32,
    hands_lost: u32,
    wars_won: u32,
    wars_lost: u32,
    biggest_win: u32,
    total_winnings: i32,  // Can be negative
}
```

**Tournament Architecture:**
- `TournamentState` is a **Resource** - global tournament information
- Each player is tracked in the `players` Vec
- `PlayerStats` accumulates performance data
- `elimination_threshold` prevents runaway chip counts

**Why Vec<TournamentPlayer> instead of separate entities?**
- Easier to sort for rankings
- Simpler iteration for tournament logic  
- Centralized state management
- Better cache locality for tournament calculations

```rust
fn setup_tournament(mut commands: Commands) {
    let ai_names = [
        ("Conservative Carl", AiPersonality::Conservative),
        ("Aggressive Alice", AiPersonality::Aggressive), 
        ("Balanced Bob", AiPersonality::Balanced),
        ("Adaptive Ada", AiPersonality::Adaptive),
        ("Chaos Charlie", AiPersonality::Chaos),
    ];

    let mut players = Vec::new();
    
    // Spawn human player
    let human_entity = commands.spawn((
        Player,
        Transform::default(),
        Visibility::default(),
    )).id();
    
    players.push(TournamentPlayer {
        entity: human_entity,
        name: "You".to_string(),
        chips: STARTING_CHIPS,
        is_human: true,
        current_bet: 0,
        is_eliminated: false,
        tournament_stats: PlayerStats::default(),
    });

    // Spawn AI players
    for (name, personality) in ai_names {
        let ai_entity = commands.spawn((
            AiPlayer::new(personality),
            Transform::default(),
            Visibility::default(),
        )).id();
        
        players.push(TournamentPlayer {
            entity: ai_entity,
            name: name.to_string(),
            chips: STARTING_CHIPS,
            is_human: false,
            current_bet: 0,
            is_eliminated: false,
            tournament_stats: PlayerStats::default(),
        });
    }

    commands.insert_resource(TournamentState {
        players,
        current_round: 1,
        elimination_threshold: 50, // Eliminate when chips < 50
        prize_pool: STARTING_CHIPS * 6,
        tournament_timer: Timer::from_seconds(300.0, TimerMode::Once), // 5-minute rounds
    });
}
```

**Tournament Initialization:**
1. **Human Player**: Spawned with `Player` component for input handling
2. **AI Players**: Each spawned with their specific `AiPlayer` component
3. **Equal Starting Conditions**: All players start with `STARTING_CHIPS`
4. **Tournament Resource**: Tracks global state and progression

**Entity-Component Separation:**
- **Entities**: Unique IDs for each player
- **Components**: Different behavior (human vs AI)
- **Resource**: Tournament-wide state and rules

```rust
fn tournament_round_system(
    mut tournament_state: ResMut<TournamentState>,
    mut commands: Commands,
    time: Res<Time>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    tournament_state.tournament_timer.tick(time.delta());
    
    // Check for eliminations
    tournament_state.players.retain_mut(|player| {
        if player.chips < tournament_state.elimination_threshold && !player.is_eliminated {
            player.is_eliminated = true;
            info!("{} has been eliminated!", player.name);
            false  // Remove from active players
        } else {
            true
        }
    });
    
    // Check tournament end conditions
    let active_players = tournament_state.players.iter()
        .filter(|p| !p.is_eliminated)
        .count();
        
    if active_players <= 1 {
        // Tournament complete - declare winner
        if let Some(winner) = tournament_state.players.iter()
            .filter(|p| !p.is_eliminated)
            .next() 
        {
            info!("Tournament Winner: {}", winner.name);
            next_phase.set(GamePhase::TournamentComplete);
        }
    } else if tournament_state.tournament_timer.finished() {
        // Time limit reached - eliminate lowest chip counts
        tournament_state.players.sort_by(|a, b| b.chips.cmp(&a.chips));
        
        // Eliminate bottom player(s) each round
        let elimination_count = (active_players / 3).max(1);
        for player in tournament_state.players.iter_mut()
            .rev()  // Start from lowest chip counts
            .take(elimination_count) 
        {
            if !player.is_eliminated {
                player.is_eliminated = true;
                info!("{} eliminated by time limit!", player.name);
            }
        }
        
        // Start next round
        tournament_state.current_round += 1;
        tournament_state.tournament_timer = Timer::from_seconds(300.0, TimerMode::Once);
    }
}
```

**Elimination Mechanics:**

**Chip-Based Elimination:**
- Players below `elimination_threshold` are immediately eliminated
- Prevents extremely long tournaments
- Creates urgency when chip counts get low

**Time-Based Elimination:**
- 5-minute rounds with forced eliminations
- Bottom 1/3 of players eliminated each round
- Ensures tournament progression even with conservative play
- `.sort_by(|a, b| b.chips.cmp(&a.chips))` sorts by chip count (descending)

**Vec Methods Explained:**
- `.retain_mut()` keeps only players matching the condition
- `.filter()` creates an iterator of non-eliminated players
- `.rev()` reverses iteration order (lowest chips first)
- `.take(n)` limits to first n items

## Section 4: AI Decision Integration

Now let's integrate AI decision making with the tournament system:

```rust
fn ai_betting_system(
    mut tournament_state: ResMut<TournamentState>,
    mut ai_query: Query<&mut AiPlayer>,
    time: Res<Time>,
) {
    for player in &mut tournament_state.players {
        if !player.is_human && !player.is_eliminated {
            if let Ok(mut ai) = ai_query.get_mut(player.entity) {
                ai.decision_timer.tick(time.delta());
                
                if ai.decision_timer.finished() {
                    // Create decision context for this AI
                    let context = create_decision_context(&tournament_state, player);
                    
                    // Make betting decision
                    let desired_bet = ai.make_betting_decision(&context);
                    let affordable_bet = desired_bet.min(player.chips);
                    
                    player.current_bet = affordable_bet;
                    
                    // Reset decision timer for next decision
                    ai.decision_timer = Timer::from_seconds(
                        fastrand::f32() * 2.0 + 0.5,  // 0.5-2.5 second intervals
                        TimerMode::Once
                    );
                    
                    info!("{} bets ${}", player.name, affordable_bet);
                }
            }
        }
    }
}

fn create_decision_context(
    tournament_state: &TournamentState, 
    current_player: &TournamentPlayer
) -> DecisionContext {
    // Calculate tournament position
    let mut sorted_players = tournament_state.players.clone();
    sorted_players.sort_by(|a, b| b.chips.cmp(&a.chips));
    
    let position = sorted_players.iter()
        .position(|p| p.entity == current_player.entity)
        .unwrap_or(0) + 1;
    
    // Calculate chip position relative to field
    let total_chips: u32 = tournament_state.players.iter()
        .filter(|p| !p.is_eliminated)
        .map(|p| p.chips)
        .sum();
    let chip_position = current_player.chips as f32 / total_chips as f32;
    
    // Calculate pressure level
    let pressure_level = if current_player.chips < tournament_state.elimination_threshold * 2 {
        0.8  // High pressure when near elimination
    } else if position <= 2 {
        0.3  // Low pressure when leading
    } else {
        0.5  // Medium pressure otherwise
    };
    
    DecisionContext {
        current_card: None,  // Will be filled during actual play
        opponent_visible_cards: Vec::new(),  // Will be filled during play
        chip_position,
        tournament_position: position,
        recent_war_outcomes: Vec::new(),  // Tracked separately
        pressure_level,
    }
}
```

**Context Creation Logic:**

**Tournament Position Calculation:**
1. Clone and sort players by chip count
2. Find current player's position in sorted list
3. Add 1 because positions start at 1st place, not 0th

**Chip Position Calculation:**
- Sum all active players' chips
- Calculate current player's percentage of total
- Higher percentage = better position

**Pressure Level Algorithm:**
- **0.8**: Near elimination (chips < 2x threshold)
- **0.3**: Leading (top 2 positions)  
- **0.5**: Middle pack (default)

**Why Clone for Sorting?**
- `sorted_players.sort_by()` modifies the Vec
- We don't want to change the original tournament state
- Cloning gives us a temporary copy to sort

```rust
fn ai_war_decision_system(
    mut tournament_state: ResMut<TournamentState>,
    mut ai_query: Query<&mut AiPlayer>,
    mut war_events: EventWriter<WarDecision>,
    war_query: Query<&WarPhase>,
) {
    if war_query.is_empty() {
        return;  // No active war
    }

    for player in &mut tournament_state.players {
        if !player.is_human && !player.is_eliminated {
            if let Ok(mut ai) = ai_query.get_mut(player.entity) {
                let context = create_decision_context(&tournament_state, player);
                let go_to_war = ai.should_go_to_war(&context);
                
                war_events.send(WarDecision {
                    player_entity: player.entity,
                    go_to_war,
                });
                
                let decision_text = if go_to_war { "goes to war" } else { "surrenders" };
                info!("{} {}", player.name, decision_text);
                
                // Update AI's war statistics for learning
                if go_to_war {
                    ai.recent_wins += 1;  // Optimistic - will be corrected if they lose
                } else {
                    ai.recent_losses += 1;  // Surrendering counts as a loss
                }
            }
        }
    }
}
```

**War Decision Flow:**
1. Check if war is active (using `war_query.is_empty()`)
2. For each AI player, create decision context
3. Call `ai.should_go_to_war()` with context
4. Send `WarDecision` event with result
5. Update AI statistics for learning

**Optimistic Statistics:**
- AIs assume they'll win wars they enter
- Statistics get corrected when actual results come in
- Creates slight bias toward war (realistic overconfidence)

**Event-Driven Architecture:**
- AI decisions generate events
- Game logic responds to events
- Loose coupling between AI and game systems

## Section 5: Personality Showcase System

Let's create a system that highlights AI personalities during play:

```rust
// UI component for showing AI thoughts
#[derive(Component)]
struct AiThoughtBubble {
    player_entity: Entity,
    fade_timer: Timer,
}

fn display_ai_thoughts_system(
    mut commands: Commands,
    tournament_state: Res<TournamentState>,
    ai_query: Query<&AiPlayer>,
    mut thought_query: Query<(Entity, &mut AiThoughtBubble, &mut Text)>,
    time: Res<Time>,
) {
    // Update existing thought bubbles
    for (entity, mut bubble, mut text) in &mut thought_query {
        bubble.fade_timer.tick(time.delta());
        
        if bubble.fade_timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Create new thought bubbles for AI decisions
    for player in &tournament_state.players {
        if !player.is_human && !player.is_eliminated {
            if let Ok(ai) = ai_query.get(player.entity) {
                if ai.decision_timer.just_finished() {
                    let thought = generate_ai_thought(ai, player);
                    spawn_thought_bubble(&mut commands, player.entity, thought);
                }
            }
        }
    }
}

fn generate_ai_thought(ai: &AiPlayer, player: &TournamentPlayer) -> String {
    match ai.personality {
        AiPersonality::Conservative => {
            if player.chips < 200 {
                "I need to play it safe...".to_string()
            } else {
                "Steady wins the race.".to_string()
            }
        },
        
        AiPersonality::Aggressive => {
            if ai.recent_wins > ai.recent_losses {
                "Time to crush them!".to_string()
            } else {
                "Go big or go home!".to_string()
            }
        },
        
        AiPersonality::Balanced => {
            format!("Position: {}. Calculated risk.", 
                match player.current_bet {
                    0..=10 => "Conservative",
                    11..=30 => "Moderate", 
                    _ => "Aggressive"
                }
            )
        },
        
        AiPersonality::Adaptive => {
            format!("Strategy: {:?}. Adapting...", ai.current_strategy)
        },
        
        AiPersonality::Chaos => {
            let thoughts = [
                "YOLO!",
                "Feeling lucky!",
                "Chaos reigns!",
                "Why not?",
                "Random is fun!",
            ];
            thoughts[fastrand::usize(0..thoughts.len())].to_string()
        }
    }
}

fn spawn_thought_bubble(
    commands: &mut Commands,
    player_entity: Entity,
    thought: String,
) {
    commands.spawn((
        AiThoughtBubble {
            player_entity,
            fade_timer: Timer::from_seconds(3.0, TimerMode::Once),
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(100.0),
            left: Val::Px(fastrand::f32() * 400.0 + 100.0),  // Random position
            padding: UiRect::all(Val::Px(8.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        BorderColor(Color::srgb(0.8, 0.8, 0.8)),
    ))
    .with_child((
        Text::new(thought),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}
```

**Thought Bubble System:**
- Shows AI "reasoning" to players
- Different personalities have different thought patterns
- Helps players understand AI decision making
- Adds personality and humor to the game

**Dynamic Content Generation:**
- **Conservative**: Cautious, safety-focused thoughts
- **Aggressive**: Bold, intimidating thoughts  
- **Balanced**: Mathematical, analytical thoughts
- **Adaptive**: Strategy-focused, learning thoughts
- **Chaos**: Random, unpredictable thoughts

**UI Positioning:**
- `position_type: PositionType::Absolute` removes from layout flow
- Random X position creates variety
- 3-second fade timer prevents screen clutter

## Section 6: Tournament Statistics and Learning

Let's implement performance tracking that affects AI behavior:

```rust
fn update_tournament_stats(
    mut tournament_state: ResMut<TournamentState>,
    mut ai_query: Query<&mut AiPlayer>,
    round_results: Res<RoundResults>,
) {
    for result in &round_results.results {
        // Find the player in tournament
        if let Some(player) = tournament_state.players.iter_mut()
            .find(|p| p.entity == result.player_entity) 
        {
            // Update tournament stats
            if result.won {
                player.tournament_stats.hands_won += 1;
                player.tournament_stats.total_winnings += result.winnings as i32;
                player.chips = player.chips.saturating_add(result.winnings);
                
                if result.was_war {
                    player.tournament_stats.wars_won += 1;
                }
                
                if result.winnings > player.tournament_stats.biggest_win {
                    player.tournament_stats.biggest_win = result.winnings;
                }
            } else {
                player.tournament_stats.hands_lost += 1;
                player.tournament_stats.total_winnings -= result.losses as i32;
                player.chips = player.chips.saturating_sub(result.losses);
                
                if result.was_war {
                    player.tournament_stats.wars_lost += 1;
                }
            }
            
            // Update AI learning if this is an AI player
            if !player.is_human {
                if let Ok(mut ai) = ai_query.get_mut(player.entity) {
                    // Update recent performance tracking
                    if result.won {
                        ai.recent_wins += 1;
                    } else {
                        ai.recent_losses += 1;
                    }
                    
                    // Keep only recent history (last 10 results)
                    if ai.recent_wins + ai.recent_losses > 10 {
                        ai.recent_wins = (ai.recent_wins * 7) / 10;  // 70% weight
                        ai.recent_losses = (ai.recent_losses * 7) / 10;
                    }
                    
                    // Adaptive learning for strategy adjustment
                    if ai.personality == AiPersonality::Adaptive {
                        ai.adapt_to_results(&result);
                    }
                }
            }
        }
    }
}

impl AiPlayer {
    fn adapt_to_results(&mut self, result: &RoundResult) {
        match self.current_strategy {
            AiStrategy::OpponentModeling => {
                // If losing, try to counter successful opponent strategies
                if !result.won && self.recent_losses > self.recent_wins {
                    self.current_strategy = AiStrategy::CounterStrategy;
                }
            },
            
            AiStrategy::CounterStrategy => {
                // If still losing, go back to position-aware play
                if !result.won && self.recent_losses > self.recent_wins + 2 {
                    self.current_strategy = AiStrategy::PositionAware;
                }
            },
            
            AiStrategy::PositionAware => {
                // If winning, maintain strategy; if losing badly, try opponent modeling
                if self.recent_losses > self.recent_wins + 3 {
                    self.current_strategy = AiStrategy::OpponentModeling;
                }
            },
            
            _ => {
                // Default adaptation - switch to position-aware if struggling
                if self.recent_losses > self.recent_wins + 2 {
                    self.current_strategy = AiStrategy::PositionAware;
                }
            }
        }
        
        // Adjust risk tolerance based on performance
        let win_rate = if (self.recent_wins + self.recent_losses) > 0 {
            self.recent_wins as f32 / (self.recent_wins + self.recent_losses) as f32
        } else {
            0.5
        };
        
        // Successful AIs become slightly more aggressive
        // Unsuccessful AIs become slightly more conservative
        self.risk_tolerance = (self.risk_tolerance * 0.95 + win_rate * 0.05)
            .clamp(0.1, 0.9);  // Keep within reasonable bounds
    }
}
```

**Performance Tracking Deep Dive:**

**saturating_add/saturating_sub:**
- Prevents integer overflow/underflow
- `saturating_add(x)` caps at u32::MAX instead of wrapping
- `saturating_sub(x)` caps at 0 instead of wrapping
- Essential for chip arithmetic that can't go negative

**Recent History Management:**
```rust
if ai.recent_wins + ai.recent_losses > 10 {
    ai.recent_wins = (ai.recent_wins * 7) / 10;  // 70% weight
    ai.recent_losses = (ai.recent_losses * 7) / 10;
}
```
- Keeps only last 10 results for "recent" performance
- Multiplies by 0.7 to maintain proportion while reducing count
- Prevents infinite accumulation of history

**Adaptive Strategy Switching:**
- **OpponentModeling** → **CounterStrategy** when losing
- **CounterStrategy** → **PositionAware** when still losing  
- **PositionAware** → **OpponentModeling** when losing badly
- Creates a learning cycle that prevents getting stuck

**Risk Tolerance Learning:**
```rust
self.risk_tolerance = (self.risk_tolerance * 0.95 + win_rate * 0.05).clamp(0.1, 0.9);
```
- 95% current risk tolerance + 5% win rate influence
- Slow adaptation prevents wild swings
- Clamped to [0.1, 0.9] to maintain personality characteristics

## Testing Your Tournament System

At this point, you should be able to:

1. **Start Tournament**: See 6 players with different names and personalities
2. **Watch AI Decisions**: Each bot makes bets with different patterns
3. **See Thought Bubbles**: AI personalities show through their "thoughts"
4. **Observe Adaptation**: Adaptive bot changes strategies based on performance
5. **Track Eliminations**: Players get eliminated as tournament progresses

## Key Concepts Mastered

1. **AI Architecture**: Separation of personality, strategy, and decisions
   - Personalities provide consistent temperament
   - Strategies adapt to game situations
   - Decisions consider multiple context factors

2. **Multi-Agent Systems**: Managing multiple AI entities
   - Each AI makes independent decisions
   - Tournament system coordinates all players
   - Event-driven communication between systems

3. **Machine Learning Concepts**: Simple adaptive behavior
   - Performance tracking affects future decisions
   - Strategy switching based on results
   - Risk tolerance evolution over time

4. **Psychological Modeling**: Human-like decision factors
   - Pressure affects decision making
   - Recent performance influences confidence
   - Different personalities react differently to stress

5. **Tournament Management**: Complex multi-player coordination
   - Elimination mechanics
   - Time-based progression
   - Statistical tracking

## Exercises

1. **Create Custom Personality**: Design a "Copycat" bot that mimics the human player's betting patterns

2. **Implement Team Alliances**: Allow AIs to form temporary partnerships against leading players

3. **Add Bluffing System**: Let AIs place fake bets to mislead opponents about their card strength

4. **Dynamic Difficulty**: Adjust AI intelligence based on human player's win rate

5. **Tournament Bracketing**: Implement Swiss-system tournament rounds instead of elimination

## Next Steps

In Part 8, we'll add:
- Real-time leaderboard displaying live rankings
- Performance metrics and trend analysis
- Position-aware visual indicators
- Competitive intelligence showing who's winning and losing

The AI foundation we've built will make these leaderboard features even more engaging as players can track how different AI personalities perform over time!