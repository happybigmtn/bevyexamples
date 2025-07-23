//! Displays touch presses, releases, and cancels.
//!
//! # Touch Input Control Theory and Multi-Touch Processing
//!
//! Touch input is fundamentally different from mouse - it supports MULTIPLE
//! simultaneous touches! Each finger gets its own ID and position.
//! It's like having multiple mice that can appear and disappear at any time.
//! This enables gestures like pinch-to-zoom or multi-finger swipes.
//!
//! ## Control Theory: Multi-Touch Input Processing
//!
//! ### Touch State Machine
//! Each touch point follows this lifecycle:
//! ```text
//! [None] --contact--> [JustPressed] --frame--> [Active] --lift--> [JustReleased] --frame--> [None]
//!                                               |
//!                                        --interrupt--> [JustCanceled] --frame--> [None]
//! ```
//!
//! ### Touch ID Management
//! - **Unique IDs**: Each touch gets a persistent identifier
//! - **ID recycling**: Released IDs can be reused for new touches
//! - **Maximum touches**: Hardware typically supports 5-10 simultaneous touches
//! - **Tracking continuity**: Same finger keeps same ID throughout gesture
//!
//! ### Multi-Touch Mathematics
//!
//! #### Distance Calculation (for pinch gestures)
//! ```text
//! distance = sqrt((x1 - x2)² + (y1 - y2)²)
//! scale_factor = current_distance / initial_distance
//! ```
//!
//! #### Centroid Calculation (for rotation/pan gestures)
//! ```text
//! centroid_x = (x1 + x2 + ... + xn) / n
//! centroid_y = (y1 + y2 + ... + yn) / n
//! ```
//!
//! #### Rotation Angle (for two-finger rotation)
//! ```text
//! angle = atan2(y2 - y1, x2 - x1)
//! rotation_delta = current_angle - previous_angle
//! ```
//!
//! ## Human Factors in Touch Interface Design
//!
//! ### Finger Size and Touch Targets
//! - **Average fingertip**: 10-14mm diameter
//! - **Minimum touch target**: 44pt (11mm) for accessibility
//! - **Comfortable target**: 48-60pt (12-15mm)
//! - **Touch spread**: Finger contact area is elliptical, not circular
//!
//! ### Touch Accuracy and Parallax
//! - **Parallax error**: Finger obscures target, user aims higher
//! - **Contact centroid**: Actual touch point vs intended target
//! - **Calibration offset**: Typically 3-5mm above fingertip center
//! - **Edge effects**: Less accuracy near screen edges
//!
//! ### Gesture Ergonomics
//! - **Single hand reach**: 75mm radius from thumb base
//! - **Two-handed gestures**: Different for portrait vs landscape
//! - **Thumb vs finger**: Different precision and force characteristics
//! - **Fatigue factors**: Extended pinch gestures cause hand strain
//!
//! ## Performance Optimization Strategies
//!
//! ### Touch Processing Efficiency
//! Touch events can be frequent (120-240Hz on modern devices):
//! - **Event coalescing**: Combine multiple moves into single update
//! - **Predictive tracking**: Estimate touch position between samples
//! - **Gesture recognition**: State machines for complex patterns
//!
//! ### Memory Management
//! Touch data structure per active touch:
//! ```rust
//! struct TouchPoint {
//!     id: u64,              // 8 bytes - unique identifier
//!     position: Vec2,       // 8 bytes - current X,Y coordinates
//!     previous_position: Vec2, // 8 bytes - for velocity calculation
//!     pressure: f32,        // 4 bytes - force (if supported)
//!     timestamp: f64,       // 8 bytes - for gesture timing
//! }
//! // Total: 36 bytes × 10 max touches = 360 bytes
//! ```
//!
//! ### Latency Optimization
//! Touch-to-response latency sources:
//! - **Touchscreen scan**: 8-16ms (60-120Hz scan rate)
//! - **OS processing**: 2-8ms (varies by system load)
//! - **Game processing**: <2ms (gesture recognition)
//! - **Display response**: 1-50ms (varies by display technology)
//!
//! Target total latency: <20ms for responsive feel
//!
//! ## Real-World Applications
//!
//! ### Mobile Game Design Patterns
//!
//! #### Touch-First Controls
//! - **Virtual joysticks**: Dynamic positioning where finger first touches
//! - **Gesture shortcuts**: Swipe patterns for quick actions
//! - **Context-sensitive areas**: Different screen regions for different functions
//! - **Simultaneous input**: Movement + action with different hands
//!
//! #### Tablet Strategy Games
//! - **Pinch-to-zoom**: Map navigation with smooth scaling
//! - **Pan gestures**: Two-finger map scrolling
//! - **Selection rectangles**: Drag to select multiple units
//! - **Rotation gestures**: Rotate camera or objects
//!
//! #### Mobile Puzzles
//! - **Direct manipulation**: Drag pieces with finger
//! - **Multi-touch sorting**: Move multiple objects simultaneously
//! - **Pressure sensitivity**: Harder press for different actions
//! - **Edge gestures**: Swipe from screen edge for menus
//!
//! ## Advanced Techniques
//!
//! ### Gesture Recognition Algorithms
//!
//! #### $1 Recognizer (for custom gestures)
//! 1. **Resample**: Normalize to fixed number of points
//! 2. **Rotate**: Align to reference orientation
//! 3. **Scale**: Normalize to reference size
//! 4. **Translate**: Center on origin
//! 5. **Match**: Compare against template library
//!
//! #### State Machine Approach
//! ```rust
//! enum GestureState {
//!     Idle,
//!     SingleTouch { start_pos: Vec2, start_time: f64 },
//!     Pinch { finger1: u64, finger2: u64, initial_distance: f32 },
//!     Pan { fingers: Vec<u64>, centroid: Vec2 },
//!     Rotate { finger1: u64, finger2: u64, initial_angle: f32 },
//! }
//! ```
//!
//! ### Predictive Touch Tracking
//! For smooth gesture processing:
//! ```text
//! // Linear prediction
//! velocity = (current_position - previous_position) / delta_time
//! predicted_position = current_position + velocity * prediction_time
//!
//! // Kalman filter prediction (more sophisticated)
//! predicted_state = A × current_state + B × control_input + noise
//! ```
//!
//! ### Palm Rejection
//! Techniques to ignore accidental palm contact:
//! - **Size filtering**: Reject touch areas above threshold
//! - **Edge rejection**: Ignore touches near screen edges
//! - **Timeline analysis**: Reject touches that appear during finger gesture
//! - **Pressure analysis**: Palms typically create broader, lower pressure
//!
//! ## Common Issues and Solutions
//!
//! ### Touch Cancellation Problems
//! Touch cancellation occurs when:
//! - **System gestures**: OS intercepts touch (notification panel, home)
//! - **Phone calls**: Incoming calls interrupt touch processing
//! - **Low memory**: OS kills background processes
//! - **Hardware issues**: Electrical interference
//!
//! Solution: Always handle cancellation gracefully:
//! ```rust
//! for touch in touches.iter_just_canceled() {
//!     // Clean up any gesture state for this touch ID
//!     gesture_state.remove(&touch.id());
//!     // Cancel any ongoing actions
//!     cancel_action_for_touch(touch.id());
//! }
//! ```
//!
//! ### Multi-Touch Gesture Conflicts
//! - **Competing recognizers**: Multiple gestures claim same touches
//! - **Solution**: Priority system, exclusive gesture modes
//! - **Gesture masking**: Disable certain gestures during others
//! - **Timeout handling**: Reset gesture state after inactivity
//!
//! ### Performance Issues
//! - **Too many simultaneous touches**: Limit active touch processing
//! - **Complex gesture math**: Cache expensive calculations
//! - **Memory allocations**: Use object pools for touch events
//! - **Update frequency**: Throttle gesture updates to frame rate
//!
//! ### Accessibility Considerations
//! - **Touch accommodations**: Larger targets for motor difficulties
//! - **Hold gestures**: Alternative to precise timing requirements
//! - **Voice alternatives**: Backup for complex gesture sequences
//! - **Switch control**: External switches for users unable to touch directly

