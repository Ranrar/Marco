//! SourceView5 native HoverProvider implementation.
//!
//! Replaces the custom EventControllerMotion + Popover approach with a proper
//! `GtkSourceHoverProvider` GObject subclass that integrates with SourceView5's
//! built-in hover infrastructure.

use crate::components::editor::ui::{
    diagnostic_at_offset, diagnostic_hover_markup, split_hover_content, RuntimeIntelligenceSettings,
};
use glib::subclass::prelude::ObjectSubclassIsExt;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;

const HOVER_POPOVER_WIDTH: i32 = 320;
const HOVER_POPOVER_HORIZONTAL_SAFE_PADDING: i32 = 8;

/// Parsed sections extracted from a hover body string.
///
/// Diagnostic bodies use prefixed lines ("Code: …", "About: …", "Fix: …").
/// Plain markdown hover bodies are treated as a single description block.
struct HoverBodyParts {
    /// Diagnostic code identifier (e.g. "MD001"). `None` for markdown hover.
    code: Option<String>,
    /// Short description or the full plain-text body for markdown hover.
    about: Option<String>,
    /// Fix suggestion. Only present for diagnostic hover.
    fix: Option<String>,
    /// `true` when body came from a diagnostic (structured format).
    is_diagnostic: bool,
}

impl HoverBodyParts {
    fn from_body(body: &str) -> Self {
        let mut code: Option<String> = None;
        let mut about: Option<String> = None;
        let mut fix: Option<String> = None;

        for line in body.lines() {
            if let Some(rest) = line.strip_prefix("Code: ") {
                code = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("About: ") {
                about = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("Fix: ") {
                fix = Some(rest.trim().to_string());
            }
        }

        let is_diagnostic = code.is_some();

        if !is_diagnostic {
            // Plain markdown hover — the whole trimmed body is the description.
            let trimmed = body.trim();
            about = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
        }

        Self {
            code,
            about,
            fix,
            is_diagnostic,
        }
    }
}

mod imp {
    use super::*;
    use glib::subclass::prelude::*;
    use sourceview5::subclass::hover_provider::HoverProviderImpl;

    #[derive(Default)]
    pub struct MarcoHoverProvider {
        pub(super) diagnostics: RefCell<Option<Rc<RefCell<Vec<marco_core::intelligence::Diagnostic>>>>>,
        pub(super) settings_fn: RefCell<Option<Rc<dyn Fn() -> RuntimeIntelligenceSettings>>>,
        // ── Popover widget refs ────────────────────────────────────────────────
        pub(super) hover_popover: RefCell<Option<gtk4::Popover>>,
        /// Big title in the header row.
        pub(super) hover_title_label: RefCell<Option<gtk4::Label>>,
        /// Diagnostic-code chip badge (top-right corner of header).
        pub(super) hover_code_chip: RefCell<Option<gtk4::Label>>,
        /// "About" section header label (hidden for plain markdown hover).
        pub(super) hover_about_header: RefCell<Option<gtk4::Label>>,
        /// Description body / plain-text body for markdown hover.
        pub(super) hover_about_body: RefCell<Option<gtk4::Label>>,
        /// Fix section box (hidden for plain markdown hover).
        pub(super) hover_fix_section: RefCell<Option<gtk4::Box>>,
        /// Fix body text label.
        pub(super) hover_fix_body: RefCell<Option<gtk4::Label>>,
        pub(super) last_signature: RefCell<Option<(usize, usize, String)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MarcoHoverProvider {
        const NAME: &'static str = "MarcoHoverProvider";
        type Type = super::MarcoHoverProvider;
        type Interfaces = (sourceview5::HoverProvider,);
    }

    impl ObjectImpl for MarcoHoverProvider {}

    impl HoverProviderImpl for MarcoHoverProvider {
        fn populate_future(
            &self,
            context: &sourceview5::HoverContext,
            display: &sourceview5::HoverDisplay,
        ) -> Pin<Box<dyn std::future::Future<Output = Result<(), glib::Error>> + 'static>> {
            let result = self.populate_sync(context, display);
            Box::pin(async move { result })
        }
    }

    impl MarcoHoverProvider {
        /// All hover computation is synchronous; we run it inline and wrap in an
        /// immediate future for the async HoverProvider interface.
        fn populate_sync(
            &self,
            context: &sourceview5::HoverContext,
            _display: &sourceview5::HoverDisplay,
        ) -> Result<(), glib::Error> {
            let no_content = || glib::Error::new(glib::FileError::Failed, "no hover content");

            let iter = context.iter().ok_or_else(no_content)?;
            let view = context.view();

            let buffer = context.buffer();
            let source = buffer
                .text(&buffer.start_iter(), &buffer.end_iter(), false)
                .to_string();

            let line = (iter.line() + 1) as usize;
            let column = (iter.line_offset() + 1) as usize;
            let char_offset = iter.offset().max(0) as usize;
            let byte_offset: usize = source
                .chars()
                .take(char_offset)
                .map(|c: char| c.len_utf8())
                .sum();

            // Read runtime settings
            let runtime_settings = {
                let settings_fn = self.settings_fn.borrow();
                settings_fn.as_ref().map(|f| f()).unwrap_or_default()
            };

            if !runtime_settings.markdown_intelligence_enabled {
                self.hide_hover_popover();
                return Err(no_content());
            }

            // Parse once and reuse the document for both hover info and span
            // analysis — avoids parsing the same source twice.
            let position = marco_core::parser::Position {
                line,
                column,
                offset: byte_offset,
            };
            let parsed_doc = marco_core::parser::parse(&source).ok();

            let diagnostic_candidate = if runtime_settings.diagnostics_hover_enabled {
                let diagnostics_ref = self.diagnostics.borrow();
                if let Some(diags) = diagnostics_ref.as_ref() {
                    diagnostic_at_offset(&diags.borrow(), byte_offset)
                } else {
                    None
                }
            } else {
                None
            };

            // Gather both candidates independently so we can compare them.
            // When a wide diagnostic span (e.g. multi-line or document-level
            // check) covers positions that also have a specific markdown element
            // OR plain text, the local node is more relevant. We choose whichever
            // candidate has the tighter span; equal/no markdown → diagnostic wins.
            let markdown_candidate = if runtime_settings.markdown_hover_enabled {
                parsed_doc
                    .as_ref()
                    .and_then(|doc| marco_core::intelligence::get_hover_info(position, doc))
            } else {
                None
            };

            match (diagnostic_candidate, markdown_candidate) {
                (Some(diagnostic), Some(info)) => {
                    let diag_span_len = diagnostic
                        .span
                        .end
                        .offset
                        .saturating_sub(diagnostic.span.start.offset);
                    let md_span_len = info
                        .range
                        .map(|s| s.end.offset.saturating_sub(s.start.offset))
                        .unwrap_or(usize::MAX);
                    if md_span_len < diag_span_len {
                        // Markdown element is more specific — show markdown hover.
                        let (start, end) = info
                            .range
                            .map(|s| (s.start.offset, s.end.offset))
                            .unwrap_or((usize::MAX, usize::MAX));
                        let signature = (start, end, info.contents.clone());
                        let (title, body) = split_hover_content(&info.contents);
                        self.show_hover_popover(&view, &iter, &title, &body, signature);
                    } else {
                        // Diagnostic is equally or more specific — show diagnostic hover.
                        let (title, body, signature) = diagnostic_hover_markup(&diagnostic);
                        self.show_hover_popover(&view, &iter, &title, &body, signature);
                    }
                    Ok(())
                }
                (Some(diagnostic), None) => {
                    // Even without a semantic hover, the cursor might be on a plain
                    // text node (e.g. a Paragraph or Text leaf) whose tightest AST
                    // span is smaller than the diagnostic's wide span. In that case
                    // the diagnostic is covering unrelated text — suppress it.
                    let diag_span_len = diagnostic
                        .span
                        .end
                        .offset
                        .saturating_sub(diagnostic.span.start.offset);
                    let has_tighter_node = parsed_doc
                        .as_ref()
                        .and_then(|doc| marco_core::intelligence::get_position_span(position, doc))
                        .map(|s| s.end.offset.saturating_sub(s.start.offset) < diag_span_len)
                        .unwrap_or(false);

                    if has_tighter_node {
                        self.hide_hover_popover();
                        Err(no_content())
                    } else {
                        let (title, body, signature) = diagnostic_hover_markup(&diagnostic);
                        self.show_hover_popover(&view, &iter, &title, &body, signature);
                        Ok(())
                    }
                }
                (None, Some(info)) => {
                    let (start, end) = info
                        .range
                        .map(|s| (s.start.offset, s.end.offset))
                        .unwrap_or((usize::MAX, usize::MAX));
                    let signature = (start, end, info.contents.clone());
                    let (title, body) = split_hover_content(&info.contents);
                    self.show_hover_popover(&view, &iter, &title, &body, signature);
                    Ok(())
                }
                (None, None) => {
                    self.hide_hover_popover();
                    Err(no_content())
                }
            }
        }

        /// Build the popover widget tree once and cache all refs, or return the
        /// cached popover on subsequent calls.
        fn ensure_popover_for_view(&self, view: &sourceview5::View) -> gtk4::Popover {
            if let Some(popover) = self.hover_popover.borrow().as_ref().cloned() {
                return popover;
            }

            let popover = gtk4::Popover::new();
            popover.set_has_arrow(true);
            popover.set_autohide(true);
            popover.set_position(gtk4::PositionType::Bottom);
            popover.set_can_focus(false);
            popover.add_css_class("marco-link-popover");
            popover.set_parent(view);

            // ── Root container (no side margins — separator spans full width) ─
            let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            root.set_width_request(HOVER_POPOVER_WIDTH);

            // ── Header row: big title (left) + code chip (right) ─────────────
            let header_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
            header_row.set_margin_start(12);
            header_row.set_margin_end(12);
            header_row.set_margin_top(10);
            header_row.set_margin_bottom(8);

            let title = gtk4::Label::new(None);
            title.set_hexpand(true);
            title.set_halign(gtk4::Align::Start);
            title.set_valign(gtk4::Align::Center);
            title.set_xalign(0.0);
            title.set_wrap(true);
            title.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            title.set_lines(2);
            title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            title.add_css_class("marco-hover-title");

            let code_chip = gtk4::Label::new(None);
            code_chip.set_halign(gtk4::Align::End);
            code_chip.set_valign(gtk4::Align::Center);
            code_chip.set_hexpand(false);
            code_chip.set_visible(false);
            code_chip.add_css_class("marco-hover-code-chip");

            header_row.append(&title);
            header_row.append(&code_chip);
            root.append(&header_row);

            // ── Horizontal rule ───────────────────────────────────────────────
            let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
            separator.set_hexpand(true);
            separator.add_css_class("marco-hover-separator");
            root.append(&separator);

            // ── Content area ─────────────────────────────────────────────────
            let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            content.set_margin_start(12);
            content.set_margin_end(12);
            content.set_margin_top(8);
            content.set_margin_bottom(10);

            // "About" section header
            let about_header = gtk4::Label::new(Some("About"));
            about_header.set_halign(gtk4::Align::Start);
            about_header.set_xalign(0.0);
            about_header.set_margin_bottom(3);
            about_header.set_visible(false);
            about_header.add_css_class("marco-hover-section-label");

            // Description / plain body text
            let about_body = gtk4::Label::new(None);
            about_body.set_halign(gtk4::Align::Start);
            about_body.set_xalign(0.0);
            about_body.set_wrap(true);
            about_body.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            about_body.set_selectable(false);
            about_body.set_max_width_chars(58);
            about_body.set_margin_bottom(0);
            about_body.set_visible(false);
            about_body.add_css_class("marco-hover-body-text");

            content.append(&about_header);
            content.append(&about_body);

            // "Fix" section (diagnostic-only)
            let fix_section = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
            fix_section.set_margin_top(10);
            fix_section.set_visible(false);

            let fix_header = gtk4::Label::new(Some("Fix"));
            fix_header.set_halign(gtk4::Align::Start);
            fix_header.set_xalign(0.0);
            fix_header.set_margin_bottom(3);
            fix_header.add_css_class("marco-hover-section-label");

            let fix_body = gtk4::Label::new(None);
            fix_body.set_halign(gtk4::Align::Start);
            fix_body.set_xalign(0.0);
            fix_body.set_wrap(true);
            fix_body.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
            fix_body.set_selectable(false);
            fix_body.set_max_width_chars(58);
            fix_body.add_css_class("marco-hover-body-text");

            fix_section.append(&fix_header);
            fix_section.append(&fix_body);
            content.append(&fix_section);

            root.append(&content);
            popover.set_child(Some(&root));

            // Cache all widget refs
            *self.hover_popover.borrow_mut() = Some(popover.clone());
            *self.hover_title_label.borrow_mut() = Some(title);
            *self.hover_code_chip.borrow_mut() = Some(code_chip);
            *self.hover_about_header.borrow_mut() = Some(about_header);
            *self.hover_about_body.borrow_mut() = Some(about_body);
            *self.hover_fix_section.borrow_mut() = Some(fix_section);
            *self.hover_fix_body.borrow_mut() = Some(fix_body);

            popover
        }

        /// Push new content into the cached popover widgets.
        fn update_popover_content(&self, title: &str, parts: &HoverBodyParts) {
            // Title
            if let Some(label) = self.hover_title_label.borrow().as_ref() {
                label.set_text(title);
            }

            // Code chip — visible only for diagnostics
            if let Some(chip) = self.hover_code_chip.borrow().as_ref() {
                match &parts.code {
                    Some(code) => {
                        chip.set_text(code);
                        chip.set_visible(true);
                    }
                    None => chip.set_visible(false),
                }
            }

            // "About" section header — only for diagnostics that have a description
            if let Some(label) = self.hover_about_header.borrow().as_ref() {
                label.set_visible(parts.is_diagnostic && parts.about.is_some());
            }

            // About / plain body
            if let Some(label) = self.hover_about_body.borrow().as_ref() {
                match &parts.about {
                    Some(text) => {
                        label.set_text(text);
                        label.set_visible(true);
                    }
                    None => label.set_visible(false),
                }
            }

            // Fix section — only for diagnostics
            if let Some(section) = self.hover_fix_section.borrow().as_ref() {
                section.set_visible(parts.is_diagnostic && parts.fix.is_some());
            }
            if let Some(label) = self.hover_fix_body.borrow().as_ref() {
                if let Some(text) = &parts.fix {
                    label.set_text(text);
                }
            }
        }

        fn show_hover_popover(
            &self,
            view: &sourceview5::View,
            iter: &gtk4::TextIter,
            title: &str,
            body: &str,
            signature: (usize, usize, String),
        ) {
            let popover = self.ensure_popover_for_view(view);

            let unchanged = self
                .last_signature
                .borrow()
                .as_ref()
                .is_some_and(|prev| prev == &signature);

            if !unchanged {
                let parts = HoverBodyParts::from_body(body);
                self.update_popover_content(title, &parts);
                *self.last_signature.borrow_mut() = Some(signature);
            }

            let text_view = view.upcast_ref::<gtk4::TextView>();
            let caret_rect = hover_rect_for_iter(iter, text_view);
            let clamped_rect = clamp_rect_to_editor(caret_rect, text_view);
            popover.set_pointing_to(Some(&clamped_rect));

            if let Some(text_area) = visible_text_area_widget_rect(text_view) {
                let x_offset = compute_popover_x_offset_for_text_area(
                    clamped_rect.x(),
                    text_area.x(),
                    text_area.x() + text_area.width(),
                    HOVER_POPOVER_WIDTH,
                    HOVER_POPOVER_HORIZONTAL_SAFE_PADDING,
                );
                popover.set_offset(x_offset, 0);
            }

            popover.popup();
        }

        fn hide_hover_popover(&self) {
            if let Some(popover) = self.hover_popover.borrow().as_ref() {
                popover.popdown();
            }
            *self.last_signature.borrow_mut() = None;
        }
    }
}

glib::wrapper! {
    pub struct MarcoHoverProvider(ObjectSubclass<imp::MarcoHoverProvider>)
        @implements sourceview5::HoverProvider;
}

impl MarcoHoverProvider {
    /// Create a new hover provider wired to shared diagnostic state and settings.
    pub(crate) fn new(
        diagnostics: Rc<RefCell<Vec<marco_core::intelligence::Diagnostic>>>,
        settings_fn: Rc<dyn Fn() -> RuntimeIntelligenceSettings>,
    ) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        *imp.diagnostics.borrow_mut() = Some(diagnostics);
        *imp.settings_fn.borrow_mut() = Some(settings_fn);
        obj
    }
}

fn hover_rect_for_iter(
    iter: &gtk4::TextIter,
    editor_view: &gtk4::TextView,
) -> gtk4::gdk::Rectangle {
    let rect = editor_view.iter_location(iter);
    let (widget_x, widget_y) =
        editor_view.buffer_to_window_coords(gtk4::TextWindowType::Widget, rect.x(), rect.y());

    gtk4::gdk::Rectangle::new(
        widget_x,
        widget_y,
        rect.width().max(1),
        rect.height().max(1),
    )
}

fn clamp_rect_to_editor(
    rect: gtk4::gdk::Rectangle,
    editor_view: &gtk4::TextView,
) -> gtk4::gdk::Rectangle {
    let view_w = editor_view.allocated_width().max(1);
    let view_h = editor_view.allocated_height().max(1);
    let w = rect.width().max(1);
    let h = rect.height().max(1);

    let max_x = (view_w - w).max(0);
    let max_y = (view_h - h).max(0);
    let x = rect.x().clamp(0, max_x);
    let y = rect.y().clamp(0, max_y);

    gtk4::gdk::Rectangle::new(x, y, w, h)
}

fn visible_text_area_widget_rect(editor_view: &gtk4::TextView) -> Option<gtk4::gdk::Rectangle> {
    let visible = editor_view.visible_rect();
    if visible.width() <= 0 || visible.height() <= 0 {
        return None;
    }

    let (left, top) =
        editor_view.buffer_to_window_coords(gtk4::TextWindowType::Widget, visible.x(), visible.y());
    let (right, bottom) = editor_view.buffer_to_window_coords(
        gtk4::TextWindowType::Widget,
        visible.x() + visible.width(),
        visible.y() + visible.height(),
    );

    let x = left.min(right);
    let y = top.min(bottom);
    let w = (right - left).abs().max(1);
    let h = (bottom - top).abs().max(1);

    Some(gtk4::gdk::Rectangle::new(x, y, w, h))
}

fn compute_popover_x_offset_for_text_area(
    cursor_x: i32,
    text_left: i32,
    text_right: i32,
    popover_width: i32,
    safe_padding: i32,
) -> i32 {
    let half = (popover_width / 2).max(1);
    let desired_left = cursor_x - half;
    let desired_right = cursor_x + half;

    let min_left = text_left + safe_padding;
    let max_right = text_right - safe_padding;

    if desired_left < min_left {
        min_left - desired_left
    } else if desired_right > max_right {
        max_right - desired_right
    } else {
        0
    }
}
