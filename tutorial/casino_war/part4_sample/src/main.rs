// Casino War Part 4: The War Mechanic
//
// This part completes the Casino War game by implementing the "war" phase -
// the exciting showdown that happens when player and dealer tie.
//
// Key concepts we'll explore:
// 1. Complex multi-card animations with choreographed timing
// 2. State machine transitions with cleanup and setup
// 3. Resource management for war betting
// 4. Visual feedback and game juice techniques
// 5. Complete game loop with win/loss conditions

use bevy::prelude::*;
use rand::prelude::*;

// Card representation
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
            Suit::Hearts | Suit::Diamonds => HEART_DIAMOND_COLOR,
            Suit::Clubs | Suit::Spades => CLUB_SPADE_COLOR,
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

// Constants for card layout
const CARD_WIDTH: f32 = 80.0;
const CARD_HEIGHT: f32 = 120.0;
const CARD_Z_BASE: f32 = 0.0;
const CARD_Z_INCREMENT: f32 = 0.1;

// Table positions
const DECK_POSITION: Vec3 = Vec3::new(-400.0, 0.0, 0.0);
const PLAYER_CARD_POSITION: Vec3 = Vec3::new(0.0, -200.0, 1.0);
const DEALER_CARD_POSITION: Vec3 = Vec3::new(0.0, 200.0, 1.0);

// NEW IN PART 4: War card positions
// During war, we need to display multiple cards:
// - 3 burn cards (face down) for each player
// - 1 final card (face up) for comparison
const WAR_CARD_SPACING: f32 = 100.0;  // Horizontal spacing between war cards
const WAR_PLAYER_Y: f32 = -200.0;     // Same Y as regular cards
const WAR_DEALER_Y: f32 = 200.0;

// Card colors and styling
const CARD_BACKGROUND: Color = Color::srgb(0.95, 0.95, 0.95);
const CARD_BACK_COLOR: Color = Color::srgb(0.2, 0.3, 0.6);
const HEART_DIAMOND_COLOR: Color = Color::srgb(0.8, 0.1, 0.1);
const CLUB_SPADE_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

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
    War,           // The exciting war phase!
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

// NEW IN PART 4: Burn cards are face-down cards dealt during war
#[derive(Component)]
struct BurnCard;

// Components for game logic
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

// NEW IN PART 4: Track which cards are part of the war sequence
#[derive(Component)]
struct WarCard {
    // Index in the war sequence (0-2 are burn cards, 3 is the final card)
    index: usize,
    // Delay before this card starts animating
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

// NEW IN PART 4: Events for war phase
#[derive(Event)]
struct WarCardsDealt;  // All war cards have been dealt

#[derive(Event)]
struct WarComplete {   // War phase is complete
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

// Marker for buttons
#[derive(Component)]
struct PlayButton;

// Component for tie decision buttons
#[derive(Component)]
struct TieDecisionButton {
    go_to_war: bool,
}

// NEW IN PART 4: Component to display round results
#[derive(Component)]
struct ResultDisplay;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Casino War".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GamePhase>()
        .init_resource::<GameState>()
        .add_event::<BetPlaced>()
        .add_event::<DealCards>()
        .add_event::<PlayerDecision>()
        .add_event::<RoundResult>()
        .add_event::<CardsDealt>()
        .add_event::<RequestCardFlip>()
        .add_event::<ComparisonComplete>()
        // NEW IN PART 4: War events
        .add_event::<WarCardsDealt>()
        .add_event::<WarComplete>()
        // Setup systems
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GamePhase::MainMenu), setup_main_menu)
        .add_systems(OnExit(GamePhase::MainMenu), cleanup_state)
        .add_systems(OnEnter(GamePhase::Betting), setup_betting_ui)
        .add_systems(OnExit(GamePhase::Betting), cleanup_state)
        .add_systems(OnEnter(GamePhase::Dealing), on_enter_dealing)
        .add_systems(OnEnter(GamePhase::Comparing), on_enter_comparing)
        .add_systems(OnEnter(GamePhase::TieDecision), setup_tie_decision_ui)
        .add_systems(OnExit(GamePhase::TieDecision), cleanup_state)
        // NEW IN PART 4: War phase systems
        .add_systems(OnEnter(GamePhase::War), on_enter_war)
        .add_systems(OnExit(GamePhase::War), cleanup_war_cards)
        .add_systems(OnEnter(GamePhase::RoundComplete), setup_round_complete_ui)
        .add_systems(OnExit(GamePhase::RoundComplete), cleanup_state)
        
        // Update systems
        .add_systems(Update, (
            handle_play_button.run_if(in_state(GamePhase::MainMenu)),
            (
                handle_chip_buttons,
                handle_deal_button,
                update_chip_display,
            ).run_if(in_state(GamePhase::Betting)),
            animate_cards,
            animate_card_flips,
            check_dealing_complete.run_if(in_state(GamePhase::Dealing)),
            // NEW IN PART 4: War animation systems
            animate_war_cards.run_if(in_state(GamePhase::War)),
            check_war_dealing_complete.run_if(in_state(GamePhase::War)),
            handle_continue_button.run_if(in_state(GamePhase::RoundComplete)),
        ))
        // Game logic systems
        .add_systems(Update, (
            deal_cards_system,
            flip_dealer_card_system,
            compare_cards_system.run_if(in_state(GamePhase::Comparing)),
            handle_comparison_result,
            handle_tie_decision_buttons.run_if(in_state(GamePhase::TieDecision)),
            // NEW IN PART 4: War logic
            deal_war_cards,
            flip_war_cards_system,
            compare_war_cards_system.run_if(in_state(GamePhase::War)),
            handle_war_result,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);
    
    // Table background
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            color: Color::srgb(0.0, 0.4, 0.2),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}

fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        StateScoped(GamePhase::MainMenu),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Casino War"),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.8, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(50.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Button,
            PlayButton,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::srgb(0.8, 0.7, 0.0)),
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Play"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
}

fn cleanup_state(
    mut commands: Commands,
    query: Query<Entity, With<StateScoped>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_play_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayButton>)
    >,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    for (interaction, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GamePhase::Betting);
            }
            Interaction::Hovered => {
                *background_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *background_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}

fn setup_betting_ui(mut commands: Commands) {
    // Root betting UI
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            padding: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        StateScoped(GamePhase::Betting),
    ))
    .with_children(|parent| {
        // Left side - Chip count
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Chips: $"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .with_child((
                TextSpan::new("1000"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.0)),
                ChipDisplay,
            ));
        });
        
        // Center - Chip buttons
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for &value in &CHIP_VALUES {
                let color = match value {
                    5 => Color::srgb(0.8, 0.2, 0.2),
                    10 => Color::srgb(0.2, 0.2, 0.8),
                    25 => Color::srgb(0.2, 0.8, 0.2),
                    50 => Color::srgb(0.8, 0.8, 0.2),
                    100 => Color::srgb(0.1, 0.1, 0.1),
                    _ => Color::WHITE,
                };
                
                parent.spawn((
                    Button,
                    ChipButton { value },
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Percent(50.0)),
                    BorderColor(Color::WHITE),
                    BackgroundColor(color),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!("${}", value)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            }
        });
        
        // Right side - Current bet and deal button
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Bet display
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Bet: $"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                parent.spawn((
                    Text::new("0"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 0.9, 0.0)),
                    BetDisplay,
                ));
            });
            
            // Deal button
            parent.spawn((
                Button,
                DealButton,
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(50.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::srgb(0.0, 0.8, 0.0)),
                BackgroundColor(Color::srgb(0.0, 0.4, 0.0)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("DEAL"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

fn handle_chip_buttons(
    mut interaction_query: Query<
        (&Interaction, &ChipButton, &mut BackgroundColor),
        Changed<Interaction>
    >,
    mut game_state: ResMut<GameState>,
    mut bet_display_query: Query<&mut Text, With<BetDisplay>>,
) {
    for (interaction, chip_button, _background) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            let new_bet = game_state.current_bet + chip_button.value;
            
            if new_bet <= game_state.player_chips && new_bet <= MAX_BET {
                game_state.current_bet = new_bet;
                
                if let Ok(mut text) = bet_display_query.single_mut() {
                    *text = Text::new(format!("{}", game_state.current_bet));
                }
            }
        }
    }
}

fn handle_deal_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<DealButton>)
    >,
    game_state: Res<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut deal_events: EventWriter<DealCards>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if game_state.current_bet >= MIN_BET {
                    next_state.set(GamePhase::Dealing);
                    deal_events.write(DealCards);
                    *background = BackgroundColor(Color::srgb(0.0, 0.6, 0.0));
                }
            }
            Interaction::Hovered => {
                if game_state.current_bet >= MIN_BET {
                    *background = BackgroundColor(Color::srgb(0.0, 0.5, 0.0));
                }
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.0, 0.4, 0.0));
            }
        }
    }
}

