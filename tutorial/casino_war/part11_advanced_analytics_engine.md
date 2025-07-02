# Casino War Tutorial - Part 11: Advanced Analytics Engine - The Data-Driven Edge

## What We're Building in Part 11

Transforming our tournament into a professional-grade analytics platform:

1. **Multi-dimensional Performance Analysis**: Deep insights into every aspect of play
2. **Machine Learning Integration**: Pattern recognition and anomaly detection
3. **Predictive Modeling**: Real-time tournament outcome predictions
4. **Strategy Optimization**: AI-powered recommendations for improvement
5. **Professional Visualization**: Casino-quality analytics dashboards

## Understanding Game Analytics Systems

### The Analytics Engine Problem

Imagine you're building the analytics system for a professional poker tournament. We need an engine that:
- Captures every micro-decision and its context
- Identifies patterns humans might miss
- Predicts future outcomes based on current trends
- Provides actionable insights for improvement
- Scales to handle millions of data points efficiently

Let's think about this like building a Formula 1 telemetry system:
1. **Data Collection**: Sensors capturing every aspect of performance
2. **Real-time Processing**: Instant analysis of incoming data streams
3. **Pattern Recognition**: Machine learning to identify winning strategies
4. **Predictive Analytics**: Forecasting race outcomes mid-competition
5. **Optimization Feedback**: Suggestions for improved performance

In programming terms, this is a **real-time data pipeline** combined with **statistical analysis** and **machine learning inference**. We need to efficiently process complex data streams while maintaining game performance.

## Section 1: Analytics Data Architecture

First, let's design our comprehensive data capture system. Think of this like creating a black box recorder for our tournament:

```rust
// Core analytics event - captures EVERYTHING
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalyticsEvent {
    // Temporal data
    timestamp: f64,
    game_time: f32,
    time_remaining: f32,
    time_phase: TimePhase,
    
    // Event identification
    event_id: Uuid,
    event_type: EventType,
    player_entity: Entity,
    player_name: String,
    
    // Game state context
    player_chips: u32,
    player_position: usize,
    total_players_remaining: usize,
    average_chip_count: f32,
    
    // Decision data
    decision: Decision,
    decision_context: DecisionContext,
    decision_time: f32,  // How long they took to decide
    
    // Outcome data
    outcome: Option<Outcome>,
    chips_gained_lost: i32,
    
    // Environmental factors
    pressure_level: f32,
    recent_performance: f32,
    psychological_state: PsychologicalState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum EventType {
    BettingDecision,
    WarDecision,
    CardDealt,
    HandComplete,
    PositionChange,
    EliminationRisk,
    StrategyChange,
    PressureResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Decision {
    action_taken: ActionType,
    alternatives_considered: Vec<ActionType>,
    confidence_score: f32,
    was_optimal: Option<bool>,  // Determined post-hoc
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ActionType {
    Bet(u32),
    GoToWar,
    Surrender,
    AllIn,
    Conservative,
    Aggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PsychologicalState {
    stress_level: f32,
    confidence: f32,
    risk_tolerance: f32,
    fatigue: f32,
    tilt_factor: f32,  // Emotional decision-making
}
```

**Event Capture Philosophy:**
- **Everything is data**: Every decision, context, and outcome
- **Temporal alignment**: Multiple timestamp formats for different analyses
- **Contextual richness**: Not just what happened, but why and when
- **Post-hoc analysis**: Can determine if decisions were optimal later

**Why UUID for event_id?**
```rust
event_id: Uuid
```
- Globally unique identifiers prevent collisions
- Enable distributed analytics processing
- Allow event correlation across systems
- Support for event replay and debugging

