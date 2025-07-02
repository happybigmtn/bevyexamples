# Casino War Tutorial - Part 9: Timed Tournament Mode - Winner Takes All

## What We're Building in Part 9

Transforming our tournament into a high-pressure, time-driven competitive arena:

1. **2-Minute Blitz Mode**: High-intensity tournament with visible countdown timer
2. **Time-Pressure Psychology**: AI strategies adapt based on remaining time
3. **Urgency Systems**: Visual and behavioral changes as time runs out
4. **Prize Pool Distribution**: Winner-take-all with dramatic final moments
5. **Overtime Mechanics**: Final hand completion when time expires

## Understanding Time-Pressure Game Design

### The Timed Competition Problem

Imagine you're designing a TV game show where contestants compete under time pressure. We need a tournament system that:
- Creates mounting tension as time runs down
- Changes player behavior based on time remaining
- Provides clear visual feedback about urgency
- Ensures fair completion even when time expires mid-hand
- Maximizes drama and excitement in the final moments

Let's think about this like building a pressure cooker:
1. **Time Visibility**: Everyone knows exactly how much time remains
2. **Escalating Pressure**: Visual and audio cues increase urgency
3. **Behavioral Changes**: Players make riskier decisions as time runs out
4. **Fair Completion**: Current hand finishes even after time expires
5. **Dramatic Climax**: Winner determined by sudden-death rules

In programming terms, this is a **time-constrained state machine** combined with **adaptive AI behavior**. We need to manage complex interactions between time pressure, player psychology, and game fairness.

## Section 1: Tournament Timer Architecture

First, let's design our time-pressure system. Think of this like creating a mission timer for a heist game:

```rust
// Tournament timer and pressure system
#[derive(Resource)]
struct TournamentTimer {
    // Core timing
    duration: Timer,
    total_duration: f32,
    is_active: bool,
    is_overtime: bool,
    
    // Pressure indicators
    pressure_level: f32,        // 0.0 to 1.0, based on time remaining
    warning_threshold: f32,     // When to start warning indicators (30 seconds)
    critical_threshold: f32,    // When to go into critical mode (10 seconds)
    
    // Visual effects
    pulse_timer: Timer,         // For flashing effects
    shake_intensity: f32,       // Screen shake based on urgency
    color_urgency: f32,         // Red tint intensity
    
    // Audio cues
    last_beep_time: f32,       // Track beeping frequency
    beep_interval: f32,        // How often to beep (decreases with time)
    
    // Overtime handling
    overtime_hands_remaining: u32,  // Hands left to complete
    final_hand_entities: Vec<Entity>, // Cards that must finish
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TimePhase {
    Opening,      // First 90 seconds - normal play
    MidGame,      // 60-90 seconds - slight pressure
    Warning,      // 30-60 seconds - moderate pressure
    Critical,     // 10-30 seconds - high pressure
    Final,        // Last 10 seconds - maximum pressure
    Overtime,     // Time expired, finishing current hands
}

// AI pressure response tracking
#[derive(Component)]
struct PressureResponse {
    base_aggression: f32,       // Original risk tolerance
    pressure_multiplier: f32,   // How much pressure affects this AI
    panic_threshold: f32,       // When this AI "panics" and plays badly
    time_awareness: f32,        // How well this AI manages time pressure
    
    // Behavioral changes under pressure
    bet_size_modifier: f32,     // Multiplier for bet sizes
    war_threshold_modifier: f32, // How pressure affects war decisions
    decision_speed_modifier: f32, // How quickly they make decisions
}
```

**Tournament Timer Design:**

**Core Timing System:**
- `duration`: Main countdown timer (2 minutes = 120 seconds)
- `total_duration`: Original time for percentage calculations
- `is_overtime`: Special mode when time expires mid-hand

**Pressure Level Calculation:**
```rust
pressure_level = 1.0 - (time_remaining / total_duration)
```
- 0.0 at start (no pressure)
- 1.0 when time expires (maximum pressure)
- Used to modify AI behavior and visual effects

**Dynamic Thresholds:**
- **Warning**: 30 seconds remaining (25% time left)
- **Critical**: 10 seconds remaining (8% time left)
- Each phase triggers different visual/audio cues

**Visual Effect Parameters:**
- `pulse_timer`: Controls flashing UI elements
- `shake_intensity`: Screen shake magnitude
- `color_urgency`: Red overlay intensity
- All scale with `pressure_level`

