# Casino War Part 2 - Cards and Betting Interface

This is the complete working code for Part 2 of the Casino War tutorial.

## What's New in Part 2

Building on Part 1, this sample adds:

1. **Visual Card System**
   - Card rendering with suits and ranks
   - Face up/down states
   - Card animations

2. **Betting Interface**
   - Chip buttons (5, 10, 25, 50, 100)
   - Bet display
   - Deal button
   - Chip count display

3. **Animation System**
   - Smooth card movement from deck to table
   - Easing functions for natural motion

4. **Game Flow**
   - Betting phase with chip selection
   - Dealing phase with card animations
   - Proper state transitions

## Running the Sample

```bash
cargo run
```

## Testing

Run the unit tests with:
```bash
cargo test
```

Tests include:
- Card value calculations
- Deck creation and validation
- Bet validation
- Card symbol generation

## Key Features Demonstrated

1. **Sprite Hierarchies**: Cards are composed of multiple sprites
2. **UI Interaction**: Responsive buttons with hover states
3. **Component Animation**: Time-based movement system
4. **State-Based UI**: Different interfaces for different game phases
5. **Resource Updates**: Real-time chip and bet tracking

## Controls

1. Click "Play" on the main menu
2. Click chip buttons to increase your bet
3. Click "DEAL" to see cards animate onto the table

## Visual Elements

- **Green felt** table background
- **Colored chips** matching casino standards
- **Card faces** with proper suit colors (red/black)
- **Card backs** in blue with spade pattern

## Next Steps

Part 3 will add:
- Card comparison logic
- Dealer card reveal animation
- Win/loss determination
- The "War" mechanic for ties
- Chip payout animations