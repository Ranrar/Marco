# Webview Backend Unification: webkit6 + wry/WebView2 → wry (gtk4-webkit6 fork)

**Status:** Implemented on branch `unified-webviewer` (2026-07-16) — see §8 for outcome and follow-ups
**Date:** 2026-07-14
**Scope:** `marco`, `polo`, workspace `Cargo.toml`, CI, docs

## 1. Goal

Replace Marco's dual webview backend — `webkit6` (native GTK4 WebKit) on Linux
and `wry`/WebView2 on Windows — with a **single `wry` dependency** on both
platforms, using the GTK4 + WebKit6 port of wry:

> https://github.com/Ranrar/wry/tree/gtk4-webkit6

This removes ~2,500 lines of duplicated backend code and the majority of
`#[cfg(target_os = ...)]` blocks that exist only to pick a backend.

## 2. Audit: current state

### 2.1 Dependency layout

Workspace root `Cargo.toml` (lines 39–47) declares the split; both apps
consume it:

| Dependency | Platform gate | Used by | Notes |
|---|---|---|---|
| `webkit6 0.6.1` | Linux only | marco, polo | Native preview backend |
| `wry 0.55` (`os-webview`, `protocol`) | Windows only | marco, polo | WebView2 backend |
| `tao 0.35` | Windows only | marco, polo | **Dead — never referenced in any source file** |
| `gdk4-win32`, `raw-window-handle` | Windows only | marco, polo | HWND acquisition for embedding WebView2 in the GTK window |
| `webview2-com`, `windows`, `windows-core` | Windows only | marco, polo | Raw COM: `PrintToPdf` (marco), `ShowPrintUI` (polo); version-matched to wry 0.55 → webview2-com 0.38.2 |
| `rfd`, `ctrlc`, `embed-resource`/`winres` | Windows only | marco, polo | Not webview-related; stays |

There are **no** direct `javascriptcore6` or `soup3` dependencies anywhere —
nothing to remove on that front.

### 2.2 Duplicated backend modules (marco/src/components/viewer/)

| Feature | Linux (webkit6) | Windows (wry) |
|---|---|---|
| Live preview webview | `webkit6.rs` (964 lines) | `wry_platform_webview.rs` (1,122 lines) |
| Detached preview window | `webkit6_detached_window.rs` (635) | `wry_detached_window.rs` (565) |
| Webview reparenting | `reparenting.rs` (411) | n/a — detached window rebuilds its own webview (§14.3) |
| Find in preview | webkit6 `FindController` (via `find_backend.rs`) | JS engine `wry_find.rs` (493) |
| Print / PDF export | `print_driver.rs` (194, WebKit `PrintOperation`) | `print_driver_windows.rs` (124) + `wry_print_to_pdf.rs` (245, WebView2 COM) |
| Base-URI / local assets | webkit6 `load_html(html, base_uri)` | custom protocol + per-webview HTML map (`wry.rs`, 224) |

Dispatch happens through cfg'd **type aliases**:

- `preview_types.rs:35-38` — `PlatformWebView`
- `backend.rs` — `PreviewWebView` + cfg'd free functions
  (`load_html_when_ready`, `update_html_content_smooth`, `evaluate_javascript`)
- `viewer/mod.rs` — `PreviewWindowType`, cfg'd module declarations (16 cfg blocks)
- `ui/dialogs/mermaid.rs`, `math.rs` — `PreviewSurface`
- `ui/dialogs/search.rs` — direct cfg'd imports
- `menu.rs`, `main.rs` — cfg'd `PreviewWindow` types and init paths

### 2.3 Cfg hotspots outside the viewer module (marco)

