/// Shared HTML preview document wrapper.
///
/// This is intentionally **UI-toolkit agnostic**: it just produces a full HTML
/// document as a String. Both `marco` and `polo` load this into a WebKit WebView.
///
/// The embedded JS exposes `window.MarcoPreview` for smooth content updates and
/// installs interactive table resizing (column + row) in the rendered preview.
pub fn wrap_preview_html_document(
    body: &str,
    css: &str,
    theme_class: &str,
    background_color: Option<&str>,
) -> String {
    // Generate inline background style for instant dark mode support (eliminates white flash)
    let inline_bg_style = if let Some(bg_color) = background_color {
        format!("body {{ background-color: {} !important; }}\n", bg_color)
    } else {
        String::new()
    };

    // Table resize affordances (JS drives cursor; CSS disables selection during drag).
    // Keep this lightweight and self-contained to avoid fighting user themes.
    let table_resize_css = r#"
/* Marco: interactive table resizing */
body.marco-table-resizing,
body.marco-table-resizing * {
    -webkit-user-select: none !important;
    user-select: none !important;
}

table.marco-resize-active {
    table-layout: fixed;
}

table.marco-resize-active th,
table.marco-resize-active td {
    overflow: hidden;
    text-overflow: ellipsis;
}

/* Marco: heading anchor links (visible on hover/focus) */
.marco-heading-anchor {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    margin-left: 0.35em;
    padding: 0.1em;
    border-radius: 4px;
    opacity: 0;
    text-decoration: none !important;
    color: inherit !important;
    user-select: none;
    -webkit-user-select: none;
    transition: opacity 0.12s ease-in-out;
}

.marco-heading-anchor:link,
.marco-heading-anchor:visited {
    /* Force link/visited to be the same as the heading text color (no purple/blue). */
    color: inherit !important;
}

.marco-heading-anchor svg {
    width: 1em;
    height: 1em;
    display: block;
    stroke: currentColor;
}

/* Show on hover like GitHub; also show for keyboard focus */
h1:hover .marco-heading-anchor,
h2:hover .marco-heading-anchor,
h3:hover .marco-heading-anchor,
h4:hover .marco-heading-anchor,
h5:hover .marco-heading-anchor,
h6:hover .marco-heading-anchor {
    opacity: 0.9;
}

.marco-heading-anchor:focus,
.marco-heading-anchor:focus-visible {
    opacity: 0.9;
    color: inherit !important;
}

/* Subtle hover affordance without changing theme colors */
.marco-heading-anchor:hover {
    opacity: 1;
    color: inherit !important;
}

.marco-heading-anchor:active {
    color: inherit !important;
}

/* Marco: internal jumping links (href starting with #)
   - Keeps internal links looking like normal links.
   - On hover/focus, appends an icon (like heading anchors) and suppresses theme hover effects.
   - Uses an SVG mask so the icon inherits the link color via `currentColor`.
   - Excludes the injected heading anchor link itself.
*/
a[href^='#']:not(.marco-heading-anchor) {
    position: relative;
}

a[href^='#']:not(.marco-heading-anchor):link,
a[href^='#']:not(.marco-heading-anchor):visited {
    /* Force internal links to stay the theme's normal link color (no visited purple). */
    color: var(--link-color) !important;
}

a[href^='#']:not(.marco-heading-anchor):hover,
a[href^='#']:not(.marco-heading-anchor):focus,
a[href^='#']:not(.marco-heading-anchor):focus-visible,
a[href^='#']:not(.marco-heading-anchor):active {
    color: var(--link-color) !important;
    text-decoration: none !important;
    text-shadow: none !important;
    background: none !important;
    box-shadow: none !important;
    transform: none !important;
    filter: none !important;
}

a[href^='#']:not(.marco-heading-anchor)::after {
    content: "";
    display: inline-block;
    width: 1em;
    height: 1em;
    margin-left: 0.35em;
    vertical-align: -0.125em;
    opacity: 0;
    background-color: currentColor;
    pointer-events: none;
    transition: opacity 0.12s ease-in-out;

    /* Tabler icon: circles-relation (stroked). Use opaque stroke for mask. */
    -webkit-mask: url("data:image/svg+xml,%3Csvg%20xmlns%3D'http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg'%20viewBox%3D'0%200%2024%2024'%20fill%3D'none'%20stroke%3D'black'%20stroke-width%3D'2'%20stroke-linecap%3D'round'%20stroke-linejoin%3D'round'%3E%3Cpath%20d%3D'M9.183%206.117a6%206%200%201%200%204.511%203.986'%2F%3E%3Cpath%20d%3D'M14.813%2017.883a6%206%200%201%200%20-4.496%20-3.954'%2F%3E%3C%2Fsvg%3E") no-repeat center / contain;
    mask: url("data:image/svg+xml,%3Csvg%20xmlns%3D'http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg'%20viewBox%3D'0%200%2024%2024'%20fill%3D'none'%20stroke%3D'black'%20stroke-width%3D'2'%20stroke-linecap%3D'round'%20stroke-linejoin%3D'round'%3E%3Cpath%20d%3D'M9.183%206.117a6%206%200%201%200%204.511%203.986'%2F%3E%3Cpath%20d%3D'M14.813%2017.883a6%206%200%201%200%20-4.496%20-3.954'%2F%3E%3C%2Fsvg%3E") no-repeat center / contain;
}

a[href^='#']:not(.marco-heading-anchor):hover::after,
a[href^='#']:not(.marco-heading-anchor):focus::after,
a[href^='#']:not(.marco-heading-anchor):focus-visible::after {
    opacity: 0.9;
}

/* Marco: external links (http/https/mailto)
   Same idea as internal jump links: keep links looking normal, but on hover/focus
   append an icon and suppress theme hover effects.
*/
a[href^='http:']:not(.marco-heading-anchor),
a[href^='https:']:not(.marco-heading-anchor),
a[href^='mailto:']:not(.marco-heading-anchor) {
    position: relative;
}

a[href^='http:']:not(.marco-heading-anchor):link,
a[href^='http:']:not(.marco-heading-anchor):visited,
a[href^='https:']:not(.marco-heading-anchor):link,
a[href^='https:']:not(.marco-heading-anchor):visited,
a[href^='mailto:']:not(.marco-heading-anchor):link,
a[href^='mailto:']:not(.marco-heading-anchor):visited {
    color: var(--link-color) !important;
}

a[href^='http:']:not(.marco-heading-anchor):hover,
a[href^='http:']:not(.marco-heading-anchor):focus,
a[href^='http:']:not(.marco-heading-anchor):focus-visible,
a[href^='http:']:not(.marco-heading-anchor):active,
a[href^='https:']:not(.marco-heading-anchor):hover,
a[href^='https:']:not(.marco-heading-anchor):focus,
a[href^='https:']:not(.marco-heading-anchor):focus-visible,
a[href^='https:']:not(.marco-heading-anchor):active,
a[href^='mailto:']:not(.marco-heading-anchor):hover,
a[href^='mailto:']:not(.marco-heading-anchor):focus,
a[href^='mailto:']:not(.marco-heading-anchor):focus-visible,
a[href^='mailto:']:not(.marco-heading-anchor):active {
    color: var(--link-color) !important;
    text-decoration: none !important;
    text-shadow: none !important;
    background: none !important;
    box-shadow: none !important;
    transform: none !important;
    filter: none !important;
}

a[href^='http:']:not(.marco-heading-anchor)::after,
a[href^='https:']:not(.marco-heading-anchor)::after,
a[href^='mailto:']:not(.marco-heading-anchor)::after {
    content: "";
    display: inline-block;
    width: 1em;
    height: 1em;
    margin-left: 0.35em;
    vertical-align: -0.125em;
    opacity: 0;
    background-color: currentColor;
    pointer-events: none;
    transition: opacity 0.12s ease-in-out;

    /* Tabler icon: link (stroked). Use opaque stroke for mask. */
    -webkit-mask: url("data:image/svg+xml,%3Csvg%20xmlns%3D'http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg'%20viewBox%3D'0%200%2024%2024'%20fill%3D'none'%20stroke%3D'black'%20stroke-width%3D'2'%20stroke-linecap%3D'round'%20stroke-linejoin%3D'round'%3E%3Cpath%20d%3D'M9%2015l6%20-6'%2F%3E%3Cpath%20d%3D'M11%206l.463%20-.536a5%205%200%200%201%207.071%207.072l-.534%20.464'%2F%3E%3Cpath%20d%3D'M13%2018l-.397%20.534a5.068%205.068%200%200%201%20-7.127%200a4.972%204.972%200%200%201%200%20-7.071l.524%20-.463'%2F%3E%3C%2Fsvg%3E") no-repeat center / contain;
    mask: url("data:image/svg+xml,%3Csvg%20xmlns%3D'http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg'%20viewBox%3D'0%200%2024%2024'%20fill%3D'none'%20stroke%3D'black'%20stroke-width%3D'2'%20stroke-linecap%3D'round'%20stroke-linejoin%3D'round'%3E%3Cpath%20d%3D'M9%2015l6%20-6'%2F%3E%3Cpath%20d%3D'M11%206l.463%20-.536a5%205%200%200%201%207.071%207.072l-.534%20.464'%2F%3E%3Cpath%20d%3D'M13%2018l-.397%20.534a5.068%205.068%200%200%201%20-7.127%200a4.972%204.972%200%200%201%200%20-7.071l.524%20-.463'%2F%3E%3C%2Fsvg%3E") no-repeat center / contain;
}

a[href^='http:']:not(.marco-heading-anchor):hover::after,
a[href^='http:']:not(.marco-heading-anchor):focus::after,
a[href^='http:']:not(.marco-heading-anchor):focus-visible::after,
a[href^='https:']:not(.marco-heading-anchor):hover::after,
a[href^='https:']:not(.marco-heading-anchor):focus::after,
a[href^='https:']:not(.marco-heading-anchor):focus-visible::after,
a[href^='mailto:']:not(.marco-heading-anchor):hover::after,
a[href^='mailto:']:not(.marco-heading-anchor):focus::after,
a[href^='mailto:']:not(.marco-heading-anchor):focus-visible::after {
    opacity: 0.9;
}

/* Marco: sliders / slide decks */
.marco-sliders {
    position: relative;
    margin: 1rem 0;
    padding: 0.75rem 0.9rem;
    border-radius: 10px;
    border: 1px solid var(--marco-sliders-border, transparent);
    background: var(--marco-sliders-bg, transparent);
}

.marco-sliders__viewport {
    position: relative;
    display: grid;
    grid-template-columns: 1fr;
    overflow: hidden;
}

.marco-sliders__slide {
    grid-area: 1 / 1;
    align-self: start;
    justify-self: stretch;
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transform: translateY(0.35rem);
    transition: opacity 180ms ease-in-out, transform 180ms ease-in-out;
}

.marco-sliders__slide.is-active {
    opacity: 1;
    visibility: visible;
    pointer-events: auto;
    transform: translateY(0);
}

@media (prefers-reduced-motion: reduce) {
    .marco-sliders__slide {
        transition: none !important;
        transform: none !important;
    }
}

.marco-sliders__controls {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    margin-top: 0.5rem;
}

.marco-sliders__btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    padding: 0.25rem 0.35rem;
    border: none;
    background: transparent;
    color: inherit;
    cursor: pointer;
    opacity: 0.85;
}