```rust
// Analytics aggregation structures
#[derive(Debug, Clone)]
struct PlayerAnalytics {
    // Identity
    player_entity: Entity,
    player_name: String,
    is_ai: bool,
    ai_personality: Option<AiPersonality>,
    
    // Performance metrics
    total_hands: u32,
    winning_hands: u32,
    total_wars: u32,
    wars_won: u32,
    
    // Advanced metrics
    expected_value: f32,        // EV of decisions
    decision_quality: f32,      // How optimal were decisions
    adaptability_score: f32,    // How well they adjust
    pressure_performance: f32,  // Performance under stress
    consistency_rating: f32,    // Variance in performance
    
    // Time-series data
    performance_timeline: Vec<PerformancePoint>,
    decision_timeline: Vec<DecisionPoint>,
    chip_timeline: Vec<ChipPoint>,
    
    // Pattern recognition results
    identified_patterns: Vec<BehaviorPattern>,
    strategy_profile: StrategyProfile,
    weakness_analysis: Vec<Weakness>,
}

#[derive(Debug, Clone)]
struct PerformancePoint {
    timestamp: f64,
    win_rate: f32,
    chip_efficiency: f32,
    decision_speed: f32,
    stress_level: f32,
}

#[derive(Debug, Clone)]
struct BehaviorPattern {
    pattern_type: PatternType,
    confidence: f32,
    occurrences: Vec<f64>,  // Timestamps
    impact_on_performance: f32,
}

#[derive(Debug, Clone)]
enum PatternType {
    TiltAfterLoss,          // Makes bad decisions after losing
    PressureChoke,          // Performance drops under pressure
    MomentumRider,          // Plays better when winning
    TimeManagement,         // Good/bad at managing tournament time
    AdaptiveStrategy,       // Changes strategy based on context
    Predictable,            // Makes same decisions repeatedly
    Exploitable,            // Has identifiable weaknesses
}

// Machine learning features
#[derive(Debug, Clone)]
struct MLFeatureVector {
    // Basic features (normalized 0-1)
    chip_position: f32,
    time_pressure: f32,
    recent_win_rate: f32,
    position_in_tournament: f32,
    
    // Behavioral features
    aggression_level: f32,
    consistency_score: f32,
    adaptability_metric: f32,
    
    // Contextual features
    opponents_remaining: f32,
    average_opponent_aggression: f32,
    tournament_phase: f32,
    
    // Historical features
    lifetime_roi: f32,
    variance_coefficient: f32,
    learning_rate: f32,
}
```

**Time-Series Analysis:**
- Performance tracked at regular intervals
- Enables trend detection and forecasting
- Identifies hot/cold streaks mathematically
- Supports advanced visualizations

**Pattern Recognition System:**
- **TiltAfterLoss**: Emotional responses to bad beats
- **PressureChoke**: Degraded performance under stress
- **MomentumRider**: Confidence-based performance swings
- Machine learning identifies these automatically

**Feature Engineering:**
All ML features normalized to [0,1] range for:
- Consistent model training
- Easy interpretation
- Stable gradient descent
- Cross-feature comparison

## Section 2: Real-time Analytics Processing

Now let's implement the real-time analytics engine that processes this data stream:

```rust
// Analytics pipeline
#[derive(Resource)]
struct AnalyticsPipeline {
    event_buffer: RingBuffer<AnalyticsEvent>,
    processing_queue: VecDeque<AnalyticsEvent>,
    
    // Real-time processors
    pattern_detector: PatternDetector,
    anomaly_detector: AnomalyDetector,
    prediction_engine: PredictionEngine,
    
    // Aggregated data
    player_analytics: HashMap<Entity, PlayerAnalytics>,
    tournament_analytics: TournamentAnalytics,
    
    // Performance metrics
    events_per_second: f32,
    processing_latency: Duration,
}

impl AnalyticsPipeline {
    fn process_event(&mut self, event: AnalyticsEvent) {
        // Add to ring buffer for historical access
        self.event_buffer.push(event.clone());
        
        // Queue for processing
        self.processing_queue.push_back(event);
        
        // Process queue in batches for efficiency
        if self.processing_queue.len() >= 10 {
            self.process_batch();
        }
    }
    
    fn process_batch(&mut self) {
        let batch_start = Instant::now();
        
        while let Some(event) = self.processing_queue.pop_front() {
            // Update player analytics
            if let Some(player) = self.player_analytics.get_mut(&event.player_entity) {
                player.update_with_event(&event);
            }
            
            // Run pattern detection
            if let Some(patterns) = self.pattern_detector.analyze(&event, &self.event_buffer) {
                self.handle_detected_patterns(event.player_entity, patterns);
            }
            
            // Check for anomalies
            if let Some(anomaly) = self.anomaly_detector.check(&event, &self.player_analytics) {
                self.handle_anomaly(anomaly);
            }
            
            // Update predictions
            self.prediction_engine.update(&event, &self.player_analytics);
        }
        
        self.processing_latency = batch_start.elapsed();
    }
}

// Pattern detection using sliding window analysis
struct PatternDetector {
    window_size: usize,
    pattern_matchers: Vec<Box<dyn PatternMatcher>>,
}

trait PatternMatcher: Send + Sync {
    fn matches(&self, events: &[AnalyticsEvent]) -> Option<BehaviorPattern>;
}

// Example: Tilt detection pattern matcher
struct TiltDetector;

impl PatternMatcher for TiltDetector {
    fn matches(&self, events: &[AnalyticsEvent]) -> Option<BehaviorPattern> {
        // Look for emotional decision-making after losses
        let mut loss_indices = Vec::new();
        let mut poor_decisions_after_loss = 0;
        
        for (i, event) in events.iter().enumerate() {
            if let Some(outcome) = &event.outcome {
                if !outcome.won && event.chips_gained_lost < -50 {
                    loss_indices.push(i);
                }
            }
        }
        
        // Check decisions made within 3 hands of losses
        for loss_idx in &loss_indices {
            for check_idx in (*loss_idx + 1)..(*loss_idx + 4).min(events.len()) {
                if let Some(event) = events.get(check_idx) {
                    if event.decision.confidence_score < 0.3 ||
                       event.decision_time < 0.5 {  // Too fast = emotional
                        poor_decisions_after_loss += 1;
                    }
                }
            }
        }
        
        if poor_decisions_after_loss >= 3 {
            Some(BehaviorPattern {
                pattern_type: PatternType::TiltAfterLoss,
                confidence: (poor_decisions_after_loss as f32 / loss_indices.len() as f32).min(1.0),
                occurrences: loss_indices.iter()
                    .map(|&i| events[i].timestamp)
                    .collect(),
                impact_on_performance: -0.3,  // Typically costs 30% performance
            })
        } else {
            None
        }
    }
}

// Anomaly detection using statistical methods
struct AnomalyDetector {
    z_score_threshold: f32,  // Standard deviations from mean
    min_sample_size: usize,
}

impl AnomalyDetector {
    fn check(&self, event: &AnalyticsEvent, player_data: &HashMap<Entity, PlayerAnalytics>) -> Option<Anomaly> {
        let player = player_data.get(&event.player_entity)?;
        
        // Check if decision time is anomalous
        let decision_times: Vec<f32> = player.decision_timeline.iter()
            .map(|d| d.decision_time)
            .collect();
            
        if decision_times.len() < self.min_sample_size {
            return None;
        }
        
        let mean = decision_times.iter().sum::<f32>() / decision_times.len() as f32;
        let variance = decision_times.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / decision_times.len() as f32;
        let std_dev = variance.sqrt();
        
        let z_score = (event.decision_time - mean) / std_dev;
        
        if z_score.abs() > self.z_score_threshold {
            Some(Anomaly {
                event_id: event.event_id,
                anomaly_type: AnomalyType::DecisionTime,
                severity: z_score.abs() / self.z_score_threshold,
                context: format!("Decision time {:.2}s is {:.1} std devs from mean {:.2}s", 
                               event.decision_time, z_score, mean),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct Anomaly {
    event_id: Uuid,
    anomaly_type: AnomalyType,
    severity: f32,
    context: String,
}

#[derive(Debug, Clone)]
enum AnomalyType {
    DecisionTime,      // Unusually fast/slow decisions
    BettingPattern,    // Unusual bet sizing
    StrategyShift,     // Sudden strategy change
    PerformanceSpike,  // Unusual win/loss streak
}
```

**Ring Buffer for Event History:**
```rust
event_buffer: RingBuffer<AnalyticsEvent>
```
- Fixed-size circular buffer for memory efficiency
- Maintains recent history for pattern analysis
- Old events automatically overwritten
- O(1) insertion, efficient for real-time systems

**Batch Processing Strategy:**
- Events queued until batch size reached
- Reduces processing overhead
- Enables vectorized operations
- Maintains low latency (process every 10 events)

**Pattern Detection Architecture:**
- Trait-based system for extensible patterns
- Each pattern matcher is independent
- Sliding window analysis for temporal patterns
- Confidence scores for pattern strength

**Statistical Anomaly Detection:**
```rust
let z_score = (event.decision_time - mean) / std_dev;
```
- Z-score measures how many standard deviations from mean
- Threshold typically 2-3 (95-99% confidence)
- Identifies statistically significant outliers
- Works for any normally distributed metric

## Section 3: Machine Learning Integration

Let's add machine learning for advanced pattern recognition and prediction:

```rust
// Neural network for outcome prediction
struct PredictionEngine {
    network: NeuralNetwork,
    feature_normalizer: FeatureNormalizer,
    prediction_buffer: RingBuffer<Prediction>,
    accuracy_tracker: AccuracyTracker,
}

#[derive(Debug, Clone)]
struct NeuralNetwork {
    layers: Vec<Layer>,
    activation: ActivationFunction,
}

#[derive(Debug, Clone)]
struct Layer {
    weights: Array2<f32>,  // Using ndarray
    biases: Array1<f32>,
}

#[derive(Debug, Clone, Copy)]
enum ActivationFunction {
    ReLU,
    Sigmoid,
    Tanh,
}

impl PredictionEngine {
    fn predict_tournament_outcome(&mut self, analytics: &HashMap<Entity, PlayerAnalytics>) -> TournamentPrediction {
        let mut predictions = Vec::new();
        
        // Generate features for each player
        for (entity, player_data) in analytics {
            let features = self.extract_features(player_data);
            let normalized = self.feature_normalizer.normalize(&features);
            
            // Forward pass through network
            let win_probability = self.network.forward(&normalized);
            
            predictions.push(PlayerPrediction {
                entity: *entity,
                name: player_data.player_name.clone(),
                win_probability,
                expected_finish: self.calculate_expected_position(win_probability, analytics.len()),
                confidence: self.calculate_prediction_confidence(player_data),
            });
        }
        
        // Sort by win probability
        predictions.sort_by(|a, b| b.win_probability.partial_cmp(&a.win_probability).unwrap());
        
        TournamentPrediction {
            predictions,
            entropy: self.calculate_outcome_entropy(&predictions),
            most_likely_winner: predictions.first().cloned(),
            upset_potential: self.calculate_upset_potential(&predictions),
        }
    }
    
    fn extract_features(&self, player: &PlayerAnalytics) -> MLFeatureVector {
        // Extract current performance metrics
        let recent_wins = player.performance_timeline.iter()
            .rev()
            .take(10)
            .filter(|p| p.win_rate > 0.5)
            .count() as f32 / 10.0;
            
        let momentum = player.performance_timeline.windows(2)
            .map(|w| w[1].chip_efficiency - w[0].chip_efficiency)
            .sum::<f32>() / player.performance_timeline.len() as f32;
            
        let pressure_handling = player.performance_timeline.iter()
            .filter(|p| p.stress_level > 0.7)
            .map(|p| p.win_rate)
            .sum::<f32>() / player.performance_timeline.len() as f32;
        
        MLFeatureVector {
            chip_position: player.total_hands as f32 / 100.0,  // Normalized
            time_pressure: 0.5,  // Would come from tournament timer
            recent_win_rate: recent_wins,
            position_in_tournament: 0.5,  // Would come from leaderboard
            aggression_level: player.decision_quality,
            consistency_score: player.consistency_rating,
            adaptability_metric: player.adaptability_score,
            opponents_remaining: 0.8,  // Would be calculated
            average_opponent_aggression: 0.5,
            tournament_phase: 0.5,
            lifetime_roi: player.expected_value / 100.0,
            variance_coefficient: 1.0 - player.consistency_rating,
            learning_rate: momentum.abs(),
        }
    }
    
    fn calculate_outcome_entropy(&self, predictions: &[PlayerPrediction]) -> f32 {
        // Shannon entropy - measures uncertainty in predictions
        predictions.iter()
            .map(|p| {
                if p.win_probability > 0.0 {
                    -p.win_probability * p.win_probability.log2()
                } else {
                    0.0
                }
            })
            .sum()
    }
}

impl NeuralNetwork {
    fn forward(&self, input: &Array1<f32>) -> f32 {
        let mut activation = input.clone();
        
        for layer in &self.layers {
            // Linear transformation: y = Wx + b
            activation = layer.weights.dot(&activation) + &layer.biases;
            
            // Apply activation function
            activation = match self.activation {
                ActivationFunction::ReLU => activation.map(|x| x.max(0.0)),
                ActivationFunction::Sigmoid => activation.map(|x| 1.0 / (1.0 + (-x).exp())),
                ActivationFunction::Tanh => activation.map(|x| x.tanh()),
            };
        }
        
        // Output layer - single probability value
        activation[0].min(1.0).max(0.0)
    }
}

// Feature normalization for stable training
struct FeatureNormalizer {
    means: HashMap<String, f32>,
    std_devs: HashMap<String, f32>,
}

impl FeatureNormalizer {
    fn normalize(&self, features: &MLFeatureVector) -> Array1<f32> {
        // Convert struct to array and normalize
        array![
            features.chip_position,
            features.time_pressure,
            features.recent_win_rate,
            features.position_in_tournament,
            features.aggression_level,
            features.consistency_score,
            features.adaptability_metric,
            features.opponents_remaining,
            features.average_opponent_aggression,
            features.tournament_phase,
            features.lifetime_roi,
            features.variance_coefficient,
            features.learning_rate,
        ]
    }
}

// Prediction tracking for model improvement
#[derive(Debug, Clone)]
struct Prediction {
    timestamp: f64,
    predicted_winner: Entity,
    actual_winner: Option<Entity>,
    confidence: f32,
    feature_importance: Vec<(String, f32)>,
}

struct AccuracyTracker {
    predictions: Vec<Prediction>,
    
    // Metrics
    accuracy: f32,
    precision: f32,
    recall: f32,
    f1_score: f32,
}

impl AccuracyTracker {
    fn update_with_outcome(&mut self, prediction: &mut Prediction, actual_winner: Entity) {
        prediction.actual_winner = Some(actual_winner);
        self.predictions.push(prediction.clone());
        self.recalculate_metrics();
    }
    
    fn recalculate_metrics(&mut self) {
        let completed_predictions: Vec<_> = self.predictions.iter()
            .filter(|p| p.actual_winner.is_some())
            .collect();
            
        if completed_predictions.is_empty() {
            return;
        }
        
        let correct_predictions = completed_predictions.iter()
            .filter(|p| p.predicted_winner == p.actual_winner.unwrap())
            .count();
            
        self.accuracy = correct_predictions as f32 / completed_predictions.len() as f32;
        
        // Calculate precision, recall, F1 for each player
        // ... (detailed implementation)
    }
}
```

