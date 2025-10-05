pub fn wheel_js(scale: f64) -> String {
    format!(
        r#"<script>
    (function(){{
        const scale = {scale};
        function findScroll(el){{
            while(el && el !== document){{
                if (el.scrollHeight > el.clientHeight) return el;
                el = el.parentNode;
            }}
            return document.scrollingElement || document.documentElement || document.body;
        }}
        window.addEventListener('wheel', function(e){{
            if (Math.abs(e.deltaY) < 0.0001) return;
            const sc = findScroll(e.target);
            sc.scrollBy({{ top: e.deltaY * scale, left: e.deltaX * scale, behavior: 'auto' }});
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
                document.title = 'marco_scroll:' + frac.toFixed(6);
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
