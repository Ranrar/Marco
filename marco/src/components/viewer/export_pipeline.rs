//! Unified Export & Print Pipeline.
//!
//! See `.dev/Roadmap_developement/marco/wry export and print/unified_export_pipeline.md`
//! for the design rationale. This module provides the public types, the
//! [`PlatformExportBackend`] trait, the lifecycle JS bridge, and the
//! [`run_export`] state machine.
//!
//! Migration status:
//! * Phase 2 (done) — Windows PDF wired via `WindowsExportBackend`.
//! * Phase 3 (done) — Linux PDF wired via `LinuxExportBackend`.
//! * Phase 4 (done) — HTML export uses a static-wrap composer
//!   ([`run_static_html_export`]) shared by both platforms. The roadmap
//!   open-question §6.4 ("static wrap is simpler and matches what we ship
//!   today") was resolved in favour of the simpler path; this avoids the
//!   wry result-callback gap and keeps HTML output byte-stable across runs.
//!
//! [`run_export`] is therefore PDF-only — HTML callers must use
//! [`run_static_html_export`].

#![allow(dead_code)]

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::components::viewer::backend as viewer_backend;
use crate::components::viewer::pagedjs;
use crate::components::viewer::preview_types::PlatformWebView;

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// Output format selected by the export dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Pdf,
    Html,
}

/// Lifecycle phases of an export run.
///
/// Each phase has a label (rendered into the [`ExportingDialog`]) and a budget
/// (the upper bound on how long the pipeline waits for the corresponding
/// lifecycle event before returning [`ExportError::Timeout`]).
///
/// [`ExportingDialog`]: crate::ui::dialogs::exporting::ExportingDialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportPhase {
    Preparing,
    Loading,
    Paginating,
    ApplyingPrintCss,
    WritingOutput,
    RestoringPreview,
    Done,
}

impl ExportPhase {
    /// Human-readable phase label for the progress UI.
    pub fn label(self) -> &'static str {
        match self {
            ExportPhase::Preparing => "Preparing…",
            ExportPhase::Loading => "Loading paged.js…",
            ExportPhase::Paginating => "Paginating pages…",
            ExportPhase::ApplyingPrintCss => "Applying print styles…",
            ExportPhase::WritingOutput => "Writing output…",
            ExportPhase::RestoringPreview => "Restoring preview…",
            ExportPhase::Done => "Done",
        }
    }

    /// Maximum time the pipeline waits in this phase before timing out.
    ///
    /// Mirrors the existing 4-second Windows wait + 30-second paged.js fallback,
    /// but is now enforced as a structured timeout rather than a fixed sleep.
    pub fn budget(self) -> Duration {
        match self {
            ExportPhase::Loading => Duration::from_secs(15),
            ExportPhase::Paginating => Duration::from_secs(30),
            ExportPhase::ApplyingPrintCss => Duration::from_secs(5),
            // Phases that do not wait on a lifecycle event use a generous
            // safety budget; the operation either completes or returns its
            // own error first.
            _ => Duration::from_secs(60),
        }
    }
}

/// Errors that can terminate an export run.
#[derive(Debug)]
pub enum ExportError {
    /// User pressed the X on the [`ExportingDialog`].
    Cancelled,
    /// A lifecycle event was not received within [`ExportPhase::budget`].
    Timeout(ExportPhase),
    /// The platform backend returned an error (PrintToPdf failure, etc.).
    Backend(String),
    /// Filesystem I/O failed (output path / metadata read / write).
    Io(String),
    /// PDF backend reported success but the resulting file is < 64 B.
    EmptyOutput,
    /// Live preview WebView is not initialised.
    NotInitialized,
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Cancelled => write!(f, "Export cancelled by user"),
            ExportError::Timeout(p) => write!(f, "Timed out during phase: {}", p.label()),
            ExportError::Backend(msg) => write!(f, "Export backend error: {}", msg),
            ExportError::Io(msg) => write!(f, "I/O error during export: {}", msg),
            ExportError::EmptyOutput => write!(f, "Export produced an empty output file"),
            ExportError::NotInitialized => write!(f, "Preview WebView is not initialised"),
        }
    }
}

impl std::error::Error for ExportError {}

/// Lifecycle events emitted by the export WebView's JavaScript bridge.
///
/// On Linux these arrive via `notify::title` (`document.title = '…'`).
/// On Windows they arrive via `with_ipc_handler` (`window.ipc.postMessage`).
/// The bridge JS (see [`LIFECYCLE_BRIDGE_JS`]) posts on **both** channels so
/// the same payload works regardless of platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifecycleEvent {
    Loaded,
    LayoutDone,
    CssApplied,
    Error(String),
}

