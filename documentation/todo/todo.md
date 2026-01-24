## Where we are today (software vs hardware rendering)

### Software rendering ("CPU / `SoftwareRenderingContext`")
Status: **Mostly implemented and functional** (I'd call it ~80-90% of a shippable pipeline).

What's already in place:

- rendering.rs
  - `create_rendering_context()` **successfully creates** a `SoftwareRenderingContext` and returns it as `Rc<dyn RenderingContext>`.
- webview.rs
  - Servo initialization creates the rendering context, calls `make_current()`, builds a `servo::WebView` with it.
  - Frame delivery path is implemented:
    - Delegate path: `notify_new_frame_ready()` calls `webview.paint()` → `rendering_context.present()` → `read_to_image()` → cache pixels → `queue_draw()`.
    - GTK path: `snapshot()` draws the cached `RgbaImage` using `gdk::MemoryTexture`.

What's "not done / needs cleanup" for software:

- **Duplicate rendering loops**
  - You currently render+present+read pixels in **two places**:
    1. `WebViewDelegateImpl::notify_new_frame_ready()`
    2. `start_ticker_if_needed()` timeout (60fps) loop
  - This is likely redundant and could cause extra work, stutter, and weird timing. In Servo's embedder model, you typically do:
    - tick/spin loop to drive Servo
    - render only when Servo says "new frame ready"
  - Recommendation (later): keep **spin in ticker**, keep **paint/present/read/cache in delegate**, and remove the extra paint/present/read from the ticker.

- **Sizing mismatch in ticker readback**
  - In ticker you build a `Box2D` based on `widget.width()/height()`, which are logical pixels; Servo rendering contexts are usually **device pixels** (scale factor dependent). Delegate uses `rendering_context.size2d()` which is more authoritative.
  - This can produce "partial frames", wrong stride expectations, or extra rescaling.

- **Lifecycle/state correctness**
  - `start_ticker_if_needed()` sets `self.initialized.set(true)` and logs "Servo initialized successfully", which is misleading (Servo init already happened in `initialize_servo()`).
  - Not a rendering blocker, but worth tightening.

So: software rendering is **real**, producing cached frames, and already wired into GTK snapshotting. It mainly needs **deduplication + correctness polish**.

---

### Hardware rendering ("OpenGL")
Status: **Not implemented yet** (call it ~0-10%, basically scaffolding only).

Evidence:

- rendering.rs:
  - `try_create_opengl_context()` is a stub:
    - `Err("OpenGL not yet implemented")`
  - So **OpenGL is never selected**, meaning `RenderingMode::OpenGL` is effectively dead code right now.
- webview.rs has:
  - `RenderingMode::OpenGL` branches in `snapshot()` (blue fallback)
  - Logging and mode handling
  - …but it will never happen until `try_create_opengl_context()` is real.

What's required to "actually implement hardware rendering":

1. **Create an actual GL-backed `RenderingContext`** compatible with Servo/compositing_traits.
   - The comment mentions **surfman**; that's the usual route in Servo embedders.
2. Decide how GTK4 will display the output:
   - True "hardware path" means *avoid CPU readback* and present GPU content directly.
   - In GTK4 terms, that usually means producing a `gdk::Texture` backed by GL (e.g., `gdk::GLTexture`) or integrating with GTK's GL rendering pipeline.
3. Handle context ownership + threading correctly:
   - Servo's render context expects `make_current()/present()` on the right thread, plus correct resize handling.

Given the current code, you're **not close** to a real GL presentation path yet—there's no surfman usage and no GTK GL texture integration.

---

## "How long will it take?" (realistic effort estimate)

Assuming someone already comfortable with Servo embedder + GTK4 rendering:

### Finish software path cleanly (dedupe, correct sizing, stabilize)
- **Half day to 2 days**.
- Main work: pick *one* render/present/read path, fix scale factor math, reduce unnecessary repainting, keep stable first-frame behavior.

### Implement actual OpenGL hardware rendering (surfman + GTK display)
- **Several days to a couple weeks**, depending on how "true GPU" you want it:
  - **Quick-but-not-true-GPU** approach: use OpenGL context but still `read_to_image()` and show `MemoryTexture` (GPU rasterization + CPU upload). This is easier but not "hardware rendering" in the UI sense.
  - **True GPU presentation** (no readback): more complex; requires correct interop with GTK4's rendering pipeline and textures.

---

## Quick status table

| Area | Current state | Practical completeness |
|---|---|---:|
| Software rendering context creation | Implemented | 100% |
| Software frame production + GTK display | Implemented (but duplicated) | ~80-90% |
| Hardware/OpenGL context creation | Stub | ~0% |
| Hardware/OpenGL presentation in GTK | Not present | ~0% |

If you want, I can do a follow-up review specifically focused on **the cleanest architecture** (who spins the event loop, who paints, who presents, who triggers redraws) and propose the minimal set of changes to make software mode "production clean" while leaving the door open for a future GL path.