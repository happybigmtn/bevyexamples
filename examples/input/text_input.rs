//! Simple text input support
//!
//! Return creates a new line, backspace removes the last character.
//! Clicking toggle IME (Input Method Editor) support, but the font used as limited support of characters.
//! You should change the provided font with another one to test other languages input.
//!
//! # Advanced Text Input Systems and IME Integration
//!
//! This example demonstrates sophisticated text input handling that goes far beyond
//! basic character input. It showcases Input Method Editor (IME) integration,
//! which is essential for international text input and professional applications.
//!
//! ## Control Theory: Text Input Architecture
//!
//! ### Multi-Layer Input Processing
//! Modern text input operates on multiple abstraction layers:
//! 1. **Hardware Layer**: Physical key presses (scan codes)
//! 2. **OS Layer**: Keyboard layout translation (virtual key codes)
//! 3. **IME Layer**: Composition and candidate selection
//! 4. **Application Layer**: Text insertion and editing
//!
//! ```text
//! Raw Input → Layout Map → IME Processing → Character Events → Text Buffer
//! ```
//!
//! ### IME State Machine
//! IME systems follow a complex state machine:
//! ```text
//! [Disabled] ↔ [Enabled] → [Composing] → [Candidates] → [Commit/Cancel]
//!     ↑                      ↓               ↓
//!     ←────────────────────┘               └───────────────┘
//! ```
//!
//! ### Unicode Complexity
//! Text input must handle complex Unicode scenarios:
//! - **Grapheme clusters**: Single "characters" made of multiple code points
//! - **Combining characters**: Accents and diacritics
//! - **Surrogate pairs**: Emoji and extended Unicode planes
//! - **Normalization**: Different representations of same character
//!
//! ## Human Factors in Text Input Design
//!
//! ### Cognitive Load and User Experience
//! - **Visual feedback**: Show composition state during IME input
//! - **Cursor positioning**: Accurate caret placement for editing
//! - **Selection handling**: Text range selection for modification
//! - **Undo/redo**: History management for error recovery
//!
//! ### International User Considerations
//! - **Writing systems**: Left-to-right, right-to-left, top-to-bottom
//! - **Input methods**: Phonetic (Pinyin), shape-based (Wubi), syllabic (Hangul)
//! - **Font requirements**: Proper glyph coverage for target languages
//! - **Cultural conventions**: Date formats, number systems, punctuation
//!
//! ### Accessibility Requirements
//! - **Screen readers**: Proper text semantics and change notifications
//! - **High contrast**: Text visibility for visual impairments
//! - **Motor disabilities**: Large click targets, sticky keys support
//! - **Cognitive disabilities**: Clear error messages, consistent behavior
//!
//! ## Performance Optimization Strategies
//!
//! ### Memory Management
//! Text processing can be memory-intensive:
//! ```rust
//! // Efficient string operations
//! struct TextBuffer {
//!     content: String,           // Main text storage
//!     undo_stack: Vec<Edit>,     // History for undo/redo
//!     composition: String,       // IME composition buffer
//!     cursor_pos: usize,         // Byte position (not char!)
//! }
//! 
//! // Memory usage analysis:
//! // - 1MB text = ~1MB String + metadata
//! // - Undo stack = O(edits) * average_edit_size
//! // - IME buffer = typically <100 bytes
//! ```
//!
//! ### Rendering Performance
//! Text rendering optimization techniques:
//! - **Glyph caching**: Pre-render common character combinations
//! - **Dirty rectangles**: Only redraw changed text regions
//! - **Virtual scrolling**: Render only visible text portions
//! - **Font subsetting**: Load only required character ranges
//!
//! ### Input Latency Minimization
//! Text input responsiveness requirements:
//! - **Keystroke latency**: <50ms from key press to visual feedback
//! - **IME responsiveness**: <100ms for composition updates
//! - **Large document handling**: Maintain responsiveness with 10MB+ text
//!
//! ## Real-World Applications
//!
//! ### Game Genre Implementations
//!
//! #### Chat Systems
//! - **Real-time updates**: Show typing indicators
//! - **Message history**: Efficient storage and retrieval
//! - **Moderation**: Auto-filter inappropriate content
//! - **Internationalization**: Support global user base
//!
//! #### RPG Text Input
//! - **Character naming**: Unicode support for global players
//! - **Guild tags**: Short text with validation rules
//! - **Quest logs**: Large text display with search
//! - **Save notes**: Player-generated content storage
//!
//! #### Developer Tools
//! - **Console commands**: Auto-completion and history
//! - **Script editors**: Syntax highlighting and validation
//! - **Debug output**: High-volume text streaming
//! - **Configuration**: Structured text input with validation
//!
//! ### Industry Standards
//! - **Web browsers**: Rich text editing with undo/redo
//! - **Office software**: Advanced typography and layout
//! - **IDEs**: Code-aware text processing
//! - **Messaging apps**: Emoji, stickers, rich media
//!
//! ## Advanced Techniques
//!
//! ### IME Integration Patterns
//! Professional IME handling:
//! ```rust
//! struct ImeState {
//!     enabled: bool,
//!     position: Vec2,           // Screen coordinates for IME window
//!     composition: String,      // Current composition string
//!     cursor_range: Range<usize>, // Selection within composition
//!     candidates: Vec<String>,  // Available completion options
//! }
//! 
//! impl ImeState {
//!     fn handle_preedit(&mut self, value: &str, cursor: Option<usize>) {
//!         self.composition = value.to_string();
//!         self.cursor_range = cursor.map(|c| c..c).unwrap_or(0..0);
//!     }
//!     
//!     fn commit(&mut self, value: &str) -> String {
//!         self.composition.clear();
//!         value.to_string()
//!     }
//! }
//! ```
//!
//! ### Text Processing Algorithms
//! 
//! #### Grapheme Boundary Detection
//! ```rust
//! use unicode_segmentation::UnicodeSegmentation;
//! 
//! fn safe_backspace(text: &mut String) {
//!     if let Some((last_grapheme_start, _)) = text
//!         .grapheme_indices(true)
//!         .last()
//!     {
//!         text.truncate(last_grapheme_start);
//!     }
//! }
//! ```
//!
//! #### Smart Text Selection
//! ```rust
//! fn select_word_at_position(text: &str, position: usize) -> Range<usize> {
//!     let mut start = position;
//!     let mut end = position;
//!     
//!     // Find word boundaries using Unicode word break rules
//!     while start > 0 && !text.chars().nth(start - 1).unwrap().is_whitespace() {
//!         start -= 1;
//!     }
//!     
//!     while end < text.len() && !text.chars().nth(end).unwrap().is_whitespace() {
//!         end += 1;
//!     }
//!     
//!     start..end
//! }
//! ```
//!
//! ### Undo/Redo System Design
//! ```rust
//! #[derive(Clone)]
//! enum TextEdit {
//!     Insert { position: usize, text: String },
//!     Delete { position: usize, text: String },
//!     Replace { position: usize, old_text: String, new_text: String },
//! }
//! 
//! struct UndoStack {
//!     edits: Vec<TextEdit>,
//!     current: usize,
//!     max_size: usize,
//! }
//! ```
//!
//! ## Common Issues and Solutions
//!
//! ### Unicode Handling Problems
//! - **String indexing**: Never index by bytes, use char boundaries
//! - **Grapheme clusters**: Use unicode-segmentation crate
//! - **Normalization**: Consistent representation of accented characters
//! - **Emoji handling**: Multi-codepoint emoji sequences
//!
//! ### IME Integration Challenges
//! - **Platform differences**: Windows (TSF), macOS (NSTextInput), Linux (XIM/IBus)
//! - **Position tracking**: Keep IME window positioned correctly
//! - **Font fallbacks**: Handle characters not in primary font
//! - **Composition conflicts**: Avoid interfering with OS composition
//!
//! ### Performance Pitfalls
//! - **String concatenation**: Use String::push_str() instead of + operator
//! - **Unnecessary allocations**: Reuse String buffers when possible
//! - **Large text rendering**: Implement virtual scrolling
//! - **Font loading**: Async font loading to avoid blocking
//!
//! ### Cross-Platform Consistency
//! Different OS text behavior:
//! ```rust
//! fn normalize_line_endings(text: &str) -> String {
//!     text.replace("\r\n", "\n")  // Windows CRLF -> Unix LF
//!         .replace("\r", "\n")   // Classic Mac CR -> Unix LF
//! }
//! ```
//!
//! ## Rust Programming Patterns
//!
//! ### Memory Safety Benefits
//! - **No buffer overflows**: Rust strings are bounds-checked
//! - **UTF-8 guarantees**: Invalid UTF-8 cannot corrupt string data
//! - **Ownership clarity**: Clear responsibility for text buffer management
//!
//! ### Zero-Cost Abstractions
//! ```rust
//! // High-level iterator chains compile to efficient loops
//! fn count_words(text: &str) -> usize {
//!     text.unicode_words().count()  // No allocations, optimal performance
//! }
//! ```
//!
//! ### Error Handling
//! ```rust
//! fn safe_text_insert(text: &mut String, position: usize, insert: &str) -> Result<(), TextError> {
//!     if !text.is_char_boundary(position) {
//!         return Err(TextError::InvalidPosition);
//!     }
//!     
//!     text.insert_str(position, insert);
//!     Ok(())
//! }
//! ```

