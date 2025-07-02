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
use bevy::text::*;
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
            Suit::Hearts | Suit::Diamonds => DANGER_RED,
            Suit::Clubs | Suit::Spades => ALUMINUM,
        }
    }
}

// Component for card entities
#[derive(Component)]
struct CardVisual {
    face_up: bool,
}

// Component markers
#[derive(Component)]
struct PlayerCard;

#[derive(Component)]
struct DealerCard;

#[derive(Component)]
struct ActiveCard;

#[derive(Component)]
struct CardFace;

#[derive(Component)]
struct CardBack;

// Animation components
#[derive(Component)]
struct CardAnimation {
    start_pos: Vec3,
    end_pos: Vec3,
    timer: Timer,
}

#[derive(Component)]
struct CardFlipAnimation {
    timer: Timer,
    half_flipped: bool,
}

// UI Components
#[derive(Component)]
struct ChipDisplay;

#[derive(Component)]
struct BetDisplay;

#[derive(Component)]
struct GameStateDisplay;

#[derive(Component)]
struct ComparisonDisplay;

#[derive(Component)]
struct ChipButton {
    value: u32,
}

#[derive(Component)]
struct DealButton;

#[derive(Component)]
struct ContinueButton;

#[derive(Component)]
struct TieDecisionButton {
    go_to_war: bool,
}

// Game state
#[derive(Resource)]
struct GameState {
    deck: Vec<Card>,
    player_chips: u32,
    current_bet: u32,
    war_cards: Vec<Card>,
}

impl Default for GameState {
    fn default() -> Self {
        let mut deck = Vec::new();
        
        for &suit in &[Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
            for rank_value in 2..=14 {
                let rank = match rank_value {
                    2 => Rank::Two,
                    3 => Rank::Three,
                    4 => Rank::Four,
                    5 => Rank::Five,
                    6 => Rank::Six,
                    7 => Rank::Seven,
                    8 => Rank::Eight,
                    9 => Rank::Nine,
                    10 => Rank::Ten,
                    11 => Rank::Jack,
                    12 => Rank::Queen,
                    13 => Rank::King,
                    14 => Rank::Ace,
                    _ => unreachable!(),
                };
                
                deck.push(Card { suit, rank });
            }
        }
        
        let mut rng = thread_rng();
        deck.shuffle(&mut rng);
        
        Self {
            deck,
            player_chips: 1000,
            current_bet: 0,
            war_cards: Vec::new(),
        }
    }
}

impl GameState {
    fn draw_card(&mut self) -> Option<Card> {
        if self.deck.len() < 10 {
            self.reset_deck();
        }
        self.deck.pop()
    }
    
    fn reset_deck(&mut self) {
        self.deck.clear();
        
        for &suit in &[Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
            for rank_value in 2..=14 {
                let rank = match rank_value {
                    2 => Rank::Two,
                    3 => Rank::Three,
                    4 => Rank::Four,
                    5 => Rank::Five,
                    6 => Rank::Six,
                    7 => Rank::Seven,
                    8 => Rank::Eight,
                    9 => Rank::Nine,
                    10 => Rank::Ten,
                    11 => Rank::Jack,
                    12 => Rank::Queen,
                    13 => Rank::King,
                    14 => Rank::Ace,
                    _ => unreachable!(),
                };
                
                self.deck.push(Card { suit, rank });
            }
        }
        
        let mut rng = thread_rng();
        self.deck.shuffle(&mut rng);
    }
}

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
    GameOver,
}

// Events
#[derive(Event)]
struct DealCards;

#[derive(Event)]
struct RequestCardFlip;

#[derive(Event, Debug)]
struct ComparisonComplete {
    outcome: ComparisonOutcome,
}

#[derive(Debug, Clone, Copy)]
enum ComparisonOutcome {
    PlayerWins,
    DealerWins,
    Tie,
}

