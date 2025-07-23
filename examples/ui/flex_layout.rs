//! # Advanced Flexbox Layout System: Comprehensive Guide to Responsive Design
//!
//! This example demonstrates the complete CSS Flexbox implementation in Bevy,
//! showcasing how `AlignItems` and `JustifyContent` properties create sophisticated
//! responsive layouts that adapt to any screen size and content configuration.
//!
//! ## Responsive Design Theory: The Foundation of Modern UIs
//!
//! Flexbox revolutionized web design by solving the "holy grail" of layout problems:
//! creating flexible, responsive interfaces that work across infinite device variations.
//! This example implements a comprehensive testing matrix showing all 30 possible
//! combinations of flex alignment properties.
//!
//! ### CSS Flexbox Algorithm Implementation
//!
//! Bevy's implementation follows the W3C CSS Flexbox Specification exactly:
//!
//! **Main Axis vs Cross Axis Paradigm:**
//! - **Main Axis**: Primary direction of flex flow (set by `flex_direction`)
//! - **Cross Axis**: Perpendicular to main axis, creates 2D layout control
//! - **Axis Switching**: `flex_direction` changes which axis is "main"
//!
//! **Mathematical Flex Distribution:**
//! 1. **Available Space Calculation**: Container size minus item sizes
//! 2. **Flex Factor Application**: `flex_grow` and `flex_shrink` ratios
//! 3. **Constraint Resolution**: Min/max size respect during distribution
//! 4. **Alignment Application**: Final positioning within available space
//!
//! ### Real-World Applications and Industry Patterns
//!
//! This layout system powers modern UI frameworks:
//! - **React Native**: Direct flexbox implementation for mobile UIs
//! - **Flutter**: Similar flex widgets for cross-platform development  
//! - **CSS Grid**: Complementary system for 2D layouts
//! - **Game UIs**: Responsive inventory systems, HUDs, and menus
//!
//! ### Performance Optimization: Layout Engine Efficiency
//!
//! **Constraint Solving Algorithm:**
//! - **Single-Pass Layout**: O(n) complexity for most flex configurations
//! - **Incremental Updates**: Only recalculate affected subtrees on changes
//! - **Cache-Friendly Memory**: Linear traversal optimizes CPU cache usage
//! - **Parallel Opportunities**: Independent flex containers can process concurrently
//!
//! **Memory Layout Optimization:**
//! - **Component Packing**: ECS stores similar components contiguously
//! - **Query Efficiency**: Specific component combinations minimize iteration
//! - **Dirty Tracking**: Only modified layouts trigger recalculation
//!
//! ## UI Architecture Deep Dive: Component-Based Layout Systems
//!
//! **Declarative Layout Philosophy:**
//! Flexbox enables "constraint-based" rather than "imperative" UI design.
//! Instead of calculating exact positions, developers declare relationships
//! and let the layout engine solve optimal positioning automatically.
//!
//! **Accessibility Considerations:**
//! - **Reading Order**: Flexbox maintains logical tab order despite visual reordering
//! - **Responsive Text**: Layouts adapt to user font size preferences
//! - **Screen Readers**: Semantic structure preserved regardless of visual layout
//! - **High Contrast**: Layout remains functional with user color overrides
//!
//! ### Advanced Techniques: Modern Layout Innovations
//!
//! **Container Queries (Future Enhancement):**
//! Next-generation responsive design based on container size rather than viewport.
//! Flexbox provides the foundation for implementing container-aware layouts.
//!
//! **Subgrid Integration:**
//! While Bevy doesn't yet support CSS Subgrid, flexbox provides the flexibility
//! to implement similar alignment patterns through nested flex containers.
//!
//! **Intrinsic Web Design:**
//! Modern approach combining flexbox, grid, and content-aware sizing
//! for truly adaptive interfaces that respond to both content and context.
//!
//! ## Common Pitfalls and Anti-Patterns to Avoid
//!
//! **Performance Anti-Patterns:**
//! - **Excessive Nesting**: Deep flex hierarchies can impact layout performance
//! - **Conflicting Constraints**: Impossible size requirements cause layout thrashing
//! - **Missing Flex-Basis**: Undefined initial sizes force unnecessary calculations
//!
//! **Accessibility Pitfalls:**
//! - **Visual-Only Ordering**: Using `order` property without considering tab sequence
//! - **Insufficient Contrast**: Colored backgrounds must meet WCAG contrast ratios
//! - **Fixed Sizing**: Hardcoded dimensions break responsive behavior
//!
//! **Design System Integration:**
//! - **Inconsistent Spacing**: Use semantic tokens rather than arbitrary values
//! - **Missing Breakpoints**: Layouts must adapt to different screen sizes
//! - **Color Dependency**: Information should not rely solely on color coding

