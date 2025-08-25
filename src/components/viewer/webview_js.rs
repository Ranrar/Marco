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
    function report(){
        try{
            var el = document.scrollingElement||document.documentElement||document.body;
            var denom = Math.max(el.scrollHeight - el.clientHeight, 1);
            var frac = Math.max(0, Math.min(1, el.scrollTop / denom));
            document.title = 'marco_scroll:' + frac.toFixed(6);
        }catch(e){}
    }
    window.addEventListener('scroll', report, {passive:true});
    setInterval(report, 500);
})();
</script>"#;
