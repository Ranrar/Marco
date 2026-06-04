//! JavaScript-based "find in preview" backend for Windows / `wry::WebView`.
//!
//! WebKit6 exposes a native `WebView::find_controller()` with built-in
//! highlight, count, and forward/backward navigation. WebView2 / `wry` has no
//! equivalent — Edge's only built-in primitive is the non-standard
//! `window.find(query, caseSensitive, backwards, wrap, wholeWord, ...)` which
//! returns a `bool`, can't count matches, and styles via the user's selection
//! color rather than as proper highlights.
//!
//! This module implements the **Tier B + Tier C** strategy from §14.1 of the
//! `webkit6_wry_parity_audit.md`:
//!
//! * **Tier B — CSS Custom Highlight API.** All matches are computed in JS
//!   via `document.createTreeWalker(...)` and registered with
//!   `CSS.highlights.set("marco-find", new Highlight(...ranges))`. The
//!   matches are then painted by a `::highlight(marco-find)` CSS rule
//!   injected by the page. This works on every WebView2 Evergreen build
//!   (Edge ≥ 105) and **does not mutate the DOM** — Mermaid diagrams,
//!   KaTeX math, and code highlighting stay intact.
//! * **Tier C — IPC count report.** After every search, the JS posts a
//!   `marco_find:count=N,index=K` IPC message that the host receives via
//!   [`PlatformWebView::set_find_report_callback`]. The host can then
//!   update its "K of N" indicator.
//! * **Tier A fallback** (`window.find`) is implemented in
//!   [`fallback_script`] for runtimes too old for `CSS.highlights`. The
//!   primary script auto-falls back to it if `typeof CSS.highlights ===
//!   "undefined"`.
//!
//! The Rust side is a thin wrapper that builds the JS payload (with safe
//! JSON-string escaping via `serde_json`) and dispatches it through
//! [`PlatformWebView::evaluate_script`]. All UI work lives in the search
//! window — this module only owns the "engine".
//!
//! # Usage
//!
//! ```ignore
//! use crate::components::viewer::wry_find;
//!
//! // One-time install (after the WebView has loaded the document):
//! wry_find::install(&platform_webview);
//!
//! // Search:
//! wry_find::search(&platform_webview, "needle", wry_find::FindOptions {
//!     case_sensitive: false,
//!     whole_word: false,
//! });
//!
//! // Step through matches:
//! wry_find::next(&platform_webview);
//! wry_find::prev(&platform_webview);
//!
//! // Clear highlights:
//! wry_find::clear(&platform_webview);
//! ```

#![cfg(target_os = "windows")]
// `install`, `search`, `next`, `prev`, `clear`, and `parse_report` are now
// driven through the `FindBackend` trait (Step 6b — see
// [`super::find_backend::WryFindBackend`]). `fallback_script` is still only
// invoked by the primary engine's runtime feature-detect path inside the
// injected JS, so keep its dead-code allow scoped to the function itself.

use crate::components::viewer::wry_platform_webview::PlatformWebView;

/// User-facing search options forwarded to the JS engine.
///
/// Mirrors the subset of `webkit6::FindOptions` that the marco search UI
/// actually exposes (case sensitivity and whole-word). Regex / Markdown-only
/// filtering happens in the editor buffer, not in the preview pane, and is
/// therefore out of scope for this backend.
#[derive(Debug, Clone, Copy, Default)]
pub struct FindOptions {
    /// Match case exactly. When `false`, both query and DOM text are
    /// lower-cased before comparison.
    pub case_sensitive: bool,
    /// Require word boundaries on both sides of every match.
    pub whole_word: bool,
}

/// Report posted by the JS engine after every `search`, `next`, or `prev`
/// call. Delivered to the host via [`PlatformWebView::set_find_report_callback`].
#[derive(Debug, Clone, Copy)]
pub struct FindReport {
    /// Total number of matches currently highlighted (0 when the document
    /// contains no match or after [`clear`]).
    pub total: usize,
    /// 1-based index of the "active" match (the one scrolled into view), or
    /// 0 when there is no active match.
    pub active: usize,
}

