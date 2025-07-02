# Casino War Tutorial - Part 2: Cards and Betting Interface

## What We're Building in Part 2

Building on Part 1's foundation, we'll add:
1. Visual card representation
2. Betting interface with chip selection
3. Card dealing animations
4. Game table layout
5. Chip management system

## Understanding Visual Card Design

### The Card Rendering Problem

Imagine you're explaining cards to someone who has never seen them. We need to display cards that can:
- Show face up (display suit and rank)
- Show face down (card back)
- Animate between positions
- Flip between face up/down

Let's think about this like building physical cards:
1. **Card Shape**: A rounded rectangle
2. **Card Face**: White background with suit/rank
3. **Card Back**: Decorative pattern
4. **Card State**: Face up or face down

In programming terms, this is a **composition problem**. We don't have one monolithic "Card" - we have multiple visual elements that combine to create the appearance of a card. This is where Bevy's Entity-Component System shines!

## Section 1: Card Rendering Components

First, let's extend our card system with visual components. Think of this like adding "behaviors" to our cards:

```rust
// Visual card components
#[derive(Component)]
struct CardVisual {
    face_up: bool,
    target_position: Vec3,  // Where the card should move to
}
```

**Rust Concept: The `#[derive(Component)]` Attribute**
- `#[derive(...)]` is a **procedural macro** - it generates code at compile time
- `Component` is a trait (like an interface) that marks this struct as a Bevy component
- This is Rust's way of saying "CardVisual can be attached to entities"
- Think of it like a label maker that says "this data can stick to game objects"

```rust
// Card position states
#[derive(Component, Debug, Clone, Copy, PartialEq)]
enum CardPosition {
    Deck,           // In the deck (hidden)
    PlayerHand,     // Player's card position
    DealerHand,     // Dealer's card position
    Discard,        // Used cards
}
```

**Rust Concepts Unpacked:**
- `enum` creates a type that can be ONE of several variants
- `Debug` allows us to print this enum with `{:?}` format
- `Clone` lets us make copies with `.clone()`
- `Copy` means assignment copies the value (like numbers) instead of moving it
- `PartialEq` enables `==` comparison between CardPositions

**Why use an enum here?** A card can only be in ONE position at a time. This is **type safety** - the compiler prevents impossible states like a card being in both the deck AND the player's hand.

```rust
// Animation component
#[derive(Component)]
struct CardAnimation {
    start_pos: Vec3,
    end_pos: Vec3,
    start_rotation: Quat,
    end_rotation: Quat,
    timer: Timer,
}
```

**Bevy Types Explained:**
- `Vec3`: A 3D vector (x, y, z coordinates). Even in 2D, we use z for layering
- `Quat`: Quaternion - a mathematical way to represent rotation without gimbal lock
- `Timer`: Bevy's built-in time tracking struct

```rust
// Constants for card layout
const CARD_WIDTH: f32 = 80.0;
const CARD_HEIGHT: f32 = 120.0;
const CARD_Z_BASE: f32 = 0.0;
const CARD_Z_INCREMENT: f32 = 0.1;  // Stack cards slightly

// Table positions
const DECK_POSITION: Vec3 = Vec3::new(-400.0, 0.0, 0.0);
const PLAYER_CARD_POSITION: Vec3 = Vec3::new(0.0, -200.0, 1.0);
const DEALER_CARD_POSITION: Vec3 = Vec3::new(0.0, 200.0, 1.0);
```

**Rust Concept: Constants vs Variables**
- `const`: Computed at compile time, inlined everywhere they're used
- Must have explicit types (`f32`, `Vec3`)
- Naming convention: `SCREAMING_SNAKE_CASE`
- `Vec3::new()` is a **const fn** - can be called at compile time

### Understanding the Design

**Why these components?**
- `CardVisual`: Tracks display state and movement target
- `CardPosition`: Semantic position for game logic
- `CardAnimation`: Smooth movement between positions

**Z-ordering**: Cards use different Z values to ensure proper layering. Higher Z values appear on top. This is like stacking transparent sheets - higher numbers are "closer" to the camera.

## Section 2: Creating Card Sprites

Now let's create the visual representation. We'll add methods to our Card type:

```rust
// Card colors and styling
const CARD_BACKGROUND: Color = Color::srgb(0.95, 0.95, 0.95);
const CARD_BACK_COLOR: Color = Color::srgb(0.2, 0.3, 0.6);
const HEART_DIAMOND_COLOR: Color = Color::srgb(0.8, 0.1, 0.1);
const CLUB_SPADE_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
```

**Bevy Color System:**
- `Color::srgb()` creates colors in sRGB color space (what monitors display)
- Values range from 0.0 to 1.0 (not 0-255 like in CSS)
- sRGB is gamma-corrected for human perception

```rust
impl Card {
    fn get_suit_symbol(&self) -> &'static str {
        match self.suit {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        }
    }
```

**Rust Concepts Here:**
- `impl Card` creates an **implementation block** - methods for the Card type
- `&self` is an **immutable borrow** - we can read but not modify the card
- `-> &'static str` returns a **string slice with static lifetime**
- `'static` means this string lives for the entire program (it's compiled in)
- `match` is **exhaustive pattern matching** - must handle ALL enum variants

**Why `&'static str` instead of `String`?**
- These symbols never change, so we return references to compile-time constants
- No heap allocation needed - more efficient!

```rust
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
```

**Pattern Matching Excellence:**
The compiler ensures we handle EVERY rank. If we add a new rank to the enum, this code won't compile until we handle it. This is **exhaustiveness checking** - a powerful feature that prevents bugs.

```rust
    fn get_color(&self) -> Color {
        match self.suit {
            Suit::Hearts | Suit::Diamonds => HEART_DIAMOND_COLOR,
            Suit::Clubs | Suit::Spades => CLUB_SPADE_COLOR,
        }
    }
}
```

**Advanced Pattern Matching:**
- `|` means "or" in patterns - matches either pattern
- This groups red suits (Hearts/Diamonds) and black suits (Clubs/Spades)
- Again, exhaustive - we MUST handle all suits

```rust
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
```

**Function Signature Breakdown:**
- `commands: &mut Commands` - mutable reference to Bevy's command queue
- `card: Card` - takes ownership of the card data (it's Copy, so it's copied)
- `-> Entity` - returns the unique ID of the created card

**Why `&mut Commands`?**
Commands is Bevy's way of queuing up changes to the game world. We can't modify the world directly during systems - we queue commands that execute between frames.

```rust
    commands.spawn((
        card,
        position,
        CardVisual {
            face_up,
            target_position: world_pos,
        },
        Transform::from_translation(world_pos),
        Visibility::default(),
    ))
```

**The Tuple Bundle Pattern:**
- `spawn((component1, component2, ...))` creates an entity with multiple components
- This is Bevy 0.16's new pattern - no more bundles!
- Each item in the tuple becomes a component on the entity

**Components Explained:**
- `card` - the Card component (data about suit/rank)
- `position` - where the card logically is
- `CardVisual` - visual state tracking
- `Transform` - position/rotation/scale in world space
- `Visibility` - whether the entity is visible

```rust
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
```

**Hierarchy and Closures:**
- `.with_children(|parent| { ... })` creates child entities
- `|parent|` is a **closure** (anonymous function) that receives a ChildBuilder
- Children inherit their parent's transform (move with the parent)
- `..default()` fills in remaining fields with Default trait values

**Why use `Some(Vec2::new(...))`?**
- `custom_size` is an `Option<Vec2>` - it can be Some(size) or None
- None means "use the texture's natural size"
- We use Some because we're drawing a colored rectangle, not a textured sprite

```rust
        // Card content (face or back)
        if face_up {
            spawn_card_face(parent, card);
        } else {
            spawn_card_back(parent);
        }
    })
    .id()
}
```

**Control Flow and Method Chaining:**
- Simple if/else determines what to spawn
- `.id()` extracts the Entity ID from the EntityCommands
- This ID can be used to reference this card later

```rust
fn spawn_card_face(parent: &mut ChildBuilder, card: Card) {
    // Rank in top-left and bottom-right
    let rank_text = card.get_rank_symbol();
    let suit_text = card.get_suit_symbol();
    let color = card.get_color();
```

**Variable Binding:**
- `let` creates immutable bindings by default
- These method calls happen once and results are reused
- More efficient than calling methods multiple times

```rust
    // Top-left rank
    parent.spawn((
        Text::new(rank_text),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(-CARD_WIDTH/2.0 + 10.0, CARD_HEIGHT/2.0 - 20.0, 0.2),
    ));
```

**Text Rendering in Bevy 0.16:**
- `Text::new()` creates the text component
- `TextFont` configures font properties
- `TextColor` sets the text color
- Text entities need Transform for positioning

**Transform Math Explained:**
- `-CARD_WIDTH/2.0 + 10.0` - start at left edge, move 10 pixels right
- `CARD_HEIGHT/2.0 - 20.0` - start at top edge, move 20 pixels down
- Remember: Y increases upward in Bevy (unlike CSS)

```rust
    // Bottom-right rank (rotated)
    parent.spawn((
        Text::new(rank_text),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(CARD_WIDTH/2.0 - 10.0, -CARD_HEIGHT/2.0 + 20.0, 0.2)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
    ));
}
```

**Method Chaining for Transform:**
- `Transform::from_xyz()` creates a transform at a position
- `.with_rotation()` modifies it with a rotation
- `Quat::from_rotation_z(PI)` rotates 180° around Z axis
- `std::f32::consts::PI` is Rust's built-in π constant

```rust
fn spawn_card_back(parent: &mut ChildBuilder) {
    // Simple pattern for card back
    parent.spawn((
        Sprite {
            custom_size: Some(Vec2::new(CARD_WIDTH - 10.0, CARD_HEIGHT - 10.0)),
            color: CARD_BACK_COLOR,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.2),
    ));
}
```

**Design Choice:**
We make the back slightly smaller (`- 10.0`) to create a border effect. This is a common trick in game development - layering simple shapes to create more complex visuals.
```

### Design Principles Explained

1. **Component Hierarchy**: Cards are entities with child entities for visuals
   - Parent holds game logic components
   - Children hold visual components
   - This separation makes it easy to flip cards or change visuals

2. **Separation of Concerns**: Card data (suit/rank) separate from visual state
   - `Card` component: immutable game data
   - `CardVisual` component: mutable display state
   - `Transform` component: position in world

3. **Flexible Positioning**: `CardPosition` enum allows semantic positioning
   - Game logic uses CardPosition
   - Rendering uses Transform
   - This decoupling allows smooth animations

4. **Color Coding**: Red for hearts/diamonds, black for clubs/spades
   - Matches real-world card conventions
   - Uses const values for consistency

## Section 3: Betting Interface

Now let's create an interactive betting system. Think of this as building a control panel:
```


```rust
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
```

**Marker Components vs Data Components:**
- `ChipButton` holds data (the chip value)
- `BetDisplay`, `ChipDisplay`, `DealButton` are **marker components** (no data)
- Marker components are like tags - they identify entities for queries

**Why use marker components?**
They let us write queries like "find the entity with BetDisplay" without storing redundant data.

```rust
// Betting constants
const CHIP_VALUES: [u32; 5] = [5, 10, 25, 50, 100];
const MIN_BET: u32 = 5;
const MAX_BET: u32 = 500;
```

**Array Type Syntax:**
- `[u32; 5]` means "array of exactly 5 u32 values"
- The size is part of the type - enforced at compile time
- Different from Vec<u32> which can grow/shrink

```rust
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
```

**Bevy UI Layout System:**
- `Node` is the core UI layout component (like a div in HTML)
- Uses Flexbox layout model internally
- `Val` enum represents CSS-like values:
  - `Val::Percent(100.0)` = 100% of parent
  - `Val::Px(20.0)` = 20 pixels
  - `Val::Auto` = automatic sizing

**Key Layout Properties:**
- `position_type: PositionType::Absolute` - removes from normal flow
- `bottom: Val::Px(20.0)` - 20 pixels from bottom edge
- `padding: UiRect::all()` - same padding on all sides
- `justify_content: JustifyContent::SpaceBetween` - space items evenly

**The StateScoped Pattern:**
`StateScoped(GamePhase::Betting)` marks this UI to be cleaned up when we leave the Betting state. This prevents UI from different game phases from overlapping!

```rust
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
```

**Text with Dynamic Content Pattern:**
- Parent `Text` component holds static text ("Chips: $")
- Child `TextSpan` holds dynamic content ("1000")
- This allows updating just the number without recreating the whole text

**The `.with_child()` Method:**
- Singular version of `.with_children()`
- Used when adding exactly one child
- More concise than a closure with one spawn

**Why separate TextFont for parent and child?**
We can style different parts of text differently - here the number is larger (28.0) than the label (24.0).
        
```rust
        // Center - Chip buttons
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            for &value in &CHIP_VALUES {
                spawn_chip_button(parent, value);
            }
        });
```

**Rust Iterator Pattern:**
- `for &value in &CHIP_VALUES` - iterate over array elements
- `&CHIP_VALUES` - borrow the array
- `&value` - pattern match to dereference (copy the u32 value)
- Without `&`, value would be `&u32` (a reference)

**Why `gap: Val::Px(10.0)`?**
This is Flexbox gap - adds 10 pixels between each chip button. Cleaner than margins!
        
        // Right side - Current bet and deal button
        parent.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                gap: Val::Px(10.0),
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

```rust
fn spawn_chip_button(parent: &mut ChildBuilder, value: u32) {
    let color = match value {
        5 => Color::srgb(0.8, 0.2, 0.2),    // Red
        10 => Color::srgb(0.2, 0.2, 0.8),   // Blue
        25 => Color::srgb(0.2, 0.8, 0.2),   // Green
        50 => Color::srgb(0.8, 0.8, 0.2),   // Yellow
        100 => Color::srgb(0.1, 0.1, 0.1),  // Black
        _ => Color::WHITE,
    };
```

**Match Expression as Value:**
- `let color = match ...` - match returns a value
- Each arm must return the same type (Color)
- `_` is the wildcard pattern - catches all other values
- This is exhaustive - compiler ensures all cases handled

**Casino Chip Color Convention:**
These colors match real casino chips! This attention to detail makes games feel authentic.

```rust
    parent.spawn((
        Button,
        ChipButton { value },
        Node {
            width: Val::Px(60.0),
            height: Val::Px(60.0),
            border: UiRect::all(Val::Px(3.0)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::WHITE),
        BackgroundColor(color),
    ))
```

**Making Circular Buttons:**
- `border_radius: BorderRadius::all(Val::Percent(50.0))` - 50% radius = circle
- `width` and `height` must be equal for a perfect circle
- `Button` component makes this entity interactive

**Struct Literal Shorthand:**
`ChipButton { value }` is shorthand for `ChipButton { value: value }` when field and variable names match.

```rust
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
```

**The `format!` Macro:**
- `format!("${}", value)` creates a String
- `{}` is a placeholder replaced by the value
- Like printf but type-safe - compiler checks types match
- Returns owned String (not &str)
```

### UI Design Philosophy

1. **Visual Hierarchy**: Important elements (chips, bet) are larger and colored
2. **Spatial Organization**: Left=resources, Center=actions, Right=current state
3. **Color Psychology**: Green for positive actions (deal), chip colors match casino standards
4. **Responsive Feedback**: Buttons change on hover/click

## Section 4: Betting Logic Systems

Now let's implement the interaction logic. This is where Bevy's query system shines:

```rust
fn handle_chip_buttons(
    mut interaction_query: Query<
        (&Interaction, &ChipButton, &mut BackgroundColor),
        Changed<Interaction>
    >,
    mut game_state: ResMut<GameState>,
    mut bet_display_query: Query<&mut Text, With<BetDisplay>>,
) {
```

**Complex Query Breakdown:**
```rust
Query<
    (&Interaction, &ChipButton, &mut BackgroundColor),  // What we want
    Changed<Interaction>                                 // Filter
>
```
- First type parameter: tuple of components to access
- Second type parameter: filter that limits which entities match
- `Changed<Interaction>` means "only entities whose Interaction changed this frame"

**Reference Types in Queries:**
- `&Interaction` - immutable borrow (read-only)
- `&ChipButton` - immutable borrow (read-only)
- `&mut BackgroundColor` - mutable borrow (can modify)

**System Parameters:**
- `Query<...>` - access entities with specific components
- `ResMut<GameState>` - mutable access to a resource
- `Res<T>` would be immutable access

**Why two separate queries?**
- First query finds chip buttons that were clicked
- Second query finds the bet display to update
- Can't combine because they query different entities!
```rust
    for (interaction, chip_button, mut background) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            // Check if player can afford this bet
            let new_bet = game_state.current_bet + chip_button.value;
            
            if new_bet <= game_state.player_chips && new_bet <= MAX_BET {
                game_state.current_bet = new_bet;
                
                // Update bet display
                if let Ok(mut text) = bet_display_query.get_single_mut() {
                    text.0 = format!("{}", game_state.current_bet);
                }
                
                // Visual feedback
                *background = BackgroundColor(Color::srgb(0.9, 0.9, 0.9));
            }
        }
    }
}
```

**Destructuring in For Loops:**
- `for (interaction, chip_button, mut background) in ...`
- Destructures the tuple returned by the query
- `mut background` - we can mutate only this component

**Dereferencing Enums:**
- `*interaction == Interaction::Pressed`
- `interaction` is `&Interaction` (a reference)
- `*` dereferences to compare values, not references

**Validation Logic:**
- Check player has enough chips
- Check bet doesn't exceed maximum
- Only update if BOTH conditions pass

**The `if let` Pattern:**
```rust
if let Ok(mut text) = bet_display_query.get_single_mut() {
    text.0 = format!("{}", game_state.current_bet);
}
```
- `get_single_mut()` returns `Result<Mut<Text>, QuerySingleError>`
- `if let Ok(mut text)` only executes if successful
- Handles the case where no BetDisplay exists gracefully

**Why `text.0`?**
- In Bevy 0.16, `Text` is a tuple struct
- `.0` accesses the first (and only) field - the string content

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
                    // Transition to dealing phase
                    next_state.set(GamePhase::Dealing);
                    deal_events.send(DealCards);
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

// Update chip display when chips change
fn update_chip_display(
    game_state: Res<GameState>,
    mut chip_display_query: Query<&mut Text, With<ChipDisplay>>,
) {
    if game_state.is_changed() {
        if let Ok(mut text) = chip_display_query.get_single_mut() {
            text.0 = format!("{}", game_state.player_chips);
        }
    }
}
```