```rust
impl TournamentTimer {
    fn new(duration_seconds: f32) -> Self {
        Self {
            duration: Timer::from_seconds(duration_seconds, TimerMode::Once),
            total_duration: duration_seconds,
            is_active: false,
            is_overtime: false,
            pressure_level: 0.0,
            warning_threshold: 30.0,
            critical_threshold: 10.0,
            pulse_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            shake_intensity: 0.0,
            color_urgency: 0.0,
            last_beep_time: 0.0,
            beep_interval: 10.0,  // Start with beeps every 10 seconds
            overtime_hands_remaining: 0,
            final_hand_entities: Vec::new(),
        }
    }

    fn start(&mut self) {
        self.is_active = true;
        self.duration.reset();
        info!("Tournament timer started: {} seconds", self.total_duration);
    }

    fn update(&mut self, delta_time: f32) -> TimePhase {
        if !self.is_active && !self.is_overtime {
            return TimePhase::Opening;
        }

        if self.is_overtime {
            return TimePhase::Overtime;
        }

        // Update main timer
        self.duration.tick(Duration::from_secs_f32(delta_time));
        self.pulse_timer.tick(Duration::from_secs_f32(delta_time));

        // Calculate pressure level
        let time_remaining = self.duration.remaining_secs();
        self.pressure_level = 1.0 - (time_remaining / self.total_duration);

        // Update visual effects based on pressure
        self.update_visual_effects();

        // Update audio cues
        self.update_audio_cues(time_remaining);

        // Determine current phase
        if time_remaining <= 0.0 {
            self.enter_overtime();
            TimePhase::Overtime
        } else if time_remaining <= self.critical_threshold {
            TimePhase::Critical
        } else if time_remaining <= self.warning_threshold {
            TimePhase::Warning
        } else if time_remaining <= 60.0 {
            TimePhase::MidGame
        } else {
            TimePhase::Opening
        }
    }

    fn update_visual_effects(&mut self) {
        // Exponential increase in urgency effects
        let urgency = self.pressure_level.powi(2);  // Square for more dramatic curve
        
        self.shake_intensity = urgency * 5.0;  // Max 5 pixel shake
        self.color_urgency = urgency * 0.3;    // Max 30% red tint
        
        // Pulse speed increases with pressure
        let pulse_speed = 1.0 + urgency * 3.0;  // 1x to 4x speed
        self.pulse_timer = Timer::from_seconds(1.0 / pulse_speed, TimerMode::Repeating);
    }

    fn update_audio_cues(&mut self, time_remaining: f32) {
        // Beep frequency increases as time runs out
        self.beep_interval = if time_remaining <= 10.0 {
            0.5  // Every 0.5 seconds in final 10
        } else if time_remaining <= 30.0 {
            2.0  // Every 2 seconds in warning phase
        } else {
            10.0 // Every 10 seconds normally
        };
    }

    fn enter_overtime(&mut self) {
        self.is_active = false;
        self.is_overtime = true;
        self.overtime_hands_remaining = 1;  // Complete current hand only
        info!("Tournament entered overtime!");
    }

    fn get_time_remaining_display(&self) -> String {
        if self.is_overtime {
            "OVERTIME".to_string()
        } else {
            let remaining = self.duration.remaining_secs();
            format!("{:02}:{:02}", 
                remaining as u32 / 60, 
                remaining as u32 % 60
            )
        }
    }
}
```

**Visual Effects Calculation:**
```rust
let urgency = self.pressure_level.powi(2);
```
- Squaring the pressure level creates exponential urgency
- Early tournament: minimal visual effects
- Final moments: dramatic visual intensity

**Dynamic Audio Timing:**
- **Normal**: Beep every 10 seconds
- **Warning**: Beep every 2 seconds  
- **Critical**: Beep every 0.5 seconds
- Creates escalating audio pressure

**Overtime Logic:**
- Timer stops but tournament continues
- Only current hands complete
- No new hands can start
- Ensures fair conclusion

## Section 2: Time-Pressure AI Psychology

Now let's implement how AI players respond to time pressure. This simulates human psychology under stress:

```rust
impl PressureResponse {
    fn new(personality: AiPersonality) -> Self {
        match personality {
            AiPersonality::Conservative => Self {
                base_aggression: 0.2,
                pressure_multiplier: 0.8,      // Less affected by pressure
                panic_threshold: 0.9,          // Panics only at very end
                time_awareness: 0.9,           // Very aware of time
                bet_size_modifier: 1.0,
                war_threshold_modifier: 1.0,
                decision_speed_modifier: 1.0,
            },
            
            AiPersonality::Aggressive => Self {
                base_aggression: 0.8,
                pressure_multiplier: 1.5,      // Highly affected by pressure
                panic_threshold: 0.7,          // Panics earlier
                time_awareness: 0.6,           // Moderate time awareness
                bet_size_modifier: 1.0,
                war_threshold_modifier: 1.0,
                decision_speed_modifier: 1.0,
            },
            
            AiPersonality::Balanced => Self {
                base_aggression: 0.5,
                pressure_multiplier: 1.0,      // Normal pressure response
                panic_threshold: 0.8,          // Balanced panic threshold
                time_awareness: 0.8,           // Good time awareness
                bet_size_modifier: 1.0,
                war_threshold_modifier: 1.0,
                decision_speed_modifier: 1.0,
            },
            
            AiPersonality::Adaptive => Self {
                base_aggression: 0.5,
                pressure_multiplier: 0.7,      // Adapts well to pressure
                panic_threshold: 0.85,         // Resists panic well
                time_awareness: 1.0,           // Excellent time awareness
                bet_size_modifier: 1.0,
                war_threshold_modifier: 1.0,
                decision_speed_modifier: 1.0,
            },
            
            AiPersonality::Chaos => Self {
                base_aggression: 0.6,
                pressure_multiplier: 2.0,      // Extremely affected by pressure
                panic_threshold: 0.5,          // Panics early and often
                time_awareness: 0.3,           // Poor time awareness
                bet_size_modifier: 1.0,
                war_threshold_modifier: 1.0,
                decision_speed_modifier: 1.0,
            },
        }
    }

    fn apply_time_pressure(&mut self, tournament_timer: &TournamentTimer, time_phase: TimePhase) {
        let pressure = tournament_timer.pressure_level;
        
        // Check if AI is panicking
        let is_panicking = pressure > self.panic_threshold;
        
        // Modify betting behavior based on pressure and personality
        match time_phase {
            TimePhase::Opening | TimePhase::MidGame => {
                // Minimal pressure effects
                self.bet_size_modifier = 1.0 + pressure * 0.1;
                self.decision_speed_modifier = 1.0;
            },
            
            TimePhase::Warning => {
                // Moderate pressure - start changing behavior
                self.bet_size_modifier = 1.0 + pressure * self.pressure_multiplier * 0.3;
                self.war_threshold_modifier = 1.0 + pressure * 0.2;
                self.decision_speed_modifier = 1.0 + pressure * 0.5;  // Faster decisions
            },
            
            TimePhase::Critical => {
                // High pressure - significant behavior changes
                if is_panicking {
                    self.bet_size_modifier = 1.0 + pressure * self.pressure_multiplier * 0.8;
                    self.war_threshold_modifier = 1.0 + pressure * 0.5;  // More likely to war
                    self.decision_speed_modifier = 1.0 + pressure * 1.0;
                } else {
                    // Non-panicking AIs become more conservative under extreme pressure
                    self.bet_size_modifier = 1.0 - pressure * 0.2;
                    self.war_threshold_modifier = 1.0 - pressure * 0.3;
                    self.decision_speed_modifier = 1.0 + pressure * 0.3;
                }
            },
            
            TimePhase::Final => {
                // Desperation time - dramatic behavior changes
                if is_panicking {
                    // Panicking AIs go all-in mentally
                    self.bet_size_modifier = 2.0;  // Double bet sizes
                    self.war_threshold_modifier = 2.0;  // Almost always war
                    self.decision_speed_modifier = 3.0;  // Very fast decisions
                } else {
                    // Composed AIs try to make every hand count
                    self.bet_size_modifier = 1.5;  // Bigger bets but controlled
                    self.war_threshold_modifier = 0.5;  // Avoid wars to preserve chips
                    self.decision_speed_modifier = 1.0;  // Maintain steady pace
                }
            },
            
            TimePhase::Overtime => {
                // Everything on the line - max modifications
                self.bet_size_modifier = 3.0;  // Maximum bets
                self.war_threshold_modifier = if is_panicking { 3.0 } else { 0.2 };
                self.decision_speed_modifier = 5.0;  // Instant decisions
            },
        }
    }

    fn get_pressure_adjusted_bet(&self, base_bet: u32, current_chips: u32) -> u32 {
        let adjusted_bet = (base_bet as f32 * self.bet_size_modifier) as u32;
        
        // Ensure we don't bet more than we have
        adjusted_bet.min(current_chips)
    }

    fn get_pressure_adjusted_war_probability(&self, base_probability: f32) -> f32 {
        (base_probability * self.war_threshold_modifier).clamp(0.0, 1.0)
    }

    fn get_decision_time(&self, base_time: f32) -> f32 {
        (base_time / self.decision_speed_modifier).max(0.1)  // Min 0.1 second decisions
    }
}
```

**Personality-Based Pressure Response:**

**Conservative Bots:**
- **Low pressure multiplier**: Steady under stress
- **High panic threshold**: Takes extreme pressure to break
- **Excellent time awareness**: Makes time-conscious decisions

**Aggressive Bots:**
- **High pressure multiplier**: Pressure amplifies aggression
- **Early panic threshold**: Goes into overdrive quickly
- **Moderate time awareness**: Focused on winning, not clock

**Chaos Bots:**
- **Extreme pressure multiplier**: Wildly unpredictable under stress
- **Very early panic**: Starts panicking halfway through
- **Poor time awareness**: Often surprised by time running out

