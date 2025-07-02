# Casino War Tutorial - Part 14: Custom AI Training - Create Your Own Opponents

## What We're Building in Part 14

Creating a sophisticated AI training and customization system:

1. **Behavior Cloning**: Train AI by learning from human/replay data
2. **Personality Designer**: Create custom AI personalities with sliders
3. **Training Arena**: Test and refine AI behaviors in controlled environments
4. **Evolution System**: AI that improves through genetic algorithms
5. **Neural Network Training**: Deep learning for advanced AI behaviors

## Understanding AI Training Systems

### The Custom AI Problem

Imagine you're creating a fighting game where players can train their own AI fighters. We need a system that:
- Learns from player behavior and successful strategies
- Allows fine-tuning of personality traits
- Provides tools to test and iterate on AI design
- Enables sharing of custom AIs with the community
- Balances between too easy and impossibly difficult

Let's think about this like training a virtual pet that plays cards:
1. **Observation**: Watch and learn from examples
2. **Personality**: Core traits that define behavior
3. **Training**: Practice in controlled scenarios
4. **Evolution**: Improve through trial and error
5. **Refinement**: Fine-tune based on performance

In programming terms, this is a **machine learning pipeline** combined with **genetic algorithms** and **behavior trees**. We need to capture, process, and reproduce intelligent behavior.

## Section 1: AI Training Architecture

First, let's design our AI training framework. Think of this like creating a dojo for AI students:

```rust
// Core AI training system
#[derive(Resource)]
struct AITrainingSystem {
    training_sessions: HashMap<Uuid, TrainingSession>,
    behavior_library: BehaviorLibrary,
    neural_network_trainer: NeuralNetworkTrainer,
    genetic_evolver: GeneticEvolver,
    validation_arena: ValidationArena,
}

#[derive(Debug, Clone)]
struct TrainingSession {
    session_id: Uuid,
    ai_name: String,
    base_personality: PersonalityTemplate,
    
    // Training data
    training_replays: Vec<ReplayData>,
    behavior_examples: Vec<BehaviorExample>,
    
    // Current state
    current_model: TrainedModel,
    training_progress: TrainingProgress,
    performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersonalityTemplate {
    // Core personality traits (0.0 to 1.0)
    aggression: f32,
    caution: f32,
    adaptability: f32,
    unpredictability: f32,
    intelligence: f32,
    
    // Behavioral tendencies
    risk_appetite: RiskProfile,
    decision_speed: DecisionSpeed,
    learning_style: LearningStyle,
    
    // Special traits
    special_abilities: Vec<SpecialAbility>,
    weaknesses: Vec<Weakness>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum RiskProfile {
    Conservative { threshold: f32 },
    Moderate { variance: f32 },
    Aggressive { multiplier: f32 },
    Dynamic { adaptation_rate: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LearningStyle {
    Imitation,      // Copies successful strategies
    Innovation,     // Tries new approaches
    Analytical,     // Deep analysis before decisions
    Intuitive,      // Fast, pattern-based decisions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SpecialAbility {
    BluffMaster { effectiveness: f32 },
    ChipCounter { accuracy: f32 },
    PressurePlayer { intimidation: f32 },
    Comeback { trigger_threshold: f32 },
    Endurance { fatigue_resistance: f32 },
}

// Behavior cloning from replays
struct BehaviorCloner {
    feature_extractor: FeatureExtractor,
    decision_tree: DecisionTree,
    imitation_threshold: f32,
}

impl BehaviorCloner {
    fn learn_from_replay(
        &mut self,
        replay: &TournamentReplay,
        target_player: Uuid,
    ) -> Result<LearnedBehaviors, LearningError> {
        let mut learned = LearnedBehaviors::default();
        
        // Extract decision points from replay
        let decisions = self.extract_decisions(replay, target_player);
        
        for decision in decisions {
            // Extract features at decision point
            let features = self.feature_extractor.extract(&decision.context);
            
            // Add to decision tree
            self.decision_tree.add_example(features, decision.action);
            
            // Identify patterns
            if let Some(pattern) = self.identify_pattern(&decision) {
                learned.patterns.push(pattern);
            }
        }
        
        // Prune decision tree for generalization
        self.decision_tree.prune(self.imitation_threshold);
        
        Ok(learned)
    }
    
    fn extract_decisions(&self, replay: &TournamentReplay, player_id: Uuid) -> Vec<DecisionPoint> {
        let mut decisions = Vec::new();
        
        for (idx, event) in replay.events.iter().enumerate() {
            if let ReplayEvent::PlayerAction { player_id: pid, action } = &event.event {
                if *pid == player_id {
                    // Build context from surrounding events
                    let context = self.build_context(replay, idx);
                    
                    decisions.push(DecisionPoint {
                        timestamp: event.timestamp,
                        context,
                        action: action.clone(),
                        outcome: self.evaluate_outcome(replay, idx),
                    });
                }
            }
        }
        
        decisions
    }
}

// Neural network architecture for AI
#[derive(Debug, Clone)]
struct AINeuralNetwork {
    input_layer: Layer,
    hidden_layers: Vec<Layer>,
    output_layer: Layer,
    activation: ActivationFunction,
    
    // Training parameters
    learning_rate: f32,
    momentum: f32,
    regularization: f32,
}

impl AINeuralNetwork {
    fn new(config: &NetworkConfig) -> Self {
        Self {
            input_layer: Layer::new(config.input_size, config.hidden_sizes[0]),
            hidden_layers: config.hidden_sizes.windows(2)
                .map(|w| Layer::new(w[0], w[1]))
                .collect(),
            output_layer: Layer::new(
                *config.hidden_sizes.last().unwrap(),
                config.output_size
            ),
            activation: config.activation,
            learning_rate: config.learning_rate,
            momentum: config.momentum,
            regularization: config.regularization,
        }
    }
    
    fn forward(&self, input: &Tensor) -> Tensor {
        let mut activation = self.input_layer.forward(input);
        activation = self.activation.apply(&activation);
        
        for hidden in &self.hidden_layers {
            activation = hidden.forward(&activation);
            activation = self.activation.apply(&activation);
        }
        
        self.output_layer.forward(&activation)
    }
    
    fn train_batch(
        &mut self,
        inputs: &[Tensor],
        targets: &[Tensor],
    ) -> f32 {
        let mut total_loss = 0.0;
        
        for (input, target) in inputs.iter().zip(targets.iter()) {
            // Forward pass
            let output = self.forward(input);
            
            // Calculate loss
            let loss = self.calculate_loss(&output, target);
            total_loss += loss;
            
            // Backward pass
            let gradients = self.backward(&output, target);
            
            // Update weights
            self.update_weights(&gradients);
        }
        
        total_loss / inputs.len() as f32
    }
}

// Training data structures
#[derive(Debug, Clone)]
struct BehaviorExample {
    situation: GameSituation,
    decision: AIDecision,
    reasoning: String,
    success_rating: f32,
}

#[derive(Debug, Clone)]
struct GameSituation {
    chip_count: u32,
    opponent_chips: Vec<u32>,
    current_bet: u32,
    tournament_position: usize,
    time_remaining: f32,
    recent_history: Vec<HandResult>,
}

#[derive(Debug, Clone)]
struct AIDecision {
    action: Action,
    confidence: f32,
    alternatives_considered: Vec<(Action, f32)>,
    decision_time: f32,
}
```