**Neural Network Architecture:**
- **Input Layer**: 13 normalized features
- **Hidden Layers**: Configurable depth and width
- **Output Layer**: Single probability value (0-1)
- **Activation Functions**: ReLU for hidden layers, Sigmoid for output

**Feature Engineering Strategy:**
- All features normalized to [0,1] range
- Temporal features capture trends
- Behavioral features capture personality
- Contextual features capture game state

**Entropy as Uncertainty Measure:**
```rust
-p.win_probability * p.win_probability.log2()
```
- Shannon entropy quantifies prediction uncertainty
- High entropy = competitive tournament
- Low entropy = clear favorite
- Used to identify exciting matches

**Model Performance Tracking:**
- Every prediction stored with outcome
- Accuracy metrics calculated in real-time
- Feature importance tracked for interpretability
- Enables continuous model improvement

## Section 4: Advanced Visualization System

Now let's create professional-grade analytics visualizations:

```rust
// Analytics dashboard components
#[derive(Component)]
struct AnalyticsDashboard;

#[derive(Component)]
struct PerformanceChart {
    chart_type: ChartType,
    data_source: DataSource,
    update_frequency: Timer,
}

#[derive(Debug, Clone)]
enum ChartType {
    LineChart,      // Performance over time
    HeatMap,        // Decision quality matrix
    RadarChart,     // Multi-dimensional player comparison
    Histogram,      // Distribution analysis
    ScatterPlot,    // Correlation analysis
    BoxPlot,        // Statistical distributions
}

#[derive(Debug, Clone)]
enum DataSource {
    PlayerPerformance(Entity),
    TournamentAggregate,
    ComparativeAnalysis(Vec<Entity>),
    PredictionAccuracy,
}

// Real-time chart rendering
fn render_performance_chart(
    mut commands: Commands,
    analytics: Res<AnalyticsPipeline>,
    chart_query: Query<(&PerformanceChart, &Node)>,
) {
    for (chart, node) in &chart_query {
        match chart.chart_type {
            ChartType::LineChart => render_line_chart(&mut commands, &analytics, chart, node),
            ChartType::HeatMap => render_heat_map(&mut commands, &analytics, chart, node),
            ChartType::RadarChart => render_radar_chart(&mut commands, &analytics, chart, node),
            _ => {}
        }
    }
}

fn render_line_chart(
    commands: &mut Commands,
    analytics: &AnalyticsPipeline,
    chart: &PerformanceChart,
    node: &Node,
) {
    if let DataSource::PlayerPerformance(entity) = chart.data_source {
        if let Some(player_data) = analytics.player_analytics.get(&entity) {
            let points = &player_data.performance_timeline;
            
            // Calculate chart dimensions
            let width = 400.0;
            let height = 200.0;
            let margin = 20.0;
            
            // Create chart container
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(width),
                    height: Val::Px(height),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            ))
            .with_children(|parent| {
                // Render axes
                render_chart_axes(parent, width, height, margin);
                
                // Render data points
                render_line_data(parent, points, width, height, margin);
                
                // Render labels
                render_chart_labels(parent, "Time", "Win Rate", width, height);
            });
        }
    }
}

fn render_heat_map(
    commands: &mut Commands,
    analytics: &AnalyticsPipeline,
    chart: &PerformanceChart,
    node: &Node,
) {
    // Create decision quality heat map
    let grid_size = 10;
    let cell_size = 30.0;
    
    commands.spawn((
        Node {
            width: Val::Px(grid_size as f32 * cell_size),
            height: Val::Px(grid_size as f32 * cell_size),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::flex(grid_size as u16, 1.0),
            grid_template_rows: RepeatedGridTrack::flex(grid_size as u16, 1.0),
            ..default()
        },
    ))
    .with_children(|parent| {
        for y in 0..grid_size {
            for x in 0..grid_size {
                let pressure = x as f32 / grid_size as f32;
                let chip_position = y as f32 / grid_size as f32;
                
                // Calculate average decision quality at this pressure/position
                let quality = calculate_decision_quality_at(
                    analytics,
                    pressure,
                    chip_position
                );
                
                // Map quality to color (red = bad, green = good)
                let color = Color::srgb(
                    1.0 - quality,
                    quality,
                    0.0
                );
                
                parent.spawn((
                    Node {
                        width: Val::Px(cell_size - 2.0),
                        height: Val::Px(cell_size - 2.0),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
            }
        }
    });
}

// Strategy recommendation engine
#[derive(Debug, Clone)]
struct StrategyRecommendation {
    recommendation_type: RecommendationType,
    confidence: f32,
    expected_improvement: f32,
    explanation: String,
    specific_actions: Vec<ActionRecommendation>,
}

#[derive(Debug, Clone)]
enum RecommendationType {
    BettingSizing,       // Adjust bet sizes
    WarStrategy,         // Change war decision criteria
    TimeManagement,      // Better tournament pacing
    PressureHandling,    // Improve under pressure
    OpponentExploiting,  // Target specific weaknesses
}

#[derive(Debug, Clone)]
struct ActionRecommendation {
    situation: String,
    current_action: String,
    recommended_action: String,
    reasoning: String,
}

fn generate_strategy_recommendations(
    analytics: &AnalyticsPipeline,
    player_entity: Entity,
) -> Vec<StrategyRecommendation> {
    let mut recommendations = Vec::new();
    
    if let Some(player_data) = analytics.player_analytics.get(&player_entity) {
        // Analyze betting patterns
        if let Some(betting_rec) = analyze_betting_efficiency(player_data) {
            recommendations.push(betting_rec);
        }
        
        // Analyze war decisions
        if let Some(war_rec) = analyze_war_strategy(player_data) {
            recommendations.push(war_rec);
        }
        
        // Analyze pressure performance
        if player_data.pressure_performance < 0.4 {
            recommendations.push(StrategyRecommendation {
                recommendation_type: RecommendationType::PressureHandling,
                confidence: 0.8,
                expected_improvement: 0.2,
                explanation: "Your performance drops significantly under pressure. Practice maintaining consistent decision-making when behind.".to_string(),
                specific_actions: vec![
                    ActionRecommendation {
                        situation: "When chips < 30% of average".to_string(),
                        current_action: "Panic betting (2x normal)".to_string(),
                        recommended_action: "Maintain normal betting patterns".to_string(),
                        reasoning: "Panic betting rarely recovers losses".to_string(),
                    }
                ],
            });
        }
    }
    
    recommendations
}

// Performance grading system
#[derive(Debug, Clone)]
struct PerformanceGrade {
    overall_grade: Grade,
    category_grades: HashMap<String, Grade>,
    strengths: Vec<String>,
    weaknesses: Vec<String>,
    improvement_areas: Vec<ImprovementArea>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Grade {
    S,  // Exceptional (top 1%)
    A,  // Excellent (top 10%)
    B,  // Good (top 30%)
    C,  // Average (middle 40%)
    D,  // Below Average (bottom 20%)
    F,  // Poor (bottom 10%)
}

#[derive(Debug, Clone)]
struct ImprovementArea {
    category: String,
    current_performance: f32,
    target_performance: f32,
    suggested_exercises: Vec<String>,
}

fn calculate_performance_grade(player_data: &PlayerAnalytics) -> PerformanceGrade {
    let mut category_grades = HashMap::new();
    
    // Grade each category
    category_grades.insert("Decision Making".to_string(), 
        grade_from_score(player_data.decision_quality));
    category_grades.insert("Consistency".to_string(),
        grade_from_score(player_data.consistency_rating));
    category_grades.insert("Adaptability".to_string(),
        grade_from_score(player_data.adaptability_score));
    category_grades.insert("Pressure Handling".to_string(),
        grade_from_score(player_data.pressure_performance));
    
    // Calculate overall grade (weighted average)
    let overall_score = (
        player_data.decision_quality * 0.3 +
        player_data.consistency_rating * 0.2 +
        player_data.adaptability_score * 0.3 +
        player_data.pressure_performance * 0.2
    );
    
    // Identify strengths and weaknesses
    let mut strengths = Vec::new();
    let mut weaknesses = Vec::new();
    
    for (category, grade) in &category_grades {
        match grade {
            Grade::S | Grade::A => strengths.push(category.clone()),
            Grade::D | Grade::F => weaknesses.push(category.clone()),
            _ => {}
        }
    }
    
    PerformanceGrade {
        overall_grade: grade_from_score(overall_score),
        category_grades,
        strengths,
        weaknesses,
        improvement_areas: generate_improvement_plan(player_data),
    }
}

fn grade_from_score(score: f32) -> Grade {
    match score {
        s if s >= 0.95 => Grade::S,
        s if s >= 0.85 => Grade::A,
        s if s >= 0.70 => Grade::B,
        s if s >= 0.50 => Grade::C,
        s if s >= 0.30 => Grade::D,
        _ => Grade::F,
    }
}
```