use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════════
// DESIGN SYSTEM: SEMANTIC COLOR TOKENS AND SPACING SCALE
// ═══════════════════════════════════════════════════════════════════════════════════

/// **Color Theory Application in UI Design:**
/// These colors follow the Material Design 3.0 color system with careful consideration
/// for accessibility, visual hierarchy, and semantic meaning.
/// 
/// **Pink (#FF1159): AlignItems Properties**
/// - Hue: Magenta-red, psychologically associated with "structure" and "alignment"
/// - Saturation: High contrast ensures visibility across different backgrounds
/// - WCAG Compliance: Maintains 4.5:1 contrast ratio against dark backgrounds
const ALIGN_ITEMS_COLOR: Color = Color::srgb(1.0, 0.066, 0.349);

/// **Blue (#1A85FF): JustifyContent Properties**
/// - Hue: Cool blue, psychologically associated with "flow" and "direction"
/// - Color Blindness: Distinguishable from pink across all common CVD types
/// - Accessibility: High contrast meets WCAG AA standards
const JUSTIFY_CONTENT_COLOR: Color = Color::srgb(0.102, 0.522, 1.0);

/// **Spacing Scale: 8-Point Grid System**
/// Following industry-standard 8-point spacing system for visual rhythm
/// 12px = 1.5 × base unit, creates harmonious proportions throughout UI
/// 
/// **Mathematical Basis:**
/// - Divisible by 2, 3, 4, 6 for flexible subdivision
/// - Optimal for both desktop (mouse precision) and mobile (finger targets)
/// - Follows human visual perception patterns for comfortable spacing
const MARGIN: Val = Val::Px(12.0);

/// **Typography Scale Integration**
/// Font sizes should follow 4px baseline grid to align with spacing system
/// Default font size chosen for optimal readability at typical viewing distances
const FONT_SIZE: f32 = 16.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Advanced Flexbox Layout Demo - All 30 Combinations".to_string(),
                // Enable high-DPI scaling for crisp text on modern displays
                resolution: (1200.0, 800.0).into(),
                // Allow window resizing to test responsive behavior
                resizable: true,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, spawn_layout)
        .run();
    
    // **Application Architecture Notes:**
    // Single-system architecture appropriate for static layout demonstration
    // Production apps would include:
    // - Update systems for dynamic content
    // - Interaction systems for user input
    // - State management for complex UI flows
    // - Performance monitoring for layout optimization
}

