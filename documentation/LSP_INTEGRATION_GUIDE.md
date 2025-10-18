# LSP Integration Guide for Marco Editor

## Overview

This document outlines the strategy for integrating Marco's custom Language Server Protocol (LSP) implementation into the GtkSourceView5 editor with GTK4. Marco has a pure Rust LSP implementation in the `core` crate that provides syntax highlighting, autocompletion, diagnostics, and hover information specifically designed for Markdown editing.

## Marco's LSP Architecture

### Current Implementation

Marco's LSP features are located in `core/src/lsp/` and include:

- **Syntax Highlighting** (`highlights.rs`) - 11 highlight types for Markdown elements
- **Autocompletion** (`completion.rs`) - Context-aware suggestions for Markdown syntax
- **Diagnostics** (`diagnostics.rs`) - Parse validation with 4 severity levels
- **Hover Information** (`hover.rs`) - Contextual information display (stub)

### Key Characteristics

- **Parser-Based**: Directly uses Marco's nom-based Markdown parser AST
- **Lightweight**: No external LSP server process required
- **Synchronous**: Operates in the main thread with GTK event loop
- **Markdown-Specific**: Tailored for Markdown editing workflows

## GtkSourceView5 Integration Points

### 1. Syntax Highlighting System

#### GtkSourceView Built-in Approach

GtkSourceView provides two primary methods for syntax highlighting:

**XML Language Definitions**
- Standard approach using `GtkSourceLanguage`
- Language files define syntax patterns in XML format
- Located in `~/.local/share/gtksourceview-5/language-specs/`
- Pattern-based matching using regex and nested contexts

**Limitations for Marco**
- Cannot access parser AST information
- No awareness of document structure
- Limited to regex-based pattern matching
- Cannot provide semantic highlighting

#### Custom Tag-Based Approach

**GtkTextTag System**
- GtkSourceBuffer extends GtkTextBuffer with tag support
- Tags can be created programmatically and applied to text ranges
- Each tag defines visual attributes (color, weight, style, underline)
- Tags override XML language definition styling

**Implementation Strategy**
1. Create custom GtkTextTag for each highlight type
2. Compute highlights from Marco's LSP on buffer changes
3. Apply/remove tags dynamically based on AST analysis
4. Use `gtk_text_buffer_apply_tag()` and `gtk_text_buffer_remove_tag()`

**Tag Attributes**
- Foreground/background colors
- Font weight (bold, normal)
- Font style (italic, oblique)
- Underline style (single, double, error, warning)
- Strikethrough
- Scale (for headings)

#### Signal Integration

**Key Signals to Connect**
- `changed` - Buffer content modified, recompute highlights
- `insert-text` - Text inserted, incremental update
- `delete-range` - Text deleted, incremental update
- `highlight-updated` - GtkSourceView's built-in signal

**Update Strategy**
- Parse document on buffer change
- Compute highlight regions from AST
- Remove old tags in changed region
- Apply new tags to updated ranges
- Batch updates to avoid flicker

### 2. Autocompletion System

#### GtkSourceCompletion Architecture

**Core Components**
- `GtkSourceCompletion` - Manages completion popups
- `GtkSourceCompletionProvider` - Interface for suggestion sources
- `GtkSourceCompletionProposal` - Individual suggestion item
- `GtkSourceCompletionContext` - Request context with cursor position

**Provider Interface Methods**
- `populate_async` - Asynchronously fetch completion suggestions
- `activate` - User selected a suggestion, insert into buffer
- `display` - Customize how suggestion appears in popup
- `is_trigger` - Determine if character triggers completion
- `get_priority` - Provider precedence when multiple match
- `refilter` - Update suggestions as user types

#### Marco Integration Approach

**Custom Completion Provider**
1. Implement `GtkSourceCompletionProvider` interface in Rust
2. Call Marco's `get_completions()` in `populate_async`
3. Convert Marco's `CompletionItem` to GtkSourceView proposals
4. Return suggestions as `GListModel` of proposals

