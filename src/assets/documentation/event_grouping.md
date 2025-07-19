# Event Grouping in Event Stream Parsers

## Overview
Event grouping allows the parser to emit events for logical groups (e.g., lists, table rows, blocks of related content) to simplify rendering and plugin processing.

## Event Types
- `Event::GroupStart(GroupType, Option<SourcePos>, Option<Attributes>)` — Marks the start of a logical group.
- `Event::GroupEnd(GroupType, Option<SourcePos>, Option<Attributes>)` — Marks the end of a logical group.
- `GroupType` enum includes: `List`, `TableRow`, `BlockGroup` (extensible).

## Usage
- **Emit Group Events:** The parser/emitter emits group events around related content (e.g., before/after a list or table row).
- **Renderer Handling:** Renderers match on group events to wrap output in containers (e.g., `<div class='group-list'>`).
- **Plugin Support:** Plugins can intercept, transform, or batch process entire groups.

## Best Practices
- Group events are additive and do not break existing event handling.
- Use `GroupType` for clarity and extensibility.
- Always include source position and attributes for advanced features.

## Extension Points
- **Plugin Interception:** Plugins can process or transform entire groups.
- **Custom Rendering:** Renderers can style or wrap groups as needed.
- **Event Filtering:** Use the event transform/filter system to modify or suppress group events.

## Example
```rust
// Emitting group events for a list
Event::GroupStart(GroupType::List, Some(SourcePos { line: 10, column: 1 }), None)
// ...emit item events...
Event::GroupEnd(GroupType::List, Some(SourcePos { line: 15, column: 1 }), None)

// Renderer output (HTML)
<div class='group-list'>
  ...list items...
</div>
```

## Testing
- Add tests to verify group event emission and rendering.
- Ensure no regressions or broken code.

---
This system makes rendering, plugin development, and analytics easier and more robust.