.marco-sliders__btn:hover,
.marco-sliders__dot:hover {
    opacity: 1;
}

.marco-sliders__btn:disabled {
    opacity: 0.35;
    cursor: default;
}

.marco-sliders__btn svg,
.marco-sliders__dot svg {
    width: 1.15em;
    height: 1.15em;
    display: block;
}

.marco-sliders__dots {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.25rem;
    margin-top: 0.35rem;
}

.marco-sliders__dot {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.1rem;
    border: none;
    background: transparent;
    color: inherit;
    cursor: pointer;
    opacity: 0.75;
}

.marco-sliders__dot.is-active {
    opacity: 1;
}

.marco-sliders__dot-icon--active {
    display: none;
}

.marco-sliders__dot.is-active .marco-sliders__dot-icon--inactive {
    display: none;
}

.marco-sliders__dot.is-active .marco-sliders__dot-icon--active {
    display: inline-flex;
}

/* Toggle button shows play when paused, pause when playing */
.marco-sliders .marco-sliders__icon--pause {
    display: none;
}

.marco-sliders.is-playing .marco-sliders__icon--play {
    display: none;
}

.marco-sliders.is-playing .marco-sliders__icon--pause {
    display: inline-flex;
}
"#;

    // NOTE: This HTML template is used as a Rust `format!` string. All literal
    // braces inside the template must be escaped as `{{` and `}}`.
    format!(
        r#"<!DOCTYPE html>
<html class="{}">
    <head>
        <meta charset=\"utf-8\">
        <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
        <style id=\"marco-preview-style\">{}{}</style>
        <style id=\"marco-preview-internal-style\">{}</style>
        <script>
            // Marco Preview Management Object - prevents global namespace pollution
            window.MarcoPreview = (function() {{
                var scrollTimeouts = [];
                var tableResizerCleanup = null;
                var tableSizeState = Object.create(null);
                var sliderDeckState = Object.create(null);
                var sliderDelegatedInstalled = false;
                var sliderResizeObservers = Object.create(null);
                var sliderMeasureScheduled = Object.create(null);
                var sliderWindowResizeInstalled = false;
                
                // Cleanup function to clear any pending timeouts
                function cleanupScrollRestoration() {{
                    scrollTimeouts.forEach(function(id) {{
                        clearTimeout(id);
                    }});
                    scrollTimeouts = [];
                }}

                // Full cleanup used on page unload / WebView destroy.
                // NOTE: updateContent() should NOT call this, otherwise it would
                // uninstall delegated event listeners and break interactions.
                function cleanup() {{
                    cleanupScrollRestoration();

                    // Stop any slider timers
                    try {{
                        Object.keys(sliderDeckState).forEach(function(deckId) {{
                            var st = sliderDeckState[deckId];
                            if (st && st.intervalId) {{
                                clearInterval(st.intervalId);
                                st.intervalId = null;
                            }}
                        }});
                    }} catch(e) {{
                        console.error('Error stopping sliders:', e);
                    }}

                    // Disconnect any ResizeObservers
                    try {{
                        Object.keys(sliderResizeObservers).forEach(function(deckId) {{
                            var ro = sliderResizeObservers[deckId];
                            if (ro && typeof ro.disconnect === 'function') {{
                                ro.disconnect();
                            }}
                        }});
                    }} catch(e) {{
                        console.error('Error disconnecting slider ResizeObservers:', e);
                    }}

                    // Remove table resizer listeners (if installed)
                    try {{
                        if (typeof tableResizerCleanup === 'function') {{
                            tableResizerCleanup();
                        }}
                    }} catch(e) {{
                        console.error('Error cleaning up table resizer:', e);
                    }}

                    // Clear any persisted state
                    tableSizeState = Object.create(null);
                    sliderDeckState = Object.create(null);
                    sliderResizeObservers = Object.create(null);
                    sliderMeasureScheduled = Object.create(null);
                }}

                function parsePositiveInt(s) {{
                    var n = parseInt(s, 10);
                    if (!isFinite(n) || isNaN(n) || n <= 0) return null;
                    return n;
                }}

                function setDeckPlaying(deck, playing) {{
                    try {{
                        if (playing) deck.classList.add('is-playing');
                        else deck.classList.remove('is-playing');
                    }} catch(_e) {{
                        // ignore
                    }}
                }}

                function getDeckState(deck) {{
                    if (!deck || !deck.id) return null;
                    return sliderDeckState[deck.id] || null;
                }}

                function measureDeckViewportHeight(deck) {{
                    try {{
                        if (!deck) return;
                        var viewport = deck.querySelector('.marco-sliders__viewport');
                        if (!viewport) return;

                        var slides = deck.querySelectorAll('.marco-sliders__slide');
                        if (!slides || slides.length === 0) return;

                        var maxH = 0;
                        for (var i = 0; i < slides.length; i++) {{
                            var el = slides[i];
                            if (!el) continue;
                            var r = el.getBoundingClientRect ? el.getBoundingClientRect() : null;
                            var h = (r && r.height) ? r.height : (el.scrollHeight || 0);
                            if (h > maxH) maxH = h;
                        }}

                        if (maxH > 0) {{
                            viewport.style.minHeight = Math.ceil(maxH) + 'px';
                        }}
                    }} catch(e) {{
                        console.error('Failed to measure slider deck height:', e);
                    }}
                }}

                function scheduleDeckMeasure(deck) {{
                    try {{
                        if (!deck || !deck.id) return;
                        if (sliderMeasureScheduled[deck.id]) return;
                        sliderMeasureScheduled[deck.id] = true;
                        requestAnimationFrame(function() {{
                            try {{
                                delete sliderMeasureScheduled[deck.id];
                                measureDeckViewportHeight(deck);
                            }} catch(_e) {{
                                // ignore
                            }}
                        }});
                    }} catch(_e) {{
                        // ignore
                    }}
                }}

                function ensureSliderWindowResizeInstalled() {{
                    if (sliderWindowResizeInstalled) return;
                    sliderWindowResizeInstalled = true;

                    window.addEventListener('resize', function() {{
                        try {{
                            Object.keys(sliderDeckState).forEach(function(deckId) {{
                                var st = sliderDeckState[deckId];
                                if (st && st.deckEl) scheduleDeckMeasure(st.deckEl);
                            }});
                        }} catch(e) {{
                            console.error('Slider resize handler error:', e);
                        }}
                    }}, true);
                }}

                function installDeckResizeObserver(deck) {{
                    try {{
                        if (!deck || !deck.id) return;
                        if (sliderResizeObservers[deck.id]) return;

                        if (!window.ResizeObserver) return;
                        var ro = new ResizeObserver(function(_entries) {{
                            scheduleDeckMeasure(deck);
                        }});

                        // Observe the viewport and each slide so height changes (e.g. images loading)
                        // trigger a re-measure.
                        var viewport = deck.querySelector('.marco-sliders__viewport');
                        if (viewport) ro.observe(viewport);

                        var slides = deck.querySelectorAll('.marco-sliders__slide');
                        for (var i = 0; i < slides.length; i++) {{
                            ro.observe(slides[i]);
                        }}

                        sliderResizeObservers[deck.id] = ro;
                    }} catch(e) {{
                        // ResizeObserver is best-effort; don't break sliders if it fails.
                        console.error('Failed to install ResizeObserver for slider deck:', e);
                    }}
                }}

                function showSlide(deck, index) {{
                    var st = getDeckState(deck);
                    if (!st) return;
                    var slides = deck.querySelectorAll('.marco-sliders__slide');
                    var dots = deck.querySelectorAll('.marco-sliders__dot');
                    if (!slides || slides.length === 0) return;

                    var n = slides.length;
                    var i = index;
                    if (i < 0) i = n - 1;
                    if (i >= n) i = 0;
                    st.index = i;

                    for (var k = 0; k < slides.length; k++) {{
                        if (k === i) slides[k].classList.add('is-active');
                        else slides[k].classList.remove('is-active');

                        // Keep hidden slides out of the accessibility tree.
                        try {{
                            if (k === i) slides[k].removeAttribute('aria-hidden');
                            else slides[k].setAttribute('aria-hidden', 'true');
                        }} catch(_e) {{
                            // ignore
                        }}
                    }}

                    for (var d = 0; d < dots.length; d++) {{
                        if (d === i) dots[d].classList.add('is-active');
                        else dots[d].classList.remove('is-active');

                        // Sync ARIA for keyboard/screen-reader navigation.
                        try {{
                            if (d === i) {{
                                dots[d].setAttribute('aria-selected', 'true');
                                dots[d].setAttribute('tabindex', '0');
                            }} else {{
                                dots[d].setAttribute('aria-selected', 'false');
                                dots[d].setAttribute('tabindex', '-1');
                            }}
                        }} catch(_e) {{
                            // ignore
                        }}
                    }}

                    // Lock the viewport size to the tallest slide to avoid layout jumps.
                    scheduleDeckMeasure(deck);
                }}

                function slidersPauseDeck(deckId) {{
                    var st = sliderDeckState[deckId];
                    if (!st) return;
                    if (st.intervalId) {{
                        clearInterval(st.intervalId);
                        st.intervalId = null;
                    }}
                    st.playing = false;
                    if (st.deckEl) setDeckPlaying(st.deckEl, false);
                }}

                function slidersPlayDeck(deckId) {{
                    var st = sliderDeckState[deckId];
                    if (!st) return;
                    if (!st.timerSeconds || st.timerSeconds <= 0) return;

                    slidersPauseDeck(deckId);
                    st.playing = true;
                    if (st.deckEl) setDeckPlaying(st.deckEl, true);

                    st.intervalId = setInterval(function() {{
                        try {{
                            var deck = st.deckEl;
                            if (!deck) return;
                            showSlide(deck, st.index + 1);
                        }} catch(e) {{
                            console.error('Slider tick error:', e);
                        }}
                    }}, st.timerSeconds * 1000);
                }}

                function slidersToggleDeck(deckId) {{
                    var st = sliderDeckState[deckId];
                    if (!st) return;
                    if (st.playing) slidersPauseDeck(deckId);
                    else slidersPlayDeck(deckId);
                }}

                function slidersPauseAll() {{
                    Object.keys(sliderDeckState).forEach(function(deckId) {{
                        slidersPauseDeck(deckId);
                    }});
                }}

                function slidersPlayAll() {{
                    Object.keys(sliderDeckState).forEach(function(deckId) {{
                        slidersPlayDeck(deckId);
                    }});
                }}

                function slidersToggleAll() {{
                    Object.keys(sliderDeckState).forEach(function(deckId) {{
                        slidersToggleDeck(deckId);
                    }});
                }}

                function initSliderDeck(deck) {{
                    if (!deck || !deck.id) return;
                    var timerSeconds = parsePositiveInt(deck.getAttribute('data-timer-seconds'));
                    var slides = deck.querySelectorAll('.marco-sliders__slide');
                    if (!slides || slides.length === 0) return;

                    sliderDeckState[deck.id] = {{
                        deckEl: deck,
                        index: 0,
                        timerSeconds: timerSeconds,
                        intervalId: null,
                        playing: false
                    }};

                    // Disable toggle button if no timer.
                    var toggleBtn = deck.querySelector('.marco-sliders__btn--toggle');
                    if (toggleBtn) {{
                        if (!timerSeconds) {{
                            toggleBtn.disabled = true;
                            toggleBtn.setAttribute('aria-disabled', 'true');
                        }} else {{
                            toggleBtn.disabled = false;
                            toggleBtn.removeAttribute('aria-disabled');
                        }}
                    }}

                    showSlide(deck, 0);
                    setDeckPlaying(deck, false);

                    // Prevent content jumps by measuring the largest slide and
                    // keeping the viewport height stable.
                    ensureSliderWindowResizeInstalled();
                    installDeckResizeObserver(deck);
                    scheduleDeckMeasure(deck);

                    // Autoplay if timer is present.
                    if (timerSeconds) {{
                        slidersPlayDeck(deck.id);
                    }}
                }}

                function ensureSliderDelegationInstalled() {{
                    if (sliderDelegatedInstalled) return;
                    sliderDelegatedInstalled = true;

                    // Delegated click handler; survives innerHTML updates.
                    document.addEventListener('click', function(ev) {{
                        try {{
                            var target = ev.target;
                            if (!target) return;
                            var btn = target.closest('button');
                            if (!btn) return;
                            var deck = btn.closest('.marco-sliders');
                            if (!deck) return;

                            var action = btn.getAttribute('data-action');
                            var st = getDeckState(deck);
                            if (!st) return;

                            if (action === 'prev') {{
                                showSlide(deck, st.index - 1);
                            }} else if (action === 'next') {{
                                showSlide(deck, st.index + 1);
                            }} else if (action === 'goto') {{
                                var idx = parseInt(btn.getAttribute('data-index'), 10);
                                if (!isNaN(idx)) showSlide(deck, idx);
                            }} else if (action === 'toggle') {{
                                slidersToggleDeck(deck.id);
                            }}
                        }} catch(e) {{
                            console.error('Slider click handler error:', e);
                        }}
                    }}, true);
                }}

                function installSliders(container) {{
                    try {{
                        // Stop existing timers and rebuild state for the new DOM.
                        slidersPauseAll();

                        // Disconnect any prior observers (they reference old DOM nodes).
                        try {{
                            Object.keys(sliderResizeObservers).forEach(function(deckId) {{
                                var ro = sliderResizeObservers[deckId];
                                if (ro && typeof ro.disconnect === 'function') ro.disconnect();
                            }});
                        }} catch(_e) {{
                            // ignore
                        }}

                        sliderDeckState = Object.create(null);
                        sliderResizeObservers = Object.create(null);
                        sliderMeasureScheduled = Object.create(null);
                        ensureSliderDelegationInstalled();

                        if (!container) return;
                        var decks = container.querySelectorAll('.marco-sliders');
                        for (var i = 0; i < decks.length; i++) {{
                            initSliderDeck(decks[i]);
                        }}
                    }} catch(e) {{
                        console.error('Failed to install sliders:', e);
                    }}
                }}

                function applyStoredTableSizes(container) {{
                    try {{
                        if (!container) return;
                        var tables = container.querySelectorAll('table');

                        function firstRowCellCount(tbl) {{
                            try {{
                                if (!tbl || !tbl.rows || tbl.rows.length === 0) return 0;
                                return (tbl.rows[0] && tbl.rows[0].cells) ? tbl.rows[0].cells.length : 0;
                            }} catch(_e) {{
                                return 0;
                            }}
                        }}

                        function ensureColGroup(tbl, colCount) {{
                            if (!tbl || colCount <= 0) return null;
                            var cg = tbl.querySelector('colgroup');
                            if (!cg) {{
                                cg = document.createElement('colgroup');

                                // Insert after caption if present, otherwise as the first child.
                                var first = tbl.firstElementChild;
                                if (first && first.tagName === 'CAPTION') {{
                                    if (first.nextSibling) {{
                                        tbl.insertBefore(cg, first.nextSibling);
                                    }} else {{
                                        tbl.appendChild(cg);
                                    }}
                                }} else if (first) {{
                                    tbl.insertBefore(cg, first);
                                }} else {{
                                    tbl.appendChild(cg);
                                }}
                            }}

                            // Normalize number of <col> elements.
                            var cols = cg.querySelectorAll('col');
                            if (cols.length !== colCount) {{
                                cg.innerHTML = '';
                                for (var i = 0; i < colCount; i++) {{
                                    cg.appendChild(document.createElement('col'));
                                }}
                            }}
                            return cg;
                        }}

                        for (var i = 0; i < tables.length; i++) {{
                            var tbl = tables[i];
                            var key = 't' + i;
                            var state = tableSizeState[key];
                            if (!state) continue;

                            // Apply stored column widths
                            if (state.cols) {{
                                var colCount = firstRowCellCount(tbl);
                                var wantCols = Math.max(colCount, state.cols.length || 0);
                                var cg = ensureColGroup(tbl, wantCols);
                                if (cg) {{
                                    var cols = cg.querySelectorAll('col');
                                    for (var ci = 0; ci < state.cols.length && ci < cols.length; ci++) {{
                                        if (state.cols[ci]) {{
                                            cols[ci].style.width = state.cols[ci];
                                        }}
                                    }}
                                    try {{
                                        tbl.classList.add('marco-resize-active');
                                        tbl.style.tableLayout = 'fixed';
                                    }} catch(_e) {{
                                        // ignore
                                    }}
                                }}
                            }}

                            // Apply stored table width (helps keep col widths stable)
                            if (state.tableWidth) {{
                                try {{
                                    tbl.style.width = state.tableWidth;
                                }} catch(_e) {{
                                    // ignore
                                }}
                            }}

                            // Apply stored row heights
                            if (state.rows) {{
                                var trs = tbl.querySelectorAll('tr');
                                for (var ri = 0; ri < state.rows.length && ri < trs.length; ri++) {{
                                    if (state.rows[ri]) {{
                                        trs[ri].style.height = state.rows[ri];
                                    }}
                                }}
                            }}
                        }}
                    }} catch(e) {{
                        console.error('Error applying stored table sizes:', e);
                    }}
                }}

                // Interactive table row/column resizing (HTML preview only).
                // - Column resize: near right edge of a TH/TD (priority over row)
                // - Row resize: near bottom edge of a TR
                // - Uses <colgroup> widths for column stability
                // - Disables text selection while actively resizing
                function installTableResizer() {{
                    var EDGE_PX = 5;
                    var MIN_COL_W = 40;
                    var MAX_COL_W = 2000;
                    var MIN_ROW_H = 18;
                    var MAX_ROW_H = 1200;

                    var active = false;
                    var mode = null; // 'col' | 'row'
                    var startX = 0;
                    var startY = 0;
                    var table = null;
                    var colIndex = -1;
                    var colEl = null;
                    var startColW = 0;
                    var startTableW = 0;
                    var rowEl = null;
                    var startRowH = 0;

                    function clamp(v, minV, maxV) {{
                        return Math.max(minV, Math.min(maxV, v));
                    }}

                    function setCursor(cursor) {{
                        try {{
                            if (document && document.body) {{
                                document.body.style.cursor = cursor || '';
                            }}
                        }} catch(_e) {{
                            // ignore
                        }}
                    }}

                    function closestCell(target) {{
                        if (!target) return null;
                        if (target.nodeType !== 1) return null;
                        if (target.tagName === 'TD' || target.tagName === 'TH') return target;
                        return target.closest ? target.closest('td, th') : null;
                    }}

                    function getTableFromCell(cell) {{
                        if (!cell) return null;
                        return cell.closest ? cell.closest('table') : null;
                    }}

                    function firstRowCellCount(tbl) {{
                        try {{
                            if (!tbl || !tbl.rows || tbl.rows.length === 0) return 0;
                            return (tbl.rows[0] && tbl.rows[0].cells) ? tbl.rows[0].cells.length : 0;
                        }} catch(_e) {{
                            return 0;
                        }}
                    }}

                    function ensureColGroup(tbl, colCount) {{
                        if (!tbl || colCount <= 0) return null;
                        var cg = tbl.querySelector('colgroup');
                        if (!cg) {{
                            cg = document.createElement('colgroup');

                            // Insert after caption if present, otherwise as the first child.
                            var first = tbl.firstElementChild;
                            if (first && first.tagName === 'CAPTION') {{
                                if (first.nextSibling) {{
                                    tbl.insertBefore(cg, first.nextSibling);
                                }} else {{
                                    tbl.appendChild(cg);
                                }}
                            }} else if (first) {{
                                tbl.insertBefore(cg, first);
                            }} else {{
                                tbl.appendChild(cg);
                            }}
                        }}

                        // Normalize number of <col> elements.
                        var cols = cg.querySelectorAll('col');
                        if (cols.length !== colCount) {{
                            cg.innerHTML = '';
                            for (var i = 0; i < colCount; i++) {{
                                cg.appendChild(document.createElement('col'));
                            }}
                        }}
                        return cg;
                    }}

                    function initColumnWidths(tbl) {{
                        var colCount = firstRowCellCount(tbl);
                        if (colCount <= 0) return null;
                        var cg = ensureColGroup(tbl, colCount);
                        if (!cg) return null;
                        var cols = cg.querySelectorAll('col');

                        // Lock initial widths only if not already explicit.
                        for (var i = 0; i < cols.length; i++) {{
                            if (!cols[i].style.width) {{
                                var cell = (tbl.rows[0] && tbl.rows[0].cells[i]) ? tbl.rows[0].cells[i] : null;
                                if (cell) {{
                                    var r = cell.getBoundingClientRect();
                                    cols[i].style.width = Math.max(MIN_COL_W, Math.round(r.width)) + 'px';
                                }}
                            }}
                        }}
                        return cg;
                    }}

                    function isInRightEdgeZone(cell, x) {{
                        if (!cell) return false;
                        var r = cell.getBoundingClientRect();
                        return Math.abs(r.right - x) <= EDGE_PX;
                    }}

                    function isInBottomEdgeZone(cell, y) {{
                        if (!cell) return false;
                        var r = cell.getBoundingClientRect();
                        return Math.abs(r.bottom - y) <= EDGE_PX;
                    }}

                    function findResizeTarget(ev) {{
                        var cell = closestCell(ev.target);
                        if (!cell) return null;
                        var tbl = getTableFromCell(cell);
                        if (!tbl) return null;

                        // Ignore nested tables (choose the closest table of the cell).
                        var x = ev.clientX;
                        var y = ev.clientY;

                        // Priority: column resize > row resize
                        if (isInRightEdgeZone(cell, x)) {{
                            return {{ mode: 'col', table: tbl, cell: cell }};
                        }}
                        if (isInBottomEdgeZone(cell, y)) {{
                            var tr = cell.parentElement;
                            if (tr && tr.tagName === 'TR') {{
                                return {{ mode: 'row', table: tbl, row: tr, cell: cell }};
                            }}
                        }}
                        return null;
                    }}

                    function startColResize(tbl, cell, ev) {{
                        var cg = initColumnWidths(tbl);
                        if (!cg) return false;

                        var idx = (typeof cell.cellIndex === 'number') ? cell.cellIndex : -1;
                        if (idx < 0) return false;

                        var cols = cg.querySelectorAll('col');
                        if (idx >= cols.length) return false;

                        table = tbl;
                        colIndex = idx;
                        colEl = cols[idx];
                        startX = ev.clientX;
                        var cellRect = cell.getBoundingClientRect();
                        startColW = Math.max(MIN_COL_W, Math.round(cellRect.width));
                        startTableW = Math.round(tbl.getBoundingClientRect().width);

                        // Freeze layout so only the target column changes.
                        try {{
                            tbl.classList.add('marco-resize-active');
                            tbl.style.tableLayout = 'fixed';
                            tbl.style.width = startTableW + 'px';
                        }} catch(_e) {{
                            // ignore
                        }}

                        // Ensure the col reflects our start width.
                        colEl.style.width = startColW + 'px';

                        mode = 'col';
                        active = true;
                        return true;
                    }}

                    function startRowResize(tr, ev) {{
                        rowEl = tr;
                        startY = ev.clientY;
                        startRowH = Math.round(tr.getBoundingClientRect().height);
                        mode = 'row';
                        active = true;
                        return true;
                    }}

                    function beginResize(ev, target) {{
                        if (!target) return false;
                        if (ev.button !== 0) return false;

                        // Prevent text selection / link activation while resizing.
                        ev.preventDefault();
                        ev.stopPropagation();

                        if (document && document.body) {{
                            document.body.classList.add('marco-table-resizing');
                        }}

                        if (target.mode === 'col') {{
                            return startColResize(target.table, target.cell, ev);
                        }}
                        if (target.mode === 'row') {{
                            return startRowResize(target.row, ev);
                        }}
                        return false;
                    }}

                    function applyResize(ev) {{
                        if (!active) return;
                        ev.preventDefault();
                        ev.stopPropagation();

                        if (mode === 'col' && table && colEl) {{
                            var dx = ev.clientX - startX;
                            var newW = clamp(startColW + dx, MIN_COL_W, MAX_COL_W);
                            colEl.style.width = Math.round(newW) + 'px';

                            // Keep other columns stable by changing the overall table width.
                            var newTableW = clamp(startTableW + (newW - startColW), MIN_COL_W, MAX_COL_W * 50);
                            table.style.width = Math.round(newTableW) + 'px';
                            return;
                        }}
                        if (mode === 'row' && rowEl) {{
                            var dy = ev.clientY - startY;
                            var newH = clamp(startRowH + dy, MIN_ROW_H, MAX_ROW_H);
                            rowEl.style.height = Math.round(newH) + 'px';
                            return;
                        }}
                    }}

                    function endResize() {{
                        if (!active) return;

                        // Persist the last resize so it survives smooth preview updates.
                        try {{
                            function getTableKey(tbl) {{
                                var container = document.getElementById('marco-content-container');
                                if (!container || !tbl) return null;
                                var tables = container.querySelectorAll('table');
                                for (var i = 0; i < tables.length; i++) {{
                                    if (tables[i] === tbl) return 't' + i;
                                }}
                                return null;
                            }}

                            function getRowIndex(tbl, tr) {{
                                if (!tbl || !tr) return -1;
                                var trs = tbl.querySelectorAll('tr');
                                for (var i = 0; i < trs.length; i++) {{
                                    if (trs[i] === tr) return i;
                                }}
                                return -1;
                            }}

                            if (mode === 'col' && table && colIndex >= 0 && colEl) {{
                                var key = getTableKey(table);
                                if (key) {{
                                    if (!tableSizeState[key]) tableSizeState[key] = {{ cols: [], rows: [] }};
                                    tableSizeState[key].cols[colIndex] = colEl.style.width || null;
                                    tableSizeState[key].tableWidth = (table.style && table.style.width) ? table.style.width : null;
                                }}
                            }} else if (mode === 'row' && rowEl) {{
                                var t = rowEl.closest ? rowEl.closest('table') : null;
                                var key2 = getTableKey(t);
                                if (key2 && t) {{
                                    if (!tableSizeState[key2]) tableSizeState[key2] = {{ cols: [], rows: [] }};
                                    var idx = getRowIndex(t, rowEl);
                                    if (idx >= 0) {{
                                        tableSizeState[key2].rows[idx] = rowEl.style.height || null;
                                    }}
                                }}
                            }}
                        }} catch(e) {{
                            console.error('Error persisting table resize state:', e);
                        }}

                        active = false;
                        mode = null;
                        colIndex = -1;
                        colEl = null;
                        rowEl = null;

                        if (document && document.body) {{
                            document.body.classList.remove('marco-table-resizing');
                        }}
                        setCursor('');
                    }}

                    function onMouseMove(ev) {{
                        if (active) {{
                            applyResize(ev);
                            return;
                        }}

                        var t = findResizeTarget(ev);
                        if (t && t.mode === 'col') {{
                            setCursor('col-resize');
                            return;
                        }}
                        if (t && t.mode === 'row') {{
                            setCursor('row-resize');
                            return;
                        }}
                        setCursor('');
                    }}

                    function onMouseDown(ev) {{
                        if (active) return;
                        var t = findResizeTarget(ev);
                        if (t) {{
                            beginResize(ev, t);
                        }}
                    }}

                    function onMouseUp(_ev) {{
                        endResize();
                    }}

                    function onKeyDown(ev) {{
                        // Escape cancels an active resize.
                        if (ev && ev.key === 'Escape') {{
                            endResize();
                        }}
                    }}

                    // Install listeners once (event delegation; works across content updates).
                    document.addEventListener('mousemove', onMouseMove, true);
                    document.addEventListener('mousedown', onMouseDown, true);
                    document.addEventListener('mouseup', onMouseUp, true);
                    window.addEventListener('blur', endResize, true);
                    document.addEventListener('keydown', onKeyDown, true);

                    return function uninstall() {{
                        try {{
                            document.removeEventListener('mousemove', onMouseMove, true);
                            document.removeEventListener('mousedown', onMouseDown, true);
                            document.removeEventListener('mouseup', onMouseUp, true);
                            window.removeEventListener('blur', endResize, true);
                            document.removeEventListener('keydown', onKeyDown, true);
                        }} catch(_e) {{
                            // ignore
                        }}
                        endResize();
                    }};
                }}

                // Install immediately (listeners are delegated, no per-table init required)
                try {{
                    tableResizerCleanup = installTableResizer();
                }} catch(e) {{
                    console.error('Failed to install table resizer:', e);
                }}

                // Initialize any sliders that are already present in the initial HTML.
                // Without this, slider slides default to `display:none` and nothing shows
                // until the host app calls setContent()/updateContent().
                try {{
                    document.addEventListener('DOMContentLoaded', function() {{
                        var container = document.getElementById('marco-content-container');
                        if (container) {{
                            applyStoredTableSizes(container);
                            installSliders(container);
                        }}
                    }});
                }} catch(e) {{
                    console.error('Failed to auto-init sliders:', e);
                }}
                
                return {{
                    setCSS: function(css) {{
                        try {{
                            var el = document.getElementById('marco-preview-style');
                            if (el) {{
                                el.innerHTML = css;
                            }}
                        }} catch(e) {{
                            console.error('Error setting CSS:', e);
                        }}
                    }},
                    
                    setTheme: function(mode) {{
                        try {{
                            document.documentElement.className = mode;
                        }} catch(e) {{
                            console.error('Error setting theme:', e);
                        }}
                    }},
                    
                    updateContent: function(htmlContent) {{
                        try {{
                            // Clean up any pending scroll restoration (keep interactions installed)
                            cleanupScrollRestoration();
                            
                            // Save current scroll position
                            var scrollTop = document.documentElement.scrollTop || document.body.scrollTop;
                            
                            // Update content container
                            var container = document.getElementById('marco-content-container');
                            if (container) {{
                                container.innerHTML = htmlContent;
                                applyStoredTableSizes(container);
                                installSliders(container);
                                
                                // Restore scroll position after a brief delay
                                var timeoutId = setTimeout(function() {{
                                    document.documentElement.scrollTop = scrollTop;
                                    document.body.scrollTop = scrollTop;
                                    // Remove this timeout from tracking
                                    var index = scrollTimeouts.indexOf(timeoutId);
                                    if (index > -1) {{
                                        scrollTimeouts.splice(index, 1);
                                    }}
                                }}, 10);
                                scrollTimeouts.push(timeoutId);
                            }}
                        }} catch(e) {{
                            console.error('Error updating content:', e);
                        }}
                    }},
                    
                    setContent: function(htmlContent) {{
                        try {{
                            var container = document.getElementById('marco-content-container');
                            if (container) {{
                                container.innerHTML = htmlContent;
                                applyStoredTableSizes(container);
                                installSliders(container);
                            }}
                        }} catch(e) {{
                            console.error('Error setting content:', e);
                        }}
                    }},

                    sliders: {{
                        playAll: slidersPlayAll,
                        pauseAll: slidersPauseAll,
                        toggleAll: slidersToggleAll,
                        playDeck: slidersPlayDeck,
                        pauseDeck: slidersPauseDeck,
                        toggleDeck: slidersToggleDeck
                    }},
                    
                    cleanup: cleanup
                }};
            }})();
            
            // Cleanup on page unload
            window.addEventListener('beforeunload', function() {{
                if (window.MarcoPreview) {{
                    MarcoPreview.cleanup();
                }}
            }});
        </script>
    </head>
    <body>
        <div id="marco-content-container">{}</div>
    </body>
</html>"#,
        theme_class, inline_bg_style, css, table_resize_css, body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_wrap_preview_contains_expected_hooks() {
        let doc = wrap_preview_html_document(
            "<table><tr><td>a</td></tr></table>",
            "body { color: red; }",
            "dark",
            Some("#000000"),
        );
        assert!(doc.contains("id=\\\"marco-preview-style\\\""));
        assert!(doc.contains("id=\\\"marco-preview-internal-style\\\""));
        assert!(doc.contains("window.MarcoPreview"));
        assert!(doc.contains("installTableResizer"));
        assert!(doc.contains("installSliders"));
        assert!(doc.contains("sliders:"));
        assert!(doc.contains("marco-content-container"));
    }
}
