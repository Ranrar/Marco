//! Insert Math Dialog
//!
//! A beginner-friendly + power-user math dialog for writing KaTeX/LaTeX in Markdown.
//! - Inline or block mode
//! - Curated templates grouped by category
//! - Live validation powered by `katex-rs`

use gtk4::{
    glib, prelude::*, Align, Box, Button, CheckButton, DropDown, Expression, Label, Orientation,
    PolicyType, PropertyExpression, ScrolledWindow, StringList, StringObject, TextView, Window,
};
use katex::{
    render_to_string as katex_render, KatexContext, OutputFormat, Settings as KatexSettings,
};
use sourceview5::{Buffer, View};
use std::sync::OnceLock;

#[cfg(target_os = "linux")]
use webkit6::prelude::WebViewExt;

#[cfg(target_os = "linux")]
type PreviewSurface = webkit6::WebView;

#[cfg(target_os = "windows")]
type PreviewSurface = crate::components::viewer::wry_platform_webview::PlatformWebView;

static KATEX_CONTEXT: OnceLock<KatexContext> = OnceLock::new();

fn is_debug_mode_enabled() -> bool {
    let settings_path = match marco_shared::paths::MarcoPaths::new() {
        Ok(paths) => paths.settings_file(),
        Err(err) => {
            log::debug!("math dialog: failed to resolve settings path for debug mode: {err}");
            return false;
        }
    };

    match marco_shared::logic::swanson::SettingsManager::initialize(settings_path) {
        Ok(manager) => manager.get_settings().debug.unwrap_or(false),
        Err(err) => {
            log::debug!("math dialog: failed to initialize settings manager for debug mode: {err}");
            false
        }
    }
}

#[cfg(target_os = "linux")]
fn preview_backend_label() -> &'static str {
    "Preview backend: Linux WebKit"
}

#[cfg(target_os = "windows")]
fn preview_backend_label() -> &'static str {
    "Preview backend: Windows Wry"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MathMode {
    Inline,
    Block,
}

#[derive(Debug, Clone, Copy)]
struct MathTemplate {
    category: &'static str,
    label: &'static str,
    latex: &'static str,
    help: &'static str,
}

const MATH_TEMPLATES: &[MathTemplate] = &[
    MathTemplate {
        category: "Arithmetic",
        label: "Fraction",
        latex: r"\\frac{a}{b}",
        help: "Basic fraction.",
    },
    MathTemplate {
        category: "Arithmetic",
        label: "Square root",
        latex: r"\\sqrt{x}",
        help: "Square root.",
    },
    MathTemplate {
        category: "Arithmetic",
        label: "Power",
        latex: r"x^{n}",
        help: "Superscript/power.",
    },
    MathTemplate {
        category: "Algebra",
        label: "Summation",
        latex: r"\\sum_{k=1}^{n} k",
        help: "Finite sum with bounds.",
    },
    MathTemplate {
        category: "Algebra",
        label: "Product",
        latex: r"\\prod_{k=1}^{n} k",
        help: "Finite product with bounds.",
    },
    MathTemplate {
        category: "Calculus",
        label: "Definite integral",
        latex: r"\\int_{a}^{b} f(x)\\,dx",
        help: "Definite integral with differential.",
    },
    MathTemplate {
        category: "Calculus",
        label: "Derivative",
        latex: r"\\frac{d}{dx} f(x)",
        help: "Ordinary derivative.",
    },
    MathTemplate {
        category: "Linear Algebra",
        label: "2x2 matrix",
        latex: r"\\begin{bmatrix} a & b \\\\ c & d \\end{bmatrix}",
        help: "Use \\\\ for row breaks.",
    },
    MathTemplate {
        category: "Linear Algebra",
        label: "Cases",
        latex: r"f(x)=\\begin{cases}x^2 & x \\ge 0\\\\-x & x < 0\\end{cases}",
        help: "Piecewise function.",
    },
    MathTemplate {
        category: "Logic & Sets",
        label: "Set builder",
        latex: r"\\{x \\in \\mathbb{R} \\mid x > 0\\}",
        help: "Set-builder notation.",
    },
    MathTemplate {
        category: "Logic & Sets",
        label: "Implication",
        latex: r"P \\Rightarrow Q",
        help: "Logical implication.",
    },
    MathTemplate {
        category: "Greek",
        label: "Greek symbols",
        latex: r"\\alpha, \\beta, \\gamma, \\Delta, \\Omega",
        help: "Common Greek letters.",
    },
];

