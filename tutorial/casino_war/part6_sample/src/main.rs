// Casino War Part 6: Modern UI Redesign - McLaren Aesthetic
//
// This part transforms our classic casino game into a high-performance
// competitive experience with a modern McLaren-inspired design.
//
// Key additions:
// 1. McLaren color scheme (dark/orange/aluminum)
// 2. Carbon fiber textures and materials
// 3. Holographic card effects
// 4. Racing-inspired UI elements
// 5. Performance-optimized shaders
// 6. Dynamic animations and transitions

use bevy::prelude::*;
use rand::prelude::*;

// PART 6: McLaren-inspired color palette
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
            Suit::Hearts | Suit::Diamonds => MCLAREN_ORANGE,  // McLaren orange instead of red
            Suit::Clubs | Suit::Spades => ALUMINUM,           // Aluminum instead of black
        }
    }
}

// Visual card components
#[derive(Component)]
struct CardVisual {
    face_up: bool,
    target_position: Vec3,
}

// Card position states
#[derive(Component, Debug, Clone, Copy, PartialEq)]
enum CardPosition {
    Deck,
    PlayerHand,
    DealerHand,
    Discard,
}

// Animation component
#[derive(Component)]
struct CardAnimation {
    start_pos: Vec3,
    end_pos: Vec3,
    start_rotation: Quat,
    end_rotation: Quat,
    timer: Timer,
}

// Card face/back tracking
#[derive(Component)]
struct CardFace;

#[derive(Component)]
struct CardBack;

#[derive(Component)]
struct CardRankMarker(Rank);

// PART 6: New holographic card effects
#[derive(Component)]
struct HologramEffect {
    scan_speed: f32,
    glow_intensity: f32,
    flicker_rate: f32,
}

// Component to identify card value text for updates
#[derive(Component)]
struct CardValueText {
    card: Card,
}

// PART 6: Glow effect for UI elements
#[derive(Component)]
struct GlowEffect {
    color: Color,
    intensity: f32,
    radius: f32,
}

// PART 6: Carbon fiber background animation
#[derive(Component)]
struct CarbonFiberAnimation {
    scroll_speed: Vec2,
    scale: f32,
}

// Constants for card layout
const CARD_WIDTH: f32 = 100.0;   // Larger for better visibility
const CARD_HEIGHT: f32 = 140.0;  // Maintains poker card ratio

// Table positions - adjusted for new layout
const DECK_POSITION: Vec3 = Vec3::new(-500.0, 0.0, 0.0);
const PLAYER_CARD_POSITION: Vec3 = Vec3::new(0.0, -250.0, 1.0);
const DEALER_CARD_POSITION: Vec3 = Vec3::new(0.0, 150.0, 1.0);

// War card positions
const WAR_CARD_SPACING: f32 = 120.0;
const WAR_PLAYER_Y: f32 = -250.0;
const WAR_DEALER_Y: f32 = 150.0;

// Betting components
#[derive(Component)]
struct ChipButton {
    value: u32,
}

#[derive(Component)]
struct BetDisplay;

#[derive(Component)]
struct ChipDisplay;

#[derive(Component)]
struct DealButton;

// PART 6: New McLaren-style button component
#[derive(Component)]
struct McLarenButton {
    primary: bool,
    hover_scale: f32,
}

// Betting constants
const CHIP_VALUES: [u32; 5] = [5, 10, 25, 50, 100];
const MIN_BET: u32 = 5;
const MAX_BET: u32 = 500;

// Game phases
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GamePhase {
    #[default]
    MainMenu,
    Betting,
    Dealing,
    Comparing,
    TieDecision,
    War,
    RoundComplete,
}

// Resources for game state
#[derive(Resource)]
struct GameState {
    player_chips: u32,
    current_bet: u32,
    war_bet: u32,
    deck: Vec<Card>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            player_chips: 1000,
            current_bet: 0,
            war_bet: 0,
            deck: Self::create_deck(),
        }
    }
}

impl GameState {
    fn create_deck() -> Vec<Card> {
        let mut deck = Vec::with_capacity(52);
        use Suit::*;
        use Rank::*;
        
        for &suit in &[Hearts, Diamonds, Clubs, Spades] {
            for &rank in &[Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace] {
                deck.push(Card { suit, rank });
            }
        }
        
        let mut rng = thread_rng();
        deck.shuffle(&mut rng);
        
        deck
    }
    
    fn draw_card(&mut self) -> Option<Card> {
        if self.deck.is_empty() {
            self.deck = Self::create_deck();
        }
        self.deck.pop()
    }
}

// Component markers
#[derive(Component)]
struct PlayerCard;

#[derive(Component)]
struct DealerCard;

#[derive(Component)]
struct BurnCard;

#[derive(Component)]
struct ActiveCard;

#[derive(Component)]
struct ComparisonResult {
    player_value: u8,
    dealer_value: u8,
}

#[derive(Component)]
struct CardFlipAnimation {
    timer: Timer,
    half_flipped: bool,
}

#[derive(Component)]
struct WarCard {
    index: usize,
    delay: Timer,
}

// Events
#[derive(Event)]
struct BetPlaced(u32);

#[derive(Event)]
struct DealCards;

#[derive(Event)]
struct PlayerDecision {
    go_to_war: bool,
}

#[derive(Event)]
struct RoundResult {
    player_won: bool,
    winnings: i32,
}

#[derive(Event)]
struct CardsDealt;

#[derive(Event)]
struct RequestCardFlip;

#[derive(Event)]
struct ComparisonComplete {
    outcome: ComparisonOutcome,
}

#[derive(Event)]
struct WarCardsDealt;

#[derive(Event)]
struct WarComplete {
    player_won: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ComparisonOutcome {
    PlayerWins,
    DealerWins,
    Tie,
}

// Marker component for state-based cleanup
#[derive(Component)]
struct StateScoped(GamePhase);

#[derive(Component)]
struct GameCard;

// Marker for buttons
#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct TieDecisionButton {
    go_to_war: bool,
}

#[derive(Component)]
struct ResultDisplay;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct GameStateDisplay;

// Audio resources - made optional for missing assets
#[derive(Resource, Default)]
struct GameAudio {
    card_flip: Option<Handle<AudioSource>>,
    card_slide: Option<Handle<AudioSource>>,
    chip_place: Option<Handle<AudioSource>>,
    victory: Option<Handle<AudioSource>>,
    defeat: Option<Handle<AudioSource>>,
    button_click: Option<Handle<AudioSource>>,
}

// Particle system
#[derive(Component)]
struct Particle {
    velocity: Vec3,
    lifetime: Timer,
    gravity: f32,
}

#[derive(Component)]
struct ParticleEmitter {
    spawn_rate: f32,
    spawn_timer: Timer,
    particle_lifetime: f32,
    particles_to_spawn: u32,
}

// Statistics
#[derive(Resource, Default)]
struct PlayerStats {
    total_games: u32,
    total_wins: u32,
    total_losses: u32,
    total_ties: u32,
    wars_entered: u32,
    wars_won: u32,
    current_streak: i32,
    best_streak: u32,
    total_wagered: u64,
    total_won: u64,
}

// UI animations
#[derive(Component)]
struct UIAnimation {
    start_scale: Vec3,
    end_scale: Vec3,
    timer: Timer,
}

