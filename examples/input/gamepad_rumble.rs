//! Shows how to trigger force-feedback, making gamepads rumble when buttons are
//! pressed.
//!
//! # Haptic Feedback Control Theory and Implementation
//!
//! Force feedback through gamepad rumble is a form of tactile communication that
//! enhances player immersion and provides important gameplay information. This
//! example demonstrates the dual-motor rumble system found in modern controllers.
//!
//! ## Control Theory: Haptic Feedback Systems
//!
//! ### Dual-Motor Architecture
//! Modern gamepads use two different motors for varied tactile sensations:
//! - **Strong motor (low-frequency)**: Large eccentric weight motor
//!   * Frequency range: 10-200 Hz
//!   * Creates deep, powerful rumble sensations
//!   * Used for explosions, heavy impacts, engine vibrations
//! - **Weak motor (high-frequency)**: Small linear actuator or vibration motor
//!   * Frequency range: 100-1000+ Hz
//!   * Creates sharp, precise tactile feedback
//!   * Used for gunshots, UI feedback, texture simulation
//!
//! ### Psychophysics of Touch
//! Human tactile perception follows specific principles:
//!
//! #### Weber's Law for Vibration
//! ```text
//! ΔI/I = k (constant)
//! ```
//! Where ΔI is the just-noticeable difference in intensity,
//! I is the base intensity, and k ≈ 0.07 for vibration.
//!
//! #### Frequency Response Curve
//! Human skin sensitivity varies by frequency:
//! - **Peak sensitivity**: 200-300 Hz (Pacinian corpuscles)
//! - **Low frequency**: 10-50 Hz (Meissner corpuscles)
//! - **Threshold**: ~0.01 mm displacement at optimal frequency
//!
//! #### Tactile Masking
//! Strong vibrations can mask weaker ones:
//! ```text
//! masked_intensity = base_intensity * masking_factor
//! masking_factor = 1.0 - (strong_intensity * overlap_coefficient)
//! ```
//!
//! ## Human Factors in Haptic Design
//!
//! ### Ergonomics and Comfort
//! - **Hand physiology**: Vibration travels through bones and tissues
//! - **Grip pressure**: Firmer grip = better tactile transmission
//! - **Hand size**: Adults vary 17-21cm hand length, affects perception
//! - **Fatigue factors**: Extended vibration causes numbness
//!
//! ### Accessibility Considerations
//! - **Hearing impaired**: Haptics provide audio replacement
//! - **Visual impaired**: Spatial audio confirmation through touch
//! - **Motor disabilities**: May need stronger/longer feedback
//! - **Sensory processing**: Some users are hypersensitive to vibration
//!
//! ### Comfort and Safety Guidelines
//! - **Maximum duration**: <30 seconds continuous at high intensity
//! - **Duty cycle**: 50% on/off ratio for extended use
//! - **Intensity ramping**: Gradual start/stop prevents jarring sensation
//! - **User control**: Always allow disabling/adjusting haptic strength
//!
//! ## Performance Optimization Strategies
//!
//! ### Memory Management
//! Haptic event structure:
//! ```rust
//! struct GamepadRumbleRequest {
//!     gamepad: Entity,              // 8 bytes
//!     intensity: GamepadRumbleIntensity, // 8 bytes (2x f32)
//!     duration: Duration,           // 16 bytes
//!     // Total: ~32 bytes per request
//! }
//! ```
//!
//! ### Timing Precision
//! Haptic feedback timing requirements:
//! - **Causality**: Feedback must occur within 50ms of trigger
//! - **Duration accuracy**: ±10ms for short pulses (<100ms)
//! - **Frequency stability**: ±5% for sustained vibrations
//! - **System latency**: USB polling + OS + hardware = 5-20ms total
//!
//! ### Battery Impact
//! Rumble motors are power-hungry:
//! - **Strong motor**: 200-500mA at full intensity
//! - **Weak motor**: 50-150mA at full intensity
//! - **Battery life impact**: 20-50% reduction with heavy use
//! - **Optimization**: Use minimum effective intensity
//!
//! ## Real-World Applications
//!
//! ### Game Genre Implementations
//!
//! #### First-Person Shooters
//! - **Weapon feedback**: Different patterns per weapon type
//! - **Impact feedback**: Intensity based on damage received
//! - **Reloading cues**: Tactile confirmation of magazine changes
//! - **Environmental effects**: Explosions, vehicle engines
//!
//! #### Racing Games
//! - **Road texture**: High-frequency vibration for surface feel
//! - **Engine rumble**: Low-frequency proportional to RPM
//! - **Collision feedback**: Sharp bursts for impacts
//! - **Gear shifts**: Distinctive tactile signatures
//!
//! #### Platformers
//! - **Jump feedback**: Confirmation of successful actions
//! - **Landing impact**: Intensity based on fall distance
//! - **Power-up collection**: Distinctive positive feedback
//! - **Damage feedback**: Warning patterns for health loss
//!
//! #### Horror Games
//! - **Heartbeat simulation**: Rhythmic low-frequency pulses
//! - **Tension building**: Gradual intensity increases
//! - **Jump scares**: Sudden high-intensity bursts
//! - **Atmospheric effects**: Subtle environmental vibrations
//!
//! ## Advanced Techniques
//!
//! ### Procedural Haptic Generation
//! Generate complex patterns programmatically:
//! ```rust
//! fn generate_heartbeat_pattern(bpm: f32, stress_level: f32) -> Vec<RumbleFrame> {
//!     let interval = 60.0 / bpm; // seconds between beats
//!     let intensity = 0.3 + stress_level * 0.4; // 0.3-0.7 range
//!     vec![
//!         RumbleFrame { duration: 0.1, strong: intensity, weak: 0.0 },
//!         RumbleFrame { duration: 0.05, strong: 0.0, weak: intensity * 0.5 },
//!         RumbleFrame { duration: interval - 0.15, strong: 0.0, weak: 0.0 },
//!     ]
//! }
//! ```
//!
//! ### Audio-Driven Haptics
//! Convert audio signals to tactile feedback:
//! ```text
//! // Low-pass filter for strong motor (bass frequencies)
//! strong_signal = low_pass_filter(audio_signal, cutoff: 200Hz)
//! 
//! // High-pass filter for weak motor (treble frequencies)
//! weak_signal = high_pass_filter(audio_signal, cutoff: 1000Hz)
//! 
//! // Map amplitude to motor intensity
//! strong_intensity = clamp(strong_signal.rms() * gain, 0.0, 1.0)
//! weak_intensity = clamp(weak_signal.rms() * gain, 0.0, 1.0)
//! ```
//!
//! ### Adaptive Haptic Systems
//! Adjust feedback based on player behavior:
//! ```rust
//! struct AdaptiveHaptics {
//!     user_sensitivity: f32,    // Learned from user interactions
//!     recent_intensity: f32,    // Running average of recent feedback
//!     fatigue_factor: f32,      // Reduced sensitivity over time
//! }
//! 
//! impl AdaptiveHaptics {
//!     fn adjust_intensity(&mut self, base_intensity: f32) -> f32 {
//!         let adapted = base_intensity * self.user_sensitivity * self.fatigue_factor;
//!         self.recent_intensity = lerp(self.recent_intensity, adapted, 0.1);
//!         adapted.clamp(0.0, 1.0)
//!     }
//! }
//! ```
//!
//! ## Common Issues and Solutions
//!
//! ### Haptic Fatigue
//! Extended vibration causes desensitization:
//! - **Problem**: Users become numb to feedback over time
//! - **Solution**: Vary patterns, include rest periods
//! - **Detection**: Monitor user response times to haptic cues
//!
//! ### Battery Drain
//! Rumble motors consume significant power:
//! - **Problem**: Controllers die quickly with heavy haptic use
//! - **Solution**: Intensity scaling, smart duty cycling
//! - **User control**: Battery level awareness in haptic intensity
//!
//! ### Cross-Platform Consistency
//! Different controllers have different capabilities:
//! ```rust
//! fn normalize_haptic_intensity(intensity: f32, controller_type: ControllerType) -> f32 {
//!     match controller_type {
//!         ControllerType::Xbox => intensity,  // Reference standard
//!         ControllerType::PlayStation => intensity * 0.8,  // Typically stronger
//!         ControllerType::Switch => intensity * 1.2,  // Typically weaker
//!         ControllerType::Generic => intensity * 0.9,  // Conservative default
//!     }
//! }
//! ```
//!
//! ### Timing Synchronization
//! Keep haptics synchronized with audio/visual:
//! - **Audio offset**: Account for audio buffer latency (20-100ms)
//! - **Visual offset**: Account for display lag (5-50ms)
//! - **Haptic prediction**: Start feedback slightly early to compensate
//!
//! ## Industry Best Practices
//!
//! ### Guidelines from Major Platforms
//! - **Xbox**: Use impulse triggers for directional feedback
//! - **PlayStation**: DualSense adaptive triggers + haptic feedback
//! - **Nintendo**: HD Rumble with precise amplitude control
//! - **Steam**: Trackpad haptics for UI navigation
//!
//! ### Accessibility Standards
//! - **WCAG compliance**: Provide haptic alternatives to audio cues
//! - **User preferences**: Respect system-level accessibility settings
//! - **Intensity scaling**: 0-100% user-adjustable strength
//! - **Pattern library**: Consistent haptic language across games