### State Management Principles

1. **Validation**: Always check if actions are valid (enough chips, within limits)
2. **Feedback**: Immediate visual response to user actions
3. **State Synchronization**: UI reflects game state changes
4. **Event-Driven**: Actions trigger events for loose coupling

## Section 5: Card Animation System

Let's add smooth card movements. Animation brings our game to life:

```rust
fn animate_cards(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut CardAnimation)>,
) {
    for (entity, mut transform, mut animation) in &mut query {
        animation.timer.tick(time.delta());
```

**Time in Bevy:**
- `Res<Time>` gives access to time information
- `time.delta()` returns time since last frame as Duration
- `.tick()` advances the timer by that duration

**The Animation Loop Pattern:**
1. Update timer
2. Check if finished
3. Either complete or interpolate

```rust
        if animation.timer.finished() {
            // Animation complete
            transform.translation = animation.end_pos;
            transform.rotation = animation.end_rotation;
            commands.entity(entity).remove::<CardAnimation>();
        } else {
            // Interpolate position and rotation
            let t = animation.timer.fraction();
            
            // Smooth easing function (ease-in-out)
            let t = t * t * (3.0 - 2.0 * t);
            
            transform.translation = animation.start_pos.lerp(animation.end_pos, t);
            transform.rotation = animation.start_rotation.slerp(animation.end_rotation, t);
        }
    }
}
```