fn available_categories() -> Vec<&'static str> {
    let mut categories: Vec<&'static str> = Vec::new();
    for template in MATH_TEMPLATES {
        if !categories.contains(&template.category) {
            categories.push(template.category);
        }
    }
    categories
}

fn templates_for_category(category: &str) -> Vec<&'static MathTemplate> {
    MATH_TEMPLATES
        .iter()
        .filter(|item| item.category == category)
        .collect()
}

fn validate_math_expression(expr: &str, mode: MathMode) -> Result<(), String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err("Math expression is empty.".to_string());
    }

    let ctx = KATEX_CONTEXT.get_or_init(KatexContext::default);
    let settings = KatexSettings::builder()
        .display_mode(mode == MathMode::Block)
        .output(OutputFormat::Mathml)
        .build();

    katex_render(ctx, trimmed, &settings)
        .map(|_| ())
        .map_err(|err| err.to_string())
}

fn render_math_preview_document(
    expr: &str,
    mode: MathMode,
    theme_class: &str,
) -> Result<String, String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Ok(empty_preview_document(theme_class));
    }

    let ctx = KATEX_CONTEXT.get_or_init(KatexContext::default);
    let settings = KatexSettings::builder()
        .display_mode(mode == MathMode::Block)
        .output(OutputFormat::HtmlAndMathml)
        .build();

    let rendered = katex_render(ctx, trimmed, &settings).map_err(|e| e.to_string())?;

    let body = if mode == MathMode::Block {
        format!("<div class='math-preview-block'>{}</div>", rendered)
    } else {
        format!(
            "<p class='math-preview-inline'>Inline preview: {}</p>",
            rendered
        )
    };

    let text_color = preview_text_color_for_theme(theme_class);
    let scrollbar = preview_scrollbar_css(theme_class);
    let css = format!(
        "
body {{
    color: {text_color};
    padding: 8px;
    margin: 0;
}}
.math-preview-inline, .math-preview-block {{
    font-size: 16px;
    line-height: 1.5;
    margin: 0;
    color: {text_color};
}}
.math-preview-block {{
    padding: 0;
    border-radius: 6px;
    background: transparent;
}}
.math-preview-inline .katex,
.math-preview-block .katex,
.math-preview-inline .katex *,
.math-preview-block .katex * {{
    color: {text_color};
}}
{scrollbar}"
    );

    let mode = if theme_class == "marco-theme-dark" {
        "dark"
    } else {
        "light"
    };
    let (background_hex, _) = preview_background_for_theme(theme_class);
    Ok(crate::components::viewer::backend::wrap_html_document(
        &body,
        &css,
        mode,
        Some(background_hex),
    ))
}

fn empty_preview_document(theme_class: &str) -> String {
    let text_color = preview_text_color_for_theme(theme_class);
    let body = format!(
        "<p style='margin:0;opacity:0.75;color:{}'>Preview will appear here as you type.</p>",
        text_color
    );
    let css = format!(
        "body {{ color: {}; padding: 8px; margin: 0; }}{}",
        text_color,
        preview_scrollbar_css(theme_class)
    );
    let mode = if theme_class == "marco-theme-dark" {
        "dark"
    } else {
        "light"
    };
    let (background_hex, _) = preview_background_for_theme(theme_class);
    crate::components::viewer::backend::wrap_html_document(&body, &css, mode, Some(background_hex))
}

