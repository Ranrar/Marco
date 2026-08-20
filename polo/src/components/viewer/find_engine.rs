//! JavaScript-based "find in page" engine (`PoloFind`) for Polo's preview
//! toolbar search bar.
//!
//! Ported from Marco's `viewer::find_engine` (`MarcoFind`) — same rationale
//! applies here: neither WebKitGTK nor WebView2 (via wry) exposes a find API
//! rich enough to drive a real search UI (count matches, highlight-all vs.
//! active-only, case/word/diacritic options). `window.find()` is the only
//! native primitive and it's a boolean, selection-based, uncountable
//! fallback — implemented here as [`fallback_script`] for engines without
//! the CSS Custom Highlight API.
//!
//! Renamed from Marco's copy (`MarcoFind` / `marco_find:`) to `PoloFind` /
//! `polo_find:` purely so Polo's own preview DOM and IPC protocol don't leak
//! the sibling app's name — the implementation strategy is identical.
//!
//! * **Tier B — CSS Custom Highlight API.** Matches are computed in JS via
//!   `document.createTreeWalker(...)` and registered with
//!   `CSS.highlights.set("polo-find", new Highlight(...ranges))`, painted by
//!   an injected `::highlight(polo-find)` CSS rule. Does not mutate the DOM.
//! * **Tier C — IPC count report.** After every search, JS posts a
//!   `polo_find:count=N,index=K` IPC message, received via
//!   [`PlatformWebView::set_find_report_callback`].
//! * **Tier A fallback** (`window.find`) in [`fallback_script`] for runtimes
//!   too old for `CSS.highlights`.
//!
//! # Usage
//!
//! ```ignore
//! use crate::components::viewer::find_engine;
//!
//! find_engine::install(&platform_webview);
//! find_engine::search(&platform_webview, "needle", find_engine::FindOptions {
//!     case_sensitive: false,
//!     whole_word: false,
//!     highlight_all: true,
//!     match_diacritics: false,
//! });
//! find_engine::next(&platform_webview);
//! find_engine::prev(&platform_webview);
//! find_engine::clear(&platform_webview);
//! ```

use crate::components::viewer::platform_webview::PlatformWebView;

/// User-facing search options forwarded to the JS engine.
#[derive(Debug, Clone, Copy)]
pub struct FindOptions {
    /// Match case exactly. When `false`, both query and DOM text are
    /// lower-cased before comparison.
    pub case_sensitive: bool,
    /// Require word boundaries on both sides of every match.
    pub whole_word: bool,
    /// Paint every match (dimmed) in addition to the active one. When
    /// `false`, only the active match is highlighted — mirrors Firefox's
    /// "Highlight All" find-bar checkbox.
    pub highlight_all: bool,
    /// Treat accented characters as distinct from their base letter. When
    /// `false` (Firefox's default), "cafe" and "café" match each other.
    pub match_diacritics: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            highlight_all: true,
            match_diacritics: false,
        }
    }
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

/// Install the `PoloFind` JS engine into the live document.
///
/// Idempotent — re-installing on an already-installed page just refreshes the
/// `<style id="polo-find-style">` block and re-runs the bootstrap. Safe to
/// call after every `load_html_with_base`.
pub fn install(webview: &PlatformWebView) {
    webview.evaluate_script(install_script());
}

/// Highlight all matches of `query` in the live document.
///
/// Posts `polo_find:count=N,index=K` back to the host once painting
/// completes. Passing an empty `query` is equivalent to [`clear`].
pub fn search(webview: &PlatformWebView, query: &str, opts: FindOptions) {
    let query_json = serde_json::to_string(query).unwrap_or_else(|_| "\"\"".to_string());
    let script = format!(
        "window.PoloFind && window.PoloFind.search({}, {{caseSensitive: {}, wholeWord: {}, highlightAll: {}, matchDiacritics: {}}});",
        query_json,
        js_bool(opts.case_sensitive),
        js_bool(opts.whole_word),
        js_bool(opts.highlight_all),
        js_bool(opts.match_diacritics),
    );
    webview.evaluate_script(&script);
}