**Component Removal Pattern:**
- `commands.entity(entity).remove::<CardAnimation>()`
- Removes ONLY the CardAnimation component
- Entity continues to exist with other components
- This is how we "complete" animations

**Easing Function Mathematics:**
```rust
let t = t * t * (3.0 - 2.0 * t);
```
This is the "smoothstep" function:
- Input `t` ranges from 0.0 to 1.0
- Output also 0.0 to 1.0 but with smooth acceleration/deceleration
- Starts slow, speeds up, then slows down
- Creates more natural motion than linear interpolation

**Interpolation Methods:**
- `.lerp()` - Linear intERPolation for positions
- `.slerp()` - Spherical Linear intERPolation for rotations
- Both take a `t` value from 0.0 (start) to 1.0 (end)

```rust
fn deal_cards_system(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut deal_events: EventReader<DealCards>,
) {
    for _ in deal_events.read() {
```

**Event Reading Pattern:**
- `EventReader<DealCards>` reads events sent this frame
- `deal_events.read()` returns an iterator
- `for _` - we don't use event data, just that it occurred
- Events are automatically cleared after being read

```rust
        // Draw cards from deck
        if let (Some(player_card), Some(dealer_card)) = 
            (game_state.draw_card(), game_state.draw_card()) 
        {
```