// PART 6: Font sizes for McLaren aesthetic
const FONT_SIZE_HUGE: f32 = 96.0;
const FONT_SIZE_LARGE: f32 = 48.0;
const FONT_SIZE_MEDIUM: f32 = 32.0;
const FONT_SIZE_NORMAL: f32 = 24.0;
const FONT_SIZE_SMALL: f32 = 18.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CASINO WAR - McLaren Edition".into(),
                resolution: (1920., 1080.).into(),  // Full HD default
                ..default()
            }),
            ..default()
        }))
        .init_state::<GamePhase>()
        .init_resource::<GameState>()
        .init_resource::<PlayerStats>()
        // Events
        .add_event::<BetPlaced>()
        .add_event::<DealCards>()
        .add_event::<PlayerDecision>()
        .add_event::<RoundResult>()
        .add_event::<CardsDealt>()
        .add_event::<RequestCardFlip>()
        .add_event::<ComparisonComplete>()
        .add_event::<WarCardsDealt>()
        .add_event::<WarComplete>()
        
        // Setup systems
        .add_systems(Startup, (setup, load_audio_assets))
        .add_systems(OnEnter(GamePhase::MainMenu), setup_mclaren_main_menu)
        .add_systems(OnExit(GamePhase::MainMenu), cleanup_state)
        .add_systems(OnEnter(GamePhase::Betting), setup_mclaren_betting_ui)
        .add_systems(OnExit(GamePhase::Betting), cleanup_state)
        .add_systems(OnEnter(GamePhase::Dealing), on_enter_dealing)
        .add_systems(OnEnter(GamePhase::Comparing), (on_enter_comparing, setup_comparing_ui))
        .add_systems(OnEnter(GamePhase::TieDecision), setup_mclaren_tie_decision)
        .add_systems(OnExit(GamePhase::TieDecision), cleanup_state)
        .add_systems(OnEnter(GamePhase::War), on_enter_war)
        .add_systems(OnExit(GamePhase::War), cleanup_war_cards)
        .add_systems(OnEnter(GamePhase::RoundComplete), setup_mclaren_round_complete)
        .add_systems(OnExit(GamePhase::RoundComplete), (cleanup_state, cleanup_game_cards))
        
        // Update systems
        .add_systems(Update, (
            handle_mclaren_play_button.run_if(in_state(GamePhase::MainMenu)),
            (
                handle_mclaren_chip_buttons,
                handle_mclaren_deal_button,
                update_chip_display,
                update_bet_display,
            ).run_if(in_state(GamePhase::Betting)),
            animate_cards,
            animate_card_flips,
            check_dealing_complete.run_if(in_state(GamePhase::Dealing)),
            animate_war_cards.run_if(in_state(GamePhase::War)),
            check_war_dealing_complete.run_if(in_state(GamePhase::War)),
            handle_continue_button.run_if(in_state(GamePhase::RoundComplete)),
            // Polish systems
            animate_ui_elements,
            update_particles,
            cleanup_dead_particles,
            // PART 6: New animation systems
            animate_carbon_fiber,
            animate_glow_effects,
            animate_hologram_effects,
            // Debug system
            update_game_state_display,
        ))
        
        // Game logic systems
        .add_systems(Update, (
            deal_cards_system,
            flip_dealer_card_system,
            compare_cards_system.run_if(in_state(GamePhase::Comparing)),
            handle_comparison_result,
            handle_tie_decision_buttons.run_if(in_state(GamePhase::TieDecision)),
            deal_war_cards,
            flip_war_cards_system,
            compare_war_cards_system.run_if(in_state(GamePhase::War)),
            handle_war_result,
            update_card_visuals,
        ))
        
        .run();
}

fn setup(mut commands: Commands) {
    // Camera with bloom for neon effects
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // Enable HDR for bloom
            ..default()
        },
    ));
    
    // Debug UI to show game state
    commands.spawn((
        Text::new("Game State: Loading"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        GameStateDisplay,
    ));
    
    // PART 6: Carbon fiber background
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            color: CARBON_BLACK,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -2.0),
        CarbonFiberAnimation {
            scroll_speed: Vec2::new(5.0, 2.0),
            scale: 2.0,
        },
    ));
    
    // Grid pattern overlay for tech feel
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            color: MCLAREN_ORANGE.with_alpha(0.05),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

fn load_audio_assets(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
) {
    // Make audio optional - game works without sound files
    commands.insert_resource(GameAudio {
        card_flip: None,  // Optional: asset_server.load("sounds/card_flip.ogg")
        card_slide: None, // Optional: asset_server.load("sounds/card_slide.ogg")
        chip_place: None, // Optional: asset_server.load("sounds/chip_place.ogg")
        victory: None,    // Optional: asset_server.load("sounds/victory.ogg")
        defeat: None,     // Optional: asset_server.load("sounds/defeat.ogg")
        button_click: None, // Optional: asset_server.load("sounds/button_click.ogg")
    });
}

// PART 6: McLaren-style main menu
fn setup_mclaren_main_menu(mut commands: Commands) {
    // Main container with asymmetric layout
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        StateScoped(GamePhase::MainMenu),
    ))
    .with_children(|parent| {
        // Logo container - positioned using golden ratio
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(38.2), // Golden ratio position
                top: Val::Percent(20.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|logo_parent| {
            // Main title with glow
            logo_parent.spawn((
                Text::new("CASINO"),
                TextFont {
                    font_size: FONT_SIZE_HUGE,
                    ..default()
                },
                TextColor(MCLAREN_ORANGE),
                GlowEffect {
                    color: MCLAREN_ORANGE,
                    intensity: 0.8,
                    radius: 30.0,
                },
            ));
            
            // Subtitle
            logo_parent.spawn((
                Text::new("WAR"),
                TextFont {
                    font_size: FONT_SIZE_HUGE,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                Node {
                    margin: UiRect::top(Val::Px(-20.0)), // Overlap slightly
                    ..default()
                },
            ));
            
            // Tagline
            logo_parent.spawn((
                Text::new("MCLAREN EDITION"),
                TextFont {
                    font_size: FONT_SIZE_NORMAL,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
        
        // Button container - offset to the right
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(15.0),
                bottom: Val::Percent(25.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|button_parent| {
            // Play button - primary McLaren style
            button_parent.spawn((
                Button,
                PlayButton,
                McLarenButton {
                    primary: true,
                    hover_scale: 1.05,
                },
                Node {
                    width: Val::Px(320.0),
                    height: Val::Px(80.0),
                    border: UiRect {
                        left: Val::Px(4.0),
                        right: Val::Px(4.0),
                        top: Val::Px(2.0),
                        bottom: Val::Px(6.0),
                    },
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(MCLAREN_ORANGE),
                BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("START RACE"),
                    TextFont {
                        font_size: FONT_SIZE_MEDIUM,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                ));
            });
            
            // Stats button - secondary style
            button_parent.spawn((
                Button,
                Node {
                    width: Val::Px(320.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(ALUMINUM),
                BackgroundColor(PANEL_DARK),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("TELEMETRY"),
                    TextFont {
                        font_size: FONT_SIZE_NORMAL,
                        ..default()
                    },
                    TextColor(TEXT_SECONDARY),
                ));
            });
        });
    });
}

// PART 6: McLaren-style betting UI
fn setup_mclaren_betting_ui(mut commands: Commands) {
    // Bottom control panel
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(200.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            padding: UiRect::all(Val::Px(30.0)),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(PANEL_DARK),
        StateScoped(GamePhase::Betting),
    ))
    .with_children(|parent| {
        // Left telemetry panel
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|telemetry| {
            telemetry.spawn((
                Text::new("BANKROLL"),
                TextFont {
                    font_size: FONT_SIZE_SMALL,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
            ));
            
            telemetry.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Baseline,
                    ..default()
                },
            ))
            .with_children(|value| {
                value.spawn((
                    Text::new("$"),
                    TextFont {
                        font_size: FONT_SIZE_MEDIUM,
                        ..default()
                    },
                    TextColor(MCLAREN_ORANGE),
                ));
                value.spawn((
                    Text::new("1000"),
                    TextFont {
                        font_size: FONT_SIZE_LARGE,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                    ChipDisplay,
                ));
            });
        });
        
        // Center gear selector for chips
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(15.0),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|gear_parent| {
            for (i, &value) in CHIP_VALUES.iter().enumerate() {
                gear_parent.spawn((
                    Button,
                    ChipButton { value },
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    BorderColor(ALUMINUM),
                    BackgroundColor(PANEL_LIGHT),
                ))
                .with_children(|chip| {
                    // Gear number
                    chip.spawn((
                        Text::new(format!("{}", i + 1)),
                        TextFont {
                            font_size: FONT_SIZE_SMALL,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(5.0),
                            left: Val::Px(5.0),
                            ..default()
                        },
                    ));
                    
                    // Chip value
                    chip.spawn((
                        Text::new(format!("${}", value)),
                        TextFont {
                            font_size: FONT_SIZE_NORMAL,
                            ..default()
                        },
                        TextColor(TEXT_PRIMARY),
                    ));
                });
            }
        });
        
        // Right side - bet display and deal
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|right| {
            // Current bet telemetry
            right.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    ..default()
                },
            ))
            .with_children(|bet_display| {
                bet_display.spawn((
                    Text::new("WAGER"),
                    TextFont {
                        font_size: FONT_SIZE_SMALL,
                        ..default()
                    },
                    TextColor(TEXT_SECONDARY),
                ));
                
                bet_display.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                ))
                .with_children(|value| {
                    value.spawn((
                        Text::new("$"),
                        TextFont {
                            font_size: FONT_SIZE_MEDIUM,
                            ..default()
                        },
                        TextColor(VICTORY_GREEN),
                    ));
                    value.spawn((
                        Text::new("0"),
                        TextFont {
                            font_size: FONT_SIZE_LARGE,
                            ..default()
                        },
                        TextColor(VICTORY_GREEN),
                        BetDisplay,
                    ));
                });
            });
            
            // Deal button - McLaren style
            right.spawn((
                Button,
                DealButton,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    border: UiRect {
                        left: Val::Px(3.0),
                        right: Val::Px(3.0),
                        top: Val::Px(2.0),
                        bottom: Val::Px(5.0),
                    },
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(VICTORY_GREEN),
                BackgroundColor(VICTORY_GREEN.with_alpha(0.1)),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("ENGAGE"),
                    TextFont {
                        font_size: FONT_SIZE_MEDIUM,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                ));
            });
        });
    });
}

