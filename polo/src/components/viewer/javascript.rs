//! JS snippets injected into rendered preview documents.

/// JS that shows/positions/hides the in-page link-hover tooltip
/// (`viewer::link_hover`) directly — no IPC round-trip to Rust. Mirrors
/// Marco's `HOVER_REPORT_JS` (`marco/src/components/viewer/javascript.rs`)
/// for the title-suppression and event-listener structure, but Polo has no
/// footer bar to report *to*, so the tooltip lives entirely client-side:
/// this script owns showing, positioning (`clientX`/`clientY`, so the cursor
/// and the tooltip share the same viewport coordinate space — no
/// GTK-widget-relative translation needed), and hiding it.
///
/// An earlier version posted `polo_hover:` IPC messages to a native
/// `gtk4::Popover` on the Rust side. That popover's own show/hide/reposition
/// calls crashed the app on Windows at hover frequency (its native surface
/// collided with the WebView2 child HWND) — see `viewer::link_hover`'s module
/// doc for the isolation history. Doing everything here, in the same
/// document WebView2 is already rendering, needs no second native window at
/// all.
///
/// Before wiring any listeners, every `a[title]` in the document has its
/// `title` attribute moved to `data-polo-title` and removed. A `title`
/// attribute is what makes WebKit/WebView2 show their own native
/// hover tooltip after ~1s — left in place, the browser's tooltip and
/// Polo's own tooltip would both show the same text, stacked on top of each
/// other. Moving it (rather than just reading it before ignoring it) is what
/// actually suppresses the native one; the in-page tooltip is meant to be
/// the *only* title readout.
///
/// Updates are throttled to one DOM/style write per animation frame
/// (`requestAnimationFrame`) rather than one per `mousemove` event, since
/// `mousemove` can fire far faster than the tooltip needs to reposition.
pub const HOVER_REPORT_JS: &str = r#"<script>
(function(){
    // Suppress the native title-attribute tooltip on links: move `title` to
    // `data-polo-title` so WebKit/WebView2 have nothing left to show, and
    // read it back from there below instead. This script runs at the end of
    // <body>, after every link in the document already exists, so a single
    // upfront pass here covers the whole page (no need to wait for
    // DOMContentLoaded or re-run per element).
    var links = document.querySelectorAll('a[title]');
    for (var i = 0; i < links.length; i++) {
        var el = links[i];
        el.setAttribute('data-polo-title', el.getAttribute('title'));
        el.removeAttribute('title');
    }

    var tooltip = document.getElementById('polo-link-hover-tooltip');
    var titleEl = document.getElementById('polo-link-hover-title');
    var urlEl = document.getElementById('polo-link-hover-url');
    if (!tooltip || !titleEl || !urlEl) return;

    var GAP = 22; // px below the cursor, clear of the pointer tip.
    var currentHref = null; // currently-hovered href, or null
    var currentTitle = '';  // currently-hovered link's `title` attribute
    var pendingX = 0, pendingY = 0;
    var rafScheduled = false;

    function apply(){
        rafScheduled = false;
        if (!currentHref) {
            tooltip.classList.remove('visible');
            return;
        }
        urlEl.textContent = currentHref;
        if (currentTitle) {
            titleEl.textContent = currentTitle;
            titleEl.style.display = '';
        } else {
            titleEl.style.display = 'none';
        }
        tooltip.classList.add('visible');

        // Keep the tooltip fully on-screen: flip above the cursor if it
        // would overflow the bottom edge, clamp horizontally otherwise.
        var x = pendingX;
        var y = pendingY + GAP;
        var maxX = window.innerWidth - tooltip.offsetWidth - 4;
        var maxY = window.innerHeight - tooltip.offsetHeight - 4;
        if (x > maxX) x = Math.max(4, maxX);
        if (y > maxY) y = Math.max(4, pendingY - GAP - tooltip.offsetHeight);
        tooltip.style.left = x + 'px';
        tooltip.style.top = y + 'px';
    }

    function schedule(x, y){
        pendingX = x;
        pendingY = y;
        if (!rafScheduled) {
            rafScheduled = true;
            requestAnimationFrame(apply);
        }
    }

    document.addEventListener('mouseover', function(e){
        var t = e.target;
        var a = (t && t.closest) ? t.closest('a[href]') : null;
        if (a) {
            var href = a.getAttribute('href') || a.href || '';
            if (href) {
                currentHref = href;
                currentTitle = a.getAttribute('data-polo-title') || '';
                schedule(e.clientX, e.clientY);
            }
        }
    }, true);

    document.addEventListener('mousemove', function(e){
        if (currentHref) {
            schedule(e.clientX, e.clientY);
        }
    }, true);

    document.addEventListener('mouseout', function(e){
        var from = e.target;
        var a = (from && from.closest) ? from.closest('a[href]') : null;
        if (!a || !currentHref) return;
        var to = e.relatedTarget;
        if (to && a.contains(to)) return; // still inside the same link
        currentHref = null;
        currentTitle = '';
        if (!rafScheduled) {
            rafScheduled = true;
            requestAnimationFrame(apply);
        }
    }, true);
})();
</script>"#;