impl LifecycleEvent {
    /// Common prefix used for both `document.title` and `ipc.postMessage`
    /// payloads. Existing handlers (`marco_scroll:`, `mc_paged_ready`)
    /// ignore this prefix and continue to fire normally.
    pub const PREFIX: &'static str = "marco_export:";

    /// Parse a raw payload (e.g. `"marco_export:layout_done"`) into a
    /// strongly-typed event. Returns `None` if the prefix does not match.
    pub fn parse(raw: &str) -> Option<Self> {
        let body = raw.strip_prefix(Self::PREFIX)?;
        // Allow optional `:detail` segment (used by `error`).
        let (name, detail) = match body.find(':') {
            Some(idx) => (&body[..idx], Some(&body[idx + 1..])),
            None => (body, None),
        };
        match name {
            "loaded" => Some(LifecycleEvent::Loaded),
            "layout_done" => Some(LifecycleEvent::LayoutDone),
            "css_applied" => Some(LifecycleEvent::CssApplied),
            "error" => Some(LifecycleEvent::Error(
                detail.unwrap_or("unknown").to_string(),
            )),
            _ => None,
        }
    }
}

/// Inputs for a single export run. Built once in `main.rs` after the user
/// confirms the export dialog and consumed by [`run_export`].
pub struct ExportRequest {
    pub format: ExportFormat,
    /// Pre-rendered HTML body (`marco_core::parse_to_html_cached` output).
    pub html_body: String,
    /// Combined theme CSS + syntax-highlighter CSS.
    pub theme_css: String,
    /// Theme class applied to the wrapping document (e.g. `"theme-dark"`).
    pub theme_class: String,
    pub paper: String,
    pub orientation: String,
    pub margin_mm: u8,
    pub show_page_numbers: bool,
    pub title: String,
    pub output_path: PathBuf,
    pub base_uri: Option<String>,
    pub dark_mode: bool,
}

/// Cooperative cancellation flag shared between the [`ExportingDialog`] and
/// the running pipeline. Setting [`Self::cancel`] causes the next
/// `wait_for_event` call to return [`ExportError::Cancelled`].
///
/// [`ExportingDialog`]: crate::ui::dialogs::exporting::ExportingDialog
#[derive(Clone, Default)]
pub struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// Sink for phase updates. Implemented by the [`ExportingDialog`] so the
/// pulse bar's label tracks the state machine. Tests use [`NoopReporter`].
pub trait ProgressReporter {
    fn set_phase(&self, phase: ExportPhase);
}

/// `ProgressReporter` that discards all phase updates. Useful in tests and
/// non-UI export paths.
pub struct NoopReporter;
impl ProgressReporter for NoopReporter {
    fn set_phase(&self, _phase: ExportPhase) {}
}

// ─────────────────────────────────────────────────────────────────────────────
// Lifecycle JS bridge
// ─────────────────────────────────────────────────────────────────────────────

/// JS injected into every export WebView. Posts `marco_export:*` payloads
/// over both `document.title` (Linux/WebKit) and `window.ipc.postMessage`
/// (Windows/wry). Coexists with the existing `mc_paged_ready` and
/// `marco_scroll:` handlers — they ignore unknown prefixes.
pub const LIFECYCLE_BRIDGE_JS: &str = r#"<script id="marco-export-lifecycle">
(function(){
    if (window.__marcoExportBridge) return;
    window.__marcoExportBridge = true;

    function post(name, detail){
        var msg = 'marco_export:' + name + (detail ? ':' + detail : '');
        try { if (window.ipc && window.ipc.postMessage) window.ipc.postMessage(msg); } catch(_){}
        try { document.title = msg; } catch(_){}
    }
    window.__marcoExportPost = post;

    function onReady(fn){
        if (document.readyState !== 'loading') fn();
        else document.addEventListener('DOMContentLoaded', fn);
    }
    onReady(function(){ post('loaded'); });

    var prevConfig = window.PagedConfig || {};
    var prevAfter = prevConfig.after;
    window.__marcoExportLayoutPosted = false;
    window.PagedConfig = Object.assign({}, prevConfig, {
        after: function(flow){
            try { if (typeof prevAfter === 'function') prevAfter(flow); } catch(_){}
            if (!window.__marcoExportLayoutPosted) {
                window.__marcoExportLayoutPosted = true;
                post('layout_done');
            }
        }
    });

    if (window.Paged && typeof window.Paged.registerHandlers === 'function' && window.Paged.Handler) {
        try {
            class MarcoExportLifecycleHandler extends window.Paged.Handler {
                afterRendered(){
                    if (!window.__marcoExportLayoutPosted) {
                        window.__marcoExportLayoutPosted = true;
                        post('layout_done');
                    }
                }
            }
            window.Paged.registerHandlers(MarcoExportLifecycleHandler);
        } catch(_){}
    }

    // Safety net: if paged.js never fires `after`, declare the layout done
    // after the same 30 s budget the pre-existing Windows fallback used.
    setTimeout(function(){
        if (!window.__marcoExportLayoutPosted) {
            window.__marcoExportLayoutPosted = true;
            post('layout_done');
        }
    }, 30000);

    window.addEventListener('error', function(e){
        try { post('error', encodeURIComponent((e && e.message) || 'unknown')); } catch(_){}
    });
})();
</script>"#;

