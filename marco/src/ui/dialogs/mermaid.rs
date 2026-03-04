//! Insert Mermaid Diagram Dialog
//!
//! A beginner-friendly + power-user dialog for inserting Mermaid diagrams.
//! - Type selector with starter templates (Flowchart, Sequence, Pie, Git, Class, Custom)
//! - Monospace source editor
//! - Live preview rendered by `core::render::diagram::render_mermaid_diagram`
//!   (pure-Rust mermaid-rs-renderer — no JS engine required)
//! - Debounced preview updates (350 ms after last keystroke)
//! - Inline error feedback without clearing the preview

use gtk4::{
    glib, prelude::*, Align, Box, Button, Expression, Frame, Label, Orientation, PolicyType,
    PropertyExpression, ScrolledWindow, StringList, StringObject, TextView, Window,
};
use sourceview5::{Buffer, View};
use std::{cell::Cell, rc::Rc, time::Duration};

#[cfg(target_os = "linux")]
use webkit6::prelude::WebViewExt;

#[cfg(target_os = "linux")]
type PreviewSurface = webkit6::WebView;

#[cfg(target_os = "windows")]
type PreviewSurface = crate::components::viewer::wry_platform_webview::PlatformWebView;

// ── Diagram types & templates ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagramType {
    Flowchart,
    Sequence,
    Pie,
    GitGraph,
    Class,
    Custom,
}

const DIAGRAM_TYPES: &[DiagramType] = &[
    DiagramType::Flowchart,
    DiagramType::Sequence,
    DiagramType::Pie,
    DiagramType::GitGraph,
    DiagramType::Class,
    DiagramType::Custom,
];

fn diagram_label(kind: DiagramType) -> &'static str {
    match kind {
        DiagramType::Flowchart => "Flowchart",
        DiagramType::Sequence => "Sequence",
        DiagramType::Pie => "Pie chart",
        DiagramType::GitGraph => "Git graph",
        DiagramType::Class => "Class diagram",
        DiagramType::Custom => "Custom",
    }
}

fn diagram_hint(kind: DiagramType) -> &'static str {
    match kind {
        DiagramType::Flowchart => {
            "Directed or undirected graph. Use TD (top-down), LR (left-right), BT, RL."
        }
        DiagramType::Sequence => {
            "Message flows between participants. Use -> for solid lines, --> for dashed."
        }
        DiagramType::Pie => "Proportional chart split as labelled slices.",
        DiagramType::GitGraph => "Branch and merge history. Use commit, branch, checkout, merge.",
        DiagramType::Class => "UML class diagram. Define classes, attributes and relationships.",
        DiagramType::Custom => "Write your own Mermaid diagram from scratch.",
    }
}

fn diagram_starter(kind: DiagramType) -> &'static str {
    match kind {
        DiagramType::Flowchart => {
            "flowchart TD\n    A[Start] --> B{Decision?}\n    B -- Yes --> C[Process]\n    B -- No  --> D[End]\n    C --> D"
        }
        DiagramType::Sequence => {
            "sequenceDiagram\n    Alice->>Bob: Hello Bob!\n    Bob-->>Alice: Hi Alice!\n    Alice->>Bob: How are you?\n    Bob-->>Alice: I am fine, thanks!"
        }
        DiagramType::Pie => {
            "pie title Browser Usage\n    \"Chrome\"  : 62.6\n    \"Firefox\" : 7.2\n    \"Edge\"    : 4.3\n    \"Safari\"  : 19.8\n    \"Other\"   : 6.1"
        }
        DiagramType::GitGraph => {
            "gitGraph\n    commit id: \"Init\"\n    branch feature\n    checkout feature\n    commit id: \"Add feature\"\n    commit id: \"Fix bug\"\n    checkout main\n    merge feature id: \"Merge\""
        }
        DiagramType::Class => {
            "classDiagram\n    class Animal {\n        +String name\n        +speak() String\n    }\n    class Dog {\n        +fetch()\n    }\n    class Cat {\n        +purr()\n    }\n    Animal <|-- Dog\n    Animal <|-- Cat"
        }
        DiagramType::Custom => "",
    }
}

fn diagram_type_from_index(idx: u32) -> DiagramType {
    DIAGRAM_TYPES
        .get(idx as usize)
        .copied()
        .unwrap_or(DiagramType::Custom)
}

