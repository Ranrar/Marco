//! Cross-platform "preview state" snapshot / restore primitive.
//!
//! On Linux, the marco preview lives in a `webkit6::WebView` that can be
//! reparented between the main editor window and the detached preview
//! window, so scroll position, expanded `<details>` panels and form state
//! all carry over for free. On Windows the `wry::WebView` is a child of a
//! specific Win32 HWND and **cannot be reparented** — this is a hard
//! WebView2 limitation (see §14.3 / §16 of the parity audit).
//!
//! This module implements approach **(a)** from §14.3: rather than
//! reparenting, we *capture* the page's user-visible state in JS, ship it
//! through IPC, store it in a thread-safe global, and *restore* it in the
//! freshly-built WebView once it signals `marco_zoom:ready`.
//!
//! ## Wire protocol
//!
//! - **Snapshot request:** host calls
//!   [`PlatformWebView::request_state_snapshot`], which evaluates
//!   [`snapshot_script`]. The script gathers `scrollX`, `scrollY`, open
//!   `<details>` ids and the body content hash, then posts back
//!   `marco_state:{...json...}` IPC.
//! - **Snapshot delivery:** the IPC handler in `wry_platform_webview.rs`
//!   parses the payload via [`parse_snapshot_payload`] and invokes the
//!   callback installed by `set_state_snapshot_callback`.
//! - **Restore:** host calls [`PlatformWebView::evaluate_script`] with the
//!   output of [`restore_script`]. The script tries
//!   `window.MarcoCorePreview.restoreState(state)` first; if the preview
//!   bootstrap isn't loaded it falls back to direct DOM manipulation
//!   (scroll + `<details open>`).
//!
//! ## State scope
//!
//! `PreviewState` captures only what the user observes and what
//! `MarcoCorePreview.restoreState` is documented to accept on Linux:
//!
//! - Scroll position (`scrollX`, `scrollY`).
//! - The set of `<details>` element `id`s that are currently `open`.
//! - The body content hash — used as a "freshness sentinel" so a
//!   stale restore (against a document that has since changed) can be
//!   detected and skipped (see [`PreviewState::is_compatible_with`]).
//!
//! Selection ranges, scroll containers inside Mermaid SVGs and
//! ephemeral KaTeX render state are **out of scope**: WebKit6 doesn't
//! preserve those across reparenting either, so this is full parity.

use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};

/// Snapshot of user-visible preview state. Round-trips through JSON to and
/// from the in-page JS engine, so all fields must be `Serialize +
/// Deserialize`-clean and use `serde(default)` for forward compatibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PreviewState {
    /// Horizontal scroll position in CSS pixels.
    #[serde(default)]
    pub scroll_x: f64,
    /// Vertical scroll position in CSS pixels.
    #[serde(default)]
    pub scroll_y: f64,
    /// `id` of every `<details>` element that is currently `open`.
    /// Elements without an `id` are skipped — they are not stable
    /// targets across re-renders.
    #[serde(default)]
    pub open_details: Vec<String>,
    /// 32-bit FNV-1a hash of `document.body.innerText` at capture time.
    ///
    /// Used as a coarse freshness check: if the body text changes between
    /// snapshot and restore (e.g. the user typed something), the restore
    /// can choose to apply only the scroll fields and skip the
    /// element-level state.
    #[serde(default)]
    pub body_hash: u32,
}

impl PreviewState {
    /// Returns `true` when both states reference the same body content.
    ///
    /// A `body_hash == 0` (default/unset) on either side is treated as
    /// "unknown" and conservatively returns `true` so callers fall back to
    /// best-effort restoration rather than dropping the snapshot entirely.
    #[allow(dead_code)]
    pub fn is_compatible_with(&self, other_body_hash: u32) -> bool {
        self.body_hash == 0 || other_body_hash == 0 || self.body_hash == other_body_hash
    }
}

/// Process-wide slot for the most recent snapshot. The detach flow:
///
/// 1. Editor pre-empts a snapshot before closing/hiding the in-editor WebView
///    and stores it via [`set_latest_state`].
/// 2. Detached window opens its own WebView, waits for `marco_zoom:ready`,
///    reads via [`take_latest_state`] (atomically clears) and dispatches
///    `restore_script(&state)`.
///
/// The slot is intentionally a one-shot — using [`take_latest_state`] avoids
/// applying the same stale snapshot twice if the detach is cancelled and
/// retried, which would silently scroll the user back to an outdated
/// position.
static LATEST_PREVIEW_STATE: OnceLock<Mutex<Option<PreviewState>>> = OnceLock::new();

