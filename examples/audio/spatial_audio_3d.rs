//! This example illustrates how to load and play an audio file, and control where the sounds seems to come from.
//! 
//! ## Key Concepts
//! 
//! - **3D Spatial Audio**: In 3D games, sounds need to account for full 3D positioning.
//!   Unlike 2D (which only pans left/right), 3D audio also simulates sounds coming from
//!   above, below, in front, and behind the listener.
//! 
//! - **Head-Related Transfer Function (HRTF)**: The way sounds change as they travel
//!   around your head to reach each ear. Bevy's spatial audio simulates this to create
//!   realistic 3D positioning.
//! 
//! - **Circular Motion**: The emitter moves in a circle around the listener, demonstrating
//!   how spatial audio handles sounds moving in 3D space. You'll hear the sound rotate
//!   around your head if wearing headphones.
//! 
//! - **SpatialAudioSink**: An enhanced version of AudioSink that provides spatial audio
//!   controls in addition to standard playback controls like muting.
//!
//! ## Audio Theory: 3D Sound Localization
//!
//! **How We Perceive 3D Sound Position**:
//! 1. **Interaural Time Difference (ITD)**: Sound reaches nearer ear first
//!    - Maximum ~0.7ms delay for sounds from the side
//!    - Most effective for frequencies below 1500 Hz
//!    - Provides left/right localization
//!
//! 2. **Interaural Level Difference (ILD)**: Head shadows high frequencies
//!    - Can be 20+ dB difference at high frequencies
//!    - Most effective above 1500 Hz
//!    - Also provides left/right information
//!
//! 3. **Spectral Cues**: Ear shape filters sound differently by direction
//!    - Pinna (outer ear) creates direction-dependent filtering
//!    - Critical for up/down and front/back disambiguation
//!    - Highly individual - everyone's ears are different!
//!
//! **The Cone of Confusion**:
//! - Sounds at same distance/angle from both ears
//! - Forms cone shape around interaural axis
//! - ITD and ILD identical for all points on cone
//! - Resolved by spectral cues and head movement
//!
//! **Distance Perception**:
//! - **Direct/Reverb Ratio**: More reverb = farther
//! - **High Frequency Rolloff**: Air absorbs highs
//! - **Loudness**: Follows inverse square law
//! - **Motion Parallax**: Near sounds move faster
//!
//! ## Game Design Context: Immersive 3D Audio
//!
//! **Why 3D Audio Matters in Games**:
//! - **Spatial Awareness**: Locate enemies without seeing them
//! - **Immersion**: Believable virtual worlds
//! - **Gameplay Mechanics**: Audio-based puzzles and stealth
//! - **Accessibility**: Critical for visually impaired players
//!
//! **Famous 3D Audio in Games**:
//! - **Hellblade**: Binaural audio for psychological horror
//! - **Overwatch**: Precise enemy footstep positioning  
//! - **Hunt: Showdown**: Audio as primary gameplay mechanic
//! - **CS:GO**: Competitive advantage through sound
//!
//! **Common 3D Audio Scenarios**:
//! ```text
//! Scenario          | Key Audio Features
//! ------------------|------------------
//! FPS Games         | Footsteps, gunfire direction, reload sounds
//! Horror Games      | Ambient threats, whispers, breathing
//! Open World        | Environmental ambience, wildlife
//! Flight Sims       | Engine position, radio chatter, warnings
//! VR Games          | Full 360° soundscape critical for presence
//! ```
//!
//! ## Performance Optimization: 3D Audio Processing
//!
//! **Per-Source CPU Costs** (rough estimates):
//! ```text
//! Operation              | CPU Cycles | Notes
//! -----------------------|------------|-------
//! Distance calculation   | 20-30      | 3D vector math
//! Attenuation           | 10-20      | Falloff curves
//! Doppler shift         | 30-50      | Velocity-based
//! HRTF filtering        | 200-1000   | Quality dependent
//! Obstruction raycast   | 500-5000   | Scene complexity
//! Reverb send           | 100-200    | Zone-based
//! ```
//!
//! **Optimization Strategies**:
//! 1. **LOD System**: Simpler processing for distant sounds
//! 2. **Culling**: Don't process inaudible sounds
//! 3. **Update Rates**: Slower updates for static sources
//! 4. **Pooling**: Reuse audio source instances
//! 5. **Baked Data**: Pre-compute reverb zones
//!
//! **Memory Considerations**:
//! - HRTF data: 1-10 MB depending on quality
//! - Per-source overhead: 1-4 KB state data
//! - Reverb impulses: 100 KB - 10 MB per space
//!
//! ## Real-World Applications
//!
//! **3D Audio APIs and Standards**:
//! - **OpenAL**: Cross-platform 3D audio
//! - **Steam Audio**: Physics-based with occlusion
//! - **Resonance Audio**: Google's VR spatial audio
//! - **Windows Sonic**: Microsoft's spatial platform
//! - **Dolby Atmos**: Object-based for games/film
//!
//! **Coordinate System Conversions**:
//! ```text
//! System     | Up  | Forward | Right | Handedness
//! -----------|-----|---------|-------|------------
//! Bevy       | +Y  | -Z      | +X    | Right
//! Unity      | +Y  | +Z      | +X    | Left
//! Unreal     | +Z  | +X      | +Y    | Left
//! OpenGL     | +Y  | -Z      | +X    | Right
//! DirectX    | +Y  | +Z      | +X    | Left
//! ```
//!
//! ## Advanced Techniques: Enhanced Spatial Audio
//!
//! **HRTF Improvements**:
//! 1. **Personalized HRTFs**: Measure individual ear shapes
//! 2. **Dynamic HRTFs**: Interpolate between measurements
//! 3. **Near-field HRTFs**: Special handling < 1 meter
//!
//! **Environmental Audio**:
//! ```rust
//! // Obstruction: Sound blocked by geometry
//! let obstruction = raycast_to_listener(source_pos);
//! volume *= 1.0 - obstruction;
//! 
//! // Occlusion: Sound travels around obstacles  
//! let occlusion = calculate_diffraction_path(source_pos);
//! apply_lowpass_filter(occlusion);
//! 
//! // Room acoustics: Early reflections + late reverb
//! let room_ir = get_room_impulse_response(room_id);
//! output = convolve(dry_signal, room_ir);
//! ```
//!
//! **Psychoacoustic Enhancements**:
//! - **Externalization**: Make headphone audio sound "outside the head"
//! - **Distance Rendering**: Realistic near/far field transitions
//! - **Source Width**: Some sounds aren't point sources
//! - **Doppler Effect**: Pitch shift for moving sources
//!
//! ## Common Issues and Solutions
//!
//! **Problem**: Front/back confusion
//! - **Solution**: Subtle spectral differences, encourage head movement
//!
//! **Problem**: Sounds feel "inside the head"
//! - **Solution**: Add room reverb, improve HRTF quality
//!
//! **Problem**: Elevation perception is poor
//! - **Solution**: Enhanced spectral cues, visual confirmation
//!
//! **Problem**: Performance with many sources
//! - **Solution**: LOD system, prioritize important sounds

