//! Prints mouse button events.
//!
//! # Mouse Input Control Theory and Implementation
//!
//! The mouse is more complex than keyboard - it has buttons AND motion!
//! Think of it as a device with both discrete inputs (clicks) and continuous
//! inputs (movement). It's like having both digital and analog sensors.
//!
//! ## Control Theory: Hybrid Input Device Design
//!
//! ### Dual Input Nature
//! Mice combine two fundamentally different input types:
//! - **Discrete inputs**: Buttons (digital on/off states)
//! - **Continuous inputs**: Movement (analog X/Y deltas)
//! - **Scroll wheels**: Discrete steps but analog-feeling
//!
//! ### Mouse Movement Mathematics
//! Mouse movement is reported as relative deltas, not absolute positions:
//! ```text
//! // Raw mouse input (counts from sensor)
//! raw_delta_x, raw_delta_y = mouse_sensor_reading()
//!
//! // Convert to screen space (pixels)
//! pixel_delta_x = raw_delta_x * sensitivity_x * dpi_scaling
//! pixel_delta_y = raw_delta_y * sensitivity_y * dpi_scaling
//!
//! // Apply acceleration curve (optional)
//! if (mouse_speed > threshold) {
//!     acceleration_factor = 1.0 + (mouse_speed - threshold) * acceleration_rate
//!     pixel_delta *= acceleration_factor
//! }
//! ```
//!
//! ### Sensitivity and DPI Relationships
//! - **DPI (Dots Per Inch)**: Hardware sensor resolution
//! - **Sensitivity**: Software multiplier for DPI
//! - **eDPI (Effective DPI)**: DPI × Sensitivity = actual responsiveness
//! - **360° distance**: Physical mouse movement for full turn in FPS
//!
//! Formula: `360_distance = (360° × screen_width) / (eDPI × field_of_view)`
//!
//! ## Human Factors in Mouse Design
//!
//! ### Ergonomics and Biomechanics
//! - **Grip styles**: Palm, claw, fingertip (affects precision vs speed)
//! - **Hand size**: 17-21cm typical adult hand length
//! - **Wrist vs arm movement**: Different precision ranges
//!   * Wrist: High precision, small movements (1-3cm)
//!   * Arm: Low precision, large movements (10-30cm)
//!
//! ### Fitts's Law for UI Design
//! Time to reach target = a + b × log₂(distance/width + 1)
//! - Larger targets are faster to click
//! - Closer targets are faster to reach
//! - Corner/edge targets are effectively infinite size
//!
//! ### Click Timing and Double-Clicks
//! - **Single click**: Immediate response (0ms delay)
//! - **Double click**: OS-dependent timing (200-500ms window)
//! - **Click-and-drag**: Threshold distance before drag mode
//! - **Context menus**: Right-click delay (0-200ms)
//!
//! ## Performance Optimization Strategies
//!
//! ### Input Polling vs Event-Driven
//! Mouse input uses hybrid approach:
//! - **Buttons**: Event-driven (press/release notifications)
//! - **Movement**: Accumulated deltas (multiple OS events per frame)
//! - **Scroll**: Event-driven with accumulation
//!
//! ### Memory Efficiency
//! Mouse state storage:
//! - **Button states**: 3-8 bits (typically 3-5 buttons)
//! - **Position**: 2 × f32 (8 bytes for X,Y)
//! - **Delta accumulation**: 2 × f32 (8 bytes)
//! - **Scroll state**: 2 × f32 (8 bytes for X,Y scroll)
//! Total: ~24 bytes per frame
//!
//! ### Latency Optimization
//! Mouse input latency sources:
//! - **USB polling**: 1-8ms (125Hz to 1000Hz)
//! - **OS processing**: 1-3ms (varies by system)
//! - **Sensor delay**: 1-5ms (varies by technology)
//! - **Game processing**: <1ms (should be minimal)
//! - **Display lag**: 1-50ms (varies by monitor)
//!
//! Total target: <10ms for competitive gaming
//!
//! ## Real-World Applications
//!
//! ### Game Genre Adaptations
//!
//! #### First-Person Shooters
//! - **Raw input**: Bypass OS acceleration for consistent feel
//! - **Low sensitivity**: 20-40cm for 360° turn (precision aiming)
//! - **High polling rate**: 1000Hz for smooth tracking
//! - **Pixel-perfect aiming**: Sub-pixel precision for long-range shots
//!
//! #### Real-Time Strategy
//! - **High sensitivity**: Fast screen traversal
//! - **Edge scrolling**: Move view when cursor hits screen edge
//! - **Selection boxes**: Click-and-drag for unit selection
//! - **Context-sensitive cursors**: Different cursors for different actions
//!
//! #### Puzzle/Adventure Games
//! - **Object highlighting**: Hover detection for interactive elements
//! - **Cursor precision**: Fine manipulation of small objects
//! - **Double-click shortcuts**: Quick actions on items
//! - **Inventory management**: Drag-and-drop systems
//!
//! ## Advanced Techniques
//!
//! ### Predictive Mouse Movement
//! For network games, predict cursor position:
//! ```text
//! predicted_x = current_x + velocity_x * ping_time
//! predicted_y = current_y + velocity_y * ping_time
//! ```
//!
//! ### Mouse Acceleration Curves
//! Different curves for different needs:
//! - **Linear**: No acceleration, 1:1 mapping
//! - **Power curve**: acceleration = (speed / threshold)^power
//! - **Sigmoid**: S-curve for natural acceleration feel
//! - **Custom**: Per-game tuning for optimal feel
//!
//! ### Input Smoothing and Filtering
//! - **Exponential smoothing**: smoothed = α × current + (1-α) × previous
//! - **Kalman filtering**: Predictive smoothing with noise estimation
//! - **Savitzky-Golay**: Polynomial smoothing preserving features
//!
//! ## Common Issues and Solutions
//!
//! ### Mouse Acceleration Problems
//! - **OS acceleration**: Inconsistent sensitivity at different speeds
//! - **Solution**: Use raw input APIs, bypass OS processing
//! - **Driver acceleration**: Gaming mice often add their own curves
//! - **Solution**: Disable in mouse software, use hardware settings
//!
//! ### Precision Loss
//! - **Integer quantization**: Mouse reports integer pixel movements
//! - **Solution**: Accumulate sub-pixel movements over time
//! - **Sensitivity too high**: Small movements cause large cursor jumps
//! - **Solution**: Lower sensitivity, use larger mouse movements
//!
//! ### Input Lag Sources
//! 1. **USB polling rate**: Use 1000Hz gaming mice
//! 2. **Wireless latency**: Modern 2.4GHz is ~1ms, Bluetooth is ~5-15ms
//! 3. **V-sync buffering**: Triple buffering adds input lag
//! 4. **Frame rate**: Higher FPS = lower input lag
//!
//! ### Jitter and Noise
//! - **Sensor noise**: Low-quality sensors report false movements
//! - **Surface issues**: Glass/reflective surfaces cause tracking problems
//! - **Interference**: Other wireless devices can disrupt wireless mice
//! - **Solution**: Quality gaming mousepad, proper sensor calibration