fn error_preview_document(theme_class: &str, err: &str) -> String {
    let escaped = htmlescape::encode_minimal(err);
    let body = format!(
        "<div style='margin:8px;color:#d14343'><strong>Preview parse warning</strong><br><code>{}</code></div>",
        escaped
    );
    let mode = if theme_class == "marco-theme-dark" {
        "dark"
    } else {
        "light"
    };
    let (background_hex, _) = preview_background_for_theme(theme_class);
    crate::components::viewer::backend::wrap_html_document(
        &body,
        preview_scrollbar_css(theme_class),
        mode,
        Some(background_hex),
    )
}

/// Generate `::-webkit-scrollbar` CSS that visually matches the GTK dialog
/// scrollbar (styled via `dialog.rs`). Injecting this into the WebView preview
/// HTML avoids the jarring mismatch between the browser's default native
/// scrollbar and the rest of the dialog.
fn preview_scrollbar_css(theme_class: &str) -> &'static str {
    if theme_class == "marco-theme-dark" {
        "
::-webkit-scrollbar { width: 12px; height: 12px; }
::-webkit-scrollbar-track { background: #252526; }
::-webkit-scrollbar-thumb { background: #3A3F44; border-radius: 0; }
::-webkit-scrollbar-thumb:hover { background: #4A4F55; }
"
    } else {
        "
::-webkit-scrollbar { width: 12px; height: 12px; }
::-webkit-scrollbar-track { background: #F0F0F0; }
::-webkit-scrollbar-thumb { background: #D0D4D8; border-radius: 0; }
::-webkit-scrollbar-thumb:hover { background: #C2C7CC; }
"
    }
}

fn preview_background_for_theme(theme_class: &str) -> (&'static str, gtk4::gdk::RGBA) {
    if theme_class == "marco-theme-dark" {
        (
            "#1e1e1e",
            gtk4::gdk::RGBA::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0, 1.0),
        )
    } else {
        ("#ffffff", gtk4::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0))
    }
}

fn preview_text_color_for_theme(theme_class: &str) -> &'static str {
    if theme_class == "marco-theme-dark" {
        "#e6e6e6"
    } else {
        "#1f2937"
    }
}

#[cfg(target_os = "linux")]
fn load_preview_document(surface: &PreviewSurface, html: String) {
    crate::components::viewer::backend::load_html_when_ready(surface, html, None);
}

#[cfg(target_os = "windows")]
fn load_preview_document(surface: &PreviewSurface, html: String) {
    surface.load_html_with_base(&html, None);
}

fn build_math_markdown(expr: &str, mode: MathMode) -> String {
    let trimmed = expr.trim();
    match mode {
        MathMode::Inline => format!("${}$", trimmed),
        MathMode::Block => format!("$$\n{}\n$$", trimmed),
    }
}

fn insert_math_at_cursor(buffer: &Buffer, view: &View, markdown: &str, mode: MathMode) {
    let insert_mark = buffer.get_insert();
    let mut cursor_iter = buffer.iter_at_mark(&insert_mark);

    let insertion_text = if mode == MathMode::Block {
        let at_line_start = cursor_iter.starts_line();
        if at_line_start {
            format!("{}\n", markdown)
        } else {
            format!("\n{}\n", markdown)
        }
    } else {
        markdown.to_string()
    };

    buffer.insert(&mut cursor_iter, &insertion_text);

    let insert_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&insert_mark);
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

pub fn show_insert_math_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    let parent_widget = parent.upcast_ref::<gtk4::Widget>();
    let initial_theme_class = if parent_widget.has_css_class("marco-theme-dark") {
        "marco-theme-dark".to_string()
    } else {
        "marco-theme-light".to_string()
    };
    let theme_class_state = std::rc::Rc::new(std::cell::RefCell::new(initial_theme_class));

    let dialog = Window::builder()
        .modal(false)
        .transient_for(parent)
        .default_width(640)
        .default_height(500)
        .resizable(true)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(&theme_class_state.borrow());

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert Math",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert Math dialog requires a close button");

    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    let root = Box::new(Orientation::Vertical, 0);

    let vbox = Box::new(Orientation::Vertical, 8);
    vbox.add_css_class("marco-dialog-content");
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(0);

    let mode_label = Label::new(Some("Math Mode"));
    mode_label.set_halign(Align::Start);
    mode_label.add_css_class("marco-dialog-section-label");
    mode_label.add_css_class("marco-dialog-section-label-strong");
    vbox.append(&mode_label);

    let mode_row = Box::new(Orientation::Horizontal, 10);
    mode_row.set_margin_start(4);

    let inline_radio = CheckButton::with_label("Inline ($...$)");
    inline_radio.add_css_class("marco-radio");
    inline_radio.set_active(false);

    let block_radio = CheckButton::with_label("Block ($$...$$)");
    block_radio.add_css_class("marco-radio");
    block_radio.set_group(Some(&inline_radio));
    block_radio.set_active(true);

    mode_row.append(&inline_radio);
    mode_row.append(&block_radio);
    vbox.append(&mode_row);

    let template_label = Label::new(Some("Templates"));
    template_label.set_halign(Align::Start);
    template_label.add_css_class("marco-dialog-section-label");
    vbox.append(&template_label);

    let template_row = Box::new(Orientation::Horizontal, 8);
    template_row.set_margin_start(4);

    let categories = std::rc::Rc::new(available_categories());
    let category_refs: Vec<&str> = categories.iter().map(|s| s.as_ref()).collect();
    let category_list = StringList::new(&category_refs);
    let category_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let category_combo = DropDown::new(Some(category_list.clone()), Some(category_expression));
    category_combo.add_css_class("marco-dropdown");
    category_combo.add_css_class(&theme_class_state.borrow());
    let initial_category_index = categories
        .iter()
        .position(|c| *c == "Arithmetic")
        .unwrap_or(0) as u32;
    category_combo.set_selected(initial_category_index);

    let snippet_list = StringList::new(&[]);
    let snippet_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let snippet_combo = DropDown::new(Some(snippet_list.clone()), Some(snippet_expression));
    snippet_combo.add_css_class("marco-dropdown");
    snippet_combo.add_css_class(&theme_class_state.borrow());
    snippet_combo.set_hexpand(true);

    let use_template_button = Button::with_label("Use Template");
    use_template_button.add_css_class("marco-btn");
    use_template_button.add_css_class("marco-btn-blue");

    template_row.append(&category_combo);
    template_row.append(&snippet_combo);
    template_row.append(&use_template_button);
    vbox.append(&template_row);

    let template_help = Label::new(None);
    template_help.set_halign(Align::Start);
    template_help.set_xalign(0.0);
    template_help.set_wrap(true);
    template_help.add_css_class("marco-dialog-option-desc");
    template_help.set_margin_start(4);
    vbox.append(&template_help);

    let expr_label = Label::new(Some("Expression"));
    expr_label.set_halign(Align::Start);
    expr_label.add_css_class("marco-dialog-section-label");
    vbox.append(&expr_label);

    let expr_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(140)
        .hexpand(true)
        .vexpand(true)
        .build();
    expr_scroll.add_css_class("marco-textfield-scroll");

    let expr_text = TextView::new();
    expr_text.set_wrap_mode(gtk4::WrapMode::WordChar);
    expr_text.set_hexpand(true);
    expr_text.set_vexpand(true);
    expr_text.add_css_class("marco-textfield-view");
    expr_scroll.set_child(Some(&expr_text));
    vbox.append(&expr_scroll);

    let preview_label = Label::new(Some("Live Preview"));
    preview_label.set_halign(Align::Start);
    preview_label.add_css_class("marco-dialog-section-label");
    vbox.append(&preview_label);

    let preview_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(120)
        .hexpand(true)
        .vexpand(true)
        .build();
    preview_scroll.add_css_class("marco-textfield-scroll");

    #[cfg(target_os = "linux")]
    let preview_surface: std::rc::Rc<PreviewSurface> = {
        let webview = webkit6::WebView::new();
        webview.set_hexpand(true);
        webview.set_vexpand(true);
        let (_, rgba) = preview_background_for_theme(&theme_class_state.borrow());
        webview.set_background_color(&rgba);
        preview_scroll.set_child(Some(&webview));
        std::rc::Rc::new(webview)
    };

    #[cfg(target_os = "windows")]
    let preview_surface: Option<std::rc::Rc<PreviewSurface>> = {
        if let Some(app_window) = parent.dynamic_cast_ref::<gtk4::ApplicationWindow>() {
            let webview =
                crate::components::viewer::wry_platform_webview::PlatformWebView::new(app_window);
            let (_, rgba) = preview_background_for_theme(&theme_class_state.borrow());
            webview.set_background_color_rgba(&rgba);
            let widget = webview.widget();
            widget.set_hexpand(true);
            widget.set_vexpand(true);
            preview_scroll.set_child(Some(&widget));
            Some(std::rc::Rc::new(webview))
        } else {
            let fallback = Label::new(Some(
                "Live browser preview is unavailable in this window on Windows. Math validation and insertion still work.",
            ));
            fallback.set_wrap(true);
            fallback.set_xalign(0.0);
            fallback.set_halign(Align::Start);
            fallback.set_margin_start(8);
            fallback.set_margin_end(8);
            fallback.set_margin_top(8);
            fallback.set_margin_bottom(8);
            preview_scroll.set_child(Some(&fallback));
            None
        }
    };

    vbox.append(&preview_scroll);

    let tip_label = Label::new(Some(
        "Tips: Use \\\\frac{a}{b}, \\\\sqrt{x}, \\\\sum_{i=1}^{n}. For multiline arrays/matrices, \\\\ is row break.",
    ));
    tip_label.set_halign(Align::Start);
    tip_label.set_xalign(0.0);
    tip_label.set_wrap(true);
    tip_label.add_css_class("marco-dialog-option-desc");
    vbox.append(&tip_label);

    let status_label = Label::new(Some("Status: waiting for input."));
    status_label.set_halign(Align::Start);
    status_label.set_xalign(0.0);
    status_label.set_wrap(true);
    status_label.add_css_class("marco-dialog-option-desc");
    vbox.append(&status_label);

    let cancel_button = Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let insert_button = Button::with_label("Insert");
    insert_button.add_css_class("marco-btn");
    insert_button.add_css_class("suggested-action");

    let button_box = Box::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);
    button_box.append(&cancel_button);
    button_box.append(&insert_button);

    let bottom_frame = gtk4::Frame::new(None);
    bottom_frame.add_css_class("marco-dialog-bottom-frame");
    bottom_frame.set_height_request(48);
    bottom_frame.set_vexpand(false);
    bottom_frame.set_margin_top(2);

    let bottom_inner = Box::new(Orientation::Horizontal, 0);
    bottom_inner.set_margin_start(8);
    bottom_inner.set_margin_end(8);
    bottom_inner.set_margin_top(4);
    bottom_inner.set_margin_bottom(4);
    bottom_inner.set_halign(Align::Fill);
    bottom_inner.set_valign(Align::Center);

    if is_debug_mode_enabled() {
        let backend_label = Label::new(Some(preview_backend_label()));
        backend_label.set_halign(Align::Start);
        backend_label.set_xalign(0.0);
        backend_label.set_wrap(true);
        backend_label.add_css_class("marco-dialog-option-desc");
        backend_label.set_margin_start(2);
        bottom_inner.append(&backend_label);
    }

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bottom_inner.append(&spacer);
    bottom_inner.append(&button_box);
    bottom_frame.set_child(Some(&bottom_inner));

    root.append(&vbox);
    root.append(&bottom_frame);
    dialog.set_child(Some(&root));

    let selected_templates =
        std::rc::Rc::new(std::cell::RefCell::new(Vec::<&'static MathTemplate>::new()));

    let refresh_templates = {
        let category_combo = category_combo.clone();
        let categories = categories.clone();
        let snippet_combo = snippet_combo.clone();
        let snippet_list = snippet_list.clone();
        let template_help = template_help.clone();
        let selected_templates = selected_templates.clone();

        move || {
            snippet_list.splice(0, snippet_list.n_items(), &[]);
            selected_templates.borrow_mut().clear();

            let category = categories
                .get(category_combo.selected() as usize)
                .cloned()
                .unwrap_or("Arithmetic");

            for item in templates_for_category(category) {
                snippet_list.append(item.label);
                selected_templates.borrow_mut().push(item);
            }

            snippet_combo.set_selected(0);

            if let Some(first) = selected_templates.borrow().first() {
                template_help.set_text(first.help);
            } else {
                template_help.set_text("No templates in this category.");
            }
        }
    };

    let update_status = {
        let expr_text = expr_text.clone();
        let block_radio = block_radio.clone();
        let status_label = status_label.clone();
        let insert_button = insert_button.clone();
        let theme_class_state = theme_class_state.clone();

        #[cfg(target_os = "linux")]
        let preview_surface = preview_surface.clone();

        #[cfg(target_os = "windows")]
        let preview_surface = preview_surface.clone();

        move || {
            let theme_class = theme_class_state.borrow().clone();
            let buf = expr_text.buffer();
            let start = buf.start_iter();
            let end = buf.end_iter();
            let expr = buf.text(&start, &end, false).to_string();

            if expr.trim().is_empty() {
                status_label.set_text("Status: waiting for input.");
                insert_button.set_sensitive(false);

                #[cfg(target_os = "linux")]
                load_preview_document(&preview_surface, empty_preview_document(&theme_class));

                #[cfg(target_os = "windows")]
                if let Some(surface) = &preview_surface {
                    load_preview_document(surface, empty_preview_document(&theme_class));
                }

                return;
            }

            let mode = if block_radio.is_active() {
                MathMode::Block
            } else {
                MathMode::Inline
            };

            match validate_math_expression(&expr, mode) {
                Ok(()) => {
                    status_label.set_text("Status: valid KaTeX expression.");
                    insert_button.set_sensitive(true);

                    if let Ok(html) = render_math_preview_document(&expr, mode, &theme_class) {
                        #[cfg(target_os = "linux")]
                        load_preview_document(&preview_surface, html);

                        #[cfg(target_os = "windows")]
                        if let Some(surface) = &preview_surface {
                            load_preview_document(surface, html);
                        }
                    }
                }
                Err(err) => {
                    status_label.set_text(&format!("Status: parse warning: {}", err));
                    // Keep insert enabled for power users who may want to insert incomplete math.
                    insert_button.set_sensitive(true);

                    #[cfg(target_os = "linux")]
                    load_preview_document(
                        &preview_surface,
                        error_preview_document(&theme_class, &err),
                    );

                    #[cfg(target_os = "windows")]
                    if let Some(surface) = &preview_surface {
                        load_preview_document(surface, error_preview_document(&theme_class, &err));
                    }
                }
            }
        }
    };

    refresh_templates();
    update_status();

    {
        let refresh_templates = refresh_templates.clone();
        category_combo.connect_selected_notify(move |_| {
            refresh_templates();
        });
    }

    {
        let selected_templates = selected_templates.clone();
        let template_help = template_help.clone();
        snippet_combo.connect_selected_notify(move |combo| {
            if let Some(item) = selected_templates.borrow().get(combo.selected() as usize) {
                template_help.set_text(item.help);
            }
        });
    }

    {
        let expr_text = expr_text.clone();
        let snippet_combo = snippet_combo.clone();
        let selected_templates = selected_templates.clone();
        let update_status = update_status.clone();

        use_template_button.connect_clicked(move |_| {
            let templates = selected_templates.borrow();
            let Some(item) = templates.get(snippet_combo.selected() as usize) else {
                return;
            };
            let latex = item.latex.to_string();

            let buf = expr_text.buffer();
            buf.set_text(&latex);
            update_status();
            expr_text.grab_focus();
        });
    }

    {
        let update_status = update_status.clone();
        inline_radio.connect_toggled(move |_| {
            update_status();
        });
    }

    {
        let update_status = update_status.clone();
        block_radio.connect_toggled(move |_| {
            update_status();
        });
    }

    {
        let update_status = update_status.clone();
        let expr_buffer = expr_text.buffer();
        expr_buffer.connect_changed(move |_| {
            update_status();
        });
    }

    {
        let dialog = dialog.clone();
        let parent_widget = parent_widget.clone();
        let theme_class_state = theme_class_state.clone();
        let category_combo = category_combo.clone();
        let snippet_combo = snippet_combo.clone();
        let update_status = update_status.clone();
        #[cfg(target_os = "linux")]
        let preview_surface = preview_surface.clone();
        #[cfg(target_os = "windows")]
        let preview_surface = preview_surface.clone();

        parent_widget.connect_notify_local(Some("css-classes"), move |widget, _| {
            let next_theme = if widget.has_css_class("marco-theme-dark") {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            };

            {
                let mut state = theme_class_state.borrow_mut();
                if state.as_str() == next_theme {
                    return;
                }
                *state = next_theme.to_string();
            }

            dialog.remove_css_class("marco-theme-dark");
            dialog.remove_css_class("marco-theme-light");
            dialog.add_css_class(next_theme);

            category_combo.remove_css_class("marco-theme-dark");
            category_combo.remove_css_class("marco-theme-light");
            category_combo.add_css_class(next_theme);

            snippet_combo.remove_css_class("marco-theme-dark");
            snippet_combo.remove_css_class("marco-theme-light");
            snippet_combo.add_css_class(next_theme);

            let (_, rgba) = preview_background_for_theme(next_theme);
            #[cfg(target_os = "linux")]
            preview_surface.set_background_color(&rgba);
            #[cfg(target_os = "windows")]
            if let Some(surface) = &preview_surface {
                surface.set_background_color_rgba(&rgba);
            }

            update_status();
        });
    }

    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    {
        let expr_text = expr_text.clone();
        let block_radio = block_radio.clone();
        let dialog_weak = dialog_weak.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();

        insert_button.connect_clicked(move |_| {
            let buf = expr_text.buffer();
            let start = buf.start_iter();
            let end = buf.end_iter();
            let expr = buf.text(&start, &end, false).to_string();
            if expr.trim().is_empty() {
                return;
            }

            let mode = if block_radio.is_active() {
                MathMode::Block
            } else {
                MathMode::Inline
            };

            let markdown = build_math_markdown(&expr, mode);
            insert_math_at_cursor(&editor_buffer, &editor_view, &markdown, mode);

            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        cancel_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        close_button.connect_clicked(move |_| {
            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    {
        let key_controller = gtk4::EventControllerKey::new();
        let dialog_weak = dialog_weak.clone();
        key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
            if key == gtk4::gdk::Key::Escape {
                if let Some(dialog) = dialog_weak.upgrade() {
                    dialog.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        dialog.add_controller(key_controller);
    }

    expr_text.grab_focus();
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_build_math_markdown_inline() {
        let got = build_math_markdown("x^2", MathMode::Inline);
        assert_eq!(got, "$x^2$");
    }

    #[test]
    fn smoke_test_build_math_markdown_block() {
        let got = build_math_markdown("x^2 + y^2", MathMode::Block);
        assert_eq!(got, "$$\nx^2 + y^2\n$$");
    }

    #[test]
    fn smoke_test_validate_math_expression_valid() {
        assert!(validate_math_expression(r"\\frac{a}{b}", MathMode::Inline).is_ok());
    }
}
