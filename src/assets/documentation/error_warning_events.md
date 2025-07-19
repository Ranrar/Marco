# Error/Warning/Unsupported Event Handling in Event Stream Parsers

## Overview
Your parser emits structured events for errors, warnings, and unsupported features, allowing renderers and plugins to handle diagnostics gracefully and extensibly.

## Event Types
- `Event::Error(msg, pos)` — for parse errors.
- `Event::Warning(msg, pos)` — for recoverable issues.
- `Event::Unsupported(msg, pos)` — for features not supported by the parser.

Each event includes a message and source position (line/column) for precise diagnostics.

## Usage
- **Emit Early:** Events are emitted as soon as issues are detected during parsing.
- **Source Position Tracking:** All diagnostics include line/column info for accurate reporting.
- **Renderer Handling:** Renderers (e.g., HTML) display diagnostics as styled spans, ensuring graceful degradation.
- **Plugin Support:** Plugins can intercept, filter, transform, or suppress diagnostics events.

## Best Practices
- Avoid panics; emit error/warning events instead of using `.unwrap()` or `.expect()`.
- Always provide source position for diagnostics.
- Ensure renderers and plugins process diagnostics events, not just skip them.

## Extension Points
- **Plugin Interception:** Plugins can filter, transform, or log error/warning events.
- **Custom Rendering:** Renderers can style or display diagnostics as needed.
- **Event Filtering:** Use the event transform/filter system to modify or suppress diagnostics.

## Example
```rust
// Emitting an error event during parsing
Event::Error("Unclosed code block", Some(SourcePos { line: 42, column: 1 }))

// Renderer output (HTML)
<span class='error'>Error: Unclosed code block at line 42, column 1</span>
```

## Testing
- Add tests to verify that error/warning/unsupported events are emitted and handled correctly by renderers and plugins.

---
This system ensures diagnostics are handled gracefully, extensibly, and safely—making your parser resilient and plugin-friendly.
