//! Footer module for Marco markdown editor
//!
//! This module provides footer functionality for displaying various editor states including:
//! - Cursor position (row and column)
//! - Line count
//! - Word and character counts
//! - Current encoding
//! - Insert/overwrite mode
//! - Markdown syntax trace for the current line
//!
//! ## Threading Safety
//! All footer update functions are designed to be thread-safe. They use `set_label_text`
//! helper which automatically detects whether it's running on the main GTK thread and
//! schedules updates using `glib::idle_add_local` if necessary.
//!
//! ## Usage
//! Footer updates can be triggered individually using specific update functions, or in
//! batch using `apply_footer_update` with a `FooterUpdate::Snapshot`.

use gio::MemoryInputStream;
use glib::Bytes;
use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{
    Box, Button, CheckButton, Label, ListBox, ListBoxRow, Orientation, Picture, Popover,
    ScrolledWindow,
};
use rsvg::{CairoRenderer, Loader};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::components::language::FooterTranslations;
use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
use crate::ui::toolbar::icons::{toolbar_icon_svg, ToolbarIcon};

static UPDATE_VIS_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Gate footer debug output behind an env var so normal runs are quiet.
#[macro_export]
macro_rules! footer_dbg {
    ($($arg:tt)*) => {{
        if std::env::var("MARCO_DEBUG_FOOTER").is_ok() {
            eprintln!($($arg)*);
        }
    }};
}

/// Message type used to update the footer from any thread via a MainContext channel
#[derive(Debug)]
pub enum FooterUpdate {
    Snapshot {
        row: usize,
        col: usize,
        errors: usize,
        warnings: usize,
        diagnostics: Vec<FooterDiagnosticItem>,
        // lines removed
        words: usize,
        chars: usize,
        encoding: String,
        is_insert: bool,
    },
}

#[derive(Debug, Clone)]
pub struct FooterDiagnosticItem {
    pub severity: core::intelligence::DiagnosticSeverity,
    pub code: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub fix_suggestion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticsSeverityFilter {
    pub errors: bool,
    pub warnings: bool,
    pub hints: bool,
    pub infos: bool,
}

impl DiagnosticsSeverityFilter {
    pub fn default_error_warning() -> Self {
        Self {
            errors: true,
            warnings: true,
            hints: false,
            infos: false,
        }
    }
}

#[derive(Clone)]
pub struct FooterLabels {
    pub cursor_row: Label,
    pub cursor_col: Label,
    pub encoding: Label,
    pub insert_mode: Label,
    pub word_count: Label,
    pub char_count: Label,
    pub diagnostics_trigger: Button,
    pub diagnostics_trigger_label: Label,
    pub diagnostics_popover: Popover,
    pub diagnostics_list: ListBox,
    pub diagnostics_error_check: CheckButton,
    pub diagnostics_warning_check: CheckButton,
    pub diagnostics_hint_check: CheckButton,
    pub diagnostics_info_check: CheckButton,
    pub diagnostics_errors: RefCell<usize>,
    pub diagnostics_warnings: RefCell<usize>,
    pub diagnostics_total: RefCell<usize>,
    pub diagnostics_filter: RefCell<DiagnosticsSeverityFilter>,
    pub diagnostics_items: RefCell<Vec<FooterDiagnosticItem>>,
    pub diagnostics_navigate_to: RefCell<Option<Rc<dyn Fn(usize, usize)>>>,
    pub hovered_link_icon: Picture,
    pub hovered_link_text: Label,
    pub row_label: RefCell<String>,
    pub column_label: RefCell<String>,
    pub words_label: RefCell<String>,
    pub characters_label: RefCell<String>,
    pub ins_label: RefCell<String>,
    pub ovr_label: RefCell<String>,
    pub encoding_label: RefCell<String>,
}

fn severity_rank(severity: &core::intelligence::DiagnosticSeverity) -> u8 {
    match severity {
        core::intelligence::DiagnosticSeverity::Error => 0,
        core::intelligence::DiagnosticSeverity::Warning => 1,
        core::intelligence::DiagnosticSeverity::Info => 2,
        core::intelligence::DiagnosticSeverity::Hint => 3,
    }
}

fn severity_label(severity: &core::intelligence::DiagnosticSeverity) -> &'static str {
    match severity {
        core::intelligence::DiagnosticSeverity::Error => "Error",
        core::intelligence::DiagnosticSeverity::Warning => "Warning",
        core::intelligence::DiagnosticSeverity::Info => "Info",
        core::intelligence::DiagnosticSeverity::Hint => "Hint",
    }
}

