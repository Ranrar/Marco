//! In-page zoom toolbar for Polo's preview, plus the Ctrl+wheel zoom handler.
//!
//! Same reasoning as Marco's `ZOOM_BAR_HTML` (`marco/src/components/viewer/javascript.rs`):
//! a native GTK `Overlay` can't draw above an embedded WebView2 child window
//! on Windows, so the control has to live inside the page itself — and using
//! the same in-page control on Linux keeps behavior identical across
//! platforms. Buttons post `polo_zoom:in|out|reset` via IPC; Ctrl+wheel posts
//! the same messages directly from its own listener.
//!
//! Simpler than Marco's bar in two ways:
//! - No GTK-side hover-tracking hooks: Polo has a single content pane (no
//!   sibling editor pane competing for pointer focus), so a plain
//!   `mousemove`-show / timeout-hide works with no native motion-tracking
//!   plumbing.
//! - No `polo_zoom:ready` round-trip: Polo re-applies the persisted zoom
//!   directly from Rust's `connect_load_finished` callback (see
//!   `viewer::zoom::reapply`), so the page never needs to tell Rust it's
//!   ready.

/// CSS appended once into every rendered document's `<style>` block. Visually
/// mirrors Marco's zoom bar (same colors/spacing/hover feel), with
/// `polo-*`-prefixed ids/classes.
pub(crate) const CSS: &str = r#"
#polo-win-zoom{position:fixed;right:10px;bottom:10px;z-index:2147483647;
    display:flex;align-items:center;gap:0;padding:2px 4px;border-radius:10px;
    background:rgba(30,30,30,0.72);border:1px solid rgba(255,255,255,0.12);
    box-shadow:0 2px 8px rgba(0,0,0,0.30);
    font-family:system-ui,-apple-system,Segoe UI,sans-serif;
    opacity:0;pointer-events:none;transition:opacity 150ms ease;
    user-select:none;-webkit-user-select:none;}
#polo-win-zoom.polo-zoom-visible{opacity:1;pointer-events:auto;}
#polo-win-zoom.polo-zoom-light{background:rgba(245,245,245,0.88);
    border-color:rgba(0,0,0,0.14);box-shadow:0 2px 8px rgba(0,0,0,0.14);}
#polo-win-zoom button{background:transparent;border:0;border-radius:6px;
    min-width:30px;min-height:30px;padding:2px 6px;
    color:rgba(220,220,220,0.92);font-size:14px;font-weight:500;
    cursor:pointer;line-height:1;transition:background 120ms ease;}
#polo-win-zoom.polo-zoom-light button{color:rgba(40,40,40,0.90);}
#polo-win-zoom button:hover{background:rgba(255,255,255,0.18);}
#polo-win-zoom.polo-zoom-light button:hover{background:rgba(0,0,0,0.10);}
#polo-win-zoom button:active{background:rgba(255,255,255,0.28);}
#polo-win-zoom.polo-zoom-light button:active{background:rgba(0,0,0,0.18);}
#polo-win-zoom .polo-zoom-label{color:rgba(200,200,200,0.85);font-size:12px;
    min-width:36px;padding:0 2px;text-align:center;font-variant-numeric:tabular-nums;}
#polo-win-zoom.polo-zoom-light .polo-zoom-label{color:rgba(60,60,60,0.85);}
#polo-win-zoom .polo-zoom-sep{width:1px;align-self:stretch;margin:4px 2px;
    background:rgba(255,255,255,0.14);}
#polo-win-zoom.polo-zoom-light .polo-zoom-sep{background:rgba(0,0,0,0.12);}
"#;

/// Markup + driver script appended once into `<body>` on every rendered
/// document.
pub(crate) fn html() -> &'static str {
    r#"<div id="polo-win-zoom" aria-hidden="true">
    <button type="button" data-polo-zoom="in" title="Zoom in">+</button>
    <span class="polo-zoom-sep"></span>
    <button type="button" data-polo-zoom="out" title="Zoom out">&minus;</button>
    <span class="polo-zoom-sep"></span>
    <button type="button" data-polo-zoom="reset" title="Reset zoom">&#x2922;</button>
    <span class="polo-zoom-sep"></span>
    <span class="polo-zoom-label" id="polo-win-zoom-label">100%</span>
