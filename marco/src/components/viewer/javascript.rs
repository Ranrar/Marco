pub fn wheel_js(scale: f64) -> String {
    format!(
        r#"<script>
    (function(){{
        const scale = {scale};

        function isElement(node){{
            return node && node.nodeType === 1;
        }}

        function getOverflowStyle(el){{
            try {{
                return window.getComputedStyle(el);
            }} catch (_) {{
                return null;
            }}
        }}

        function isScrollable(el){{
            if (!el) return false;
            const style = getOverflowStyle(el);
            if (!style) return false;

            const overflowY = (style.overflowY || '').toLowerCase();
            const overflowX = (style.overflowX || '').toLowerCase();

            const canScrollY = (overflowY === 'auto' || overflowY === 'scroll' || overflowY === 'overlay')
                && (el.scrollHeight - el.clientHeight) > 1;
            const canScrollX = (overflowX === 'auto' || overflowX === 'scroll' || overflowX === 'overlay')
                && (el.scrollWidth - el.clientWidth) > 1;

            return canScrollY || canScrollX;
        }}

        function findScroll(target){{
            let el = target;

            // Wheel targets can be text nodes; normalize to an element.
            while (el && !isElement(el)) el = el.parentNode;

            while (el && el !== document.body && el !== document.documentElement){{
                // Only treat *explicitly scrollable* elements as scroll containers.
                // This avoids false positives on headings where scrollHeight/clientHeight
                // can differ slightly due to rounding.
                if (isScrollable(el)) return el;
                el = el.parentNode;
            }}

            return document.scrollingElement || document.documentElement || document.body;
        }}

        window.addEventListener('wheel', function(e){{
            if (e.ctrlKey) {{
                e.preventDefault();
                try {{
                    if (window.ipc && typeof window.ipc.postMessage === 'function') {{
                        window.ipc.postMessage('marco_zoom:' + (e.deltaY < 0 ? 'in' : 'out'));
                    }}
                }} catch (_) {{}}
                return;
            }}

            if (Math.abs(e.deltaY) < 0.0001 && Math.abs(e.deltaX) < 0.0001) return;

            const sc = findScroll(e.target);

            // For the document scroller, prefer window.scrollBy().
            if (sc === document.body || sc === document.documentElement || sc === document.scrollingElement) {{
                window.scrollBy({{ top: e.deltaY * scale, left: e.deltaX * scale, behavior: 'auto' }});
            }} else {{
                sc.scrollBy({{ top: e.deltaY * scale, left: e.deltaX * scale, behavior: 'auto' }});
            }}

            e.preventDefault();
        }}, {{ passive: false }});
    }})();
    </script>"#,
        scale = scale
    )
}