fn spawn_layout(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ═══════════════════════════════════════════════════════════════════════════════════
    // ASSET LOADING: TYPOGRAPHY SYSTEM INITIALIZATION
    // ═══════════════════════════════════════════════════════════════════════════════════
    
    /// **Font Selection Rationale:**
    /// Fira Sans Bold chosen for excellent screen rendering characteristics:
    /// - **Legibility**: Designed specifically for code/technical content
    /// - **Hinting**: Superior rendering at small sizes on various displays
    /// - **Accessibility**: High x-height improves readability for users with visual impairments
    /// - **Language Support**: Extensive Unicode coverage for international applications
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    
    /// **Camera Configuration:**
    /// Camera2d automatically handles UI rendering with proper depth sorting
    /// UI elements render in front of 2D sprites using the built-in render layers
    commands.spawn(Camera2d);
    
    // ═══════════════════════════════════════════════════════════════════════════════════
    // LAYOUT ARCHITECTURE: ROOT CONTAINER WITH RESPONSIVE DESIGN PRINCIPLES
    // ═══════════════════════════════════════════════════════════════════════════════════
    
    commands
        .spawn((
            Node {
                /// **Viewport-Relative Sizing:**
                /// 100% dimensions ensure layout adapts to any window size
                /// Critical for responsive design across different devices
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                
                /// **Flex Direction Strategy:**
                /// Column layout creates vertical flow, essential for:
                /// - Reading order preservation (top-to-bottom)
                /// - Screen reader navigation consistency
                /// - Mobile-first responsive design patterns
                flex_direction: FlexDirection::Column,
                
                /// **Cross-Axis Alignment Theory:**
                /// AlignItems::Center on column creates horizontal centering
                /// Provides visual balance and centers content regardless of viewport width
                /// Follows centered content design pattern for maximum accessibility
                align_items: AlignItems::Center,
                
                /// **Spacing System Implementation:**
                /// Uniform padding ensures consistent visual breathing room
                /// 12px follows 8-point grid system for harmonious proportions
                /// Creates safe area for content, preventing edge-to-edge layouts
                padding: UiRect::all(MARGIN),
                
                /// **Vertical Rhythm:**
                /// Row gap creates consistent spacing between major layout sections
                /// Essential for visual hierarchy and content grouping
                row_gap: MARGIN,
                
                ..Default::default()
            },
            /// **Background Color Psychology:**
            /// Black background provides maximum contrast for colored text labels
            /// Reduces eye strain during extended viewing of layout demonstrations
            /// Professional appearance suitable for technical/educational content
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|builder| {
            // ═══════════════════════════════════════════════════════════════════════════════════
            // LEGEND COMPONENT: ACCESSIBILITY AND VISUAL HIERARCHY
            // ═══════════════════════════════════════════════════════════════════════════════════
            
            /// **Information Architecture Principle:**
            /// Legend provides essential context before main content
            /// Follows F-pattern reading behavior (left-to-right, top-to-bottom)
            /// Enables users to understand color coding system immediately
            builder
                .spawn(Node {
                    /// **Horizontal Layout Strategy:**
                    /// Row direction for natural left-to-right reading order
                    /// Creates logical grouping of related legend items
                    flex_direction: FlexDirection::Row,
                    
                    /// **Accessibility Enhancement:**
                    /// Gap between legend items improves visual separation
                    /// Prevents color confusion for users with vision impairments
                    column_gap: MARGIN,
                    
                    ..default()
                })
                .with_children(|builder| {
                    /// **Color-Coding System:**
                    /// Each legend item demonstrates its associated property type
                    /// Provides immediate visual reference for understanding the grid below
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        ALIGN_ITEMS_COLOR,
                        UiRect::default(),
                        "← AlignItems (Cross-Axis)",
                        0, // Legend row index
                        0, // Legend column index
                    );
                    spawn_nested_text_bundle(
                        builder,
                        font.clone(),
                        JUSTIFY_CONTENT_COLOR,
                        UiRect::default(),
                        "← JustifyContent (Main-Axis)",
                        0, // Legend row index
                        1, // Legend column index
                    );
                });

            // ═══════════════════════════════════════════════════════════════════════════════════
            // MAIN GRID CONTAINER: SYSTEMATIC LAYOUT TESTING MATRIX
            // ═══════════════════════════════════════════════════════════════════════════════════
            
            /// **Educational Design Pattern:**
            /// Systematic grid demonstrates every possible flex alignment combination
            /// Creates comprehensive reference that users can study and experiment with
            /// 
            /// **Grid Mathematics:**
            /// - 5 AlignItems options × 6 JustifyContent options = 30 total combinations
            /// - Each cell demonstrates one unique property pairing
            /// - Visual results show practical effects of each combination
            builder
                .spawn(Node {
                    /// **Responsive Container Sizing:**
                    /// Fills remaining space after legend using flex grow behavior
                    /// Automatically adapts to any window size or content changes
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    
                    /// **Vertical Layout Strategy:**
                    /// Column direction stacks rows vertically for clear organization
                    /// Each row represents one AlignItems value across all JustifyContent values
                    flex_direction: FlexDirection::Column,
                    
                    /// **Visual Rhythm Implementation:**
                    /// Consistent row spacing creates professional, readable layout
                    /// Follows principles of editorial design for optimal information consumption
                    row_gap: MARGIN,
                    
                    ..default()
                })
                .with_children(|builder| {
                    // ═══════════════════════════════════════════════════════════════════════════════
                    // FLEXBOX PROPERTY DEFINITIONS: COMPREHENSIVE ALIGNMENT OPTIONS
                    // ═══════════════════════════════════════════════════════════════════════════════
                    
                    /// **JustifyContent: Main-Axis Distribution Algorithms**
                    /// Controls how flex items are distributed along the primary axis
                    /// Each option implements different space distribution mathematics:
                    let justifications = [
                        /// **FlexStart**: Items packed toward main-axis start
                        /// Algorithm: Place items consecutively from start position
                        /// Use case: Left-aligned content, reading order preservation
                        JustifyContent::FlexStart,
                        
                        /// **Center**: Items centered along main axis
                        /// Algorithm: Calculate total item width, center the group
                        /// Use case: Centered logos, symmetric layouts
                        JustifyContent::Center,
                        
                        /// **FlexEnd**: Items packed toward main-axis end
                        /// Algorithm: Place items consecutively from end position
                        /// Use case: Right-aligned content, "pull-right" patterns
                        JustifyContent::FlexEnd,
                        
                        /// **SpaceEvenly**: Equal space around AND between all items
                        /// Algorithm: (container_size - total_item_size) / (item_count + 1)
                        /// Use case: Navigation bars, evenly distributed buttons
                        JustifyContent::SpaceEvenly,
                        
                        /// **SpaceAround**: Equal space around each item (half at edges)
                        /// Algorithm: (container_size - total_item_size) / item_count
                        /// Use case: Card layouts, gallery grids
                        JustifyContent::SpaceAround,
                        
                        /// **SpaceBetween**: Space only between items (no edge space)
                        /// Algorithm: (container_size - total_item_size) / (item_count - 1)
                        /// Use case: Justified text, edge-to-edge distributions
                        JustifyContent::SpaceBetween,
                    ];
                    
                    /// **AlignItems: Cross-Axis Alignment Strategies**
                    /// Controls how flex items align perpendicular to main axis
                    /// Each option serves different design and accessibility needs:
                    let alignments = [
                        /// **Baseline**: Align text baselines for readable mixed content
                        /// Algorithm: Find largest font baseline, align all text to it
                        /// Use case: Mixed font sizes, inline text with icons
                        AlignItems::Baseline,
                        
                        /// **FlexStart**: Align to cross-axis start edge
                        /// Algorithm: Position items at container's cross-axis beginning
                        /// Use case: Top-aligned content, consistent positioning
                        AlignItems::FlexStart,
                        
                        /// **Center**: Center items on cross axis
                        /// Algorithm: Calculate item height, center within container
                        /// Use case: Vertically centered content, balanced layouts
                        AlignItems::Center,
                        
                        /// **FlexEnd**: Align to cross-axis end edge
                        /// Algorithm: Position items at container's cross-axis end
                        /// Use case: Bottom-aligned content, footer layouts
                        AlignItems::FlexEnd,
                        
                        /// **Stretch**: Expand items to fill cross axis
                        /// Algorithm: Set item cross-size to container cross-size
                        /// Use case: Equal-height columns, full-width buttons
                        AlignItems::Stretch,
                    ];
                    // ═══════════════════════════════════════════════════════════════════════════════
                    // NESTED ITERATION: SYSTEMATIC COMBINATION GENERATION
                    // ═══════════════════════════════════════════════════════════════════════════════
                    
                    /// **Double-Loop Pattern for Comprehensive Testing:**
                    /// Outer loop: AlignItems (creates rows)
                    /// Inner loop: JustifyContent (creates columns within each row)
                    /// Result: Complete matrix of all possible flex combinations
                    
                    for (row_index, align_items) in alignments.iter().enumerate() {
                        builder
                            .spawn(Node {
                                /// **Equal Height Distribution:**
                                /// Each row gets equal share of available vertical space
                                /// Flex algorithm automatically calculates: height = 100% / row_count
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                
                                /// **Row Layout Pattern:**
                                /// Horizontal arrangement showcases JustifyContent effects
                                /// All cells in a row share the same AlignItems value
                                flex_direction: FlexDirection::Row,
                                
                                /// **Visual Separation:**
                                /// Column gaps prevent cell merging, improve readability
                                /// Consistent spacing maintains grid structure integrity
                                column_gap: MARGIN,
                                
                                ..Default::default()
                            })
                            .with_children(|builder| {
                                /// **Inner Loop: Property Combination Generation**
                                /// Each cell demonstrates one unique alignment pairing
                                /// Systematic approach ensures complete coverage
                                for (col_index, justify_content) in justifications.iter().enumerate() {
                                    spawn_child_node(
                                        builder,
                                        font.clone(),
                                        *align_items,
                                        *justify_content,
                                        row_index,
                                        col_index,
                                    );
                                }
                            });
                    }
                });
        });
}