**Pressure Effect Phases:**

**Opening/MidGame**: Minimal behavioral changes
**Warning**: 20-50% increase in aggression and speed
**Critical**: Major changes - panic vs composure split
**Final**: Dramatic changes - desperation vs control
**Overtime**: Maximum modifications - everything on the line

## Section 3: Tournament Timer UI System

Let's create a dramatic countdown display that escalates tension:

```rust
// Timer display components
#[derive(Component)]
struct TimerDisplay;

#[derive(Component)]
struct TimerBackground;

#[derive(Component)]
struct PressureOverlay;

#[derive(Component)]
struct UrgencyPulse {
    intensity: f32,
}

fn setup_tournament_timer_ui(mut commands: Commands) {
    // Main timer container
    commands.spawn((
        TimerDisplay,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Percent(50.0),
            width: Val::Px(200.0),
            height: Val::Px(80.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(3.0)),
            transform: Transform::from_translation(Vec3::new(-100.0, 0.0, 100.0)), // Center it
            ..default()
        },
        TimerBackground,
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
    ))
    .with_children(|parent| {
        // Timer label
        parent.spawn((
            Text::new("TIME REMAINING"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
        
        // Timer value
        parent.spawn((
            Text::new("02:00"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            UrgencyPulse { intensity: 0.0 },
        ));
    });

    // Full-screen pressure overlay (initially invisible)
    commands.spawn((
        PressureOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.0)), // Transparent red initially
        ZIndex(50), // Above most other UI
    ));
}

fn update_timer_display(
    tournament_timer: Res<TournamentTimer>,
    time_phase: Res<TimePhase>,
    mut timer_query: Query<&mut Text, With<UrgencyPulse>>,
    mut background_query: Query<&mut BackgroundColor, With<TimerBackground>>,
    mut border_query: Query<&mut BorderColor, With<TimerBackground>>,
    mut overlay_query: Query<&mut BackgroundColor, (With<PressureOverlay>, Without<TimerBackground>)>,
    mut pulse_query: Query<(&mut UrgencyPulse, &mut TextColor)>,
) {
    // Update timer text
    if let Ok(mut text) = timer_query.get_single_mut() {
        text.0 = tournament_timer.get_time_remaining_display();
    }

    // Update visual urgency based on time phase
    let pressure = tournament_timer.pressure_level;
    
    // Timer background color changes
    if let Ok(mut background) = background_query.get_single_mut() {
        let urgency_color = match *time_phase {
            TimePhase::Opening | TimePhase::MidGame => Color::srgba(0.0, 0.0, 0.0, 0.8),
            TimePhase::Warning => Color::srgba(0.3, 0.15, 0.0, 0.8),  // Orange tint
            TimePhase::Critical => Color::srgba(0.5, 0.1, 0.0, 0.8),  // Red-orange
            TimePhase::Final => Color::srgba(0.7, 0.0, 0.0, 0.8),     // Deep red
            TimePhase::Overtime => Color::srgba(1.0, 0.0, 0.0, 0.9),  // Bright red
        };
        *background = BackgroundColor(urgency_color);
    }

    // Border color and thickness changes
    if let Ok(mut border) = border_query.get_single_mut() {
        let border_color = match *time_phase {
            TimePhase::Opening | TimePhase::MidGame => Color::srgb(0.5, 0.5, 0.5),
            TimePhase::Warning => Color::srgb(1.0, 0.5, 0.0),  // Orange
            TimePhase::Critical => Color::srgb(1.0, 0.3, 0.0), // Red-orange  
            TimePhase::Final => Color::srgb(1.0, 0.0, 0.0),    // Pure red
            TimePhase::Overtime => Color::srgb(1.0, 1.0, 0.0), // Yellow (overtime)
        };
        *border = BorderColor(border_color);
    }

    // Full-screen pressure overlay
    if let Ok(mut overlay) = overlay_query.get_single_mut() {
        let overlay_alpha = match *time_phase {
            TimePhase::Opening | TimePhase::MidGame => 0.0,
            TimePhase::Warning => pressure * 0.05,   // Very subtle red tint
            TimePhase::Critical => pressure * 0.1,   // More noticeable
            TimePhase::Final => pressure * 0.15,     // Clear red overlay
            TimePhase::Overtime => 0.2,              // Constant strong overlay
        };
        *overlay = BackgroundColor(Color::srgba(1.0, 0.0, 0.0, overlay_alpha));
    }

    // Pulsing text effect
    if let Ok((mut pulse, mut text_color)) = pulse_query.get_single_mut() {
        pulse.intensity = pressure;
        
        if tournament_timer.pulse_timer.just_finished() {
            // Flash between normal and bright during critical phases
            let flash_intensity = match *time_phase {
                TimePhase::Critical => 0.3,
                TimePhase::Final => 0.5,
                TimePhase::Overtime => 1.0,
                _ => 0.0,
            };
            
            if flash_intensity > 0.0 {
                let flash_color = Color::srgb(1.0, 1.0 - flash_intensity, 1.0 - flash_intensity);
                *text_color = TextColor(flash_color);
            } else {
                *text_color = TextColor(Color::WHITE);
            }
        }
    }
}
```