pub const SCROLL_REPORT_JS: &str = r#"<script>
(function(){
    let lastReportedPosition = -1;
    let animationFrameId = null;
    let isScrolling = false;
    let scrollTimeout = null;
    
    function reportPosition(){
        // In paged.js mode __pagedJsReady is set to false until layout is complete.
        // Suppress reports while the DOM is being restructured to avoid spurious
        // scroll:0 messages that would yank the editor to the top.
        // In normal mode __pagedJsReady is never defined, so typeof returns
        // 'undefined' and the guard is a no-op.
        if (typeof window.__pagedJsReady !== 'undefined' && !window.__pagedJsReady) return;
        try{
            var el = document.scrollingElement||document.documentElement||document.body;
            var denom = Math.max(el.scrollHeight - el.clientHeight, 1);
            var frac = Math.max(0, Math.min(1, el.scrollTop / denom));

            // After paged.js reload the page starts at position 0.  Silently
            // initialise the baseline so the fresh-load 0 is never sent as a
            // scroll report (which would yank the editor to the top).
            if (window.__pagedJsJustReady) { lastReportedPosition = frac; return; }

            // Only report if position has changed significantly (avoid noise)
            if (Math.abs(frac - lastReportedPosition) > 0.0001) {
                var msg = 'marco_scroll:' + frac.toFixed(6);

                // Prefer IPC (wry/WebView2), fall back to title (WebKit).
                try {
                    if (window.ipc && typeof window.ipc.postMessage === 'function') {
                        window.ipc.postMessage(msg);
                    } else {
                        document.title = msg;
                    }
                } catch (e) {
                    document.title = msg;
                }
                lastReportedPosition = frac;
            }
        }catch(e){}
    }
    
    function scheduleReport(){
        if (animationFrameId === null) {
            animationFrameId = requestAnimationFrame(() => {
                reportPosition();
                animationFrameId = null;
            });
        }
    }
    
    // Optimized scroll event handling
    window.addEventListener('scroll', () => {
        if (!isScrolling) {
            isScrolling = true;
            scheduleReport();
        }
        
        // Clear existing timeout and set new one
        if (scrollTimeout) {
            clearTimeout(scrollTimeout);
        }
        
        // Mark scrolling as finished after 150ms of inactivity
        scrollTimeout = setTimeout(() => {
            isScrolling = false;
            reportPosition(); // Final position report
        }, 150);
        
        scheduleReport();
    }, {passive: true});
    
    // Reduced polling frequency - only when not actively scrolling
    setInterval(() => {
        if (!isScrolling) {
            reportPosition();
        }
    }, 1000); // Reduced from 500ms to 1000ms
    
    // Initial position report
    reportPosition();
})();
</script>"#;

/// JS that saves `window.scrollY` to `sessionStorage` before a full page
/// reload and restores it once the new page has laid out.
///
/// Injected once into every page body (via `wheel_js_rc` in `ui.rs`).
/// The save half is called from Rust just before `load_html_when_ready`
/// fires (improvement #2); the restore half runs automatically on each
/// page load.
pub const SCROLL_RESTORE_JS: &str = r#"<script>
(function(){
    try {
        var s = sessionStorage.getItem('marco-scroll');
        if (s !== null) {
            sessionStorage.removeItem('marco-scroll');
            var y = parseInt(s, 10);
            if (!isNaN(y) && y > 0) {
                if (document.readyState === 'loading') {
                    window.addEventListener('load', function() {
                        window.scrollTo(0, y);
                    }, { once: true });
                } else {
                    window.scrollTo(0, y);
                }
            }
        }
    } catch(e) {}
})();
</script>"#;

/// JS that reports hovered link URLs back to the host via `window.ipc.postMessage`.
///
/// Only meaningful on Windows where the wry/WebView2 backend lacks a native
/// hit-test signal (Linux uses `webkit6::WebView::connect_mouse_target_changed`).
/// Posts `marco_hover:<url>` when the cursor enters an `<a>` element with an
/// href, and `marco_hover:` (empty payload) when it leaves.
pub const HOVER_REPORT_JS: &str = r#"<script>
(function(){
    var current = null;
    function send(url){
        try {
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage('marco_hover:' + (url || ''));
            }
        } catch (e) {}
    }
    document.addEventListener('mouseover', function(e){
        var t = e.target;
        var a = (t && t.closest) ? t.closest('a[href]') : null;
        if (a) {
            var href = a.getAttribute('href') || a.href || '';
            if (href && href !== current) {
                current = href;
                send(href);
            }
        }
    }, true);
    document.addEventListener('mouseout', function(e){
        var t = e.target;
        var a = (t && t.closest) ? t.closest('a[href]') : null;
        if (a && current) {
            current = null;
            send('');
        }
    }, true);
})();
</script>"#;