**Tuple Pattern Matching:**
- Matches only if BOTH cards are Some
- If deck is empty, neither card spawns
- This prevents partial deals

**Why Option<Card>?**
- `draw_card()` might fail (empty deck)
- Rust forces us to handle this case
- No null pointer exceptions!

```rust
            // Spawn player card
            let player_entity = spawn_card(
                &mut commands,
                player_card,
                CardPosition::PlayerHand,
                true,  // Face up
            );
            
            // Add animation from deck to hand
            commands.entity(player_entity).insert(CardAnimation {
                start_pos: DECK_POSITION,
                end_pos: PLAYER_CARD_POSITION,
                start_rotation: Quat::IDENTITY,
                end_rotation: Quat::IDENTITY,
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            });
```

**Entity Command Pattern:**
1. Spawn entity, get its ID
2. Use ID to add more components
3. This is cleaner than spawning everything at once

**Quat::IDENTITY:**
- The identity quaternion = no rotation
- Like multiplying by 1 in regular math
- Default orientation

**Timer Modes:**
- `TimerMode::Once` - runs once then stops
- `TimerMode::Repeating` - loops forever
- We use Once for one-shot animations

```rust
            // Spawn dealer card (face down initially)
            let dealer_entity = spawn_card(
                &mut commands,
                dealer_card,
                CardPosition::DealerHand,
                false,  // Face down
            );
            
            // Add animation with delay
            commands.entity(dealer_entity).insert(CardAnimation {
                start_pos: DECK_POSITION,
                end_pos: DEALER_CARD_POSITION,
                start_rotation: Quat::IDENTITY,
                end_rotation: Quat::IDENTITY,
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            });
        }
    }
}
```