/// Inject [`LIFECYCLE_BRIDGE_JS`] into an HTML document, before `</head>` if
/// possible, otherwise before `</body>`.
pub fn inject_lifecycle_bridge(html: &str) -> String {
    if html.contains("id=\"marco-export-lifecycle\"") {
        return html.to_string();
    }
    if let Some(pos) = html.find("</head>") {
        let mut out = String::with_capacity(html.len() + LIFECYCLE_BRIDGE_JS.len());
        out.push_str(&html[..pos]);
        out.push_str(LIFECYCLE_BRIDGE_JS);
        out.push_str(&html[pos..]);
        return out;
    }
    if let Some(pos) = html.find("</body>") {
        let mut out = String::with_capacity(html.len() + LIFECYCLE_BRIDGE_JS.len());
        out.push_str(&html[..pos]);
        out.push_str(LIFECYCLE_BRIDGE_JS);
        out.push_str(&html[pos..]);
        return out;
    }
    format!("{}{}", html, LIFECYCLE_BRIDGE_JS)
}

/// Build a `<script>` payload that injects the print-export CSS and posts
/// `marco_export:css_applied` after the next animation frame so the pipeline
/// knows the repaint has happened.
pub fn build_inject_css_js(css: &str) -> String {
    let css_lit = js_string_literal(css);
    format!(
        r#"(function(){{
    try {{
        var existing = document.getElementById('marco-dynamic-export-css');
        if (existing) {{ existing.parentNode.removeChild(existing); }}
        var style = document.createElement('style');
        style.id = 'marco-dynamic-export-css';
        style.appendChild(document.createTextNode({css}));
        (document.head || document.documentElement).appendChild(style);
        requestAnimationFrame(function(){{
            requestAnimationFrame(function(){{
                if (window.__marcoExportPost) window.__marcoExportPost('css_applied');
            }});
        }});
    }} catch (e) {{
        if (window.__marcoExportPost) window.__marcoExportPost('error', encodeURIComponent(String(e)));
    }}
}})();"#,
        css = css_lit,
    )
}

/// Encode an arbitrary string as a JavaScript double-quoted string literal.
/// Escapes `<` to `\u003c` so embedded `</style>` cannot terminate a script.
fn js_string_literal(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Platform backend trait
// ─────────────────────────────────────────────────────────────────────────────

/// Boxed future shorthand for trait methods.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

/// Methods the [`run_export`] state machine needs from the host platform.
///
/// Each platform supplies its own implementation:
/// * Linux: drives `webkit6::WebView` via `notify::title` events and
///   `webkit6::PrintOperation` for PDF output.
/// * Windows: drives `wry::WebView` via `with_ipc_handler` events and
///   `ICoreWebView2_7::PrintToPdf` for PDF output.
pub trait PlatformExportBackend {
    /// Load the patched export HTML into the live preview WebView.
    fn load_export_html(&self, html: &str, base_uri: Option<&str>) -> Result<(), ExportError>;

    /// Evaluate a fire-and-forget JS snippet inside the export WebView.
    fn evaluate(&self, js: &str) -> Result<(), ExportError>;

    /// Wait for the next lifecycle event posted by the bridge JS, honouring
    /// the cancellation token and the per-phase timeout (derived from
    /// `phase.budget()`).  On timeout the error carries the phase label so
    /// the UI can show which step stalled.
    fn wait_for_event<'a>(
        &'a self,
        phase: ExportPhase,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<LifecycleEvent, ExportError>>;

    /// Drive the platform-native PDF exporter against the currently loaded
    /// (and now print-CSS-styled) WebView contents.
    fn print_to_pdf<'a>(
        &'a self,
        request: &'a ExportRequest,
    ) -> BoxFuture<'a, Result<(), ExportError>>;

    /// Restore the previously-displayed live preview HTML so the user does
    /// not see the export-styled document after the pipeline returns.
    fn restore_live_html(&self) -> Result<(), ExportError>;
}