/// HTML + JS overlay that renders the zoom toolbar in the bottom-right corner
/// of the preview page, on both platforms. This is the single zoom control —
/// there is no native GTK overlay counterpart (see `documentation/` history:
/// a GTK `Overlay` cannot draw above an embedded WebView2 child window on
/// Windows, so the control has to live inside the page itself; using the same
/// in-page control on Linux avoids running two independent zoom UIs at once).
/// Buttons post `marco_zoom:in|out|reset` via IPC.
///
/// Visually mirrors the GTK `zoom-bar` CSS classes (`marco/src/ui/css/zoom_bar.rs`,
/// now removed) — same colors, spacing, and hover-reveal feel — via
/// `.marco-zoom-light`/`.marco-zoom-visible` classes toggled in JS below.
/// Reveal-on-hover is shown from in-page `mousemove` but *hidden* from
/// native GTK motion tracking (`editor/ui.rs`) via the exposed
/// `window.__marcoZoomBarShow`/`__marcoZoomBarHide` hooks — see the comment
/// on those hooks below for why.
///
/// The toolbar is relocated out of `<body>` and onto `documentElement` on
/// load so paged.js (used in page/print preview mode) cannot hide it when it
/// re-parents body content into its own pagination containers. This
/// relocation is also why the toolbar never scales with content zoom: the
/// host applies zoom to `<body>` only (see `window.__marcoApplyZoom` below),
/// and the toolbar — a *sibling* of `<body>` under `<html>`, not a
/// descendant — is structurally outside that subtree, so it can't inherit
/// the zoom no matter how the `zoom` property cascades. No counter-scaling
/// CSS trick needed on the toolbar itself.
pub const ZOOM_BAR_HTML: &str = r#"<style>
#marco-win-zoom{position:fixed;right:10px;bottom:10px;z-index:2147483647;
    display:flex;align-items:center;gap:0;padding:2px 4px;border-radius:10px;
    background:rgba(30,30,30,0.72);border:1px solid rgba(255,255,255,0.12);
    box-shadow:0 2px 8px rgba(0,0,0,0.30);
    font-family:system-ui,-apple-system,Segoe UI,sans-serif;
    opacity:0;pointer-events:none;transition:opacity 150ms ease;
    user-select:none;-webkit-user-select:none;}
#marco-win-zoom.marco-zoom-visible{opacity:1;pointer-events:auto;}
#marco-win-zoom.marco-zoom-light{background:rgba(245,245,245,0.88);
    border-color:rgba(0,0,0,0.14);box-shadow:0 2px 8px rgba(0,0,0,0.14);}
#marco-win-zoom button{background:transparent;border:0;border-radius:6px;
    min-width:30px;min-height:30px;padding:2px 6px;
    color:rgba(220,220,220,0.92);font-size:14px;font-weight:500;
    cursor:pointer;line-height:1;transition:background 120ms ease;}
#marco-win-zoom.marco-zoom-light button{color:rgba(40,40,40,0.90);}
#marco-win-zoom button:hover{background:rgba(255,255,255,0.18);}
#marco-win-zoom.marco-zoom-light button:hover{background:rgba(0,0,0,0.10);}
#marco-win-zoom button:active{background:rgba(255,255,255,0.28);}
#marco-win-zoom.marco-zoom-light button:active{background:rgba(0,0,0,0.18);}
#marco-win-zoom .marco-zoom-label{color:rgba(200,200,200,0.85);font-size:12px;
    min-width:36px;padding:0 2px;text-align:center;font-variant-numeric:tabular-nums;}
#marco-win-zoom.marco-zoom-light .marco-zoom-label{color:rgba(60,60,60,0.85);}
#marco-win-zoom .marco-zoom-sep{width:1px;align-self:stretch;margin:4px 2px;
    background:rgba(255,255,255,0.14);}
#marco-win-zoom.marco-zoom-light .marco-zoom-sep{background:rgba(0,0,0,0.12);}
</style>
<div id="marco-win-zoom" aria-hidden="true">
    <button type="button" data-marco-zoom="in" title="Zoom in">+</button>
    <span class="marco-zoom-sep"></span>
    <button type="button" data-marco-zoom="out" title="Zoom out">&minus;</button>
    <span class="marco-zoom-sep"></span>
    <button type="button" data-marco-zoom="reset" title="Reset zoom">&#x2922;</button>
    <span class="marco-zoom-sep"></span>
    <span class="marco-zoom-label" id="marco-win-zoom-label">100%</span>
