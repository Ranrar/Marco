## Review: current architecture (what's good, what's fighting itself)

You already have the *right* Servo embedder shape in place:

- GTK widget lifecycle creates Servo + WebView (`connect_realize` → `initialize_servo`)
- URL load triggers Servo activity (`load_url` → `webview.load`)
- Servo signals new frames (`WebViewDelegate::notify_new_frame_ready`)
- GTK draws via `snapshot()` using a cached pixel buffer

The main architectural issue is that **you currently have two competing "render loops"**:

1. **Delegate-driven rendering (good, Servo-native)**
   - `notify_new_frame_ready()` does:
     - `webview.paint()`
     - `rendering_context.present()`
     - `read_to_image()` → cache pixels
     - schedule `queue_draw()`

2. **Ticker-driven rendering (redundant/conflicting)**
   - `start_ticker_if_needed()` timeout does:
     - `servo.spin_event_loop()`
     - `webview.paint()` (again)
     - `ctx.present()` (again)
     - `read_to_image()` (again, but using widget width/height)
     - `queue_draw()` every 16ms

This duplication is risky because:
- You can paint/present while Servo is *not* ready → "blank" frames, timing weirdness.
- You can read back with mismatched sizes (logical widget px vs device px).
- You're doing a lot more work than needed (every 16ms even if nothing changed).

## Recommended "clean" architecture (minimal change, software-first)

### Principle: **only paint/present/read when Servo says a new frame exists**
Servo's embedder model is effectively: "I will tell you when a frame is ready; you request your platform to redraw."

So:

- **Ticker job**: drive Servo so it can produce events/frames  
  ✅ keep: `servo.spin_event_loop()`

- **Delegate job**: when Servo says "new frame ready", do all rendering work  
  ✅ keep: `paint()` → `present()` → (software: `read_to_image` + cache) → request GTK redraw

- **GTK job**: `snapshot()` only consumes cached output and draws it  
  ✅ keep: draw `MemoryTexture` from cached RGBA

### Concrete minimal edits (conceptual)
In `start_ticker_if_needed()`:
- Remove:
  - `webview.paint()`
  - `ctx.present()`
  - `read_to_image(...)`
- Keep:
  - `servo.spin_event_loop()`
  - maybe `queue_draw()` only if you know Servo is animating (optional; see below)

In `notify_new_frame_ready()`:
- Keep as the *single source of truth* for software frame capture + redraw request.

### Optional (nice-to-have) improvement: redraw only when needed
Servo `WebView` has `animating()` in docs. If you can query `webview.animating()` (or some equivalent), you can:
- run ticker only while animating or while loads are pending
- or keep ticker but stop forcing redraw every 16ms

This will cut CPU use *a lot* in software mode.

## Hardware rendering plan (what "good" looks like)

Right now **OpenGL is scaffolding only**: `try_create_opengl_context()` always errors, so you're always on software.

To make OpenGL real, you'll want to pick one of these milestones:

### Milestone A (fastest, not "true GPU in GTK")
- Use surfman/OpenGL rendering context so Servo renders via GPU
- Still do `read_to_image()` and upload as `MemoryTexture`
- Pros: easier, reuses your current snapshot pipeline
- Cons: still heavy (GPU→CPU readback + CPU→GPU texture upload inside GTK)

### Milestone B (true GPU presentation)
- Create an OpenGL rendering context + share the rendered surface/texture with GTK4
- Draw using a GPU-backed `gdk::Texture` or GTK renderer integration
- Pros: avoids readback, best performance
- Cons: significantly more integration work + platform quirks

Given where the code is today, getting to **Milestone A** is "days", Milestone B is "week(s)".

## Flowchart (current target architecture)

```mermaid
flowchart TD
  A[GTK Widget constructed] --> B[realize signal]
  B --> C[initialize_servo()]
  C --> D[create RenderingContext<br/>(Software now, OpenGL later)]
  D --> E[build Servo WebView<br/>with delegate]
  E --> F[map signal]
  F --> G[queue_draw initial]

  H[load_url()] --> I[webview.load(url)]
  I --> J[start_ticker_if_needed()]
  J --> K[GTK timeout ~16ms]
  K --> L[servo.spin_event_loop()]

  L --> M{Servo has new frame?}
  M -->|yes| N[delegate.notify_new_frame_ready()]
  N --> O[webview.paint()]
  O --> P[rendering_context.present()]

  P --> Q{Rendering mode}
  Q -->|Software| R[read_to_image(viewport)]
  R --> S[cache RGBA frame]
  S --> T[request GTK redraw<br/>invoke/idle -> queue_draw]

  Q -->|OpenGL (future)| U[produce/share GPU texture]
  U --> T

  T --> V[GTK snapshot()]
  V --> W{Cached frame/texture available?}
  W -->|yes| X[draw texture to snapshot]
  W -->|no| Y[draw fallback color]
```

## What's next (if you want me to implement it)
I can implement the minimal cleanup (single render path) safely in webview.rs, then run a release build and a quick smoke test. It should also reduce the chance of timing-related blank frames and lower CPU usage.

## Todo status (delta)
- 1) Review render loop responsibilities — **in progress**
- Next up: 2) Design software render flow — **ready to apply as code changes**
- 4) Flowchart — **done (above)**