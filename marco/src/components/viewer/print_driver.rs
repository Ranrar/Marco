//! Print and PDF export driver (Linux / WebKit6 only).
//!
//! Wraps `webkit6::PrintOperation` with helpers for native print dialogs and
//! silent PDF export.  All functions must be called on the GTK main thread
//! (`PrintOperation` is `!Send + !Sync`).

/// Map a paged.js paper name to the GTK/CUPS PPD media name so `PageSetup`
/// gets the correct physical dimensions.
fn ppd_paper_name(paper: &str) -> &'static str {
    match paper.to_uppercase().as_str() {
        "A3" => "iso_a3_297x420mm",
        "A4" => "iso_a4_210x297mm",
        "A5" => "iso_a5_148x210mm",
        "LETTER" => "na_letter_8.5x11in",
        "LEGAL" => "na_legal_8.5x14in",
        "B5" => "iso_b5_176x250mm",
        _ => "iso_a4_210x297mm",
    }
}

/// Build the `@media print` CSS block for PDF export.
///
/// * `paper`       - e.g. `"A4"`.  Empty string = omit `size` (print dialog
///                  user picks paper).
/// * `orientation` - `"portrait"` or `"landscape"`.
///
/// ### What each rule does (per paged.js docs)
///
/// paged.js lays out content into `.pagedjs_page` boxes that are exactly the
/// paper size. When the browser's PDF renderer prints those boxes it must:
///
/// 1. Use the *same* paper dimensions so each box fills exactly one PDF page.
///    → `@page { size: <paper> <orientation>; margin: 0 }`
///    (`margin: 0` because paged.js visual margins are already baked in.)
///
/// 2. Avoid adding extra forced breaks on `.pagedjs_page`.
///    paged.js already performs fragmentation; extra break rules can stack and
///    intermittently produce blank pages.
///
/// 3. Remove all visual gaps between boxes (flex gap, padding, body BG).
///    → Reset `.pagedjs_pages` and body.
pub fn make_print_export_css(paper: &str, orientation: &str, dark_mode: bool) -> String {
    // `size` directive: include only when paper is known.  For the live print
    // dialog the user's paper choice takes precedence; for silent PDF export
    // we supply the exact size so the PDF renderer matches paged.js layout.
    let size_rule = if paper.is_empty() {
        String::new()
    } else {
        format!("    size: {} {} !important;\n", paper, orientation)
    };

    // In dark-mode exports the page-box background is dark.  If there is any
    // sub-pixel gap between the paged.js box and the PDF page boundary the
    // html/body background shows through.  Matching that background to the
    // dark paper avoids a thin white sliver at the edge.
    let body_bg = if dark_mode { "#111111" } else { "white" };

    format!(
        r#"@media print {{
  /* ── Browser page setup ─────────────────────────────────────────────────
   * margin: 0 so paged.js visual margins fill the full PDF page.
   * size must match the paged.js @page layout rule exactly.               */
  @page {{
{size}    margin: 0 !important;
  }}

  /* Strip the "desk" (grey viewport around page boxes).                   *
   * Use a background colour that matches the paper so sub-pixel gaps      *
   * between the page box edge and the PDF page boundary are invisible.    *
   * opacity: 1 overrides paged.js's fade-in transition mid-animation so   *
   * the PDF is never captured while the body is still semi-transparent.   *
   * transition/animation: none prevents the 120 ms opacity ease-in from   *
   * bleeding into the print render.                                       */
  html, body {{
    background: {body_bg} !important;
    margin: 0 !important;
    padding: 0 !important;
        height: auto !important;
        min-height: 0 !important;
        max-height: none !important;
        overflow: visible !important;
    opacity: 1 !important;
    transition: none !important;
    animation: none !important;
  }}

  /* ── paged.js outer container ───────────────────────────────────────────
   * Switch from flex (screen) to block so page-break props apply.
   * Remove all padding / gap so no white ribbon appears between pages.    */
  .pagedjs_pages {{
    display: block !important;
    padding: 0 !important;
    margin: 0 !important;
    gap: 0 !important;
        overflow: visible !important;
    background: transparent !important;
  }}

    /* ── Individual page boxes ──────────────────────────────────────────────
     * Keep each paged.js box clean and clipped, but do NOT force additional
     * page breaks here.  paged.js already performs fragmentation; adding
     * break-after/page-break-after at this layer can stack with other break
     * constraints and intermittently produce extra blank pages.              */
  .pagedjs_page {{
    display: block !important;
    margin: 0 !important;
    padding: 0 !important;
    box-shadow: none !important;
    outline: none !important;
    border: none !important;
    overflow: hidden !important;
        break-before: auto !important;
        break-after: auto !important;
        page-break-before: auto !important;
        page-break-after: auto !important;
        break-inside: auto !important;
        page-break-inside: auto !important;
  }}

    .pagedjs_page:last-child {{
        break-after: auto !important;
        page-break-after: auto !important;
    }}

  /* ── Page box grid (margin areas + content) ─────────────────────────── */
  .pagedjs_pagebox {{
    box-shadow: none !important;
    outline: none !important;
    width: 100% !important;
    height: 100% !important;
    box-sizing: border-box !important;
  }}
}}"#,
        size = size_rule,
        body_bg = body_bg,
    )
}