fn js_bool(b: bool) -> &'static str {
    if b {
        "true"
    } else {
        "false"
    }
}

/// Advance to the next match and scroll it into view. No-op when no search
/// is active.
pub fn next(webview: &PlatformWebView) {
    webview.evaluate_script("window.PoloFind && window.PoloFind.next();");
}

/// Move to the previous match and scroll it into view. No-op when no search
/// is active.
pub fn prev(webview: &PlatformWebView) {
    webview.evaluate_script("window.PoloFind && window.PoloFind.prev();");
}

/// Remove all highlights and post `polo_find:count=0,index=0`.
pub fn clear(webview: &PlatformWebView) {
    webview.evaluate_script("window.PoloFind && window.PoloFind.clear();");
}

/// Parse a `polo_find:` IPC payload into a [`FindReport`].
///
/// Returns `None` if the payload does not match the documented format
/// `count=<N>,index=<K>`. Used by `platform_webview::on_ipc_message`.
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
/// Defines `window.PoloFind` with `search`, `next`, `prev`, `clear`. Uses
/// the CSS Custom Highlight API when available; falls back to
/// `window.find()`-based search otherwise.
fn install_script() -> &'static str {
    r#"
(function() {
    if (window.PoloFind && window.PoloFind.__installed) {
        return;
    }

    // Inject the highlight stylesheet exactly once.
    var styleId = 'polo-find-style';
    var styleEl = document.getElementById(styleId);
    if (!styleEl) {
        styleEl = document.createElement('style');
        styleEl.id = styleId;
        styleEl.textContent =
            "::highlight(polo-find) {" +
            "  background-color: rgba(255, 200, 0, 0.45);" +
            "  color: inherit;" +
            "}" +
            "::highlight(polo-find-active) {" +
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
        highlightAll: true,
        matchDiacritics: false,
        ranges: [],
        index: -1,
    };

    function postReport() {
        try {
            var idx = (state.ranges.length === 0) ? 0 : (state.index + 1);
            var msg = 'polo_find:count=' + state.ranges.length + ',index=' + idx;
            if (window.ipc && window.ipc.postMessage) {
                window.ipc.postMessage(msg);
            }
        } catch (e) {
            console.error('[PoloFind] postReport failed:', e);
        }
    }

    function forceRepaint() {
        // paintAll() is always followed by scrollActiveIntoView(), whose
        // scroll incidentally forces the compositor to flush the
        // ::highlight() overlay layer. clear() has no such follow-up scroll,
        // so on WebKitGTK (and some WebView2 builds) a highlight removal
        // isn't actually painted until some unrelated scroll/resize happens.
        // Nudging opacity forces a synchronous repaint with no visible or
        // scroll-position change.
        var el = document.body || document.documentElement;
        if (!el) return;
        try {
            var prev = el.style.opacity;
            el.style.opacity = '0.999999';
            void el.offsetHeight; // force synchronous layout/paint flush
            el.style.opacity = prev;
        } catch (e) {}
    }

    function clearHighlights() {
        if (hasCssHighlights) {
            try { CSS.highlights.delete('polo-find'); } catch (e) {}
            try { CSS.highlights.delete('polo-find-active'); } catch (e) {}
            forceRepaint();
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

    // Strips combining diacritical marks character-by-character, keeping a
    // parallel index map so match offsets found in the folded string can be
    // mapped back to offsets in the original node text. Needed because
    // NFD decomposition can change string length (e.g. a single precomposed
    // 'é' becomes 'e' + a combining accent), so folded and original offsets
    // aren't interchangeable without this map.
    function foldDiacritics(str) {
        var text = '';
        var map = [];
        for (var i = 0; i < str.length; i++) {
            var base = str[i].normalize('NFD').replace(/[\u0300-\u036f]/g, '');
            for (var j = 0; j < base.length; j++) {
                text += base[j];
                map.push(i);
            }
        }
        return { text: text, map: map };
    }

    function findRanges(query) {
        var ranges = [];
        if (!query) return ranges;

        var nodes = collectTextNodes(document.body || document.documentElement);
        var needle = state.matchDiacritics ? query : foldDiacritics(query).text;
        needle = state.caseSensitive ? needle : needle.toLowerCase();
        var nlen = needle.length;
        if (nlen === 0) return ranges;

        for (var i = 0; i < nodes.length; i++) {
            var node = nodes[i];
            var raw = node.nodeValue;
            var hay, map;
            if (state.matchDiacritics) {
                hay = state.caseSensitive ? raw : raw.toLowerCase();
                map = null; // identity mapping
            } else {
                var folded = foldDiacritics(raw);
                hay = state.caseSensitive ? folded.text : folded.text.toLowerCase();
                map = folded.map;
            }

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

                var startOrig = map ? map[pos] : pos;
                var endOrig = map ? (map[pos + nlen - 1] + 1) : (pos + nlen);

                try {
                    var r = document.createRange();
                    r.setStart(node, startOrig);
                    r.setEnd(node, endOrig);
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
            return;
        }
        try {
            if (state.ranges.length === 0) {
                CSS.highlights.delete('polo-find');
                CSS.highlights.delete('polo-find-active');
                return;
            }
            var active = (state.index >= 0 && state.index < state.ranges.length)
                ? [state.ranges[state.index]]
                : [];

            if (state.highlightAll) {
                var all = state.ranges.slice();
                if (active.length > 0) {
                    all.splice(state.index, 1);
                }
                CSS.highlights.set('polo-find', new Highlight(...all));
            } else {
                CSS.highlights.delete('polo-find');
            }
            CSS.highlights.set('polo-find-active', new Highlight(...active));
        } catch (e) {
            console.error('[PoloFind] paintAll failed:', e);
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
        try {
            var found = window.find(state.query, state.caseSensitive, !forward, true, state.wholeWord, false, false);
            state.ranges = [];
            state.index = found ? 0 : -1;
            postReport();
        } catch (e) {
            console.error('[PoloFind] fallbackFind failed:', e);
        }
    }

    window.PoloFind = {
        __installed: true,

        search: function(query, opts) {
            opts = opts || {};
            state.query = query || '';
            state.caseSensitive = !!opts.caseSensitive;
            state.wholeWord = !!opts.wholeWord;
            state.highlightAll = opts.highlightAll !== false;
            state.matchDiacritics = !!opts.matchDiacritics;

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
/// callers that want to bypass the `PoloFind` engine entirely.
///
/// Returns a script that:
/// - Calls `window.find(query, caseSensitive, backwards, wrap, wholeWord)`.
/// - Posts `polo_find:count=0,index=0` regardless of outcome (no native
///   count is available).
#[allow(dead_code)] // Reserved for the runtime feature-detect path; not yet driven from Rust.
pub fn fallback_script(query: &str, opts: FindOptions, backwards: bool) -> String {
    let query_json = serde_json::to_string(query).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "(function(){{ \
            try {{ window.find({query}, {cs}, {bk}, true, {ww}, false, false); }} catch(e) {{}} \
            try {{ if (window.ipc && window.ipc.postMessage) {{ \
                window.ipc.postMessage('polo_find:count=0,index=0'); \
            }} }} catch(e) {{}} \
        }})();",
        query = query_json,
        cs = js_bool(opts.case_sensitive),
        bk = js_bool(backwards),
        ww = js_bool(opts.whole_word),
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
                highlight_all: true,
                match_diacritics: false,
            },
            false,
        );
        assert!(s.contains(r#"she said \"hi\""#));
        assert!(s.contains(r"\n"));
        assert!(s.contains("true, false, true, false, false"));
    }

    #[test]
    fn smoke_test_install_script_is_idempotent_check() {
        let s = install_script();
        assert!(s.contains("__installed"));
        assert!(s.contains("window.PoloFind"));
        assert!(s.contains("CSS.highlights"));
        assert!(s.contains("polo_find:count="));
    }
}