**Chart Rendering Architecture:**
- Component-based chart system
- Real-time data binding
- Multiple chart types for different insights
- Responsive sizing and positioning

**Heat Map Visualization:**
- 2D grid showing decision quality
- X-axis: Pressure level
- Y-axis: Chip position
- Color: Performance (red=bad, green=good)
- Instantly identifies weak spots

**Strategy Recommendation Engine:**
- Analyzes player weaknesses
- Provides specific, actionable advice
- Confidence scores for each recommendation
- Expected improvement quantified

**Performance Grading System:**
- Letter grades (S-F) like academic/gaming systems
- Category breakdown for detailed analysis
- Strengths/weaknesses clearly identified
- Improvement plan with specific exercises

## Section 5: Analytics Dashboard UI

Let's create the professional analytics interface:

```rust
fn setup_analytics_dashboard(mut commands: Commands) {
    // Main dashboard container
    commands.spawn((
        AnalyticsDashboard,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::None,  // Hidden by default
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.05, 0.95)),
        ZIndex(2000),  // Above game UI
    ))
    .with_children(|parent| {
        // Dashboard header
        spawn_dashboard_header(parent);
        
        // Main content area with tabs
        parent.spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Left sidebar - Player selection
            spawn_player_selector(parent);
            
            // Center - Main analytics view
            spawn_analytics_view(parent);
            
            // Right sidebar - Recommendations
            spawn_recommendations_panel(parent);
        });
    });
}

fn spawn_analytics_view(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            margin: UiRect::horizontal(Val::Px(20.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        // Tab navigation
        spawn_tab_navigation(parent);
        
        // Tab content area
        parent.spawn((
            Node {
                flex_grow: 1.0,
                padding: UiRect::all(Val::Px(15.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.5)),
        ));
    });
}

fn spawn_tab_navigation(parent: &mut ChildBuilder) {
    let tabs = [
        ("Overview", TabType::Overview),
        ("Performance", TabType::Performance),
        ("Decisions", TabType::Decisions),
        ("Predictions", TabType::Predictions),
        ("Patterns", TabType::Patterns),
    ];
    
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            gap: Val::Px(10.0),
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        },
    ))
    .with_children(|parent| {
        for (label, tab_type) in tabs {
            parent.spawn((
                Button,
                AnalyticsTab { tab_type },
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::top(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_child((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        }
    });
}

// Real-time metric displays
fn spawn_metric_card(
    parent: &mut ChildBuilder,
    title: &str,
    value: &str,
    trend: Trend,
    subtitle: &str,
) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(15.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            min_width: Val::Px(150.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
    ))
    .with_children(|parent| {
        // Title
        parent.spawn((
            Text::new(title),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        // Value with trend
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                gap: Val::Px(5.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Main value
            parent.spawn((
                Text::new(value),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            
            // Trend indicator
            let (symbol, color) = match trend {
                Trend::Up(pct) => (format!("↑{:.1}%", pct), Color::srgb(0.0, 0.8, 0.0)),
                Trend::Down(pct) => (format!("↓{:.1}%", pct), Color::srgb(0.8, 0.0, 0.0)),
                Trend::Stable => ("→".to_string(), Color::srgb(0.5, 0.5, 0.5)),
            };
            
            parent.spawn((
                Text::new(symbol),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(color),
            ));
        });
        
        // Subtitle
        parent.spawn((
            Text::new(subtitle),
            TextFont {
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
        ));
    });
}

#[derive(Debug, Clone, Copy)]
enum Trend {
    Up(f32),    // Percentage increase
    Down(f32),  // Percentage decrease  
    Stable,
}

// Export functionality
fn export_analytics_data(
    analytics: &AnalyticsPipeline,
    export_format: ExportFormat,
) -> Result<String, ExportError> {
    match export_format {
        ExportFormat::JSON => {
            let data = AnalyticsExport {
                timestamp: SystemTime::now(),
                tournament_id: Uuid::new_v4(),
                player_analytics: analytics.player_analytics.values().cloned().collect(),
                patterns_detected: collect_all_patterns(analytics),
                predictions_made: analytics.prediction_engine.prediction_buffer.to_vec(),
            };
            
            serde_json::to_string_pretty(&data)
                .map_err(|e| ExportError::Serialization(e.to_string()))
        },
        
        ExportFormat::CSV => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            
            // Write headers
            wtr.write_record(&[
                "Player", "Hands", "Win Rate", "EV", "Consistency", 
                "Pressure Performance", "Grade"
            ])?;
            
            // Write player data
            for player in analytics.player_analytics.values() {
                wtr.write_record(&[
                    &player.player_name,
                    &player.total_hands.to_string(),
                    &format!("{:.2}%", player.winning_hands as f32 / player.total_hands as f32 * 100.0),
                    &format!("{:.2}", player.expected_value),
                    &format!("{:.2}", player.consistency_rating),
                    &format!("{:.2}", player.pressure_performance),
                    &format!("{:?}", calculate_performance_grade(player).overall_grade),
                ])?;
            }
            
            String::from_utf8(wtr.into_inner()?)
                .map_err(|e| ExportError::Encoding(e.to_string()))
        },
    }
}

#[derive(Debug, Clone)]
enum ExportFormat {
    JSON,
    CSV,
}

#[derive(Debug, Serialize)]
struct AnalyticsExport {
    timestamp: SystemTime,
    tournament_id: Uuid,
    player_analytics: Vec<PlayerAnalytics>,
    patterns_detected: Vec<PatternSummary>,
    predictions_made: Vec<Prediction>,
}
```

