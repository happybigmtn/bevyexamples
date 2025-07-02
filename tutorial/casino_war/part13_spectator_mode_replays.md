# Casino War Tutorial - Part 13: Spectator Mode & Replays - Learn from the Masters

## What We're Building in Part 13

Creating a comprehensive spectator and replay system:

1. **Live Spectator Mode**: Watch ongoing tournaments in real-time
2. **Replay Recording System**: Capture every detail for later playback
3. **Interactive Replay Controls**: Pause, rewind, slow-motion analysis
4. **Commentary System**: Add notes and analysis to key moments
5. **Learning Tools**: Extract insights from top player performances

## Understanding Game Recording Systems

### The Replay System Problem

Imagine you're building a sports broadcast system for esports. We need a system that:
- Records games with perfect fidelity for replay
- Allows spectators to watch from multiple perspectives
- Provides analysis tools for learning
- Maintains small file sizes despite recording everything
- Enables sharing and studying great moments

Let's think about this like building a film production system:
1. **Recording**: Multiple cameras capturing every angle
2. **Storage**: Efficient compression without losing detail
3. **Playback**: Smooth replay with full controls
4. **Analysis**: Tools to understand what happened
5. **Distribution**: Easy sharing of highlights

In programming terms, this is a **deterministic replay system** combined with **spectator architecture** and **analysis tools**. We need to capture game state efficiently while providing powerful viewing tools.

## Section 1: Replay Recording Architecture

First, let's design our replay recording system. Think of this like creating a flight data recorder:

```rust
// Core replay data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TournamentReplay {
    // Metadata
    replay_id: Uuid,
    recorded_at: SystemTime,
    game_version: String,
    duration: Duration,
    
    // Tournament info
    tournament_config: TournamentConfig,
    participants: Vec<ReplayParticipant>,
    final_standings: Vec<FinalStanding>,
    
    // The actual replay data
    initial_state: InitialGameState,
    events: Vec<TimestampedEvent>,
    checkpoints: Vec<StateCheckpoint>,
    
    // Analysis data
    highlights: Vec<Highlight>,
    commentary: Vec<Commentary>,
    statistics: ReplayStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimestampedEvent {
    timestamp: f64,  // Seconds since start
    frame: u64,      // Frame number for precise sync
    event: ReplayEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ReplayEvent {
    // Player inputs
    PlayerAction {
        player_id: Uuid,
        action: PlayerAction,
    },
    
    // Game state changes
    CardDealt {
        player_id: Uuid,
        card: Card,
        position: Vec3,
    },
    
    BetPlaced {
        player_id: Uuid,
        amount: u32,
    },
    
    HandResult {
        winner: Uuid,
        chips_won: u32,
        was_war: bool,
    },
    
    // AI decisions
    AiDecision {
        ai_id: Uuid,
        decision_type: AiDecisionType,
        reasoning: String,  // For learning
    },
    
    // Visual events (for smooth replay)
    Animation {
        entity_id: u64,
        animation_type: AnimationType,
        duration: f32,
    },
    
    // Tournament events
    PlayerEliminated {
        player_id: Uuid,
        final_position: usize,
    },
    
    TournamentPhaseChange {
        new_phase: TournamentPhase,
    },
}

// Efficient state checkpointing
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateCheckpoint {
    timestamp: f64,
    frame: u64,
    compressed_state: Vec<u8>,  // LZ4 compressed game state
}

// Recording system
#[derive(Resource)]
struct ReplayRecorder {
    current_recording: Option<RecordingSession>,
    event_buffer: Vec<TimestampedEvent>,
    checkpoint_interval: Duration,
    last_checkpoint: Instant,
    compression_level: CompressionLevel,
}

struct RecordingSession {
    start_time: Instant,
    start_frame: u64,
    replay_data: TournamentReplay,
    state_hasher: StateHasher,
}

impl ReplayRecorder {
    fn start_recording(&mut self, tournament_config: TournamentConfig) -> Result<(), RecordError> {
        if self.current_recording.is_some() {
            return Err(RecordError::AlreadyRecording);
        }
        
        let initial_state = self.capture_initial_state();
        
        self.current_recording = Some(RecordingSession {
            start_time: Instant::now(),
            start_frame: 0,
            replay_data: TournamentReplay {
                replay_id: Uuid::new_v4(),
                recorded_at: SystemTime::now(),
                game_version: env!("CARGO_PKG_VERSION").to_string(),
                duration: Duration::default(),
                tournament_config,
                participants: Vec::new(),
                final_standings: Vec::new(),
                initial_state,
                events: Vec::new(),
                checkpoints: Vec::new(),
                highlights: Vec::new(),
                commentary: Vec::new(),
                statistics: ReplayStatistics::default(),
            },
            state_hasher: StateHasher::new(),
        });
        
        info!("Started recording tournament");
        Ok(())
    }
    
    fn record_event(&mut self, event: ReplayEvent, world: &World) {
        if let Some(session) = &mut self.current_recording {
            let timestamp = session.start_time.elapsed().as_secs_f64();
            let frame = world.resource::<FrameCount>().0;
            
            session.replay_data.events.push(TimestampedEvent {
                timestamp,
                frame,
                event,
            });
            
            // Check if we need a checkpoint
            if self.last_checkpoint.elapsed() > self.checkpoint_interval {
                self.create_checkpoint(world, session);
                self.last_checkpoint = Instant::now();
            }
        }
    }
    
    fn create_checkpoint(&mut self, world: &World, session: &mut RecordingSession) {
        // Capture essential game state
        let state = GameStateSnapshot {
            players: self.extract_player_states(world),
            cards: self.extract_card_states(world),
            tournament_phase: world.resource::<State<TournamentPhase>>().get().clone(),
            chip_counts: self.extract_chip_counts(world),
            current_hand: self.extract_current_hand(world),
        };
        
        // Compress state
        let serialized = bincode::serialize(&state).unwrap();
        let compressed = lz4::block::compress(&serialized, None, false).unwrap();
        
        let checkpoint = StateCheckpoint {
            timestamp: session.start_time.elapsed().as_secs_f64(),
            frame: world.resource::<FrameCount>().0,
            compressed_state: compressed,
        };
        
        session.replay_data.checkpoints.push(checkpoint);
        info!("Created replay checkpoint at {:?}", session.start_time.elapsed());
    }
}

// Deterministic game state for replays
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GameStateSnapshot {
    players: Vec<PlayerState>,
    cards: Vec<CardState>,
    tournament_phase: TournamentPhase,
    chip_counts: HashMap<Uuid, u32>,
    current_hand: Option<HandState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerState {
    id: Uuid,
    name: String,
    position: Vec3,
    chips: u32,
    is_eliminated: bool,
    current_bet: u32,
}

// Highlight detection system
struct HighlightDetector {
    detectors: Vec<Box<dyn HighlightDetectorTrait>>,
}

trait HighlightDetectorTrait: Send + Sync {
    fn check_event(&self, event: &TimestampedEvent, context: &ReplayContext) -> Option<Highlight>;
}

struct ComebackDetector;

impl HighlightDetectorTrait for ComebackDetector {
    fn check_event(&self, event: &TimestampedEvent, context: &ReplayContext) -> Option<Highlight> {
        if let ReplayEvent::HandResult { winner, chips_won, .. } = &event.event {
            // Check if winner was way behind
            if let Some(winner_state) = context.get_player_state(winner) {
                if winner_state.chips < context.average_chips() / 3 && chips_won > 1000 {
                    return Some(Highlight {
                        timestamp: event.timestamp,
                        duration: 30.0,  // 30 second highlight
                        highlight_type: HighlightType::Comeback,
                        title: "Amazing Comeback!".to_string(),
                        description: format!("{} wins {} chips when down to {}", 
                                           winner_state.name, chips_won, winner_state.chips),
                        importance: 0.9,  // High importance
                    });
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Highlight {
    timestamp: f64,
    duration: f32,
    highlight_type: HighlightType,
    title: String,
    description: String,
    importance: f32,  // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HighlightType {
    BigWin,
    Comeback,
    PerfectPlay,
    Elimination,
    CloseCall,
    StrategyShift,
}
```

