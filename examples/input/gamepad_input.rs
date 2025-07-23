//! Shows handling of gamepad input, connections, and disconnections.
//!
//! # Gamepad Control Theory Fundamentals
//!
//! Gamepads are complex beasts! They have:
//! - Digital buttons (pressed or not)
//! - Analog triggers (0.0 to 1.0)
//! - Analog sticks (-1.0 to 1.0 in X and Y)
//! It's like having a keyboard, mouse, and joystick all in one device!
//!
//! ## Control Theory: Input Response Curves
//!
//! Gamepad input processing involves several mathematical concepts:
//!
//! ### Dead Zone Mathematics
//! Dead zones prevent "stick drift" - unwanted movement when the stick is centered.
//! This uses a circular dead zone formula:
//! ```text
//! magnitude = sqrt(x² + y²)
//! if magnitude < dead_zone_threshold:
//!     x = 0.0, y = 0.0
//! else:
//!     // Optionally remap to maintain full range
//!     scale = (magnitude - threshold) / (1.0 - threshold)
//!     x *= scale, y *= scale
//! ```
//!
//! ### Response Curves and Sensitivity
//! Different games need different input curves:
//! - **Linear**: Direct 1:1 mapping (racing games)
//! - **Quadratic**: More precision near center (FPS aiming)
//! - **Cubic**: Maximum precision at rest (flight sims)
//! - **Exponential**: Aggressive scaling for fast movement
//!
//! Formula: `output = sign(input) * |input|^curve_power`
//!
//! ### Smoothing and Filtering
//! Raw gamepad input can be noisy. Common filtering techniques:
//! - **Low-pass filter**: Removes high-frequency noise
//! - **Moving average**: Smooths over multiple frames
//! - **Kalman filter**: Predictive smoothing for complex systems
//!
//! ## Human Factors in Gamepad Design
//!
//! ### Ergonomics and Accessibility
//! - **Thumb range of motion**: ~30mm circle, affects stick placement
//! - **Button pressure sensitivity**: 150-500g force typically
//! - **Trigger throw distance**: 6-8mm for optimal control
//! - **Hand size accommodation**: 95th percentile adult male hand
//!
//! ### Cognitive Load Considerations
//! - **Button mapping consistency**: Spatial memory across games
//! - **Action-button distance**: Frequently used actions need easy reach
//! - **Modal vs non-modal controls**: Context-sensitive vs universal
//!
//! ## Performance Optimization Strategies
//!
//! ### Input Polling vs Event-Driven
//! Bevy uses polling for continuous input (movement) and events for discrete actions:
//! - **Polling**: Check state every frame (60-120 Hz)
//! - **Events**: React to state changes (button press/release)
//! - **Hybrid approach**: Combine both for optimal responsiveness
//!
//! ### Memory Efficiency
//! - **Bitfields for buttons**: 32+ buttons in single u32
//! - **Input state caching**: Avoid redundant queries
//! - **Frame-rate independence**: Delta time compensation
//!
//! ### Latency Minimization
//! Input latency budget (60 FPS = 16.67ms frame):
//! - USB polling: 1-8ms
//! - Input processing: <1ms
//! - Game logic: 2-5ms
//! - Rendering: 8-12ms
//! - Display: 1-16ms (depends on refresh rate)
//!
//! Total latency: 13-42ms (target: <20ms for competitive games)
//!
//! ## Real-World Applications
//!
//! ### Industry Practices
//! - **Fighting games**: 1-frame precision, custom input buffers
//! - **Racing games**: Analog precision, force feedback integration
//! - **FPS games**: High-frequency polling, predictive movement
//! - **Platformers**: Coyote time, input buffering for forgiveness
//!
//! ### Adaptive Input Systems
//! Modern games adapt to player behavior:
//! - **Dynamic sensitivity**: Adjust based on player performance
//! - **Accessibility options**: Customizable dead zones, button remapping
//! - **Play style detection**: Aggressive vs defensive player patterns
//!
//! ## Advanced Techniques
//!
//! ### Predictive Input Processing
//! For online games, predict player movement:
//! ```text
//! predicted_position = current_position + velocity * ping_time
//! ```
//!
//! ### Input Buffering Strategies
//! - **Ring buffer**: Fixed-size circular buffer for input history
//! - **Time-based buffering**: Store inputs for X milliseconds
//! - **Action queuing**: Buffer complex move sequences
//!
//! ### Lag Compensation
//! Techniques for maintaining responsiveness:
//! - **Client-side prediction**: Execute immediately, correct later
//! - **Input extrapolation**: Estimate missing inputs during lag
//! - **Rollback networking**: Rewind and replay with corrected inputs
//!
//! ## Common Issues and Solutions
//!
//! ### Input Lag Sources
//! 1. **USB polling rate**: Use 1000Hz for competitive gaming
//! 2. **V-sync enabled**: Causes input buffering, disable for low latency
//! 3. **Frame rate drops**: Inconsistent timing affects feel
//! 4. **Operating system**: Windows has higher baseline latency than Linux
//!
//! ### Stick Drift Mitigation
//! - **Calibration routines**: Let players set their own center points
//! - **Adaptive dead zones**: Automatically adjust over time
//! - **Hardware solutions**: Hall effect sensors vs potentiometers
//!
//! ### Responsiveness Tuning
//! - **Input polling frequency**: Higher than frame rate for best feel
//! - **Delta time compensation**: Smooth movement regardless of FPS
//! - **Acceleration curves**: Natural feeling speed ramp-up

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, gamepad_system)
        .run();
}

