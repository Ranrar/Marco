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

/// JS that reports hovered link URLs back to the host via `window.ipc.postMessage`.
///
/// Only meaningful on Windows where the wry/WebView2 backend lacks a native
/// hit-test signal (Linux uses `webkit6::WebView::connect_mouse_target_changed`).
/// Posts `marco_hover:<url>` when the cursor enters an `<a>` element with an
/// href, and `marco_hover:` (empty payload) when it leaves.
#[cfg(target_os = "windows")]
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

/// HTML + JS overlay that renders a small zoom toolbar in the bottom-right
/// corner of the preview page on Windows. Required because the wry/WebView2
/// child window draws *over* the GTK overlay used for the Linux zoom bar,
/// hiding it. Buttons post `marco_zoom:in|out|reset` via IPC.
///
/// The toolbar is relocated out of `<body>` and onto `documentElement` on
/// load so paged.js (used in page/print preview mode) cannot hide it when it
/// re-parents body content into its own pagination containers.
///
/// Because the host applies zoom by setting `documentElement.style.zoom`, the
/// toolbar would normally scale together with the content. We counter-scale
/// it via `transform: scale(1 / zoom)` from `window.__marcoApplyZoom` so the
/// buttons remain a constant visual size at any zoom level.
#[cfg(target_os = "windows")]
pub const WIN_ZOOM_BAR_HTML: &str = r#"<style>
#marco-win-zoom{position:fixed;right:14px;bottom:14px;z-index:2147483647;
    display:flex;gap:4px;padding:4px 6px;border-radius:8px;
    background:rgba(40,40,40,0.78);box-shadow:0 2px 6px rgba(0,0,0,0.3);
    font-family:system-ui,-apple-system,Segoe UI,sans-serif;font-size:13px;
    color:#fff;opacity:0.15;transition:opacity 120ms ease;
    user-select:none;-webkit-user-select:none;
    transform-origin:bottom right;}
#marco-win-zoom:hover{opacity:1;}
#marco-win-zoom button{background:transparent;border:0;color:inherit;
    cursor:pointer;padding:2px 8px;border-radius:4px;font:inherit;line-height:1;}
#marco-win-zoom button:hover{background:rgba(255,255,255,0.18);}
#marco-win-zoom .marco-zoom-label{padding:2px 6px;min-width:42px;text-align:center;
    font-variant-numeric:tabular-nums;}
</style>
<div id="marco-win-zoom" aria-hidden="true">
    <button type="button" data-marco-zoom="out" title="Zoom out">&minus;</button>
    <span class="marco-zoom-label" id="marco-win-zoom-label">100%</span>
    <button type="button" data-marco-zoom="reset" title="Reset zoom">&#x2922;</button>
    <button type="button" data-marco-zoom="in" title="Zoom in">+</button>
</div>
<script>
(function(){
    var bar = document.getElementById('marco-win-zoom');
    if (!bar) return;
    // Move the toolbar out of <body> so paged.js (which re-parents body
    // content into its own pagination containers) cannot hide it.
    function relocate(){
        if (bar.parentNode !== document.documentElement) {
            try { document.documentElement.appendChild(bar); } catch(e) {}
        }
    }
    relocate();
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
    // Apply a zoom level: scale the document, counter-scale the toolbar so
    // its visual size stays constant, and update the percent label.
    window.__marcoApplyZoom = function(z){
        try {
            var n = parseFloat(z);
            if (!isFinite(n) || n <= 0) return;
            document.documentElement.style.zoom = n;
            relocate();
            bar.style.transform = 'scale(' + (1 / n) + ')';
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
    function notifyReady(){ relocate(); send('ready'); }
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