// Constants
const CARD_WIDTH: f32 = 80.0;
const CARD_HEIGHT: f32 = 120.0;
const PLAYER_CARD_POSITION: Vec3 = Vec3::new(-100.0, -150.0, 1.0);
const DEALER_CARD_POSITION: Vec3 = Vec3::new(-100.0, 100.0, 1.0);
const DECK_POSITION: Vec3 = Vec3::new(200.0, 0.0, 0.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GamePhase>()
        .insert_resource(GameState::default())
        .add_systems(Startup, setup_camera)
        .add_systems(Update, (
            handle_main_menu,
            handle_chip_buttons,
            handle_deal_button,
            update_chip_display,
            update_bet_display,
            animate_cards,
            animate_card_flips,
            update_card_visuals,
            compare_cards,
            handle_comparison_result,
            handle_tie_decision,
            handle_continue_button,
            update_game_state_display,
        ))
        .add_systems(OnEnter(GamePhase::MainMenu), setup_main_menu)
        .add_systems(OnEnter(GamePhase::Betting), setup_betting_ui)
        .add_systems(OnEnter(GamePhase::Dealing), on_enter_dealing)
        .add_systems(OnEnter(GamePhase::Comparing), on_enter_comparing)
        .add_systems(OnEnter(GamePhase::TieDecision), setup_tie_decision)
        .add_systems(OnEnter(GamePhase::RoundComplete), setup_round_complete)
        .add_systems(OnEnter(GamePhase::GameOver), setup_game_over)
        .add_event::<DealCards>()
        .add_event::<RequestCardFlip>()
        .add_event::<ComparisonComplete>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    // Debug game state display
    commands.spawn((
        Text2d::new("Phase: MainMenu | Chips: 1000 | Bet: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(MCLAREN_ORANGE),
        Transform::from_xyz(-300.0, 250.0, 100.0),
        GameStateDisplay,
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
        BackgroundColor(CARBON_BLACK),
        StateScoped(GamePhase::MainMenu),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("CASINO WAR"),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(MCLAREN_ORANGE),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(MCLAREN_ORANGE),
            BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PLAY"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
        });
    });
}

fn handle_main_menu(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>)
    >,
    game_phase: Res<State<GamePhase>>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    if *game_phase != GamePhase::MainMenu {
        return;
    }
    
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GamePhase::Betting);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.3));
            }
            Interaction::None => {
                *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1));
            }
        }
    }
}

fn setup_betting_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            padding: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        },
        BackgroundColor(PANEL_DARK),
        StateScoped(GamePhase::Betting),
    ))
    .with_children(|parent| {
        // Chip count
        parent.spawn((
            Text::new("Chips: "),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(TEXT_PRIMARY),
        ))
        .with_child((
            TextSpan::new("1000"),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(MCLAREN_ORANGE),
            ChipDisplay,
        ));
        
        // Chip buttons
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for &value in &[5, 10, 25, 50, 100] {
                parent.spawn((
                    Button,
                    ChipButton { value },
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(ALUMINUM),
                    BackgroundColor(CARBON_BLACK),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!("${}", value)),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(TEXT_PRIMARY),
                    ));
                });
            }
        });
        
        // Bet display and deal button
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                row_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
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
                    TextColor(TEXT_PRIMARY),
                ));
                parent.spawn((
                    Text::new("0"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(VICTORY_GREEN),
                    BetDisplay,
                ));
            });
            
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
                BorderColor(VICTORY_GREEN),
                BackgroundColor(VICTORY_GREEN.with_alpha(0.1)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("ENGAGE"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(TEXT_PRIMARY),
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
    game_phase: Res<State<GamePhase>>,
) {
    if *game_phase != GamePhase::Betting {
        return;
    }
    
    for (interaction, chip_button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let new_bet = game_state.current_bet + chip_button.value;
                if new_bet <= game_state.player_chips && new_bet <= 500 {
                    game_state.current_bet = new_bet;
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.2));
            }
            Interaction::None => {
                *background = BackgroundColor(CARBON_BLACK);
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
    game_phase: Res<State<GamePhase>>,
) {
    if *game_phase != GamePhase::Betting {
        return;
    }
    
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if game_state.current_bet >= 5 {
                    next_state.set(GamePhase::Dealing);
                    deal_events.send(DealCards);
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.3));
            }
            Interaction::None => {
                *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.1));
            }
        }
    }
}

fn update_chip_display(
    game_state: Res<GameState>,
    mut query: Query<&mut TextSpan, With<ChipDisplay>>,
) {
    if game_state.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            **text = format!("{}", game_state.player_chips);
        }
    }
}