fn severity_row_css_class(severity: &core::intelligence::DiagnosticSeverity) -> &'static str {
    match severity {
        core::intelligence::DiagnosticSeverity::Error => "footer-issue-row-error",
        core::intelligence::DiagnosticSeverity::Warning => "footer-issue-row-warning",
        core::intelligence::DiagnosticSeverity::Info => "footer-issue-row-info",
        core::intelligence::DiagnosticSeverity::Hint => "footer-issue-row-hint",
    }
}

fn diagnostics_filter_from_settings(
    settings: &core::logic::swanson::Settings,
) -> DiagnosticsSeverityFilter {
    let Some(editor) = settings.editor.as_ref() else {
        return DiagnosticsSeverityFilter::default_error_warning();
    };
    let Some(filter) = editor.diagnostics_filter.as_ref() else {
        return DiagnosticsSeverityFilter::default_error_warning();
    };

    DiagnosticsSeverityFilter {
        errors: filter.errors.unwrap_or(true),
        warnings: filter.warnings.unwrap_or(true),
        hints: filter.hints.unwrap_or(false),
        infos: filter.infos.unwrap_or(false),
    }
}

fn persist_diagnostics_filter(
    settings_manager: &Arc<core::logic::swanson::SettingsManager>,
    filter: DiagnosticsSeverityFilter,
) {
    if let Err(err) = settings_manager.update_settings(|settings| {
        if settings.editor.is_none() {
            settings.editor = Some(core::logic::swanson::EditorSettings::default());
        }

        if let Some(editor) = settings.editor.as_mut() {
            editor.diagnostics_filter = Some(core::logic::swanson::DiagnosticsFilterSettings {
                errors: Some(filter.errors),
                warnings: Some(filter.warnings),
                hints: Some(filter.hints),
                infos: Some(filter.infos),
            });
        }
    }) {
        log::error!("Failed to persist diagnostics filter settings: {}", err);
    } else {
        // Keep editor highlights/hover in sync with footer filter toggles.
        crate::components::editor::editor_manager::trigger_intelligence_refresh();
    }
}

fn update_diagnostics_checkbox_labels(
    labels: &FooterLabels,
    errors: usize,
    warnings: usize,
    hints: usize,
    infos: usize,
) {
    labels
        .diagnostics_error_check
        .set_label(Some(&format!("Error ({})", errors)));
    labels
        .diagnostics_warning_check
        .set_label(Some(&format!("Warning ({})", warnings)));
    labels
        .diagnostics_hint_check
        .set_label(Some(&format!("Hint ({})", hints)));
    labels
        .diagnostics_info_check
        .set_label(Some(&format!("Info ({})", infos)));
}

fn fallback_texture() -> gdk::MemoryTexture {
    let bytes = Bytes::from_owned(vec![0u8, 0u8, 0u8, 0u8]);
    gdk::MemoryTexture::new(1, 1, gdk::MemoryFormat::B8g8r8a8Premultiplied, &bytes, 4)
}

// ── Link-type icons ──────────────────────────────────────────────────────────
// SVG source: Tabler Icons (MIT)

const LINK_ICON: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 21v-4"/><path d="M12 13v-4"/><path d="M12 5v-2"/><path d="M10 21h4"/><path d="M8 5v4h11l2 -2l-2 -2l-11 0"/><path d="M14 13v4h-8l-2 -2l2 -2l8 0"/></svg>"#;

fn render_link_type_icon(svg: &str, color: &str, icon_size: f64) -> gdk::MemoryTexture {
    let svg_colored = svg.replace("currentColor", color);
    let bytes = Bytes::from_owned(svg_colored.into_bytes());
    let stream = MemoryInputStream::from_bytes(&bytes);

    let handle =
        match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load link icon SVG: {}", e);
                return fallback_texture();
            }
        };

    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);

    let render_scale = display_scale * 2.0;
    let render_size = (icon_size * render_scale) as i32;

    let mut surface =
        match cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size) {
            Ok(s) => s,
            Err(e) => {
                log::error!("create link icon surface: {}", e);
                return fallback_texture();
            }
        };
    {
        let cr = match cairo::Context::new(&surface) {
            Ok(c) => c,
            Err(e) => {
                log::error!("create link icon cairo context: {}", e);
                return fallback_texture();
            }
        };
        cr.scale(render_scale, render_scale);
        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
        if let Err(e) = renderer.render_document(&cr, &viewport) {
            log::error!("render link icon SVG: {}", e);
            return fallback_texture();
        }
    }

    let data = match surface.data() {
        Ok(d) => d.to_vec(),
        Err(e) => {
            log::error!("get link icon surface data: {}", e);
            return fallback_texture();
        }
    };
    let bytes = Bytes::from_owned(data);
    gdk::MemoryTexture::new(
        render_size,
        render_size,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    )
}

