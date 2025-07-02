# Casino War Part 1 - Sample Code

This is the complete working code for Part 1 of the Casino War tutorial.

## What's Implemented

- Basic game structure with states
- Card representation (Suit, Rank, Card)
- Game state resource with deck management
- Main menu with clickable Play button
- Event system setup
- State transitions

## Running the Sample

```bash
cargo run
```

You should see:
1. A main menu with "Casino War" title
2. A clickable "Play" button that changes color on hover
3. Console output when clicking Play (transitions to Betting state)

## Key Learning Points from Part 1

1. **Bevy 0.16 UI Pattern**: Direct component spawning without NodeBundle
2. **State Management**: Using `States` derive and `NextState` for transitions
3. **Event System**: Defining custom events for game flow
4. **Component Markers**: Using empty components to identify entities
5. **Resource Pattern**: Global game state as a resource

## Next Steps

Part 2 will add:
- Card rendering
- Betting interface
- Card dealing animations
- Game logic implementation