**Replay Architecture Design:**

**Event-Based Recording:**
- Records player actions, not full state
- Dramatically reduces file size
- Enables perfect reconstruction
- Supports variable playback speed

**Checkpoint System:**
- Periodic full state captures
- Enables fast seeking in replay
- LZ4 compression for efficiency
- Balance between size and seek speed

**Highlight Detection:**
- Automatic identification of key moments
- Multiple detector types
- Importance scoring for filtering
- Timestamps for easy navigation

## Section 2: Spectator Mode System

Now let's implement live spectator functionality:

```rust
// Spectator mode components
#[derive(Component)]
struct Spectator {
    spectating_tournament: Option<Entity>,
    view_mode: ViewMode,
    focused_player: Option<Entity>,
    camera_controller: SpectatorCamera,
}

#[derive(Debug, Clone)]
enum ViewMode {
    Overview,           // See all players
    PlayerFocus,        // Follow specific player
    DirectorMode,       // AI-directed camera
    FreeCam,           // Manual camera control
    Analytics,         // Data visualization view
}

#[derive(Component)]
struct SpectatorCamera {
    base_position: Vec3,
    target_position: Vec3,
    zoom_level: f32,
    rotation: Quat,
    transition_speed: f32,
}

// Spectator system management
#[derive(Resource)]
struct SpectatorSystem {
    active_tournaments: Vec<TournamentInfo>,
    spectator_count: HashMap<Entity, u32>,
    featured_tournament: Option<Entity>,
    director_ai: DirectorAI,
}

struct DirectorAI {
    interest_calculator: InterestCalculator,
    camera_positions: Vec<CameraPreset>,
    current_shot: CameraShot,
    shot_timer: Timer,
}

#[derive(Debug, Clone)]
struct CameraShot {
    preset: CameraPreset,
    target: Option<Entity>,
    duration: f32,
}

#[derive(Debug, Clone)]
enum CameraPreset {
    WideShot { position: Vec3, look_at: Vec3 },
    PlayerCloseUp { offset: Vec3, fov: f32 },
    HandFocus { height: f32, angle: f32 },
    ChipCount { side_angle: f32 },
    Dramatic { orbit_radius: f32, height: f32 },
}

impl DirectorAI {
    fn select_next_shot(&mut self, tournament_state: &TournamentState) -> CameraShot {
        // Calculate interest levels for different shots
        let mut shot_candidates = Vec::new();
        
        // Check for interesting events
        if let Some(close_hand) = self.detect_close_hand(tournament_state) {
            shot_candidates.push((
                CameraShot {
                    preset: CameraPreset::HandFocus { height: 5.0, angle: 45.0 },
                    target: Some(close_hand.player),
                    duration: 10.0,
                },
                0.9,  // High interest
            ));
        }
        
        // Low chip player under pressure
        if let Some(pressure_player) = self.detect_pressure_situation(tournament_state) {
            shot_candidates.push((
                CameraShot {
                    preset: CameraPreset::PlayerCloseUp { 
                        offset: Vec3::new(2.0, 1.5, 3.0), 
                        fov: 60.0 
                    },
                    target: Some(pressure_player),
                    duration: 8.0,
                },
                0.8,
            ));
        }
        
        // Default to overview if nothing interesting
        if shot_candidates.is_empty() {
            shot_candidates.push((
                CameraShot {
                    preset: CameraPreset::WideShot {
                        position: Vec3::new(0.0, 15.0, 10.0),
                        look_at: Vec3::ZERO,
                    },
                    target: None,
                    duration: 5.0,
                },
                0.3,
            ));
        }
        
        // Select shot based on interest weights
        shot_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        shot_candidates[0].0.clone()
    }
}

// Spectator UI overlay
fn setup_spectator_ui(mut commands: Commands) {
    // Main spectator container
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        VisibilityBundle::default(),
        SpectatorUI,
    ))
    .with_children(|parent| {
        // Top bar - Tournament info
        spawn_tournament_info_bar(parent);
        
        // Left panel - Player list
        spawn_player_list_panel(parent);
        
        // Bottom bar - Camera controls
        spawn_camera_controls(parent);
        
        // Right panel - Live stats
        spawn_live_stats_panel(parent);
    });
}

fn spawn_camera_controls(parent: &mut ChildBuilder) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(50.0),
            width: Val::Px(400.0),
            height: Val::Px(60.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
    ))
    .with_children(|parent| {
        // View mode buttons
        let modes = [
            ("Overview", ViewMode::Overview),
            ("Player", ViewMode::PlayerFocus),
            ("Director", ViewMode::DirectorMode),
            ("Free", ViewMode::FreeCam),
            ("Stats", ViewMode::Analytics),
        ];
        
        for (label, mode) in modes {
            parent.spawn((
                Button,
                ViewModeButton { mode },
                Node {
                    padding: UiRect::axes(Val::Px(15.0), Val::Px(8.0)),
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

// Spectator camera movement
fn update_spectator_camera(
    mut spectator_query: Query<(&mut SpectatorCamera, &Spectator)>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    time: Res<Time>,
    tournament_state: Res<TournamentState>,
) {
    for (mut spec_cam, spectator) in &mut spectator_query {
        // Update target position based on view mode
        match spectator.view_mode {
            ViewMode::PlayerFocus => {
                if let Some(player_entity) = spectator.focused_player {
                    // Follow player smoothly
                    if let Some(player_pos) = get_player_position(player_entity, &tournament_state) {
                        spec_cam.target_position = player_pos + Vec3::new(3.0, 5.0, 5.0);
                    }
                }
            },
            
            ViewMode::Overview => {
                // Calculate center of all active players
                let active_positions: Vec<Vec3> = tournament_state.players.iter()
                    .filter(|p| !p.is_eliminated)
                    .filter_map(|p| get_player_position(p.entity, &tournament_state))
                    .collect();
                    
                if !active_positions.is_empty() {
                    let center = active_positions.iter()
                        .fold(Vec3::ZERO, |acc, pos| acc + *pos) / active_positions.len() as f32;
                    
                    // Position camera to see all players
                    let spread = calculate_player_spread(&active_positions);
                    let distance = spread * 2.0 + 10.0;
                    
                    spec_cam.target_position = center + Vec3::new(0.0, distance * 0.7, distance);
                }
            },
            
            ViewMode::DirectorMode => {
                // AI-controlled camera handled separately
            },
            
            ViewMode::FreeCam => {
                // Manual control handled by input system
            },
            
            ViewMode::Analytics => {
                // Fixed overhead view for data viz
                spec_cam.target_position = Vec3::new(0.0, 20.0, 5.0);
            },
        }
        
        // Smooth camera movement
        let delta = time.delta_seconds();
        spec_cam.base_position = spec_cam.base_position.lerp(
            spec_cam.target_position,
            spec_cam.transition_speed * delta
        );
        
        // Apply to actual camera
        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation = spec_cam.base_position;
            
            // Look at table center or focused player
            let look_target = match spectator.view_mode {
                ViewMode::PlayerFocus => {
                    spectator.focused_player
                        .and_then(|e| get_player_position(e, &tournament_state))
                        .unwrap_or(Vec3::ZERO)
                },
                _ => Vec3::ZERO,
            };
            
            transform.look_at(look_target, Vec3::Y);
        }
    }
}

// Picture-in-picture for multiple views
#[derive(Component)]
struct PictureInPicture {
    main_view: ViewportRect,
    pip_views: Vec<(ViewportRect, ViewMode)>,
}

fn setup_pip_cameras(mut commands: Commands) {
    // Main camera (full screen)
    commands.spawn((
        Camera3d::default(),
        Camera {
            viewport: Some(ViewportRect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(1.0, 1.0),
            }),
            priority: 0,
            ..default()
        },
        SpectatorCamera {
            base_position: Vec3::new(0.0, 10.0, 10.0),
            target_position: Vec3::new(0.0, 10.0, 10.0),
            zoom_level: 1.0,
            rotation: Quat::IDENTITY,
            transition_speed: 3.0,
        },
    ));
    
    // PiP camera (bottom right)
    commands.spawn((
        Camera3d::default(),
        Camera {
            viewport: Some(ViewportRect {
                min: Vec2::new(0.7, 0.0),
                max: Vec2::new(1.0, 0.3),
            }),
            priority: 1,  // Renders on top
            ..default()
        },
        PipCamera,
    ));
}
```