// ─────────────────────────────────────────────────────────────────────────────
// State machine
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level entry point. Runs the unified pipeline against a platform
/// backend and returns once the output file is written or an error occurs.
///
/// The caller is responsible for showing an [`ExportingDialog`] and wiring
/// its X-button to [`CancelToken::cancel`].
///
/// [`ExportingDialog`]: crate::ui::dialogs::exporting::ExportingDialog
pub async fn run_export(
    backend: &dyn PlatformExportBackend,
    request: ExportRequest,
    progress: &dyn ProgressReporter,
    cancel: &CancelToken,
) -> Result<(), ExportError> {
    // [`run_export`] drives the live WebView through paged.js and is PDF-only.
    // HTML export uses the static-wrap composer ([`run_static_html_export`]).
    debug_assert_eq!(
        request.format,
        ExportFormat::Pdf,
        "run_export is PDF-only; use run_static_html_export for HTML"
    );
    if request.format != ExportFormat::Pdf {
        return Err(ExportError::Backend(
            "run_export only supports PDF; use run_static_html_export for HTML".into(),
        ));
    }

    progress.set_phase(ExportPhase::Preparing);
    cancel_check(cancel)?;

    // 1. Wrap the body using the same canonical helper the live preview
    //    uses, then inject the lifecycle bridge so the WebView posts
    //    `marco_export:*` events as it loads / paginates.
    let page_opts = marco_core::render::PageViewOptions {
        paged_js_source: pagedjs::PAGED_POLYFILL_JS,
        paper: &request.paper,
        orientation: &request.orientation,
        margin_mm: request.margin_mm,
        show_page_numbers: request.show_page_numbers,
        wheel_js: "",
        columns_per_row: 1,
        for_export: false,
        title: &request.title,
        standalone_export: false,
    };
    let wrapped = viewer_backend::wrap_html_document_paged(
        &request.html_body,
        &request.theme_css,
        &request.theme_class,
        None,
        &page_opts,
    );
    let patched = inject_lifecycle_bridge(&wrapped);

    // 2. Load + wait for paged.js readiness.
    progress.set_phase(ExportPhase::Loading);
    backend.load_export_html(&patched, request.base_uri.as_deref())?;
    expect_event(backend, ExportPhase::Loading, cancel).await?;

    progress.set_phase(ExportPhase::Paginating);
    expect_event(backend, ExportPhase::Paginating, cancel).await?;

    // 3. Generate the PDF. Restore happens regardless of outcome.
    let result = write_pdf(backend, &request, progress, cancel).await;

    progress.set_phase(ExportPhase::RestoringPreview);
    if let Err(e) = backend.restore_live_html() {
        log::warn!("[export_pipeline] restore_live_html failed: {}", e);
    }
    progress.set_phase(ExportPhase::Done);
    result
}

fn cancel_check(cancel: &CancelToken) -> Result<(), ExportError> {
    if cancel.is_cancelled() {
        Err(ExportError::Cancelled)
    } else {
        Ok(())
    }
}

/// Wait until the lifecycle event matching `phase` arrives. Out-of-order
/// duplicate events (e.g. paged.js firing `layout_done` twice) are tolerated.
async fn expect_event(
    backend: &dyn PlatformExportBackend,
    phase: ExportPhase,
    cancel: &CancelToken,
) -> Result<(), ExportError> {
    loop {
        let event = backend.wait_for_event(phase, cancel).await?;
        match (phase, &event) {
            (_, LifecycleEvent::Error(msg)) => return Err(ExportError::Backend(msg.clone())),
            (ExportPhase::Loading, LifecycleEvent::Loaded) => return Ok(()),
            (ExportPhase::Paginating, LifecycleEvent::LayoutDone) => return Ok(()),
            (ExportPhase::ApplyingPrintCss, LifecycleEvent::CssApplied) => return Ok(()),
            // Duplicate / out-of-phase events are ignored.
            _ => continue,
        }
    }
}

async fn write_pdf(
    backend: &dyn PlatformExportBackend,
    request: &ExportRequest,
    progress: &dyn ProgressReporter,
    cancel: &CancelToken,
) -> Result<(), ExportError> {
    progress.set_phase(ExportPhase::ApplyingPrintCss);
    let css = marco_shared::logic::print_css::make_print_export_css(
        &request.paper,
        &request.orientation,
        request.dark_mode,
    );
    backend.evaluate(&build_inject_css_js(&css))?;
    expect_event(backend, ExportPhase::ApplyingPrintCss, cancel).await?;

    progress.set_phase(ExportPhase::WritingOutput);
    backend.print_to_pdf(request).await?;

    let meta =
        std::fs::metadata(&request.output_path).map_err(|e| ExportError::Io(e.to_string()))?;
    if meta.len() < 64 {
        return Err(ExportError::EmptyOutput);
    }
    Ok(())
}

// ──────────────────────────────────────────────────────────────────────────────────
// Static-wrap HTML export (no WebView)
// ──────────────────────────────────────────────────────────────────────────────────