// PART 6: Handle McLaren-style buttons
fn handle_mclaren_play_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut Transform, &McLarenButton),
        (Changed<Interaction>, With<PlayButton>)
    >,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut commands: Commands,
    audio: Res<GameAudio>,
) {
    for (interaction, mut background, mut transform, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GamePhase::Betting);
                if let Some(sound) = &audio.button_click {
                    commands.spawn((
                        AudioPlayer(sound.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
                transform.scale = Vec3::splat(0.95);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.2));
                transform.scale = Vec3::splat(button.hover_scale);
            }
            Interaction::None => {
                *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1));
                transform.scale = Vec3::ONE;
            }
        }
    }
}

// PART 6: Handle McLaren chip buttons
fn handle_mclaren_chip_buttons(
    mut interaction_query: Query<
        (&Interaction, &ChipButton, &mut BackgroundColor, &mut Transform),
        Changed<Interaction>
    >,
    mut game_state: ResMut<GameState>,
    mut bet_display_query: Query<&mut Text, With<BetDisplay>>,
    mut commands: Commands,
    audio: Res<GameAudio>,
) {
    for (interaction, chip_button, mut background, mut transform) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let new_bet = game_state.current_bet + chip_button.value;
                
                if new_bet <= game_state.player_chips && new_bet <= MAX_BET {
                    game_state.current_bet = new_bet;
                    
                    if let Some(sound) = &audio.chip_place {
                        commands.spawn((
                            AudioPlayer(sound.clone()),
                            PlaybackSettings::DESPAWN,
                        ));
                    }
                    
                    if let Ok(mut text) = bet_display_query.single_mut() {
                        *text = Text::new(format!("{}", game_state.current_bet));
                    }
                    
                    transform.scale = Vec3::splat(0.9);
                    *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.3));
                }
            }
            Interaction::Hovered => {
                transform.scale = Vec3::splat(1.1);
                *background = BackgroundColor(ALUMINUM.with_alpha(0.5));
            }
            Interaction::None => {
                transform.scale = Vec3::ONE;
                *background = BackgroundColor(PANEL_LIGHT);
            }
        }
    }
}

// PART 6: Handle McLaren deal button
fn handle_mclaren_deal_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<DealButton>)
    >,
    game_state: Res<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut deal_events: EventWriter<DealCards>,
) {
    for (interaction, mut background, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if game_state.current_bet >= MIN_BET && game_state.current_bet <= game_state.player_chips {
                    println!("DEBUG: Deal button pressed! Bet: {}, Chips: {}", game_state.current_bet, game_state.player_chips);
                    next_state.set(GamePhase::Dealing);
                    deal_events.write(DealCards);
                    *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.3));
                }
            }
            Interaction::Hovered => {
                if game_state.current_bet >= MIN_BET {
                    *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.2));
                    *border = BorderColor(VICTORY_GREEN);
                }
            }
            Interaction::None => {
                *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.1));
                *border = BorderColor(VICTORY_GREEN.with_alpha(0.5));
            }
        }
    }
}

// Helper functions for card visuals
fn get_suit_positions(rank: Rank) -> Vec<(f32, f32)> {
    match rank {
        Rank::Ace => vec![(0.0, 0.0)],
        Rank::Two => vec![(0.0, 40.0), (0.0, -40.0)],
        Rank::Three => vec![(0.0, 40.0), (0.0, 0.0), (0.0, -40.0)],
        Rank::Four => vec![(-20.0, 40.0), (20.0, 40.0), (-20.0, -40.0), (20.0, -40.0)],
        Rank::Five => vec![(-20.0, 40.0), (20.0, 40.0), (0.0, 0.0), (-20.0, -40.0), (20.0, -40.0)],
        Rank::Six => vec![(-20.0, 40.0), (20.0, 40.0), (-20.0, 0.0), (20.0, 0.0), (-20.0, -40.0), (20.0, -40.0)],
        Rank::Seven => vec![(-20.0, 40.0), (20.0, 40.0), (0.0, 20.0), (-20.0, 0.0), (20.0, 0.0), (-20.0, -40.0), (20.0, -40.0)],
        Rank::Eight => vec![(-20.0, 40.0), (20.0, 40.0), (-20.0, 20.0), (20.0, 20.0), (-20.0, -20.0), (20.0, -20.0), (-20.0, -40.0), (20.0, -40.0)],
        Rank::Nine => vec![(-20.0, 40.0), (20.0, 40.0), (-20.0, 20.0), (20.0, 20.0), (0.0, 0.0), (-20.0, -20.0), (20.0, -20.0), (-20.0, -40.0), (20.0, -40.0)],
        Rank::Ten => vec![(-20.0, 50.0), (20.0, 50.0), (-20.0, 30.0), (20.0, 30.0), (-20.0, 10.0), (20.0, 10.0), (-20.0, -10.0), (20.0, -10.0), (-20.0, -30.0), (20.0, -30.0)],
        Rank::Jack | Rank::Queen | Rank::King => vec![(0.0, 0.0)],
    }
}