**AI Training Architecture:**

**Multi-System Approach:**
- **Behavior Cloning**: Learn from examples
- **Neural Networks**: Deep decision making
- **Genetic Algorithms**: Evolution through competition
- **Decision Trees**: Interpretable logic

**Personality System:**
- Core traits define behavior tendencies
- Special abilities for unique playstyles
- Weaknesses for balance and character
- Risk profiles for varied strategies

**Training Data Management:**
- Store examples from replays
- Track decision contexts
- Measure outcome success
- Build comprehensive datasets

## Section 2: Personality Designer Interface

Now let's create the UI for designing custom AI personalities:

```rust
// AI personality designer UI
#[derive(Component)]
struct PersonalityDesigner;

#[derive(Component)]
struct TraitSlider {
    trait_type: PersonalityTrait,
    min_value: f32,
    max_value: f32,
    current_value: f32,
}

#[derive(Debug, Clone, Copy)]
enum PersonalityTrait {
    Aggression,
    Caution,
    Adaptability,
    Unpredictability,
    Intelligence,
}

fn setup_personality_designer(mut commands: Commands) {
    commands.spawn((
        PersonalityDesigner,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
    ))
    .with_children(|parent| {
        // Left panel - Trait sliders
        spawn_trait_panel(parent);
        
        // Center - AI preview
        spawn_ai_preview(parent);
        
        // Right panel - Ability selection
        spawn_ability_panel(parent);
    });
}

fn spawn_trait_panel(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Personality Traits"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        // Trait sliders
        let traits = [
            ("Aggression", PersonalityTrait::Aggression, Color::srgb(0.8, 0.2, 0.2)),
            ("Caution", PersonalityTrait::Caution, Color::srgb(0.2, 0.6, 0.2)),
            ("Adaptability", PersonalityTrait::Adaptability, Color::srgb(0.2, 0.4, 0.8)),
            ("Unpredictability", PersonalityTrait::Unpredictability, Color::srgb(0.8, 0.2, 0.8)),
            ("Intelligence", PersonalityTrait::Intelligence, Color::srgb(0.8, 0.8, 0.2)),
        ];
        
        for (name, trait_type, color) in traits {
            spawn_trait_slider(parent, name, trait_type, color);
        }
    });
}

fn spawn_trait_slider(
    parent: &mut ChildBuilder,
    name: &str,
    trait_type: PersonalityTrait,
    color: Color,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            flex_direction: FlexDirection::Column,
            margin: UiRect::vertical(Val::Px(10.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        // Label with value
        parent.spawn((
            Node {
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            
            parent.spawn((
                Text::new("50"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
                TraitValueDisplay { trait_type },
            ));
        });
        
        // Slider track
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(20.0),
                margin: UiRect::top(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            TraitSlider {
                trait_type,
                min_value: 0.0,
                max_value: 100.0,
                current_value: 50.0,
            },
        ))
        .with_children(|parent| {
            // Slider handle
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    left: Val::Percent(50.0),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(color),
                SliderHandle { trait_type },
            ));
        });
    });
}

// AI preview visualization
fn spawn_ai_preview(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        // AI name input
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
        ))
        .with_child((
            Text::new("Enter AI Name..."),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        // Personality visualization
        parent.spawn((
            Node {
                width: Val::Px(400.0),
                height: Val::Px(400.0),
                margin: UiRect::vertical(Val::Px(20.0)),
                align_self: AlignSelf::Center,
                ..default()
            },
            PersonalityRadarChart,
        ));
        
        // Predicted behavior
        spawn_behavior_preview(parent);
    });
}

// Radar chart for personality visualization
fn render_personality_radar_chart(
    personality: &PersonalityTemplate,
    gizmos: &mut Gizmos,
    center: Vec2,
    radius: f32,
) {
    let traits = [
        ("Aggression", personality.aggression),
        ("Caution", personality.caution),
        ("Adaptability", personality.adaptability),
        ("Unpredictability", personality.unpredictability),
        ("Intelligence", personality.intelligence),
    ];
    
    let num_traits = traits.len();
    let angle_step = std::f32::consts::TAU / num_traits as f32;
    
    // Draw axes
    for i in 0..num_traits {
        let angle = i as f32 * angle_step - std::f32::consts::FRAC_PI_2;
        let end = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        gizmos.line_2d(center, end, Color::srgb(0.3, 0.3, 0.3));
        
        // Draw concentric circles
        for level in 1..=5 {
            let level_radius = radius * (level as f32 / 5.0);
            let start_angle = angle;
            let end_angle = angle + angle_step;
            
            let start = center + Vec2::new(start_angle.cos(), start_angle.sin()) * level_radius;
            let end = center + Vec2::new(end_angle.cos(), end_angle.sin()) * level_radius;
            gizmos.line_2d(start, end, Color::srgb(0.2, 0.2, 0.2));
        }
    }
    
    // Draw personality shape
    let mut points = Vec::new();
    for (i, (_, value)) in traits.iter().enumerate() {
        let angle = i as f32 * angle_step - std::f32::consts::FRAC_PI_2;
        let distance = radius * value;
        let point = center + Vec2::new(angle.cos(), angle.sin()) * distance;
        points.push(point);
    }
    
    // Connect points
    for i in 0..points.len() {
        let next = (i + 1) % points.len();
        gizmos.line_2d(points[i], points[next], Color::srgb(0.8, 0.4, 0.0));
    }
    
    // Fill area (simplified - would use mesh in real implementation)
    for point in &points {
        gizmos.circle_2d(*point, 3.0, Color::srgb(1.0, 0.5, 0.0));
    }
}

// Behavior prediction based on personality
fn spawn_behavior_preview(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Predicted Behaviors"),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
        
        // Behavior predictions
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
            BehaviorPredictionList,
        ));
    });
}

// Real-time behavior prediction
fn update_behavior_predictions(
    personality_query: Query<&PersonalityTemplate>,
    mut prediction_query: Query<&mut Text, With<BehaviorPredictionList>>,
) {
    if let Ok(personality) = personality_query.get_single() {
        let predictions = generate_behavior_predictions(personality);
        
        if let Ok(mut text) = prediction_query.get_single_mut() {
            text.0 = predictions.join("\n");
        }
    }
}

fn generate_behavior_predictions(personality: &PersonalityTemplate) -> Vec<String> {
    let mut predictions = Vec::new();
    
    // Betting behavior
    if personality.aggression > 0.7 {
        predictions.push("• Will make large bets frequently".to_string());
    } else if personality.caution > 0.7 {
        predictions.push("• Will bet conservatively, rarely going all-in".to_string());
    }
    
    // War decisions
    let war_tendency = personality.aggression - personality.caution + 0.5;
    if war_tendency > 0.7 {
        predictions.push("• Almost always goes to war on ties".to_string());
    } else if war_tendency < 0.3 {
        predictions.push("• Rarely goes to war, preferring to surrender".to_string());
    }
    
    // Adaptation
    if personality.adaptability > 0.8 {
        predictions.push("• Quickly adjusts strategy based on opponents".to_string());
    } else if personality.adaptability < 0.3 {
        predictions.push("• Sticks to same strategy regardless of situation".to_string());
    }
    
    // Pressure response
    if personality.intelligence > 0.7 && personality.caution > 0.5 {
        predictions.push("• Performs well under pressure".to_string());
    } else if personality.unpredictability > 0.8 {
        predictions.push("• Becomes erratic when losing".to_string());
    }
    
    predictions
}
```

