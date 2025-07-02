# Part 3: Game Logic and Card Comparison

Welcome back! In Part 2, we built a beautiful card dealing system with animations. But our cards just sit there - they don't actually *do* anything yet. In this part, we'll breathe life into our game by implementing the core Casino War logic: comparing cards, handling ties, and managing the game flow.

## What Makes a Game Feel Complete?

Think about any card game you've played. There's a rhythm to it - deal, reveal, compare, resolve. Each phase has its own tension and release. In Casino War, that rhythm is:

1. **The Deal** - Anticipation builds as cards fly across the table
2. **The Reveal** - The dealer's card flips dramatically 
3. **The Comparison** - Instant recognition of who won
4. **The Resolution** - Clear feedback and next steps

We're going to implement all of this, and along the way, we'll explore some profound game programming concepts.

## The Architecture of Decision Making

### Events as the Nervous System

In Part 2, we used events to trigger card dealing. Now we'll expand that into a complete nervous system for our game. Here's a mental model: think of events as nerve impulses traveling through your game's body:

```rust
// The brain (player) decides to act
PlayerDecision { go_to_war: true }
    ↓
// The nerves (event system) carry the signal
EventWriter<PlayerDecision>
    ↓
// The muscles (systems) respond
handle_player_decision_system
```

In code, we add several new events:

```rust
#[derive(Event)]
struct CardsDealt;  // Fired when dealing animation completes

#[derive(Event)]
struct RequestCardFlip;  // Request to flip dealer's card

#[derive(Event)]
struct ComparisonComplete {
    outcome: ComparisonOutcome,
}
```

### The State Machine: Your Game's Personality

Every game has a personality - how it flows, when it pauses, how it builds tension. In Bevy, we express this personality through state machines. We've expanded our `GamePhase` enum:

```rust
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GamePhase {
    #[default]
    MainMenu,
    Betting,
    Dealing,     // Cards flying
    Comparing,   // Dealer reveals, tension peaks
    TieDecision, // "Will you go to war?"
    War,         // The dramatic showdown
    RoundComplete, // Resolution and reflection
}
```

Each state is like a scene in a movie - it has its own:
- **Entry** (OnEnter) - Set the stage
- **Action** (Update systems) - The drama unfolds  
- **Exit** (OnExit) - Clean up for the next scene

## The Drama of the Reveal

### Flipping Cards: A Study in Timing

The dealer's card flip is a crucial moment. Too fast, and there's no suspense. Too slow, and players get impatient. We use a two-part animation:

```rust
#[derive(Component)]
struct CardFlipAnimation {
    timer: Timer,
    half_flipped: bool,  // At 90°, we swap from back to face
}
```

The mathematics of a card flip involve quaternions (4D rotations), but here's the beautiful part - Bevy handles the complex math for us:

```rust
// Rotate around Y axis for horizontal flip
let rotation = progress * std::f32::consts::PI;
transform.rotation = Quat::from_rotation_y(rotation);

// Halfway through, swap the card face
if rotation >= std::f32::consts::PI / 2.0 && !flip_anim.half_flipped {
    flip_anim.half_flipped = true;
    visual.face_up = true;
}
```

This creates the illusion that we're seeing the "back" of the card transform into the "front" - but really, we're just swapping the displayed content at the perfect moment.

### The Philosophy of Waiting

Good games know when to make players wait. We don't compare cards immediately - we wait for the flip animation to complete:

```rust
fn compare_cards_system(
    // ... queries ...
    dealer_visual: Query<&CardVisual, (With<DealerCard>, With<ActiveCard>)>,
) {
    // Only compare when dealer card is face up
    if let Ok(visual) = dealer_visual.single() {
        if !visual.face_up {
            return; // Wait for the dramatic reveal
        }
    }
    // Now we can compare...
}
```

This is a profound principle: **games are about managing anticipation**. The moment between action and outcome is where engagement lives.

## The Logic of War

### Comparing Cards: Simplicity in Design

Casino War has perhaps the simplest comparison logic of any card game:

```rust
let outcome = match player_value.cmp(&dealer_value) {
    std::cmp::Ordering::Greater => ComparisonOutcome::PlayerWins,
    std::cmp::Ordering::Less => ComparisonOutcome::DealerWins,
    std::cmp::Ordering::Equal => ComparisonOutcome::Tie,
};
```

But this simplicity is deceptive. The real complexity lies in what happens *after* the comparison.

### The Tie: A Fork in the Road

When cards tie, Casino War offers a choice - a game design pattern called **meaningful decision**:

```rust
struct TieDecisionButton {
    go_to_war: bool,
}
```

This boolean represents two different player psychologies:
- `false` (Surrender): "I'll take my losses and live to fight another day"
- `true` (War): "I'm doubling down on my luck!"

The UI reflects this drama:

```rust
Text::new("Surrender\n(Lose half bet)")  // Safety, but at a cost
Text::new("Go to War!\n(Match bet)")     // Risk for glory
```

### Resource Management: The Heart of Strategy

Watch how we handle the war bet:

```rust
if button.go_to_war {
    if game_state.player_chips >= game_state.current_bet {
        game_state.war_bet = game_state.current_bet;
        game_state.player_chips -= game_state.war_bet;
        
        player_events.write(PlayerDecision { go_to_war: true });
        next_state.set(GamePhase::War);
    }
}
```

This code embodies a fundamental game design principle: **resources create meaningful choices**. Without the chip limit, going to war would be a no-brainer. The constraint creates the decision.

## Entity Relationships: A Dance of Components

### The Active Card Pattern

We introduced a new component pattern - marking cards as "active" during comparison:

```rust
#[derive(Component)]
struct ActiveCard;  // Marks cards currently being compared
```

This is a powerful ECS pattern. Instead of storing card references in game state, we mark entities with components, then query for them:

```rust
// Find all player cards that are active
player_cards: Query<&Card, (With<PlayerCard>, With<ActiveCard>)>
```

This approach is:
- **Flexible**: Cards can be active/inactive dynamically
- **Queryable**: Easy to find all active cards
- **Clean**: No dangling references or complex ownership

### The Lifecycle of a Card

A card in our game goes through a complete lifecycle:

1. **Birth**: Spawned from the deck
2. **Identity**: Tagged as PlayerCard or DealerCard  
3. **Purpose**: Marked as ActiveCard when dealt
4. **Transformation**: Flipped to reveal its face
5. **Judgment**: Compared for victory
6. **Death**: Despawned when the round ends

This lifecycle mirrors how we think about objects in the real world - things have purpose, change over time, and eventually pass on.

## Visual Feedback: The Language of Games

### The Modal Pattern

When presenting the tie decision, we use a modal overlay:

```rust
Node {
    width: Val::Percent(100.0),
    height: Val::Percent(100.0),
    position_type: PositionType::Absolute,
}
BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Semi-transparent
```

This creates a "focus moment" - the rest of the game fades into the background, highlighting the importance of this decision. It's like a spotlight on a stage.

### Color as Communication

Notice how we use color to communicate:

```rust
BorderColor(Color::srgb(0.8, 0.0, 0.0)),  // Red = danger (surrender)
BorderColor(Color::srgb(0.0, 0.8, 0.0)),  // Green = go (war)
```

Colors speak faster than words. Red means "stop and think," green means "go ahead." This taps into universal human associations.

## Testing Game Logic

Our tests now cover game logic, not just data structures:

```rust
#[test]
fn test_comparison_outcomes() {
    assert_eq!(
        match 10u8.cmp(&5u8) {
            std::cmp::Ordering::Greater => ComparisonOutcome::PlayerWins,
            std::cmp::Ordering::Less => ComparisonOutcome::DealerWins,
            std::cmp::Ordering::Equal => ComparisonOutcome::Tie,
        },
        ComparisonOutcome::PlayerWins
    );
}
```

Testing game logic is different from testing regular code. You're testing:
- **Rules**: Does the game follow its own laws?
- **Edge cases**: What happens in unusual situations?
- **Feel**: Does the math create the intended experience?

## Performance Patterns

### Query Efficiency

Notice how we structure our queries:

```rust
dealer_visual: Query<&CardVisual, (With<DealerCard>, With<ActiveCard>)>,
```

This query is highly specific - it only matches entities that have BOTH components. In a game with hundreds of entities, this specificity is crucial for performance.

### System Ordering

The order of our systems matters:

```rust
.add_systems(Update, (
    animate_cards,                    // First: update positions
    animate_card_flips,              // Second: handle flips
    check_dealing_complete           // Third: check if done
        .run_if(in_state(GamePhase::Dealing)),
))
```

This is like a assembly line - each system does its job in order, passing the results to the next.

## What's Next?

We've built the core game loop - cards are dealt, compared, and ties are resolved. But we haven't implemented the actual "war" yet! That's where things get really interesting.

In Part 4, we'll:
- Implement the war sequence (multiple cards, burn cards)
- Add win/loss animations and juice
- Create a complete game loop with proper betting resolution
- Polish the experience with particle effects and sound

The foundation is solid. Now let's build something spectacular on top of it!

## Key Takeaways

1. **Events are communication** - They let parts of your game talk without tight coupling
2. **States create rhythm** - Each game phase has its own pace and purpose
3. **Timing is gameplay** - When things happen is as important as what happens
4. **Components are flexible markers** - Use them to tag entities dynamically
5. **Visual feedback teaches rules** - Players learn by seeing, not reading

Remember: A game is more than its rules. It's the feeling those rules create. Every line of code should serve that feeling.