/// Open the native GTK print dialog for the given WebKit [`WebView`].
///
/// `paper` / `orientation` are used as the default `PageSetup` shown in the
/// dialog *and* injected into the print CSS so paged.js output maps cleanly
/// to PDF pages.  The user can still change paper in the dialog.
pub fn trigger_print_dialog(
    webview: &webkit6::WebView,
    parent: Option<&gtk4::Window>,
    paper: &str,
    orientation: &str,
    dark_mode: bool,
) {
    // Inject print-clean CSS (no title override needed for the interactive dialog).
    inject_export_css(webview, paper, orientation, "", dark_mode);

    let print_op = webkit6::PrintOperation::new(webview);

    // Set a default PageSetup so the dialog opens pre-configured for the
    // correct paper and zero printer margins (paged.js margins are visual).
    if !paper.is_empty() {
        if let Some(page_setup) = build_page_setup(paper, orientation) {
            print_op.set_page_setup(&page_setup);
        }
    }

    let _ = print_op.run_dialog(parent);
    remove_export_css(webview);
}

/// Silently export the WebView's rendered content to a PDF file.
///
/// `paper` / `orientation` must match the paged.js layout so the PDF renderer
/// uses the same page dimensions.  Pass empty strings to skip paper setup
/// (e.g. for plain non-paged content).
pub fn export_to_pdf<F>(
    webview: &webkit6::WebView,
    output_path: &std::path::Path,
    paper: &str,
    orientation: &str,
    on_done: F,
) where
    F: Fn(Result<(), String>) + 'static,
{
    use gtk4::{
        PrintSettings, PRINT_SETTINGS_OUTPUT_FILE_FORMAT, PRINT_SETTINGS_OUTPUT_URI,
        PRINT_SETTINGS_PRINTER,
    };
    use std::sync::Arc;

    let print_op = webkit6::PrintOperation::new(webview);
    let settings = PrintSettings::new();
    settings.set(PRINT_SETTINGS_OUTPUT_FILE_FORMAT, Some("pdf"));
    let uri = format!("file://{}", output_path.display());
    settings.set(PRINT_SETTINGS_OUTPUT_URI, Some(&uri));
    settings.set(PRINT_SETTINGS_PRINTER, Some("Print to File"));

    // Set orientation in PrintSettings so the CUPS PDF backend (and WebKit's
    // internal PS→PDF renderer) honours portrait / landscape.  Without this,
    // only PageSetup carries the orientation and some backends ignore it.
    if !orientation.is_empty() {
        let page_orient = if orientation.eq_ignore_ascii_case("landscape") {
            gtk4::PageOrientation::Landscape
        } else {
            gtk4::PageOrientation::Portrait
        };
        settings.set_orientation(page_orient);
    }

    print_op.set_print_settings(&settings);

    // Apply a PageSetup with the correct paper size and zero printer margins.
    // Without this, CUPS / the PS-to-PDF pipeline may add its own default
    // margins on top of paged.js's already-correct layout margins.
    if !paper.is_empty() {
        if let Some(page_setup) = build_page_setup(paper, orientation) {
            print_op.set_page_setup(&page_setup);
        }
    }

    let on_done = Arc::new(on_done);
    let on_done_fail = Arc::clone(&on_done);
    print_op.connect_finished(move |_| on_done(Ok(())));
    print_op.connect_failed(move |_, err| on_done_fail(Err(err.to_string())));
    print_op.print();
}