// ── Markdown generation ───────────────────────────────────────────────────────

fn build_mermaid_markdown(source: &str) -> String {
    let trimmed = source.trim();
    format!("```mermaid\n{}\n```", trimmed)
}

// ── Cursor insertion ──────────────────────────────────────────────────────────

fn insert_mermaid_at_cursor(buffer: &Buffer, view: &View, markdown: &str) {
    let cursor_mark = buffer.get_insert();
    let mut cursor_iter = buffer.iter_at_mark(&cursor_mark);

    // Detect current-line indentation for block alignment.
    let line_start = {
        let mut iter = cursor_iter;
        iter.set_line_offset(0);
        iter
    };
    let line_end = {
        let mut iter = cursor_iter;
        if !iter.ends_line() {
            iter.forward_to_line_end();
        }
        iter
    };
    let current_line = buffer.text(&line_start, &line_end, false);
    let indent: String = current_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();

    // Insert on its own paragraph — ensure blank lines around the block.
    let at_line_start = cursor_iter.starts_line();
    let needs_leading_newline = !at_line_start;
    let prefix = if needs_leading_newline { "\n" } else { "" };

    let indented = if indent.is_empty() {
        format!("{prefix}{markdown}\n")
    } else {
        let indented_lines: Vec<String> = markdown
            .lines()
            .map(|l| {
                if l.is_empty() {
                    String::new()
                } else {
                    format!("{indent}{l}")
                }
            })
            .collect();
        format!("{prefix}{}\n", indented_lines.join("\n"))
    };

    buffer.insert(&mut cursor_iter, &indented);

    let end_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&end_mark);
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

// ── Theme helpers ─────────────────────────────────────────────────────────────

/// Generate `::-webkit-scrollbar` CSS that visually matches the GTK dialog
/// scrollbar (styled via `dialog.rs`). Injecting this into the WebView preview
/// HTML avoids the jarring mismatch between the browser's default native
/// scrollbar and the rest of the dialog.
fn preview_scrollbar_css(theme_class: &str) -> String {
    let (track, thumb, thumb_hover) = if theme_class == "marco-theme-dark" {
        ("#252526", "#3A3F44", "#4A4F55")
    } else {
        ("#F0F0F0", "#D0D4D8", "#C2C7CC")
    };
    format!(
        "
::-webkit-scrollbar {{ width: 12px; height: 12px; }}
::-webkit-scrollbar-track {{ background: {track}; }}
::-webkit-scrollbar-thumb {{ background: {thumb}; border-radius: 0; }}
::-webkit-scrollbar-thumb:hover {{ background: {thumb_hover}; }}
"
    )
}

fn preview_bg_for_theme(theme_class: &str) -> (&'static str, gtk4::gdk::RGBA) {
    if theme_class == "marco-theme-dark" {
        (
            "#1e1e1e",
            gtk4::gdk::RGBA::new(30.0 / 255.0, 30.0 / 255.0, 30.0 / 255.0, 1.0),
        )
    } else {
        ("#ffffff", gtk4::gdk::RGBA::new(1.0, 1.0, 1.0, 1.0))
    }
}

fn preview_text_color(theme_class: &str) -> &'static str {
    if theme_class == "marco-theme-dark" {
        "#e6e6e6"
    } else {
        "#1f2937"
    }
}

// ── Preview HTML builders ─────────────────────────────────────────────────────

fn mermaid_preview_html(svg: &str, theme_class: &str) -> String {
    let text_color = preview_text_color(theme_class);
    let body = format!("<div style='text-align:center;padding:8px;'>{}</div>", svg);
    let css = format!(
        "body {{ margin: 0; padding: 0; color: {}; background: transparent; }}{}",
        text_color,
        preview_scrollbar_css(theme_class)
    );
    let mode = if theme_class == "marco-theme-dark" {
        "dark"
    } else {
        "light"
    };
    let (bg, _) = preview_bg_for_theme(theme_class);
    crate::components::viewer::backend::wrap_html_document(&body, &css, mode, Some(bg))
}