**Dashboard Architecture:**
- **Hidden by default**: Toggled on/off during gameplay
- **Tab-based navigation**: Different views of same data
- **Responsive layout**: Adapts to screen size
- **Real-time updates**: Data refreshes automatically

**Metric Cards Design:**
- Compact, information-dense displays
- Trend indicators show change over time
- Color coding for quick scanning
- Consistent visual language

**Export Functionality:**
- **JSON**: Complete data export for analysis
- **CSV**: Simplified format for spreadsheets
- Preserves all important metrics
- Enables external analysis tools

## Testing Your Analytics System

At this point, you should be able to:

1. **View Real-time Analytics**: See comprehensive metrics during gameplay
2. **Detect Patterns**: Automatic identification of player behaviors
3. **Get Predictions**: ML-powered tournament outcome forecasts
4. **Receive Recommendations**: Personalized strategy improvements
5. **Export Data**: Save tournament data for external analysis

## Key Concepts Mastered

1. **Data Pipeline Architecture**: Efficient real-time event processing
   - Ring buffers for memory-efficient history
   - Batch processing for performance
   - Asynchronous analytics calculations

2. **Statistical Analysis**: Advanced metrics and pattern detection
   - Z-score anomaly detection
   - Sliding window pattern matching
   - Multi-dimensional performance tracking

3. **Machine Learning Integration**: Neural networks for game predictions
   - Feature engineering from game data
   - Real-time inference during tournaments
   - Model accuracy tracking

4. **Professional Visualization**: Casino-quality analytics displays
   - Multiple chart types for different insights
   - Real-time updating with smooth animations
   - Interactive dashboard with drill-down capability

5. **Actionable Intelligence**: Converting data into improvements
   - Specific strategy recommendations
   - Performance grading system
   - Weakness identification and remediation

## Exercises

1. **Add Clustering Analysis**: Group players by play style using k-means clustering

2. **Implement A/B Testing**: Test different AI strategies and measure effectiveness

3. **Create Custom Alerts**: Notify when specific patterns or anomalies detected

4. **Add Replay Analysis**: Step through tournaments with analytics overlay

5. **Build Training Mode**: Use analytics to create targeted practice scenarios

## Next Steps

In Part 12, we'll add:
- Tournament seasons with long-term progression
- Player profiles and statistics persistence
- Unlockable content and achievements
- Ranking systems and leagues

The analytics engine provides the foundation for meaningful long-term progression!