/// Build the standalone HTML document (no WebView round-trip) from an
/// [`ExportRequest`].
///
/// Branches on `request.paper`:
/// * `"none"` → plain document via [`viewer_backend::wrap_html_document`] with
///   the chosen `<title>` spliced in (the helper has no title parameter).
/// * any other value → paginated document via
///   [`viewer_backend::wrap_html_document_paged`] with `standalone_export: true`
///   so the embedded JS uses the file-export integration block (no WebKit hooks).
pub fn compose_static_html_export(request: &ExportRequest) -> String {
    if request.paper.eq_ignore_ascii_case("none") {
        let title_escaped = request
            .title
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        let raw = viewer_backend::wrap_html_document(
            &request.html_body,
            &request.theme_css,
            &request.theme_class,
            None,
        );
        if title_escaped.is_empty() {
            raw
        } else {
            raw.replacen(
                "<meta charset=",
                &format!("<title>{}</title>\n        <meta charset=", title_escaped),
                1,
            )
        }
    } else {
        let page_opts = marco_core::render::PageViewOptions {
            paged_js_source: pagedjs::PAGED_POLYFILL_JS,
            paper: &request.paper,
            orientation: &request.orientation,
            margin_mm: request.margin_mm,
            show_page_numbers: request.show_page_numbers,
            wheel_js: "",
            columns_per_row: 1,
            for_export: false,
            title: &request.title,
            standalone_export: true,
        };
        viewer_backend::wrap_html_document_paged(
            &request.html_body,
            &request.theme_css,
            &request.theme_class,
            None,
            &page_opts,
        )
    }
}

/// Async orchestrator for static-wrap HTML export.
///
/// Provides the same UX shape as [`run_export`] (phase reporting, cancel
/// token) without requiring a [`PlatformExportBackend`] — because static
/// HTML composition does not need the live WebView.
///
/// Phases: `Preparing` → (cancel check) → `WritingOutput` → `Done`.
pub async fn run_static_html_export(
    request: ExportRequest,
    progress: &dyn ProgressReporter,
    cancel: &CancelToken,
) -> Result<(), ExportError> {
    progress.set_phase(ExportPhase::Preparing);
    cancel_check(cancel)?;
    let full_html = compose_static_html_export(&request);

    // Yield once so the dialog can repaint between Preparing and WritingOutput.
    gtk4::glib::timeout_future(Duration::from_millis(20)).await;
    cancel_check(cancel)?;

    progress.set_phase(ExportPhase::WritingOutput);
    if let Some(parent) = request.output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ExportError::Io(e.to_string()))?;
    }
    std::fs::write(&request.output_path, full_html.as_bytes())
        .map_err(|e| ExportError::Io(e.to_string()))?;

    progress.set_phase(ExportPhase::Done);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Platform backend stubs
//
// Phase 1 ships compile-clean shells. Phases 2 / 3 will replace the
// `unimplemented!()` bodies with the real WebKit / WebView2 plumbing.
// ─────────────────────────────────────────────────────────────────────────────

/// Linux (WebKit6) export backend. Holds a clone of the live preview WebView
/// so the pipeline can drive the same widget the user is currently looking at.
#[cfg(target_os = "linux")]
pub struct LinuxExportBackend {
    /// Live preview WebView — the pipeline loads export HTML into it and
    /// restores the cached body on the way out.
    pub webview: PlatformWebView,
    /// HTML to reload into the WebView in [`PlatformExportBackend::restore_live_html`].
    pub saved_live_html: String,
    /// Set to `true` once `load_export_html` has actually navigated the
    /// preview WebView away from the live document. If the pipeline aborts
    /// before that point, `restore_live_html` skips the costly re-navigation.
    did_load_export: std::cell::Cell<bool>,
}

#[cfg(target_os = "linux")]
impl LinuxExportBackend {
    pub fn new(webview: PlatformWebView, saved_live_html: String) -> Self {
        Self {
            webview,
            saved_live_html,
            did_load_export: std::cell::Cell::new(false),
        }
    }
}

#[cfg(target_os = "linux")]
impl PlatformExportBackend for LinuxExportBackend {
    fn load_export_html(&self, html: &str, base_uri: Option<&str>) -> Result<(), ExportError> {
        viewer_backend::load_html_when_ready(
            &self.webview,
            html.to_string(),
            base_uri.map(str::to_owned),
        );
        self.did_load_export.set(true);
        Ok(())
    }

    fn evaluate(&self, js: &str) -> Result<(), ExportError> {
        viewer_backend::evaluate_javascript(&self.webview, js);
        Ok(())
    }