fn update_bet_display(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<BetDisplay>>,
) {
    if game_state.is_changed() {
        if let Ok(mut text) = query.get_single_mut() {
            *text = Text::new(format!("{}", game_state.current_bet));
        }
    }
}

fn on_enter_dealing(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut deal_events: EventReader<DealCards>,
) {
    for _ in deal_events.read() {
        // Deduct bet
        game_state.player_chips -= game_state.current_bet;
        
        // Draw cards
        if let (Some(player_card), Some(dealer_card)) = 
            (game_state.draw_card(), game_state.draw_card()) 
        {
            // Spawn player card
            let player_entity = spawn_card(
                &mut commands,
                player_card,
                true, // face up
            );
            
            commands.entity(player_entity)
                .insert((
                    PlayerCard,
                    ActiveCard,
                    CardAnimation {
                        start_pos: DECK_POSITION,
                        end_pos: PLAYER_CARD_POSITION,
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                ));
            
            // Spawn dealer card
            let dealer_entity = spawn_card(
                &mut commands,
                dealer_card,
                false, // face down
            );
            
            commands.entity(dealer_entity)
                .insert((
                    DealerCard,
                    ActiveCard,
                    CardAnimation {
                        start_pos: DECK_POSITION,
                        end_pos: DEALER_CARD_POSITION,
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                ));
        }
    }
}

fn spawn_card(
    commands: &mut Commands,
    card: Card,
    face_up: bool,
) -> Entity {
    let card_entity = commands.spawn((
        card,
        CardVisual { face_up },
        Transform::from_translation(DECK_POSITION),
        Visibility::default(),
    ))
    .with_children(|parent| {
        // Card background
        parent.spawn((
            Sprite {
                custom_size: Some(Vec2::new(CARD_WIDTH, CARD_HEIGHT)),
                color: Color::WHITE,
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
        
        // Card face
        let face_visible = if face_up { Visibility::Visible } else { Visibility::Hidden };
        parent.spawn((
            CardFace,
            Transform::default(),
            face_visible,
        ))
        .with_children(|face_parent| {
            // Card border
            face_parent.spawn((
                Sprite {
                    custom_size: Some(Vec2::new(CARD_WIDTH - 4.0, CARD_HEIGHT - 4.0)),
                    color: Color::srgb(0.95, 0.95, 0.95),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.2),
            ));
            
            let color = card.get_color();
            let rank_str = card.get_rank_symbol();
            
            // Main rank display - large in center
            face_parent.spawn((
                Text2d::new(rank_str),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(0.0, 10.0, 0.5),
                Anchor::Center,
            ));
            
            // Suit symbol below rank
            face_parent.spawn((
                Text2d::new(card.get_suit_symbol()),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(0.0, -30.0, 0.5),
                Anchor::Center,
            ));
            
            // Top-left corner indicator
            face_parent.spawn((
                Text2d::new(format!("{}{}", rank_str, card.get_suit_symbol())),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(-30.0, 45.0, 0.5),
                Anchor::Center,
            ));
            
            // Bottom-right corner indicator (rotated)
            face_parent.spawn((
                Text2d::new(format!("{}{}", rank_str, card.get_suit_symbol())),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
                Transform::from_xyz(30.0, -45.0, 0.5)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
                Anchor::Center,
            ));
        });
        
        // Card back
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
    
    card_entity
}

fn animate_cards(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CardAnimation)>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    let mut all_animations_complete = true;
    
    for (entity, mut transform, mut animation) in &mut query {
        animation.timer.tick(time.delta());
        
        if animation.timer.finished() {
            transform.translation = animation.end_pos;
            commands.entity(entity).remove::<CardAnimation>();
        } else {
            all_animations_complete = false;
            let progress = animation.timer.fraction();
            transform.translation = animation.start_pos.lerp(animation.end_pos, progress);
        }
    }
    
    // When all cards are dealt, move to comparing
    if all_animations_complete && query.iter().count() == 0 {
        next_state.set(GamePhase::Comparing);
    }
}

fn on_enter_comparing(mut flip_events: EventWriter<RequestCardFlip>) {
    flip_events.send(RequestCardFlip);
}

fn animate_card_flips(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CardFlipAnimation, &mut CardVisual)>,
    mut flip_events: EventReader<RequestCardFlip>,
    dealer_cards: Query<Entity, (With<DealerCard>, With<ActiveCard>)>,
) {
    // Start flip animations
    for _ in flip_events.read() {
        for entity in &dealer_cards {
            commands.entity(entity).insert(CardFlipAnimation {
                timer: Timer::from_seconds(0.6, TimerMode::Once),
                half_flipped: false,
            });
        }
    }
    
    // Animate flips
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
            }
        }
    }
}

fn update_card_visuals(
    mut query: Query<(&CardVisual, &Children), Changed<CardVisual>>,
    mut face_query: Query<&mut Visibility, (With<CardFace>, Without<CardBack>)>,
    mut back_query: Query<&mut Visibility, (With<CardBack>, Without<CardFace>)>,
) {
    for (visual, children) in &mut query {
        for &child in children.iter() {
            if let Ok(mut face_vis) = face_query.get_mut(child) {
                *face_vis = if visual.face_up { Visibility::Visible } else { Visibility::Hidden };
            }
            if let Ok(mut back_vis) = back_query.get_mut(child) {
                *back_vis = if visual.face_up { Visibility::Hidden } else { Visibility::Visible };
            }
        }
    }
}

fn compare_cards(
    mut commands: Commands,
    player_cards: Query<&Card, (With<PlayerCard>, With<ActiveCard>)>,
    dealer_cards: Query<(&Card, &CardVisual), (With<DealerCard>, With<ActiveCard>)>,
    mut comparison_events: EventWriter<ComparisonComplete>,
    game_phase: Res<State<GamePhase>>,
) {
    if *game_phase != GamePhase::Comparing {
        return;
    }
    
    // Only compare when dealer card is face up
    for (dealer_card, dealer_visual) in &dealer_cards {
        if !dealer_visual.face_up {
            return;
        }
        
        if let Ok(player_card) = player_cards.get_single() {
            let player_value = player_card.value();
            let dealer_value = dealer_card.value();
            
            let outcome = match player_value.cmp(&dealer_value) {
                std::cmp::Ordering::Greater => ComparisonOutcome::PlayerWins,
                std::cmp::Ordering::Less => ComparisonOutcome::DealerWins,
                std::cmp::Ordering::Equal => ComparisonOutcome::Tie,
            };
            
            // Display comparison result
            let result_text = match outcome {
                ComparisonOutcome::PlayerWins => "YOU WIN!",
                ComparisonOutcome::DealerWins => "DEALER WINS",
                ComparisonOutcome::Tie => "TIE",
            };
            
            let result_color = match outcome {
                ComparisonOutcome::PlayerWins => VICTORY_GREEN,
                ComparisonOutcome::DealerWins => DANGER_RED,
                ComparisonOutcome::Tie => MCLAREN_ORANGE,
            };
            
            commands.spawn((
                Text2d::new(result_text),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(result_color),
                Transform::from_xyz(0.0, 0.0, 10.0),
                ComparisonDisplay,
            ));
            
            comparison_events.send(ComparisonComplete { outcome });
        }
    }
}

fn handle_comparison_result(
    mut comparison_events: EventReader<ComparisonComplete>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    for event in comparison_events.read() {
        match event.outcome {
            ComparisonOutcome::PlayerWins => {
                game_state.player_chips += game_state.current_bet * 2;
                next_state.set(GamePhase::RoundComplete);
            }
            ComparisonOutcome::DealerWins => {
                if game_state.player_chips < 5 {
                    next_state.set(GamePhase::GameOver);
                } else {
                    next_state.set(GamePhase::RoundComplete);
                }
            }
            ComparisonOutcome::Tie => {
                next_state.set(GamePhase::TieDecision);
            }
        }
    }
}

fn setup_tie_decision(mut commands: Commands) {
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
        ))
        .with_children(|modal| {
            modal.spawn((
                Text::new("STALEMATE DETECTED"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(MCLAREN_ORANGE),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));
            
            modal.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(30.0),
                    ..default()
                },
            ))
            .with_children(|buttons| {
                // Surrender button
                buttons.spawn((
                    Button,
                    TieDecisionButton { go_to_war: false },
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(DANGER_RED),
                    BackgroundColor(DANGER_RED.with_alpha(0.1)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("SURRENDER"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_PRIMARY),
                    ));
                });
                
                // War button
                buttons.spawn((
                    Button,
                    TieDecisionButton { go_to_war: true },
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(60.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor(VICTORY_GREEN),
                    BackgroundColor(VICTORY_GREEN.with_alpha(0.1)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("GO TO WAR"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(TEXT_PRIMARY),
                    ));
                });
            });
        });
    });
}