</div>
<script>
(function(){
    var bar = document.getElementById('marco-win-zoom');
    if (!bar) return;
    // Match the page's dark/light mode — same "contains dark" heuristic the
    // Rust side uses (`theme_mode.contains("dark")` / `eq_ignore_ascii_case`).
    function applyThemeClass(){
        var isLight = document.documentElement.className.toLowerCase().indexOf('dark') === -1;
        bar.classList.toggle('marco-zoom-light', isLight);
    }
    applyThemeClass();
    // Move the toolbar out of <body> so paged.js (which re-parents body
    // content into its own pagination containers) cannot hide it.
    function relocate(){
        if (bar.parentNode !== document.documentElement) {
            try { document.documentElement.appendChild(bar); } catch(e) {}
        }
    }
    relocate();
    // Reveal on hover: hidden until the pointer moves anywhere over the
    // page — the whole document *is* the preview pane here.
    //
    // Showing on in-page `mousemove` works fine. Hiding does not: DOM-level
    // `mouseleave`/`mouseout` do not reliably fire when the pointer moves
    // onto a *sibling native widget* (the editor pane) rather than crossing
    // a normal browser-window boundary — the embedded webview surface
    // (WebKitGTK / WebView2 HWND) handles its own input once the pointer is
    // over a different native widget, without a clean DOM "leave" event.
    // So hiding is driven from GTK's own motion tracking instead (the same
    // technique the removed `marco/src/ui/zoom_overlay.rs` used for its
    // native bar's hover-reveal), which calls these via `evaluate_script`.
    function show(){ bar.classList.add('marco-zoom-visible'); }
    function hide(){ bar.classList.remove('marco-zoom-visible'); }
    window.__marcoZoomBarShow = show;
    window.__marcoZoomBarHide = hide;
    document.addEventListener('mousemove', show, {passive:true});
    function send(action){
        try {
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage('marco_zoom:' + action);
            }
        } catch (e) {}
    }
    bar.addEventListener('click', function(e){
        var btn = e.target && e.target.closest && e.target.closest('button[data-marco-zoom]');
        if (!btn) return;
        e.preventDefault();
        e.stopPropagation();
        send(btn.getAttribute('data-marco-zoom'));
    }, true);
    // Apply a zoom level: scale <body> only, never <html>. The toolbar is
    // relocated onto documentElement (a *sibling* of <body>, not a
    // descendant) precisely so it structurally cannot be affected by zoom
    // applied here, regardless of how the `zoom` property's cascade to
    // descendants behaves — no CSS trick on the toolbar needs to fight it.
    window.__marcoApplyZoom = function(z){
        try {
            var n = parseFloat(z);
            if (!isFinite(n) || n <= 0) return;
            document.body.style.zoom = n;
            relocate();
            var lbl = document.getElementById('marco-win-zoom-label');
            if (lbl) lbl.textContent = Math.round(n * 100) + '%';
        } catch (e) {}
    };
    // Back-compat helper retained for any callers that just want the label.
    window.__marcoSetZoomLabel = function(pct){
        var lbl = document.getElementById('marco-win-zoom-label');
        if (lbl) lbl.textContent = pct + '%';
    };
    // Notify the host so it can re-apply the persisted zoom each time the
    // document is (re)loaded — `style.zoom` is reset on every navigation.
    function notifyReady(){ relocate(); applyThemeClass(); send('ready'); }
    if (document.readyState === 'complete' || document.readyState === 'interactive') {
        setTimeout(notifyReady, 0);
    } else {
        document.addEventListener('DOMContentLoaded', notifyReady);
    }
    // In paged.js (print preview) mode the layout is rebuilt asynchronously.
    // Re-apply the zoom once paged.js signals it has finished.
    var pollStart = Date.now();
    var poll = setInterval(function(){
        if (window.__pagedJsReady === true) {
            clearInterval(poll);
            notifyReady();
        } else if (Date.now() - pollStart > 15000) {
            clearInterval(poll);
        }
    }, 150);
})();
</script>"#;