**Game Design Note:**
Dealer card starts face down to build suspense. In Part 3, we'll add the flip animation when comparing cards!
```

### Animation Principles

1. **Easing Functions**: Smooth acceleration/deceleration for natural motion
2. **Timing**: Staggered animations create visual interest
3. **Component-Based**: Animation is just another component
4. **Cleanup**: Remove animation components when complete

## Section 6: Integrating Everything

Let's update our main app structure. This is where all the pieces come together:

```rust
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
```

**Plugin Configuration Pattern:**
- `DefaultPlugins` includes rendering, input, audio, etc.
- `.set()` modifies specific plugins
- We customize only WindowPlugin, keeping others default

**The `.into()` Trait:**
- Converts `&str` to `String` automatically
- Part of Rust's type conversion system
- Makes APIs more ergonomic

```rust
        .init_state::<GamePhase>()
        .init_resource::<GameState>()
        .add_event::<BetPlaced>()
        .add_event::<DealCards>()
        .add_event::<PlayerDecision>()
        .add_event::<RoundResult>()
```

**Initialization Methods:**
- `init_state` - sets up state machine with default state
- `init_resource` - creates resource using Default trait
- `add_event` - registers event type for sending/receiving

**Type Parameters with Turbofish:**
- `::<GamePhase>` is the "turbofish" syntax
- Explicitly specifies generic type parameter
- Needed when Rust can't infer the type

```rust
        // Setup systems
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GamePhase::MainMenu), setup_main_menu)
        .add_systems(OnExit(GamePhase::MainMenu), cleanup_menu)
        .add_systems(OnEnter(GamePhase::Betting), setup_betting_ui)
```

**System Scheduling:**
- `Startup` - runs once when app starts
- `OnEnter(state)` - runs when entering a state
- `OnExit(state)` - runs when leaving a state
- These are **exclusive** systems - won't run in parallel

```rust
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
```

**System Grouping and Conditions:**
- Outer tuple groups all Update systems
- Inner tuple groups betting-related systems
- `.run_if()` conditionally executes systems
- `animate_cards` runs every frame (no condition)

**Why group systems?**
- Better performance (Bevy can optimize)
- Clearer code organization
- Easier to add/remove features

```rust
        // Game logic systems
        .add_systems(Update, (
            deal_cards_system.run_if(on_event::<DealCards>()),
        ))
        .run();
}
```

**Event-Driven Systems:**
- `on_event::<T>()` only runs when event exists
- More efficient than checking every frame
- Decouples event sending from handling

```rust
// Add background to setup
fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2d);
    
    // Table background
    commands.spawn((
        Sprite {
            custom_size: Some(Vec2::new(1280.0, 720.0)),
            color: Color::srgb(0.0, 0.4, 0.2),  // Green felt
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
    ));
}
```

**Camera2d:**
- Simple 2D camera with orthographic projection
- Origin (0,0) is at center of screen
- No configuration needed for basic use

**Background Sprite:**
- Covers entire window (1280x720)
- Z position of -1.0 ensures it's behind everything
- Green color mimics casino table felt

**Why custom_size?**
Without a texture, sprites have no natural size. We must specify dimensions explicitly.
```