**Spectator Features:**

**Multiple View Modes:**
- **Overview**: See entire tournament
- **Player Focus**: Follow specific player
- **Director Mode**: AI cinematography
- **Free Cam**: Manual exploration
- **Analytics**: Data visualization

**Director AI:**
- Automatically finds interesting moments
- Switches between camera angles
- Creates broadcast-quality coverage
- Tracks multiple storylines

**Picture-in-Picture:**
- Multiple simultaneous views
- Main view + smaller windows
- Perfect for tracking multiple players
- Customizable layouts

## Section 3: Replay Playback System

Now let's create the replay viewing experience:

```rust
// Replay playback controller
#[derive(Resource)]
struct ReplayPlayer {
    current_replay: Option<LoadedReplay>,
    playback_state: PlaybackState,
    playback_speed: f32,
    current_time: f64,
    current_frame: u64,
    
    // Seeking and buffering
    seek_buffer: SeekBuffer,
    checkpoint_cache: HashMap<u64, GameStateSnapshot>,
    
    // Analysis tools
    bookmarks: Vec<Bookmark>,
    annotations: Vec<Annotation>,
}

#[derive(Debug, Clone)]
struct LoadedReplay {
    data: TournamentReplay,
    event_index: BTreeMap<u64, usize>,  // Frame -> event index for fast seeking
    duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PlaybackState {
    Playing,
    Paused,
    Seeking,
    Buffering,
}

struct SeekBuffer {
    target_time: Option<f64>,
    nearest_checkpoint: Option<usize>,
    events_to_replay: VecDeque<TimestampedEvent>,
}

impl ReplayPlayer {
    fn load_replay(&mut self, replay_path: &Path) -> Result<(), ReplayError> {
        // Load and decompress replay file
        let compressed = std::fs::read(replay_path)?;
        let decompressed = lz4::block::decompress(&compressed, None)?;
        let replay_data: TournamentReplay = bincode::deserialize(&decompressed)?;
        
        // Build frame index for fast seeking
        let mut event_index = BTreeMap::new();
        for (idx, event) in replay_data.events.iter().enumerate() {
            event_index.insert(event.frame, idx);
        }
        
        let duration = replay_data.duration;
        
        self.current_replay = Some(LoadedReplay {
            data: replay_data,
            event_index,
            duration,
        });
        
        self.playback_state = PlaybackState::Paused;
        self.current_time = 0.0;
        self.current_frame = 0;
        
        info!("Loaded replay: {} duration", format_duration(duration));
        
        Ok(())
    }
    
    fn update_playback(&mut self, delta_time: f32, world: &mut World) {
        if self.playback_state != PlaybackState::Playing {
            return;
        }
        
        if let Some(replay) = &self.current_replay {
            // Advance time based on playback speed
            self.current_time += delta_time as f64 * self.playback_speed as f64;
            
            // Find and apply events up to current time
            while let Some(event) = self.get_next_event() {
                if event.timestamp > self.current_time {
                    break;  // Future event, stop here
                }
                
                self.apply_event(event, world);
                self.current_frame = event.frame;
            }
            
            // Check for end of replay
            if self.current_time >= replay.duration.as_secs_f64() {
                self.playback_state = PlaybackState::Paused;
                info!("Replay finished");
            }
        }
    }
    
    fn seek(&mut self, target_time: f64) {
        if let Some(replay) = &self.current_replay {
            self.playback_state = PlaybackState::Seeking;
            
            // Find nearest checkpoint before target time
            let checkpoint_idx = replay.data.checkpoints
                .binary_search_by(|cp| cp.timestamp.partial_cmp(&target_time).unwrap())
                .unwrap_or_else(|idx| idx.saturating_sub(1));
            
            if let Some(checkpoint) = replay.data.checkpoints.get(checkpoint_idx) {
                // Load checkpoint state
                self.load_checkpoint(checkpoint);
                
                // Queue events from checkpoint to target time
                let start_frame = checkpoint.frame;
                let events_to_replay: VecDeque<_> = replay.data.events.iter()
                    .filter(|e| e.frame > start_frame && e.timestamp <= target_time)
                    .cloned()
                    .collect();
                
                self.seek_buffer = SeekBuffer {
                    target_time: Some(target_time),
                    nearest_checkpoint: Some(checkpoint_idx),
                    events_to_replay,
                };
                
                self.current_time = checkpoint.timestamp;
                self.current_frame = checkpoint.frame;
                
                info!("Seeking to {:.1}s via checkpoint at {:.1}s", 
                      target_time, checkpoint.timestamp);
            }
        }
    }
    
    fn set_playback_speed(&mut self, speed: f32) {
        self.playback_speed = speed.clamp(0.1, 8.0);
        info!("Playback speed set to {}x", self.playback_speed);
    }
}

// Replay UI controls
fn setup_replay_controls(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(50.0),
            width: Val::Px(600.0),
            height: Val::Px(80.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            transform: Transform::from_translation(Vec3::new(-300.0, 0.0, 0.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        ReplayControls,
    ))
    .with_children(|parent| {
        // Timeline scrubber
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            Timeline,
        ))
        .with_children(|parent| {
            // Progress bar
            parent.spawn((
                Node {
                    width: Val::Percent(0.0),  // Updated dynamically
                    height: Val::Percent(100.0),
                    background_color: Color::srgb(0.0, 0.8, 0.0),
                    ..default()
                },
                TimelineProgress,
            ));
            
            // Highlights on timeline
            spawn_timeline_highlights(parent);
        });
        
        // Control buttons
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Play/Pause button
            spawn_play_pause_button(parent);
            
            // Speed controls
            spawn_speed_controls(parent);
            
            // Skip buttons
            spawn_skip_buttons(parent);
            
            // Bookmark button
            spawn_bookmark_button(parent);
        });
    });
}

// Interactive timeline with highlights
fn update_timeline(
    mut timeline_query: Query<&mut Node, With<TimelineProgress>>,
    replay_player: Res<ReplayPlayer>,
    interaction_query: Query<&Interaction, With<Timeline>>,
    window_query: Query<&Window>,
) {
    if let Some(replay) = &replay_player.current_replay {
        let progress = replay_player.current_time / replay.duration.as_secs_f64();
        
        // Update progress bar width
        if let Ok(mut progress_node) = timeline_query.get_single_mut() {
            progress_node.width = Val::Percent(progress as f32 * 100.0);
        }
        
        // Handle timeline clicking for seeking
        if let Ok(interaction) = interaction_query.get_single() {
            if *interaction == Interaction::Pressed {
                if let Ok(window) = window_query.get_single() {
                    if let Some(cursor_pos) = window.cursor_position() {
                        // Calculate seek position based on click
                        let timeline_width = 600.0;  // Should match timeline width
                        let click_x = cursor_pos.x - (window.width() / 2.0 - timeline_width / 2.0);
                        let seek_progress = (click_x / timeline_width).clamp(0.0, 1.0);
                        let seek_time = seek_progress as f64 * replay.duration.as_secs_f64();
                        
                        replay_player.seek(seek_time);
                    }
                }
            }
        }
    }
}

// Bookmark and annotation system
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bookmark {
    id: Uuid,
    timestamp: f64,
    name: String,
    description: String,
    color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Annotation {
    id: Uuid,
    timestamp: f64,
    duration: f32,
    text: String,
    author: String,
    position: Option<Vec2>,  // Screen position for overlay
}

impl ReplayPlayer {
    fn add_bookmark(&mut self, name: String) {
        let bookmark = Bookmark {
            id: Uuid::new_v4(),
            timestamp: self.current_time,
            name,
            description: String::new(),
            color: Color::srgb(1.0, 0.8, 0.0),
        };
        
        self.bookmarks.push(bookmark);
        info!("Added bookmark at {:.1}s", self.current_time);
    }
    
    fn jump_to_bookmark(&mut self, bookmark_id: &Uuid) {
        if let Some(bookmark) = self.bookmarks.iter().find(|b| b.id == *bookmark_id) {
            self.seek(bookmark.timestamp);
        }
    }
    
    fn add_annotation(&mut self, text: String, author: String, position: Option<Vec2>) {
        let annotation = Annotation {
            id: Uuid::new_v4(),
            timestamp: self.current_time,
            duration: 5.0,  // Default 5 second duration
            text,
            author,
            position,
        };
        
        self.annotations.push(annotation);
    }
}

// Highlight reel generation
fn generate_highlight_reel(
    replay: &TournamentReplay,
    max_duration: Duration,
) -> Result<HighlightReel, HighlightError> {
    // Sort highlights by importance
    let mut sorted_highlights = replay.highlights.clone();
    sorted_highlights.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
    
    let mut reel = HighlightReel {
        clips: Vec::new(),
        total_duration: Duration::from_secs(0),
        transitions: Vec::new(),
    };
    
    let mut current_duration = Duration::from_secs(0);
    
    for highlight in sorted_highlights {
        let clip_duration = Duration::from_secs_f32(highlight.duration);
        
        if current_duration + clip_duration > max_duration {
            break;  // Reel is full
        }
        
        // Add padding before and after highlight
        let start = (highlight.timestamp - 5.0).max(0.0);
        let end = highlight.timestamp + highlight.duration + 5.0;
        
        reel.clips.push(HighlightClip {
            start_time: start,
            end_time: end,
            highlight: highlight.clone(),
            transition_in: TransitionType::Fade,
            transition_out: TransitionType::Fade,
        });
        
        current_duration += clip_duration;
    }
    
    reel.total_duration = current_duration;
    
    Ok(reel)
}
```

