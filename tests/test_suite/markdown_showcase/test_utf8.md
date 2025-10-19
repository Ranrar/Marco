# Test UTF-8 Handling

This file contains various UTF-8 characters to test sanitization:

**Custom Markdown grammar** — hand-crafted parser with nom combinators.

## Multi-byte Characters

- Em dash: —
- En dash: –
- Emoji: 😀 🎉 🚀
- Japanese: こんにちは世界
- Chinese: 你好世界
- Arabic: مرحبا بالعالم
- Hebrew: שלום עולם

## Edge Cases

Quote marks: "Hello" 'World'
Ellipsis: …
Bullet: •
Copyright: ©
Registered: ®
Trademark: ™

This text should load without crashes!