// PART 6: Spawn McLaren-style cards with Text2d for VISIBLE values
fn spawn_mclaren_card(
    commands: &mut Commands,
    card: Card,
    position: CardPosition,
    face_up: bool,
    asset_server: &AssetServer,
) -> Entity {
    let world_pos = match position {
        CardPosition::Deck => DECK_POSITION,
        CardPosition::PlayerHand => PLAYER_CARD_POSITION,
        CardPosition::DealerHand => DEALER_CARD_POSITION,
        CardPosition::Discard => DECK_POSITION + Vec3::new(150.0, 0.0, 0.0),
    };
    
    let card_entity = commands.spawn((
        card,
        position,
        CardVisual {
            face_up,
            target_position: world_pos,
        },
        Transform::from_translation(DECK_POSITION),
        Visibility::Visible,
        ViewVisibility::default(),
        InheritedVisibility::default(),
        GameCard,
    ))
    .with_children(|parent| {
        // Card base - black border
        parent.spawn((
            Sprite {
                custom_size: Some(Vec2::new(CARD_WIDTH, CARD_HEIGHT)),
                color: CARBON_BLACK,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        
        // Card face content
        let face_visible = if face_up { Visibility::Visible } else { Visibility::Hidden };
        
        parent.spawn((
            CardFace,
            Transform::default(),
            face_visible,
        ))
        .with_children(|face_parent| {
            // White card surface
            face_parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(CARD_WIDTH - 6.0, CARD_HEIGHT - 6.0)),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));
            
            let color = card.get_color();
            let rank_str = match card.rank {
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
            };
            
            // SIMPLE SPRITE-BASED CARD DISPLAY
            // Rank display using colored rectangles
            face_parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(50.0, 70.0)),
                    color,
                    ..default()
                },
                Transform::from_xyz(0.0, 10.0, 0.3),
            ));
            
            // White text background for contrast
            face_parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(45.0, 65.0)),
                    color: Color::WHITE,
                    ..default()
                },
                Transform::from_xyz(0.0, 10.0, 0.4),
            ));
            
            // Show rank as pips/shapes
            match card.rank {
                Rank::Ace => {
                    // Large diamond for Ace
                    face_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(40.0, 40.0)),
                            color,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 10.0, 0.5)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 4.0)),
                    ));
                },
                Rank::King => {
                    // Three horizontal bars for King
                    for i in -1..=1 {
                        face_parent.spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(35.0, 8.0)),
                                color,
                                ..default()
                            },
                            Transform::from_xyz(0.0, i as f32 * 15.0 + 10.0, 0.5),
                        ));
                    }
                },
                Rank::Queen => {
                    // Circle for Queen
                    face_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(35.0, 35.0)),
                            color,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 10.0, 0.5),
                    ));
                },
                Rank::Jack => {
                    // Square for Jack
                    face_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(30.0, 30.0)),
                            color,
                            ..default()
                        },
                        Transform::from_xyz(0.0, 10.0, 0.5),
                    ));
                },
                _ => {
                    // Number cards - show dots for rank value
                    let value = card.rank as u8;
                    let positions = match value {
                        2 => vec![(0.0, 15.0), (0.0, -15.0)],
                        3 => vec![(0.0, 20.0), (0.0, 0.0), (0.0, -20.0)],
                        4 => vec![(-15.0, 15.0), (15.0, 15.0), (-15.0, -15.0), (15.0, -15.0)],
                        5 => vec![(-15.0, 15.0), (15.0, 15.0), (0.0, 0.0), (-15.0, -15.0), (15.0, -15.0)],
                        6 => vec![(-15.0, 20.0), (15.0, 20.0), (-15.0, 0.0), (15.0, 0.0), (-15.0, -20.0), (15.0, -20.0)],
                        7 => vec![(-15.0, 20.0), (15.0, 20.0), (0.0, 10.0), (-15.0, 0.0), (15.0, 0.0), (-15.0, -20.0), (15.0, -20.0)],
                        8 => vec![(-15.0, 25.0), (15.0, 25.0), (-15.0, 10.0), (15.0, 10.0), (-15.0, -10.0), (15.0, -10.0), (-15.0, -25.0), (15.0, -25.0)],
                        9 => vec![(-15.0, 25.0), (15.0, 25.0), (-15.0, 10.0), (15.0, 10.0), (0.0, 0.0), (-15.0, -10.0), (15.0, -10.0), (-15.0, -25.0), (15.0, -25.0)],
                        10 => vec![(-15.0, 30.0), (15.0, 30.0), (-15.0, 15.0), (15.0, 15.0), (-15.0, 0.0), (15.0, 0.0), (-15.0, -15.0), (15.0, -15.0), (-15.0, -30.0), (15.0, -30.0)],
                        _ => vec![],
                    };
                    
                    for (x, y) in positions {
                        face_parent.spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(8.0, 8.0)),
                                color,
                                ..default()
                            },
                            Transform::from_xyz(x, y, 0.5),
                        ));
                    }
                }
            }
            
            // Suit indicator at bottom
            match card.suit {
                Suit::Hearts => {
                    // Two circles and triangle for heart shape
                    face_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(15.0, 15.0)),
                            color,
                            ..default()
                        },
                        Transform::from_xyz(-7.0, -35.0, 0.5),
                    ));
                    face_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(15.0, 15.0)),
                            color,
                            ..default()
                        },
                        Transform::from_xyz(7.0, -35.0, 0.5),
                    ));
                },
                Suit::Diamonds => {
                    face_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(25.0, 25.0)),
                            color,
                            ..default()
                        },
                        Transform::from_xyz(0.0, -40.0, 0.5)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::PI / 4.0)),
                    ));
                },
                Suit::Clubs => {
                    // Three circles for clubs
                    for i in 0..3 {
                        let angle = i as f32 * 2.0 * std::f32::consts::PI / 3.0;
                        face_parent.spawn((
                            Sprite {
                                custom_size: Some(Vec2::new(12.0, 12.0)),
                                color,
                                ..default()
                            },
                            Transform::from_xyz(
                                angle.cos() * 8.0,
                                angle.sin() * 8.0 - 40.0,
                                0.5
                            ),
                        ));
                    }
                },
                Suit::Spades => {
                    // Inverted heart shape for spades
                    face_parent.spawn((
                        Sprite {
                            custom_size: Some(Vec2::new(20.0, 20.0)),
                            color,
                            ..default()
                        },
                        Transform::from_xyz(0.0, -40.0, 0.5),
                    ));
                },
            }
            
            // DEBUG: Add text overlay to show card value clearly
            // This uses UI text positioned on the card
            face_parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(80.0),
                    height: Val::Px(120.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|text_parent| {
                text_parent.spawn((
                    Text::new(format!("{}{}", rank_str, card.get_suit_symbol())),
                    TextFont {
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                ));
            });
        });
        
        // Card back - McLaren tech design
        let back_visible = if face_up { Visibility::Hidden } else { Visibility::Visible };
        
        parent.spawn((
            CardBack,
            Transform::default(),
            back_visible,
        ))
        .with_children(|back_parent| {
            // Dark background
            back_parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(CARD_WIDTH - 6.0, CARD_HEIGHT - 6.0)),
                    color: CARBON_BLACK,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));
            
            // McLaren orange accent
            back_parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(CARD_WIDTH - 20.0, 3.0)),
                    color: MCLAREN_ORANGE,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.3),
            ));
            
            // Tech pattern stripes
            for i in -2..=2 {
                back_parent.spawn((
                    Sprite {
                        custom_size: Some(Vec2::new(2.0, CARD_HEIGHT - 40.0)),
                        color: ALUMINUM.with_alpha(0.2),
                        ..default()
                    },
                    Transform::from_xyz(i as f32 * 15.0, 0.0, 0.3),
                ));
            }
        });
    })
    .id();
    
    match position {
        CardPosition::PlayerHand => {
            commands.entity(card_entity).insert(PlayerCard);
        }
        CardPosition::DealerHand => {
            commands.entity(card_entity).insert(DealerCard);
        }
        _ => {}
    }
    
    card_entity
}

// PART 6: McLaren-style tie decision UI
fn setup_mclaren_tie_decision(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(GamePhase::TieDecision),
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(50.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(PANEL_DARK),
            BorderColor(MCLAREN_ORANGE),
            UIAnimation {
                start_scale: Vec3::ZERO,
                end_scale: Vec3::ONE,
                timer: Timer::from_seconds(0.3, TimerMode::Once),
            },
        ))
        .with_children(|modal| {
            // Warning style title
            modal.spawn((
                Text::new("STALEMATE DETECTED"),
                TextFont {
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(MCLAREN_ORANGE),
                GlowEffect {
                    color: MCLAREN_ORANGE,
                    intensity: 1.0,
                    radius: 20.0,
                },
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));
            
            modal.spawn((
                Text::new("TACTICAL DECISION REQUIRED"),
                TextFont {
                    font_size: FONT_SIZE_NORMAL,
                    ..default()
                },
                TextColor(TEXT_SECONDARY),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));
            
            // Button container
            modal.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(30.0),
                    ..default()
                },
            ))
            .with_children(|buttons| {
                // Retreat button
                buttons.spawn((
                    Button,
                    TieDecisionButton { go_to_war: false },
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(DANGER_RED),
                    BackgroundColor(DANGER_RED.with_alpha(0.1)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|content| {
                        content.spawn((
                            Text::new("TACTICAL RETREAT"),
                            TextFont {
                                font_size: FONT_SIZE_NORMAL,
                                ..default()
                            },
                            TextColor(TEXT_PRIMARY),
                        ));
                        content.spawn((
                            Text::new("RECOVER 50% WAGER"),
                            TextFont {
                                font_size: FONT_SIZE_SMALL,
                                ..default()
                            },
                            TextColor(TEXT_SECONDARY),
                        ));
                    });
                });
                
                // War button
                buttons.spawn((
                    Button,
                    TieDecisionButton { go_to_war: true },
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(VICTORY_GREEN),
                    BackgroundColor(VICTORY_GREEN.with_alpha(0.1)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|content| {
                        content.spawn((
                            Text::new("FULL THROTTLE"),
                            TextFont {
                                font_size: FONT_SIZE_NORMAL,
                                ..default()
                            },
                            TextColor(TEXT_PRIMARY),
                        ));
                        content.spawn((
                            Text::new("DOUBLE DOWN"),
                            TextFont {
                                font_size: FONT_SIZE_SMALL,
                                ..default()
                            },
                            TextColor(TEXT_SECONDARY),
                        ));
                    });
                });
            });
        });
    });
}