**Timer Visual Progression:**

**Opening/MidGame:**
- Gray background and border
- White text
- No screen effects

**Warning Phase:**
- Orange background tint
- Orange border
- Subtle red screen overlay (5% alpha)

**Critical Phase:**
- Red-orange background
- Red border  
- Noticeable red overlay (10% alpha)
- Text starts flashing

**Final Phase:**
- Deep red background
- Pure red border
- Strong red overlay (15% alpha)
- Intense text flashing

**Overtime:**
- Bright red background
- Yellow border (different from countdown)
- Constant red overlay (20% alpha)
- Maximum text brightness

## Section 4: Screen Shake and Audio Pressure

Let's add physical tension through screen effects and audio cues:

```rust
// Screen shake system
#[derive(Resource)]
struct ScreenShake {
    intensity: f32,
    duration: f32,
    frequency: f32,
    offset: Vec2,
}

fn update_screen_shake(
    mut shake: ResMut<ScreenShake>,
    tournament_timer: Res<TournamentTimer>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    let target_intensity = tournament_timer.shake_intensity;
    
    // Smoothly adjust shake intensity
    shake.intensity = shake.intensity * 0.9 + target_intensity * 0.1;
    
    if shake.intensity > 0.1 {
        // Calculate shake offset using sine waves for smooth motion
        let time_sec = time.elapsed_seconds();
        shake.offset = Vec2::new(
            (time_sec * shake.frequency).sin() * shake.intensity,
            (time_sec * shake.frequency * 1.3).cos() * shake.intensity,
        );
        
        // Apply shake to camera
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation.x = shake.offset.x;
            camera_transform.translation.y = shake.offset.y;
        }
    } else {
        // Reset camera position when no shake
        shake.offset = Vec2::ZERO;
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation.x = 0.0;
            camera_transform.translation.y = 0.0;
        }
    }
}

// Audio pressure system
#[derive(Resource)]
struct TournamentAudio {
    beep_sound: Handle<AudioSource>,
    warning_sound: Handle<AudioSource>,
    critical_sound: Handle<AudioSource>,
    overtime_sound: Handle<AudioSource>,
    last_beep: f32,
}

fn update_audio_pressure(
    mut tournament_audio: ResMut<TournamentAudio>,
    tournament_timer: Res<TournamentTimer>,
    time_phase: Res<TimePhase>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_seconds();
    
    // Check if it's time for a beep
    if current_time - tournament_audio.last_beep >= tournament_timer.beep_interval {
        let sound_handle = match *time_phase {
            TimePhase::Opening | TimePhase::MidGame => None,
            TimePhase::Warning => Some(tournament_audio.warning_sound.clone()),
            TimePhase::Critical => Some(tournament_audio.critical_sound.clone()),
            TimePhase::Final => Some(tournament_audio.critical_sound.clone()),
            TimePhase::Overtime => Some(tournament_audio.overtime_sound.clone()),
        };
        
        if let Some(sound) = sound_handle {
            // Play sound with volume based on urgency
            let volume = match *time_phase {
                TimePhase::Warning => 0.3,
                TimePhase::Critical => 0.5,
                TimePhase::Final => 0.7,
                TimePhase::Overtime => 1.0,
                _ => 0.0,
            };
            
            commands.spawn((
                AudioPlayer(sound),
                PlaybackSettings {
                    volume: Volume::new(volume),
                    mode: PlaybackMode::Despawn,
                    ..default()
                },
            ));
            
            tournament_audio.last_beep = current_time;
        }
    }
}

// Heartbeat effect for maximum tension
fn update_heartbeat_effect(
    tournament_timer: Res<TournamentTimer>,
    time_phase: Res<TimePhase>,
    time: Res<Time>,
    mut ui_query: Query<(&mut BackgroundColor, &mut Transform), With<TimerDisplay>>,
) {
    if matches!(*time_phase, TimePhase::Final | TimePhase::Overtime) {
        let heartbeat_frequency = match *time_phase {
            TimePhase::Final => 2.0,      // 2 beats per second
            TimePhase::Overtime => 3.0,   // 3 beats per second
            _ => 1.0,
        };
        
        let time_sec = time.elapsed_seconds();
        let heartbeat = ((time_sec * heartbeat_frequency).sin() * 0.5 + 0.5).powi(3);
        
        if let Ok((mut background, mut transform)) = ui_query.get_single_mut() {
            // Pulse the timer size with heartbeat
            let scale = 1.0 + heartbeat * 0.1;
            transform.scale = Vec3::splat(scale);
            
            // Pulse the background brightness
            let brightness = 0.8 + heartbeat * 0.2;
            background.0 = Color::srgba(1.0, 0.0, 0.0, brightness);
        }
    }
}
```