/// Build a [`gtk4::PageSetup`] with the given paper and zero printer margins.
///
/// Zero margins are correct because paged.js has already baked the visual
/// page margins into the `.pagedjs_page` box dimensions.  Any printer margin
/// here would add extra whitespace *outside* the paged.js paper boxes.
fn build_page_setup(paper: &str, orientation: &str) -> Option<gtk4::PageSetup> {
    let ppd = ppd_paper_name(paper);
    let paper_size = gtk4::PaperSize::new(Some(ppd));
    let page_setup = gtk4::PageSetup::new();
    page_setup.set_paper_size_and_default_margins(&paper_size);
    // Override margins back to zero after `set_paper_size_and_default_margins`
    // fills in the printer's suggested margins.
    page_setup.set_top_margin(0.0, gtk4::Unit::Mm);
    page_setup.set_bottom_margin(0.0, gtk4::Unit::Mm);
    page_setup.set_left_margin(0.0, gtk4::Unit::Mm);
    page_setup.set_right_margin(0.0, gtk4::Unit::Mm);
    let orient = if orientation.eq_ignore_ascii_case("landscape") {
        gtk4::PageOrientation::Landscape
    } else {
        gtk4::PageOrientation::Portrait
    };
    page_setup.set_orientation(orient);
    Some(page_setup)
}

/// Inject the print export CSS into the WebView via JavaScript.
///
/// `paper` / `orientation` are forwarded to [`make_print_export_css`] so the
/// `@page { size }` directive matches the paged.js layout.
pub fn inject_export_css(
    webview: &webkit6::WebView,
    paper: &str,
    orientation: &str,
    title: &str,
    dark_mode: bool,
) {
    use crate::components::viewer::backend::evaluate_javascript;

    let css = make_print_export_css(paper, orientation, dark_mode);
    // Escape the CSS for safe embedding inside a JS string literal.
    let css_escaped = css
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");

    // Escape the title for safe embedding in JS.
    let title_escaped = title
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");

    // Bundle the document.title update with the CSS injection so both reach the
    // WebKit JS engine in the same call, before print rendering begins.
    // WebKit uses document.title as the PDF document title metadata.
    let title_js = if title.is_empty() {
        String::new()
    } else {
        format!("  document.title = \"{}\";\n", title_escaped)
    };

    let js = format!(
        r#"(function(){{
{title}  var el=document.getElementById('marco-dynamic-export-css');
  if(!el){{el=document.createElement('style');el.id='marco-dynamic-export-css';document.head.appendChild(el);}}
  el.textContent="{css}";
}})();"#,
        title = title_js,
        css = css_escaped
    );
    evaluate_javascript(webview, &js);
}

/// Remove the dynamically injected export CSS element from the WebView.
pub fn remove_export_css(webview: &webkit6::WebView) {
    use crate::components::viewer::backend::evaluate_javascript;
    evaluate_javascript(
        webview,
        "(function(){var el=document.getElementById('marco-dynamic-export-css');if(el)el.remove();})()",
    );
}