fn render_footer_svg_icon(icon: ToolbarIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
    let svg = toolbar_icon_svg(icon).replace("currentColor", color);
    let bytes = Bytes::from_owned(svg.into_bytes());
    let stream = MemoryInputStream::from_bytes(&bytes);

    let handle =
        match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
            Ok(h) => h,
            Err(e) => {
                log::error!("load footer SVG handle: {}", e);
                return fallback_texture();
            }
        };

    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);

    let render_scale = display_scale * 2.0;
    let render_size = (icon_size * render_scale) as i32;

    let mut surface =
        match cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size) {
            Ok(s) => s,
            Err(e) => {
                log::error!("create footer SVG image surface: {}", e);
                return fallback_texture();
            }
        };

    {
        let cr = match cairo::Context::new(&surface) {
            Ok(c) => c,
            Err(e) => {
                log::error!("create footer SVG cairo context: {}", e);
                return fallback_texture();
            }
        };

        cr.scale(render_scale, render_scale);

        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
        if let Err(e) = renderer.render_document(&cr, &viewport) {
            log::error!("render footer SVG: {}", e);
            return fallback_texture();
        }
    }

    let data = match surface.data() {
        Ok(d) => d.to_vec(),
        Err(e) => {
            log::error!("get footer SVG surface data: {}", e);
            return fallback_texture();
        }
    };

    let bytes = Bytes::from_owned(data);
    gdk::MemoryTexture::new(
        render_size,
        render_size,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    )
}

fn footer_is_dark_theme(widget: &gtk4::Widget) -> bool {
    widget
        .root()
        .and_then(|r| r.downcast::<gtk4::Window>().ok())
        .map(|w| w.has_css_class("marco-theme-dark"))
        .unwrap_or(false)
}

fn footer_icon_color_for_flags(widget: &gtk4::Widget, flags: gtk4::StateFlags) -> &'static str {
    let dark = footer_is_dark_theme(widget);

    if flags.contains(gtk4::StateFlags::ACTIVE) {
        if dark {
            DARK_PALETTE.control_icon_active
        } else {
            LIGHT_PALETTE.control_icon_active
        }
    } else if flags.contains(gtk4::StateFlags::PRELIGHT) {
        if dark {
            DARK_PALETTE.control_icon_hover
        } else {
            LIGHT_PALETTE.control_icon_hover
        }
    } else if dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    }
}

fn create_footer_status_button(text: &str, icon: ToolbarIcon, icon_size: f64) -> (Button, Label) {
    let button = Button::new();
    button.set_has_frame(false);
    button.set_visible(true);
    button.set_halign(gtk4::Align::Start);
    button.set_hexpand(false);
    button.set_valign(gtk4::Align::Center);
    button.set_margin_top(0);
    button.set_margin_bottom(0);
    button.set_margin_end(0);
    button.set_margin_start(0);
    button.add_css_class("footer-diagnostics-trigger");
    button.add_css_class("footer-diagnostics-ok");

    let content = Box::new(Orientation::Horizontal, 2);
    content.set_halign(gtk4::Align::Start);
    content.set_hexpand(false);
    content.set_valign(gtk4::Align::Center);
    content.set_margin_top(0);
    content.set_margin_bottom(0);
    content.set_margin_start(2);
    content.set_margin_end(2);

    let icon_widget = Picture::new();
    let initial_flags = button.state_flags();
    let initial_icon_color = footer_icon_color_for_flags(button.upcast_ref(), initial_flags);
    let icon_texture = render_footer_svg_icon(icon, initial_icon_color, icon_size);
    icon_widget.set_paintable(Some(&icon_texture));
    icon_widget.set_size_request(icon_size as i32, icon_size as i32);
    icon_widget.set_can_shrink(false);
    icon_widget.set_halign(gtk4::Align::Center);
    icon_widget.set_valign(gtk4::Align::Center);

    let label = Label::new(Some(text));
    label.set_visible(true);
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(false);
    label.set_valign(gtk4::Align::Center);
    label.set_yalign(0.5);
    label.set_margin_top(0);
    label.set_margin_bottom(0);
    label.set_margin_start(0);
    label.set_margin_end(0);
    label.add_css_class("footer-status-label");

    content.append(&icon_widget);
    content.append(&label);
    button.set_child(Some(&content));

    {
        let pic_update = icon_widget.clone();
        let btn_update = button.clone();
        let update_icon = move || {
            let flags = btn_update.state_flags();
            let color = footer_icon_color_for_flags(btn_update.upcast_ref(), flags);
            let texture = render_footer_svg_icon(icon, color, icon_size);
            pic_update.set_paintable(Some(&texture));
        };

        let update_for_state = update_icon.clone();
        button.connect_state_flags_changed(move |btn, _| {
            if btn.is_mapped() {
                update_for_state();
            }
        });

        let update_for_map = update_icon.clone();
        button.connect_map(move |_| {
            update_for_map();
        });

        button.connect_clicked(move |_| {
            update_icon();
        });
    }

    (button, label)
}