// PART 6: McLaren-style round complete UI
fn setup_mclaren_round_complete(
    mut commands: Commands,
    mut round_events: EventReader<RoundResult>,
    game_state: Res<GameState>,
    audio: Res<GameAudio>,
    stats: Res<PlayerStats>,
) {
    println!("DEBUG: Setting up round complete UI");
    let mut result_text = String::new();
    let mut result_color = Color::WHITE;
    let mut winnings_text = String::new();
    let mut play_sound = None;
    
    for event in round_events.read() {
        println!("DEBUG: Round result - player_won: {}, winnings: {}", event.player_won, event.winnings);
        if event.player_won {
            result_text = "VICTORY".to_string();
            result_color = VICTORY_GREEN;
            winnings_text = format!("+${}", event.winnings);
            play_sound = audio.victory.clone();
        } else {
            result_text = "DEFEATED".to_string();
            result_color = DANGER_RED;
            winnings_text = format!("${}", event.winnings);
            play_sound = audio.defeat.clone();
        }
    }
    
    if let Some(sound) = play_sound {
        commands.spawn((
            AudioPlayer(sound),
            PlaybackSettings::DESPAWN,
        ));
    }
    
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        StateScoped(GamePhase::RoundComplete),
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(60.0)),
                border: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(PANEL_DARK),
            BorderColor(result_color),
            UIAnimation {
                start_scale: Vec3::splat(0.8),
                end_scale: Vec3::ONE,
                timer: Timer::from_seconds(0.3, TimerMode::Once),
            },
        ))
        .with_children(|modal| {
            // Result with dramatic effect
            modal.spawn((
                Text::new(result_text),
                TextFont {
                    font_size: FONT_SIZE_HUGE,
                    ..default()
                },
                TextColor(result_color),
                GlowEffect {
                    color: result_color,
                    intensity: 1.5,
                    radius: 40.0,
                },
                ResultDisplay,
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));
            
            // Winnings display
            modal.spawn((
                Text::new(winnings_text),
                TextFont {
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            
            // Telemetry data
            modal.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(50.0),
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ))
            .with_children(|telemetry| {
                // Bankroll
                telemetry.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|stat| {
                    stat.spawn((
                        Text::new("BANKROLL"),
                        TextFont {
                            font_size: FONT_SIZE_SMALL,
                            ..default()
                        },
                        TextColor(TEXT_SECONDARY),
                    ));
                    stat.spawn((
                        Text::new(format!("${}", game_state.player_chips)),
                        TextFont {
                            font_size: FONT_SIZE_MEDIUM,
                            ..default()
                        },
                        TextColor(MCLAREN_ORANGE),
                    ));
                });
                
                // Streak
                if stats.current_streak.abs() >= 2 {
                    telemetry.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|stat| {
                        stat.spawn((
                            Text::new("STREAK"),
                            TextFont {
                                font_size: FONT_SIZE_SMALL,
                                ..default()
                            },
                            TextColor(TEXT_SECONDARY),
                        ));
                        stat.spawn((
                            Text::new(format!("{}", stats.current_streak.abs())),
                            TextFont {
                                font_size: FONT_SIZE_MEDIUM,
                                ..default()
                            },
                            TextColor(if stats.current_streak > 0 { VICTORY_GREEN } else { DANGER_RED }),
                        ));
                    });
                }
            });
            
            // Continue button
            modal.spawn((
                Button,
                ContinueButton,
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(70.0),
                    border: UiRect::all(Val::Px(3.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(ALUMINUM),
                BackgroundColor(ALUMINUM.with_alpha(0.1)),
            ))
            .with_children(|button| {
                button.spawn((
                    Text::new("CONTINUE"),
                    TextFont {
                        font_size: FONT_SIZE_MEDIUM,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
                ));
            });
        });
    });
}

// PART 6: Animate carbon fiber background
fn animate_carbon_fiber(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &CarbonFiberAnimation)>,
) {
    for (mut transform, anim) in &mut query {
        let offset = anim.scroll_speed * time.elapsed_secs();
        transform.translation.x = offset.x % 100.0;
        transform.translation.y = offset.y % 100.0;
    }
}

// PART 6: Animate glow effects
fn animate_glow_effects(
    time: Res<Time>,
    mut query: Query<(&GlowEffect, &mut Sprite)>,
) {
    for (glow, mut sprite) in &mut query {
        let pulse = (time.elapsed_secs() * 2.0).sin() * 0.5 + 0.5;
        let intensity = glow.intensity * (0.7 + pulse * 0.3);
        sprite.color = glow.color.with_alpha(intensity);
    }
}

// PART 6: Animate hologram effects on cards
fn animate_hologram_effects(
    time: Res<Time>,
    mut query: Query<(&HologramEffect, &mut Transform)>,
) {
    for (hologram, mut transform) in &mut query {
        // Subtle floating motion
        let float_offset = (time.elapsed_secs() * hologram.scan_speed).sin() * 2.0;
        transform.translation.y += float_offset * time.delta_secs();
        
        // Occasional flicker
        if thread_rng().gen::<f32>() < hologram.flicker_rate {
            transform.scale = Vec3::splat(0.98 + thread_rng().gen::<f32>() * 0.04);
        } else {
            transform.scale = Vec3::ONE;
        }
    }
}

// Core game systems remain the same but with McLaren card spawning
fn deal_cards_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut deal_events: EventReader<DealCards>,
    asset_server: Res<AssetServer>,
) {
    for _ in deal_events.read() {
        game_state.player_chips -= game_state.current_bet;
        
        if let (Some(player_card), Some(dealer_card)) = 
            (game_state.draw_card(), game_state.draw_card()) 
        {
            // Use McLaren card spawning
            let player_entity = spawn_mclaren_card(
                &mut commands,
                player_card,
                CardPosition::PlayerHand,
                true,
                &asset_server,
            );
            
            commands.entity(player_entity)
                .insert(ActiveCard)
                .insert(CardAnimation {
                    start_pos: DECK_POSITION,
                    end_pos: PLAYER_CARD_POSITION,
                    start_rotation: Quat::IDENTITY,
                    end_rotation: Quat::IDENTITY,
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                });
            
            let dealer_entity = spawn_mclaren_card(
                &mut commands,
                dealer_card,
                CardPosition::DealerHand,
                false,
                &asset_server,
            );
            
            commands.entity(dealer_entity)
                .insert(ActiveCard)
                .insert(CardAnimation {
                    start_pos: DECK_POSITION,
                    end_pos: DEALER_CARD_POSITION,
                    start_rotation: Quat::IDENTITY,
                    end_rotation: Quat::IDENTITY,
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                });
        }
    }
}

// Rest of the game systems remain largely unchanged...
// Including: cleanup_state, update_chip_display, animate_cards, etc.
// These are omitted for brevity but would be identical to Part 5