**Trigger Characters**
- `#` - Heading suggestions
- `` ` `` - Code span/block suggestions
- `*` - Emphasis/strong suggestions
- `_` - Emphasis/strong suggestions (alternative)
- `[` - Link suggestions

**Completion Item Properties**
- Label - Display text in popup
- Info - Additional documentation/preview
- Icon - Optional icon for item type
- Insert text - Actual text to insert
- Cursor offset - Where to place cursor after insertion

**User Interaction Flow**
1. User types trigger character
2. Provider checks context (cursor position, surrounding text)
3. Marco's LSP computes valid completions
4. GtkSourceView displays popup with suggestions
5. User selects item or continues typing to filter
6. Selection activates provider's insert logic

### 3. Diagnostics and Error Display

#### Diagnostic Rendering Approaches

**GtkTextTag for Underlines**
- Create tags with underline properties
- Error - Squiggly red underline
- Warning - Squiggly yellow/orange underline
- Info - Dotted blue underline
- Hint - Dotted grey underline

**GtkSourceMark for Gutter Icons**
- Place marks at diagnostic line positions
- Associate mark category with icon
- Display in gutter column next to line numbers
- Multiple marks per line supported

**Implementation Strategy**
1. Compute diagnostics from parser on buffer change
2. Create diagnostic tags if not exist
3. Apply underline tags to error ranges
4. Create source marks for gutter icons
5. Store diagnostic data for hover tooltips

#### Diagnostic Lifecycle

**Update Trigger Points**
- After user stops typing (debounced)
- On file save
- On explicit validation request
- Background re-parse completion

**Visual Feedback Timing**
- Immediate for syntax errors (as user types)
- Debounced for warnings (500ms after typing stops)
- On-demand for hints (explicit action)

**Severity Mapping**
- Error → Red squiggly underline + red gutter icon
- Warning → Yellow squiggly underline + yellow gutter icon
- Info → Blue dotted underline + blue gutter icon
- Hint → Grey dotted underline + grey gutter icon

### 4. Hover Information System

#### GtkSourceHover Architecture

**Core Components**
- `GtkSourceHover` - Manages hover tooltips
- `GtkSourceHoverProvider` - Interface for hover content
- `GtkSourceHoverDisplay` - Popup widget showing information
- `GtkSourceHoverContext` - Request context with cursor position

**Provider Interface Methods**
- `populate_async` - Fetch hover content for position
- `populate_finish` - Complete async operation
- Display content can be text, markup, or custom widget

#### Marco Integration Approach

**Custom Hover Provider**
1. Implement `GtkSourceHoverProvider` interface
2. Call Marco's `get_hover_info()` with cursor position
3. Convert hover result to display markup or widget
4. Return content to GtkSourceHoverDisplay

**Hover Information Types**
- Syntax element description
- Link URL preview
- Image alt text display
- Code block language info
- Diagnostic details (when hovering over error)

**Content Formatting**
- Markdown formatting in hover popup
- Syntax highlighting for code examples
- Clickable links
- Multi-line formatted text

**Activation Settings**
- Hover delay (configurable, default 500ms)
- Modifier keys (Ctrl+hover for extended info)
- Mouse vs keyboard hover behavior

## Implementation Architecture

### Component Structure

#### Core LSP Layer

**Location**: `core/src/lsp/`

**Responsibilities**
- Parse Markdown to AST
- Compute highlight regions
- Generate completion suggestions
- Validate document and create diagnostics
- Provide hover information

**No GTK Dependencies**
- Pure Rust business logic
- Testable in isolation
- Reusable for other editor integrations

#### GTK Integration Layer

**Location**: `marco/src/components/editor/lsp_integration.rs`

**Responsibilities**
- Bridge between GTK and Core LSP
- Implement GtkSourceView provider interfaces
- Manage GTK-specific objects (tags, marks, signals)
- Handle async operations with GTK main loop
- Convert between GTK and Core data types

#### Editor UI Layer

**Location**: `marco/src/components/editor/editor_ui.rs`

**Responsibilities**
- Initialize LSP providers
- Connect providers to GtkSourceView
- Manage buffer change signals
- Configure completion and hover behavior
- Handle user preferences for LSP features

### Data Flow

#### Highlighting Flow

1. User edits buffer → `changed` signal emitted
2. Signal handler reads buffer content
3. Core parser converts text to AST
4. Core LSP computes highlight regions from AST
5. GTK integration removes old tags in changed region
6. GTK integration applies new tags to ranges
7. GtkSourceView renders updated highlighting

#### Completion Flow

1. User types trigger character → completion triggered
2. GtkSourceCompletion creates context with cursor position
3. Marco's provider `populate_async` called
4. Provider reads buffer text and cursor position
5. Core LSP computes completions based on context
6. Provider converts to GtkSourceView proposals
7. GtkSourceCompletion displays popup
8. User selects → provider inserts text into buffer

#### Diagnostic Flow

1. Buffer content changed → debounced timer started
2. Timer expires → validation triggered
3. Core parser validates document
4. Core LSP generates diagnostics
5. GTK integration creates/updates diagnostic tags
6. GTK integration creates/updates source marks
7. GtkSourceView displays underlines and gutter icons
8. User hovers over diagnostic → hover provider shows details

#### Hover Flow

1. User hovers over text → hover delay timer started
2. Timer expires → hover triggered
3. GtkSourceHover creates context with cursor position
4. Marco's provider `populate_async` called
5. Provider reads buffer text and determines element
6. Core LSP provides hover information
7. Provider formats content with markup
8. GtkSourceHoverDisplay shows popup near cursor

## User Experience Considerations

### Performance Optimization

**Incremental Updates**
- Only re-highlight changed regions, not entire document
- Use rope data structure for efficient text operations
- Cache AST nodes that haven't changed
- Debounce expensive operations

**Background Processing**
- Parse large documents in background thread
- Stream results back to main thread
- Show progress indicator for slow operations
- Cancel outdated operations when new changes arrive

**Memory Management**
- Limit cache size for large documents
- Garbage collect unused tags and marks
- Reuse tag objects instead of creating new ones
- Profile memory usage with moka cache statistics

### Accessibility

**Keyboard Navigation**
- Tab through completion suggestions
- Escape to dismiss popups
- Arrow keys to navigate diagnostics
- Keyboard shortcuts for hover display

**Screen Reader Support**
- Announce completion popup opened
- Read suggestion labels and descriptions
- Announce diagnostic severity and message
- Provide text alternatives for icons

**Visual Feedback**
- High contrast mode support
- Configurable colors for diagnostics
- Adjustable underline thickness
- Alternative visual indicators beyond color

### Configuration Options

**User Preferences**
- Enable/disable syntax highlighting
- Enable/disable autocompletion
- Enable/disable diagnostics
- Enable/disable hover
- Adjust timing delays (hover, completion trigger)
- Customize diagnostic severity display
- Choose diagnostic underline style

**Theme Integration**
- Respect GTK theme colors
- Use SourceView style scheme colors
- Provide light/dark mode variants
- Allow custom highlight color overrides

## Integration Checklist

### Phase 1: Syntax Highlighting

- [ ] Create GtkTextTag for each highlight type
- [ ] Connect to buffer `changed` signal
- [ ] Implement tag application logic
- [ ] Handle incremental updates efficiently
- [ ] Test with large documents
- [ ] Profile performance and optimize

### Phase 2: Autocompletion

- [ ] Implement GtkSourceCompletionProvider interface
- [ ] Create custom proposal objects
- [ ] Define trigger characters
- [ ] Implement context detection
- [ ] Handle proposal activation
- [ ] Test all completion types
- [ ] Add keyboard shortcuts

### Phase 3: Diagnostics

- [ ] Create diagnostic underline tags
- [ ] Create gutter icons for severity levels
- [ ] Implement diagnostic computation
- [ ] Add debounced validation
- [ ] Display diagnostics in status bar
- [ ] Implement diagnostic navigation
- [ ] Test error recovery

### Phase 4: Hover Information

- [ ] Implement GtkSourceHoverProvider interface
- [ ] Format hover content with markup
- [ ] Add diagnostic hover details
- [ ] Configure hover delay
- [ ] Support multiple hover providers
- [ ] Test hover positioning
- [ ] Handle edge cases

### Phase 5: Polish

- [ ] Add configuration UI
- [ ] Implement preference persistence
- [ ] Write user documentation
- [ ] Create keyboard shortcut reference
- [ ] Add accessibility features
- [ ] Performance profiling
- [ ] Integration testing

## Technical Challenges

### Challenge 1: Async Integration with GTK

**Problem**: GtkSourceView APIs expect async operations, but Marco's LSP is synchronous.

**Solutions**
- Wrap synchronous calls in async futures
- Use `glib::spawn_future_local` for GTK main loop integration
- Implement cancellation tokens for long operations
- Return immediately for fast operations, async for slow ones

### Challenge 2: Tag Management Overhead

**Problem**: Creating/destroying tags frequently can be expensive.

**Solutions**
- Reuse tag objects with dynamic ranges
- Maintain tag pool for common types
- Batch tag operations before applying
- Use tag priorities to minimize conflicts

### Challenge 3: Incremental Parsing

**Problem**: Full document re-parse is slow for large files.

**Solutions**
- Implement incremental parser that updates only changed nodes
- Use rope data structure for efficient text operations
- Cache unchanged AST subtrees
- Detect affected regions and re-parse only those

### Challenge 4: Multi-Cursor Support

**Problem**: Marco supports multiple cursors, GtkSourceView completion expects single cursor.

**Solutions**
- Track primary cursor for completion
- Apply completion to all cursors if appropriate
- Handle per-cursor completion state
- Test interaction between features

## Testing Strategy

### Unit Tests

- Test each provider interface method independently
- Mock GtkSourceView objects where possible
- Test data conversion functions
- Validate async operation handling

### Integration Tests

- Test complete highlighting pipeline
- Test completion popup and activation
- Test diagnostic display and navigation
- Test hover display and content

### User Acceptance Tests

- Real-world Markdown editing scenarios
- Performance benchmarks with large documents
- Accessibility compliance testing
- Cross-platform testing (Linux, macOS)

## Future Enhancements

### Advanced Features

**Snippet Support**
- Use GtkSourceView's built-in snippet system
- Define Markdown-specific snippets
- Support tab stops and placeholders
- Allow user-defined snippets

**Folding Regions**
- Define foldable regions for headings
- Allow folding of code blocks
- Persist fold state across sessions

**Semantic Actions**
- Quick fixes for common errors
- Refactoring operations (e.g., change heading level)
- Format document actions
- Insert template actions

**Enhanced Hover**
- Preview images inline
- Render Markdown in hover popup
- Show link destinations
- Display reference definitions

## References

### GtkSourceView Documentation

- GtkSourceView 5 API Reference
- Language Definition v2.0 Tutorial
- Completion Provider Interface Documentation
- Hover Provider Interface Documentation

### Marco Architecture

- `core/src/lsp/` - LSP implementation modules
- `core/src/parser/` - Markdown parser
- `marco/src/components/editor/` - Editor UI components

### Related Projects

- GNOME Builder - GtkSourceView LSP integration example
- ThiefMD - Custom Markdown highlighting in GtkSourceView
- Various GtkSourceView-based editors for reference implementations