pub fn bind_diagnostics_navigation(
    labels: &Rc<FooterLabels>,
    buffer: &sourceview5::Buffer,
    source_view: &sourceview5::View,
) {
    let buffer = buffer.clone();
    let source_view = source_view.clone();
    *labels.diagnostics_navigate_to.borrow_mut() = Some(Rc::new(move |line, column| {
        let gtk_line = line.saturating_sub(1) as i32;
        let Some(mut iter) = buffer.iter_at_line(gtk_line) else {
            return;
        };

        let max_chars = iter.chars_in_line().max(0) as usize;
        let gtk_col = column.saturating_sub(1).min(max_chars) as i32;
        iter.set_line_offset(gtk_col);

        // Highlight the whole target line so navigation is visually obvious.
        let Some(line_start) = buffer.iter_at_line(gtk_line) else {
            return;
        };
        let mut line_end = line_start;
        line_end.forward_to_line_end();
        buffer.select_range(&line_start, &line_end);

        // Keep editor viewport aligned with selected diagnostic row target.
        source_view.scroll_to_iter(&mut iter, 0.18, true, 0.0, 0.5);
        source_view.grab_focus();
    }));
}

/// Update the row label independently
pub fn update_cursor_row(labels: &FooterLabels, row: usize) {
    let text = format!("{}: {}", labels.row_label.borrow(), row);
    footer_dbg!("[footer] update_cursor_row called: {}", text);
    set_label_text(&labels.cursor_row, text);
}

/// Update the column label independently
pub fn update_cursor_col(labels: &FooterLabels, col: usize) {
    let text = format!("{}: {}", labels.column_label.borrow(), col);
    footer_dbg!("[footer] update_cursor_col called: {}", text);
    set_label_text(&labels.cursor_col, text);
}

// line count removed: no-op omitted

/// Updates the encoding label
pub fn update_encoding(labels: &FooterLabels, encoding: &str) {
    let enc = encoding.to_string();
    footer_dbg!("[footer] update_encoding called: {}", enc);
    set_label_text(&labels.encoding, enc);
}

/// Updates the insert/overwrite mode label
pub fn update_insert_mode(labels: &FooterLabels, is_insert: bool) {
    let text = if is_insert {
        labels.ins_label.borrow()
    } else {
        labels.ovr_label.borrow()
    };
    footer_dbg!("[footer] update_insert_mode called: {}", text);
    set_label_text(&labels.insert_mode, text.to_string());
}

/// Updates the word count label
pub fn update_word_count(labels: &FooterLabels, words: usize) {
    let text = format!("{}: {}", labels.words_label.borrow(), words);
    footer_dbg!("[footer] update_word_count called: {}", text);
    set_label_text(&labels.word_count, text);
}

/// Updates the character count label
pub fn update_char_count(labels: &FooterLabels, chars: usize) {
    let text = format!("{}: {}", labels.characters_label.borrow(), chars);
    footer_dbg!("[footer] update_char_count called: {}", text);
    set_label_text(&labels.char_count, text);
}

/// Updates the error count label
pub fn update_error_count(labels: &FooterLabels, errors: usize) {
    *labels.diagnostics_errors.borrow_mut() = errors;
    update_diagnostics_trigger_state(labels);
}

/// Updates the warning count label
pub fn update_warning_count(labels: &FooterLabels, warnings: usize) {
    *labels.diagnostics_warnings.borrow_mut() = warnings;
    update_diagnostics_trigger_state(labels);
}

pub fn update_issue_count(labels: &FooterLabels, total: usize) {
    *labels.diagnostics_total.borrow_mut() = total;
    set_label_text(
        &labels.diagnostics_trigger_label,
        format!("Issue: {}", total),
    );
}

/// Show a hovered link URL in the footer spacer area.
///
/// Pass `Some(url)` when the mouse enters a link, `None` when it leaves.
/// The appropriate icon is chosen automatically based on the URL scheme.
pub fn update_hovered_link(labels: &FooterLabels, url: Option<&str>) {
    match url {
        None => {
            let color = if footer_is_dark_theme(labels.hovered_link_icon.upcast_ref()) {
                DARK_PALETTE.control_icon
            } else {
                LIGHT_PALETTE.control_icon
            };
            let texture = render_link_type_icon(LINK_ICON, color, 8.0);
            labels.hovered_link_icon.set_paintable(Some(&texture));
            labels.hovered_link_text.set_text("");
        }
        Some(url) => {
            let color = if footer_is_dark_theme(labels.hovered_link_icon.upcast_ref()) {
                DARK_PALETTE.control_icon
            } else {
                LIGHT_PALETTE.control_icon
            };
            let texture = render_link_type_icon(LINK_ICON, color, 8.0);
            labels.hovered_link_icon.set_paintable(Some(&texture));
            labels.hovered_link_icon.set_visible(true);
            labels.hovered_link_text.set_text(url);
        }
    }
}