fn latest_state_mutex() -> &'static Mutex<Option<PreviewState>> {
    LATEST_PREVIEW_STATE.get_or_init(|| Mutex::new(None))
}

/// Replace the latest stored snapshot. A `None` argument clears the slot.
#[allow(dead_code)]
pub fn set_latest_state(state: Option<PreviewState>) {
    if let Ok(mut guard) = latest_state_mutex().lock() {
        *guard = state;
    }
}

/// Non-destructively read the latest snapshot. Returns `None` if no
/// snapshot has been stored or the mutex is poisoned.
///
/// Reserved for diagnostics; the detach flow uses [`take_latest_state`]
/// for its one-shot semantics.
#[allow(dead_code)]
pub fn peek_latest_state() -> Option<PreviewState> {
    latest_state_mutex().lock().ok().and_then(|g| g.clone())
}

/// Atomically consume and return the latest snapshot (one-shot semantics).
#[allow(dead_code)]
pub fn take_latest_state() -> Option<PreviewState> {
    latest_state_mutex().lock().ok().and_then(|mut g| g.take())
}

/// Parse the `marco_state:` IPC payload (everything after the colon) into a
/// [`PreviewState`]. Returns `None` on malformed / non-JSON payloads.
#[allow(dead_code)]
pub fn parse_snapshot_payload(payload: &str) -> Option<PreviewState> {
    match serde_json::from_str::<PreviewState>(payload) {
        Ok(state) => Some(state),
        Err(e) => {
            log::debug!(
                "[preview_state] failed to parse snapshot payload ({}): {:?}",
                e,
                payload
            );
            None
        }
    }
}

/// Top-level JavaScript that captures state and posts it back via IPC.
///
/// Posted message format: `marco_state:{"scroll_x":...,"scroll_y":...,...}`.
/// Idempotent — calling it multiple times just emits multiple snapshots;
/// the host's stored slot keeps the latest.
#[allow(dead_code)]
pub fn snapshot_script() -> &'static str {
    r#"
(function() {
    try {
        // FNV-1a 32-bit hash over the body text. Deterministic and matches
        // the Rust-side `body_hash` field semantically (PreviewState docs).
        function fnv1a(s) {
            var h = 0x811c9dc5 >>> 0;
            for (var i = 0; i < s.length; i++) {
                h ^= s.charCodeAt(i);
                h = (h + ((h << 1) + (h << 4) + (h << 7) + (h << 8) + (h << 24))) >>> 0;
            }
            return h >>> 0;
        }

        var openDetails = [];
        var allDetails = document.querySelectorAll('details[open][id]');
        for (var i = 0; i < allDetails.length; i++) {
            var id = allDetails[i].getAttribute('id');
            if (id) openDetails.push(id);
        }

        var bodyText = (document.body && document.body.innerText) ? document.body.innerText : '';
        var state = {
            scroll_x: window.scrollX || 0,
            scroll_y: window.scrollY || 0,
            open_details: openDetails,
            body_hash: fnv1a(bodyText)
        };

        var msg = 'marco_state:' + JSON.stringify(state);
        if (window.ipc && window.ipc.postMessage) {
            window.ipc.postMessage(msg);
        } else {
            console.warn('[MarcoPreviewState] window.ipc.postMessage unavailable; snapshot dropped');
        }
    } catch (e) {
        console.error('[MarcoPreviewState] snapshot failed:', e);
    }
})();
"#
}