**Screen Shake Mathematics:**
```rust
shake.offset = Vec2::new(
    (time_sec * shake.frequency).sin() * shake.intensity,
    (time_sec * shake.frequency * 1.3).cos() * shake.intensity,
);
```
- Uses sine/cosine waves for smooth, natural-feeling shake
- Different frequencies for X/Y create complex motion
- Intensity scales the overall magnitude
- Creates organic trembling rather than jarring jumps

**Audio Pressure Escalation:**
- **Warning**: Soft beeps every 2 seconds (30% volume)
- **Critical**: Louder beeps every 2 seconds (50% volume)  
- **Final**: Urgent beeps every 0.5 seconds (70% volume)
- **Overtime**: Constant intense beeps (100% volume)

**Heartbeat Effect:**
```rust
let heartbeat = ((time_sec * heartbeat_frequency).sin() * 0.5 + 0.5).powi(3);
```
- Sine wave normalized to [0, 1] range
- Cubed (`.powi(3)`) for sharp pulse shape
- Creates realistic heartbeat rhythm
- Applied to both scale and brightness

## Section 5: Overtime and Final Resolution

Let's implement the dramatic conclusion system:

```rust
// Overtime management
fn handle_overtime_system(
    mut tournament_timer: ResMut<TournamentTimer>,
    mut tournament_state: ResMut<TournamentState>,
    active_hands_query: Query<Entity, With<ActiveHand>>,
    mut next_phase: ResMut<NextState<GamePhase>>,
) {
    if tournament_timer.is_overtime {
        let active_hands = active_hands_query.iter().count();
        
        if active_hands == 0 {
            // All hands completed - determine winner
            determine_tournament_winner(&mut tournament_state, &mut next_phase);
            tournament_timer.is_overtime = false;
        } else {
            // Update overtime display
            tournament_timer.overtime_hands_remaining = active_hands as u32;
        }
    }
}

fn determine_tournament_winner(
    tournament_state: &mut TournamentState,
    next_phase: &mut ResMut<NextState<GamePhase>>,
) {
    // Sort players by chip count
    tournament_state.players.sort_by(|a, b| b.chips.cmp(&a.chips));
    
    if let Some(winner) = tournament_state.players.first() {
        info!("🏆 TOURNAMENT WINNER: {} with {} chips!", 
              winner.name, winner.chips);
        
        // Award the entire prize pool to the winner
        winner.chips += tournament_state.prize_pool;
        
        // Transition to victory screen
        next_phase.set(GamePhase::TournamentComplete);
    }
}

// Prize distribution system
#[derive(Resource)]
struct PrizeDistribution {
    total_pool: u32,
    winner_percentage: f32,      // 60% to winner
    second_percentage: f32,      // 25% to second place
    third_percentage: f32,       // 15% to third place
}

impl Default for PrizeDistribution {
    fn default() -> Self {
        Self {
            total_pool: 0,
            winner_percentage: 0.6,
            second_percentage: 0.25,
            third_percentage: 0.15,
        }
    }
}

fn distribute_prizes(
    tournament_state: &mut TournamentState,
    prize_distribution: &PrizeDistribution,
) {
    // Sort by final chip count
    tournament_state.players.sort_by(|a, b| b.chips.cmp(&a.chips));
    
    let total_pool = tournament_state.prize_pool;
    
    // Award prizes to top 3
    if let Some(first) = tournament_state.players.get_mut(0) {
        let prize = (total_pool as f32 * prize_distribution.winner_percentage) as u32;
        first.tournament_stats.total_winnings += prize as i32;
        info!("🥇 {} wins ${}", first.name, prize);
    }
    
    if let Some(second) = tournament_state.players.get_mut(1) {
        let prize = (total_pool as f32 * prize_distribution.second_percentage) as u32;
        second.tournament_stats.total_winnings += prize as i32;
        info!("🥈 {} gets ${}", second.name, prize);
    }
    
    if let Some(third) = tournament_state.players.get_mut(2) {
        let prize = (total_pool as f32 * prize_distribution.third_percentage) as u32;
        third.tournament_stats.total_winnings += prize as i32;
        info!("🥉 {} gets ${}", third.name, prize);
    }
}

// Dramatic finish detection
fn detect_dramatic_finish(
    tournament_state: &TournamentState,
    tournament_timer: &TournamentTimer,
) -> FinishType {
    let time_remaining = tournament_timer.duration.remaining_secs();
    let leader_chips = tournament_state.players.iter()
        .map(|p| p.chips)
        .max()
        .unwrap_or(0);
    let second_chips = tournament_state.players.iter()
        .map(|p| p.chips)
        .filter(|&chips| chips < leader_chips)
        .max()
        .unwrap_or(0);
    
    let chip_gap = leader_chips - second_chips;
    let average_chips = tournament_state.players.iter()
        .map(|p| p.chips)
        .sum::<u32>() / tournament_state.players.len() as u32;
    
    if time_remaining <= 5.0 && chip_gap < average_chips / 4 {
        FinishType::NailBiter   // Very close with little time
    } else if tournament_timer.is_overtime {
        FinishType::Overtime    // Went to overtime
    } else if chip_gap < average_chips / 8 {
        FinishType::PhotoFinish // Extremely close
    } else if leader_chips > average_chips * 2 {
        FinishType::Dominant    // Clear winner
    } else {
        FinishType::Standard    // Regular finish
    }
}

#[derive(Debug, Clone, Copy)]
enum FinishType {
    NailBiter,     // Close race with time pressure
    PhotoFinish,   // Extremely close final chips
    Overtime,      // Went to overtime to resolve
    Dominant,      // Clear winner throughout
    Standard,      // Regular competitive finish
}
```