fn update_chip_display(
    game_state: Res<GameState>,
    mut chip_display_query: Query<&mut TextSpan, With<ChipDisplay>>,
) {
    if game_state.is_changed() {
        if let Ok(mut text_span) = chip_display_query.single_mut() {
            **text_span = format!("{}", game_state.player_chips);
        }
    }
}

fn spawn_card(
    commands: &mut Commands,
    card: Card,
    position: CardPosition,
    face_up: bool,
) -> Entity {
    let world_pos = match position {
        CardPosition::Deck => DECK_POSITION,
        CardPosition::PlayerHand => PLAYER_CARD_POSITION,
        CardPosition::DealerHand => DEALER_CARD_POSITION,
        CardPosition::Discard => DECK_POSITION + Vec3::new(100.0, 0.0, 0.0),
    };
    
    let card_entity = commands.spawn((
        card,
        position,
        CardVisual {
            face_up,
            target_position: world_pos,
        },
        Transform::from_translation(DECK_POSITION),
        Visibility::default(),
        StateScoped(GamePhase::Dealing),
    ))
    .with_children(|parent| {
        // Card background
        parent.spawn((
            Sprite {
                custom_size: Some(Vec2::new(CARD_WIDTH, CARD_HEIGHT)),
                color: CARD_BACKGROUND,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        
        // Card content
        if face_up {
            // Card face
            let rank_text = card.get_rank_symbol();
            let suit_text = card.get_suit_symbol();
            let color = card.get_color();
            
            // Top-left rank
            parent.spawn((
                Text::new(rank_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(-CARD_WIDTH/2.0 + 15.0, CARD_HEIGHT/2.0 - 20.0, 0.2),
            ));
            
            // Center suit
            parent.spawn((
                Text::new(suit_text),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));
            
            // Bottom-right rank (rotated)
            parent.spawn((
                Text::new(rank_text),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(CARD_WIDTH/2.0 - 15.0, -CARD_HEIGHT/2.0 + 20.0, 0.2)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
            ));
        } else {
            // Card back
            parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(CARD_WIDTH - 10.0, CARD_HEIGHT - 10.0)),
                    color: CARD_BACK_COLOR,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));
            
            // Add a simple pattern
            parent.spawn((
                Text::new("♠"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.4, 0.7)),
                Transform::from_xyz(0.0, 0.0, 0.3),
            ));
        }
    })
    .id();
    
    // Add appropriate marker
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

fn animate_cards(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CardAnimation)>,
) {
    for (entity, mut transform, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        
        if animation.timer.finished() {
            transform.translation = animation.end_pos;
            transform.rotation = animation.end_rotation;
            commands.entity(entity).remove::<CardAnimation>();
        } else {
            let t = animation.timer.fraction();
            let t = t * t * (3.0 - 2.0 * t); // Smooth easing
            
            transform.translation = animation.start_pos.lerp(animation.end_pos, t);
            transform.rotation = animation.start_rotation.slerp(animation.end_rotation, t);
        }
    }
}

fn on_enter_dealing(mut deal_events: EventWriter<DealCards>) {
    // Trigger the deal when entering dealing phase
    deal_events.write(DealCards);
}