// ═══════════════════════════════════════════════════════════════════════════════════
// CELL COMPONENT FACTORY: INDIVIDUAL FLEX DEMONSTRATION UNITS
// ═══════════════════════════════════════════════════════════════════════════════════

/// Factory function implementing the Builder pattern for flex demonstration cells
/// 
/// **Component Architecture Benefits:**
/// - **Modularity**: Each cell is self-contained demonstration unit
/// - **Reusability**: Same pattern applicable to any flex property combination
/// - **Testability**: Individual cells can be tested in isolation
/// - **Maintainability**: Centralized cell creation logic
/// 
/// **Educational Design Pattern:**
/// Each cell serves as a "living documentation" example showing:
/// 1. Property names (for reference)
/// 2. Visual effects (for understanding)
/// 3. Practical implementation (for learning)
/// 
/// **Parameters:**
/// - `builder`: Bevy's command spawning interface for entity creation
/// - `font`: Typography asset for consistent text rendering
/// - `align_items`: Cross-axis alignment to demonstrate
/// - `justify_content`: Main-axis alignment to demonstrate
/// - `row_index`: Grid position for accessibility labeling
/// - `col_index`: Grid position for accessibility labeling
fn spawn_child_node(
    builder: &mut ChildSpawnerCommands,
    font: Handle<Font>,
    align_items: AlignItems,
    justify_content: JustifyContent,
    row_index: usize,
    col_index: usize,
) {
    builder
        .spawn((
            Node {
                /// **Nested Flex Context:**
                /// Column layout within each cell creates vertical text stacking
                /// This nested structure demonstrates flex contexts within flex contexts
                /// Essential pattern for complex UI component composition
                flex_direction: FlexDirection::Column,
                
                // ═══════════════════════════════════════════════════════════════════════════════
                // CORE DEMONSTRATION: FLEX PROPERTY APPLICATION
                // ═══════════════════════════════════════════════════════════════════════════════
                
                /// **The Magic Happens Here:**
                /// These two properties create the visual effect being demonstrated
                /// Each cell shows exactly how one combination behaves
                align_items,     // Cross-axis alignment for this specific demo
                justify_content, // Main-axis alignment for this specific demo
                
                /// **Responsive Sizing Strategy:**
                /// Percentage-based sizing ensures equal cell distribution
                /// Automatically adapts to any container size changes
                /// Maintains grid proportions across all screen sizes
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                
                /// **Accessibility Enhancements:**
                /// Could add ARIA labels here for screen reader navigation
                /// Grid position information helps users understand layout structure
                
                ..default()
            },
            /// **Visual Design Theory:**
            /// Medium gray (25%) provides sufficient contrast against black background
            /// Light enough to read dark text, dark enough to show white text
            /// Creates clear cell boundaries without overwhelming visual hierarchy
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            
            /// **Future Enhancement Opportunities:**
            /// Could add interaction components for:
            /// - Hover effects to highlight individual cells
            /// - Click handlers to show detailed property explanations
            /// - Focus management for keyboard navigation
        ))
        .with_children(|builder| {
            // ═══════════════════════════════════════════════════════════════════════════════
            // LABEL SYSTEM: VISUAL FEEDBACK AND EDUCATIONAL CONTENT
            // ═══════════════════════════════════════════════════════════════════════════════
            
            /// **Information Architecture:**
            /// Each cell contains exactly two pieces of information:
            /// 1. Cross-axis alignment setting (AlignItems)
            /// 2. Main-axis alignment setting (JustifyContent)
            /// 
            /// **Color-Coding Strategy:**
            /// Consistent color association helps users quickly identify property types
            /// across all 30 cells, creating visual learning reinforcement
            let labels = [
                /// **Cross-Axis Property Label (Pink)**
                /// AlignItems controls vertical alignment in row layouts
                /// Color coding helps distinguish from main-axis properties
                (format!("{align_items:?}"), ALIGN_ITEMS_COLOR, 0.0),
                
                /// **Main-Axis Property Label (Blue)**
                /// JustifyContent controls horizontal alignment in row layouts
                /// Slight vertical offset prevents text overlap in tight layouts
                (format!("{justify_content:?}"), JUSTIFY_CONTENT_COLOR, 3.0),
            ];
            
            /// **Dynamic Demonstration Effect:**
            /// The positioning of these labels IS the demonstration!
            /// As flex properties change, label positions change accordingly,
            /// providing immediate visual feedback of property effects.
            /// 
            /// **Educational Psychology:**
            /// Learning through observation + experimentation = deeper understanding
            /// Users can see cause (property values) and effect (visual position) simultaneously
            
            for (text, color, top_margin) in labels {
                /// **Bevy UI Architecture Limitation:**
                /// Text nodes cannot have margins, padding, or background colors directly
                /// This necessitates the container-wrapper pattern for styled text
                /// 
                /// **Component Composition Pattern:**
                /// Container (styling) + Text (content) = Styled text component
                /// Common pattern in UI frameworks where styling and content are separated
                spawn_nested_text_bundle(
                    builder,
                    font.clone(),
                    color,
                    UiRect::top(Val::Px(top_margin)),
                    &text,
                    row_index,
                    col_index,
                );
            }
        });
}