**Personality Designer Features:**

**Interactive Sliders:**
- Real-time personality adjustment
- Visual feedback with colors
- Smooth drag controls
- Value display updates

**Radar Chart Visualization:**
- 5-axis personality display
- Shows trait balance
- Immediate visual feedback
- Professional appearance

**Behavior Prediction:**
- Real-time behavior analysis
- Natural language descriptions
- Based on trait combinations
- Helps understand AI personality

## Section 3: Training Arena

Let's create a controlled environment for testing and training AI:

```rust
// Training arena for AI development
#[derive(Resource)]
struct TrainingArena {
    arena_id: Uuid,
    training_config: TrainingConfig,
    active_sessions: Vec<TrainingSessionHandle>,
    
    // Performance tracking
    session_results: Vec<SessionResult>,
    aggregate_stats: AggregateStatistics,
}

#[derive(Debug, Clone)]
struct TrainingConfig {
    // Opponent configuration
    opponent_type: OpponentType,
    difficulty_progression: DifficultyProgression,
    
    // Training parameters
    rounds_per_session: u32,
    time_limit_per_round: Option<Duration>,
    
    // Scenario configuration
    scenarios: Vec<TrainingScenario>,
    randomization: RandomizationSettings,
    
    // Success criteria
    success_metrics: Vec<SuccessMetric>,
    graduation_threshold: f32,
}

#[derive(Debug, Clone)]
enum OpponentType {
    FixedPersonality(PersonalityTemplate),
    RandomPersonalities,
    AdaptiveOpponent,
    PreviousVersions,  // Train against older versions of self
    HumanReplays,      // Train against human replay data
}

#[derive(Debug, Clone)]
enum TrainingScenario {
    // Specific situations to master
    LowChipRecovery {
        starting_chips: u32,
        opponent_chips: u32,
    },
    
    HighPressure {
        time_limit: Duration,
        elimination_threshold: u32,
    },
    
    WarDecisions {
        force_tie_frequency: f32,
    },
    
    EndgameStrategy {
        starting_players: usize,
        target_position: usize,
    },
    
    AdversarialPlay {
        opponent_knows_cards: bool,
    },
}

// Training session runner
fn run_training_session(
    mut arena: ResMut<TrainingArena>,
    mut ai_trainer: ResMut<AITrainingSystem>,
    time: Res<Time>,
) {
    for session_handle in &mut arena.active_sessions {
        if let Some(session) = ai_trainer.training_sessions.get_mut(&session_handle.session_id) {
            // Update training
            match session_handle.state {
                SessionState::Running => {
                    // Execute training round
                    let round_result = execute_training_round(session, &arena.training_config);
                    
                    // Update AI based on result
                    update_ai_from_result(session, &round_result);
                    
                    // Check completion
                    if session.training_progress.rounds_completed >= arena.training_config.rounds_per_session {
                        session_handle.state = SessionState::Evaluating;
                    }
                },
                
                SessionState::Evaluating => {
                    // Run evaluation matches
                    let evaluation = evaluate_trained_ai(session, &arena.training_config);
                    
                    // Check if graduated
                    if evaluation.score >= arena.training_config.graduation_threshold {
                        session_handle.state = SessionState::Graduated;
                        info!("AI {} graduated with score {:.2}!", 
                              session.ai_name, evaluation.score);
                    } else {
                        // Continue training
                        session_handle.state = SessionState::Running;
                        adjust_training_difficulty(session, &evaluation);
                    }
                },
                
                _ => {}
            }
        }
    }
}

// Scenario-based training
fn execute_training_round(
    session: &mut TrainingSession,
    config: &TrainingConfig,
) -> RoundResult {
    // Select scenario
    let scenario = select_training_scenario(session, config);
    
    // Create game state for scenario
    let mut game_state = create_scenario_state(&scenario);
    
    // Run the round
    let mut decisions_made = Vec::new();
    let mut round_complete = false;
    
    while !round_complete {
        // Get AI decision
        let situation = extract_game_situation(&game_state);
        let decision = session.current_model.make_decision(&situation);
        
        decisions_made.push((situation.clone(), decision.clone()));
        
        // Apply decision
        apply_ai_decision(&mut game_state, &decision);
        
        // Check round completion
        round_complete = is_round_complete(&game_state);
    }
    
    // Evaluate performance
    let performance = evaluate_round_performance(&game_state, &decisions_made, &scenario);
    
    RoundResult {
        scenario,
        decisions: decisions_made,
        performance,
        game_state,
    }
}

// Reinforcement learning update
fn update_ai_from_result(
    session: &mut TrainingSession,
    result: &RoundResult,
) {
    // Calculate rewards for each decision
    let rewards = calculate_decision_rewards(result);
    
    // Update neural network if using NN
    if let TrainedModel::NeuralNetwork(ref mut nn) = session.current_model {
        for ((situation, decision), reward) in result.decisions.iter().zip(rewards.iter()) {
            // Convert to training example
            let input = situation_to_tensor(situation);
            let target = decision_to_tensor(decision, *reward);
            
            // Train on this example
            nn.train_single(&input, &target);
        }
    }
    
    // Update decision tree if using DT
    if let TrainedModel::DecisionTree(ref mut dt) = session.current_model {
        for ((situation, decision), reward) in result.decisions.iter().zip(rewards.iter()) {
            if *reward > 0.0 {
                // Reinforce good decisions
                dt.strengthen_path(situation, decision);
            } else {
                // Weaken bad decisions
                dt.weaken_path(situation, decision);
            }
        }
    }
    
    // Update training progress
    session.training_progress.rounds_completed += 1;
    session.training_progress.total_reward += rewards.iter().sum::<f32>();
}

// Genetic algorithm for AI evolution
#[derive(Debug, Clone)]
struct GeneticEvolver {
    population_size: usize,
    mutation_rate: f32,
    crossover_rate: f32,
    elitism_count: usize,
    
    current_generation: Vec<AIGenome>,
    generation_number: u32,
    best_genome_ever: Option<(AIGenome, f32)>,
}

#[derive(Debug, Clone)]
struct AIGenome {
    genes: Vec<f32>,  // Encoded personality and network weights
    fitness: Option<f32>,
    lineage: Vec<Uuid>,  // Track ancestry
}

impl GeneticEvolver {
    fn evolve_generation(&mut self) -> Vec<AIGenome> {
        // Evaluate fitness if needed
        self.evaluate_fitness();
        
        // Sort by fitness
        self.current_generation.sort_by(|a, b| 
            b.fitness.unwrap_or(0.0).partial_cmp(&a.fitness.unwrap_or(0.0)).unwrap()
        );
        
        // Track best ever
        if let Some(fitness) = self.current_generation[0].fitness {
            if self.best_genome_ever.is_none() || 
               fitness > self.best_genome_ever.as_ref().unwrap().1 {
                self.best_genome_ever = Some((self.current_generation[0].clone(), fitness));
            }
        }
        
        let mut new_generation = Vec::new();
        
        // Elitism - keep best individuals
        for i in 0..self.elitism_count {
            new_generation.push(self.current_generation[i].clone());
        }
        
        // Create rest through crossover and mutation
        while new_generation.len() < self.population_size {
            // Tournament selection
            let parent1 = self.tournament_select();
            let parent2 = self.tournament_select();
            
            // Crossover
            let mut child = if fastrand::f32() < self.crossover_rate {
                self.crossover(&parent1, &parent2)
            } else {
                parent1.clone()
            };
            
            // Mutation
            if fastrand::f32() < self.mutation_rate {
                self.mutate(&mut child);
            }
            
            new_generation.push(child);
        }
        
        self.generation_number += 1;
        self.current_generation = new_generation.clone();
        
        new_generation
    }
    
    fn crossover(&self, parent1: &AIGenome, parent2: &AIGenome) -> AIGenome {
        let crossover_point = fastrand::usize(..parent1.genes.len());
        
        let mut child_genes = Vec::new();
        
        // Take first part from parent1, second from parent2
        child_genes.extend_from_slice(&parent1.genes[..crossover_point]);
        child_genes.extend_from_slice(&parent2.genes[crossover_point..]);
        
        AIGenome {
            genes: child_genes,
            fitness: None,
            lineage: vec![parent1.lineage.last().copied().unwrap_or_default(),
                         parent2.lineage.last().copied().unwrap_or_default()],
        }
    }
    
    fn mutate(&self, genome: &mut AIGenome) {
        for gene in &mut genome.genes {
            if fastrand::f32() < 0.1 {  // 10% chance per gene
                // Gaussian mutation
                let mutation = fastrand::f32() * 0.2 - 0.1;  // [-0.1, 0.1]
                *gene = (*gene + mutation).clamp(0.0, 1.0);
            }
        }
    }
}
```