use bevy::{
    // Import basic color constants for visual indicators
    color::palettes::basic::{BLUE, LIME, RED},
    prelude::*,
    // Stopwatch for timing the circular motion
    time::Stopwatch,
};

// ## Rust Programming Fundamentals: Color Palettes
//
// **Bevy's Color System**:
// - `palettes::basic`: Common named colors (RED, BLUE, etc.)
// - `palettes::css`: CSS color names (CRIMSON, DODGER_BLUE)
// - `palettes::tailwind`: Tailwind CSS colors with shades
//
// **Color Space Considerations**:
// ```rust
// // Linear RGB (physics-correct)
// let linear = Color::linear_rgb(1.0, 0.0, 0.0);
// 
// // sRGB (perceptually uniform)
// let srgb = Color::srgb(1.0, 0.0, 0.0);
// 
// // HSL (intuitive for artists)
// let hsl = Color::hsl(0.0, 1.0, 0.5);  // Red
// ```

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // Update systems for the interactive demo:
        .add_systems(Update, update_positions)  // Moves emitter in a circle
        .add_systems(Update, update_listener)   // Keyboard control for listener
        .add_systems(Update, mute)              // Toggle mute with M key
        .run();
}

/// Sets up the 3D spatial audio demo scene.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Distance between the listener's ears in world units.
    // In 3D, we use realistic measurements - 4 units might represent 0.17 meters
    // (the average distance between human ears).
    //
    // ## Spatial Scale in 3D Audio
    //
    // **Real-World Measurements**:
    // - Average head width: 15-17 cm
    // - Ear-to-ear distance: 17-21 cm (varies by person)
    // - Sound travel time across head: ~0.6 ms
    //
    // **Game Scale Considerations**:
    // ```text
    // Game Type    | 1 Unit Equals | Ear Gap | Notes
    // -------------|---------------|---------|-------
    // Realistic    | 1 meter       | 0.17    | Human-scale
    // Large World  | 10 meters     | 0.017   | Exaggerated distances
    // Small World  | 0.1 meters    | 1.7     | Miniature perspective
    // Abstract     | No real equiv | 4.0     | Whatever feels right
    // ```
    //
    // **Why 4.0 in this example?**
    // - Visually clear in the demo
    // - Exaggerated for learning purposes
    // - Easy to see the ear positions
    // - Could represent ~17cm if 1 unit = 4.25cm
    let gap = 4.0;

    // Spawn the sound emitter - a blue sphere that orbits the origin
    commands.spawn((
        // Create a sphere mesh with radius 0.2
        // The uv(32, 18) parameters control the mesh resolution:
        // - 32 segments around the equator (longitude)
        // - 18 segments from pole to pole (latitude)
        //
        // ## 3D Mesh Generation
        //
        // **Sphere UV Mapping**:
        // - U (0-1): Wraps around equator (longitude)
        // - V (0-1): From south to north pole (latitude)
        // - More segments = smoother appearance
        // - Fewer segments = better performance
        //
        // **Segment Count Guidelines**:
        // ```text
        // Quality     | U Segments | V Segments | Triangles
        // ------------|------------|------------|----------
        // Low         | 16         | 8          | 256
        // Medium      | 32         | 18         | 1,152
        // High        | 64         | 36         | 4,608
        // Ultra       | 128        | 72         | 18,432
        // ```
        //
        // **Why These Numbers?**
        // - 32 horizontal: Good circular appearance
        // - 18 vertical: Half of horizontal maintains aspect
        // - Powers of 2 for U: Better texture mapping
        Mesh3d(meshes.add(Sphere::new(0.2).mesh().uv(32, 18))),
        // Blue material for the emitter
        //
        // ## Material Systems in 3D
        //
        // **StandardMaterial Components**:
        // - Base color: Albedo/diffuse color
        // - Metallic: 0.0 (dielectric) to 1.0 (metal)
        // - Roughness: 0.0 (mirror) to 1.0 (matte)
        // - Emission: Self-illumination
        //
        // **Color::from() Conversions**:
        // - From named colors (like BLUE)
        // - From RGB tuples: (1.0, 0.0, 0.0)
        // - From hex: Color::from(0x0000FF)
        // - From CSS: Color::from("blue")
        MeshMaterial3d(materials.add(Color::from(BLUE))),
        // Start at the origin - it will move in update_positions
        Transform::from_xyz(0.0, 0.0, 0.0),
        // Component to track animation state
        Emitter::default(),
        // Load and play the audio file
        //
        // ## AudioPlayer in 3D
        //
        // **3D vs 2D Audio Setup**:
        // ```rust
        // // 2D Audio (stereo panning only)
        // AudioPlayer::new(handle),
        // PlaybackSettings::LOOP.with_spatial(false),
        // 
        // // 3D Audio (full spatial)
        // AudioPlayer::new(handle),
        // PlaybackSettings::LOOP.with_spatial(true),
        // Transform::from_xyz(x, y, z),  // Required!
        // ```
        //
        // **Transform Requirement**:
        // - 3D spatial audio needs world position
        // - Transform component provides this
        // - Missing Transform = audio at origin
        AudioPlayer::new(asset_server.load("sounds/Windless Slopes.ogg")),
        // Configure for looping spatial playback
        //
        // ## Spatial Audio Settings
        //
        // **What with_spatial(true) Enables**:
        // 1. Distance attenuation (volume by distance)
        // 2. 3D panning (left/right based on angle)
        // 3. HRTF filtering (up/down, front/back)
        // 4. Doppler effect (if configured)
        // 5. Obstruction (if implemented)
        //
        // **Performance Impact**:
        // - Non-spatial: ~10 μs per source
        // - Spatial: ~50-200 μs per source
        // - Worth it for positional sounds
        // - Skip for music, UI sounds
        PlaybackSettings::LOOP.with_spatial(true),
    ));

    // Create the spatial listener with specified ear separation
    let listener = SpatialListener::new(gap);
    commands.spawn((
        // Listener starts at origin
        Transform::default(),
        Visibility::default(),
        // Clone because we need to access ear offsets below
        //
        // ## Clone vs Reference in Rust
        //
        // **Why Clone Here?**
        // - We need listener data in two places:
        // 1. As a component (moved into spawn)
        // 2. To access ear offsets (need to read)
        // - Clone gives us two owned copies
        //
        // **SpatialListener Memory Layout**:
        // ```rust
        // pub struct SpatialListener {
        //     pub left_ear_offset: Vec3,   // 12 bytes
        //     pub right_ear_offset: Vec3,  // 12 bytes
        // }  // Total: 24 bytes (cheap to clone)
        // ```
        //
        // **Alternative Patterns**:
        // ```rust
        // // Store offsets separately
        // let left_offset = listener.left_ear_offset;
        // let right_offset = listener.right_ear_offset;
        // 
        // // Or use references in closure
        // let listener_ref = &listener;
        // ```
        listener.clone(),
        // Spawn visual indicators for each ear as children
        //
        // ## Parent-Child Hierarchies in 3D
        //
        // **Transform Propagation**:
        // ```text
        // Parent Transform × Child Local Transform = Child World Transform
        // 
        // Example:
        // Parent at (10, 0, 0), rotated 45°
        // Child at local (1, 0, 0)
        // Child world position: (10.7, 0, -0.7)
        // ```
        //
        // **Why Use Hierarchies?**
        // 1. Ears move with head automatically
        // 2. Can rotate listener without updating ears
        // 3. Natural representation of connected objects
        // 4. Efficient batch transformations
        children![
            // Left ear - red cube
            (
                // Cuboid::new creates a box with given dimensions
                //
                // ## Primitive Mesh Types
                //
                // **Available 3D Primitives**:
                // - Sphere::new(radius)
                // - Cuboid::new(x, y, z)
                // - Cylinder::new(radius, height)
                // - Capsule::new(radius, height)
                // - Torus::new(ring_radius, tube_radius)
                // - Plane::new(width, height)
                //
                // **Mesh Reuse Pattern**:
                // ```rust
                // // Create once, use many times
                // let cube_mesh = meshes.add(Cuboid::new(0.2, 0.2, 0.2));
                // for i in 0..100 {
                //     commands.spawn(Mesh3d(cube_mesh.clone()));
                // }
                // ```
                Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
                MeshMaterial3d(materials.add(Color::from(RED))),
                // Position at the left ear offset calculated by SpatialListener
                //
                // ## Local vs World Space
                //
                // **This Transform is in Parent Space**:
                // - left_ear_offset is relative to listener
                // - If listener moves, ear follows
                // - If listener rotates, ear orbits
                //
                // **Ear Offset Calculation**:
                // ```rust
                // left_ear_offset = Vec3::new(-gap / 2.0, 0.0, 0.0)
                // // For gap = 4.0: (-2.0, 0.0, 0.0)
                // ```
                Transform::from_translation(listener.left_ear_offset),
            ),
            // Right ear - green cube
            (
                Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
                MeshMaterial3d(materials.add(Color::from(LIME))),
                // Position at the right ear offset
                Transform::from_translation(listener.right_ear_offset),
            )
        ],
    ));

    // Add lighting so we can see the 3D objects
    commands.spawn((
        DirectionalLight::default(),
        // Position light above and to the side, looking at origin
        // looking_at() creates a transform that faces a specific point
        //
        // ## 3D Lighting Fundamentals
        //
        // **Light Types in Bevy**:
        // 1. **DirectionalLight**: Sun-like, parallel rays
        // 2. **PointLight**: Omnidirectional from a point
        // 3. **SpotLight**: Cone of light
        // 4. **AmbientLight**: Global illumination (via resource)
        //
        // **DirectionalLight Defaults**:
        // ```rust
        // DirectionalLight {
        //     color: Color::WHITE,
        //     illuminance: 100_000.0,  // Lux (bright daylight)
        //     shadows_enabled: false,
        //     ..default()
        // }
        // ```
        //
        // **Real-World Illuminance Values** (lux):
        // - Moonlight: 0.25
        // - Living room: 50
        // - Office: 500
        // - Overcast day: 1,000
        // - Direct sunlight: 100,000
        //
        // **Transform::looking_at Explained**:
        // - First param: Target point to look at
        // - Second param: Which way is "up"
        // - Creates rotation matrix to face target
        // - Common for cameras and lights
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Display control instructions
    commands.spawn((
        Text::new(
            "Up/Down/Left/Right: Move Listener\nSpace: Toggle Emitter Movement\nM: Toggle Mute",
        ),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        //
        // ## UI Positioning in Bevy
        //
        // **PositionType Options**:
        // - `Relative`: Normal flow (default)
        // - `Absolute`: Positioned relative to parent
        //
        // **Val (Value) Types**:
        // ```rust
        // Val::Px(100.0)    // Pixels
        // Val::Percent(50.0) // Percentage of parent
        // Val::Auto          // Automatic sizing
        // Val::Vw(10.0)      // Viewport width units
        // Val::Vh(10.0)      // Viewport height units
        // ```
        //
        // **Coordinate System**:
        // - Origin: Top-left of parent
        // - X increases right
        // - Y increases down (opposite of world space!)
        // - bottom/right position from opposite edges
    ));

    // Add a 3D camera positioned above and behind the origin
    commands.spawn((
        Camera3d::default(),
        // Position camera at (0, 5, 5) looking down at the origin
        // The up direction (Vec3::Y) ensures the camera isn't tilted
        //
        // ## Camera Setup for 3D Scenes
        //
        // **Camera3d Default Settings**:
        // ```rust
        // Camera3d {
        //     projection: Perspective {
        //         fov: 60.0 degrees,
        //         near: 0.1,  // Closest visible distance
        //         far: 1000.0, // Farthest visible distance
        //     },
        //     hdr: true,  // High dynamic range
        // }
        // ```
        //
        // **Camera Positioning Math**:
        // - Position: (0, 5, 5) forms 45° angle to origin
        // - Distance: √(5² + 5²) = 7.07 units from origin
        // - Field of view captures roughly 10 units wide
        //
        // **Common Camera Setups**:
        // ```rust
        // // Isometric (no perspective)
        // Transform::from_xyz(10.0, 10.0, 10.0)
        //     .looking_at(Vec3::ZERO, Vec3::Y)
        // 
        // // First-person (at eye height)
        // Transform::from_xyz(0.0, 1.7, 0.0)
        // 
        // // Top-down
        // Transform::from_xyz(0.0, 10.0, 0.0)
        //     .looking_at(Vec3::ZERO, Vec3::NEG_Z)
        // ```
        Transform::from_xyz(0.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Component that tracks the emitter's animation state.
//
// ## Component Design Patterns
//
// **Data-Oriented Design**:
// - Components store data, not behavior
// - Systems provide behavior
// - Enables cache-efficient iteration
// - Allows parallelization
//
// **Default Trait Benefits**:
// ```rust
// // Can use default() in spawn
// commands.spawn((Transform::default(), Emitter::default()));
// 
// // Can implement custom defaults
// impl Default for Emitter {
//     fn default() -> Self {
//         Self {
//             stopwatch: Stopwatch::new(),
//             speed: 1.0,
//             radius: 3.0,
//         }
//     }
// }
// ```
#[derive(Component, Default)]
struct Emitter {
    /// Tracks elapsed time for circular motion animation
    //
    // ## Stopwatch vs Timer vs Time
    //
    // **Stopwatch**: Counts up from 0, can pause/resume
    // ```rust
    // stopwatch.tick(delta);
    // let elapsed = stopwatch.elapsed_secs();
    // ```
    //
    // **Timer**: Counts down/up to target
    // ```rust  
    // timer.tick(delta);
    // if timer.finished() { /* do something */ }
    // ```
    //
    // **Time Resource**: Frame timing info
    // ```rust
    // time.delta_secs()     // Since last frame
    // time.elapsed_secs()   // Since app start
    // ```
    stopwatch: Stopwatch,
}

/// Updates emitter positions, creating circular motion around the origin.
/// 
/// The circular path demonstrates how 3D spatial audio handles movement in
/// all directions. With headphones, you'll hear the sound circle around your head.
//
// ## System Architecture: Transform Updates
//
// **Query Types**:
// - `&Transform`: Read-only access
// - `&mut Transform`: Mutable access
// - `With<T>`: Filter for entities with component T
// - `Without<T>`: Filter for entities without T
//
// **Transform Mutation Patterns**:
// ```rust
// // Direct assignment
// transform.translation = Vec3::new(x, y, z);
// 
// // Incremental updates
// transform.translation.x += delta;
// 
// // Helper methods
// transform.rotate_y(angle);
// transform.look_at(target, up);
// ```
fn update_positions(
    time: Res<Time>,
    mut emitters: Query<(&mut Transform, &mut Emitter), With<Emitter>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (mut emitter_transform, mut emitter) in emitters.iter_mut() {
        // Toggle circular motion with spacebar
        if keyboard.just_pressed(KeyCode::Space) {
            if emitter.stopwatch.is_paused() {
                emitter.stopwatch.unpause();
            } else {
                emitter.stopwatch.pause();
            }
        }

        // Update the animation timer
        emitter.stopwatch.tick(time.delta());

        if !emitter.stopwatch.is_paused() {
            // Create circular motion using sine and cosine:
            // - x = radius * sin(angle) gives horizontal position
            // - z = radius * cos(angle) gives depth position
            // - y stays at 0 (moving in the horizontal plane)
            // 
            // As time increases, the angle increases, creating rotation.
            // The radius of 3.0 keeps the sound source clearly audible.
            //
            // ## Circular Motion Mathematics
            //
            // **Parametric Circle Equation**:
            // ```text
            // x(t) = r × sin(ωt + φ)
            // z(t) = r × cos(ωt + φ)
            // 
            // Where:
            // r = radius (3.0)
            // ω = angular velocity (1 rad/s here)
            // t = time (seconds)
            // φ = phase offset (0 here)
            // ```
            //
            // **Motion Characteristics**:
            // - Period: 2π seconds (≈ 6.28s)
            // - Speed: 3.0 units/second (constant)
            // - Angular velocity: 1 radian/second
            // - Centripetal acceleration: v²/r = 3 units/s²
            //
            // **3D Audio Effects During Motion**:
            // 1. **Doppler Shift**: Pitch changes as source approaches/recedes
            // 2. **Distance Variation**: Volume changes (minimal here)
            // 3. **Panning**: Smooth left-right transition
            // 4. **HRTF Changes**: Front/back filtering
            emitter_transform.translation.x = ops::sin(emitter.stopwatch.elapsed_secs()) * 3.0;
            emitter_transform.translation.z = ops::cos(emitter.stopwatch.elapsed_secs()) * 3.0;
            
            // The sound will move:
            // - Front (z=3) → Right (x=3) → Back (z=-3) → Left (x=-3) → Front
            // This creates a counter-clockwise rotation when viewed from above
            //
            // ## Spatial Audio During Circular Motion
            //
            // **Position Timeline** (first rotation):
            // ```text
            // Time  | Position      | Audio Effect
            // ------|---------------|-------------
            // 0.00s | (0, 0, 3)     | Front center
            // 1.57s | (3, 0, 0)     | Right side
            // 3.14s | (0, 0, -3)    | Behind listener
            // 4.71s | (-3, 0, 0)    | Left side
            // 6.28s | (0, 0, 3)     | Front again
            // ```
            //
            // **What You Should Hear**:
            // - Smooth panning as sound circles
            // - Subtle filtering when behind (HRTF)
            // - Possible slight Doppler effect
            // - Consistent volume (constant distance)
            //
            // **Troubleshooting Audio Perception**:
            // - Use headphones for best effect
            // - Ensure audio device supports stereo
            // - Close eyes to focus on audio
            // - Move head slightly to disambiguate
        }
    }
}

/// Moves the spatial listener based on keyboard input.
/// 
/// In 3D, we move along the X and Z axes (the ground plane),
/// keeping Y constant. This simulates walking around at ground level.
//
// ## Coordinate Systems in 3D Audio
//
// **Bevy's Right-Handed System**:
// ```text
//      +Y (up)
//       |
//       |  / -Z (forward)
//       | /
// ------+------ +X (right)
//      /|
//     / |
//   +Z  -X
// (back) (left)
// ```
//
// **Why Z is "Backward"**:
// - Common in graphics (OpenGL convention)
// - Camera looks down -Z by default
// - Makes math consistent with 2D (X,Y)
//
// **Movement Mapping**:
// - W/Up: Move forward (-Z)
// - S/Down: Move backward (+Z)
// - A/Left: Move left (-X)
// - D/Right: Move right (+X)
fn update_listener(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    // Single query expects exactly one listener entity
    //
    // ## Single vs Query
    //
    // **Single<T>**: Exactly one entity expected
    // ```rust
    // // Panics if 0 or 2+ entities match
    // let mut listener: Single<&mut Transform> = ...;
    // listener.translation.x += 1.0;
    // ```
    //
    // **Query<T>**: Zero or more entities
    // ```rust
    // // Iterate over all matches
    // for mut transform in query.iter_mut() {
    //     transform.translation.x += 1.0;
    // }
    // ```
    //
    // **When to Use Single**:
    // - Player entity
    // - Main camera
    // - Unique game objects
    // - Better performance (no iteration)
    mut listeners: Single<&mut Transform, With<SpatialListener>>,
) {
    // Movement speed in world units per second
    let speed = 2.;

    // Right/Left movement along the X axis
    if keyboard.pressed(KeyCode::ArrowRight) {
        listeners.translation.x += speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        listeners.translation.x -= speed * time.delta_secs();
    }
    
    // Forward/Backward movement along the Z axis
    // Note: In Bevy's coordinate system:
    // - +X is right
    // - +Y is up  
    // - +Z is backward (away from the default camera view)
    // So pressing "Down" moves +Z (backward) and "Up" moves -Z (forward)
    if keyboard.pressed(KeyCode::ArrowDown) {
        listeners.translation.z += speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        listeners.translation.z -= speed * time.delta_secs();
    }
    
    // ## 3D Spatial Audio Update Pipeline
    //
    // **What Happens When Listener Moves**:
    // 1. **Transform System**: Updates world position
    // 2. **Audio System**: Reads listener position
    // 3. **Per-Source Calculation**:
    //    ```rust
    //    let relative_pos = source_pos - listener_pos;
    //    let distance = relative_pos.length();
    //    let direction = relative_pos.normalize();
    //    ```
    // 4. **Audio Processing**:
    //    - Distance → Volume attenuation
    //    - Direction → HRTF filter selection
    //    - Direction → Stereo panning
    //    - Velocity → Doppler shift
    //
    // **Listener Movement Effects**:
    // - Moving toward sound: Volume increases, pitch up (Doppler)
    // - Moving away: Volume decreases, pitch down
    // - Moving perpendicular: Maximum Doppler shift
    // - Stationary: Stable audio properties
}

/// Toggles mute for all spatial audio sinks when M is pressed.
/// 
/// This demonstrates runtime control over spatial audio playback.
/// SpatialAudioSink provides all the controls of a regular AudioSink,
/// plus spatial positioning features.
//
// ## Audio Control Hierarchy
//
// **AudioSink Methods** (2D and 3D):
// ```rust
// sink.pause();              // Stop playback, keep position
// sink.play();               // Resume from paused position
// sink.stop();               // Stop and reset to beginning
// sink.set_volume(0.5);      // Adjust volume (0.0 - 1.0)
// sink.set_speed(1.5);       // Playback speed (pitch changes)
// sink.toggle_mute();        // Mute/unmute
// ```
//
// **SpatialAudioSink Additional Methods**:
// ```rust
// sink.set_position(pos);    // Update 3D position
// sink.set_velocity(vel);    // For Doppler effect
// sink.set_emitter_scale(2.0); // Sound size/spread
// ```
//
// **Mute vs Pause vs Stop**:
// - **Mute**: Audio runs silently (preserves timing)
// - **Pause**: Playback halts (can resume)
// - **Stop**: Resets to beginning
fn mute(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    // Query for all spatial audio sinks in the scene
    //
    // ## SpatialAudioSink Component
    //
    // **Created Automatically When**:
    // - AudioPlayer spawned with spatial audio
    // - After one frame delay (next update)
    // - Provides runtime control handle
    //
    // **Sink Lifecycle**:
    // ```text
    // Frame 0: Spawn AudioPlayer
    // Frame 1: Audio system creates SpatialAudioSink
    // Frame 2+: Can control via sink methods
    // Despawn: Sink removed, playback stops
    // ```
    //
    // **Multiple Sinks Pattern**:
    // ```rust
    // // Control specific sounds
    // Query<&mut SpatialAudioSink, With<EnemySound>>
    // Query<&mut SpatialAudioSink, With<AmbientSound>>
    // ```
    mut sinks: Query<&mut SpatialAudioSink>
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        // Toggle mute state for all playing spatial audio
        for mut sink in sinks.iter_mut() {
            // toggle_mute() flips between muted and unmuted states
            // Unlike pause(), muted audio continues playing silently,
            // maintaining its position in the track
            //
            // ## Mute Implementation Details
            //
            // **What Happens During Mute**:
            // 1. Audio samples still generated
            // 2. Samples multiplied by 0 before output
            // 3. Position tracking continues
            // 4. Perfect sync when unmuted
            //
            // **Use Cases for Mute**:
            // - Temporary silence (cutscenes)
            // - Audio debug (isolate sounds)
            // - Accessibility options
            // - Performance (mute distant sounds)
            //
            // **Performance Note**:
            // Muted audio still consumes CPU for:
            // - Sample generation
            // - Position updates  
            // - Effect processing
            // Use pause() or despawn for true savings
            sink.toggle_mute();
        }
    }
}