## Testing Your Implementation

At this point, you should be able to:

1. **Start the game**: See the main menu
2. **Click Play**: Transition to betting interface
3. **Click chip buttons**: Increase your bet
4. **Click Deal**: See cards animate from deck to positions
5. **See visual feedback**: Buttons respond to hover/click

## Key Concepts Mastered

Let's review what we've learned through building this:

1. **Sprite Hierarchies**: Parent-child relationships for complex visuals
   - Parents hold logical components
   - Children hold visual components
   - Transforms propagate down the hierarchy

2. **UI Interaction**: Button states and event handling
   - `Interaction` enum tracks hover/click states
   - `Changed<T>` filter for efficiency
   - Visual feedback for user actions

3. **Animation Systems**: Time-based interpolation
   - Components hold animation state
   - Systems update based on time
   - Easing functions for natural motion

4. **State-Based UI**: Different UI for different game phases
   - `StateScoped` for automatic cleanup
   - `OnEnter`/`OnExit` for setup/teardown
   - `.run_if()` for conditional systems

5. **Resource Management**: Tracking and displaying game state
   - Resources for global state
   - Components for entity state
   - Events for communication

## Exercises

1. **Add Bet Clearing**: Add a "Clear Bet" button that resets to 0
   ```rust
   // Hint: Create a ClearButton component
   // Set game_state.current_bet = 0
   // Update the bet display
   ```

2. **Implement Max Bet**: Add a button that bets all available chips
   ```rust
   // Hint: game_state.current_bet = game_state.player_chips.min(MAX_BET)
   ```

3. **Card Flip Animation**: Animate dealer card flipping face up
   ```rust
   // Hint: Rotate around Y axis from 0 to PI
   // Swap card face/back at rotation = PI/2
   ```

4. **Sound Effects**: Add sounds for chip placement and card dealing
   ```rust
   // Hint: Use AudioPlayer component
   // commands.spawn((AudioPlayer(sound_handle), PlaybackSettings::DESPAWN));
   ```

5. **Bet Validation**: Disable deal button if bet is invalid
   ```rust
   // Hint: Change button color based on validation
   // Gray out when current_bet < MIN_BET
   ```

## Common Pitfalls and Solutions

1. **Z-Fighting**: Use small Z increments (0.1) to layer elements
   - Cards at z=1.0, card children at z=1.1, 1.2, etc.
   - Consistent spacing prevents flickering

2. **UI Overlap**: Use proper flex layouts and spacing
   - Test at different resolutions
   - Use percentage-based sizing when possible

3. **Animation Timing**: Test different durations for feel
   - 0.3s feels snappy
   - 0.5s feels smooth
   - 1.0s+ feels sluggish

4. **State Leaks**: Use `StateScoped` for automatic cleanup
   - Prevents UI from multiple states appearing together
   - Simpler than manual cleanup

## Debugging Tips

1. **Use Bevy Inspector**: Add `bevy_inspector_egui` to see entity hierarchy
2. **Print Components**: `info!(?entity, ?component)` with `Debug` derive
3. **Slow Animations**: Temporarily increase duration to see issues
4. **Check Z-Order**: Use different colors to verify layering

## Performance Considerations

1. **Query Filters**: Use `Changed<T>` to avoid unnecessary work
2. **Event Systems**: Use `.run_if(on_event::<T>())` for event handlers
3. **Batch Spawning**: Spawn related entities together
4. **Reuse Components**: Modify existing entities instead of despawn/respawn

## Next Steps

In Part 3, we'll implement:
- Card comparison logic
- Win/loss animations
- The "War" mechanic
- Round completion and reset

The foundation we've built makes these additions straightforward! Each concept we've learned - components, systems, events, and animations - will come together to create the complete game experience.