/// Install the `MarcoFind` JS engine into the live WebView2 document.
///
/// Idempotent — re-installing on an already-installed page just refreshes the
/// `<style id="marco-find-style">` block and re-runs the bootstrap. Safe to
/// call after every `load_html_with_base`.
pub fn install(webview: &PlatformWebView) {
    webview.evaluate_script(install_script());
}

/// Highlight all matches of `query` in the live document.
///
/// Posts `marco_find:count=N,index=K` back to the host once painting
/// completes. Passing an empty `query` is equivalent to [`clear`].
pub fn search(webview: &PlatformWebView, query: &str, opts: FindOptions) {
    let query_json = serde_json::to_string(query).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        "window.MarcoFind && window.MarcoFind.search({}, {{caseSensitive: {}, wholeWord: {}}});",
        query_json,
        if opts.case_sensitive { "true" } else { "false" },
        if opts.whole_word { "true" } else { "false" },
    );
    webview.evaluate_script(&script);
}

/// Advance to the next match and scroll it into view. No-op when no search
/// is active.
pub fn next(webview: &PlatformWebView) {
    webview.evaluate_script("window.MarcoFind && window.MarcoFind.next();");
}

/// Move to the previous match and scroll it into view. No-op when no search
/// is active.
pub fn prev(webview: &PlatformWebView) {
    webview.evaluate_script("window.MarcoFind && window.MarcoFind.prev();");
}

/// Remove all highlights and post `marco_find:count=0,index=0`.
pub fn clear(webview: &PlatformWebView) {
    webview.evaluate_script("window.MarcoFind && window.MarcoFind.clear();");
}

/// Parse a `marco_find:` IPC payload into a [`FindReport`].
///
/// Returns `None` if the payload does not match the documented format
/// `count=<N>,index=<K>`. Used by `wry_platform_webview::on_ipc_message`.
pub fn parse_report(payload: &str) -> Option<FindReport> {
    let mut total: Option<usize> = None;
    let mut active: Option<usize> = None;
    for part in payload.split(',') {
        let (key, value) = part.split_once('=')?;
        match key.trim() {
            "count" => total = value.trim().parse().ok(),
            "index" => active = value.trim().parse().ok(),
            _ => {}
        }
    }
    Some(FindReport {
        total: total?,
        active: active?,
    })
}

/// Top-level JavaScript bootstrap installed by [`install`].
///
/// Defines `window.MarcoFind` with `search`, `next`, `prev`, `clear`. Uses
/// the CSS Custom Highlight API when available; falls back to
/// [`fallback_script`]-equivalent `window.find()` selection-based search
/// otherwise.
fn install_script() -> &'static str {
    r#"