**Replay Features:**

**Efficient Seeking:**
- Binary search for checkpoints
- Frame-indexed events
- Fast forward/rewind
- Smooth scrubbing

**Playback Controls:**
- Variable speed (0.1x to 8x)
- Frame-by-frame stepping
- Jump to highlights
- Timeline visualization

**Analysis Tools:**
- Bookmarks for key moments
- Text annotations
- Drawing tools (future)
- Highlight extraction

## Section 4: Learning and Sharing Tools

Let's add tools for learning from replays and sharing great moments:

```rust
// Learning analysis system
#[derive(Resource)]
struct LearningAnalyzer {
    pattern_library: PatternLibrary,
    skill_extractor: SkillExtractor,
    comparison_engine: ComparisonEngine,
}

struct PatternLibrary {
    patterns: HashMap<PatternId, GamePattern>,
    examples: HashMap<PatternId, Vec<PatternExample>>,
}

#[derive(Debug, Clone)]
struct GamePattern {
    id: PatternId,
    name: String,
    description: String,
    category: PatternCategory,
    skill_level: SkillLevel,
    
    // Pattern matching criteria
    preconditions: Vec<Condition>,
    actions: Vec<ExpectedAction>,
    outcomes: Vec<Outcome>,
}

#[derive(Debug, Clone)]
enum PatternCategory {
    BettingStrategy,
    WarDecisions,
    ChipManagement,
    PressurePlay,
    Bluffing,
    TimeManagement,
}

impl LearningAnalyzer {
    fn analyze_player_decisions(
        &self,
        replay: &TournamentReplay,
        player_id: Uuid,
    ) -> PlayerAnalysis {
        let mut analysis = PlayerAnalysis {
            player_id,
            total_decisions: 0,
            optimal_decisions: 0,
            missed_opportunities: Vec::new(),
            successful_patterns: Vec::new(),
            improvement_suggestions: Vec::new(),
        };
        
        // Analyze each decision point
        for event in &replay.events {
            if let ReplayEvent::PlayerAction { player_id: pid, action } = &event.event {
                if *pid == player_id {
                    analysis.total_decisions += 1;
                    
                    // Check if decision was optimal
                    let context = self.build_context_at(replay, event.timestamp);
                    let optimal_action = self.calculate_optimal_action(&context);
                    
                    if action == &optimal_action {
                        analysis.optimal_decisions += 1;
                    } else {
                        analysis.missed_opportunities.push(MissedOpportunity {
                            timestamp: event.timestamp,
                            actual_action: action.clone(),
                            optimal_action,
                            expected_value_loss: self.calculate_ev_loss(action, &optimal_action, &context),
                            explanation: self.explain_optimal_play(&context, &optimal_action),
                        });
                    }
                    
                    // Check for pattern matches
                    for pattern in self.pattern_library.patterns.values() {
                        if self.matches_pattern(&context, pattern) {
                            analysis.successful_patterns.push(pattern.id.clone());
                        }
                    }
                }
            }
        }
        
        // Generate improvement suggestions
        analysis.improvement_suggestions = self.generate_suggestions(&analysis);
        
        analysis
    }
    
    fn compare_players(
        &self,
        replay: &TournamentReplay,
        player1: Uuid,
        player2: Uuid,
    ) -> PlayerComparison {
        let analysis1 = self.analyze_player_decisions(replay, player1);
        let analysis2 = self.analyze_player_decisions(replay, player2);
        
        PlayerComparison {
            player1_stats: analysis1,
            player2_stats: analysis2,
            key_differences: self.identify_key_differences(&analysis1, &analysis2),
            similar_situations: self.find_similar_situations(replay, player1, player2),
            recommendation: self.generate_comparison_insights(&analysis1, &analysis2),
        }
    }
}

// Clip sharing system
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedClip {
    id: Uuid,
    title: String,
    description: String,
    replay_id: Uuid,
    start_time: f64,
    end_time: f64,
    
    // Sharing metadata
    created_by: String,
    created_at: SystemTime,
    views: u32,
    likes: u32,
    
    // Analysis data
    highlight_type: HighlightType,
    skill_showcase: Vec<SkillTag>,
    teaching_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SkillTag {
    PerfectTiming,
    ChipManagement,
    PressureDecision,
    Comeback,
    Strategic,
    Lucky,
    Aggressive,
    Conservative,
}

// Clip export system
fn export_clip(
    replay: &TournamentReplay,
    start_time: f64,
    end_time: f64,
    export_settings: &ExportSettings,
) -> Result<ExportedClip, ExportError> {
    // Extract relevant events
    let clip_events: Vec<_> = replay.events.iter()
        .filter(|e| e.timestamp >= start_time && e.timestamp <= end_time)
        .cloned()
        .collect();
    
    // Find relevant checkpoints
    let start_checkpoint = replay.checkpoints.iter()
        .rev()
        .find(|cp| cp.timestamp <= start_time)
        .ok_or(ExportError::NoCheckpoint)?;
    
    // Create standalone clip
    let clip = ExportedClip {
        metadata: ClipMetadata {
            original_replay_id: replay.replay_id,
            duration: end_time - start_time,
            player_count: replay.participants.len(),
            game_version: replay.game_version.clone(),
        },
        
        // Include necessary data for standalone playback
        initial_state: start_checkpoint.clone(),
        events: clip_events,
        
        // Optional: include analysis
        analysis: if export_settings.include_analysis {
            Some(analyze_clip_content(&clip_events))
        } else {
            None
        },
    };
    
    // Compress based on settings
    let serialized = bincode::serialize(&clip)?;
    let compressed = match export_settings.compression {
        CompressionLevel::Fast => lz4::block::compress(&serialized, None, false)?,
        CompressionLevel::Best => lz4::block::compress(&serialized, None, true)?,
        CompressionLevel::None => serialized,
    };
    
    Ok(ExportedClip {
        data: compressed,
        format: ExportFormat::CasinoWarClip,
        size_bytes: compressed.len(),
    })
}

// Tutorial generation from replays
fn generate_tutorial_from_replay(
    replay: &TournamentReplay,
    focus: TutorialFocus,
) -> Tutorial {
    let mut tutorial = Tutorial {
        title: format!("{} Tutorial", focus.title()),
        sections: Vec::new(),
        estimated_duration: Duration::from_secs(0),
    };
    
    match focus {
        TutorialFocus::BettingStrategy => {
            // Find examples of good betting decisions
            let betting_examples = find_betting_examples(replay);
            
            for (idx, example) in betting_examples.iter().enumerate() {
                tutorial.sections.push(TutorialSection {
                    title: format!("Example {}: {}", idx + 1, example.title),
                    timestamp: example.timestamp,
                    duration: 30.0,
                    explanation: example.explanation.clone(),
                    key_points: example.key_points.clone(),
                    pause_points: vec![
                        example.timestamp - 5.0,  // Before decision
                        example.timestamp + 2.0,   // After decision
                    ],
                });
            }
        },
        
        TutorialFocus::WarDecisions => {
            // Find war scenarios
            let war_examples = find_war_examples(replay);
            
            for example in war_examples {
                tutorial.sections.push(create_war_tutorial_section(example));
            }
        },
        
        _ => {}
    }
    
    tutorial
}

#[derive(Debug, Clone)]
enum TutorialFocus {
    BettingStrategy,
    WarDecisions,
    ChipManagement,
    TournamentStrategy,
    AICounterplay,
}

// Community features
#[derive(Resource)]
struct CommunityHub {
    featured_replays: Vec<FeaturedReplay>,
    top_clips: Vec<SharedClip>,
    tutorials: Vec<CommunityTutorial>,
    strategy_guides: Vec<StrategyGuide>,
}

#[derive(Debug, Clone)]
struct FeaturedReplay {
    replay_info: ReplayMetadata,
    featured_reason: String,
    curator_notes: String,
    community_rating: f32,
    download_count: u32,
}

fn upload_replay_to_community(
    replay: &TournamentReplay,
    metadata: UploadMetadata,
) -> Result<(), UploadError> {
    // Validate replay
    if replay.events.is_empty() {
        return Err(UploadError::EmptyReplay);
    }
    
    // Remove any personal information
    let sanitized = sanitize_replay(replay);
    
    // Add community metadata
    let community_replay = CommunityReplay {
        base_replay: sanitized,
        upload_id: Uuid::new_v4(),
        uploader: metadata.username,
        tags: metadata.tags,
        description: metadata.description,
        visibility: metadata.visibility,
    };
    
    // Upload to community server
    // ... (network code)
    
    Ok(())
}
```

