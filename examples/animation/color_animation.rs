//! Demonstrates how to animate colors in different color spaces using mixing and splines.
//!
//! Color animation is fascinating! Different color spaces have different properties:
//! some are linear (like physics), others are perceptual (like human vision).
//! This example shows how choosing the right color space affects your animations.
//! It's like choosing between Cartesian and polar coordinates - same destination,
//! very different journey!
//!
//! ## Animation Theory: Color Spaces and Interpolation
//!
//! Color spaces fundamentally affect animation:
//! - **Linear spaces** (LinearRgba, Xyza, Oklaba): Math operations work correctly
//! - **Perceptual spaces** (Oklch, Hsla): Match human vision
//! - **Device spaces** (Srgba): Match display hardware
//!
//! Interpolation differences:
//! 1. **RGB**: Can create muddy colors when mixing (goes through gray)
//! 2. **HSL**: Hue rotation can create rainbow effects
//! 3. **Lab**: Perceptually uniform gradients
//! 4. **XYZ**: Physically accurate light mixing
//!
//! ## Game Feel Context: Color and Emotion
//!
//! Color animation creates mood and feedback:
//! - **Health bars**: Green to red (but which path?)
//! - **Day/night**: Orange sunset transitions
//! - **Magic effects**: Sparkles through spectrum
//! - **UI feedback**: Button color on hover
//!
//! This example demonstrates:
//! - Same color journey, different paths
//! - Why sky gradients look wrong in sRGB
//! - How to avoid "muddy middle" colors
//!
//! ## Performance Optimization: Color Math
//!
//! Color space conversions have costs:
//! 1. **sRGB ↔ Linear**: Gamma curve (pow functions)
//! 2. **RGB ↔ HSL**: Trigonometry
//! 3. **RGB ↔ Lab**: Matrix multiplications
//! 4. **Lab ↔ LCH**: Polar conversions
//!
//! Optimization strategies:
//! - Convert once, animate in one space
//! - Use lookup tables for gamma
//! - SIMD for color batches
//! - Cache commonly used gradients
//!
//! ## Real-World Applications: Color in Games
//!
//! Games using advanced color:
//! - **Journey**: Emotional color progression
//! - **Ori**: Bioluminescent color design
//! - **Hue**: Entire game about color mixing
//! - **Gris**: Color represents emotional stages
//!
//! Common uses:
//! 1. **Damage indication**: Flash red on hit
//! 2. **Power-ups**: Cycling rainbow effects
//! 3. **Environmental**: Sunrise/sunset cycles
//! 4. **Emotional**: Color grading for mood
//!
//! ## Advanced Techniques: Color Science
//!
//! 1. **HDR Color**: Extended range for bloom
//! 2. **Color Blindness**: Accessible palettes
//! 3. **Procedural Palettes**: Generate harmonious colors
//! 4. **Temporal Dithering**: Smooth color banding
//! 5. **Spectral Rendering**: Wavelength-based color
//!
//! ## Common Issues and Solutions
//!
//! 1. **Muddy Colors**: RGB interpolation through gray
//!    - Solution: Use HSL or Lab space
//!
//! 2. **Hue Discontinuity**: Red (0°) to Red (360°)
//!    - Solution: Take shortest path around wheel
//!
//! 3. **Gamma Issues**: Dark colors look wrong
//!    - Solution: Work in linear space
//!
//! 4. **Banding**: Visible steps in gradients
//!    - Solution: Dithering or higher precision

use bevy::{math::VectorSpace, prelude::*};

// We define traits to handle different color animation strategies
// This is generic programming - writing code once that works for many types!

// CurveColor: For colors that can be interpolated along curves (linear spaces)
//
// ## Rust Pattern: Trait Bounds
// VectorSpace means the type supports vector math (add, subtract, scale)
// Into<Color> means it can convert to Bevy's Color type
// Send + Sync + 'static are required for use in Bevy systems
//
// ## Why VectorSpace?
// Curves need math operations: p(t) = (1-t)³p₀ + 3(1-t)²tp₁ + 3(1-t)t²p₂ + t³p₃
// Only linear color spaces support this correctly!
trait CurveColor: VectorSpace + Into<Color> + Send + Sync + 'static {}
impl<T: VectorSpace + Into<Color> + Send + Sync + 'static> CurveColor for T {}

// MixedColor: For colors that can be mixed/blended (any color space)
//
// ## The Mix Trait
// Mix provides the mix() method: color_a.mix(&color_b, t)
// This works in ANY color space, even non-linear ones
// Each space defines its own mixing logic (e.g., HSL mixes hues circularly)
trait MixedColor: Mix + Into<Color> + Send + Sync + 'static {}
impl<T: Mix + Into<Color> + Send + Sync + 'static> MixedColor for T {}

// Component holding a cubic curve through color space
#[derive(Debug, Component)]
struct Curve<T: CurveColor>(CubicCurve<T>);