fn empty_preview_html(theme_class: &str) -> String {
    let text_color = preview_text_color(theme_class);
    let body = format!(
        "<p style='margin:0;opacity:0.6;color:{};font-size:13px;text-align:center;padding:16px'>Preview will appear here as you type.</p>",
        text_color
    );
    let css = format!(
        "body {{ margin:0; background: transparent; color: {}; }}{}",
        text_color,
        preview_scrollbar_css(theme_class)
    );
    let mode = if theme_class == "marco-theme-dark" {
        "dark"
    } else {
        "light"
    };
    let (bg, _) = preview_bg_for_theme(theme_class);
    crate::components::viewer::backend::wrap_html_document(&body, &css, mode, Some(bg))
}

// ── Platform-specific preview loading ─────────────────────────────────────────

#[cfg(target_os = "linux")]
fn load_preview(surface: &PreviewSurface, html: String) {
    crate::components::viewer::backend::load_html_when_ready(surface, html, None);
}

#[cfg(target_os = "windows")]
fn load_preview(surface: &PreviewSurface, html: String) {
    surface.load_html_with_base(&html, None);
}

// ── Main dialog ───────────────────────────────────────────────────────────────

pub fn show_insert_mermaid_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    let parent_widget = parent.upcast_ref::<gtk4::Widget>();
    let initial_theme = if parent_widget.has_css_class("marco-theme-dark") {
        "marco-theme-dark".to_string()
    } else {
        "marco-theme-light".to_string()
    };
    let theme_state = Rc::new(std::cell::RefCell::new(initial_theme));

    let dialog = Window::builder()
        .modal(false)
        .transient_for(parent)
        .default_width(580)
        .default_height(560)
        .resizable(true)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(&theme_state.borrow().clone());

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert Mermaid Diagram",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );
    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert Mermaid dialog requires a close button");
    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    // ── Root layout ────────────────────────────────────────────────────────────
    let root = Box::new(Orientation::Vertical, 0);

    let vbox = Box::new(Orientation::Vertical, 8);
    vbox.add_css_class("marco-dialog-content");
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(8);
    vbox.set_margin_bottom(0);

    // ── Diagram type selector ──────────────────────────────────────────────────
    let type_section_label = Label::new(Some("Diagram type"));
    type_section_label.set_halign(Align::Start);
    type_section_label.add_css_class("marco-dialog-section-label");
    type_section_label.add_css_class("marco-dialog-section-label-strong");
    vbox.append(&type_section_label);

    let type_labels: Vec<&str> = DIAGRAM_TYPES.iter().map(|k| diagram_label(*k)).collect();
    let type_list = StringList::new(&type_labels);
    let type_expression =
        PropertyExpression::new(StringObject::static_type(), None::<&Expression>, "string");
    let type_combo = gtk4::DropDown::new(Some(type_list), Some(type_expression));
    type_combo.add_css_class("marco-dropdown");
    type_combo.add_css_class(&theme_state.borrow().clone());
    type_combo.set_selected(0);
    type_combo.set_hexpand(true);

    let use_template_button = Button::with_label("Use Template");
    use_template_button.add_css_class("marco-btn");
    use_template_button.add_css_class("marco-btn-blue");

    let type_row = Box::new(Orientation::Horizontal, 8);
    type_row.set_margin_start(4);
    type_row.append(&type_combo);
    type_row.append(&use_template_button);
    vbox.append(&type_row);

    let hint_label = Label::new(Some(diagram_hint(DiagramType::Flowchart)));
    hint_label.set_halign(Align::Start);
    hint_label.set_xalign(0.0);
    hint_label.set_wrap(true);
    hint_label.add_css_class("marco-dialog-option-desc");
    hint_label.set_margin_start(4);
    vbox.append(&hint_label);

    // ── Source editor ──────────────────────────────────────────────────────────
    let source_section_label = Label::new(Some("Source"));
    source_section_label.set_halign(Align::Start);
    source_section_label.add_css_class("marco-dialog-section-label");
    vbox.append(&source_section_label);

    let source_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(160)
        .hexpand(true)
        .vexpand(true)
        .build();
    source_scroll.add_css_class("marco-textfield-scroll");

    let source_view = TextView::new();
    // Use monospace font via dedicated CSS class.
    source_view.add_css_class("marco-textfield-view");
    source_view.add_css_class("marco-mermaid-source");
    source_view.set_wrap_mode(gtk4::WrapMode::None); // Mermaid source is typically line-oriented
    source_view.set_hexpand(true);
    source_view.set_vexpand(true);
    source_scroll.set_child(Some(&source_view));
    vbox.append(&source_scroll);

    // ── Error label (hidden until render fails) ────────────────────────────────
    let error_label = Label::new(None);
    error_label.set_halign(Align::Start);
    error_label.set_xalign(0.0);
    error_label.set_wrap(true);
    error_label.add_css_class("marco-mermaid-error-label");
    error_label.set_visible(false);
    vbox.append(&error_label);

    // ── Live preview ───────────────────────────────────────────────────────────
    let preview_section_label = Label::new(Some("Live Preview"));
    preview_section_label.set_halign(Align::Start);
    preview_section_label.add_css_class("marco-dialog-section-label");
    vbox.append(&preview_section_label);

    let preview_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(140)
        .hexpand(true)
        .vexpand(true)
        .build();
    preview_scroll.add_css_class("marco-mermaid-preview-scroll");

    #[cfg(target_os = "linux")]
    let preview_surface: Rc<PreviewSurface> = {
        let wv = webkit6::WebView::new();
        wv.set_hexpand(true);
        wv.set_vexpand(true);
        let (_, rgba) = preview_bg_for_theme(&theme_state.borrow());
        wv.set_background_color(&rgba);
        preview_scroll.set_child(Some(&wv));
        Rc::new(wv)
    };

    #[cfg(target_os = "windows")]
    let preview_surface: Option<Rc<PreviewSurface>> = {
        if let Some(app_window) = parent.dynamic_cast_ref::<gtk4::ApplicationWindow>() {
            let wv =
                crate::components::viewer::wry_platform_webview::PlatformWebView::new(app_window);
            let (_, rgba) = preview_bg_for_theme(&theme_state.borrow());
            wv.set_background_color_rgba(&rgba);
            let widget = wv.widget();
            widget.set_hexpand(true);
            widget.set_vexpand(true);
            preview_scroll.set_child(Some(&widget));
            Some(Rc::new(wv))
        } else {
            let fallback = Label::new(Some(
                "Live preview is unavailable in this window on Windows.",
            ));
            fallback.set_wrap(true);
            fallback.set_margin_start(8);
            fallback.set_margin_top(8);
            preview_scroll.set_child(Some(&fallback));
            None
        }
    };

    vbox.append(&preview_scroll);

    // ── Bottom action bar ──────────────────────────────────────────────────────
    let cancel_button = Button::with_label("Cancel");
    cancel_button.add_css_class("marco-btn");
    cancel_button.add_css_class("marco-btn-yellow");

    let insert_button = Button::with_label("Insert");
    insert_button.add_css_class("marco-btn");
    insert_button.add_css_class("suggested-action");
    insert_button.set_sensitive(false); // disabled until source is non-empty

    let button_box = Box::new(Orientation::Horizontal, 8);
    button_box.set_halign(Align::End);
    button_box.append(&cancel_button);
    button_box.append(&insert_button);

    let bottom_frame = Frame::new(None);
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

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    bottom_inner.append(&spacer);
    bottom_inner.append(&button_box);
    bottom_frame.set_child(Some(&bottom_inner));

    root.append(&vbox);
    root.append(&bottom_frame);
    dialog.set_child(Some(&root));

    // ── Debounce timer ─────────────────────────────────────────────────────────
    let debounce_id: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

    // ── update_preview closure ─────────────────────────────────────────────────
    // Reads source, tries rendering, updates webview + error label.
    let update_preview = {
        let source_view = source_view.clone();
        let error_label = error_label.clone();
        let insert_button = insert_button.clone();
        let theme_state = theme_state.clone();

        #[cfg(target_os = "linux")]
        let preview_surface = preview_surface.clone();

        #[cfg(target_os = "windows")]
        let preview_surface = preview_surface.clone();

        move || {
            let theme = theme_state.borrow().clone();
            let buf = source_view.buffer();
            let start = buf.start_iter();
            let end = buf.end_iter();
            let source = buf.text(&start, &end, false).to_string();

            if source.trim().is_empty() {
                insert_button.set_sensitive(false);
                error_label.set_visible(false);

                #[cfg(target_os = "linux")]
                load_preview(&preview_surface, empty_preview_html(&theme));

                #[cfg(target_os = "windows")]
                if let Some(surface) = &preview_surface {
                    load_preview(surface, empty_preview_html(&theme));
                }
                return;
            }

            insert_button.set_sensitive(true);

            // theme_hint: mermaid-rs-renderer picks dark vs light colours.
            let theme_hint = if theme == "marco-theme-dark" {
                "dark"
            } else {
                "light"
            };
            match core::render::diagram::render_mermaid_diagram(source.trim(), theme_hint) {
                Ok(svg) => {
                    error_label.set_visible(false);

                    #[cfg(target_os = "linux")]
                    load_preview(&preview_surface, mermaid_preview_html(&svg, &theme));

                    #[cfg(target_os = "windows")]
                    if let Some(surface) = &preview_surface {
                        load_preview(surface, mermaid_preview_html(&svg, &theme));
                    }
                }
                Err(e) => {
                    // Show error but keep the previous valid SVG in the webview.
                    let msg = e.to_string();
                    let display = if msg.chars().count() > 200 {
                        format!("{}…", msg.chars().take(200).collect::<String>())
                    } else {
                        msg
                    };
                    error_label.set_text(&format!("⚠  {}", display));
                    error_label.set_visible(true);
                }
            }
        }
    };

    // ── Type selector: update hint + (optionally) source ──────────────────────

    // Track whether the current source matches the last loaded starter template,
    // so we know whether it's safe to overwrite it when the type changes.
    let last_starter: Rc<Cell<&'static str>> =
        Rc::new(Cell::new(diagram_starter(DiagramType::Flowchart)));

    // Load initial template into source view.
    source_view
        .buffer()
        .set_text(diagram_starter(DiagramType::Flowchart));

    {
        let hint_label = hint_label.clone();
        type_combo.connect_selected_notify(move |combo| {
            let kind = diagram_type_from_index(combo.selected());
            hint_label.set_text(diagram_hint(kind));
        });
    }

    {
        let type_combo = type_combo.clone();
        let source_view = source_view.clone();
        let last_starter = last_starter.clone();
        let update_preview = update_preview.clone();

        use_template_button.connect_clicked(move |_| {
            let kind = diagram_type_from_index(type_combo.selected());
            let starter = diagram_starter(kind);

            // Overwrite source — user explicitly requested the template.
            source_view.buffer().set_text(starter);
            last_starter.set(starter);
            update_preview();
            source_view.grab_focus();
        });
    }

    // ── Source changes → debounced preview ────────────────────────────────────

    {
        let source_buffer = source_view.buffer();
        let debounce_id = debounce_id.clone();
        let update_preview = update_preview.clone();

        source_buffer.connect_changed(move |_| {
            // Cancel pending render.
            if let Some(id) = debounce_id.take() {
                crate::logic::signal_manager::safe_source_remove(id);
            }
            let update_preview = update_preview.clone();
            let debounce_id_inner = debounce_id.clone();
            let new_id = glib::timeout_add_local(Duration::from_millis(350), move || {
                debounce_id_inner.set(None);
                update_preview();
                glib::ControlFlow::Break
            });
            debounce_id.set(Some(new_id));
        });
    }

    // ── Theme switching ────────────────────────────────────────────────────────

    {
        let dialog = dialog.clone();
        let type_combo = type_combo.clone();
        let theme_state = theme_state.clone();
        let update_preview = update_preview.clone();

        #[cfg(target_os = "linux")]
        let preview_surface = preview_surface.clone();

        #[cfg(target_os = "windows")]
        let preview_surface = preview_surface.clone();

        parent_widget.connect_notify_local(Some("css-classes"), move |widget, _| {
            let next = if widget.has_css_class("marco-theme-dark") {
                "marco-theme-dark"
            } else {
                "marco-theme-light"
            };

            {
                let mut state = theme_state.borrow_mut();
                if state.as_str() == next {
                    return;
                }
                *state = next.to_string();
            }

            dialog.remove_css_class("marco-theme-dark");
            dialog.remove_css_class("marco-theme-light");
            dialog.add_css_class(next);

            type_combo.remove_css_class("marco-theme-dark");
            type_combo.remove_css_class("marco-theme-light");
            type_combo.add_css_class(next);

            let (_, rgba) = preview_bg_for_theme(next);

            #[cfg(target_os = "linux")]
            preview_surface.set_background_color(&rgba);

            #[cfg(target_os = "windows")]
            if let Some(surface) = &preview_surface {
                surface.set_background_color_rgba(&rgba);
            }

            update_preview();
        });
    }

    // ── Insert button ─────────────────────────────────────────────────────────

    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    {
        let source_view = source_view.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        let dialog_weak = dialog_weak.clone();

        insert_button.connect_clicked(move |_| {
            let buf = source_view.buffer();
            let start = buf.start_iter();
            let end = buf.end_iter();
            let source = buf.text(&start, &end, false).to_string();
            if source.trim().is_empty() {
                return;
            }
            let markdown = build_mermaid_markdown(&source);
            insert_mermaid_at_cursor(&editor_buffer, &editor_view, &markdown);
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
        });
    }

    // ── Cancel / close ────────────────────────────────────────────────────────

    {
        let dialog_weak = dialog_weak.clone();
        cancel_button.connect_clicked(move |_| {
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
        });
    }

    {
        let dialog_weak = dialog_weak.clone();
        close_button.connect_clicked(move |_| {
            if let Some(d) = dialog_weak.upgrade() {
                d.close();
            }
        });
    }

    {
        let key_controller = gtk4::EventControllerKey::new();
        let dialog_weak = dialog_weak.clone();
        key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
            if key == gtk4::gdk::Key::Escape {
                if let Some(d) = dialog_weak.upgrade() {
                    d.close();
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        dialog.add_controller(key_controller);
    }

    // Initial render pass using the starter template.
    update_preview();

    // Place focus in the source editor so the user can type immediately.
    source_view.grab_focus();
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── diagram_starter ───────────────────────────────────────────────────────

    #[test]
    fn smoke_diagram_starter_flowchart_contains_keyword() {
        let s = diagram_starter(DiagramType::Flowchart);
        assert!(s.starts_with("flowchart"));
    }

    #[test]
    fn smoke_diagram_starter_sequence_contains_keyword() {
        let s = diagram_starter(DiagramType::Sequence);
        assert!(s.contains("sequenceDiagram"));
    }

    #[test]
    fn smoke_diagram_starter_pie_contains_keyword() {
        let s = diagram_starter(DiagramType::Pie);
        assert!(s.starts_with("pie"));
    }

    #[test]
    fn smoke_diagram_starter_git_contains_keyword() {
        let s = diagram_starter(DiagramType::GitGraph);
        assert!(s.starts_with("gitGraph"));
    }

    #[test]
    fn smoke_diagram_starter_class_contains_keyword() {
        let s = diagram_starter(DiagramType::Class);
        assert!(s.contains("classDiagram"));
    }

    #[test]
    fn smoke_diagram_starter_custom_is_blank() {
        assert_eq!(diagram_starter(DiagramType::Custom), "");
    }

    // ── build_mermaid_markdown ────────────────────────────────────────────────

    #[test]
    fn smoke_build_mermaid_markdown_basic() {
        let md = build_mermaid_markdown("flowchart TD\n    A --> B");
        assert!(md.starts_with("```mermaid\n"));
        assert!(md.ends_with("\n```"));
        assert!(md.contains("flowchart TD"));
    }

    #[test]
    fn smoke_build_mermaid_markdown_trims_whitespace() {
        let md = build_mermaid_markdown("  flowchart TD  ");
        assert!(md.contains("flowchart TD"));
        assert!(!md.contains("  flowchart"));
    }

    // ── diagram_type_from_index ───────────────────────────────────────────────

    #[test]
    fn smoke_diagram_type_from_index_known() {
        assert_eq!(diagram_type_from_index(0), DiagramType::Flowchart);
        assert_eq!(diagram_type_from_index(5), DiagramType::Custom);
    }

    #[test]
    fn smoke_diagram_type_from_index_out_of_range_returns_custom() {
        assert_eq!(diagram_type_from_index(99), DiagramType::Custom);
    }

    // ── hint / label ──────────────────────────────────────────────────────────

    #[test]
    fn smoke_all_types_have_hint_and_label() {
        for &kind in DIAGRAM_TYPES {
            assert!(!diagram_label(kind).is_empty());
            assert!(!diagram_hint(kind).is_empty());
        }
    }
}