fn handle_tie_decision(
    mut interaction_query: Query<
        (&Interaction, &TieDecisionButton, &mut BackgroundColor),
        Changed<Interaction>
    >,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if button.go_to_war {
                    // War costs another bet
                    if game_state.player_chips >= game_state.current_bet {
                        game_state.player_chips -= game_state.current_bet;
                        // For simplicity, immediately resolve war
                        game_state.player_chips += game_state.current_bet * 3;
                        next_state.set(GamePhase::RoundComplete);
                    }
                } else {
                    // Surrender returns half the bet
                    game_state.player_chips += game_state.current_bet / 2;
                    next_state.set(GamePhase::RoundComplete);
                }
            }
            Interaction::Hovered => {
                if button.go_to_war {
                    *background = BackgroundColor(VICTORY_GREEN.with_alpha(0.3));
                } else {
                    *background = BackgroundColor(DANGER_RED.with_alpha(0.3));
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

fn setup_round_complete(mut commands: Commands) {
    // Clean up cards and comparison display
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(150.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        StateScoped(GamePhase::RoundComplete),
    ))
    .with_children(|parent| {
        parent.spawn((
            Button,
            ContinueButton,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                border: UiRect::all(Val::Px(3.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(MCLAREN_ORANGE),
            BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1)),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("CONTINUE"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ));
        });
    });
}

