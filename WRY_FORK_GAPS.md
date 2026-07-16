# Gaps in `wry` (gtk4-webkit6 fork) blocking full removal of `webkit6` / `webview2-com`

**Status:** Audit only — no code changes proposed here.
**Date:** 2026-07-16
**Scope:** `marco`, `polo` — every direct `webkit6::` and `webview2_com::`/`windows::core::`
call site remaining after the webview unification (`WEBVIEW_MIGRATION.md`).
**Fork:** https://github.com/Ranrar/wry/tree/gtk4-webkit6 (currently pinned to
commit `e00a02d5`)

## 1. Why these deps still exist

`WEBVIEW_MIGRATION.md` unified the preview backend onto `wry` but deliberately
kept two thin direct dependencies as escape hatches:

- `webkit6 = "0.6.1"` (Linux) — for "raw WebKit APIs not exposed through wry"
- `webview2-com` / `windows` / `windows-core` (Windows) — for "unsafe COM call
  sites"

This document catalogs **exactly what those escape hatches are used for**,
checks whether `wry` (the fork, or upstream) has since grown an equivalent,
and states what would need to land upstream/in the fork before each one could
be deleted.

## 2. Audit: every direct call site

| # | File | API used | Purpose |
|---|---|---|---|
| L1 | `marco/src/components/viewer/platform_webview.rs:545,567,931` | `webkit6::prelude::WebViewExt::load_html(view, html, base_uri)` | Load preview HTML with a **base URI** so relative `file://` image/asset paths resolve |
| L1 | `polo/src/components/viewer/platform_webview.rs:238,299` | same | same |
| L2 | `marco/src/components/viewer/print_driver_linux.rs:62,98` | `webkit6::PrintOperation::new(view)` | Native print dialog + silent PDF export |
| L2 | `polo/src/components/viewer/platform_webview.rs:513` | same | same |
| W1 | `marco/src/components/viewer/print_driver_windows.rs:198-274` | `ICoreWebView2_7::PrintToPdf` (via `webview2-com`) | Silent PDF export |
| W2 | `marco/src/components/viewer/platform_webview.rs:1141-1154` | `ICoreWebView2_16::ShowPrintUI(SYSTEM)` | Native print dialog |
| W2 | `polo/src/components/viewer/platform_webview.rs:589-602` | same | same |
| W3 | `marco/src/components/viewer/platform_webview.rs:25,268-306` | `gdk4-win32` + `raw-window-handle` (no `windows::core`/`webview2-com`) | Extract the Win32 HWND from the GDK4 surface to embed WebView2 as a child window |
| W3 | `polo/src/components/viewer/platform_webview.rs:89-115` | same | same |

Four distinct gaps (L1, L2, W1/W2 combined as "print", W3), not four
independent ones — W1 and W2 are both instances of "wry has no print API," so
they're covered together below.

## 3. Gap 1 (Linux) — `load_html` cannot carry a base URI

**Fork behavior, confirmed against `src/webkitgtk/mod.rs`:**

```rust
pub fn load_html(&self, html: &str) -> Result<()> {
    self.webview.load_html(html, None);  // base URI is always None
    Ok(())
}
```

The fork's portable `load_html` hardcodes `None` for the base URI — same as
upstream `tauri-apps/wry`. There is no `load_html_with_base` or equivalent in
the public API. Marco's escape hatch
(`webkit6::WebViewExt::load_html(&view.webview(), html, Some(base_uri))`)
exists solely to reach the one parameter wry drops.