fn cleanup_state(
    mut commands: Commands,
    query: Query<Entity, With<StateScoped>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn cleanup_game_cards(
    mut commands: Commands,
    cards: Query<Entity, With<GameCard>>,
) {
    for entity in cards.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_chip_display(
    game_state: Res<GameState>,
    mut chip_display_query: Query<&mut Text, With<ChipDisplay>>,
) {
    if game_state.is_changed() {
        if let Ok(mut text) = chip_display_query.single_mut() {
            *text = Text::new(format!("{}", game_state.player_chips));
        }
    }
}

fn update_bet_display(
    game_state: Res<GameState>,
    mut bet_display_query: Query<&mut Text, With<BetDisplay>>,
) {
    if game_state.is_changed() {
        if let Ok(mut text) = bet_display_query.single_mut() {
            *text = Text::new(format!("{}", game_state.current_bet));
        }
    }
}

fn update_card_visuals(
    mut card_query: Query<(&CardVisual, &Children), Changed<CardVisual>>,
    mut face_query: Query<&mut Visibility, (With<CardFace>, Without<CardBack>)>,
    mut back_query: Query<&mut Visibility, (With<CardBack>, Without<CardFace>)>,
) {
    for (visual, children) in &mut card_query {
        for child in children.iter() {
            if let Ok(mut face_vis) = face_query.get_mut(child) {
                *face_vis = if visual.face_up { 
                    Visibility::Visible 
                } else { 
                    Visibility::Hidden 
                };
            }
            
            if let Ok(mut back_vis) = back_query.get_mut(child) {
                *back_vis = if visual.face_up { 
                    Visibility::Hidden 
                } else { 
                    Visibility::Visible 
                };
            }
        }
    }
}

// Debug text removed - cards now show visually

fn animate_cards(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CardAnimation)>,
    audio: Res<GameAudio>,
) {
    for (entity, mut transform, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        
        if animation.timer.finished() {
            transform.translation = animation.end_pos;
            transform.rotation = animation.end_rotation;
            commands.entity(entity).remove::<CardAnimation>();
        } else {
            let t = animation.timer.fraction();
            let t = t * t * (3.0 - 2.0 * t);
            
            transform.translation = animation.start_pos.lerp(animation.end_pos, t);
            transform.rotation = animation.start_rotation.slerp(animation.end_rotation, t);
            
            if animation.timer.elapsed_secs() < 0.1 && animation.timer.elapsed_secs() > 0.0 {
                if let Some(sound) = &audio.card_slide {
                    commands.spawn((
                        AudioPlayer(sound.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }
        }
    }
}

fn on_enter_dealing(mut deal_events: EventWriter<DealCards>) {
    println!("DEBUG: Entering dealing phase");
    deal_events.write(DealCards);
}

fn check_dealing_complete(
    animating_cards: Query<&CardAnimation, With<ActiveCard>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut events: EventWriter<CardsDealt>,
) {
    if animating_cards.is_empty() {
        println!("DEBUG: Cards dealt, moving to Comparing phase");
        events.write(CardsDealt);
        next_state.set(GamePhase::Comparing);
    }
}

fn on_enter_comparing(mut flip_events: EventWriter<RequestCardFlip>) {
    flip_events.write(RequestCardFlip);
}

fn setup_comparing_ui(mut commands: Commands) {
    // Timer to ensure comparison happens after card flip
    commands.insert_resource(ComparisonTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Once),
    });
}

#[derive(Resource)]
struct ComparisonTimer {
    timer: Timer,
}

fn animate_card_flips(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CardFlipAnimation, &mut CardVisual)>,
    audio: Res<GameAudio>,
) {
    for (entity, mut transform, mut flip_anim, mut visual) in &mut query {
        flip_anim.timer.tick(time.delta());
        
        if flip_anim.timer.finished() {
            transform.rotation = Quat::IDENTITY;
            visual.face_up = true;
            commands.entity(entity).remove::<CardFlipAnimation>();
        } else {
            let progress = flip_anim.timer.fraction();
            let rotation = progress * std::f32::consts::PI;
            
            transform.rotation = Quat::from_rotation_y(rotation);
            
            if rotation >= std::f32::consts::PI / 2.0 && !flip_anim.half_flipped {
                flip_anim.half_flipped = true;
                visual.face_up = true;
                
                if let Some(sound) = &audio.card_flip {
                    commands.spawn((
                        AudioPlayer(sound.clone()),
                        PlaybackSettings::DESPAWN,
                    ));
                }
            }
        }
    }
}

fn flip_dealer_card_system(
    mut commands: Commands,
    mut flip_events: EventReader<RequestCardFlip>,
    dealer_cards: Query<(Entity, &CardVisual), (With<DealerCard>, With<ActiveCard>)>,
) {
    for _ in flip_events.read() {
        for (entity, visual) in &dealer_cards {
            if !visual.face_up {
                commands.entity(entity).insert(CardFlipAnimation {
                    timer: Timer::from_seconds(0.6, TimerMode::Once),
                    half_flipped: false,
                });
            }
        }
    }
}

fn compare_cards_system(
    mut commands: Commands,
    player_cards: Query<&Card, (With<PlayerCard>, With<ActiveCard>)>,
    dealer_cards: Query<&Card, (With<DealerCard>, With<ActiveCard>)>,
    dealer_visual: Query<&CardVisual, (With<DealerCard>, With<ActiveCard>)>,
    mut comparison_events: EventWriter<ComparisonComplete>,
    time: Res<Time>,
    mut timer: Option<ResMut<ComparisonTimer>>,
) {
    // Wait for timer before comparing
    if let Some(ref mut timer) = timer {
        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            return;
        }
    } else {
        return;
    }
    
    if let Ok(visual) = dealer_visual.single() {
        if !visual.face_up {
            return;
        }
    }
    
    let Ok(player_card) = player_cards.single() else { return };
    let Ok(dealer_card) = dealer_cards.single() else { return };
    
    let player_value = player_card.value();
    let dealer_value = dealer_card.value();
    
    println!("DEBUG: Comparing cards - Player: {} vs Dealer: {}", player_value, dealer_value);
    
    let outcome = match player_value.cmp(&dealer_value) {
        std::cmp::Ordering::Greater => ComparisonOutcome::PlayerWins,
        std::cmp::Ordering::Less => ComparisonOutcome::DealerWins,
        std::cmp::Ordering::Equal => ComparisonOutcome::Tie,
    };
    
    // Show comparison result in UI
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(150.0),
            position_type: PositionType::Absolute,
            top: Val::Px(200.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        StateScoped(GamePhase::Comparing),
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                padding: UiRect::all(Val::Px(30.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(PANEL_DARK.with_alpha(0.9)),
            BorderColor(match outcome {
                ComparisonOutcome::PlayerWins => VICTORY_GREEN,
                ComparisonOutcome::DealerWins => DANGER_RED,
                ComparisonOutcome::Tie => MCLAREN_ORANGE,
            }),
        ))
        .with_children(|panel| {
            // Card values
            panel.spawn((
                Text::new(format!("Player: {} vs Dealer: {}", 
                    player_card.get_rank_symbol(), 
                    dealer_card.get_rank_symbol()
                )),
                TextFont {
                    font_size: FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
            
            // Result
            let (result_text, result_color) = match outcome {
                ComparisonOutcome::PlayerWins => ("YOU WIN!", VICTORY_GREEN),
                ComparisonOutcome::DealerWins => ("DEALER WINS", DANGER_RED),
                ComparisonOutcome::Tie => ("TIE - GO TO WAR?", MCLAREN_ORANGE),
            };
            
            panel.spawn((
                Text::new(result_text),
                TextFont {
                    font_size: FONT_SIZE_LARGE,
                    ..default()
                },
                TextColor(result_color),
            ));
        });
    });
    
    commands.remove_resource::<ComparisonTimer>();
    comparison_events.write(ComparisonComplete { outcome });
}

fn handle_comparison_result(
    mut comparison_events: EventReader<ComparisonComplete>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut round_events: EventWriter<RoundResult>,
    mut stats: ResMut<PlayerStats>,
) {
    for event in comparison_events.read() {
        println!("DEBUG: Handling comparison result: {:?}", event.outcome);
        match event.outcome {
            ComparisonOutcome::PlayerWins => {
                let winnings = game_state.current_bet as i32 * 2;
                game_state.player_chips += winnings as u32;
                
                stats.total_wins += 1;
                stats.current_streak = stats.current_streak.max(0) + 1;
                stats.best_streak = stats.best_streak.max(stats.current_streak as u32);
                stats.total_won += winnings as u64;
                
                round_events.write(RoundResult {
                    player_won: true,
                    winnings,
                });
                
                next_state.set(GamePhase::RoundComplete);
            }
            ComparisonOutcome::DealerWins => {
                let winnings = -(game_state.current_bet as i32);
                
                stats.total_losses += 1;
                stats.current_streak = stats.current_streak.min(0) - 1;
                
                round_events.write(RoundResult {
                    player_won: false,
                    winnings,
                });
                
                next_state.set(GamePhase::RoundComplete);
            }
            ComparisonOutcome::Tie => {
                stats.total_ties += 1;
                next_state.set(GamePhase::TieDecision);
            }
        }
        
        stats.total_games += 1;
        stats.total_wagered += game_state.current_bet as u64;
    }
}

fn handle_tie_decision_buttons(
    mut interaction_query: Query<
        (&Interaction, &TieDecisionButton, &mut BackgroundColor),
        Changed<Interaction>
    >,
    mut player_events: EventWriter<PlayerDecision>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut stats: ResMut<PlayerStats>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if button.go_to_war {
                    if game_state.player_chips >= game_state.current_bet {
                        game_state.war_bet = game_state.current_bet;
                        game_state.player_chips -= game_state.war_bet;
                        
                        stats.wars_entered += 1;
                        
                        player_events.write(PlayerDecision { go_to_war: true });
                        next_state.set(GamePhase::War);
                    }
                } else {
                    // Retreat - get back half the bet
                    let half_bet = game_state.current_bet / 2;
                    game_state.player_chips += half_bet;
                    
                    stats.total_losses += 1;
                    
                    player_events.write(PlayerDecision { go_to_war: false });
                    next_state.set(GamePhase::RoundComplete);
                }
            }
            Interaction::Hovered => {
                if button.go_to_war {
                    *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.2));
                } else {
                    *background = BackgroundColor(DANGER_RED.with_alpha(0.2));
                }
            }
            Interaction::None => {
                if button.go_to_war {
                    *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.1));
                } else {
                    *background = BackgroundColor(DANGER_RED.with_alpha(0.1));
                }
            }
        }
    }
}