fn update_diagnostics_trigger_state(labels: &FooterLabels) {
    let trigger = &labels.diagnostics_trigger;
    trigger.remove_css_class("footer-diagnostics-ok");
    trigger.remove_css_class("footer-diagnostics-warning");
    trigger.remove_css_class("footer-diagnostics-error");

    let errors = *labels.diagnostics_errors.borrow();
    let warnings = *labels.diagnostics_warnings.borrow();

    if errors > 0 {
        trigger.add_css_class("footer-diagnostics-error");
    } else if warnings > 0 {
        trigger.add_css_class("footer-diagnostics-warning");
    } else {
        trigger.add_css_class("footer-diagnostics-ok");
    }
}

pub fn update_diagnostics_panel(labels: &FooterLabels, diagnostics: &[FooterDiagnosticItem]) {
    *labels.diagnostics_items.borrow_mut() = diagnostics.to_vec();
    render_diagnostics_panel(labels);
}

fn render_diagnostics_panel(labels: &FooterLabels) {
    while let Some(child) = labels.diagnostics_list.first_child() {
        labels.diagnostics_list.remove(&child);
    }

    let source_items: Vec<FooterDiagnosticItem> = labels.diagnostics_items.borrow().clone();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut hints = 0usize;
    let mut infos = 0usize;

    for item in &source_items {
        match item.severity {
            core::intelligence::DiagnosticSeverity::Error => errors += 1,
            core::intelligence::DiagnosticSeverity::Warning => warnings += 1,
            core::intelligence::DiagnosticSeverity::Hint => hints += 1,
            core::intelligence::DiagnosticSeverity::Info => infos += 1,
        }
    }

    update_diagnostics_checkbox_labels(labels, errors, warnings, hints, infos);

    let filter = *labels.diagnostics_filter.borrow();
    let mut items: Vec<FooterDiagnosticItem> = source_items
        .into_iter()
        .filter(|item| match item.severity {
            core::intelligence::DiagnosticSeverity::Error => filter.errors,
            core::intelligence::DiagnosticSeverity::Warning => filter.warnings,
            core::intelligence::DiagnosticSeverity::Hint => filter.hints,
            core::intelligence::DiagnosticSeverity::Info => filter.infos,
        })
        .collect();

    items.sort_by(|a, b| {
        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            .then(a.line.cmp(&b.line))
            .then(a.column.cmp(&b.column))
    });

    if items.is_empty() {
        let empty = Label::new(Some("No diagnostics"));
        empty.add_css_class("footer-issue-empty");
        labels.diagnostics_list.append(&empty);
        return;
    }

    for item in items.into_iter().take(200) {
        let severity_text = severity_label(&item.severity);

        let detail_text = format!(
            "{} • {}  {}:{}\n{}\n{}",
            severity_text, item.code, item.line, item.column, item.message, item.fix_suggestion
        );
        let detail = Label::new(Some(&detail_text));
        detail.set_xalign(0.0);
        detail.set_wrap(true);
        // Keep clicks flowing to the row gesture (so clicking text also navigates).
        detail.set_selectable(false);
        detail.add_css_class("footer-issue-row-label");

        let row_box = Box::new(Orientation::Horizontal, 0);
        row_box.add_css_class("footer-issue-row");
        row_box.add_css_class(severity_row_css_class(&item.severity));
        row_box.append(&detail);

        let row = ListBoxRow::new();
        row.set_child(Some(&row_box));
        row.set_selectable(false);
        row.set_activatable(true);

        let goto = labels.diagnostics_navigate_to.borrow().clone();
        let popover = labels.diagnostics_popover.clone();
        let line = item.line;
        let column = item.column;
        let click = gtk4::GestureClick::new();
        click.connect_pressed(move |_, _, _, _| {
            if let Some(go) = &goto {
                go(line, column);
            }
            popover.popdown();
        });
        row.add_controller(click);

        labels.diagnostics_list.append(&row);
    }
}

