//! Demonstrates handling a key press/release.
//!
//! # Keyboard Input Control Theory
//!
//! Input handling is how games sense the outside world - like a nervous system!
//! This example shows the three states of a button: just pressed (the moment of contact),
//! pressed (continuous holding), and just released (the moment of letting go).
//! It's like the difference between a light switch flipping and a light being on.
//!
//! ## Control Theory: Digital Input Processing
//!
//! ### State Machine Fundamentals
//! Keyboard keys follow a simple state machine:
//! ```text
//! [Released] --keydown--> [JustPressed] --frame--> [Pressed] --keyup--> [JustReleased] --frame--> [Released]
//! ```
//!
//! ### Timing and Frame Rate Independence
//! - **JustPressed/JustReleased**: Single frame events, frame-rate independent
//! - **Pressed**: Duration-based, needs delta time for consistent behavior
//! - **Input buffering**: Store inputs longer than one frame for forgiveness
//!
//! ### Response Curves for Digital Input
//! Even digital inputs can have "curves" through timing:
//! - **Initial delay**: Time before autorepeat starts (OS dependent)
//! - **Repeat rate**: How fast keys repeat when held (OS dependent) 
//! - **Debouncing**: Ignore rapid on/off cycles from mechanical switches
//!
//! ## Human Factors in Keyboard Design
//!
//! ### Ergonomics and Typing Mechanics
//! - **Key travel distance**: 2-4mm for tactile feedback
//! - **Actuation force**: 45-80g for different switch types
//! - **Actuation point**: 50-70% of total travel
//! - **Reset point**: Where key registers as released
//!
//! ### Cognitive Load and Layout
//! - **QWERTY familiarity**: Muscle memory from typing
//! - **Gaming layouts**: WASD, arrow keys, spacebar prominence
//! - **Hand positioning**: Left hand movement, right hand actions
//! - **Modifier keys**: Shift, Ctrl, Alt for action multiplexing
//!
//! ### Accessibility Considerations
//! - **Sticky keys**: Hold modifiers without continuous pressure
//! - **Slow keys**: Ignore brief presses (accidental contact)
//! - **Bounce keys**: Debounce rapid repeated presses
//! - **Key remapping**: Custom layouts for different abilities
//!
//! ## Performance Optimization Strategies
//!
//! ### Input Polling vs Events
//! - **Polling**: Check state every frame (consistent timing)
//! - **Events**: React to OS notifications (lower latency)
//! - **Hybrid approach**: Events for actions, polling for movement
//!
//! ### Memory Efficiency
//! - **Bitfield storage**: 256 keys fit in 32 bytes (256 bits)
//! - **State tracking**: Previous frame state for change detection
//! - **Event buffering**: Ring buffer for input history
//!
//! ### Latency Optimization
//! Keyboard input latency sources:
//! - **USB polling**: 1-8ms (125Hz to 1000Hz)
//! - **OS processing**: 1-5ms (depends on system load)
//! - **Game processing**: <1ms (should be minimal)
//! - **Display lag**: 1-50ms (varies by monitor)
//!
//! Target total latency: <20ms for competitive gaming
//!
//! ## Real-World Applications
//!
//! ### Game Genre Patterns
//!
//! #### Fighting Games
//! - **Frame-perfect inputs**: 16.67ms windows at 60 FPS
//! - **Input buffering**: 3-5 frame windows for complex moves
//! - **Negative edge**: Actions on key release, not press
//!
//! #### First-Person Shooters
//! - **Movement keys**: Continuous polling for smooth motion
//! - **Action keys**: Event-driven for weapons/abilities
//! - **Key binding systems**: Customizable layouts per weapon
//!
//! #### Real-Time Strategy
//! - **Hotkey systems**: Quick access to commands
//! - **Modifier combinations**: Ctrl+click, Shift+click for batch operations
//! - **Command queuing**: Hold Shift to queue multiple orders
//!
//! #### Racing Games
//! - **Binary throttle**: On/off acceleration (not ideal)
//! - **Combined inputs**: Multiple keys for gradual control
//! - **Handbrake timing**: Precise timing for drift initiation
//!
//! ## Advanced Techniques
//!
//! ### Input Prediction and Compensation
//! ```text
//! // Predict where player will be based on current input
//! predicted_position = current_position + velocity * prediction_time
//! 
//! // Compensate for network lag in multiplayer
//! client_time = server_time - round_trip_time / 2
//! ```
//!
//! ### Input Buffering Strategies
//! - **Time-based buffering**: Keep inputs for X milliseconds
//! - **Frame-based buffering**: Keep inputs for X frames
//! - **Action-specific buffers**: Different buffer sizes per action
//!
//! ### Macro Detection and Prevention
//! - **Timing analysis**: Human timing has natural variation
//! - **Pattern recognition**: Detect inhuman perfect timing
//! - **Rate limiting**: Prevent impossible action speeds
//!
//! ## Common Issues and Solutions
//!
//! ### Input Lag Problems
//! 1. **V-sync causing input buffering**: Disable for competitive play
//! 2. **Frame rate drops**: Causes inconsistent input timing
//! 3. **Background processes**: Other apps stealing CPU time
//! 4. **USB oversubscription**: Too many devices on one controller
//!
//! ### Key Ghosting and N-Key Rollover
//! - **Ghosting**: Phantom key presses from electrical interference
//! - **Blocking**: Missing key presses when too many held
//! - **N-Key Rollover**: Ability to register N simultaneous keys
//! - **Solution**: Use gaming keyboards with proper circuitry
//!
//! ### Operating System Issues
//! - **Key repeat delay**: OS setting affects held key behavior
//! - **Input method interference**: Language IMEs can block inputs
//! - **Focus stealing**: Other apps grabbing input focus
//! - **Permission issues**: Some keys require elevated privileges
//!
//! ### Frame Rate Dependencies
//! - **Fixed timestep**: Use for physics, independent of frame rate
//! - **Variable timestep**: Use for rendering, smooth visuals
//! - **Input timestep**: Often independent of both
//!
//! ## Rust Programming Patterns
//!
//! ### Type Safety Benefits
//! - **KeyCode enum**: Compile-time guarantee of valid keys
//! - **ButtonInput<T>**: Generic over input types (keyboard, mouse, gamepad)
//! - **Resource system**: Shared input state without global variables
//!
//! ### Memory Safety
//! - **No dangling pointers**: Rust's ownership prevents input corruption
//! - **Thread safety**: Send/Sync traits ensure safe concurrent access
//! - **Zero-cost abstractions**: High-level APIs with no runtime overhead
//!
//! ### Error Handling
//! ```rust
//! // Rust's Result type for fallible input operations
//! match keyboard_input.get_just_pressed().next() {
//!     Some(key) => handle_key_press(key),
//!     None => (), // No keys pressed this frame
//! }
//! ```

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, keyboard_input_system)
        .run();
}