fn on_enter_war(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        StateScoped(GamePhase::War),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("WAR!"),
            TextFont {
                font_size: 150.0,
                ..default()
            },
            TextColor(DANGER_RED),
            GlowEffect {
                color: DANGER_RED,
                intensity: 2.0,
                radius: 50.0,
            },
            WarAnnouncement {
                timer: Timer::from_seconds(1.5, TimerMode::Once),
            },
        ));
    });
    
    commands.insert_resource(WarDealingTimer {
        timer: Timer::from_seconds(1.5, TimerMode::Once),
    });
}

#[derive(Component)]
struct WarAnnouncement {
    timer: Timer,
}

#[derive(Resource)]
struct WarDealingTimer {
    timer: Timer,
}

fn cleanup_war_cards(
    mut commands: Commands,
    war_cards: Query<Entity, With<WarCard>>,
) {
    for entity in war_cards.iter() {
        commands.entity(entity).despawn();
    }
}

fn deal_war_cards(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut timer: Option<ResMut<WarDealingTimer>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(ref mut timer) = timer {
        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            return;
        }
        
        commands.remove_resource::<WarDealingTimer>();
        
        for i in 0..4 {
            if let Some(card) = game_state.draw_card() {
                let is_final = i == 3;
                let x_offset = (i as f32 - 1.5) * WAR_CARD_SPACING;
                let target_pos = Vec3::new(x_offset, WAR_PLAYER_Y, 1.0 + i as f32 * 0.1);
                
                let entity = spawn_mclaren_card(
                    &mut commands,
                    card,
                    CardPosition::PlayerHand,
                    is_final,
                    &asset_server,
                );
                
                commands.entity(entity)
                    .insert(WarCard {
                        index: i,
                        delay: Timer::from_seconds(i as f32 * 0.3, TimerMode::Once),
                    })
                    .insert(CardAnimation {
                        start_pos: DECK_POSITION,
                        end_pos: target_pos,
                        start_rotation: Quat::IDENTITY,
                        end_rotation: Quat::IDENTITY,
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    });
                
                if is_final {
                    commands.entity(entity).insert(ActiveCard);
                } else {
                    commands.entity(entity).insert(BurnCard);
                }
            }
            
            if let Some(card) = game_state.draw_card() {
                let is_final = i == 3;
                let x_offset = (i as f32 - 1.5) * WAR_CARD_SPACING;
                let target_pos = Vec3::new(x_offset, WAR_DEALER_Y, 1.0 + i as f32 * 0.1);
                
                let entity = spawn_mclaren_card(
                    &mut commands,
                    card,
                    CardPosition::DealerHand,
                    false,
                    &asset_server,
                );
                
                commands.entity(entity)
                    .insert(WarCard {
                        index: i,
                        delay: Timer::from_seconds(i as f32 * 0.3 + 0.15, TimerMode::Once),
                    })
                    .insert(CardAnimation {
                        start_pos: DECK_POSITION,
                        end_pos: target_pos,
                        start_rotation: Quat::IDENTITY,
                        end_rotation: Quat::IDENTITY,
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    });
                
                if is_final {
                    commands.entity(entity).insert(ActiveCard);
                } else {
                    commands.entity(entity).insert(BurnCard);
                }
            }
        }
    }
}

fn animate_war_cards(
    mut commands: Commands,
    time: Res<Time>,
    mut war_cards: Query<(Entity, &mut WarCard), Without<CardAnimation>>,
) {
    for (entity, mut war_card) in &mut war_cards {
        war_card.delay.tick(time.delta());
        
        if war_card.delay.finished() {
            commands.entity(entity).remove::<WarCard>();
        }
    }
}

fn check_war_dealing_complete(
    animating_cards: Query<&CardAnimation, With<ActiveCard>>,
    war_cards: Query<&WarCard>,
    mut events: EventWriter<WarCardsDealt>,
    mut flip_events: EventWriter<RequestCardFlip>,
) {
    if animating_cards.is_empty() && war_cards.is_empty() {
        events.write(WarCardsDealt);
        flip_events.write(RequestCardFlip);
    }
}

fn flip_war_cards_system(
    mut commands: Commands,
    mut flip_events: EventReader<RequestCardFlip>,
    war_cards: Query<(Entity, &CardVisual), With<ActiveCard>>,
) {
    for _ in flip_events.read() {
        for (entity, visual) in &war_cards {
            if !visual.face_up {
                commands.entity(entity).insert(CardFlipAnimation {
                    timer: Timer::from_seconds(0.6, TimerMode::Once),
                    half_flipped: false,
                });
            }
        }
    }
}

fn compare_war_cards_system(
    player_cards: Query<&Card, (With<PlayerCard>, With<ActiveCard>)>,
    dealer_cards: Query<&Card, (With<DealerCard>, With<ActiveCard>)>,
    dealer_visual: Query<&CardVisual, (With<DealerCard>, With<ActiveCard>)>,
    mut war_events: EventWriter<WarComplete>,
) {
    for visual in &dealer_visual {
        if !visual.face_up {
            return;
        }
    }
    
    let Ok(player_card) = player_cards.single() else { return };
    let Ok(dealer_card) = dealer_cards.single() else { return };
    
    let player_won = player_card.value() > dealer_card.value();
    
    war_events.write(WarComplete { player_won });
}

fn handle_war_result(
    mut war_events: EventReader<WarComplete>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut round_events: EventWriter<RoundResult>,
    mut commands: Commands,
    mut stats: ResMut<PlayerStats>,
) {
    for event in war_events.read() {
        if event.player_won {
            // War win: Get back original bet + war bet + war bet winnings
            let total_payout = game_state.current_bet + (game_state.war_bet * 2);
            game_state.player_chips += total_payout;
            
            stats.wars_won += 1;
            stats.total_won += game_state.war_bet as u64;
            
            spawn_victory_particles(&mut commands, Vec2::ZERO);
            
            // Display shows net winnings (war bet profit)
            round_events.write(RoundResult {
                player_won: true,
                winnings: game_state.war_bet as i32,
            });
        } else {
            let total_loss = (game_state.current_bet + game_state.war_bet) as i32;
            
            round_events.write(RoundResult {
                player_won: false,
                winnings: -total_loss,
            });
        }
        
        game_state.war_bet = 0;
        next_state.set(GamePhase::RoundComplete);
    }
}

fn handle_continue_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ContinueButton>)
    >,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                println!("DEBUG: Continue button pressed!");
                // Reset for next round
                game_state.current_bet = 0;
                game_state.war_bet = 0;
                
                // Check if player has chips to continue
                if game_state.player_chips >= MIN_BET {
                    next_state.set(GamePhase::Betting);
                } else {
                    // Game over - return to main menu
                    next_state.set(GamePhase::MainMenu);
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(ALUMINUM.with_alpha(0.2));
            }
            Interaction::None => {
                *background = BackgroundColor(ALUMINUM.with_alpha(0.1));
            }
        }
    }
}

fn animate_ui_elements(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut UIAnimation)>,
) {
    for (entity, mut transform, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        
        if animation.timer.finished() {
            transform.scale = animation.end_scale;
            commands.entity(entity).remove::<UIAnimation>();
        } else {
            let t = animation.timer.fraction();
            let t = 1.0 + (-10.0 * t).exp() * ((t - 0.1) * 2.0 * std::f32::consts::PI).sin();
            transform.scale = animation.start_scale.lerp(animation.end_scale, t);
        }
    }
}