| File | cfg blocks | Nature |
|---|---|---|
| `components/editor/ui.rs` | 38 | Backend selection — removable |
| `components/editor/scroll_sync.rs` | 11 | Backend selection — removable |
| `components/editor/editor_manager.rs` | 9 | Backend selection — removable |
| `menu.rs`, `main.rs` | several | Detached window type + webkit6 link handler |
| `components/search/*` (state, engine, navigation, window) | several | FindController vs MarcoFind |
| `ui/toc_panel.rs`, `theme.rs`, `logic/signal_handlers.rs` | few | webkit6 JS eval calls |
| `ui/dialogs/{mermaid,math,search,export,export_complete}.rs` | few | PreviewSurface / export paths |

`marco-shared` cfgs (paths, fonts, file logging) are genuine platform code —
**out of scope, do not touch**.

### 2.4 polo

Same split, smaller: `platform_webview.rs` (803 lines, 27 cfg blocks),
`loading_overlay.rs`, `dialog.rs`, `main.rs`.

### 2.5 Build / CI

- `ci-linux.yml` and `release.yml` **already install `libwebkitgtk-6.0-dev`**
  (the webkit6 crate requires it today, and the fork links the same library).
  No Linux system-package change is needed.
- No old `libwebkit2gtk-4.x` references exist anywhere.
- Windows CI has no webview-specific system steps; WebView2 resolves through
  wry's crates.

### 2.6 Docs referencing the dual backend

- `README.md:148` — "WebKit6 / WebView2 … `webkit6` on Linux, `wry`/WebView2 on Windows"
- `documentation/contributions/architecture.md` — contains the
  "webkit6→wry parity" plan (the "§14.x" references in code comments)
- `build/README.md`, `build/linux/README.md`, `build/linux/build_deb.sh` —
  mention webkit packages (verify, likely already webkit6)

## 3. Fork validation (inspected, branch `gtk4-webkit6`)

The fork is a real GTK4/WebKit6 port of wry's Linux backend and is
**ecosystem-compatible with Marco's pins**: it builds on `webkit6 0.6` and
`gtk4 0.11` (workspace pins `webkit6 0.6.1`, `gtk4 0.11.2`).

APIs Marco needs, all present:

| Need | Fork API |
|---|---|
| Embed webview in existing GTK4 layout | `WebViewBuilderExtUnix::build_gtk(&widget)` — appends into `gtk4::Box`, `Fixed::put` for `gtk4::Fixed`, `set_child` otherwise |
| Move webview between windows (detached preview) | `WebViewExtUnix::reparent_gtk(&widget)` — replaces Marco's manual `reparenting.rs` |
| Raw WebKit access (print, settings) | `WebViewExtUnix::webview() -> webkit6::WebView` escape hatch |
| IPC (scroll-sync, hover, find, state snapshot) | `with_ipc_handler` |
| Serve HTML + local assets | `with_custom_protocol` (+ async variant) |
| Bootstrap JS | `with_initialization_script` / `_for_main_only` |
| JS execution | `evaluate_script` |

Caveats found:

1. The fork does **not** re-export `webkit6`. If Marco keeps any raw WebKit
   call (e.g. native `PrintOperation` on Linux), it needs a direct
   `webkit6 = "0.6"` Linux dep to *name* the type returned by `.webview()`.
   Acceptance criteria allow this ("unless still required elsewhere") — keep
   it minimal or eliminate it.
2. wry has no rich print API — Linux print must either go through the
   `.webview()` escape hatch to WebKit `PrintOperation`, or move to a
   JS/`window.print()` approach. **Decision needed during Phase 5.**
3. `build_gtk` panics if `gtk4::init` hasn't run on the thread — fine, Marco
   is a GTK app, but webview construction must stay on the main thread.

## 4. Strategy

**Promote `wry_platform_webview.rs` to the single cross-platform
implementation; delete the webkit6-native modules.**

The Windows wrapper already implements every preview feature over portable
wry APIs (string IPC channels `marco_scroll:`/`marco_hover:`/`marco_find:`/
`marco_state:`/`marco_zoom:ready`, custom-protocol HTML serving keyed by
webview id, JS bootstrap). The only genuinely platform-specific part is
**construction/embedding**:

- Linux: `WebViewBuilder::…​.build_gtk(&container)`
- Windows: existing HWND path (GDK surface → `raw-window-handle` → child window)

Everything above the constructor becomes one code path. Cfgs that select a
backend disappear; cfgs inside the wrapper for HWND plumbing and WebView2 COM
print remain (they are platform-specific by necessity, not backend choice).

Bonus unification: the JS `MarcoFind` engine (`wry_find.rs`) works on any
webview → it replaces webkit6 `FindController` on Linux too, deleting that
whole branch of `find_backend.rs` and the search-dialog cfgs.

## 5. Phased plan

### Phase 0 — Audit ✅ (this document)

### Phase 1 — Dependencies
1. Workspace `Cargo.toml`: `wry = { git = "https://github.com/Ranrar/wry", branch = "gtk4-webkit6", default-features = false, features = ["protocol", ...] }`
   (confirm the fork's Linux feature name, e.g. `webkit6`/`os-webview`, from its `Cargo.toml`).
2. `marco/Cargo.toml`, `polo/Cargo.toml`: move `wry` from the
   `[target.'cfg(windows)']` section to common `[dependencies]`.
3. Delete `tao` everywhere (dead).
4. Keep `gdk4-win32`, `raw-window-handle`, `webview2-com`, `windows`,
   `windows-core` Windows-only (embedding + PrintToPdf).
5. Decide direct `webkit6` dep: keep as slim Linux dep only if Phase 5 keeps
   native `PrintOperation`; otherwise delete.
6. Regenerate `Cargo.lock`; confirm single wry resolution.

### Phase 2 — Cross-platform PlatformWebView
1. Rework `wry_platform_webview.rs` (consider renaming to
   `platform_webview.rs`): cfg only the constructor —
   Linux `build_gtk`, Windows HWND child-window path.
2. Verify resize/visibility handling on Linux (GTK manages the widget; the
   Windows manual bounds-tracking code becomes Windows-only or unnecessary).
3. Base URI / local images: unify on the custom-protocol approach or use
   `load_html` + base-uri via the escape hatch — pick one, document it.

### Phase 3 — Alias + dispatch collapse
1. `preview_types::PlatformWebView`, `backend::PreviewWebView`,
   dialog `PreviewSurface`, `viewer/mod.rs`/`menu.rs` `PreviewWindowType`
   → one un-cfg'd type.
2. Collapse `backend.rs` free functions to single bodies.
3. Un-gate `renderer.rs` (removes the `#[allow(dead_code)]` noted in
   `backend.rs:95`).

### Phase 4 — Feature merges
1. **Detached window**: merge `webkit6_detached_window.rs` +
   `wry_detached_window.rs` into one `detached_window.rs`. Linux uses
   `reparent_gtk` (state-preserving, current UX); Windows keeps
   snapshot/restore (§14.3). Delete `reparenting.rs` if fully superseded.
2. **Find**: `MarcoFind` JS engine on both platforms; delete webkit6
   `FindController` path in `find_backend.rs`, `components/search/*`,
   `ui/dialogs/search.rs`.
3. **Scroll-sync**: single IPC path in `scroll_sync.rs` (11 cfgs → 0).
4. **Local-file link handler**: port `main.rs:762`
   (`webkit6::setup_local_file_link_handler`) onto wry navigation/IPC.

### Phase 5 — Print / export
1. Decide Linux print: (a) `.webview()` → WebKit `PrintOperation`
   (keeps direct webkit6 dep) or (b) JS `window.print()`.
2. Keep `wry_print_to_pdf.rs` (WebView2 COM) Windows-only.
3. Reconcile `print_driver.rs` / `print_driver_windows.rs` /
   `export_pipeline.rs` (6 cfgs) behind one driver interface.

### Phase 6 — Sweep remaining cfgs (marco)
`editor/ui.rs` (38), `viewer/mod.rs` (16), `editor_manager.rs` (9),
`javascript.rs`, `code_view_html.rs`, `loading_overlay.rs`,
`preview_state.rs`, `allocation_wait.rs`, `menu.rs`, `main.rs`,
`toc_panel.rs`, `theme.rs`, `signal_handlers.rs`,
`ui/dialogs/{mermaid,math,export,export_complete}.rs`, `ui/toolbar/link.rs`,
`ui/settings/tabs/debug.rs`, `ui/menu_items/files.rs`.
Rule: a cfg survives only if it gates genuinely platform-specific behavior
(paths, rfd, ctrlc, HWND, COM), never backend choice.

### Phase 7 — polo
Same treatment: `platform_webview.rs` (27 cfgs), `loading_overlay.rs`,
`dialog.rs`, `main.rs`. Keep `ShowPrintUI` COM path Windows-only.

### Phase 8 — Delete dead code
`viewer/webkit6.rs`, `webkit6_detached_window.rs`, `wry.rs` (absorb helpers),
`reparenting.rs` (if superseded), `find_backend.rs` webkit6 branch, any
leftover cfg'd module decls in `viewer/mod.rs`; fix stale module docs in
`viewer/mod.rs` header ("Windows: Not yet implemented" is wrong today).

### Phase 9 — Build / CI / docs
1. CI: Linux packages unchanged (`libwebkitgtk-6.0-dev` already present);
   verify Windows job resolves WebView2 through the fork; commit new
   `Cargo.lock` (git dependency).
2. Check `build/linux/build_deb.sh` runtime deps name webkitgtk-6.0.
3. Docs: `README.md:148`, `architecture.md` §14 parity plan (mark completed /
   rewrite as "unified wry backend"), `build/README.md`,
   `CONTRIBUTING.md` if it lists system deps.

### Phase 10 — Verification
- `cargo build` + `cargo test` on Linux and Windows.
- Manual matrix, both platforms: HTML preview render · CSS theme switch
  (light/dark) · scroll-sync both directions · local images relative to the
  document · find-in-preview · detached preview window (state preserved) ·
  mermaid + math dialogs · print + PDF export · RTL documents ·
  multiple editor tabs (per-webview HTML map).

## 6. Risks / open questions

| # | Risk | Mitigation |
|---|---|---|
| 1 | Fork feature-flag names / build differ from wry 0.55 release | Confirmed webkit6/gtk4 versions match; confirm feature names in Phase 1 before anything else |
| 2 | Linux print parity (wry lacks a print API) | Decision point in Phase 5; `.webview()` escape hatch keeps current behavior |
| 3 | `reparent_gtk` semantics vs Marco's manual reparenting (focus, size, scroll position) | Test detached-window flow early in Phase 4 |
| 4 | Custom-protocol HTML serving on WebKit6 (scheme registration differences vs WebView2) | Fork exposes the same `with_custom_protocol` API; verify local-image loading in Phase 2 |
| 5 | Git dependency = no crates.io pin | Pin a `rev =` instead of `branch =` for reproducible builds; revisit if fork is upstreamed/published |
| 6 | Fork maintenance burden (tracking upstream wry) | Out of scope here; note as ongoing cost |

## 7. Acceptance criteria

- [ ] Single `wry` dependency (gtk4-webkit6 fork) is the only webview backend;
      no `tao`; direct `webkit6` only if Phase 5 keeps native print (and then
      only as a slim Linux dep).
- [ ] No `#[cfg(target_os = ...)]` in preview/webview code that exists purely
      to choose a backend.
- [ ] Preview features (HTML render, CSS theming, scroll-sync, local images,
      find, detached window, print/export) work identically on both platforms.
- [ ] `cargo build` and `cargo test` pass on Linux and Windows.
- [ ] Linux builds against `libwebkitgtk-6.0-dev` (already true); Windows
      still resolves WebView2 through wry.
- [ ] `README.md`, `architecture.md`, and build docs describe a single
      wry-based preview engine.

## 8. Outcome (2026-07-16, branch `unified-webviewer`)

Implemented. `wry` (gtk4-webkit6 fork, pinned via `rev =` to commit
`e00a02d5` — `ee1dc548` plus a fix for an IPC panic on `load_html` URIs,
resolving Risk #5's reproducibility concern) is the single webview backend in
both apps; `tao` deleted; `webkit6` remains a slim
direct Linux dep for `PrintOperation` and the `load_html(html, base_uri)`
escape hatch. `viewer/webkit6.rs` (~964 lines) deleted; dialogs, scroll-sync,
find/search, hover, zoom, code view, export lifecycle, and the local-`.md`
link handler are single code paths. Both apps build warning-free on Linux;
`marco` 429/429 and `polo` 43/43 tests pass.

Remaining `#[cfg(target_os = ...)]` splits are **strategy- or OS-specific,
not backend selection**:

- Detached preview window: Linux reparents the live webview
  (`detached_window_linux` + `reparenting`), Windows rebuilds from the
  recorded preview HTML (`detached_window_windows`, §14.3). **Not** a
  candidate for a `wry` `reparent`-API merge — investigated 2026-07-16, see
  `WRY_FORK_GAPS.md` §7.
- Print/PDF: WebKit `PrintOperation` (Linux) vs WebView2 COM
  `ShowPrintUI`/`PrintToPdf` (Windows); dispatched from shared actions.
- The `export` action in `main.rs` is still one large cfg'd pair (both sides
  already drive the unified `export_pipeline`) — follow-up cleanup.
- Win32 HWND embedding, `rfd`/`ctrlc`, and file-dialog plumbing.

Verification still owed: a Windows CI build/run, and manual feature passes on
both platforms (§ Phase 10 of the plan).

### 8.1 Follow-up cleanup (2026-07-16)

- `print_driver_windows.rs` + `wry_print_to_pdf.rs` merged into one file
  (`print_driver_windows.rs`), matching the Linux `print_driver.rs` shape of
  one file for both the dialog and PDF-export paths.
- `find_backend.rs` deleted: it was a `FindBackend` trait scaffold wrapping
  `wry_find.rs`'s functions 1:1 (originally meant to abstract over a Linux
  `webkit6::FindController` implementation that the unification made
  unnecessary). Nothing constructed it outside its own tests; callers already
  used `wry_find::*` directly.

