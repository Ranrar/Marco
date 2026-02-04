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
        try{
            var el = document.scrollingElement||document.documentElement||document.body;
            var denom = Math.max(el.scrollHeight - el.clientHeight, 1);
            var frac = Math.max(0, Math.min(1, el.scrollTop / denom));
            
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