use bevy::{
    input::gamepad::{Gamepad, GamepadRumbleIntensity, GamepadRumbleRequest},
    prelude::*,
};
use core::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, gamepad_system)
        .run();
}

// HAPTIC FEEDBACK SYSTEM - Demonstrates professional gamepad rumble patterns
// This system showcases the four main types of haptic feedback:
// 1. Strong motor only (deep, powerful sensations)
// 2. Weak motor only (sharp, precise sensations) 
// 3. Dual motor combination (complex, layered feedback)
// 4. Emergency stop (user control and accessibility)
fn gamepad_system(
    // Query all connected gamepads - supports multiple controllers simultaneously
    gamepads: Query<(Entity, &Gamepad)>,
    // Event writer for haptic commands - uses Bevy's efficient event system
    mut rumble_requests: EventWriter<GamepadRumbleRequest>,
) {
    // MULTI-CONTROLLER SUPPORT - Process each connected gamepad independently
    // This allows local multiplayer with individual haptic feedback per player
    for (entity, gamepad) in &gamepads {
        // PERFORMANCE NOTE: This loop is O(n) where n = connected gamepads
        // Typically n = 1-4, so performance impact is negligible
        // Each gamepad can have independent rumble effects running simultaneously
        if gamepad.just_pressed(GamepadButton::North) {
            info!(
                "North face button: strong (low-frequency) with low intensity for rumble for 5 seconds. Press multiple times to increase intensity."
            );
            // STRONG MOTOR DEMONSTRATION - Low-frequency, high-amplitude vibration
            // Strong motor characteristics:
            // - Frequency: 10-200 Hz (feels "deep" and "powerful")
            // - Use cases: Explosions, heavy impacts, engine vibrations
            // - Intensity 0.1 = 10% power (gentle rumble for demonstration)
            // - Duration 5 seconds allows feeling the sustained effect
            rumble_requests.write(GamepadRumbleRequest::Add {
                gamepad: entity,
                intensity: GamepadRumbleIntensity::strong_motor(0.1),
                duration: Duration::from_secs(5),
            });
        }

        if gamepad.just_pressed(GamepadButton::East) {
            info!("East face button: maximum rumble on both motors for 5 seconds");
            // MAXIMUM INTENSITY DEMONSTRATION - Both motors at full power
            // WARNING: This is the most intense haptic feedback possible
            // Real games should rarely use MAX intensity due to:
            // - Battery drain (400-650mA power consumption)
            // - User comfort (can cause hand fatigue quickly)
            // - Tactile masking (overwhelms subtle feedback)
            // - Accessibility (may be painful for some users)
            rumble_requests.write(GamepadRumbleRequest::Add {
                gamepad: entity,
                duration: Duration::from_secs(5),
                intensity: GamepadRumbleIntensity::MAX,
            });
        }

        if gamepad.just_pressed(GamepadButton::South) {
            info!("South face button: low-intensity rumble on the weak motor for 0.5 seconds");
            // WEAK MOTOR DEMONSTRATION - High-frequency, precise vibration
            // Weak motor characteristics:
            // - Frequency: 100-1000+ Hz (feels "sharp" and "precise")
            // - Use cases: UI feedback, gunshots, texture simulation
            // - Intensity 0.25 = 25% power (noticeable but not overwhelming)
            // - Duration 0.5 seconds = short burst perfect for UI confirmation
            //
            // HUMAN FACTORS: Short, precise feedback is ideal for:
            // - Button press confirmation
            // - Menu navigation
            // - Quick gameplay events (coin collection, etc.)
            rumble_requests.write(GamepadRumbleRequest::Add {
                gamepad: entity,
                duration: Duration::from_secs_f32(0.5),
                intensity: GamepadRumbleIntensity::weak_motor(0.25),
            });
        }

        if gamepad.just_pressed(GamepadButton::West) {
            info!("West face button: custom rumble intensity for 5 second");
            // DUAL-MOTOR COMBINATION - Complex haptic pattern
            // This demonstrates layered tactile feedback using both motors simultaneously
            rumble_requests.write(GamepadRumbleRequest::Add {
                gamepad: entity,
                intensity: GamepadRumbleIntensity {
                    // STRONG MOTOR (low-frequency): 50% intensity
                    // - Provides the "bass" foundation of the haptic experience
                    // - Creates deep, rumbling sensation felt through palms
                    // - Usually positioned on left side of controller
                    strong_motor: 0.5,
                    // WEAK MOTOR (high-frequency): 25% intensity  
                    // - Adds "treble" detail on top of strong motor
                    // - Creates sharp, precise sensation felt through fingertips
                    // - Usually positioned on right side of controller
                    weak_motor: 0.25,
                    //
                    // PSYCHOPHYSICS: Combining frequencies creates rich tactile texture
                    // - Low + High = Complex sensation similar to real-world impacts
                    // - Different ratios = Different "feel signatures" for game events
                    // - This 2:1 ratio (strong:weak) mimics explosion with debris
                },
                duration: Duration::from_secs(5),
            });
        }

        if gamepad.just_pressed(GamepadButton::Start) {
            info!("Start button: Interrupt the current rumble");
            // HAPTIC INTERRUPTION - Emergency stop for user comfort
            // Important for accessibility and user control:
            // - Allows immediate cessation of uncomfortable feedback
            // - Prevents haptic "stacking" (multiple effects overlapping)
            // - Essential for users with sensory sensitivities
            // - Good practice: Always provide user control over haptic intensity
            //
            // TECHNICAL NOTE: Stop command takes precedence over all active effects
            // Bevy's event system ensures this processes immediately
            rumble_requests.write(GamepadRumbleRequest::Stop { gamepad: entity });
        }
    }
}