(function() {
    if (window.MarcoFind && window.MarcoFind.__installed) {
        return;
    }

    // Inject the highlight stylesheet exactly once.
    var styleId = 'marco-find-style';
    var styleEl = document.getElementById(styleId);
    if (!styleEl) {
        styleEl = document.createElement('style');
        styleEl.id = styleId;
        styleEl.textContent =
            "::highlight(marco-find) {" +
            "  background-color: rgba(255, 200, 0, 0.45);" +
            "  color: inherit;" +
            "}" +
            "::highlight(marco-find-active) {" +
            "  background-color: rgba(255, 140, 0, 0.85);" +
            "  color: black;" +
            "}";
        document.head.appendChild(styleEl);
    }

    var hasCssHighlights =
        typeof CSS !== 'undefined' &&
        CSS.highlights &&
        typeof Highlight === 'function';

    var state = {
        query: '',
        caseSensitive: false,
        wholeWord: false,
        ranges: [],
        index: -1,
    };

    function postReport() {
        try {
            var idx = (state.ranges.length === 0) ? 0 : (state.index + 1);
            var msg = 'marco_find:count=' + state.ranges.length + ',index=' + idx;
            if (window.ipc && window.ipc.postMessage) {
                window.ipc.postMessage(msg);
            }
        } catch (e) {
            console.error('[MarcoFind] postReport failed:', e);
        }
    }

    function clearHighlights() {
        if (hasCssHighlights) {
            try { CSS.highlights.delete('marco-find'); } catch (e) {}
            try { CSS.highlights.delete('marco-find-active'); } catch (e) {}
        } else {
            try { window.getSelection().removeAllRanges(); } catch (e) {}
        }
    }

    function collectTextNodes(root) {
        var walker = document.createTreeWalker(
            root,
            NodeFilter.SHOW_TEXT,
            {
                acceptNode: function(n) {
                    if (!n.nodeValue || n.nodeValue.length === 0) {
                        return NodeFilter.FILTER_REJECT;
                    }
                    // Skip script/style/non-rendered.
                    var p = n.parentNode;
                    while (p && p !== root) {
                        var tag = p.nodeName;
                        if (tag === 'SCRIPT' || tag === 'STYLE' || tag === 'NOSCRIPT') {
                            return NodeFilter.FILTER_REJECT;
                        }
                        p = p.parentNode;
                    }
                    return NodeFilter.FILTER_ACCEPT;
                },
            }
        );
        var nodes = [];
        var n;
        while ((n = walker.nextNode())) {
            nodes.push(n);
        }
        return nodes;
    }

    function findRanges(query) {
        var ranges = [];
        if (!query) return ranges;

        var nodes = collectTextNodes(document.body || document.documentElement);
        var needle = state.caseSensitive ? query : query.toLowerCase();
        var nlen = needle.length;

        for (var i = 0; i < nodes.length; i++) {
            var node = nodes[i];
            var hay = state.caseSensitive ? node.nodeValue : node.nodeValue.toLowerCase();
            var from = 0;
            while (true) {
                var pos = hay.indexOf(needle, from);
                if (pos < 0) break;

                if (state.wholeWord) {
                    var before = pos > 0 ? hay.charAt(pos - 1) : ' ';
                    var after = (pos + nlen) < hay.length ? hay.charAt(pos + nlen) : ' ';
                    var isWord = function(c) { return /[A-Za-z0-9_]/.test(c); };
                    if (isWord(before) || isWord(after)) {
                        from = pos + 1;
                        continue;
                    }
                }

                try {
                    var r = document.createRange();
                    r.setStart(node, pos);
                    r.setEnd(node, pos + nlen);
                    ranges.push(r);
                } catch (e) {
                    /* skip invalid range */
                }
                from = pos + nlen;
            }
        }
        return ranges;
    }

    function paintAll() {
        if (!hasCssHighlights) {
            // Tier A fallback handles painting through native selection.
            return;
        }
        try {
            if (state.ranges.length === 0) {
                CSS.highlights.delete('marco-find');
                CSS.highlights.delete('marco-find-active');
                return;
            }
            var all = state.ranges.slice();
            var active = (state.index >= 0 && state.index < state.ranges.length)
                ? [state.ranges[state.index]]
                : [];
            // Active match excluded from the "all" highlight so it can paint
            // with the active color on top of the dimmer overlay.
            if (active.length > 0) {
                all.splice(state.index, 1);
            }
            CSS.highlights.set('marco-find', new Highlight(...all));
            CSS.highlights.set('marco-find-active', new Highlight(...active));
        } catch (e) {
            console.error('[MarcoFind] paintAll failed:', e);
        }
    }

    function scrollActiveIntoView() {
        if (state.index < 0 || state.index >= state.ranges.length) return;
        try {
            var r = state.ranges[state.index];
            var el = r.startContainer.parentElement;
            if (el && el.scrollIntoView) {
                el.scrollIntoView({ block: 'center', inline: 'nearest', behavior: 'auto' });
            }
        } catch (e) {
            /* ignore */
        }
    }

    function fallbackFind(forward) {
        // Tier A: native window.find. Wraps the document automatically when
        // its 4th arg is true.
        try {
            var found = window.find(state.query, state.caseSensitive, !forward, true, state.wholeWord, false, false);
            // No reliable count without DOM walk — report 0/0 to signal
            // "we found something but cannot count".
            state.ranges = [];
            state.index = found ? 0 : -1;
            postReport();
        } catch (e) {
            console.error('[MarcoFind] fallbackFind failed:', e);
        }
    }

    window.MarcoFind = {
        __installed: true,

        search: function(query, opts) {
            opts = opts || {};
            state.query = query || '';
            state.caseSensitive = !!opts.caseSensitive;
            state.wholeWord = !!opts.wholeWord;

            if (!state.query) {
                state.ranges = [];
                state.index = -1;
                clearHighlights();
                postReport();
                return;
            }

            if (hasCssHighlights) {
                state.ranges = findRanges(state.query);
                state.index = state.ranges.length > 0 ? 0 : -1;
                paintAll();
                scrollActiveIntoView();
                postReport();
            } else {
                fallbackFind(true);
            }
        },

        next: function() {
            if (hasCssHighlights) {
                if (state.ranges.length === 0) { postReport(); return; }
                state.index = (state.index + 1) % state.ranges.length;
                paintAll();
                scrollActiveIntoView();
                postReport();
            } else {
                fallbackFind(true);
            }
        },

        prev: function() {
            if (hasCssHighlights) {
                if (state.ranges.length === 0) { postReport(); return; }
                state.index = (state.index - 1 + state.ranges.length) % state.ranges.length;
                paintAll();
                scrollActiveIntoView();
                postReport();
            } else {
                fallbackFind(false);
            }
        },

        clear: function() {
            state.query = '';
            state.ranges = [];
            state.index = -1;
            clearHighlights();
            postReport();
        },
    };
})();
"#
}