**Training Arena Features:**

**Scenario-Based Training:**
- Specific situations to master
- Progressive difficulty
- Controlled environments
- Measurable objectives

**Multiple Training Methods:**
- Reinforcement learning
- Supervised learning from examples
- Genetic evolution
- Hybrid approaches

**Performance Tracking:**
- Round-by-round analysis
- Aggregate statistics
- Progress visualization
- Graduation criteria

## Section 4: AI Behavior Tree System

Let's implement a behavior tree system for complex AI logic:

```rust
// Behavior tree architecture
#[derive(Debug, Clone)]
struct BehaviorTree {
    root: Box<dyn BehaviorNode>,
    blackboard: Blackboard,
}

#[derive(Debug, Clone)]
struct Blackboard {
    data: HashMap<String, BlackboardValue>,
}

#[derive(Debug, Clone)]
enum BlackboardValue {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Entity(Entity),
}

trait BehaviorNode: Debug + Send + Sync {
    fn execute(&mut self, context: &mut AIContext, blackboard: &mut Blackboard) -> NodeStatus;
    fn reset(&mut self);
    fn clone_node(&self) -> Box<dyn BehaviorNode>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeStatus {
    Success,
    Failure,
    Running,
}

// Composite nodes
#[derive(Debug, Clone)]
struct SequenceNode {
    children: Vec<Box<dyn BehaviorNode>>,
    current_child: usize,
}

impl BehaviorNode for SequenceNode {
    fn execute(&mut self, context: &mut AIContext, blackboard: &mut Blackboard) -> NodeStatus {
        while self.current_child < self.children.len() {
            match self.children[self.current_child].execute(context, blackboard) {
                NodeStatus::Success => {
                    self.current_child += 1;
                },
                NodeStatus::Failure => {
                    self.reset();
                    return NodeStatus::Failure;
                },
                NodeStatus::Running => {
                    return NodeStatus::Running;
                },
            }
        }
        
        self.reset();
        NodeStatus::Success
    }
    
    fn reset(&mut self) {
        self.current_child = 0;
        for child in &mut self.children {
            child.reset();
        }
    }
    
    fn clone_node(&self) -> Box<dyn BehaviorNode> {
        Box::new(self.clone())
    }
}

// Selector (OR) node
#[derive(Debug, Clone)]
struct SelectorNode {
    children: Vec<Box<dyn BehaviorNode>>,
    current_child: usize,
}

// Decorator nodes
#[derive(Debug, Clone)]
struct RepeatNode {
    child: Box<dyn BehaviorNode>,
    times: usize,
    current_count: usize,
}

#[derive(Debug, Clone)]
struct ConditionalNode {
    condition: Box<dyn Condition>,
    child: Box<dyn BehaviorNode>,
}

// Leaf nodes (actions)
#[derive(Debug, Clone)]
struct PlaceBetNode {
    bet_strategy: BetStrategy,
}

impl BehaviorNode for PlaceBetNode {
    fn execute(&mut self, context: &mut AIContext, blackboard: &mut Blackboard) -> NodeStatus {
        // Calculate bet based on strategy
        let bet_amount = match self.bet_strategy {
            BetStrategy::Conservative => {
                let min_bet = blackboard.get_int("min_bet").unwrap_or(10);
                min_bet as u32
            },
            BetStrategy::Aggressive => {
                let chips = context.get_chip_count();
                (chips as f32 * 0.3) as u32  // 30% of chips
            },
            BetStrategy::Calculated => {
                // Complex calculation based on multiple factors
                calculate_optimal_bet(context, blackboard)
            },
        };
        
        // Place the bet
        context.place_bet(bet_amount);
        
        NodeStatus::Success
    }
    
    fn reset(&mut self) {}
    
    fn clone_node(&self) -> Box<dyn BehaviorNode> {
        Box::new(self.clone())
    }
}

// Behavior tree builder for easy construction
struct BehaviorTreeBuilder {
    nodes: Vec<Box<dyn BehaviorNode>>,
}

impl BehaviorTreeBuilder {
    fn new() -> Self {
        Self { nodes: Vec::new() }
    }
    
    fn sequence(mut self) -> Self {
        let sequence = SequenceNode {
            children: self.nodes,
            current_child: 0,
        };
        self.nodes = vec![Box::new(sequence)];
        self
    }
    
    fn selector(mut self) -> Self {
        let selector = SelectorNode {
            children: self.nodes,
            current_child: 0,
        };
        self.nodes = vec![Box::new(selector)];
        self
    }
    
    fn condition<C: Condition + 'static>(mut self, condition: C) -> Self {
        if let Some(child) = self.nodes.pop() {
            let conditional = ConditionalNode {
                condition: Box::new(condition),
                child,
            };
            self.nodes.push(Box::new(conditional));
        }
        self
    }
    
    fn action<N: BehaviorNode + 'static>(mut self, node: N) -> Self {
        self.nodes.push(Box::new(node));
        self
    }
    
    fn build(mut self) -> BehaviorTree {
        let root = self.nodes.pop().expect("No root node");
        BehaviorTree {
            root,
            blackboard: Blackboard { data: HashMap::new() },
        }
    }
}

// Example: Building a complex AI behavior
fn create_tournament_ai_behavior() -> BehaviorTree {
    BehaviorTreeBuilder::new()
        .selector()  // Root selector
            // Branch 1: Emergency situations
            .sequence()
                .condition(LowChipsCondition { threshold: 100 })
                .action(AllInStrategyNode)
            .selector()  // Back to main selector
            
            // Branch 2: Normal play
            .sequence()
                .action(AnalyzeSituationNode)
                .selector()
                    // Sub-branch: Aggressive play
                    .sequence()
                        .condition(GoodPositionCondition)
                        .action(PlaceBetNode { bet_strategy: BetStrategy::Aggressive })
                    .selector()
                    
                    // Sub-branch: Conservative play
                    .action(PlaceBetNode { bet_strategy: BetStrategy::Conservative })
        .build()
}

// AI personality to behavior tree converter
fn personality_to_behavior_tree(personality: &PersonalityTemplate) -> BehaviorTree {
    let mut builder = BehaviorTreeBuilder::new();
    
    // Build tree based on personality traits
    if personality.aggression > 0.7 {
        builder = builder
            .selector()
            .sequence()
                .condition(OpponentWeaknessCondition)
                .action(ExploitWeaknessNode)
            .selector();
    }
    
    if personality.caution > 0.7 {
        builder = builder
            .sequence()
                .condition(RiskyScituationCondition)
                .action(PlaySafeNode)
            .selector();
    }
    
    if personality.adaptability > 0.7 {
        builder = builder
            .action(AdaptToOpponentNode)
            .selector();
    }
    
    // Default action
    builder
        .action(StandardPlayNode)
        .build()
}
```

