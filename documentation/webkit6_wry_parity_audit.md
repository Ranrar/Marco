# WebKit6 → wry Parity Audit (marco + polo)

**Date:** May 28, 2026
**Scope:** Full audit of every webkit6 API surface used in `marco` and `polo`, and the corresponding wry / WebView2 implementation on Windows.

This document is **analysis only** — no code changes are described as required. Each gap is rated:

| Rating | Meaning |
|---|---|
| ✅ | Full parity (functionally equivalent on both platforms) |
| ⚠️  | Reduced parity — Windows works but with degraded UX or features |
| ❌ | Missing — Windows behaviour is broken or absent |
| ➖ | Linux-only by design — no Windows equivalent required |

---

## 1. Architecture overview

### marco workspace

| File | Platform | Purpose |
|---|---|---|
| [marco/src/components/viewer/webkit6.rs](marco/src/components/viewer/webkit6.rs) | Linux | WebKit6 WebView implementation (10 pub fns) |
| [marco/src/components/viewer/webkit6_detached_window.rs](marco/src/components/viewer/webkit6_detached_window.rs) | Linux | Detached preview window — reparents WebView |
| [marco/src/components/viewer/wry.rs](marco/src/components/viewer/wry.rs) | Windows | wry helper module (8 fns) — mirrors a subset of webkit6 API |
| [marco/src/components/viewer/wry_platform_webview.rs](marco/src/components/viewer/wry_platform_webview.rs) | Windows | `PlatformWebView` wrapper around `wry::WebView` (11 pub methods) |
| [marco/src/components/viewer/wry_detached_window.rs](marco/src/components/viewer/wry_detached_window.rs) | Windows | Detached preview window — creates a fresh `PlatformWebView` |
| [marco/src/components/viewer/wry_print_to_pdf.rs](marco/src/components/viewer/wry_print_to_pdf.rs) | Windows | WebView2 `PrintToPdf` COM wrapper |
| [marco/src/components/viewer/backend.rs](marco/src/components/viewer/backend.rs) | Both | Thin platform dispatch layer (`PreviewWebView` type alias) |
| [marco/src/components/viewer/renderer.rs](marco/src/components/viewer/renderer.rs) | Linux | Async section-based render pipeline (`#[cfg(target_os = "linux")]`) |
| [marco/src/components/viewer/reparenting.rs](marco/src/components/viewer/reparenting.rs) | Linux | WebView reparenting utilities |
| [marco/src/components/viewer/print_driver.rs](marco/src/components/viewer/print_driver.rs) | Linux | `webkit6::PrintOperation` driver (5 pub fns) |
| [marco/src/components/viewer/print_driver_windows.rs](marco/src/components/viewer/print_driver_windows.rs) | Windows | Minimal stub (1 pub fn) — PDF goes through `export_pipeline` |
| [marco/src/components/viewer/export_pipeline.rs](marco/src/components/viewer/export_pipeline.rs) | Both | Unified export pipeline — `LinuxExportBackend` + `WindowsExportBackend` |

### polo workspace

| File | Platform | Purpose |
|---|---|---|
| [polo/src/components/viewer/platform_webview.rs](polo/src/components/viewer/platform_webview.rs) | Both | Single `PlatformWebView` struct with `#[cfg]` Linux/Windows impls (8 pub methods each) |
| [polo/src/components/viewer/rendering.rs](polo/src/components/viewer/rendering.rs) | Both | Markdown → HTML render coordinator |
| [polo/src/components/viewer/loading_overlay.rs](polo/src/components/viewer/loading_overlay.rs) | Both | Loading overlay (parity ✅) |
| [polo/src/components/viewer/empty_state.rs](polo/src/components/viewer/empty_state.rs) | Both | Empty state UI (parity ✅) |

---

## 2. marco — `viewer/webkit6.rs` → `viewer/wry.rs` + `viewer/wry_platform_webview.rs`

### 2.1 Function-level parity

| webkit6 (Linux) | wry / Windows location | Status | Notes |
|---|---|---|---|
| `load_html_when_ready(&WebView, String, Option<String>)` | `PlatformWebView::load_html_with_base(&str, Option<&str>)` (via `backend::load_html_when_ready`) | ⚠️ | Linux polls allocation every 16 ms × 300 retries until `allocated_width > 1`. Windows has only a **single fallback** to `max(100)` when first allocation isn't ready — no retry loop. Side-effect: HTML may load against a wrongly sized child Win32 window on the very first load when GTK has not yet sized the container. |
| `create_html_viewer_with_base(html, base_uri, bg_color)` | `PlatformWebView::new(&ApplicationWindow)` + `set_background_color_rgba` + `load_html_with_base` | ✅ | Same end result; Windows path uses HWND from `gdk4-win32`. |
| `update_html_content_smooth(&WebView, &str)` | `PlatformWebView::update_html_content_smooth(&str)` | ✅ | Both inject JS to call `MarcoCorePreview.updateContent(<json>)`, preserving scroll position. Windows falls back to `load_html_with_base` before the first successful load. Implemented in Step 3. |
| `wrap_html_document(body, css, theme, bg)` | `wry::wrap_html_document(...)` | ✅ | Both delegate to `marco_core::render::wrap_preview_html_document` and apply the RTL fix. |
| `sliders_play_all(&WebView)` | **none** | ❌ | Slider deck auto-play / pause control on theme switch. No-op on Windows means slider decks may keep animating after preview is hidden. Marked `#[allow(dead_code)]` on Linux, so likely not actively wired even there. |
| `sliders_pause_all(&WebView)` | **none** | ❌ | Same as above. |
| `create_html_source_viewer_webview(html, theme, base, bg, fg, thumb_color, track_color)` | `wry::create_html_source_viewer_webview(parent, html, theme, base, bg, fg, thumb, track)` | ✅ | Both return a syntect-highlighted WebView. Shared builder in [`code_view_html.rs`](marco/src/components/viewer/code_view_html.rs) ensures identical HTML output on both platforms. Windows returns a `PlatformWebView` (Step 5b). |
| `update_code_view_smooth(widget, html, theme, bg, fg, thumb, track)` | `wry::update_code_view_smooth(pv, html, theme, bg, fg, thumb, track)` | ✅ | Both dispatch the `build_smooth_update_js` payload from [`code_view_html.rs`](marco/src/components/viewer/code_view_html.rs) through `evaluate_script`. Output is identical (Step 5b). |
| `setup_link_hover_status(&WebView, on_hover)` | `PlatformWebView::set_hover_link_callback(F)` | ✅ | Linux uses native `connect_mouse_target_changed`. Windows uses `marco_hover:<url>` IPC message posted from injected JS (`HOVER_REPORT_JS`). Functionally equivalent. |
| `setup_local_file_link_handler(&WebView, on_local_md)` | `PlatformWebView::set_local_md_link_handler(F)` (built into navigation handler) | ✅ | Both intercept `.md` / `.markdown` `file://` navigations. |
| `setup_link_handling(&WebView)` (private — opens external in browser) | Built into `PlatformWebView` navigation handler + `wry::open_external_uri` | ✅ | Both use `gio::AppInfo::launch_default_for_uri`. |

### 2.2 Windows-only functions in `wry.rs` (no Linux counterpart needed)

