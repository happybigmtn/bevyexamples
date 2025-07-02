# Casino War Part 6 - Fixed with Text2d

## What's Fixed

1. **Card Display**: Now using proper Bevy 0.16 Text2d for world-space text
   - Cards show clear rank (A, 2-10, J, Q, K) in large text
   - Suit symbols (♠♥♦♣) displayed below rank
   - Corner indicators for traditional card look
   - Proper color coding (red for hearts/diamonds, gray for clubs/spades)

2. **Complete Game Loop**:
   - Main Menu → Click "PLAY" to start
   - Betting Phase → Click chip buttons ($5, $10, $25, $50, $100) to place bet
   - Click "ENGAGE" to deal cards
   - Cards animate from deck position to play areas
   - Dealer card flips after a delay
   - Result displayed clearly ("YOU WIN!", "DEALER WINS", "TIE")
   - "CONTINUE" button appears to play next round
   - Returns to betting phase for continuous play

3. **UI Elements**:
   - Debug display in top-left shows: Phase | Chips | Bet
   - McLaren-inspired design with orange/black/aluminum colors
   - All UI uses proper Bevy 0.16 patterns (no deprecated bundles)

## How to Run

```bash
cargo run
```

## Expected Behavior

1. **Start**: Black screen with "CASINO WAR" title and orange "PLAY" button
2. **Click Play**: Bottom UI appears with chip buttons and current bet
3. **Click Chips**: Bet amount increases (up to $500 or your chip total)
4. **Click ENGAGE**: Two cards dealt with smooth animation
5. **Cards Show**: 
   - Player card (bottom) shows rank and suit immediately
   - Dealer card (top) starts face down, then flips
6. **Result**: Large text shows who won
7. **Continue**: Click button to play another round

## Key Code Changes

- Replaced UI Text components with Text2d for card display
- Removed Anchor import (not needed)
- Fixed all deprecated API calls
- Cards now properly show their values using world-space text

The game is now fully functional with visible cards and a complete game loop!