**Behavior Tree Benefits:**

**Modular AI Logic:**
- Reusable behavior nodes
- Easy to understand flow
- Visual debugging possible
- Hot-swappable behaviors

**Complex Decision Making:**
- Sequences for ordered actions
- Selectors for alternatives
- Decorators for modifications
- Conditions for context

**Personality Integration:**
- Convert traits to behaviors
- Dynamic tree generation
- Personality-driven decisions
- Consistent character

## Section 5: AI Sharing and Community

Let's create a system for sharing custom AIs:

```rust
// AI sharing marketplace
#[derive(Resource)]
struct AIMarketplace {
    published_ais: Vec<PublishedAI>,
    categories: HashMap<String, Vec<Uuid>>,
    featured_ais: Vec<Uuid>,
    user_ratings: HashMap<Uuid, Vec<Rating>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PublishedAI {
    id: Uuid,
    name: String,
    author: String,
    description: String,
    
    // AI data
    personality: PersonalityTemplate,
    trained_model: SerializedModel,
    behavior_tree: Option<SerializedBehaviorTree>,
    
    // Metadata
    created_at: SystemTime,
    updated_at: SystemTime,
    version: u32,
    
    // Stats
    downloads: u32,
    wins: u32,
    losses: u32,
    average_rating: f32,
    
    // Tags
    tags: Vec<String>,
    difficulty: DifficultyRating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SerializedModel {
    NeuralNetwork {
        architecture: NetworkArchitecture,
        weights: Vec<f32>,
    },
    DecisionTree {
        nodes: Vec<TreeNode>,
    },
    Hybrid {
        components: Vec<SerializedModel>,
    },
}

// AI preview and testing
fn setup_ai_preview_arena(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        AIPreviewArena,
    ))
    .with_children(|parent| {
        // Left: AI selection
        spawn_ai_browser(parent);
        
        // Center: Preview arena
        spawn_preview_game(parent);
        
        // Right: AI details
        spawn_ai_details_panel(parent);
    });
}

fn spawn_ai_browser(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            width: Val::Px(300.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
    ))
    .with_children(|parent| {
        // Search bar
        spawn_ai_search_bar(parent);
        
        // Category filters
        spawn_category_filters(parent);
        
        // AI list
        spawn_ai_list(parent);
    });
}

// AI download and integration
fn download_community_ai(
    ai_id: Uuid,
    marketplace: &AIMarketplace,
    ai_library: &mut AILibrary,
) -> Result<(), DownloadError> {
    // Find AI in marketplace
    let published_ai = marketplace.published_ais.iter()
        .find(|ai| ai.id == ai_id)
        .ok_or(DownloadError::NotFound)?;
    
    // Deserialize model
    let model = deserialize_model(&published_ai.trained_model)?;
    
    // Create local AI instance
    let local_ai = CustomAI {
        id: Uuid::new_v4(),
        name: published_ai.name.clone(),
        source: AISource::Community {
            original_id: published_ai.id,
            author: published_ai.author.clone(),
        },
        personality: published_ai.personality.clone(),
        model,
        behavior_tree: published_ai.behavior_tree.as_ref()
            .and_then(|bt| deserialize_behavior_tree(bt).ok()),
    };
    
    // Add to library
    ai_library.add_ai(local_ai);
    
    // Update download count
    // (In real implementation, this would be a server call)
    
    Ok(())
}

// AI performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AIPerformanceTracker {
    ai_id: Uuid,
    matches_played: u32,
    
    // Win rates
    overall_win_rate: f32,
    win_rate_vs_human: f32,
    win_rate_vs_ai: HashMap<String, f32>,  // Per personality type
    
    // Detailed stats
    average_game_length: Duration,
    average_chips_won: f32,
    favorite_strategies: Vec<(String, f32)>,  // Strategy name, frequency
    
    // Learning progress
    performance_over_time: Vec<(SystemTime, f32)>,
    skill_rating: f32,
    improvement_rate: f32,
}

// AI tournament system
fn run_ai_tournament(
    participants: Vec<CustomAI>,
    tournament_rules: TournamentRules,
) -> TournamentResults {
    let mut bracket = create_tournament_bracket(participants);
    let mut results = TournamentResults::new();
    
    // Run rounds
    while bracket.has_remaining_matches() {
        let match_ups = bracket.get_current_round_matches();
        
        for match_up in match_ups {
            let result = simulate_ai_match(
                &match_up.ai1,
                &match_up.ai2,
                &tournament_rules
            );
            
            bracket.record_result(match_up.id, result);
            results.add_match_result(result);
        }
        
        bracket.advance_round();
    }
    
    results.champion = bracket.get_champion();
    results
}

// Export trained AI
fn export_trained_ai(
    ai: &CustomAI,
    export_config: &ExportConfig,
) -> Result<ExportedAI, ExportError> {
    let exported = ExportedAI {
        format_version: 1,
        ai_data: AIExportData {
            personality: ai.personality.clone(),
            model: serialize_model(&ai.model)?,
            behavior_tree: ai.behavior_tree.as_ref()
                .map(|bt| serialize_behavior_tree(bt))
                .transpose()?,
            training_stats: ai.training_stats.clone(),
        },
        metadata: ExportMetadata {
            name: ai.name.clone(),
            author: export_config.author.clone(),
            description: export_config.description.clone(),
            export_date: SystemTime::now(),
            compatible_versions: vec!["1.0.0".to_string()],
        },
    };
    
    // Serialize to chosen format
    match export_config.format {
        ExportFormat::Binary => {
            bincode::serialize(&exported).map_err(|e| ExportError::Serialization(e))
        },
        ExportFormat::JSON => {
            serde_json::to_string_pretty(&exported).map_err(|e| ExportError::Serialization(e))
        },
    }
}
```