</div>
<script>
(function(){
    var bar = document.getElementById('polo-win-zoom');
    if (!bar) return;

    function applyThemeClass(){
        var isLight = document.documentElement.className.toLowerCase().indexOf('dark') === -1;
        bar.classList.toggle('polo-zoom-light', isLight);
    }
    applyThemeClass();

    // Relocate onto <html> (a sibling of <body>, not a descendant) so the
    // bar itself is structurally immune to zoom applied to <body>.
    function relocate(){
        if (bar.parentNode !== document.documentElement) {
            try { document.documentElement.appendChild(bar); } catch(e) {}
        }
    }
    relocate();

    // Reveal on movement anywhere over the page (the whole document *is*
    // the preview pane here), hide after a short idle period. Polo has no
    // sibling native pane to lose pointer focus to, so a plain timeout is
    // enough — no GTK-side motion tracking needed (unlike Marco's bar).
    var hideTimer = null;
    function show(){
        bar.classList.add('polo-zoom-visible');
        if (hideTimer) clearTimeout(hideTimer);
        hideTimer = setTimeout(function(){
            bar.classList.remove('polo-zoom-visible');
        }, 1500);
    }
    document.addEventListener('mousemove', show, {passive:true});

    function send(action){
        try {
            if (window.ipc && typeof window.ipc.postMessage === 'function') {
                window.ipc.postMessage('polo_zoom:' + action);
            }
        } catch (e) {}
    }
    bar.addEventListener('click', function(e){
        var btn = e.target && e.target.closest && e.target.closest('button[data-polo-zoom]');
        if (!btn) return;
        e.preventDefault();
        e.stopPropagation();
        send(btn.getAttribute('data-polo-zoom'));
    }, true);

    // Ctrl+wheel: one wheel notch = one zoom step, same as the buttons and
    // the Ctrl+=/Ctrl+-/Ctrl+0 keyboard shortcuts. Plain (non-Ctrl) wheel
    // scrolling is left completely alone.
    window.addEventListener('wheel', function(e){
        if (!e.ctrlKey) return;
        e.preventDefault();
        send(e.deltaY < 0 ? 'in' : 'out');
    }, {passive:false});

    // Apply a zoom level: scale <body> only, never <html>, so the
    // relocated bar (a sibling of <body>) can't be affected no matter how
    // the `zoom` property's cascade to descendants behaves.
    window.__poloApplyZoom = function(z){
        try {
            var n = parseFloat(z);
            if (!isFinite(n) || n <= 0) return;
            document.body.style.zoom = n;
            relocate();
            var lbl = document.getElementById('polo-win-zoom-label');
            if (lbl) lbl.textContent = Math.round(n * 100) + '%';
        } catch (e) {}
    };
    window.__poloSetZoomLabel = function(pct){
        var lbl = document.getElementById('polo-win-zoom-label');
        if (lbl) lbl.textContent = pct + '%';
    };

    applyThemeClass();
})();
</script>"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_html_contains_bar_and_buttons() {
        let markup = html();
        assert!(markup.contains("polo-win-zoom"));
        assert!(markup.contains("data-polo-zoom=\"in\""));
        assert!(markup.contains("data-polo-zoom=\"out\""));
        assert!(markup.contains("data-polo-zoom=\"reset\""));
        assert!(markup.contains("__poloApplyZoom"));
        assert!(markup.contains("polo_zoom:"));
    }

    #[test]
    fn smoke_test_css_covers_both_themes() {
        assert!(CSS.contains("#polo-win-zoom"));
        assert!(CSS.contains("polo-zoom-light"));
        assert!(CSS.contains("polo-zoom-visible"));
    }
}