use std::mem;

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                toggle_ime,
                listen_ime_events,
                listen_keyboard_input_events,
                bubbling_text,
            ),
        )
        .run();
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // The default font has a limited number of glyphs, so use the full version for
    // sections that will hold text input.
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        Text::default(),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        children![
            TextSpan::new("Click to toggle IME. Press return to start a new line.\n\n",),
            TextSpan::new("IME Enabled: "),
            TextSpan::new("false\n"),
            TextSpan::new("IME Active:  "),
            TextSpan::new("false\n"),
            TextSpan::new("IME Buffer:  "),
            (
                TextSpan::new("\n"),
                TextFont {
                    font: font.clone(),
                    ..default()
                },
            ),
        ],
    ));

    commands.spawn((
        Text2d::new(""),
        TextFont {
            font,
            font_size: 100.0,
            ..default()
        },
    ));
}

// IME CONTROL SYSTEM - Toggle Input Method Editor functionality
// IME is essential for international text input (Chinese, Japanese, Korean, etc.)
fn toggle_ime(
    input: Res<ButtonInput<MouseButton>>,
    mut window: Single<&mut Window>,
    status_text: Single<Entity, (With<Node>, With<Text>)>,
    mut ui_writer: TextUiWriter,
) {
    if input.just_pressed(MouseButton::Left) {
        // IME POSITIONING - Set where the IME composition window appears
        // Critical for user experience: IME window should appear near text cursor
        window.ime_position = window.cursor_position().unwrap();
        
        // IME STATE TOGGLE - Enable/disable composition processing
        window.ime_enabled = !window.ime_enabled;

        // UI FEEDBACK - Update status display
        *ui_writer.text(*status_text, 3) = format!("{}\n", window.ime_enabled);
    }
}