// Gamepad input is different - each gamepad is an ENTITY with a Gamepad component!
// This allows multiple controllers and hot-plugging support
fn gamepad_system(
    // Query finds all entities that have a Gamepad component
    // Each connected gamepad gets its own entity automatically
    gamepads: Query<(Entity, &Gamepad)>
) {
    for (entity, gamepad) in &gamepads {
        // DIGITAL BUTTONS - work like keyboard keys
        // "South" is the bottom face button (X on PlayStation, A on Xbox)
        // Using cardinal directions avoids platform-specific names!
        if gamepad.just_pressed(GamepadButton::South) {
            info!("{} just pressed South", entity);
        } else if gamepad.just_released(GamepadButton::South) {
            info!("{} just released South", entity);
        }

        // ANALOG TRIGGERS - return values from 0.0 to 1.0
        // RightTrigger2 is typically the right shoulder trigger
        let right_trigger = gamepad.get(GamepadButton::RightTrigger2).unwrap();
        // DEAD ZONE IMPLEMENTATION - Critical for analog input quality
        // Real analog inputs are never perfectly zero when at rest!
        // This threshold (0.01 = 1% of full range) filters electrical noise
        //
        // CONTROL THEORY: Dead zone mathematics
        // For circular dead zones (analog sticks):
        // magnitude = sqrt(x² + y²)
        // if magnitude < threshold: apply dead zone
        //
        // LINEAR DEAD ZONE (triggers): Simple threshold check
        // PROS: Fast, simple, predictable
        // CONS: Sudden activation, lost precision
        if right_trigger.abs() > 0.01 {
            info!("{} RightTrigger2 value is {}", entity, right_trigger);
        }

        // ANALOG STICKS - return values from -1.0 to 1.0
        // Negative = left/down, Positive = right/up
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        // STICK DRIFT PREVENTION - Essential for professional feel
        // Stick drift occurs when analog sticks don't return to perfect center
        // Causes unwanted movement and poor player experience
        //
        // IMPROVED DEAD ZONE: For production games, consider:
        // 1. Circular dead zones: sqrt(x² + y²) < threshold
        // 2. Per-axis thresholds: Different X/Y sensitivities
        // 3. Graduated response: Smooth scaling beyond dead zone
        // 4. Player-adjustable: Let users set their comfort level
        //
        // HUMAN FACTORS: Typical values
        // - Conservative: 0.15 (15% dead zone)
        // - Standard: 0.10 (10% dead zone)
        // - Competitive: 0.05 (5% dead zone)
        if left_stick_x.abs() > 0.01 {
            info!("{} LeftStickX value is {}", entity, left_stick_x);
        }
        
        // BEVY ARCHITECTURE: Entity-based gamepad design
        // Why entities for gamepads?
        // - Supports multiple controllers (local multiplayer!)
        // - Hot-plugging (connect/disconnect during play)
        // - Each player can have their own gamepad entity
        //
        // RUST PROGRAMMING: Query pattern benefits
        // - Type safety: Can't confuse different input types
        // - Memory efficiency: Only iterate connected gamepads
        // - Parallel processing: Multiple gamepads processed concurrently
        //
        // PERFORMANCE OPTIMIZATION: Input processing efficiency
        // - O(n) iteration where n = connected gamepads (typically 1-4)
        // - No heap allocations during input processing
        // - Cache-friendly memory access patterns
        //
        // REAL-WORLD APPLICATIONS:
        // - Fighting games: Frame-perfect input detection
        // - Racing games: Analog precision for steering/throttle
        // - Platformers: Responsive jump timing
        // - FPS games: Aim assist and acceleration curves
    }
}