**Upstream status:** this is a known, long-standing limitation across all of
`wry` (not fork-specific) — see [tauri-apps/wry#820, "Using Local Files in
HTML"](https://github.com/tauri-apps/wry/issues/820). The standard answer
from Tauri maintainers is *"don't use `load_html` for anything with relative
assets — use a custom protocol instead."* Windows already does this in Marco
(`CUSTOM_SCHEME` / `marco-preview://`); Linux does not (Phase 2 of
`WEBVIEW_MIGRATION.md` explicitly deferred this choice: *"pick one, document
it"*).

**What would close this gap:**
- **(a)** wry adds a `load_html`-with-base-URI variant (unlikely upstream —
  it conflicts with the custom-protocol design direction), **or**
- **(b)** Marco moves Linux onto the same custom-protocol scheme Windows
  already uses, eliminating the need for a base URI at all.

(b) is achievable **today**, independent of any fork change — it's a Marco
refactor, not a wry gap. This is the one item on this list that doesn't
actually require upstream/fork work.

**Status: CLOSED (2026-07-16).** Implemented (b), and went further than
"mirror Windows's existing scheme" — Windows's own approach still passed a
base URI internally (via an injected `<base href="file://…">` tag). The
actual fix: the **document's own `marco-preview://` URL** now encodes its
directory, so relative asset references resolve via ordinary URL
relative-resolution to a path the custom-protocol handler reads straight off
disk — no `<base>` tag, no base URI, anywhere, on either platform. Closes the
Linux `webkit6::WebViewExt::load_html` call entirely; `load_html_with_base`
is now one cross-platform function instead of two `#[cfg]`'d ones. Full
design and verification: see the `Eliminate base_uri` plan in this session's
history and the live-tested confirmation below.

- **Linux (marco):** verified live — a real document with relative
  (`img.png`, `./img.png`) and absolute (outside the document's own
  directory) local image references all resolved and served correctly
  (confirmed via explicit byte-count/MIME logging of each request), zero
  errors/warnings in the app log across three separate runs.
- **Linux (polo):** code is a faithful structural port of the same design
  (adapted for polo's eager-build-at-construction Linux path and single
  global HTML store, vs. marco's per-id map). Compiles clean, clippy clean,
  same protocol-logic unit tests pass. **Could not be live-verified** — a
  pre-existing issue (confirmed present on the unmodified baseline, before
  any of this change, via a stashed A/B comparison) prevents polo's async
  render pipeline from ever reaching `load_html_with_base` in this sandbox,
  unrelated to this change.
- **Windows (both apps):** compile-check only — no Windows machine available
  in this environment. The one Windows-specific piece (translating
  `marco-preview://localhost/` → `http://marco-preview.localhost/` for
  `load_url()` reload calls, since wry's URI workaround only runs at build
  time) is carried over unchanged from the proven-working pre-existing
  pattern; only the URL *shape* being translated changed.
- `webkit6` remains a direct dependency on Linux for both apps —
  `print_driver_linux.rs` (Gap 2, still open) and the `WebViewExt::settings()`
  file-access flags (kept as a fallback for explicit `file://` links written
  directly in Markdown, which bypass the document's base and hit WebKit's
  native file loader).

## 4. Gap 2 (both platforms) — no print / PDF export API in `wry`

**Fork behavior, confirmed against `src/webkitgtk/`:** no print-related file
exists in the fork's WebKitGTK backend at all — it inherits upstream's
complete absence of a print API.

**Upstream status:**
- [tauri-apps/wry#707, "Add ability to print webview to pdf
  silently"](https://github.com/tauri-apps/wry/issues/707) — **open**, no
  maintainer commitment to a timeline or design.
- [tauri-apps/wry#1317, "enhance: Add API to allow print
  options"](https://github.com/tauri-apps/wry/pull/1317) — **draft**, stalled
  since July 2024. Covers macOS (reusing #1259) and a partial Linux attempt;
  the author explicitly gave up on Linux PDF generation via WebKitGTK's
  printer-discovery mechanism and pivoted toward an HTTP-server-based print
  dialog instead of automated PDF. **No Windows implementation attempted** —
  commenters only point at `PrintToPdfAsync` as the reference API, same as
  what Marco already does via the escape hatch.
- [tauri-apps/wry#235, "Allow printing"](https://github.com/tauri-apps/wry/issues/235)
  — older duplicate/related issue, also open.

**Conclusion: this is not a fork limitation — it is an upstream `wry` gap the
fork inherited as-is.** Nothing in the fork's own commit history adds print
support beyond what `tauri-apps/wry` has (which is nothing, on Linux or
Windows).

**What would close this gap:** #707 or #1317 (or a fork-local equivalent)
shipping a cross-platform print/PDF API. Given #1317 has been stalled over a
year and explicitly failed on Linux, this is not close. **If full removal of
`webkit6`/`webview2-com` is a hard requirement, print/PDF export is the
long pole** — both `print_driver_linux.rs` and `print_driver_windows.rs`
exist entirely because of this one gap.

## 5. Gap 3 (Windows) — HWND embedding (`gdk4-win32` + `raw-window-handle`)

This is **not** a `webview2-com`/`windows` crate dependency — it doesn't
appear anywhere in the W1/W2 call sites above. It's how Marco (a GTK4 app)
obtains a native window handle to hand to wry's Windows backend, which
expects a `raw-window-handle`-compatible handle, not a GTK widget.

**Why this likely can't be closed by any wry fork change:** wry's Windows
backend is designed for host applications built on `winit`/`tao`
(HWND-native), not GTK4. The fork adds `build_gtk`/`reparent_gtk` for Linux
specifically because WebKitGTK is a native GTK widget — there is no Windows
equivalent because WebView2 is not a native GTK widget on any platform.
Pairing "GTK4 host + WebView2" will always require *some* HWND handoff; the
only open question is whether that handoff is 20 lines of `raw-window-handle`
glue (current state) or something the fork could wrap. **Recommend treating
this as permanent, not tracked as a removable gap.**

## 6. What "full removal" actually requires

| Gap | Fork-specific? | Closable today without upstream/fork work? | Blocking print/base-uri removal |
|---|---|---|---|
| G1: Linux `load_html` base URI | No (upstream-wide) | ~~Yes~~ **CLOSED 2026-07-16** — see §3 | ~~Only the Linux `webkit6::WebViewExt::load_html` call~~ Resolved |
| G2: Print / PDF (Linux) | No (upstream-wide, worse: PR author gave up on Linux) | No | `print_driver_linux.rs`, all of `webkit6::PrintOperation` |
| G2: Print / PDF (Windows) | No (upstream-wide) | No | `print_driver_windows.rs`, `ShowPrintUI` in both `platform_webview.rs` files |
| G3: HWND embedding | N/A — not a webkit6/webview2 dependency | N/A | Not in scope; `gdk4-win32`/`raw-window-handle` would remain regardless |

**Bottom line:** `webkit6` still cannot be fully removed — G1 is closed, but
`print_driver_linux.rs` still needs `webkit6::PrintOperation` (G2).
`webview2-com`/`windows::core` cannot be removed at all today; nothing in
`wry` (fork or upstream) does print/PDF, and #1317's stall suggests this
won't land soon. Full removal is now blocked *entirely* on upstream `wry`
print support (§4) — it is not something achievable by changing Marco's code
or by patching the fork alone, short of the fork's maintainer independently
implementing GTK `PrintOperation` and WebView2 `PrintToPdf`/`ShowPrintUI`
wrapping inside the fork itself (i.e., redoing the work
`print_driver_linux.rs`/`print_driver_windows.rs` already do, just relocated
into the wry crate).

## 7. Investigated and rejected: `reparent()` / `reparent_window()` (2026-07-16)

While auditing `src/webkitgtk/mod.rs`, the fork exposes:

```rust
pub fn reparent<W: IsA<gtk::Widget>>(&self, container: &W) -> Result<()>
pub fn reparent_window<W: HasWindowHandle>(&self, window: &W) -> Result<()>
```

`WEBVIEW_MIGRATION.md` §8 originally flagged the Linux/Windows
detached-window merge as a "candidate for a future merge via wry's `reparent`
APIs — needs Windows testing." Investigated and **rejected** — neither
function is a fit for `marco/src/components/viewer/reparenting.rs` /
`detached_window_linux.rs`.

**`reparent()`** — full body:

```rust
pub fn reparent<W: IsA<gtk::Widget>>(&self, container: &W) -> Result<()> {
    self.webview.unparent();
    if let Some(box_) = container.dynamic_cast_ref::<gtk::Box>() {
        self.webview.set_hexpand(true);
        self.webview.set_vexpand(true);
        box_.append(&self.webview);
    } else if let Some(fixed) = container.dynamic_cast_ref::<gtk::Fixed>() {
        fixed.put(&self.webview, 0.0, 0.0);
    }
    Ok(())
}
```

Two disqualifying problems:
- Only `gtk::Box` and `gtk::Fixed` are handled. Marco's actual containers at
  the reparent boundary are `gtk::Stack`, `gtk::Paned`, and
  `gtk::ScrolledWindow` — none of which match either branch. Calling it with
  any of them unparents the webview and **silently reattaches nothing**
  (still returns `Ok(())`) — worse failure mode than today's code, which
  returns a descriptive `Err`.
- It operates on the raw WebKit widget (`self.webview`), not on
  `PlatformWebView.container` (the stable `gtk4::Box` Marco already wraps
  the webview in via `build_gtk`). Marco's current code already reparents
  that one stable Box between container types — the exact problem
  `reparent()` exists to solve is one Marco's own wrapper already solves,
  just with wider container-type coverage.

**`reparent_window()`** — X11 branch uses raw `XReparentWindow` on the
webview's own X11 window; the Wayland branch re-anchors a native surface.
Both assume the webview owns an independent native surface outside the GTK
widget tree — true for WebKitGTK's *pre*-accelerated-compositing embedding
model, not for the DMA-BUF texture-painting model that WebKitGTK ≥ 2.42 (this
fork's minimum, via its `v2_42` feature) actually uses. Per [Igalia's
accelerated-compositing
writeup](https://blogs.igalia.com/carlosgc/2023/04/03/webkitgtk-accelerated-compositing-rendering/),
the web process exports frames as a DMA-BUF texture that GTK simply paints —
no separate native surface to migrate. GTK4's own documented pattern for
moving a widget between top-level windows is exactly `unparent()` +
`set_child()`/`append()` — i.e., exactly what `attach_webview`/
`detach_webview` already do. `reparent_window()` appears to be inherited
plumbing from wry's older WebKit2GTK/GTK3 support, not something this fork
added or that GTK4 WebKitGTK6 needs.

**Conclusion:** `marco/src/components/viewer/reparenting.rs` is not a
workaround for something the fork does better — it's already the
architecturally correct GTK4 approach for this case, and handles more
container types than the fork's helper does. No action taken.

## 8. Recommendation

G1 is closed (§3). What's left depends on upstream `wry` print support that
has been requested since at least 2023 (#235) and hasn't shipped — not a
"few weeks of Marco-side work" gap. Options, roughly in order of effort:

1. **Do nothing further** — keep the one remaining slim escape-hatch dep
   (`webkit6`, for `PrintOperation` + the `WebViewExt::settings()` file-access
   flags) as-is. This is the current state and is not blocking anything.
2. ~~Close G1~~ **Done** — see §3.
3. **Implement print/PDF inside the fork** — the only path to fully dropping
   both `webkit6` and `webview2-com`. This means porting
   `print_driver_linux.rs`'s `PrintOperation` usage and
   `print_driver_windows.rs`'s `PrintToPdf`/`ShowPrintUI` COM calls *into*
   the fork as a new `WebView::print()`/`print_to_pdf()` API — real,
   nontrivial upstream-style work, on top of what a stalled year-old draft
   PR already found hard on Linux.

Given the cost/benefit, (3) is a
standalone project, not a follow-up task.