| Function | Purpose | Required on Linux? |
|---|---|---|
| `set_latest_live_html` / `get_latest_live_html` | Cache of "clean" preview HTML for **Save as HTML** export. | No — Linux re-renders directly from the live `WebView`. |
| `set_latest_preview_base_uri` / `get_latest_preview_base_uri` | Cache of base URI so detached preview window can resolve relative paths after re-creation. | No — Linux detached window keeps the same `WebView` via reparenting. |
| `generate_test_html(wheel_js)` | Welcome HTML for empty buffers. | No — Linux uses `MarcoCorePreview` directly. |
| `generate_base_uri_from_path` | `file:///` URI with Windows-specific forward-slash + trailing-slash handling. | No — Linux uses `backend::generate_base_uri_from_path` (different impl). |

### 2.3 Windows-only methods on `PlatformWebView`

| Method | Purpose | Linux equivalent |
|---|---|---|
| `set_scroll_report_callback(F)` | Subscribe to `marco_scroll:N` IPC for scroll-sync. | Linux uses native GTK adjustment signals on the WebView. |
| `set_export_event_listener(F)` / `clear_export_event_listener()` | Subscribe to `marco_export:<phase>` IPC for export lifecycle. | Linux uses `notify::title` signal on the WebView (see `export_pipeline.rs::LinuxExportBackend`). |
| `trigger_print_dialog()` | Calls WebView2 `ShowPrintUI(SYSTEM)` (with fallback to `view.print()`). | `print_driver::trigger_print_dialog` uses `webkit6::PrintOperation::run_dialog`. |
| `print_to_pdf(path, paper, orientation, margin)` | WebView2 `ICoreWebView2_7::PrintToPdf` via COM. | `print_driver::export_to_pdf` uses `webkit6::PrintOperation::print` to file. |

### 2.4 Linux-only helpers in `webkit6.rs` (no Windows equivalent)

| Helper | Status on Windows |
|---|---|
| `run_once_when_mapped(webview, F)` | ❌ Missing — no "wait until mapped" mechanism for hidden Win32 child windows. May contribute to stack-switching glitches. |
| `setup_user_content_manager(&WebView)` | ➖ Linux memory-leak guard for accumulated JS/CSS injections; not relevant on WebView2 which uses a separate process. |
| `connect_destroy` cleanup with `MarcoCorePreview.cleanup()` | ❌ Missing — Windows has no equivalent JS-cleanup hook on WebView destruction. Note: `WEBVIEW_HTML_MAP` entries are now auto-evicted when a `PlatformWebView` is dropped via the `IdGuard` RAII guard (Step 9b); the JS-cleanup hook itself remains absent. |
| `parse_hex_to_rgba(hex)` helper | ➖ Windows uses RGBA tuples directly from GTK. |
| Debug write to `/tmp/marco_code_view_debug.html` | ➖ Linux-only path (would fail on Windows, but only called from Linux module). |

---

## 3. marco — Editor / preview integration in `editor/ui.rs`

This file diverges between Linux and Windows after roughly line 1050, and the differences are **significant**.

### 3.1 Initial WebView creation

| Step | Linux | Windows |
|---|---|---|
| Wrap HTML | `webkit6::wrap_html_document` | `backend::wrap_html_document` (uses `wry::wrap_html_document` internally) |
| Create WebView | `webkit6::create_html_viewer_with_base` | `wry_platform_webview::PlatformWebView::new` |
| Link-hover wiring | `webkit6::setup_link_hover_status` | `PlatformWebView::set_hover_link_callback` |
| Code viewer widget | `webkit6::create_html_source_viewer_webview` (full syntect WebView) | `wry::create_html_source_viewer_webview` (full syntect `PlatformWebView`, Step 5b) |
| Loading-overlay hide | `connect_load_changed → LoadEvent::Finished` (native signal) | `marco_zoom:ready` IPC posted by `WIN_ZOOM_BAR_HTML` (DOMContentLoaded) |

### 3.2 Preview refresh closure (`refresh_preview_impl`)

| Feature | Linux closure | Windows closure |
|---|---|---|
| Section-based incremental render | ✅ via `renderer::refresh_preview_content_sections` | ❌ Always full-document reload |
| Smooth in-place update | ✅ via `update_html_content_smooth` (for non-paged small docs) | ✅ `PlatformWebView::update_html_content_smooth` for routine edits; force-full-reload on first load, CSS/theme change, doc-path change, or paged.js toggle (Step 4b) |
| AST cache warming | ✅ background thread `parse_and_cache_ast` after first load | ❌ Not performed |
| Generation counter (`preview_generation`) | ✅ wraps every render request, discards stale results | ✅ `preview_generation_win` — incremented on each render entry, hash guard reset if stale |
| In-flight guard (`preview_in_flight`) | ✅ prevents concurrent renders | ✅ `preview_in_flight_win` — guards the synchronous Windows render path |
| Content-hash skip | ✅ avoids re-render when text unchanged | ✅ `last_preview_hash` cell deduplicates no-op refreshes (Step 4b) |
| Page-view (paged.js) full-reload trigger | ✅ explicit transition detection (`page_view_changed`) | ✅ explicit `last_page_view_enabled` + `page_view_changed` detection (Step 4b) |
| Background CSS hash tracking | ✅ `last_css_hash` short-circuits unchanged CSS | ✅ `last_css_hash_win` cell (Step 4b) |
| Document-path change detection | ✅ `last_document_path` differentiates new-file open from edit | ✅ `last_document_path_win` cell (Step 4b) |