#[derive(Component)]
struct Bubble {
    timer: Timer,
}

fn bubbling_text(
    mut commands: Commands,
    mut bubbles: Query<(Entity, &mut Transform, &mut Bubble)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut bubble) in bubbles.iter_mut() {
        if bubble.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
        transform.translation.y += time.delta_secs() * 100.0;
    }
}

fn listen_ime_events(
    mut events: EventReader<Ime>,
    status_text: Single<Entity, (With<Node>, With<Text>)>,
    mut edit_text: Single<&mut Text2d, (Without<Node>, Without<Bubble>)>,
    mut ui_writer: TextUiWriter,
) {
    for event in events.read() {
        match event {
            Ime::Preedit { value, cursor, .. } if !cursor.is_none() => {
                *ui_writer.text(*status_text, 7) = format!("{value}\n");
            }
            Ime::Preedit { cursor, .. } if cursor.is_none() => {
                *ui_writer.text(*status_text, 7) = "\n".to_string();
            }
            Ime::Commit { value, .. } => {
                edit_text.push_str(value);
            }
            Ime::Enabled { .. } => {
                *ui_writer.text(*status_text, 5) = "true\n".to_string();
            }
            Ime::Disabled { .. } => {
                *ui_writer.text(*status_text, 5) = "false\n".to_string();
            }
            _ => (),
        }
    }
}

fn listen_keyboard_input_events(
    mut commands: Commands,
    mut events: EventReader<KeyboardInput>,
    edit_text: Single<(&mut Text2d, &TextFont), (Without<Node>, Without<Bubble>)>,
) {
    let (mut text, style) = edit_text.into_inner();
    for event in events.read() {
        // Only trigger changes when the key is first pressed.
        if !event.state.is_pressed() {
            continue;
        }

        match (&event.logical_key, &event.text) {
            (Key::Enter, _) => {
                if text.is_empty() {
                    continue;
                }
                let old_value = mem::take(&mut **text);

                commands.spawn((
                    Text2d::new(old_value),
                    style.clone(),
                    Bubble {
                        timer: Timer::from_seconds(5.0, TimerMode::Once),
                    },
                ));
            }
            (Key::Backspace, _) => {
                text.pop();
            }
            (_, Some(inserted_text)) => {
                // Make sure the text doesn't have any control characters,
                // which can happen when keys like Escape are pressed
                if inserted_text.chars().all(is_printable_char) {
                    text.push_str(inserted_text);
                }
            }
            _ => continue,
        }
    }
}

// this logic is taken from egui-winit:
// https://github.com/emilk/egui/blob/adfc0bebfc6be14cee2068dee758412a5e0648dc/crates/egui-winit/src/lib.rs#L1014-L1024
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}