/// Update translation labels used by the footer and refresh static text.
pub fn update_footer_translations(
    labels: &FooterLabels,
    translations: &FooterTranslations,
    is_insert: bool,
) {
    *labels.row_label.borrow_mut() = translations.row.clone();
    *labels.column_label.borrow_mut() = translations.column.clone();
    *labels.words_label.borrow_mut() = translations.words.clone();
    *labels.characters_label.borrow_mut() = translations.characters.clone();
    *labels.ins_label.borrow_mut() = translations.ins.clone();
    *labels.ovr_label.borrow_mut() = translations.ovr.clone();
    *labels.encoding_label.borrow_mut() = translations.encoding_utf8.clone();

    update_encoding(labels, &translations.encoding_utf8);
    update_insert_mode(labels, is_insert);
}

/// Apply a FooterUpdate snapshot to the labels. Must be called on main context.
pub fn apply_footer_update(labels: &FooterLabels, update: FooterUpdate) {
    match update {
        FooterUpdate::Snapshot {
            row,
            col,
            errors,
            warnings,
            diagnostics,
            /*lines,*/ words,
            chars,
            encoding,
            is_insert,
        } => {
            let issue_total = diagnostics.len();
            update_cursor_row(labels, row);
            update_cursor_col(labels, col);
            update_error_count(labels, errors);
            update_warning_count(labels, warnings);
            update_issue_count(labels, issue_total);
            update_diagnostics_panel(labels, &diagnostics);
            update_word_count(labels, words);
            update_char_count(labels, chars);
            update_encoding(labels, &encoding);
            update_insert_mode(labels, is_insert);
        }
    }
}

/// Helper: set a Label's text on the main GTK context, scheduling if needed.
/// This function ensures thread safety and provides consistent label updating.
fn set_label_text(label: &Label, text: String) {
    let mut final_text = text.clone();

    // If debug env var set, append a small counter so updates are visually detectable
    if std::env::var("MARCO_DEBUG_FOOTER_VIS").is_ok() {
        let n = UPDATE_VIS_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        final_text = format!("{} [{}]", text, n);
    }

    let use_markup = std::env::var("MARCO_DEBUG_FOOTER_VIS").is_ok();

    // Check if we're on the main thread
    if glib::MainContext::default().is_owner() {
        // We're on the main thread, update immediately
        update_label_immediate(label, &final_text, use_markup);
    } else {
        // We're not on the main thread, schedule the update
        let lbl = label.clone();
        glib::idle_add_local(move || {
            update_label_immediate(&lbl, &final_text, use_markup);
            glib::ControlFlow::Break
        });
    }
}

/// Immediately update a label on the main thread
fn update_label_immediate(label: &Label, text: &str, use_markup: bool) {
    if use_markup {
        // Escape and set markup for a bold visual (debug mode)
        let escaped_text = glib::markup_escape_text(text);
        label.set_markup(&format!("<b>{}</b>", escaped_text));
    } else {
        label.set_text(text);
    }

    footer_dbg!("[footer] set_label_text immediate -> {}", label.text());
    footer_dbg!(
        "[footer] label visible: {}, parent visible: {}",
        label.is_visible(),
        label.parent().map(|p| p.is_visible()).unwrap_or(false)
    );

    // Ensure widget is visible and request a redraw for better reliability
    label.set_visible(true);
    // Avoid calling queue_draw() directly here; GTK may issue warnings when widgets
    // are not yet allocated. Rely on set_visible and normal GTK redraw scheduling.

    // Also ensure parent is visible
    if let Some(parent) = label.parent() {
        parent.set_visible(true);
    }
}