**Winner Determination Logic:**
1. **Chip Count**: Primary ranking criteria
2. **Active Hands**: Wait for all hands to complete in overtime
3. **Prize Pool**: Winner gets accumulated tournament chips
4. **Immediate Victory**: No tiebreakers - highest chips wins

**Prize Distribution Options:**
- **Winner-Take-All**: 100% to first place (dramatic)
- **Top-Heavy**: 60%/25%/15% split (balanced drama)
- **Even Split**: More egalitarian distribution

**Dramatic Finish Detection:**
- **NailBiter**: Close race + time pressure
- **PhotoFinish**: Chip gap < 12.5% of average
- **Overtime**: Any overtime finish is dramatic
- **Dominant**: Leader has 2x average chips
- Used for post-game celebration intensity

## Section 6: Victory Screen and Tournament Results

Let's create a climactic conclusion to our tournament:

```rust
// Victory screen components
#[derive(Component)]
struct VictoryScreen;

#[derive(Component)]
struct WinnerDisplay;

#[derive(Component)]
struct FinalStandings;

#[derive(Component)]
struct TournamentStats;

fn setup_victory_screen(
    mut commands: Commands,
    tournament_state: Res<TournamentState>,
    finish_type: Res<FinishType>,
) {
    // Full-screen victory overlay
    commands.spawn((
        VictoryScreen,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        ZIndex(1000), // Above everything
    ))
    .with_children(|parent| {
        // Winner announcement
        spawn_winner_announcement(parent, &tournament_state, &finish_type);
        
        // Final standings
        spawn_final_standings(parent, &tournament_state);
        
        // Tournament statistics
        spawn_tournament_statistics(parent, &tournament_state);
        
        // Play again button
        spawn_play_again_button(parent);
    });
}

fn spawn_winner_announcement(
    parent: &mut ChildBuilder,
    tournament_state: &TournamentState,
    finish_type: &FinishType,
) {
    if let Some(winner) = tournament_state.players.first() {
        // Victory banner
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            // "WINNER" text
            parent.spawn((
                Text::new("🏆 TOURNAMENT WINNER 🏆"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.0)), // Gold
            ));
            
            // Winner name
            parent.spawn((
                Text::new(&winner.name),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            
            // Final chip count
            parent.spawn((
                Text::new(format!("Final Chips: {}", winner.chips)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
            
            // Finish type descriptor
            let finish_description = match finish_type {
                FinishType::NailBiter => "What a nail-biting finish!",
                FinishType::PhotoFinish => "Photo finish! Incredibly close!",
                FinishType::Overtime => "Overtime thriller!",
                FinishType::Dominant => "Dominant performance!",
                FinishType::Standard => "Well-played tournament!",
            };
            
            parent.spawn((
                Text::new(finish_description),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.6, 0.0)),
            ));
        });
    }
}

fn spawn_final_standings(
    parent: &mut ChildBuilder,
    tournament_state: &TournamentState,
) {
    parent.spawn((
        FinalStandings,
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(400.0),
            padding: UiRect::all(Val::Px(20.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
    ))
    .with_children(|parent| {
        // Header
        parent.spawn((
            Text::new("FINAL STANDINGS"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.6, 0.0)),
        ));
        
        // Player rankings
        for (index, player) in tournament_state.players.iter().enumerate() {
            let position_emoji = match index {
                0 => "🥇",
                1 => "🥈", 
                2 => "🥉",
                _ => "  ",
            };
            
            let position_color = match index {
                0 => Color::srgb(1.0, 0.8, 0.0), // Gold
                1 => Color::srgb(0.7, 0.7, 0.7), // Silver
                2 => Color::srgb(0.8, 0.5, 0.2), // Bronze
                _ => Color::WHITE,
            };
            
            parent.spawn((
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
            ))
            .with_children(|parent| {
                // Position and name
                parent.spawn((
                    Text::new(format!("{} {}. {}", 
                        position_emoji, 
                        index + 1, 
                        player.name
                    )),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(position_color),
                ));
                
                // Final chips
                parent.spawn((
                    Text::new(format!("{} chips", player.chips)),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        }
    });
}

fn spawn_tournament_statistics(
    parent: &mut ChildBuilder,
    tournament_state: &TournamentState,
) {
    let total_hands: u32 = tournament_state.players.iter()
        .map(|p| p.tournament_stats.hands_won + p.tournament_stats.hands_lost)
        .sum();
    
    let total_wars: u32 = tournament_state.players.iter()
        .map(|p| p.tournament_stats.wars_won + p.tournament_stats.wars_lost)
        .sum();
    
    let biggest_win = tournament_state.players.iter()
        .map(|p| p.tournament_stats.biggest_win)
        .max()
        .unwrap_or(0);
    
    parent.spawn((
        TournamentStats,
        Node {
            flex_direction: FlexDirection::Column,
            width: Val::Px(300.0),
            padding: UiRect::all(Val::Px(15.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
    ))
    .with_children(|parent| {
        // Stats header
        parent.spawn((
            Text::new("TOURNAMENT STATISTICS"),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
        
        let stats = [
            ("Total Hands Played", format!("{}", total_hands)),
            ("Wars Fought", format!("{}", total_wars)),
            ("Biggest Single Win", format!("${}", biggest_win)),
            ("Tournament Duration", format!("2:00")), // Always 2 minutes
        ];
        
        for (label, value) in stats {
            parent.spawn((
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    margin: UiRect::vertical(Val::Px(3.0)),
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
                    TextColor(Color::WHITE),
                ));
                
                parent.spawn((
                    Text::new(value),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.8, 0.0)),
                ));
            });
        }
    });
}
```