**Learning Features:**

**Pattern Recognition:**
- Identifies successful strategies
- Finds missed opportunities
- Calculates EV of decisions
- Provides explanations

**Player Comparison:**
- Side-by-side analysis
- Decision differences
- Skill gap identification
- Improvement recommendations

**Tutorial Generation:**
- Automatic tutorial creation
- Key moment extraction
- Pause points for learning
- Explanatory overlays

**Community Integration:**
- Share clips and replays
- Rate and comment
- Curated collections
- Strategy guides

## Section 5: Performance Optimization

Let's optimize replay storage and playback:

```rust
// Efficient replay storage
struct ReplayCompressor {
    compression_strategy: CompressionStrategy,
    delta_encoder: DeltaEncoder,
}

#[derive(Debug, Clone)]
enum CompressionStrategy {
    Fast,      // LZ4 for speed
    Balanced,  // Zstd for balance
    Maximum,   // Brotli for size
}

impl ReplayCompressor {
    fn compress_replay(&self, replay: &TournamentReplay) -> Result<Vec<u8>, CompressionError> {
        // First pass: Delta encoding for events
        let delta_encoded = self.delta_encoder.encode_events(&replay.events)?;
        
        // Second pass: Remove redundant data
        let deduplicated = self.deduplicate_replay_data(&delta_encoded)?;
        
        // Third pass: Compression
        let compressed = match self.compression_strategy {
            CompressionStrategy::Fast => {
                lz4::block::compress(&deduplicated, None, false)?
            },
            CompressionStrategy::Balanced => {
                zstd::encode_all(&deduplicated[..], 3)?
            },
            CompressionStrategy::Maximum => {
                brotli::compress(&deduplicated, 11)?
            },
        };
        
        info!("Compressed replay from {} to {} bytes ({:.1}% reduction)",
              deduplicated.len(), compressed.len(),
              (1.0 - compressed.len() as f32 / deduplicated.len() as f32) * 100.0);
        
        Ok(compressed)
    }
}

// Delta encoding for sequential events
struct DeltaEncoder;

impl DeltaEncoder {
    fn encode_events(&self, events: &[TimestampedEvent]) -> Result<Vec<u8>, EncodingError> {
        let mut encoded = Vec::new();
        let mut last_timestamp = 0.0;
        let mut last_frame = 0u64;
        
        for event in events {
            // Encode time as delta
            let time_delta = ((event.timestamp - last_timestamp) * 1000.0) as u32;
            encoded.extend_from_slice(&time_delta.to_le_bytes());
            
            // Encode frame as delta
            let frame_delta = event.frame - last_frame;
            encoded.extend_from_slice(&frame_delta.to_le_bytes());
            
            // Encode event type and data
            self.encode_event(&event.event, &mut encoded)?;
            
            last_timestamp = event.timestamp;
            last_frame = event.frame;
        }
        
        Ok(encoded)
    }
    
    fn encode_event(&self, event: &ReplayEvent, buffer: &mut Vec<u8>) -> Result<(), EncodingError> {
        // Use compact binary encoding for events
        match event {
            ReplayEvent::PlayerAction { player_id, action } => {
                buffer.push(0x01);  // Event type ID
                buffer.extend_from_slice(&player_id.as_bytes());
                self.encode_action(action, buffer)?;
            },
            ReplayEvent::CardDealt { player_id, card, position } => {
                buffer.push(0x02);
                buffer.extend_from_slice(&player_id.as_bytes());
                buffer.push(card.suit as u8);
                buffer.push(card.rank as u8);
                // Quantize position to save space
                buffer.extend_from_slice(&(position.x as i16).to_le_bytes());
                buffer.extend_from_slice(&(position.y as i16).to_le_bytes());
                buffer.extend_from_slice(&(position.z as i16).to_le_bytes());
            },
            _ => {
                // Handle other event types...
            }
        }
        
        Ok(())
    }
}

// Streaming replay loader for large files
struct StreamingReplayLoader {
    file_handle: File,
    header: ReplayHeader,
    event_chunks: Vec<ChunkMetadata>,
    loaded_chunks: LruCache<usize, LoadedChunk>,
}

impl StreamingReplayLoader {
    fn new(path: &Path) -> Result<Self, LoadError> {
        let mut file = File::open(path)?;
        
        // Read header
        let header = ReplayHeader::read_from(&mut file)?;
        
        // Read chunk metadata
        let chunk_count = header.chunk_count as usize;
        let mut event_chunks = Vec::with_capacity(chunk_count);
        
        for _ in 0..chunk_count {
            event_chunks.push(ChunkMetadata::read_from(&mut file)?);
        }
        
        Ok(Self {
            file_handle: file,
            header,
            event_chunks,
            loaded_chunks: LruCache::new(NonZeroUsize::new(10).unwrap()),  // Cache 10 chunks
        })
    }
    
    fn get_events_in_range(&mut self, start_time: f64, end_time: f64) -> Result<Vec<TimestampedEvent>, LoadError> {
        let mut events = Vec::new();
        
        // Find relevant chunks
        for (idx, chunk) in self.event_chunks.iter().enumerate() {
            if chunk.end_time >= start_time && chunk.start_time <= end_time {
                // Load chunk if not cached
                if !self.loaded_chunks.contains(&idx) {
                    let loaded = self.load_chunk(idx)?;
                    self.loaded_chunks.put(idx, loaded);
                }
                
                // Get events from chunk
                if let Some(chunk) = self.loaded_chunks.get(&idx) {
                    events.extend(
                        chunk.events.iter()
                            .filter(|e| e.timestamp >= start_time && e.timestamp <= end_time)
                            .cloned()
                    );
                }
            }
        }
        
        Ok(events)
    }
    
    fn load_chunk(&mut self, index: usize) -> Result<LoadedChunk, LoadError> {
        let metadata = &self.event_chunks[index];
        
        // Seek to chunk position
        self.file_handle.seek(SeekFrom::Start(metadata.file_offset))?;
        
        // Read compressed chunk
        let mut compressed = vec![0u8; metadata.compressed_size as usize];
        self.file_handle.read_exact(&mut compressed)?;
        
        // Decompress
        let decompressed = lz4::block::decompress(&compressed, Some(metadata.uncompressed_size as usize))?;
        
        // Deserialize events
        let events: Vec<TimestampedEvent> = bincode::deserialize(&decompressed)?;
        
        Ok(LoadedChunk {
            index,
            events,
            memory_size: decompressed.len(),
        })
    }
}
```