fn deal_cards_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut deal_events: EventReader<DealCards>,
    mut _next_state: ResMut<NextState<GamePhase>>,
) {
    for _ in deal_events.read() {
        // Deduct bet from player chips
        game_state.player_chips -= game_state.current_bet;
        
        // Draw cards
        if let (Some(player_card), Some(dealer_card)) = 
            (game_state.draw_card(), game_state.draw_card()) 
        {
            // Spawn player card
            let player_entity = spawn_card(
                &mut commands,
                player_card,
                CardPosition::PlayerHand,
                true,
            );
            
            // Mark as active card and animate
            commands.entity(player_entity)
                .insert(ActiveCard)
                .insert(CardAnimation {
                    start_pos: DECK_POSITION,
                    end_pos: PLAYER_CARD_POSITION,
                    start_rotation: Quat::IDENTITY,
                    end_rotation: Quat::IDENTITY,
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                });
            
            // Spawn dealer card (face down)
            let dealer_entity = spawn_card(
                &mut commands,
                dealer_card,
                CardPosition::DealerHand,
                false,
            );
            
            // Mark as active and animate
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

fn check_dealing_complete(
    animating_cards: Query<&CardAnimation, With<ActiveCard>>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut events: EventWriter<CardsDealt>,
) {
    if animating_cards.is_empty() {
        events.write(CardsDealt);
        next_state.set(GamePhase::Comparing);
    }
}

fn on_enter_comparing(mut flip_events: EventWriter<RequestCardFlip>) {
    flip_events.write(RequestCardFlip);
}

fn animate_card_flips(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CardFlipAnimation, &Card, &mut CardVisual)>,
) {
    for (entity, mut transform, mut flip_anim, _card, mut visual) in &mut query {
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
) {
    if let Ok(visual) = dealer_visual.single() {
        if !visual.face_up {
            return;
        }
    }
    
    let Ok(player_card) = player_cards.single() else { return };
    let Ok(dealer_card) = dealer_cards.single() else { return };
    
    let player_value = player_card.value();
    let dealer_value = dealer_card.value();
    
    let outcome = match player_value.cmp(&dealer_value) {
        std::cmp::Ordering::Greater => ComparisonOutcome::PlayerWins,
        std::cmp::Ordering::Less => ComparisonOutcome::DealerWins,
        std::cmp::Ordering::Equal => ComparisonOutcome::Tie,
    };
    
    commands.spawn(ComparisonResult {
        player_value,
        dealer_value,
    });
    
    comparison_events.write(ComparisonComplete { outcome });
}

fn handle_comparison_result(
    mut comparison_events: EventReader<ComparisonComplete>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut round_events: EventWriter<RoundResult>,
) {
    for event in comparison_events.read() {
        match event.outcome {
            ComparisonOutcome::PlayerWins => {
                let winnings = game_state.current_bet as i32 * 2;
                game_state.player_chips += winnings as u32;
                
                round_events.write(RoundResult {
                    player_won: true,
                    winnings,
                });
                
                next_state.set(GamePhase::RoundComplete);
            }
            ComparisonOutcome::DealerWins => {
                let winnings = -(game_state.current_bet as i32);
                
                round_events.write(RoundResult {
                    player_won: false,
                    winnings,
                });
                
                next_state.set(GamePhase::RoundComplete);
            }
            ComparisonOutcome::Tie => {
                next_state.set(GamePhase::TieDecision);
            }
        }
    }
}

fn setup_tie_decision_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        StateScoped(GamePhase::TieDecision),
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor(Color::srgb(0.8, 0.8, 0.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("IT'S A TIE!"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));
            
            parent.spawn((
                Text::new("Choose your action:"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Button,
                    TieDecisionButton { go_to_war: false },
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::srgb(0.8, 0.0, 0.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Surrender\n(Lose half bet)"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
                
                parent.spawn((
                    Button,
                    TieDecisionButton { go_to_war: true },
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(80.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(Color::srgb(0.0, 0.8, 0.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Go to War!\n(Match bet)"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            });
        });
    });
}

fn handle_tie_decision_buttons(
    mut interaction_query: Query<
        (&Interaction, &TieDecisionButton, &mut BackgroundColor),
        Changed<Interaction>
    >,
    mut player_events: EventWriter<PlayerDecision>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if button.go_to_war {
                    // Going to war costs another bet equal to the original
                    if game_state.player_chips >= game_state.current_bet {
                        // Store the war bet amount
                        game_state.war_bet = game_state.current_bet;
                        game_state.player_chips -= game_state.war_bet;
                        
                        player_events.write(PlayerDecision { go_to_war: true });
                        next_state.set(GamePhase::War);
                    }
                } else {
                    // Surrender - get half bet back
                    let half_bet = game_state.current_bet / 2;
                    game_state.player_chips += half_bet;
                    
                    player_events.write(PlayerDecision { go_to_war: false });
                    next_state.set(GamePhase::RoundComplete);
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}

// NEW IN PART 4: War phase implementation
// The war mechanic is the heart of Casino War's excitement!
// When there's a tie, players can "go to war" by matching their bet.
// Then, each side gets 3 burn cards (face down) and 1 final card (face up).
// The higher final card wins both the original bet and the war bet.
fn on_enter_war(mut commands: Commands) {
    // Create a dramatic "WAR!" announcement
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
                font_size: 120.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.0, 0.0)),
            // This will be animated later
            WarAnnouncement {
                timer: Timer::from_seconds(1.5, TimerMode::Once),
            },
        ));
    });
    
    // Trigger war card dealing after announcement
    commands.insert_resource(WarDealingTimer {
        timer: Timer::from_seconds(1.5, TimerMode::Once),
    });
}

// Component for the war announcement animation
#[derive(Component)]
struct WarAnnouncement {
    timer: Timer,
}

// Resource to delay war card dealing
#[derive(Resource)]
struct WarDealingTimer {
    timer: Timer,
}

// Clean up war cards when leaving war phase
fn cleanup_war_cards(
    mut commands: Commands,
    war_cards: Query<Entity, With<WarCard>>,
) {
    // Remove all war cards
    for entity in war_cards.iter() {
        commands.entity(entity).despawn();
    }
}

// Deal the war cards with staggered animation
fn deal_war_cards(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    mut timer: Option<ResMut<WarDealingTimer>>,
) {
    // Wait for announcement to finish
    if let Some(ref mut timer) = timer {
        timer.timer.tick(time.delta());
        if !timer.timer.finished() {
            return;
        }
        
        // Only deal once
        commands.remove_resource::<WarDealingTimer>();
        
        // Deal war cards
        // Each player gets 3 burn cards (face down) + 1 final card (face up)
        for i in 0..4 {
            // Player cards
            if let Some(card) = game_state.draw_card() {
                let is_final = i == 3;
                let x_offset = (i as f32 - 1.5) * WAR_CARD_SPACING;
                let target_pos = Vec3::new(x_offset, WAR_PLAYER_Y, 1.0 + i as f32 * 0.1);
                
                let entity = spawn_card(
                    &mut commands,
                    card,
                    CardPosition::PlayerHand,
                    false, // Start face down, will flip if final
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
            
            // Dealer cards
            if let Some(card) = game_state.draw_card() {
                let is_final = i == 3;
                let x_offset = (i as f32 - 1.5) * WAR_CARD_SPACING;
                let target_pos = Vec3::new(x_offset, WAR_DEALER_Y, 1.0 + i as f32 * 0.1);
                
                let entity = spawn_card(
                    &mut commands,
                    card,
                    CardPosition::DealerHand,
                    false, // All start face down
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

// Animate war cards with delay
fn animate_war_cards(
    mut commands: Commands,
    time: Res<Time>,
    mut war_cards: Query<(Entity, &mut WarCard), Without<CardAnimation>>,
) {
    for (entity, mut war_card) in &mut war_cards {
        war_card.delay.tick(time.delta());
        
        if war_card.delay.finished() {
            // Card is ready to animate - the animation is already attached
            // Just need to remove the WarCard component so it doesn't tick again
            commands.entity(entity).remove::<WarCard>();
        }
    }
}

// Check when all war cards are dealt
fn check_war_dealing_complete(
    animating_cards: Query<&CardAnimation, With<ActiveCard>>,
    war_cards: Query<&WarCard>,
    mut events: EventWriter<WarCardsDealt>,
    mut flip_events: EventWriter<RequestCardFlip>,
) {
    // All cards must be spawned and animated
    if animating_cards.is_empty() && war_cards.is_empty() {
        events.write(WarCardsDealt);
        // Request flip of final cards
        flip_events.write(RequestCardFlip);
    }
}

// Flip the final war cards
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

// Compare war cards
fn compare_war_cards_system(
    player_cards: Query<&Card, (With<PlayerCard>, With<ActiveCard>)>,
    dealer_cards: Query<&Card, (With<DealerCard>, With<ActiveCard>)>,
    dealer_visual: Query<&CardVisual, (With<DealerCard>, With<ActiveCard>)>,
    mut war_events: EventWriter<WarComplete>,
) {
    // Wait for all cards to be face up
    for visual in &dealer_visual {
        if !visual.face_up {
            return;
        }
    }
    
    let Ok(player_card) = player_cards.single() else { return };
    let Ok(dealer_card) = dealer_cards.single() else { return };
    
    let player_won = player_card.value() > dealer_card.value();
    
    // In war, ties go to the dealer (house edge)
    war_events.write(WarComplete { player_won });
}

// Handle war result
fn handle_war_result(
    mut war_events: EventReader<WarComplete>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut round_events: EventWriter<RoundResult>,
) {
    for event in war_events.read() {
        if event.player_won {
            // Player wins:
            // - Original bet pays 1:1
            // - War bet pays 1:1
            // Total winnings = original bet * 2 + war bet * 2
            let total_winnings = (game_state.current_bet * 2) + (game_state.war_bet * 2);
            game_state.player_chips += total_winnings;
            
            round_events.write(RoundResult {
                player_won: true,
                winnings: total_winnings as i32,
            });
        } else {
            // Player loses both bets (already deducted)
            let total_loss = (game_state.current_bet + game_state.war_bet) as i32;
            
            round_events.write(RoundResult {
                player_won: false,
                winnings: -total_loss,
            });
        }
        
        // Reset war bet
        game_state.war_bet = 0;
        next_state.set(GamePhase::RoundComplete);
    }
}

// NEW IN PART 4: Round complete UI
fn setup_round_complete_ui(
    mut commands: Commands,
    mut round_events: EventReader<RoundResult>,
    game_state: Res<GameState>,
) {
    let mut result_text = String::new();
    let mut result_color = Color::WHITE;
    let mut winnings_text = String::new();
    
    for event in round_events.read() {
        if event.player_won {
            result_text = "YOU WIN!".to_string();
            result_color = Color::srgb(0.0, 1.0, 0.0);
            winnings_text = format!("+${}", event.winnings);
        } else {
            result_text = "DEALER WINS".to_string();
            result_color = Color::srgb(1.0, 0.0, 0.0);
            winnings_text = format!("${}", event.winnings);
        }
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
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        StateScoped(GamePhase::RoundComplete),
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor(result_color),
        ))
        .with_children(|parent| {
            // Result text
            parent.spawn((
                Text::new(result_text),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(result_color),
                ResultDisplay,
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            
            // Winnings
            parent.spawn((
                Text::new(winnings_text),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));
            
            // Chip count
            parent.spawn((
                Text::new(format!("Chips: ${}", game_state.player_chips)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));
            
            // Continue button
            parent.spawn((
                Button,
                ContinueButton,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(60.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor(Color::srgb(0.8, 0.8, 0.8)),
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Continue"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
        });
    });
}

// Component for continue button
#[derive(Component)]
struct ContinueButton;

// Handle continue button
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
                // Reset for next round
                game_state.current_bet = 0;
                game_state.war_bet = 0;
                
                // Go back to betting
                next_state.set(GamePhase::Betting);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}

// Tests remain the same as Part 3 with additional war tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_war_betting() {
        let mut game_state = GameState {
            player_chips: 100,
            current_bet: 20,
            war_bet: 0,
            deck: Vec::new(),
        };
        
        // Test going to war
        let can_afford_war = game_state.player_chips >= game_state.current_bet;
        assert!(can_afford_war);
        
        // Simulate war bet
        game_state.war_bet = game_state.current_bet;
        game_state.player_chips -= game_state.war_bet;
        assert_eq!(game_state.player_chips, 80);
        assert_eq!(game_state.war_bet, 20);
    }
    
    #[test]
    fn test_war_payouts() {
        let game_state = GameState {
            player_chips: 100,
            current_bet: 20,
            war_bet: 20,
            deck: Vec::new(),
        };
        
        // Test player win payout
        let win_payout = (game_state.current_bet * 2) + (game_state.war_bet * 2);
        assert_eq!(win_payout, 80); // 40 + 40
        
        // Test player loss
        let total_loss = game_state.current_bet + game_state.war_bet;
        assert_eq!(total_loss, 40);
    }
}