/// Build the canonical `@media print` CSS block used by print/PDF export.
///
/// This function is shared across platform backends to keep print fidelity
/// and pagination behavior consistent between Linux and Windows.
pub fn make_print_export_css(paper: &str, orientation: &str, dark_mode: bool) -> String {
    // `size` directive: include only when paper is known. For the live print
    // dialog the user's paper choice takes precedence; for silent PDF export
    // we supply the exact size so the PDF renderer matches paged.js layout.
    let size_rule = if paper.is_empty() {
        String::new()
    } else {
        format!("    size: {} {} !important;\n", paper, orientation)
    };

    // In dark-mode exports the page-box background is dark. If there is any
    // sub-pixel gap between the paged.js box and the PDF page boundary the
    // html/body background shows through. Matching that background to the
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
   * page breaks here. paged.js already performs fragmentation; adding
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

#[cfg(test)]
mod tests {
    use super::make_print_export_css;

    #[test]
    fn smoke_print_css_contains_required_rules() {
        let css = make_print_export_css("A4", "portrait", false);
        assert!(css.contains("@page"));
        assert!(css.contains("margin: 0 !important;"));
        assert!(css.contains("opacity: 1 !important;"));
        assert!(css.contains("transition: none !important;"));
        assert!(css.contains(".pagedjs_pages"));
        assert!(css.contains("gap: 0 !important;"));
    }
}