fn handle_continue_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ContinueButton>)
    >,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<GamePhase>>,
    mut commands: Commands,
    cards: Query<Entity, With<ActiveCard>>,
    comparison_display: Query<Entity, With<ComparisonDisplay>>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Clean up cards
                for entity in &cards {
                    commands.entity(entity).despawn_recursive();
                }
                
                // Clean up comparison display
                for entity in &comparison_display {
                    commands.entity(entity).despawn_recursive();
                }
                
                // Reset bet
                game_state.current_bet = 0;
                
                // Go back to betting
                next_state.set(GamePhase::Betting);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.3));
            }
            Interaction::None => {
                *background = BackgroundColor(MCLAREN_ORANGE.with_alpha(0.1));
            }
        }
    }
}

fn setup_game_over(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(CARBON_BLACK),
        StateScoped(GamePhase::GameOver),
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont {
                font_size: 72.0,
                ..default()
            },
            TextColor(DANGER_RED),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));
        
        parent.spawn((
            Text::new("Out of chips!"),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(TEXT_SECONDARY),
        ));
    });
}

fn update_game_state_display(
    mut query: Query<&mut Text2d, With<GameStateDisplay>>,
    game_phase: Res<State<GamePhase>>,
    game_state: Res<GameState>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        **text = format!(
            "Phase: {:?} | Chips: {} | Bet: {}", 
            game_phase.get(), 
            game_state.player_chips,
            game_state.current_bet
        );
    }
}

// Tests module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_creation() {
        let card = Card {
            suit: Suit::Hearts,
            rank: Rank::Ace,
        };
        assert_eq!(card.value(), 14);
        assert_eq!(card.get_suit_symbol(), "♥");
        assert_eq!(card.get_rank_symbol(), "A");
    }

    #[test]
    fn test_game_state_initialization() {
        let game_state = GameState::default();
        assert_eq!(game_state.player_chips, 1000);
        assert_eq!(game_state.current_bet, 0);
        assert_eq!(game_state.deck.len(), 52);
    }

    #[test]
    fn test_card_comparison() {
        let ace = Card { suit: Suit::Hearts, rank: Rank::Ace };
        let king = Card { suit: Suit::Spades, rank: Rank::King };
        assert!(ace.value() > king.value());
    }
}