// Component holding colors to mix between
#[derive(Debug, Component)]
struct Mixed<T: MixedColor>([T; 4]);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                // Systems for curve-based animation (linear color spaces)
                animate_curve::<LinearRgba>, // Physically linear RGB
                animate_curve::<Oklaba>,     // Perceptually uniform
                animate_curve::<Xyza>,       // CIE 1931 color space
                // Systems for mixing-based animation (any color space)
                animate_mixed::<Hsla>,       // Hue, Saturation, Lightness
                animate_mixed::<Srgba>,      // Standard RGB (gamma corrected)
                animate_mixed::<Oklcha>,     // Cylindrical OKLab
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // COLOR SPACE SCIENCE!
    // Linear color spaces allow mathematical operations like curves
    // Think of it like this: in linear space, halfway between two colors
    // is actually the physical halfway point. In non-linear spaces, it's not!

    // Define our color journey: White -> Yellow -> Red -> Black
    let colors = [
        LinearRgba::WHITE,
        LinearRgba::rgb(1., 1., 0.), // Yellow (full red + green)
        LinearRgba::RED,
        LinearRgba::BLACK,
    ];
    
    // TOP SECTION: Curve-based animations (only work in linear spaces)
    // Each row shows the same animation in a different color space
    
    // LinearRgba - physically accurate light mixing
    spawn_curve_sprite(&mut commands, 275., colors);

    // Xyza - CIE 1931, based on human color perception experiments
    spawn_curve_sprite(&mut commands, 175., colors.map(Xyza::from));

    // Oklaba - modern perceptually uniform space
    spawn_curve_sprite(&mut commands, 75., colors.map(Oklaba::from));

    // BOTTOM SECTION: Mixing-based animations (work in any space)
    // Non-linear spaces can't use curves, but we can still interpolate!
    
    // Hsla - intuitive for artists (hue wheel + lightness)
    spawn_mixed_sprite(&mut commands, -75., colors.map(Hsla::from));

    // Srgba - what your monitor uses (gamma corrected)
    spawn_mixed_sprite(&mut commands, -175., colors.map(Srgba::from));

    // Oklcha - cylindrical version of OKLab (hue + chroma)
    spawn_mixed_sprite(&mut commands, -275., colors.map(Oklcha::from));
}

// Helper function to spawn sprites that use curve-based color animation
fn spawn_curve_sprite<T: CurveColor>(commands: &mut Commands, y: f32, points: [T; 4]) {
    commands.spawn((
        Sprite::sized(Vec2::new(75., 75.)),
        Transform::from_xyz(0., y, 0.),
        // Create a cubic Bezier curve through color space!
        // The curve smoothly interpolates through our control points
        Curve(CubicBezier::new([points]).to_curve().unwrap()),
    ));
}

// Helper function to spawn sprites that use mixing-based color animation
fn spawn_mixed_sprite<T: MixedColor>(commands: &mut Commands, y: f32, colors: [T; 4]) {
    commands.spawn((
        Transform::from_xyz(0., y, 0.),
        Sprite::sized(Vec2::new(75., 75.)),
        Mixed(colors), // Just store the colors for mixing
    ));
}

// Animate sprites using cubic curves through color space
// This only works for linear color spaces where interpolation makes mathematical sense
//
// ## Animation Timing
// We use a sine wave for smooth, continuous motion:
// - Period: 2π seconds (about 6.28 seconds per cycle)
// - No jarring stops or starts
// - Naturally eases in and out
//
// ## Cubic Bezier Curves
// The curve provides C² continuity (smooth position, velocity, and acceleration)
// This creates more natural motion than linear interpolation
fn animate_curve<T: CurveColor>(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &Curve<T>)>,
) {
    // Create a smooth oscillation between 0 and 1 using sine wave
    // sin(t) oscillates between -1 and 1, so (sin(t) + 1) / 2 gives us 0 to 1
    let t = (ops::sin(time.elapsed_secs()) + 1.) / 2.;

    for (mut transform, mut sprite, cubic_curve) in &mut query {
        // Sample the curve at position t (0 = start, 1 = end)
        // The curve smoothly interpolates through our color control points
        sprite.color = cubic_curve.0.position(t).into();
        // Move sprite horizontally to show animation progress
        transform.translation.x = 600. * (t - 0.5); // -300 to +300
    }
}

// Animate sprites using color mixing (works in any color space)
//
// ## Mix vs Curve
// Mixing creates piecewise linear interpolation:
// - Straight lines between color points
// - C⁰ continuity (position continuous, velocity jumps)
// - Works in ANY color space
//
// ## Color Space Effects
// The same mix operation behaves differently in each space:
// - RGB: May go through gray (desaturation)
// - HSL: Follows hue wheel (rainbow effect)
// - Lab: Perceptually uniform transitions
fn animate_mixed<T: MixedColor>(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Sprite, &Mixed<T>)>,
) {
    let t = (ops::sin(time.elapsed_secs()) + 1.) / 2.;

    for (mut transform, mut sprite, mixed) in &mut query {
        sprite.color = {
            // Manual interpolation through our color array
            // This is like connecting dots with straight lines instead of curves
            
            // For 4 colors, we have 3 intervals: [0-1], [1-2], [2-3]
            let intervals = (mixed.0.len() - 1) as f32;

            // Which interval are we in? (0, 1, or 2)
            let start_i = (t * intervals).floor().min(intervals - 1.);

            // How far through this interval? (0.0 to 1.0)
            let local_t = (t * intervals) - start_i;

            // Mix between the two colors in this interval
            let color = mixed.0[start_i as usize].mix(&mixed.0[start_i as usize + 1], local_t);
            color.into()
        };
        
        // Animate position to show progression
        // Maps t ∈ [0,1] to x ∈ [-300, 300]
        transform.translation.x = 600. * (t - 0.5);
    }
}