/// Inline Tier A snippet that calls `window.find` directly. Exposed for
/// callers that want to bypass the `MarcoFind` engine entirely (e.g. a
/// diagnostic fallback path or unit-style integration tests).
///
/// Returns a script that:
/// - Calls `window.find(query, caseSensitive, backwards, wrap, wholeWord)`.
/// - Posts `marco_find:count=0,index=0` regardless of outcome (no native
///   count is available).
#[allow(dead_code)] // Reserved for the runtime feature-detect path; not yet driven from Rust.
pub fn fallback_script(query: &str, opts: FindOptions, backwards: bool) -> String {
    let query_json = serde_json::to_string(query).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "(function(){{ \
            try {{ window.find({query}, {cs}, {bk}, true, {ww}, false, false); }} catch(e) {{}} \
            try {{ if (window.ipc && window.ipc.postMessage) {{ \
                window.ipc.postMessage('marco_find:count=0,index=0'); \
            }} }} catch(e) {{}} \
        }})();",
        query = query_json,
        cs = if opts.case_sensitive { "true" } else { "false" },
        bk = if backwards { "true" } else { "false" },
        ww = if opts.whole_word { "true" } else { "false" },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_parse_report_round_trip() {
        let r = parse_report("count=12,index=3").expect("parse");
        assert_eq!(r.total, 12);
        assert_eq!(r.active, 3);
    }

    #[test]
    fn smoke_test_parse_report_tolerates_whitespace() {
        let r = parse_report(" count = 5 , index = 2 ").expect("parse");
        assert_eq!(r.total, 5);
        assert_eq!(r.active, 2);
    }

    #[test]
    fn smoke_test_parse_report_rejects_missing_field() {
        assert!(parse_report("count=5").is_none());
        assert!(parse_report("index=2").is_none());
        assert!(parse_report("").is_none());
    }

    #[test]
    fn smoke_test_parse_report_zero_zero_means_cleared() {
        let r = parse_report("count=0,index=0").expect("parse");
        assert_eq!(r.total, 0);
        assert_eq!(r.active, 0);
    }

    #[test]
    fn smoke_test_fallback_script_escapes_query() {
        let s = fallback_script(
            "she said \"hi\"\n<x>",
            FindOptions {
                case_sensitive: true,
                whole_word: false,
            },
            false,
        );
        // serde_json escapes inner quotes and the newline, so the literal
        // sequence `\"hi\"` and `\n` must appear in the rendered script.
        assert!(s.contains(r#"she said \"hi\""#));
        assert!(s.contains(r"\n"));
        // Flags are forwarded verbatim.
        assert!(s.contains("true, false, true, false, false"));
    }

    #[test]
    fn smoke_test_install_script_is_idempotent_check() {
        // Hard-coded guard that the install script keeps the
        // `__installed` re-entry guard. Catches accidental removal.
        let s = install_script();
        assert!(s.contains("__installed"));
        assert!(s.contains("window.MarcoFind"));
        assert!(s.contains("CSS.highlights"));
        assert!(s.contains("marco_find:count="));
    }
}