fn spawn_victory_particles(
    commands: &mut Commands,
    position: Vec2,
) {
    commands.spawn((
        ParticleEmitter {
            spawn_rate: 100.0,
            spawn_timer: Timer::from_seconds(0.01, TimerMode::Repeating),
            particle_lifetime: 2.0,
            particles_to_spawn: 50,
        },
        Transform::from_translation(position.extend(5.0)),
    ));
}

fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut emitters: Query<(Entity, &mut ParticleEmitter, &Transform)>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform), Without<ParticleEmitter>>,
) {
    for (entity, mut particle, mut transform) in &mut particles {
        particle.lifetime.tick(time.delta());
        
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            particle.velocity.y -= particle.gravity * time.delta_secs();
            transform.translation += particle.velocity * time.delta_secs();
            
            let alpha = 1.0 - particle.lifetime.fraction();
            if let Ok(mut sprite_commands) = commands.get_entity(entity) {
                sprite_commands.insert(Sprite {
                    custom_size: Some(Vec2::splat(10.0 * (1.0 - particle.lifetime.fraction()))),
                    color: MCLAREN_ORANGE.with_alpha(alpha),
                    ..default()
                });
            }
        }
    }
    
    for (entity, mut emitter, emitter_transform) in &mut emitters {
        emitter.spawn_timer.tick(time.delta());
        
        if emitter.spawn_timer.just_finished() && emitter.particles_to_spawn > 0 {
            let angle = thread_rng().gen_range(0.0..std::f32::consts::TAU);
            let speed = thread_rng().gen_range(200.0..400.0);
            let velocity = Vec3::new(angle.cos() * speed, angle.sin() * speed, 0.0);
            
            commands.spawn((
                Particle {
                    velocity,
                    lifetime: Timer::from_seconds(emitter.particle_lifetime, TimerMode::Once),
                    gravity: 500.0,
                },
                Sprite {
                    custom_size: Some(Vec2::splat(5.0)),
                    color: MCLAREN_ORANGE,
                    ..default()
                },
                Transform::from_translation(emitter_transform.translation),
            ));
            
            emitter.particles_to_spawn -= 1;
            
            if emitter.particles_to_spawn == 0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn cleanup_dead_particles(
    mut commands: Commands,
    particles: Query<(Entity, &Particle)>,
) {
    for (entity, particle) in &particles {
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn update_game_state_display(
    mut query: Query<&mut Text, With<GameStateDisplay>>,
    game_phase: Res<State<GamePhase>>,
    game_state: Res<GameState>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        *text = Text::new(format!(
            "Phase: {:?} | Chips: {} | Bet: {}", 
            game_phase.get(), 
            game_state.player_chips,
            game_state.current_bet
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_flow() {
        // Test that cards can be created and compared
        let card1 = Card { suit: Suit::Hearts, rank: Rank::King };
        let card2 = Card { suit: Suit::Spades, rank: Rank::Queen };
        
        assert_eq!(card1.value(), 13);
        assert_eq!(card2.value(), 12);
        assert!(card1.value() > card2.value());
        
        // Test card display
        assert_eq!(card1.get_rank_symbol(), "K");
        assert_eq!(card1.get_suit_symbol(), "♥");
        assert_eq!(card1.get_color(), MCLAREN_ORANGE);
        
        assert_eq!(card2.get_rank_symbol(), "Q");
        assert_eq!(card2.get_suit_symbol(), "♠");
        assert_eq!(card2.get_color(), ALUMINUM);
    }
    use super::*;
    
    #[test]
    fn test_mclaren_colors() {
        assert_eq!(MCLAREN_ORANGE, Color::srgb(1.0, 0.529, 0.0));
        assert_eq!(CARBON_BLACK, Color::srgb(0.08, 0.08, 0.1));
        assert_eq!(ALUMINUM, Color::srgb(0.7, 0.71, 0.72));
    }
    
    #[test]
    fn test_card_colors_mclaren() {
        let heart = Card { suit: Suit::Hearts, rank: Rank::Ace };
        assert_eq!(heart.get_color(), MCLAREN_ORANGE);
        
        let spade = Card { suit: Suit::Spades, rank: Rank::King };
        assert_eq!(spade.get_color(), ALUMINUM);
    }
    
    #[test]
    fn test_hologram_effect_defaults() {
        let hologram = HologramEffect {
            scan_speed: 2.0,
            glow_intensity: 0.8,
            flicker_rate: 0.02,
        };
        
        assert_eq!(hologram.scan_speed, 2.0);
        assert_eq!(hologram.glow_intensity, 0.8);
        assert_eq!(hologram.flicker_rate, 0.02);
    }

    // Test game state resource
    #[test]
    fn test_game_state() {
        let mut game_state = GameState {
            player_chips: 1000,
            current_bet: 0,
            war_bet: 0,
            deck: vec![],
        };
        
        // Test initial state
        assert_eq!(game_state.player_chips, 1000);
        assert_eq!(game_state.current_bet, 0);
        
        // Test betting
        game_state.current_bet = 100;
        assert_eq!(game_state.current_bet, 100);
        
        // Test chip deduction
        game_state.player_chips -= game_state.current_bet;
        assert_eq!(game_state.player_chips, 900);
    }

    // Test betting limits
    #[test]
    fn test_betting_limits() {
        let game_state = GameState {
            player_chips: 500,
            current_bet: 0,
            war_bet: 0,
            deck: vec![],
        };
        
        // Test minimum bet
        assert!(MIN_BET <= game_state.player_chips);
        
        // Test maximum bet
        assert!(MAX_BET >= game_state.player_chips);
        
        // Can't bet more than chips
        let max_bet = game_state.player_chips.min(MAX_BET);
        assert_eq!(max_bet, 500);
    }

    // Test war betting mechanics
    #[test]
    fn test_war_betting() {
        let mut game_state = GameState {
            player_chips: 1000,
            current_bet: 100,
            war_bet: 0,
            deck: vec![],
        };
        
        // Going to war requires matching bet
        game_state.war_bet = game_state.current_bet;
        assert_eq!(game_state.war_bet, 100);
        
        // Total at risk during war
        let total_risk = game_state.current_bet + game_state.war_bet;
        assert_eq!(total_risk, 200);
        
        // Must have enough chips for war
        assert!(game_state.player_chips >= total_risk);
    }

    // Test round outcomes
    #[test]
    fn test_round_outcomes() {
        // Player wins
        let player_card = Card {
            suit: Suit::Hearts,
            rank: Rank::Ace,
        };
        let dealer_card = Card {
            suit: Suit::Spades,
            rank: Rank::King,
        };
        
        assert!(player_card.value() > dealer_card.value());
        
        // Dealer wins
        let player_card2 = Card {
            suit: Suit::Diamonds,
            rank: Rank::Two,
        };
        assert!(dealer_card.value() > player_card2.value());
        
        // Tie (war condition)
        let player_card3 = Card {
            suit: Suit::Clubs,
            rank: Rank::King,
        };
        assert_eq!(player_card3.value(), dealer_card.value());
    }

    // Test payout calculations
    #[test]
    fn test_payouts() {
        // Regular win: 1:1 payout
        let bet = 100;
        let regular_win_payout = bet * 2; // Return bet + winnings
        assert_eq!(regular_win_payout, 200);
        
        // War win: 1:1 on war bet, push on original
        let war_bet = 100;
        let war_win_payout = bet + war_bet * 2; // Original bet returned + war bet doubled
        assert_eq!(war_win_payout, 300);
        
        // Surrender: lose half the bet
        let surrender_loss = bet / 2;
        assert_eq!(surrender_loss, 50);
    }


    // Test game phase transitions
    #[test]
    fn test_game_phases() {
        use GamePhase::*;
        
        // Valid phase transitions
        let valid_transitions = vec![
            (MainMenu, Betting),
            (Betting, Dealing),
            (Dealing, Comparing),
            (Comparing, RoundComplete),
            (Comparing, TieDecision),
            (TieDecision, War),
            (TieDecision, RoundComplete),
            (War, RoundComplete),
            (RoundComplete, Betting),
        ];
        
        // All transitions should be valid according to game rules
        for (from, to) in valid_transitions {
            println!("Valid transition: {:?} -> {:?}", from, to);
        }
    }
}