    fn wait_for_event<'a>(
        &'a self,
        phase: ExportPhase,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<LifecycleEvent, ExportError>> {
        use glib::object::ObjectExt;
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::time::Instant;
        use webkit6::prelude::WebViewExt;

        // Shared one-shot slot — the title-notify handler writes here.
        let slot: Rc<RefCell<Option<LifecycleEvent>>> = Rc::new(RefCell::new(None));
        let slot_write = slot.clone();
        // notify::title is the same channel `mc_paged_ready` already uses
        // for the live preview; we filter both `marco_export:*` (new bridge)
        // and `mc_paged_ready` as the paged-ready signal from marco-core v1.1.0.
        let handler_id = self.webview.connect_notify_local(Some("title"), {
            let slot_write = slot_write.clone();
            move |wv: &webkit6::WebView, _| {
                let Some(t) = wv.title() else { return };
                let title = t.as_str();
                let evt = if let Some(e) = LifecycleEvent::parse(title) {
                    Some(e)
                } else if title == "mc_paged_ready" {
                    // Paged-ready signal from marco-core v1.1.0 — treated as LayoutDone.
                    Some(LifecycleEvent::LayoutDone)
                } else {
                    None
                };
                if let Some(evt) = evt {
                    let mut borrow = slot_write.borrow_mut();
                    if borrow.is_none() {
                        *borrow = Some(evt);
                    }
                }
            }
        });

        Box::pin(async move {
            let deadline = Instant::now() + phase.budget();
            // See `WindowsExportBackend::wait_for_event` for the poll-interval
            // rationale; same trade-off applies here.
            let poll = Duration::from_millis(100);

            let result = loop {
                if cancel.is_cancelled() {
                    break Err(ExportError::Cancelled);
                }
                if let Some(evt) = slot.borrow_mut().take() {
                    break Ok(evt);
                }
                if Instant::now() >= deadline {
                    break Err(ExportError::Timeout(phase));
                }
                gtk4::glib::timeout_future(poll).await;
            };

            // Always disconnect to avoid stale signals from later page loads.
            self.webview.disconnect(handler_id);
            result
        })
    }

    fn print_to_pdf<'a>(
        &'a self,
        request: &'a ExportRequest,
    ) -> BoxFuture<'a, Result<(), ExportError>> {
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::time::Instant;

        Box::pin(async move {
            // Set document.title so WebKit's PDF backend uses the user-chosen
            // title as PDF metadata. The legacy print_driver::inject_export_css
            // bundled this with the CSS injection; the unified pipeline keeps
            // CSS injection cross-platform and applies the title separately.
            if !request.title.is_empty() {
                let title_lit = js_string_literal(&request.title);
                viewer_backend::evaluate_javascript(
                    &self.webview,
                    &format!("document.title = {};", title_lit),
                );
                // Yield so the title update is processed before print starts.
                gtk4::glib::timeout_future(Duration::from_millis(50)).await;
            }

            // Bridge the on_done callback into a polled slot.
            let slot: Rc<RefCell<Option<Result<(), String>>>> = Rc::new(RefCell::new(None));
            let slot_write = slot.clone();
            crate::components::viewer::print_driver::export_to_pdf(
                &self.webview,
                &request.output_path,
                &request.paper,
                &request.orientation,
                move |result| {
                    let mut borrow = slot_write.borrow_mut();
                    if borrow.is_none() {
                        *borrow = Some(result);
                    }
                },
            );

            // Generous deadline — large documents can take many seconds.
            let deadline = Instant::now() + ExportPhase::WritingOutput.budget();
            let poll = Duration::from_millis(50);
            loop {
                if let Some(r) = slot.borrow_mut().take() {
                    return r.map_err(ExportError::Backend);
                }
                if Instant::now() >= deadline {
                    return Err(ExportError::Timeout(ExportPhase::WritingOutput));
                }
                gtk4::glib::timeout_future(poll).await;
            }
        })
    }

    fn restore_live_html(&self) -> Result<(), ExportError> {
        // Drop the dynamic export CSS first so the cached live HTML doesn't
        // re-paint with print styles before its own theme stylesheet wins.
        crate::components::viewer::print_driver::remove_export_css(&self.webview);
        if !self.did_load_export.get() {
            // Pipeline aborted before navigating away — nothing to restore.
            return Ok(());
        }
        if !self.saved_live_html.is_empty() {
            viewer_backend::load_html_when_ready(&self.webview, self.saved_live_html.clone(), None);
        }
        Ok(())
    }
}

/// Windows (wry / WebView2) export backend.
#[cfg(target_os = "windows")]
pub struct WindowsExportBackend {
    pub webview: PlatformWebView,
    pub saved_live_html: String,
    /// Set to `true` once `load_export_html` has actually navigated the
    /// preview WebView away from the live document. If the pipeline aborts
    /// before that point, `restore_live_html` skips the costly re-navigation.
    did_load_export: std::cell::Cell<bool>,
}

#[cfg(target_os = "windows")]
impl WindowsExportBackend {
    pub fn new(webview: PlatformWebView, saved_live_html: String) -> Self {
        Self {
            webview,
            saved_live_html,
            did_load_export: std::cell::Cell::new(false),
        }
    }
}

#[cfg(target_os = "windows")]
impl PlatformExportBackend for WindowsExportBackend {
    fn load_export_html(&self, html: &str, base_uri: Option<&str>) -> Result<(), ExportError> {
        self.webview.load_html_with_base(html, base_uri);
        self.did_load_export.set(true);
        Ok(())
    }