**Victory Screen Design:**
- **Full-screen overlay**: Dramatic black background
- **Winner spotlight**: Large, prominent announcement
- **Final standings**: Complete ranking with medals
- **Tournament stats**: Summary of the competition
- **Visual hierarchy**: Gold for winner, silver/bronze for podium

**Finish Type Integration:**
Different finish types get different victory messages:
- **NailBiter**: Emphasizes the tension and close finish
- **PhotoFinish**: Highlights how incredibly close it was
- **Overtime**: Celebrates the dramatic overtime resolution
- **Dominant**: Recognizes a commanding performance
- **Standard**: Provides a respectful conclusion

## Testing Your Tournament Timer System

At this point, you should be able to:

1. **See Countdown Timer**: 2-minute timer prominently displayed at top
2. **Experience Escalating Pressure**: Visual/audio effects intensify over time  
3. **Watch AI Behavior Change**: Bots make different decisions under time pressure
4. **Enter Overtime**: Time expires but current hands complete
5. **See Dramatic Victory Screen**: Winner announced with full tournament results

## Key Concepts Mastered

1. **Time-Pressure Game Design**: Creating urgency and tension through temporal constraints
   - Escalating visual/audio cues
   - Behavioral changes under pressure
   - Fair overtime resolution

2. **Psychological Modeling**: Simulating human responses to stress
   - Different personalities react differently
   - Panic thresholds and pressure multipliers
   - Time awareness affecting decision quality

3. **Dynamic UI Systems**: Interfaces that adapt to game state
   - Color/size changes based on urgency
   - Screen shake and overlay effects
   - Heartbeat rhythms for maximum tension

4. **Audio-Visual Coordination**: Synchronized effects for maximum impact
   - Beeping frequency increases with pressure
   - Screen shake intensity matches audio urgency
   - Color progressions create atmosphere

5. **Tournament Resolution**: Fair and dramatic conclusions
   - Overtime mechanics for incomplete hands
   - Winner determination and prize distribution
   - Victory celebration matching finish type

## Exercises

1. **Add Countdown Announcements**: Voice countdown for final 10 seconds

2. **Implement Sudden Death**: If tied when timer expires, play one final elimination hand

3. **Create Pressure Achievements**: Award special recognition for performance under pressure

4. **Add Time Bonus System**: Bonus chips for winning with time remaining

5. **Implement Multi-Round Tournaments**: Multiple 2-minute rounds with cumulative scoring

## Next Steps

In Part 10, we'll add:
- Advanced analytics and performance tracking
- Machine learning for AI improvement
- Predictive modeling for tournament outcomes
- Professional-grade statistical analysis

The timed tournament creates the perfect high-pressure environment for showcasing our sophisticated AI and analytics systems!