use bevy::{input::touch::*, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, touch_system)
        .run();
}

// Touch handling - designed for phones, tablets, and touchscreens
fn touch_system(
    // Touches resource tracks all active touches
    // Unlike mouse, there can be many simultaneous touches!
    touches: Res<Touches>
) {
    // JUST PRESSED - New fingers touching the screen
    for touch in touches.iter_just_pressed() {
        info!(
            "just pressed touch with id: {}, at: {}",
            touch.id(),       // Unique ID for this finger/touch point
            touch.position()  // Screen coordinates where touched
        );
        // Each touch gets a unique ID that persists until released
        // This lets you track individual fingers in multi-touch gestures
    }

    // JUST RELEASED - Fingers lifted off the screen
    for touch in touches.iter_just_released() {
        info!(
            "just released touch with id: {}, at: {}",
            touch.id(),
            touch.position() // Last known position before release
        );
    }

    // JUST CANCELED - System interrupted the touch
    // This happens when:
    // - Phone call comes in
    // - System gesture takes over
    // - Palm rejection triggers
    for touch in touches.iter_just_canceled() {
        info!("canceled touch with id: {}", touch.id());
    }

    // ACTIVE TOUCHES - All fingers currently on screen
    // You can also iterate all current touches and retrieve their state like this:
    for touch in touches.iter() {
        info!("active touch: {touch:?}");
        // You can check if a specific touch was just pressed
        info!("  just_pressed: {}", touches.just_pressed(touch.id()));
        
        // Use cases for multi-touch:
        // - Two fingers = pinch/zoom gesture
        // - Three fingers = swipe between apps
        // - Track each finger's movement independently
    }
    
    // CONTROL THEORY: Touch vs Mouse fundamental differences
    // - Mouse: One pointer, always exists, can hover
    // - Touch: Multiple pointers, appear/disappear, no hover state
    // Design your input handling accordingly!
    //
    // BEVY ARCHITECTURE: Touch ID management
    // Each touch maintains persistent ID throughout its lifecycle:
    // - Enables tracking individual fingers through complex gestures
    // - Allows gesture recognition across multiple frames
    // - Supports multi-touch game mechanics (two-thumb movement)
    //
    // PERFORMANCE OPTIMIZATION: Touch processing efficiency
    // touches.iter() family methods use efficient iteration:
    // - No heap allocations during iteration
    // - Cache-friendly access patterns
    // - Early termination when specific touch found
    //
    // RUST PROGRAMMING: Iterator patterns
    // Bevy provides different iterator types for different use cases:
    // - iter(): All current touches (most common)
    // - iter_just_pressed(): New touches this frame
    // - iter_just_released(): Ended touches this frame
    // - iter_just_canceled(): Interrupted touches this frame
    //
    // GESTURE RECOGNITION: Basic patterns
    // With this foundation, you can build:
    // - Tap detection: just_pressed -> just_released quickly
    // - Long press: pressed for extended duration
    // - Drag: movement while pressed
    // - Pinch: two touches moving toward/away from each other
    // - Rotation: two touches rotating around centroid
    //
    // REAL-WORLD APPLICATION: Mobile game controls
    // Common patterns for touch-first games:
    // 1. Virtual joystick: First touch creates stick at that position
    // 2. Action buttons: Fixed screen regions for different actions
    // 3. Gesture shortcuts: Swipe patterns for quick commands
    // 4. Direct manipulation: Touch and drag game objects
    //
    // HUMAN FACTORS: Touch interface design
    // - Minimum target size: 44pt (11mm) for accessibility
    // - Touch feedback: Visual/haptic confirmation of contact
    // - Error recovery: Easy to undo accidental touches
    // - One-handed operation: Reachable zones for thumb
    //
    // COMMON ISSUES: Touch handling mistakes
    // ❌ Assuming single touch: Breaks with accidental palm contact
    // ❌ Ignoring cancellation: Leaves game in inconsistent state
    // ❌ Fixed UI positions: Poor ergonomics on different devices
    // ✅ Handle all touch states: Robust against interruptions
    // ✅ Adaptive UI: Position controls based on hand position
}