    fn evaluate(&self, js: &str) -> Result<(), ExportError> {
        self.webview.evaluate_script(js);
        Ok(())
    }

    fn wait_for_event<'a>(
        &'a self,
        phase: ExportPhase,
        cancel: &'a CancelToken,
    ) -> BoxFuture<'a, Result<LifecycleEvent, ExportError>> {
        use std::cell::RefCell;
        use std::rc::Rc;
        use std::time::Instant;

        // Shared one-shot slot: the IPC callback writes here, the poll loop reads.
        let slot: Rc<RefCell<Option<LifecycleEvent>>> = Rc::new(RefCell::new(None));
        let slot_write = slot.clone();
        self.webview.set_export_event_listener(move |msg: String| {
            if let Some(evt) = LifecycleEvent::parse(&msg) {
                let mut borrow = slot_write.borrow_mut();
                if borrow.is_none() {
                    *borrow = Some(evt);
                }
            }
        });

        Box::pin(async move {
            let budget = phase.budget();
            let deadline = Instant::now() + budget;
            // 100 ms poll interval keeps CPU wakeups well under 10/sec for the
            // longest phase (Paginating, 30 s budget) while remaining
            // imperceptible against paged.js layout times. Lifecycle events
            // typically resolve in well under 200 ms, so the worst-case extra
            // latency on the happy path is one poll tick.
            let poll = Duration::from_millis(100);

            loop {
                if cancel.is_cancelled() {
                    self.webview.clear_export_event_listener();
                    return Err(ExportError::Cancelled);
                }
                if let Some(evt) = slot.borrow_mut().take() {
                    self.webview.clear_export_event_listener();
                    return Ok(evt);
                }
                if Instant::now() >= deadline {
                    self.webview.clear_export_event_listener();
                    return Err(ExportError::Timeout(phase));
                }
                gtk4::glib::timeout_future(poll).await;
            }
        })
    }

    fn print_to_pdf<'a>(
        &'a self,
        request: &'a ExportRequest,
    ) -> BoxFuture<'a, Result<(), ExportError>> {
        Box::pin(async move {
            // Yield once so GTK can repaint the dialog label before the COM
            // PrintToPdf call starts (which can be tens of seconds long for
            // large documents).
            //
            // Threading note: `wry_print_to_pdf::print_to_pdf` is *synchronous*
            // but uses `webview2_com::wait_with_pump`, which pumps the Win32
            // message queue. GTK on Windows is built on top of that same
            // queue, so glib timers / repaints continue to fire and the
            // "Exporting…" dialog stays animated. The COM object is STA-bound
            // to the GTK main thread, so moving this call to a worker thread
            // is *not* an option.
            //
            // Cancellation note: there is no `PrintToPdf.Cancel` API on
            // `ICoreWebView2_7`. The user's X-button can only abort the
            // surrounding pipeline (it is checked again before / after this
            // call). Mid-flight cancel is a known WebView2 limitation.
            gtk4::glib::timeout_future(Duration::from_millis(50)).await;
            if request.output_path.as_os_str().is_empty() {
                return Err(ExportError::Io("empty output path".into()));
            }
            self.webview
                .print_to_pdf(
                    &request.output_path,
                    &request.paper,
                    &request.orientation,
                    request.margin_mm,
                )
                .map_err(ExportError::Backend)
        })
    }

    fn restore_live_html(&self) -> Result<(), ExportError> {
        if !self.did_load_export.get() {
            // The pipeline aborted before navigating away from the live
            // document; the WebView is already showing the user's content,
            // so a second `load_html_with_base` would only cause a flicker.
            return Ok(());
        }
        if !self.saved_live_html.is_empty() {
            self.webview
                .load_html_with_base(&self.saved_live_html, None);
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_lifecycle_event_parses_known_payloads() {
        assert_eq!(
            LifecycleEvent::parse("marco_export:loaded"),
            Some(LifecycleEvent::Loaded)
        );
        assert_eq!(
            LifecycleEvent::parse("marco_export:layout_done"),
            Some(LifecycleEvent::LayoutDone)
        );
        assert_eq!(
            LifecycleEvent::parse("marco_export:css_applied"),
            Some(LifecycleEvent::CssApplied)
        );
        match LifecycleEvent::parse("marco_export:error:boom%20boom") {
            Some(LifecycleEvent::Error(msg)) => assert_eq!(msg, "boom%20boom"),
            other => panic!("expected Error, got {:?}", other),
        }
    }

    #[test]
    fn smoke_lifecycle_event_ignores_other_prefixes() {
        // The bridge must coexist with `marco_scroll:` and `mc_paged_ready`
        // without consuming them.
        assert_eq!(LifecycleEvent::parse("marco_scroll:0.5"), None);
        assert_eq!(LifecycleEvent::parse("mc_paged_ready"), None);
        assert_eq!(LifecycleEvent::parse("loaded"), None);
        assert_eq!(LifecycleEvent::parse(""), None);
    }

    #[test]
    fn smoke_inject_lifecycle_bridge_into_head() {
        let html = "<html><head><title>x</title></head><body>y</body></html>";
        let out = inject_lifecycle_bridge(html);
        assert!(out.contains("id=\"marco-export-lifecycle\""));
        assert!(out.contains("marco_export:"));
        let head_close = out.find("</head>").unwrap();
        let bridge_pos = out.find("marco-export-lifecycle").unwrap();
        assert!(bridge_pos < head_close);
    }

    #[test]
    fn smoke_inject_lifecycle_bridge_is_idempotent() {
        let html = "<html><head></head><body><script id=\"marco-export-lifecycle\"></script></body></html>";
        let out = inject_lifecycle_bridge(html);
        assert_eq!(out, html);
    }

    #[test]
    fn smoke_inject_lifecycle_bridge_falls_back_to_body() {
        let html = "<body>only-body</body>";
        let out = inject_lifecycle_bridge(html);
        assert!(out.contains("marco-export-lifecycle"));
        let body_close = out.find("</body>").unwrap();
        let bridge_pos = out.find("marco-export-lifecycle").unwrap();
        assert!(bridge_pos < body_close);
    }

    #[test]
    fn smoke_export_phase_labels_are_user_friendly() {
        assert_eq!(ExportPhase::Loading.label(), "Loading paged.js…");
        assert_eq!(ExportPhase::Paginating.label(), "Paginating pages…");
        assert!(ExportPhase::ApplyingPrintCss.budget() <= Duration::from_secs(10));
    }

    #[test]
    fn smoke_cancel_token_is_threadsafe_and_clonable() {
        let token = CancelToken::new();
        assert!(!token.is_cancelled());
        let cloned = token.clone();
        cloned.cancel();
        assert!(token.is_cancelled());
        assert!(cloned.is_cancelled());
    }

    #[test]
    fn smoke_build_inject_css_js_escapes_and_posts_event() {
        let js = build_inject_css_js("body{color:red;}\n@page{}");
        assert!(js.contains("marco-dynamic-export-css"));
        assert!(js.contains("css_applied"));
        assert!(js.contains("requestAnimationFrame"));
        // Make sure </style> in CSS is escaped so the script tag is not closed early.
        let js_with_style = build_inject_css_js("body{}</style><script>alert(1)</script>");
        assert!(!js_with_style.contains("</style>"));
        assert!(js_with_style.contains("\\u003c"));
    }

    #[test]
    fn smoke_export_error_display_includes_phase_label() {
        let err = ExportError::Timeout(ExportPhase::Paginating);
        let msg = format!("{}", err);
        assert!(msg.contains("Paginating"));
    }

    #[test]
    fn smoke_noop_reporter_is_send_safe() {
        // Compile-time check that a NoopReporter satisfies the trait.
        fn assert_reporter<R: ProgressReporter>(_: &R) {}
        assert_reporter(&NoopReporter);
        NoopReporter.set_phase(ExportPhase::Loading);
    }

    #[test]
    fn smoke_compose_static_html_paper_none_injects_title() {
        let req = ExportRequest {
            format: ExportFormat::Html,
            html_body: "<p>hi</p>".into(),
            theme_css: "/* css */".into(),
            theme_class: "theme-light".into(),
            paper: "none".into(),
            orientation: "portrait".into(),
            margin_mm: 10,
            show_page_numbers: false,
            title: "Hello & <World>".into(),
            output_path: std::path::PathBuf::from("/tmp/never-written.html"),
            base_uri: None,
            dark_mode: false,
        };
        let html = compose_static_html_export(&req);
        // Title is HTML-escaped and inserted before <meta charset=>.
        assert!(html.contains("<title>Hello &amp; &lt;World&gt;</title>"));
        assert!(html.contains("<p>hi</p>"));
    }

    #[test]
    fn smoke_compose_static_html_paged_uses_paper_size() {
        let req = ExportRequest {
            format: ExportFormat::Html,
            html_body: "<p>x</p>".into(),
            theme_css: String::new(),
            theme_class: "theme-light".into(),
            paper: "A4".into(),
            orientation: "portrait".into(),
            margin_mm: 15,
            show_page_numbers: true,
            title: "T".into(),
            output_path: std::path::PathBuf::from("/tmp/never.html"),
            base_uri: None,
            dark_mode: false,
        };
        let html = compose_static_html_export(&req);
        // Paged.js polyfill must be embedded.
        assert!(html.contains("Paged"));
        assert!(html.contains("<p>x</p>"));
    }
}