**AI Sharing Features:**

**Marketplace System:**
- Browse community AIs
- Categories and tags
- Ratings and reviews
- Download tracking

**Preview System:**
- Try before download
- Watch AI play
- Compare behaviors
- Performance stats

**Export/Import:**
- Multiple formats
- Version compatibility
- Metadata preservation
- Easy sharing

## Testing Your AI Training System

At this point, you should be able to:

1. **Design Personalities**: Use sliders to create unique AI traits
2. **Train from Replays**: Clone successful player behaviors
3. **Run Training Sessions**: Progressive difficulty scenarios
4. **Evolve AI**: Genetic algorithms for improvement
5. **Share Creations**: Upload and download community AIs

## Key Concepts Mastered

1. **Machine Learning Integration**: Multiple learning approaches
   - Behavior cloning from examples
   - Neural network training
   - Genetic evolution
   - Reinforcement learning

2. **Personality Systems**: Trait-based behavior control
   - Multi-dimensional personalities
   - Predictable behaviors from traits
   - Special abilities and weaknesses

3. **Behavior Trees**: Modular AI logic
   - Composite nodes for complex logic
   - Decorators for behavior modification
   - Visual debugging capabilities

4. **Training Environments**: Controlled learning scenarios
   - Progressive difficulty
   - Specific situation training
   - Performance measurement

5. **Community Integration**: Sharing and collaboration
   - AI marketplace
   - Performance tracking
   - Tournament systems

## Exercises

1. **Add Visual Behavior Editor**: Drag-and-drop behavior tree creation

2. **Implement AI Mutations**: Random personality variations for diversity

3. **Create Challenge Mode**: Specific AI opponents with unique strategies

4. **Add Training Replays**: Record and analyze training sessions

5. **Build AI Leagues**: Ranked competitions for custom AIs

## Next Steps

In Part 15, we'll add:
- Network multiplayer support
- Real-time PvP tournaments
- Lobby and matchmaking systems
- Anti-cheat measures

The AI training system creates endless variety and replayability through custom opponents!