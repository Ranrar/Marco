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
/// Delegates to the shared canonical implementation in `marco-shared` so Linux
/// and Windows export paths stay in sync.
pub fn make_print_export_css(paper: &str, orientation: &str, dark_mode: bool) -> String {
    marco_shared::logic::print_css::make_print_export_css(paper, orientation, dark_mode)
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
