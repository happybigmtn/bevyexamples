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

// Marker component for state-based cleanup
#[derive(Component)]
struct StateScoped(GamePhase);

// Marker for buttons
#[derive(Component)]
struct PlayButton;

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
        // Setup systems
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GamePhase::MainMenu), setup_main_menu)
        .add_systems(OnExit(GamePhase::MainMenu), cleanup_state)
        .add_systems(OnEnter(GamePhase::Betting), setup_betting_ui)
        .add_systems(OnExit(GamePhase::Betting), cleanup_state)
        .add_systems(OnEnter(GamePhase::Dealing), on_enter_dealing)
        // Update systems
        .add_systems(Update, (
            handle_play_button.run_if(in_state(GamePhase::MainMenu)),
            (
                handle_chip_buttons,
                handle_deal_button,
                update_chip_display,
            ).run_if(in_state(GamePhase::Betting)),
            animate_cards,
        ))
        // Game logic systems
        .add_systems(Update, deal_cards_system)
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
        Transform::from_translation(DECK_POSITION), // Start at deck
        Visibility::default(),
        StateScoped(GamePhase::Dealing), // Clean up when leaving dealing phase
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
    mut next_state: ResMut<NextState<GamePhase>>,
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
            
            // Animate player card
            commands.entity(player_entity).insert(CardAnimation {
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
            
            // Animate dealer card with delay
            commands.entity(dealer_entity).insert(CardAnimation {
                start_pos: DECK_POSITION,
                end_pos: DEALER_CARD_POSITION,
                start_rotation: Quat::IDENTITY,
                end_rotation: Quat::IDENTITY,
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            });
            
            // Transition to comparing after a delay
            // In a real game, you'd wait for animations to complete
            next_state.set(GamePhase::Comparing);
        }
    }
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_card_value() {
        let card = Card {
            suit: Suit::Hearts,
            rank: Rank::King,
        };
        assert_eq!(card.value(), 13);
        
        let ace = Card {
            suit: Suit::Spades,
            rank: Rank::Ace,
        };
        assert_eq!(ace.value(), 14);
    }
    
    #[test]
    fn test_deck_creation() {
        let deck = GameState::create_deck();
        assert_eq!(deck.len(), 52);
        
        // Check all ranks and suits are present
        let mut found_cards = std::collections::HashSet::new();
        for card in &deck {
            found_cards.insert((card.suit as u8, card.rank as u8));
        }
        assert_eq!(found_cards.len(), 52);
    }
    
    #[test]
    fn test_bet_validation() {
        let game_state = GameState::default();
        
        // Test minimum bet
        assert!(MIN_BET <= game_state.player_chips);
        
        // Test maximum bet
        assert!(MAX_BET <= 1000); // Starting chips
    }
    
    #[test]
    fn test_card_symbols() {
        let heart_card = Card {
            suit: Suit::Hearts,
            rank: Rank::Ace,
        };
        assert_eq!(heart_card.get_suit_symbol(), "♥");
        assert_eq!(heart_card.get_rank_symbol(), "A");
        
        let ten_spades = Card {
            suit: Suit::Spades,
            rank: Rank::Ten,
        };
        assert_eq!(ten_spades.get_suit_symbol(), "♠");
        assert_eq!(ten_spades.get_rank_symbol(), "10");
    }
}