**Storage Optimization:**

**Delta Encoding:**
- Store time/frame differences, not absolutes
- Dramatically reduces timestamp storage
- Quantize positions to 16-bit integers
- Custom binary format for events

**Compression Strategies:**
- **Fast**: LZ4 for quick save/load
- **Balanced**: Zstd for good ratio/speed
- **Maximum**: Brotli for smallest files

**Streaming Loader:**
- Chunked file format
- LRU cache for active chunks
- Load only needed portions
- Supports huge replay files

## Testing Your Spectator System

At this point, you should be able to:

1. **Watch Live Tournaments**: Spectate ongoing games with multiple camera modes
2. **Record Games**: Automatic recording with highlight detection
3. **Load Replays**: Browse and load saved tournament replays
4. **Control Playback**: Full VCR controls with timeline scrubbing
5. **Analyze Decisions**: See optimal plays and missed opportunities

## Key Concepts Mastered

1. **Deterministic Replay**: Recording actions, not states
   - Event-based architecture
   - Checkpoint system for seeking
   - Perfect reproduction

2. **Spectator Architecture**: Multiple viewing perspectives
   - AI director for cinematography
   - Picture-in-picture support
   - Smooth camera transitions

3. **Compression Techniques**: Efficient storage
   - Delta encoding
   - Hierarchical compression
   - Streaming architecture

4. **Learning Tools**: Extract knowledge from games
   - Pattern recognition
   - Decision analysis
   - Tutorial generation

5. **Community Features**: Sharing and learning
   - Clip export system
   - Replay ratings
   - Strategy guides

## Exercises

1. **Add Voice Commentary**: Record audio commentary tracks for replays

2. **Implement Heatmaps**: Visualize betting patterns and decision points

3. **Create AI Analysis**: Use ML to identify key turning points

4. **Add Drawing Tools**: Allow annotations with shapes and arrows

5. **Build Replay Editor**: Cut and combine clips into montages

## Next Steps

In Part 14, we'll add:
- Custom AI training system
- Behavior cloning from replays
- AI personality designer
- Training arena mode

The replay system provides the foundation for learning and improving at the game!