**Net effect (updated — Steps 3 + 4b + generation/in-flight, this session):** Windows now uses `update_html_content_smooth` for routine edits, with content-hash deduplication, CSS-hash tracking, document-path detection, explicit page-view transition guards, generation counter, and in-flight guard. Force-full-reloads still occur for first load, CSS/theme changes, document-path changes, and paged.js mode transitions — these are by design. The visible per-keystroke flash is eliminated for all routine edits. Section-incremental DOM patching (Linux's `prev_section_hashes` / `refresh_preview_content_sections` path) remains Linux-only and is a future performance optimisation.

### 3.3 Loading-overlay flow

| Event | Linux | Windows |
|---|---|---|
| `show()` | Triggered when opening a file (`file_operations.rs`) | Same |
| `hide()` | `WebView::connect_load_changed → Finished` (single wired handler) | `marco_zoom:ready` IPC (fires on every `DOMContentLoaded` via `WIN_ZOOM_BAR_HTML`) |

This was the *"new GTK loader is not working in windows"* issue from the previous session — already fixed.

---

## 4. marco — Detached preview window

| Feature | Linux (`webkit6_detached_window.rs`) | Windows (`wry_detached_window.rs`) |
|---|---|---|
| `new()` signature | `(parent, app)` | `(parent)` — no `app` parameter |
| `load_preview_content()` (renamed from `attach_webview`) | `attach_webview(&WebView)` reparents the existing live WebView | ✅ Renamed to `load_preview_content()` (no parameter — argument was always ignored). Creates a brand-new `PlatformWebView` and loads HTML from the `LATEST_PREVIEW_HTML` global cache. Reparenting remains impossible (WebView2 hard limit §14.3). |
| `detach_webview()` | ✅ Returns `Option<WebView>` to caller for reparenting back | ❌ **Method missing entirely** |
| `has_webview()` | ✅ | ❌ **Method missing** |
| Scroll position preserved on detach/reattach | ✅ (same WebView instance) | ✅ `preview_state.rs` snapshot captures `scrollY`/`scrollX`; `restore_script` replays on `marco_zoom:ready` in the new WebView (Step 7b) |
| DOM state / JS state preserved | ✅ | ✅ `open_details` list and `body_hash` captured and restored. Arbitrary JS/form state is not replayable — inherent to re-creation (hard WebView2 limit). |
| Runtime cost per detach | Constant — pointer move | High — full WebView2 process spin-up + HTML re-parse + JS re-execution |
| Memory cost | Single WebView | Two parallel WebView2 instances during the detached session |
| Custom titlebar with light/dark icons | ✅ Full implementation | ⚠️ Minimal implementation |

**Architectural reason:** wry / WebView2 child windows are tied to a single HWND parent and cannot be reparented to a different top-level window. The Windows code works around this by re-creating the WebView in the new window. This is the fundamental WebView2 limitation, not a bug — but the API surface differs and several consumer call sites must special-case Windows.

---

## 5. marco — Print and PDF export

| Function | Linux (`print_driver.rs`) | Windows |
|---|---|---|
| `trigger_print_dialog(webview, parent)` | ✅ `webkit6::PrintOperation::run_dialog` | ✅ `print_driver_windows::trigger_print_dialog` → `PlatformWebView::trigger_print_dialog` → `ShowPrintUI(SYSTEM)` |
| `make_print_export_css(paper, orientation, dark)` | ✅ Used directly by `webkit6::PrintOperation` | ✅ Re-used by Windows path (shared in `marco_shared::logic::print_css`) |
| `export_to_pdf(...)` async | ✅ `webkit6::PrintOperation::print` with PDF format | ✅ Goes through `export_pipeline::WindowsExportBackend` → `wry_print_to_pdf::print_to_pdf` (WebView2 `PrintToPdf`) |
| `inject_export_css(webview, css, paper, orientation)` | ✅ | ⚠️ Windows version inlines this inside the export backend, not as a public API |
| `remove_export_css(webview)` | ✅ | ⚠️ Windows uses self-removing `setTimeout` from JS instead of an explicit removal call |
| Native paper-size dialog | ✅ User picks in GTK dialog | ✅ User picks in WebView2 system print dialog |
| PDF metadata (title, author) | ⚠️ Limited by `webkit6::PrintOperation` API | ⚠️ Limited by WebView2 `PrintToPdf` API — both roughly equivalent |

**Status:** parity is good — both platforms reach feature equivalence through different backends, both unified by `export_pipeline.rs` (`LinuxExportBackend` vs `WindowsExportBackend` implementing the `PlatformExportBackend` trait).

---

## 6. marco — Scroll synchronisation

[marco/src/components/editor/scroll_sync.rs](marco/src/components/editor/scroll_sync.rs)

| API | Linux signature | Windows signature | Status |
|---|---|---|---|
| `connect_scrolled_window_to_webview` | `(&ScrolledWindow, &webkit6::WebView)` | — | Linux only |
| `connect_scrolled_window_to_platform_webview` | — | `(&ScrolledWindow, &PlatformWebView)` | Windows only |
| `connect_scrolled_window_and_webview` | `(&ScrolledWindow, &webkit6::WebView)` | — | Linux only |
| `connect_scrolled_window_and_platform_webview` | — | `(&ScrolledWindow, &PlatformWebView)` | Windows only |
| `setup_webview_title_listener` | `(&webkit6::WebView, F)` | — | Used for export lifecycle on Linux; Windows uses IPC instead |

**Status:** ✅ functional parity — wrapped in separate functions for each backend; call sites are `#[cfg]`-gated in `editor/ui.rs`.

---

## 7. marco — Search (find-in-preview)

| File | Platform | Status |
|---|---|---|
| [marco/src/components/search/window.rs](marco/src/components/search/window.rs) | `#[cfg(target_os = "linux")]` only | ❌ Linux-only UI creation |
| [marco/src/components/search/navigation.rs](marco/src/components/search/navigation.rs) | `sync_html_preview_scroll` on both platforms; Windows `wry_find::next/prev` wired in debounce timer | ✅ |
| [marco/src/components/search/state.rs](marco/src/components/search/state.rs) | `CURRENT_PLATFORM_WEBVIEW` on Windows; `wry_find::clear` called from `clear_search_highlighting` | ✅ |
| [marco/src/ui/dialogs/search.rs](marco/src/ui/dialogs/search.rs) | Windows `show_search_window_no_webview` stores `PlatformWebView` in thread-local | ✅ |
| [marco/src/components/search/engine.rs](marco/src/components/search/engine.rs) | Windows: `wry_find::install + search` called at end of `perform_search` | ✅ |

**Root cause:** WebKit6 exposes `WebView::find_controller()` returning a native `FindController` with built-in highlight, count, and next/previous. **WebView2 / wry has no equivalent** — text search in the live HTML preview is implemented in JavaScript (`wry_find.rs`) and tied via IPC.

**Net effect on Windows:**
- ✅ Find-in-preview: all matches highlighted in WebView; active match advances with editor navigation (next/prev buttons and Enter key); cleared when search box is emptied
- ✅ Find-in-editor (SourceView5) works as before
- ❌ Remaining gap: `window.rs::get_or_create_search_window` is still Linux-only. The Windows search window (`create_windows_search_window`) shows the same UI widgets but find-in-preview callbacks (`wry_find::search`) are wired through `engine::perform_search`, not the window factory. A future improvement: deduplicate `window.rs` behind a cross-platform factory.

---

## 8. marco — TOC panel

[marco/src/ui/toc_panel.rs](marco/src/ui/toc_panel.rs)

| Action | Linux (line 209) | Windows (line 222) |
|---|---|---|
| Scroll-to-heading on TOC click | Uses `webkit6::WebViewExt::evaluate_javascript` with native callback | Uses `PlatformWebView::evaluate_script` (fire-and-forget) |

**Status:** ✅ functional parity — both branches scroll to the requested heading by ID.

---

## 9. marco — Dialog WebViews (math, mermaid)

### 9.1 Math dialog ([marco/src/ui/dialogs/math.rs](marco/src/ui/dialogs/math.rs))

| Branch | Behaviour |
|---|---|
| Linux | Creates `webkit6::WebView` directly with theme background. Always works. |
| Windows | `PlatformWebView::new` now accepts `&impl IsA<gtk4::Window>` so any window type is accepted without a downcast. The `Label` fallback is removed (Step 2). |

**Status:** ✅ Full parity. `PlatformWebView::new` accepts `&impl IsA<gtk4::Window>` since Step 2.

### 9.2 Mermaid dialog ([marco/src/ui/dialogs/mermaid.rs](marco/src/ui/dialogs/mermaid.rs))

Same pattern as math.rs — resolved together with Step 2.

**Status:** ✅ Full parity. `PlatformWebView::new` accepts `&impl IsA<gtk4::Window>` since Step 2.

---

## 10. polo — `viewer/platform_webview.rs`

Polo's `PlatformWebView` is a **single struct** with `#[cfg]`-gated Linux/Windows impls. Method names match perfectly, which is the cleanest cross-platform architecture in the workspace.

| Method | Linux | Windows | Status |
|---|---|---|---|
| `new(window)` | ✅ creates `webkit6::WebView`, configures security | ✅ obtains HWND, builds container `gtk4::Box`, schedules tick callback for child-window bounds | ✅ |
| `widget()` | Returns `webkit6::WebView.upcast()` | Returns `gtk4::Box.upcast()` (container — wry child painted on top) | ✅ |
| `set_background_color_rgba(&RGBA)` | `WebView::set_background_color` | Stores RGBA + calls `wry::WebView::set_background_color` | ✅ |
| `load_html_with_base(html, base_uri)` | `idle_add_local_once → load_html` | Injects `<base href>` into `<head>`, then `wry::WebView::load_html` (or lazy-builds the WebView with current allocation rect) | ✅ |
| `connect_load_finished(F)` | `connect_load_changed → LoadEvent::Finished` | Stored handler invoked from `on_page_load_handler` when `wry::PageLoadEvent::Finished` | ✅ |
| `evaluate_script(&str)` | `WebView::evaluate_javascript` (`#[allow(dead_code)]`) | `wry::WebView::evaluate_script` (`#[allow(dead_code)]`) | ✅ |
| `setup_link_policy(on_local_md)` | `connect_decide_policy` with full URI inspection | Stores handler, applied to `WebViewBuilder::with_navigation_handler` on first build | ✅ |
| `print(parent)` | `webkit6::PrintOperation::new` + `run_dialog` with injected `@media print` CSS | `wry::WebView::print()` with injected CSS + 60s self-removal timer; falls back to `window.print()` if wry print() fails | ✅ |

### 10.1 Polo-specific findings

| Item | Status |
|---|---|
| No `update_html_content_smooth` equivalent on either platform | Polo always reloads HTML on render — by design (it's a viewer, not an editor with live edits). |
| External link handling | ✅ Linux uses `gio::AppInfo`; Windows uses `cmd /c start "" <url>` |
| Local `.md` link interception | ✅ Identical semantics on both platforms |
| `print()` UX | ⚠️ Linux opens native GTK print dialog. Windows opens WebView2 browser-style print preview *inside the preview area* (visible flicker as content reflows). Mitigated by `view.print()` fallback to system print UI on newer WebView2. |
| `WEBVIEW2_USER_DATA_FOLDER` set to `user_data_dir().join("webview")` | ✅ Portable-mode friendly |
| Loading overlay | ✅ Wired to `connect_load_finished` on both platforms — works correctly |

**Status:** Polo has the cleanest parity of the entire workspace.

---

## 11. Summary — Severity ranking of gaps

### Open — still requires work

4. **⚠️ `renderer.rs` section-incremental DOM patching remains Linux-only.** Content-hash skip, CSS-hash tracking, doc-path detection, smooth updates, generation counter, and in-flight guard are all now in place on Windows. The remaining Linux-only feature is section-incremental DOM patching (`prev_section_hashes` / `refresh_preview_content_sections` path) — a future performance optimisation.

### Resolved (previously Critical)

1. **✅ Find-in-preview (`Ctrl+F`) fully wired on Windows.** `wry_find::install` + `wry_find::search` called from `engine::perform_search`; `wry_find::next`/`prev` called from the debounced navigation timer in `navigation.rs`; `wry_find::clear` called from `state::clear_search_highlighting`. All highlights and active-match advancement are synchronised between the editor buffer and the WebView preview.
2. **✅ `update_html_content_smooth` implemented on Windows (Steps 3 + 4b).** Routine edits use smooth JS injection; only structural reloads (first load, CSS change, doc-path change, paged.js toggle) do a full reload.
3. **✅ Detached preview scroll/state preserved on Windows (Steps 7a + 7b).** `preview_state.rs` snapshot/restore covers `scrollY`, `scrollX`, and open `<details>`. True WebView reparenting remains impossible (hard WebView2 limit).

### Resolved (previously Important)

5. **✅ Code-view widget on Windows now uses a full syntect `PlatformWebView` (Step 5b).** Output is bit-identical to the webkit6 / Linux code view.
6. **✅ Math and Mermaid dialogs work in all window contexts (Step 2).** `PlatformWebView::new` accepts `&impl IsA<gtk4::Window>`; `Label` fallback removed.
8. **✅ `load_html_when_ready` retry logic added for Windows (Step 1).** `allocation_wait.rs` provides the same 16 ms × 300 retry poll as Linux.

### Resolved (previously Minor)

7. **✅ Print fallback on Windows cleaned up.** Dead `evaluate_script("window.print()")` when the WebView is not yet initialized replaced with a `log::warn!`. Primary path (`ShowPrintUI(SYSTEM)`) opens the Windows system dialog; `view.print()` fallback is the Chromium in-page dialog (unavoidable when COM fails).
9. **✅ `sliders_play_all` / `sliders_pause_all` implemented on Windows (Step 9a)** — both remain `#[allow(dead_code)]` on both platforms pending a caller.
9c. **✅ `attach_webview` API renamed to `load_preview_content()`.** Misleading `Option<&gtk4::Widget>` parameter removed. All three call sites updated (`wry_detached_window.rs`, `menu.rs`, `viewer/mod.rs`).
10. **➡️ Windows-only `LATEST_PREVIEW_HTML` / `LATEST_LIVE_HTML` globals** — exist because detached preview can't reparent; no Linux counterpart needed. No change planned.
11. **✅ `WEBVIEW_HTML_MAP` eviction via `IdGuard` RAII drop (Step 9b).** Entries removed automatically when the owning `PlatformWebView` is dropped.
12. **➡️ Windows `print_driver_windows.rs` is intentionally minimal** — PDF flows through `export_pipeline::WindowsExportBackend`. No change planned.

---

## 12. Cross-cutting observations

- **Polo's approach is the gold standard:** a single struct with `#[cfg]`-gated impls and matching method names. Marco's separation into `webkit6.rs` (free functions) + `wry.rs` (free functions) + `wry_platform_webview.rs` (struct with methods) causes most of the API-shape mismatch. The Windows-side method names are mostly different from the Linux-side function names (e.g. `setup_link_hover_status` vs `set_hover_link_callback`).
- **No unified WebView trait.** A `trait PreviewBackend` covering `load_html_with_base`, `evaluate_script`, `set_hover_callback`, `set_local_md_link_handler`, `set_load_finished_callback`, `set_background_color_rgba`, `widget` would let `renderer.rs`, `editor/ui.rs`, and the detached-window code share a single code path. Currently they fork with `#[cfg(target_os)]` blocks at every call site.
- **IPC strings (`marco_scroll:`, `marco_hover:`, `marco_zoom:`, `marco_export:`) are duplicated** between the JS generator (`viewer/javascript.rs`) and the IPC parser (`wry_platform_webview.rs`). A shared constants module would reduce drift risk.
- **`renderer.rs` section-incremental DOM patching remains Linux-only** (`prev_section_hashes` / `refresh_preview_content_sections`). The Windows branch now has content-hash deduplication, smooth-update routing, generation counter, and in-flight guard. Full parity for incremental patching requires either porting the async section pipeline or extracting a `PreviewBackend` trait (§14.4).
- **`wry_detached_window.rs::load_preview_content()`** (renamed from `attach_webview`) always creates a fresh `PlatformWebView` — true WebView reparenting remains impossible (WebView2 hard limit §14.3). State capture/restore via `preview_state.rs` (Step 7b) mitigates the user-visible impact.

---

## 13. Appendix — Files inspected

### marco

- [marco/src/components/viewer/backend.rs](marco/src/components/viewer/backend.rs)
- [marco/src/components/viewer/webkit6.rs](marco/src/components/viewer/webkit6.rs)
- [marco/src/components/viewer/webkit6_detached_window.rs](marco/src/components/viewer/webkit6_detached_window.rs)
- [marco/src/components/viewer/wry.rs](marco/src/components/viewer/wry.rs)
- [marco/src/components/viewer/wry_platform_webview.rs](marco/src/components/viewer/wry_platform_webview.rs)
- [marco/src/components/viewer/wry_detached_window.rs](marco/src/components/viewer/wry_detached_window.rs)
- [marco/src/components/viewer/wry_print_to_pdf.rs](marco/src/components/viewer/wry_print_to_pdf.rs)
- [marco/src/components/viewer/renderer.rs](marco/src/components/viewer/renderer.rs)
- [marco/src/components/viewer/reparenting.rs](marco/src/components/viewer/reparenting.rs)
- [marco/src/components/viewer/print_driver.rs](marco/src/components/viewer/print_driver.rs)
- [marco/src/components/viewer/print_driver_windows.rs](marco/src/components/viewer/print_driver_windows.rs)
- [marco/src/components/viewer/export_pipeline.rs](marco/src/components/viewer/export_pipeline.rs)
- [marco/src/components/viewer/javascript.rs](marco/src/components/viewer/javascript.rs)
- [marco/src/components/editor/ui.rs](marco/src/components/editor/ui.rs)
- [marco/src/components/editor/scroll_sync.rs](marco/src/components/editor/scroll_sync.rs)
- [marco/src/components/editor/editor_manager.rs](marco/src/components/editor/editor_manager.rs)
- [marco/src/components/search/window.rs](marco/src/components/search/window.rs)
- [marco/src/components/search/navigation.rs](marco/src/components/search/navigation.rs)
- [marco/src/components/search/state.rs](marco/src/components/search/state.rs)
- [marco/src/ui/dialogs/search.rs](marco/src/ui/dialogs/search.rs)
- [marco/src/ui/dialogs/math.rs](marco/src/ui/dialogs/math.rs)
- [marco/src/ui/dialogs/mermaid.rs](marco/src/ui/dialogs/mermaid.rs)
- [marco/src/ui/toc_panel.rs](marco/src/ui/toc_panel.rs)
- [marco/src/ui/menu_items/file_operations.rs](marco/src/ui/menu_items/file_operations.rs)

### polo

- [polo/src/components/viewer/platform_webview.rs](polo/src/components/viewer/platform_webview.rs)
- [polo/src/components/viewer/rendering.rs](polo/src/components/viewer/rendering.rs)
- [polo/src/components/viewer/loading_overlay.rs](polo/src/components/viewer/loading_overlay.rs)
- [polo/src/components/viewer/empty_state.rs](polo/src/components/viewer/empty_state.rs)
- [polo/src/main.rs](polo/src/main.rs)

---

## 14. Solutions research — closing each gap without degradation

This section is the result of researching the wry 0.55 API, the WebView2 SDK reference, the Chromium JS engine surface, and the existing marco/polo code patterns. For each gap from §11, the goal is **bit-for-bit feature parity on Windows with no visible regression vs. the Linux WebKit6 path**.

The general strategy uses three building blocks already present in the workspace:
- **JS injection** via `wry::WebView::evaluate_script` / `evaluate_script_with_callback` — exact same approach Linux uses for `update_html_content_smooth`.
- **IPC** via `window.ipc.postMessage("marco_*:...")` — already wired in `wry_platform_webview.rs`.
- **Direct WebView2 COM** via `windows-rs` crate, bypassing wry — already proven by `wry_print_to_pdf.rs` which casts `wry::WebView` to `ICoreWebView2_7` for `PrintToPdf`.

The key research insight is that **none of the four Critical gaps require touching wry upstream or shipping additional binaries.** They are all addressable with code we already have permission to write.

### 14.1 Find-in-preview on Windows (Gap #1) — ✅ Resolved

**Implementation.** JS-based find engine (`wry_find.rs`) with CSS Custom Highlight API (tier B) + `window.find()` fallback (tier A). IPC arm `marco_find:count=N,index=K` in `wry_platform_webview.rs`. Wired into the search pipeline:

- `engine::perform_search` calls `wry_find::install(pv)` then `wry_find::search(pv, query, opts)` at the end of each search — highlights all matches in the WebView, sets active index 0.
- Debounced navigation timer in `navigation::immediate_position_update_with_debounced_navigation` calls `wry_find::next(pv)` / `wry_find::prev(pv)` after `navigate_to_current_position()` — keeps editor and preview active-match indices in sync.
- `state::clear_search_highlighting` calls `wry_find::clear(pv)` — removes all preview highlights when the search box is emptied or the window is closed.

**Remaining gap (minor / cosmetic):** `window.rs::get_or_create_search_window` is still Linux-only. The Windows search window factory (`create_windows_search_window` in `ui/dialogs/search.rs`) builds the same UI shell, and preview find callbacks reach `wry_find` through `engine::perform_search`. A future improvement would deduplicate the window factory behind a cross-platform function.

### 14.2 `update_html_content_smooth` on Windows (Gap #2, Critical)

**Why it's broken today.** Linux's `webkit6::update_html_content_smooth` injects JS that calls `MarcoCorePreview.updateContent(newBodyHtml)` to swap only the changed `<body>` contents while preserving `scrollY`. Windows currently does a full `load_html_with_base` on every refresh, causing a flash + scroll reset.

**Strategy: identical JS, executed via `wry::WebView::evaluate_script`.** The JS already exists in the page (`MarcoCorePreview.updateContent` is part of the bundled preview JS). The Linux Rust side does literally `webview.evaluate_javascript(&format!("MarcoCorePreview.updateContent({});", json_body), ...)`. wry has the equivalent `evaluate_script(&js)` which is documented to work identically on WebView2.

**Implementation plan:**
1. Add `PlatformWebView::update_html_content_smooth(&self, body_html: &str)` to `wry_platform_webview.rs`. Body is the **inner HTML only** (no wrapping document).
2. The method should JSON-escape `body_html` (use `serde_json::to_string`) and dispatch `MarcoCorePreview.updateContent(<json>)` through `evaluate_script`. If the runtime call fails (e.g. very first load before JS bundle is ready), fall back to `load_html_with_base`.
3. In `editor/ui.rs`, the Windows refresh closure can then mirror the Linux branch: if `MarcoCorePreview` is initialised (track via the existing `marco_zoom:ready` signal), call `update_html_content_smooth`; else `load_html_with_base`.

**No degradation:** WebView2's `evaluate_script` runs synchronously in the main frame, just like WebKit6's `evaluate_javascript`. The DOM diff happens entirely in JS — no platform-specific behaviour is involved.

### 14.3 Detached preview scroll/state loss (Gap #3, Critical)

**Why it's broken today.** WebView2 child WebViews are bound to a single HWND parent and cannot be reparented across top-level windows. This is a **hard WebView2 limitation** and cannot be removed at the wry layer.

**Strategy: don't reparent — preserve and restore state.** Three approaches, ranked by fidelity:

| Approach | Fidelity | Implementation complexity |
|---|---|---|
| **(a) State capture + restore via JS** (recommended) | Visually indistinguishable from reparenting for the common case (scroll + form state + collapsed details). Theme/zoom carry via `LATEST_PREVIEW_*` globals. | Low — 30 lines of JS, one IPC round-trip. |
| **(b) Two WebView2 instances + Wallpaper bridge** | Pixel-perfect, but doubles RAM. | High — two instances kept in sync via `update_html_content_smooth` on every edit. |
| **(c) Single owned `tao::Window` reused** | Avoids creating a new HWND each detach: the WebView lives inside a hidden `tao` child window that gets reparented between marco's main HWND and the detached window's HWND. | Medium — requires native HWND `SetParent` Win32 call. Has known WebView2 issues with hit-testing after parent change; not officially supported. |

**Recommended path: (a).** Before destroying the in-main-window WebView, send a `marco_state_snapshot:` IPC request, wait for the JSON reply (`{scrollY, scrollX, selection, openDetails, hash}`), stash it in a `LATEST_PREVIEW_STATE` global, then after the new detached WebView fires `marco_zoom:ready`, dispatch `MarcoCorePreview.restoreState(<json>)` via `evaluate_script`. **End-user UX: scroll position preserved, no visible state loss.** This is not architecturally identical to Linux but is **functionally equivalent** for everything users observe.

**Bonus:** apply the same `restoreState` flow on every full reload in `refresh_preview_impl` to eliminate the keystroke-flash issue from Gap #2's edge case when `update_html_content_smooth` falls back to `load_html_with_base`.

### 14.4 Port `renderer.rs` section pipeline to Windows (Gap #4) — ✅ Partially Resolved

**What was done.** Generation counter (`preview_generation_win`) and in-flight guard (`preview_in_flight_win`) added to the Windows refresh closure in `editor/ui.rs`. Both mirror the Linux `preview_generation` / `preview_in_flight` cells:
- `preview_in_flight` is set before the render and cleared after (including the empty-document early-return path).
- `preview_generation` is incremented on each render entry; on completion, if the generation advanced (stale result), the content-hash guard is reset so the next debounce fires a fresh pass.

**Remaining gap (minor / performance only):** Section-incremental DOM patching (`prev_section_hashes` / `refresh_preview_content_sections`) remains Linux-only. The Windows branch renders the full document on each non-trivial edit (but with content-hash dedup, smooth-update routing, and the generation/in-flight guards). A future `PreviewBackend` trait (see original plan below) would allow sharing `renderer.rs` across platforms.

**Original strategy: introduce a `PreviewBackend` trait and make `renderer.rs` cross-platform.**

```text
trait PreviewBackend {                       // implemented for both platforms
    fn load_html_with_base(&self, html: &str, base_uri: Option<&str>);
    fn update_html_content_smooth(&self, body_html: &str);
    fn evaluate_script(&self, js: &str);
    fn widget(&self) -> &gtk4::Widget;
}
```
- Content-hash skip (no-op on unchanged text).

**No degradation:** the trait is a thin pass-through with zero runtime cost (monomorphised). All behavioural logic remains in the shared `renderer.rs`. **This is the single highest-ROI change** — it closes Gap #4 entirely and also delivers parity for Gap #2's "smart refresh" decision tree for free.

### 14.5 Code-view syntect highlighting on Windows (Gap #5, Important)

**Why it's degraded today.** [wry.rs](marco/src/components/viewer/wry.rs)'s `create_html_source_viewer_webview` and `update_code_view_smooth` return a plain `gtk4::TextView` with no syntax highlighting. The Linux version returns a `webkit6::WebView` rendering a syntect-highlighted HTML document.

**Strategy: render syntect-highlighted HTML inside a wry WebView, same as Linux.** Syntect is pure Rust, already linked, already produces HTML on Linux. Nothing about syntect is platform-specific.

**Implementation plan:**
1. Have `wry::create_html_source_viewer_webview` build a `PlatformWebView` (HWND child of the parent passed in) instead of `TextView`.
2. Build the syntect-highlighted HTML using the **exact same helper** Linux uses — extract it from `webkit6.rs` into a shared module (e.g. `viewer/code_view_html.rs`) so both platforms call the same function.
3. `update_code_view_smooth` then becomes `update_html_content_smooth` against the code-view's WebView (reusing Gap #2's plumbing).

**Caveat:** code-view is usually inside a paned widget — the same allocation-rect logic that `wry_platform_webview.rs::new` uses for the main preview WebView applies here. The existing helper already handles this.

**No degradation:** identical HTML, identical syntect theme, identical scrollbar styling. The only difference vs. Linux is the WebView backend (WebView2 vs. WebKit6) which is invisible to the user.

### 14.6 Math / Mermaid dialog parent requirement (Gap #6, Important)

**Why it's degraded today.** `PlatformWebView::new` takes `&ApplicationWindow` because `gdk4-win32::Win32Surface` is obtained from the window's `Native`. Dialogs sometimes have a parent of `gtk4::Window` (not `ApplicationWindow`), so the downcast fails and the dialog falls back to a plain `Label`.

**Strategy: accept any `IsA<gtk4::Window>`.** Inspect [marco/src/components/viewer/wry_platform_webview.rs](marco/src/components/viewer/wry_platform_webview.rs)'s `new`:
- `gtk4::ApplicationWindow` and `gtk4::Window` both implement `gtk4::Native`, which is the trait that gives you a `gdk::Surface`.
- `gdk4-win32::Win32Surface::downcast` works on **any** `gdk::Surface` from a Win32 native window, regardless of whether it came from an `ApplicationWindow`.

**Implementation plan:**
1. Change `PlatformWebView::new` signature from `&ApplicationWindow` to `impl IsA<gtk4::Window>` (or `&dyn IsA<gtk4::Native>` to be even more general).
2. Update the call sites in `math.rs`, `mermaid.rs`, and the dialog modules — they already have a `gtk4::Window` parent, so they just stop downcasting.
3. Remove the `Label` fallback branches.

**No degradation:** the HWND comes from the same `Win32Surface::handle()` call. The only thing that changes is the static type accepted at the API boundary.

### 14.7 Print preview UX on Windows (Gap #7, Important)

**Why it's degraded today.** Inside the editor preview, hitting Print on Windows can render WebView2's in-page print preview (a browser-style overlay) because `wry::WebView::print()` may call `view.print()` JS which Chromium overrides to its own UI. Linux shows a true modal GTK print dialog.

**Strategy: always call `ICoreWebView2_7::ShowPrintUI(SYSTEM)` via direct COM.** This is already done in `PlatformWebView::trigger_print_dialog` for the menu path — the gap is when other paths (legacy `view.print()` calls in JS) bypass it.

**Implementation plan:**
1. Audit `viewer/javascript.rs` (especially `WIN_ZOOM_BAR_HTML` and any keybinding handlers) for raw `window.print()` calls. Replace with `window.ipc.postMessage("marco_print:dialog")` so the host invokes the COM path.
2. Add an IPC arm in `wry_platform_webview.rs::on_ipc_message` for `marco_print:` that calls `self.trigger_print_dialog()`.
3. Intercept Ctrl+P at the GTK level on Windows (same as Linux) and route through `trigger_print_dialog`.

**No degradation:** `ICoreWebView2_7::ShowPrintUI(COREWEBVIEW2_PRINT_DIALOG_KIND_SYSTEM)` opens the Windows system print dialog — the same modal UX Linux users get from `webkit6::PrintOperation::run_dialog`.

### 14.8 `load_html_when_ready` retry on Windows (Gap #8, Important)

**Why it's degraded today.** Linux polls the WebView's `allocated_width` every 16 ms (up to 300 retries = ~5 s) before calling `load_html` to ensure the WebView has a valid initial layout. Windows has only a single fallback to `max(100, allocation_width)` — if GTK has not laid out the container yet, the WebView2 child renders against a wrong size and may need to be resized later (causing reflow).

**Strategy: identical glib polling loop.** This is **pure GLib code with no platform dependency**.

**Implementation plan:**
1. Lift the existing Linux retry loop from `webkit6.rs::load_html_when_ready` into a shared helper `viewer/allocation_wait.rs`:
   ```text
   pub fn run_when_allocated<W: IsA<gtk4::Widget>>(
       widget: &W,
       max_retries: u32,
       cb: impl FnOnce() + 'static,
   )
   ```
2. Both `webkit6::load_html_when_ready` and `wry_platform_webview::load_html_with_base` call this helper before invoking their respective `load_html` paths.

**No degradation:** same 16 ms × 300 retry behaviour as Linux.

### 14.9 Slider play/pause (Gap #9, Minor)

`sliders_play_all` / `sliders_pause_all` are pure JS calls (`MarcoCorePreview.sliders.playAll()` etc.). Reuse Gap #2's `evaluate_script` plumbing. Even though they are `#[allow(dead_code)]` on Linux, adding Windows parity costs almost nothing.

### 14.10 `WEBVIEW_HTML_MAP` eviction (Gap #11, Minor)

The `WEBVIEW_HTML_MAP: HashMap<u64, String>` in `wry_platform_webview.rs` grows unboundedly. Two options:
- **Drop-based eviction** — implement `Drop for PlatformWebView` that removes its own entry from the map.
- **Weak-ref map** — store `Weak<...>` keyed by id and let entries expire when the strong ref count hits zero.

Drop-based is simpler and avoids a re-entrancy hazard during shutdown. **No degradation** — Linux doesn't need this because it stores HTML on the WebView directly.

### 14.11 Detached-window API shape (cross-cutting)

`wry_detached_window::attach_webview(_widget)` ignoring its argument is confusing. **Strategy: rename to two methods that reflect the actual semantics:**
- `set_initial_html(html: String, base_uri: Option<String>)` — Windows-only.
- `attach_webview(widget: &Widget)` — Linux-only.

Or, with the `PreviewBackend` trait from §14.4, expose `attach_backend(backend: Box<dyn PreviewBackend>)` and have each platform's backend know how to make itself appear in the new window.

---

## 15. Recommended implementation order

If undertaking all of §14, the dependency graph suggests this order:

| Step | Closes gaps | Status | Rationale |
|---|---|---|---|
| 1. Allocation-wait helper (§14.8) | #8 | ✅ Done | Trivial pure-Rust refactor; unblocks confident widget creation. Lives in [`marco/src/components/viewer/allocation_wait.rs`](marco/src/components/viewer/allocation_wait.rs). |
| 2. Accept `IsA<Window>` in `PlatformWebView::new` (§14.6) | #6 | ✅ Done | Dialogs (`ui/dialogs/math.rs`, `mermaid.rs`) no longer fall back to a `Label`. |
| 3. `update_html_content_smooth` on Windows (§14.2) | #2 | ✅ Done | Implemented on `PlatformWebView` using `serde_json` escaping. Wired into the Windows branch of `refresh_preview_impl` in Step 4b. |
| 4a. `renderer.rs` un-gated (§14.4) | #4 (partial) | ✅ Done | Section-incremental renderer compiles on both targets via the cross-platform `backend` shim. |
| 4b. Windows `ui.rs` adoption of `renderer` (§14.4) | #4 (rest) | ✅ Done | Windows branch of `refresh_preview_impl` now tracks `is_initial_load`, `last_css_hash`, `last_document_path`, `last_page_view_enabled`, and `last_preview_hash`. Cursor moves / no-op refreshes are deduped by content hash. First load, CSS / theme changes, document-path changes, and any paged.js render still take a full `load_html_with_base` reload (required so the `<html data-theme>` root and `<style>` block update); all other edits go through `update_html_content_smooth`, eliminating the white flash and preserving scroll position. Section-incremental DOM patches (Linux's `prev_section_hashes` / `refresh_preview_content_sections` path) are left as a future perf optimization. |
| 5a. Shared `code_view_html` builders (§14.5) | #5 (partial) | ✅ Done | New [`code_view_html.rs`](marco/src/components/viewer/code_view_html.rs) owns the syntect+HTML page and the smooth-update JS. `webkit6::create_html_source_viewer_webview` and `update_code_view_smooth` now delegate to it (~250 lines of duplication removed). |
| 5b. wry WebView code viewer (§14.5) | #5 (rest) | ✅ Done | `wry::create_html_source_viewer_webview` now builds a real `PlatformWebView` via the shared `code_view_html::build_full_page` helper (parent window threaded through from `editor/ui.rs`), and `wry::update_code_view_smooth` dispatches the shared `build_smooth_update_js` payload through `PlatformWebView::evaluate_script`. The Windows code-view storage in `editor/ui.rs` was switched from `Rc<RefCell<Option<gtk4::Widget>>>` to `Rc<RefCell<Option<PlatformWebView>>>`; all three smooth-update call sites updated. Output is now bit-identical to the webkit6 / Linux code view (syntect highlighting + theme / scrollbar CSS). Runtime validation pending on Windows (two `PlatformWebView`s under stack switching). |
| 6a. JS find engine + IPC plumbing (§14.1) | #1 (foundation) | ✅ Done | New [`wry_find.rs`](marco/src/components/viewer/wry_find.rs) implements Tier B (CSS Custom Highlight API) with Tier A (`window.find`) fallback. `PlatformWebView::set_find_report_callback` and the `marco_find:` IPC arm in [`wry_platform_webview.rs`](marco/src/components/viewer/wry_platform_webview.rs) deliver parsed `FindReport`s to the host. |
| 6b. `FindBackend` trait + Windows search UI (§14.1) | #1 (rest) | ✅ Windows trait done / ⏳ Linux impl + UI deferred | New [`marco/src/components/viewer/find_backend.rs`](marco/src/components/viewer/find_backend.rs) defines the cross-platform `FindBackend` trait plus `FindOptions` / `FindReport` / `FindReportCallback`. `WryFindBackend` (Windows) wraps the existing [`wry_find`](marco/src/components/viewer/wry_find.rs) engine and `PlatformWebView::set_find_report_callback`; the module-level `#![allow(dead_code)]` on `wry_find.rs` is removed (only `fallback_script` keeps a scoped allow). `WebKit6FindBackend` (Linux) is a placeholder that stores the `WebView` + callback slot; method bodies carry `TODO(§14.1)` markers awaiting `webkit6::FindController` wiring when the search-window UI exposes find-in-preview. Trait surface is identical on both platforms. 3 smoke tests cover defaults + object-safety. Wiring the trait into `components/search/{window,navigation,state}.rs` and adding a Windows-side search-window UI remain deferred. |
| 7a. State snapshot/restore primitive (§14.3) | #3 (foundation) | ✅ Done | New [`preview_state.rs`](marco/src/components/viewer/preview_state.rs) defines `PreviewState` (`scroll_x`, `scroll_y`, `open_details`, `body_hash`) plus `snapshot_script()` / `restore_script(&state)` JS builders, a one-shot `LATEST_PREVIEW_STATE` slot (`set_latest_state` / `take_latest_state`), and `parse_snapshot_payload`. `PlatformWebView::set_state_snapshot_callback` and `request_state_snapshot` plus the `marco_state:` IPC arm in [`wry_platform_webview.rs`](marco/src/components/viewer/wry_platform_webview.rs) deliver parsed `PreviewState` snapshots to the host (and auto-stash in the global slot). |
| 7b. Detach-flow wiring (§14.3) | #3 (rest) | ✅ Done | New `PlatformWebView::set_ready_callback` in [`wry_platform_webview.rs`](marco/src/components/viewer/wry_platform_webview.rs) (fires on the existing `marco_zoom:ready` IPC). `wry_detached_window::attach_webview` in [`wry_detached_window.rs`](marco/src/components/viewer/wry_detached_window.rs) installs a one-shot `take_latest_state` → `restore_script` handler. Detach trigger sites in [`menu.rs`](marco/src/menu.rs) and [`viewer/mod.rs`](marco/src/components/viewer/mod.rs) call `request_state_snapshot()` on the editor's live WebView before handing off to the detached window. Runtime validation (verify scroll position + open `<details>` survive detach round-trip) still recommended on the Windows host. |
| 8. Print path audit + IPC routing (§14.7) | #7 | ✅ Done | Audited [`viewer/javascript.rs`](marco/src/components/viewer/javascript.rs) (incl. `WIN_ZOOM_BAR_HTML`) — no in-page `window.print()` calls exist, so no replacements were needed. `app.print` is already bound to `<Control>p` on both targets via `set_accels_for_action` in [`main.rs`](marco/src/main.rs), routed through `print_driver_windows::trigger_print_dialog` → `ICoreWebView2_7::ShowPrintUI(SYSTEM)`. New defensive `marco_print:` IPC arm in [`wry_platform_webview.rs`](marco/src/components/viewer/wry_platform_webview.rs) routes any future in-page print request through `show_system_print_ui()` with `view.print()` fallback. |
| 9a. Sliders play/pause parity (§14.9) | #9 (partial) | ✅ Done | New `sliders_play_all` / `sliders_pause_all` in [`wry.rs`](marco/src/components/viewer/wry.rs) mirror the WebKit6 helpers byte-for-byte using the new `evaluate_script` plumbing. Both remain `#[allow(dead_code)]` because no caller is wired yet on either backend. |
| 9b. `WEBVIEW_HTML_MAP` drop-eviction (§14.10) | #11 | ✅ Done | New private `IdGuard(u64)` in [`wry_platform_webview.rs`](marco/src/components/viewer/wry_platform_webview.rs) implements `Drop` to remove the matching `WEBVIEW_HTML_MAP` entry. Held as `Rc<IdGuard>` inside `PlatformWebView`, so eviction fires only when the last clone is dropped — safe against the existing `#[derive(Clone)]`. |
| 9c. Detached-window API rename (§14.11) | #9 (rest) | ✅ Done | `attach_webview(&self, _webview: Option<&gtk4::Widget>)` renamed to `load_preview_content(&self)` (parameter removed). All four call sites updated: `wry_detached_window.rs` (×2), `menu.rs`, `viewer/mod.rs`. |

**Current end state (June 2026):** All originally-critical, originally-important, and originally-minor gaps from §11 are resolved at the code level and verified clean on `x86_64-pc-windows-msvc` (427 tests pass): smooth updates (#2, Step 3+4b), detached-window state capture/restore (#3, Steps 7a+7b), generation counter + in-flight guard (#4, this session), syntect code-view (#5, Step 5b), `IsA<Window>` dialogs (#6, Step 2), print IPC routing + dead-fallback cleanup (#7, Step 8 + this session), allocation-wait helper (#8, Step 1), sliders parity (#9, Step 9a), `IdGuard` map eviction (#11, Step 9b), `load_preview_content` rename (#9c, this session), find-in-preview wired (#1, this session — `wry_find::search/next/prev/clear` called from `engine.rs`, `navigation.rs`, `state.rs`). **Only remaining open item:** section-incremental DOM patching (`prev_section_hashes` / `refresh_preview_content_sections`) is still Linux-only — a future performance optimisation with no user-visible gap. Runtime validation on Windows is the recommended next step.

---

## 16. What is genuinely impossible to fix

For completeness — items where parity simply cannot be reached without abandoning wry/WebView2:

| Item | Reason |
|---|---|
| True WebView reparenting across top-level windows | WebView2 child WebViews are bound to one HWND for their lifetime. **No fix possible.** §14.3 approach (a) is the best achievable surrogate. |
| Synchronous JS evaluation result returning a Rust value in the same call | `wry::WebView::evaluate_script_with_callback` is async-only on Windows. WebKit6 has the same restriction now (GTK4 removed the sync API), so this is not a regression — both platforms must use callbacks. |
| Translucent backgrounds with arbitrary alpha on Windows < 8 | wry doc explicitly notes alpha ≠ 0 is replaced with 255 on Windows 7. Affects almost no users (Marco's MSRV likely Windows 10+). |
| Inspector (DevTools) close on Windows | `wry::WebView::close_devtools` documented as "not supported" on Windows. Cosmetic only. |

Everything else in §11 is closeable with the strategies in §14.

- [marco/src/components/search/state.rs](marco/src/components/search/state.rs)
- [marco/src/ui/dialogs/search.rs](marco/src/ui/dialogs/search.rs)
- [marco/src/ui/dialogs/math.rs](marco/src/ui/dialogs/math.rs)
- [marco/src/ui/dialogs/mermaid.rs](marco/src/ui/dialogs/mermaid.rs)
- [marco/src/ui/toc_panel.rs](marco/src/ui/toc_panel.rs)
- [marco/src/ui/menu_items/file_operations.rs](marco/src/ui/menu_items/file_operations.rs)

### polo

- [polo/src/components/viewer/platform_webview.rs](polo/src/components/viewer/platform_webview.rs)
- [polo/src/components/viewer/rendering.rs](polo/src/components/viewer/rendering.rs)
- [polo/src/components/viewer/loading_overlay.rs](polo/src/components/viewer/loading_overlay.rs)
- [polo/src/components/viewer/empty_state.rs](polo/src/components/viewer/empty_state.rs)
- [polo/src/main.rs](polo/src/main.rs)