pub fn create_footer(
    translations: &FooterTranslations,
    settings_manager: Arc<core::logic::swanson::SettingsManager>,
) -> (Box, Rc<FooterLabels>) {
    let footer_box = Box::new(Orientation::Horizontal, 10);
    footer_box.set_margin_top(0);
    footer_box.set_margin_bottom(0);
    footer_box.set_margin_start(0);
    footer_box.set_margin_end(0);

    // Ensure footer is visible and properly allocated
    footer_box.set_visible(true);
    footer_box.set_can_focus(false);
    footer_box.set_vexpand(false);
    footer_box.set_hexpand(true);
    footer_box.set_height_request(0); // Minimum height

    // Add CSS class for potential styling
    footer_box.add_css_class("footer");

    // Diagnostics trigger (left side)
    let status_buttons_box = Box::new(Orientation::Horizontal, 8);
    status_buttons_box.set_visible(true);
    status_buttons_box.set_halign(gtk4::Align::Fill);
    status_buttons_box.set_hexpand(true);
    status_buttons_box.set_valign(gtk4::Align::Center);

    let (toc_stub_button, _toc_stub_label) =
        create_footer_status_button("TOC", ToolbarIcon::Toc, 8.0);
    toc_stub_button.set_tooltip_text(Some("Toggle Table of Contents panel"));
    toc_stub_button.connect_clicked(|_| {
        crate::components::editor::ui::with_toc_panel(|h| h.toggle());
    });
    status_buttons_box.append(&toc_stub_button);

    let (terminal_stub_button, _terminal_stub_label) =
        create_footer_status_button("Terminal", ToolbarIcon::Terminal, 8.0);
    terminal_stub_button.set_tooltip_text(Some("Terminal: new stub"));
    terminal_stub_button.connect_clicked(|_| {
        log::debug!("Footer Terminal stub clicked");
    });
    status_buttons_box.append(&terminal_stub_button);

    let (diagnostics_trigger_button, diagnostics_trigger_label) =
        create_footer_status_button("Issue: 0", ToolbarIcon::Issue, 8.0);

    status_buttons_box.append(&diagnostics_trigger_button);

    // Hovered-link area: icon + URL text. Sits right of Issue in the status bar.
    let hovered_link_box = Box::new(Orientation::Horizontal, 2);
    hovered_link_box.set_hexpand(true);
    hovered_link_box.set_halign(gtk4::Align::Start);
    hovered_link_box.set_valign(gtk4::Align::Center);
    hovered_link_box.add_css_class("footer-hovered-link");

    let hovered_link_icon = Picture::new();
    hovered_link_icon.set_size_request(8, 8);
    hovered_link_icon.set_can_shrink(false);
    hovered_link_icon.set_halign(gtk4::Align::Center);
    hovered_link_icon.set_valign(gtk4::Align::Center);
    hovered_link_icon.set_visible(true);
    // Show the default book icon immediately — it will be replaced on hover.
    {
        let color = if footer_is_dark_theme(hovered_link_icon.upcast_ref()) {
            DARK_PALETTE.control_icon
        } else {
            LIGHT_PALETTE.control_icon
        };
        let texture = render_link_type_icon(LINK_ICON, color, 8.0);
        hovered_link_icon.set_paintable(Some(&texture));
    }
    hovered_link_box.append(&hovered_link_icon);

    let hovered_link_text = Label::new(None);
    hovered_link_text.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    hovered_link_text.set_visible(true);
    hovered_link_text.set_valign(gtk4::Align::Center);
    hovered_link_text.add_css_class("footer-status-label");
    hovered_link_box.append(&hovered_link_text);

    status_buttons_box.append(&hovered_link_box);

    footer_box.append(&status_buttons_box);

    // Info labels (right side)
    let word_count_label = Label::new(Some(&format!("{}: 0", translations.words)));
    word_count_label.set_visible(true);
    footer_box.append(&word_count_label);

    let char_count_label = Label::new(Some(&format!("{}: 0", translations.characters)));
    char_count_label.set_visible(true);
    footer_box.append(&char_count_label);

    let cursor_row_label = Label::new(Some(&format!("{}: 1", translations.row)));
    cursor_row_label.set_visible(true);
    footer_box.append(&cursor_row_label);

    let cursor_col_label = Label::new(Some(&format!("{}: 1", translations.column)));
    cursor_col_label.set_visible(true);
    footer_box.append(&cursor_col_label);

    let encoding_label = Label::new(Some(&translations.encoding_utf8));
    encoding_label.set_visible(true);
    footer_box.append(&encoding_label);

    let insert_mode_label = Label::new(Some(&translations.ins));
    insert_mode_label.set_visible(true);
    footer_box.append(&insert_mode_label);

    let diagnostics_popover = Popover::new();
    diagnostics_popover.set_has_arrow(true);
    diagnostics_popover.set_autohide(true);
    diagnostics_popover.set_position(gtk4::PositionType::Top);
    diagnostics_popover.add_css_class("marco-diagnostics-popover");

    let diagnostics_scrolled = ScrolledWindow::new();
    diagnostics_scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
    diagnostics_scrolled.set_min_content_width(520);
    diagnostics_scrolled.set_min_content_height(300);
    diagnostics_scrolled.add_css_class("footer-issue-scrolled");

    let filter_bar = Box::new(Orientation::Horizontal, 6);
    filter_bar.add_css_class("footer-diag-filter-bar");
    let initial_filter = diagnostics_filter_from_settings(&settings_manager.get_settings());

    let error_check = CheckButton::with_label("Error (0)");
    let warning_check = CheckButton::with_label("Warning (0)");
    let hint_check = CheckButton::with_label("Hint (0)");
    let info_check = CheckButton::with_label("Info (0)");

    error_check.add_css_class("footer-diag-filter-check");
    error_check.add_css_class("marco-checkbutton");
    warning_check.add_css_class("footer-diag-filter-check");
    warning_check.add_css_class("marco-checkbutton");
    hint_check.add_css_class("footer-diag-filter-check");
    hint_check.add_css_class("marco-checkbutton");
    info_check.add_css_class("footer-diag-filter-check");
    info_check.add_css_class("marco-checkbutton");

    error_check.set_active(initial_filter.errors);
    warning_check.set_active(initial_filter.warnings);
    hint_check.set_active(initial_filter.hints);
    info_check.set_active(initial_filter.infos);

    filter_bar.append(&error_check);
    filter_bar.append(&warning_check);
    filter_bar.append(&hint_check);
    filter_bar.append(&info_check);

    let diagnostics_list = ListBox::new();
    diagnostics_list.add_css_class("footer-issue-list");
    diagnostics_list.set_selection_mode(gtk4::SelectionMode::None);
    diagnostics_list.append(&Label::new(Some("No diagnostics")));
    diagnostics_scrolled.set_child(Some(&diagnostics_list));

    let diagnostics_content = Box::new(Orientation::Vertical, 8);
    diagnostics_content.append(&filter_bar);
    diagnostics_content.append(&diagnostics_scrolled);
    diagnostics_popover.set_child(Some(&diagnostics_content));

    let popover_for_diagnostics = diagnostics_popover.clone();
    diagnostics_trigger_button.connect_clicked(move |_| {
        popover_for_diagnostics.popup();
    });

    diagnostics_popover.set_parent(&diagnostics_trigger_button);

    let labels = Rc::new(FooterLabels {
        cursor_row: cursor_row_label,
        cursor_col: cursor_col_label,
        encoding: encoding_label,
        insert_mode: insert_mode_label,
        word_count: word_count_label,
        char_count: char_count_label,
        diagnostics_trigger: diagnostics_trigger_button,
        diagnostics_trigger_label,
        diagnostics_popover: diagnostics_popover.clone(),
        diagnostics_list,
        diagnostics_error_check: error_check,
        diagnostics_warning_check: warning_check,
        diagnostics_hint_check: hint_check,
        diagnostics_info_check: info_check,
        diagnostics_errors: RefCell::new(0),
        diagnostics_warnings: RefCell::new(0),
        diagnostics_total: RefCell::new(0),
        diagnostics_filter: RefCell::new(initial_filter),
        diagnostics_items: RefCell::new(Vec::new()),
        diagnostics_navigate_to: RefCell::new(None),
        hovered_link_icon,
        hovered_link_text,
        row_label: RefCell::new(translations.row.clone()),
        column_label: RefCell::new(translations.column.clone()),
        words_label: RefCell::new(translations.words.clone()),
        characters_label: RefCell::new(translations.characters.clone()),
        ins_label: RefCell::new(translations.ins.clone()),
        ovr_label: RefCell::new(translations.ovr.clone()),
        encoding_label: RefCell::new(translations.encoding_utf8.clone()),
    });

    {
        let labels_for_errors = Rc::clone(&labels);
        let settings_manager = settings_manager.clone();
        labels
            .diagnostics_error_check
            .connect_toggled(move |check| {
                let mut filter = *labels_for_errors.diagnostics_filter.borrow();
                filter.errors = check.is_active();
                *labels_for_errors.diagnostics_filter.borrow_mut() = filter;
                persist_diagnostics_filter(&settings_manager, filter);
                render_diagnostics_panel(&labels_for_errors);
            });
    }

    {
        let labels_for_warnings = Rc::clone(&labels);
        let settings_manager = settings_manager.clone();
        labels
            .diagnostics_warning_check
            .connect_toggled(move |check| {
                let mut filter = *labels_for_warnings.diagnostics_filter.borrow();
                filter.warnings = check.is_active();
                *labels_for_warnings.diagnostics_filter.borrow_mut() = filter;
                persist_diagnostics_filter(&settings_manager, filter);
                render_diagnostics_panel(&labels_for_warnings);
            });
    }

    {
        let labels_for_hints = Rc::clone(&labels);
        let settings_manager = settings_manager.clone();
        labels.diagnostics_hint_check.connect_toggled(move |check| {
            let mut filter = *labels_for_hints.diagnostics_filter.borrow();
            filter.hints = check.is_active();
            *labels_for_hints.diagnostics_filter.borrow_mut() = filter;
            persist_diagnostics_filter(&settings_manager, filter);
            render_diagnostics_panel(&labels_for_hints);
        });
    }

    {
        let labels_for_infos = Rc::clone(&labels);
        labels.diagnostics_info_check.connect_toggled(move |check| {
            let mut filter = *labels_for_infos.diagnostics_filter.borrow();
            filter.infos = check.is_active();
            *labels_for_infos.diagnostics_filter.borrow_mut() = filter;
            persist_diagnostics_filter(&settings_manager, filter);
            render_diagnostics_panel(&labels_for_infos);
        });
    }

    (footer_box, labels)
}
