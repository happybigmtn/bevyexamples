#!/bin/bash
echo "=== CASINO WAR PART 6 DEBUG RUN ==="
echo "Watch for:"
echo "1. Game state display in top-left corner"
echo "2. Cards should show rank+suit text overlay"
echo "3. Continue button after round"
echo "4. Debug messages in console"
echo ""
echo "Press Ctrl+C to exit"
echo ""
RUST_BACKTRACE=1 cargo run 2>&1 | grep -E "(DEBUG:|Phase:|ERROR:|panic)" &
cargo run