use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Notice: two systems in one Update schedule - they run in parallel!
        .add_systems(Update, (mouse_click_system, mouse_move_system))
        .run();
}

// This system prints messages when you press or release the left mouse button:
// Mouse buttons work exactly like keyboard keys - same three states
fn mouse_click_system(
    // ButtonInput<MouseButton> tracks all mouse buttons
    mouse_button_input: Res<ButtonInput<MouseButton>>
) {
    // Currently held down - fires every frame while pressed
    if mouse_button_input.pressed(MouseButton::Left) {
        info!("left mouse currently pressed");
    }

    // Just clicked - fires for exactly one frame
    // Perfect for "click to shoot" or "click to select"
    if mouse_button_input.just_pressed(MouseButton::Left) {
        info!("left mouse just pressed");
    }

    // Just released - fires for exactly one frame
    // Useful for "drag and drop" end detection
    if mouse_button_input.just_released(MouseButton::Left) {
        info!("left mouse just released");
    }
}

// This system tracks mouse movement and scroll wheel
// These are ACCUMULATED values - they sum up all motion since last frame
fn mouse_move_system(
    // AccumulatedMouseMotion - sums all mouse movement in pixels
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    // AccumulatedMouseScroll - sums all scroll wheel motion
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
) {
    // Check if mouse moved at all (Vec2::ZERO = no movement)
    if accumulated_mouse_motion.delta != Vec2::ZERO {
        let delta = accumulated_mouse_motion.delta;
        // Delta = change in position since last frame
        // Positive X = right, Negative X = left
        // Positive Y = down, Negative Y = up (screen coordinates!)
        info!("mouse moved ({}, {})", delta.x, delta.y);
    }
    
    // Check if scroll wheel moved
    if accumulated_mouse_scroll.delta != Vec2::ZERO {
        let delta = accumulated_mouse_scroll.delta;
        // Most mice only have Y scroll (vertical)
        // Some mice have X scroll too (horizontal)
        info!("mouse scrolled ({}, {})", delta.x, delta.y);
    }
    
    // ACCUMULATION MATHEMATICS - Critical for smooth mouse feel
    // Why "Accumulated"? 
    // Between frames, the OS might report multiple mouse events.
    // Bevy adds them all up so you get the total movement, not individual events.
    // This prevents missing fast movements!
    //
    // FRAME RATE INDEPENDENCE: Mouse movement accumulation
    // At 60 FPS (16.67ms frames), mouse might report at 1000Hz (1ms intervals)
    // Each frame could contain 16+ mouse reports that need to be summed
    // Formula: total_delta = Σ(individual_deltas) across frame time
    //
    // PRECISION PRESERVATION: Sub-pixel accuracy
    // Mouse sensors report in "counts" (typically 400-12000 DPI)
    // Multiple small movements can accumulate to significant motion
    // Example: 16 movements of 0.1 pixels = 1.6 pixels total
    //
    // PERFORMANCE OPTIMIZATION: Memory and CPU efficiency
    // AccumulatedMouseMotion structure:
    // - Uses Vec2 (8 bytes) for X,Y delta
    // - Reset to zero each frame after systems process it
    // - No allocations, just addition operations
    // - Cache-friendly: Single memory location per frame
    //
    // CONTROL THEORY: Input frequency vs processing frequency
    // Mouse polling: 125Hz - 1000Hz (gaming mice)
    // Game frame rate: 30Hz - 240Hz (varies by hardware)
    // Accumulation bridges this frequency mismatch
    //
    // RUST PROGRAMMING: Resource system benefits
    // - Shared state without global variables
    // - Thread-safe access through Bevy's resource system
    // - Automatic lifetime management
    // - Type safety prevents accessing wrong input type
    //
    // BEVY ARCHITECTURE: Event vs Resource design
    // Events: Good for discrete actions (clicks)
    // Resources: Good for continuous state (movement)
    // Mouse combines both patterns appropriately
    //
    // REAL-WORLD APPLICATION: Camera control implementation
    // let sensitivity = 0.001; // Radians per pixel
    // let pitch_delta = -accumulated_mouse_motion.delta.y * sensitivity;
    // let yaw_delta = -accumulated_mouse_motion.delta.x * sensitivity;
    // camera_transform.rotate_local_x(pitch_delta);
    // camera_transform.rotate_global_y(yaw_delta);
    //
    // COMMON ISSUES: Missing the accumulation concept
    // ❌ Using individual mouse events: Choppy movement
    // ✅ Using accumulated deltas: Smooth, responsive movement
}