/// This system prints 'A' key state
/// 
/// ButtonInput<KeyCode> is Bevy's keyboard state tracker. It remembers:
/// - What keys are currently down
/// - What keys were just pressed this frame
/// - What keys were just released this frame
fn keyboard_input_system(
    // ButtonInput is a resource that tracks all keyboard state
    // Think of it as a dashboard showing every key's status
    keyboard_input: Res<ButtonInput<KeyCode>>
) {
    // pressed() - Is the key currently down?
    // This is true for EVERY frame while the key is held
    // Like asking "Is the light on?"
    if keyboard_input.pressed(KeyCode::KeyA) {
        info!("'A' currently pressed");
    }

    // just_pressed() - Was the key pressed down THIS FRAME?
    // This is true for EXACTLY ONE frame when the key goes down
    // Like detecting the exact moment someone flips a switch ON
    if keyboard_input.just_pressed(KeyCode::KeyA) {
        info!("'A' just pressed");
    }
    
    // just_released() - Was the key released THIS FRAME?
    // This is true for EXACTLY ONE frame when the key goes up
    // Like detecting the exact moment someone flips a switch OFF
    if keyboard_input.just_released(KeyCode::KeyA) {
        info!("'A' just released");
    }
    
    // CONTROL THEORY: Why these distinctions matter
    // - Use pressed() for continuous actions (moving while key held)
    //   * Physics integration: velocity += acceleration * delta_time
    //   * Smooth movement: position += velocity * delta_time
    // - Use just_pressed() for one-time actions (jumping, shooting)
    //   * State transitions: grounded -> airborne
    //   * Resource consumption: ammo--, health++
    // - Use just_released() for actions on key release (charging then releasing)
    //   * Charge mechanics: bow draw, magic spells
    //   * Timing-based actions: golf swing power
    //
    // PERFORMANCE OPTIMIZATION: Input processing efficiency
    // ButtonInput<KeyCode> uses bitfields internally:
    // - Current state: 256 bits (32 bytes) for all possible keys
    // - Previous state: 256 bits for change detection
    // - Just pressed: current AND NOT previous
    // - Just released: NOT current AND previous
    // - Currently pressed: current
    //
    // RUST PROGRAMMING: Zero-cost abstractions
    // These method calls compile to simple bitwise operations:
    // - pressed(key): (current_state & key_bit) != 0
    // - just_pressed(key): (current_state & !previous_state & key_bit) != 0
    // - just_released(key): (!current_state & previous_state & key_bit) != 0
    //
    // BEVY ARCHITECTURE: System scheduling benefits
    // Input systems run in special early stages:
    // 1. PreUpdate: OS events processed into Bevy resources
    // 2. Update: Game logic reads input state
    // 3. PostUpdate: Input state prepared for next frame
    // This ensures consistent input state within a frame
    //
    // ADVANCED TECHNIQUES: Input buffering example
    // For combo systems or timing-critical games:
    // struct InputBuffer {
    //     buffer: VecDeque<(KeyCode, f32)>, // key, timestamp
    //     window: f32, // how long to keep inputs
    // }
    // 
    // This allows "coyote time" - accepting jumps slightly after leaving platform
}