/// Build a JavaScript snippet that restores `state` in the live document.
///
/// The script:
/// 1. Tries `window.MarcoCorePreview.restoreState(state)` so the preview
///    bootstrap can apply any framework-specific re-hydration first
///    (Mermaid pan/zoom reset, KaTeX re-render guards, etc.).
/// 2. Falls back to direct DOM manipulation: re-open every captured
///    `<details>` by id and `window.scrollTo(scrollX, scrollY)`.
///
/// `state` is serialized with `serde_json` so any future field additions
/// flow through automatically. The fallback intentionally ignores
/// `body_hash` — it is a host-side compatibility hint, not something the
/// browser needs to act on.
#[allow(dead_code)]
pub fn restore_script(state: &PreviewState) -> Result<String, String> {
    let state_json = serde_json::to_string(state)
        .map_err(|e| format!("Failed to serialize PreviewState: {}", e))?;

    Ok(format!(
        r#"
(function() {{
    try {{
        var state = {state};
        // Preferred path: hand off to the preview's own restore handler when
        // it exists. The handler may apply framework-specific re-hydration
        // before scrolling.
        if (window.MarcoCorePreview && typeof window.MarcoCorePreview.restoreState === 'function') {{
            try {{
                window.MarcoCorePreview.restoreState(state);
                return;
            }} catch (e) {{
                console.warn('[MarcoPreviewState] restoreState handler threw; falling back:', e);
            }}
        }}

        // Fallback path: best-effort direct DOM restore.
        if (state.open_details && state.open_details.length) {{
            for (var i = 0; i < state.open_details.length; i++) {{
                var el = document.getElementById(state.open_details[i]);
                if (el && el.tagName === 'DETAILS') {{
                    el.open = true;
                }}
            }}
        }}
        // Scroll is applied last so any layout shifts from re-opening
        // <details> elements don't fight the scroll target.
        var x = (typeof state.scroll_x === 'number') ? state.scroll_x : 0;
        var y = (typeof state.scroll_y === 'number') ? state.scroll_y : 0;
        window.scrollTo(x, y);
    }} catch (e) {{
        console.error('[MarcoPreviewState] restore failed:', e);
    }}
}})();
"#,
        state = state_json,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_round_trip_through_json() {
        let s = PreviewState {
            scroll_x: 12.5,
            scroll_y: 480.0,
            open_details: vec!["a".to_string(), "b".to_string()],
            body_hash: 0xdead_beef,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: PreviewState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn smoke_test_parse_snapshot_payload_accepts_partial() {
        // Forward-compatible: unknown fields ignored, missing fields default.
        let p = parse_snapshot_payload(r#"{"scroll_y": 200, "unknown_future_field": true}"#)
            .expect("parse");
        assert_eq!(p.scroll_y, 200.0);
        assert_eq!(p.scroll_x, 0.0);
        assert!(p.open_details.is_empty());
    }

    #[test]
    fn smoke_test_parse_snapshot_payload_rejects_garbage() {
        assert!(parse_snapshot_payload("not json").is_none());
        assert!(parse_snapshot_payload("").is_none());
    }

    #[test]
    fn smoke_test_take_latest_state_is_one_shot() {
        let s = PreviewState {
            scroll_y: 99.0,
            ..Default::default()
        };
        set_latest_state(Some(s.clone()));
        assert_eq!(take_latest_state(), Some(s));
        // Second take returns None.
        assert_eq!(take_latest_state(), None);
        // Cleanup any cross-test residue.
        set_latest_state(None);
    }

    #[test]
    fn smoke_test_is_compatible_with_treats_zero_as_unknown() {
        let s = PreviewState {
            body_hash: 0,
            ..Default::default()
        };
        assert!(s.is_compatible_with(123));

        let s = PreviewState {
            body_hash: 42,
            ..Default::default()
        };
        assert!(s.is_compatible_with(0));
        assert!(s.is_compatible_with(42));
        assert!(!s.is_compatible_with(43));
    }

    #[test]
    fn smoke_test_snapshot_script_invariants() {
        // Hard-coded guards so accidental refactors trip CI.
        let s = snapshot_script();
        assert!(s.contains("marco_state:"));
        assert!(s.contains("window.ipc.postMessage"));
        assert!(s.contains("scroll_x"));
        assert!(s.contains("scroll_y"));
        assert!(s.contains("open_details"));
        assert!(s.contains("body_hash"));
    }

    #[test]
    fn smoke_test_restore_script_embeds_state_safely() {
        let s = PreviewState {
            scroll_x: 1.0,
            scroll_y: 2.0,
            open_details: vec!["my-id\"with-quote".to_string()],
            body_hash: 1,
        };
        let js = restore_script(&s).expect("script");
        // `evaluate_script` takes raw JS (not HTML), so the only injection
        // surface is JS string-literal escaping done by serde_json. The
        // embedded quote inside `open_details[0]` must be escaped.
        assert!(js.contains(r#"my-id\"with-quote"#));
        // Restoration entrypoints must be present.
        assert!(js.contains("MarcoCorePreview"));
        assert!(js.contains("restoreState"));
        assert!(js.contains("window.scrollTo"));
        // Scroll coordinates make it through serde_json verbatim.
        assert!(js.contains("\"scroll_x\":1.0"));
        assert!(js.contains("\"scroll_y\":2.0"));
    }
}