### 8.2 Renames to drop stale backend-name prefixes (2026-07-16)

Several files still carried `wry_`/`webkit6_` prefixes from the pre-unification
dual-backend layout even though the code itself is now either fully
cross-platform or split by OS strategy, not by backend. Renamed to match
current reality (old name → new name):

| Old | New | Why |
|---|---|---|
| `wry_platform_webview.rs` | `platform_webview.rs` | Cross-platform; matches `polo`'s file of the same purpose (this was the Phase 2 suggestion above). |
| `wry.rs` | `preview_helpers.rs` | Cross-platform grab-bag (latest-HTML/base-URI cache, code-view webview, external-link opener) — never Windows-only. |
| `wry_find.rs` | `find_engine.rs` | `MarcoFind` runs on both platforms since Phase 4; nothing "wry-specific" about it. |
| `webkit6_detached_window.rs` | `detached_window_linux.rs` | Linux-only by *strategy* (reparents the live webview), not by backend — it doesn't call raw `webkit6::` APIs. |
| `wry_detached_window.rs` | `detached_window_windows.rs` | Windows-only by *strategy* (rebuilds from recorded HTML, §14.3), not by backend. |
| `print_driver.rs` | `print_driver_linux.rs` | Symmetry with `print_driver_windows.rs`. |

`print_driver_windows.rs` was already correctly named and is unchanged.
