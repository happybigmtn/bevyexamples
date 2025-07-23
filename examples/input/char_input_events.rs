//! Prints out all chars as they are inputted.
//!
//! # Character Input Processing and Text System Integration
//!
//! This example demonstrates character-level input processing, which is different
//! from key-level input. Character events represent the actual text that would
//! appear when typing, taking into account keyboard layouts, modifiers, and
//! input method editors (IMEs).
//!
//! ## Control Theory: Character vs Key Input
//!
//! ### Two-Layer Input Processing
//! Modern input systems operate on multiple abstraction levels:
//! 1. **Physical Layer**: Hardware key press/release (KeyCode)
//! 2. **Logical Layer**: Character generation (Key::Character)
//!
//! ```text
//! Physical Input    →    Logical Output
//! =============          ==============
//! Shift + '1'       →    '!'
//! Alt + '4'         →    '¤' (on some layouts)
//! Caps Lock + 'a'   →    'A'
//! Compose + 'e' + '`' → 'è'
//! ```
//!
//! ### Keyboard Layout Translation
//! Character events handle complex input transformations:
//! - **Layout mapping**: QWERTY, DVORAK, AZERTY, etc.
//! - **Modifier combinations**: Shift, AltGr, Ctrl+Alt
//! - **Dead keys**: Accent keys that modify next character
//! - **Compose sequences**: Multi-key combinations for special chars
//!
//! ### Input Method Editor (IME) Support
//! For international text input:
//! - **Pinyin**: Chinese character input via Latin alphabet
//! - **Hiragana/Katakana**: Japanese syllabic input systems
//! - **Hangul**: Korean character composition
//! - **Arabic/Hebrew**: Right-to-left text with contextual shaping
//!
//! ## Performance Optimization Strategies
//!
//! ### Event Processing Efficiency
//! Character events are typically less frequent than key events:
//! ```text
//! Typing speed: 40-80 WPM average
//! Characters/second: 3-7 typical, 15+ for fast typists
//! Key events/second: 6-14 (press+release), 30+ for fast typists
//! Event processing: O(n) where n = events per frame
//! ```
//!
//! ### Memory Management
//! Character event structure:
//! ```rust
//! struct KeyboardInput {
//!     key_code: Option<KeyCode>,     // 8 bytes (enum + option)
//!     logical_key: Key,              // 24+ bytes (string for characters)
//!     state: ButtonState,            // 1 byte
//!     window: Entity,                // 8 bytes
//! }
//! // String allocation for multi-byte Unicode characters
//! ```
//!
//! ### Unicode Processing
//! Character strings can contain complex Unicode:
//! - **Basic Latin**: 1 byte per character (ASCII)
//! - **Extended Latin**: 2 bytes per character (accented)
//! - **CJK characters**: 3-4 bytes per character
//! - **Emoji/symbols**: 4+ bytes per character
//! - **Combining characters**: Multiple code points per visible character
//!
//! ## Real-World Applications
//!
//! ### Text Input Systems
//!
//! #### Chat and Communication
//! - **Real-time typing**: Character events for live chat updates
//! - **Autocomplete**: Character-by-character suggestion building
//! - **Spam filtering**: Pattern recognition in character streams
//! - **Profanity filtering**: Real-time content moderation
//!
//! #### Game UI Text Fields
//! - **Player names**: Unicode support for international players
//! - **Search boxes**: Incremental search with character input
//! - **Command consoles**: Programming-style input with syntax highlighting
//! - **Save file names**: File system compatible character validation
//!
//! #### Creative Tools
//! - **Level editors**: Object naming and tagging systems
//! - **Script editors**: Code input with syntax awareness
//! - **Dialogue systems**: Branching narrative text entry
//! - **Note-taking**: In-game documentation and planning tools
//!
//! ### Industry Implementations
//!
//! #### Game Engine Approaches
//! - **Unreal Engine**: Uses FInputKeyEvent with character translation
//! - **Unity**: InputSystem provides character events via Keyboard.onTextInput
//! - **Godot**: Uses _input() with InputEventKey for character handling
//! - **Web browsers**: KeyboardEvent.key provides logical character values
//!
//! ## Advanced Techniques
//!
//! ### Text Composition Handling
//! For complex input methods:
//! ```rust
//! struct TextComposition {
//!     composing_text: String,    // Current composition
//!     cursor_position: usize,   // Cursor within composition
//!     candidates: Vec<String>,  // Available completions
//!     confirmed_text: String,   // Finalized text
//! }
//! ```
//!
//! ### Character Filtering and Validation
//! Different contexts need different character sets:
//! ```rust
//! fn validate_character(character: &str, context: InputContext) -> bool {
//!     match context {
//!         InputContext::PlayerName => {
//!             // Allow letters, numbers, spaces, some punctuation
//!             character.chars().all(|c| c.is_alphanumeric() || " -_.".contains(c))
//!         }
//!         InputContext::Filename => {
//!             // Exclude filesystem reserved characters
//!             !character.chars().any(|c| "<>:\"/|?*".contains(c))
//!         }
//!         InputContext::Numeric => {
//!             // Only digits and decimal point
//!             character.chars().all(|c| c.is_ascii_digit() || c == '.')
//!         }
//!     }
//! }
//! ```
//!
//! ### Input History and Undo Systems
//! Track character input for editing features:
//! ```rust
//! struct InputHistory {
//!     operations: Vec<TextOperation>,
//!     current_index: usize,
//!     max_history: usize,
//! }
//!
//! enum TextOperation {
//!     Insert { position: usize, text: String },
//!     Delete { position: usize, length: usize },
//!     Replace { position: usize, old: String, new: String },
//! }
//! ```
//!
//! ## Common Issues and Solutions
//!
//! ### Unicode Normalization
//! Same visual character can have different encodings:
//! ```text
//! 'é' can be:
//! - U+00E9 (precomposed)
//! - U+0065 U+0301 (e + combining acute accent)
//! 
//! Solution: Normalize to NFC (Canonical Decomposition + Composition)
//! ```
//!
//! ### Input Method Conflicts
//! Game shortcuts vs text input:
//! ```rust
//! // Problem: 'W' key moves character even during text input
//! if keyboard_input.just_pressed(KeyCode::KeyW) && !text_input_active {
//!     move_character_forward();
//! }
//! ```
//!
//! ### Performance with Long Text
//! String operations can be expensive:
//! ```rust
//! // ❌ Slow: Recreating string each character
//! text = text + new_character;
//! 
//! // ✅ Fast: Using mutable string buffer
//! text.push_str(new_character);
//! ```
//!
//! ### Cross-Platform Consistency
//! Different OS keyboard handling:
//! - **Windows**: Uses WM_CHAR messages for character input
//! - **macOS**: NSEvent.characters provides composed characters
//! - **Linux**: XIM/IBus for input method integration
//! - **Web**: KeyboardEvent.key with standard key values
//!
//! ## Rust Programming Patterns
//!
//! ### String Handling Safety
//! Rust's UTF-8 strings prevent invalid character sequences:
//! ```rust
//! // Character validation is built into Rust's String type
//! match std::str::from_utf8(bytes) {
//!     Ok(valid_string) => process_character_input(valid_string),
//!     Err(_) => log::warn!("Invalid UTF-8 in input"),
//! }
//! ```
//!
//! ### Memory Management
//! Character events involve heap allocation for strings:
//! ```rust
//! // String allocation happens in Key::Character
//! match &event.logical_key {
//!     Key::Character(ref character) => {
//!         // `character` is a heap-allocated String
//!         // Rust's ownership ensures memory safety
//!     }
//! }
//! ```
//!
//! ### Error Handling
//! Text processing can fail in various ways:
//! ```rust
//! use std::char::REPLACEMENT_CHARACTER;
//! 
//! fn safe_character_processing(input: &str) -> String {
//!     input.chars()
//!         .filter(|c| !c.is_control())
//!         .map(|c| if c.is_ascii() { c } else { REPLACEMENT_CHARACTER })
//!         .collect()
//! }
//! ```

use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, print_char_event_system)
        .run();
}

/// This system prints out all char events as they come in.
fn print_char_event_system(mut char_input_events: EventReader<KeyboardInput>) {
    for event in char_input_events.read() {
        // Only check for characters when the key is pressed.
        if !event.state.is_pressed() {
            continue;
        }
        if let Key::Character(character) = &event.logical_key {
            info!("{:?}: '{}'", event, character);
        }
    }
}