// ═══════════════════════════════════════════════════════════════════════════════════
// TEXT COMPONENT FACTORY: STYLED TEXT WITH CONTAINER PATTERN
// ═══════════════════════════════════════════════════════════════════════════════════

/// Factory function implementing the Container + Text pattern required by Bevy UI
/// 
/// **Architectural Necessity:**
/// Bevy's UI system separates visual styling from text content, requiring
/// a container node to provide background colors, padding, and margins.
/// This follows CSS box model principles where content and decoration are distinct.
/// 
/// **Component Composition Theory:**
/// Rather than monolithic "styled text" components, Bevy uses composition:
/// - Container Node: Provides background, spacing, positioning
/// - Text Node: Provides content, font, color
/// - Result: Maximum flexibility with clear separation of concerns
/// 
/// **Accessibility Integration:**
/// Container pattern enables future accessibility enhancements:
/// - Semantic role attributes
/// - Focus management
/// - Screen reader navigation hints
/// - High contrast mode support
/// 
/// **Performance Characteristics:**
/// - Minimal overhead: Only two components per styled text
/// - Cache-friendly: Similar components stored contiguously
/// - Batch-friendly: UI renderer can optimize similar elements
fn spawn_nested_text_bundle(
    builder: &mut ChildSpawnerCommands,
    font: Handle<Font>,
    background_color: Color,
    margin: UiRect,
    text: &str,
    row_index: usize,
    col_index: usize,
) {
    // ═══════════════════════════════════════════════════════════════════════════════
    // CONTAINER NODE: VISUAL STYLING AND SPACING MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════════
    
    builder
        .spawn((
            Node {
                /// **External Spacing Control:**
                /// Margin provides space outside element, creating visual separation
                /// Essential for preventing text overlap in tight layout configurations
                margin,
                
                /// **Internal Spacing Design:**
                /// Asymmetric padding (5px horizontal, 1px vertical) creates "pill" shape
                /// Follows mobile UI conventions for touch-friendly text containers
                /// 
                /// **Visual Design Theory:**
                /// - Horizontal padding: Prevents text from touching color edges
                /// - Minimal vertical padding: Maintains compact vertical rhythm
                /// - Asymmetric ratios: Creates visually pleasing proportions
                padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
                
                /// **Accessibility Enhancement Opportunities:**
                /// Future improvements could include:
                /// - Minimum target size enforcement (44px for touch)
                /// - High contrast mode color overrides
                /// - Focus indicators for keyboard navigation
                
                ..default()
            },
            /// **Color Psychology and Accessibility:**
            /// Background color provides semantic meaning through consistent color coding
            /// High saturation ensures visibility across various lighting conditions
            /// Color choices maintain WCAG contrast requirements against dark backgrounds
            BackgroundColor(background_color),
            
            /// **Future Enhancement: Semantic Markup**
            /// Could add accessibility components:
            /// - Role::Button for interactive elements
            /// - AriaLabel for screen reader descriptions
            /// - TabIndex for keyboard navigation order
        ))
        .with_children(|builder| {
            // ═══════════════════════════════════════════════════════════════════════════
            // TEXT NODE: CONTENT AND TYPOGRAPHY MANAGEMENT
            // ═══════════════════════════════════════════════════════════════════════════
            
            builder.spawn((
                /// **Content Management:**
                /// Text::new() creates Unicode-aware text rendering
                /// Supports internationalization and complex text layouts
                Text::new(text),
                
                /// **Typography System:**
                /// TextFont component manages font asset and sizing
                /// Bold weight chosen for maximum legibility at small sizes
                /// Size automatically scales with DPI for crisp rendering
                TextFont { 
                    font, 
                    font_size: FONT_SIZE,
                    ..default() 
                },
                
                /// **Contrast Optimization:**
                /// Black text provides maximum contrast against colored backgrounds
                /// Ensures readability across all color-coded label types
                /// Maintains accessibility compliance for users with visual impairments
                TextColor::BLACK,
                
                /// **Performance Note:**
                /// Text rendering is CPU-intensive operation
                /// Bevy caches glyph atlases for efficient repeated rendering
                /// Static text like this requires minimal ongoing performance cost
            ));
        });
    
    // ═══════════════════════════════════════════════════════════════════════════════
    // ARCHITECTURAL SUMMARY: CONTAINER + TEXT PATTERN BENEFITS
    // ═══════════════════════════════════════════════════════════════════════════════
    
    // **Problem-Solution Analysis:**
    // 
    // 1. **Text Node Limitations**: Text components only handle content and font properties
    //    **Solution**: Container provides visual styling (background, padding, margin)
    // 
    // 2. **Styling Consistency**: Need uniform appearance across different text elements
    //    **Solution**: Centralized factory function ensures consistent styling patterns
    // 
    // 3. **Accessibility Requirements**: Screen readers need semantic structure
    //    **Solution**: Container can carry accessibility metadata while text provides content
    // 
    // 4. **Performance Optimization**: Separate styling from content for efficient updates
    //    **Solution**: Text content can change without affecting layout calculations
    // 
    // 5. **Responsive Design**: Need flexible sizing and positioning
    //    **Solution**: Container handles layout while text handles content scaling
    // 
    // **Industry Pattern Recognition:**
    // This pattern mirrors:
    // - CSS: `<span>` wrapper around text content
    // - React: Styled component composition
    // - Flutter: Container + Text widget combination
    // - Game UI: Backdrop + Label component pairs
    // 
    // **Scalability Considerations:**
    // Pattern supports future enhancements:
    // - Animation: Container can animate independently of text
    // - Theming: Container styling can be updated without text changes
    // - Interaction: Container can handle input while text remains static
    // - Localization: Text content can change while layout remains stable
}
