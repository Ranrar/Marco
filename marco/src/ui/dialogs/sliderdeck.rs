//! Insert Slider Deck Dialog
//!
//! Provides a slider-deck builder dialog using GTK4 ListView.
//! Each slide has free-form markdown content; slides are separated by `---`.
//! An optional auto-advance timer can be set (0 = disabled).

use gtk4::{
    gio, glib, prelude::*, Align, Box, Button, Frame, Label, ListView, Orientation, PolicyType,
    ScrolledWindow, SignalListItemFactory, SingleSelection, SpinButton, Stack, TextView, Window,
};
use sourceview5::{Buffer, View};
use std::{cell::Cell, rc::Rc};

const MAX_SLIDES: u32 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
struct SlideItem {
    content: String,
}

fn selected_position(selection: &SingleSelection) -> Option<u32> {
    let pos = selection.selected();
    if pos == gtk4::INVALID_LIST_POSITION {
        None
    } else {
        Some(pos)
    }
}

fn slide_object_at(store: &gio::ListStore, position: u32) -> Option<glib::BoxedAnyObject> {
    store
        .item(position)
        .and_then(|obj| obj.downcast::<glib::BoxedAnyObject>().ok())
}

/// Returns the display name for a slide at a given zero-based position.
fn slide_name(position: u32) -> String {
    format!("Slide {}", position + 1)
}

fn collect_slides(store: &gio::ListStore) -> Vec<SlideItem> {
    let mut slides = Vec::with_capacity(store.n_items() as usize);
    for idx in 0..store.n_items() {
        if let Some(item_obj) = slide_object_at(store, idx) {
            let item = item_obj.borrow::<SlideItem>();
            slides.push(item.clone());
        }
    }
    slides
}

fn generate_sliders_markdown(slides: &[SlideItem], timer_seconds: u32) -> String {
    let source_slides = if slides.is_empty() {
        vec![SlideItem {
            content: String::new(),
        }]
    } else {
        slides.to_vec()
    };

    // Opening marker: `@slidestart` or `@slidestart:t<N>` when timer is active.
    let opening = if timer_seconds > 0 {
        format!("@slidestart:t{}", timer_seconds)
    } else {
        "@slidestart".to_string()
    };

    let mut parts: Vec<String> = vec![opening];

    for (idx, slide) in source_slides.iter().enumerate() {
        if idx > 0 {
            // Horizontal slide separator.
            parts.push("\n---".to_string());
        }

        if slide.content.trim().is_empty() {
            parts.push(String::new());
        } else {
            parts.push(slide.content.clone());
        }
    }

    parts.push("@slideend".to_string());
    parts.join("\n")
}

fn insert_sliders_at_cursor(buffer: &Buffer, view: &View, markdown: &str) {
    let cursor_mark = buffer.get_insert();
    let mut cursor_iter = buffer.iter_at_mark(&cursor_mark);

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
    let indent = current_line
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect::<String>();

    let indented_text = if indent.is_empty() {
        markdown.to_string()
    } else {
        markdown
            .lines()
            .map(|line| {
                if line.is_empty() {
                    String::new()
                } else {
                    format!("{}{}", indent, line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    buffer.insert(&mut cursor_iter, &indented_text);

    // Marks are invalidated after a buffer mutation; re-fetch to scroll to the new position.
    let end_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&end_mark);
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

pub fn show_insert_slider_deck_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    let theme_class = if parent.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let dialog = Window::builder()
        // Non-modal: user can scroll/read the document while building the deck.
        .modal(false)
        .transient_for(parent)
        .default_width(520)
        .default_height(480)
        .resizable(false)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(theme_class);

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert Slider Deck",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert Slider Deck dialog requires a close button");
    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    let store = gio::ListStore::new::<glib::BoxedAnyObject>();

    let selection = SingleSelection::new(Some(store.clone()));
    selection.set_autoselect(true);
    selection.set_can_unselect(false);
    selection.set_selected(0);

    // ── Timer state ────────────────────────────────────────────────────────────
    // Shared u32 for the timer value (0 = disabled).  Kept in sync with the
    // SpinButton via connect_value_changed.
    let timer_seconds: Rc<Cell<u32>> = Rc::new(Cell::new(3));

    // ── Root layout ────────────────────────────────────────────────────────────
    let root = Box::new(Orientation::Vertical, 0);

    let content_box = Box::new(Orientation::Vertical, 8);
    content_box.add_css_class("marco-dialog-content");
    content_box.set_margin_start(12);
    content_box.set_margin_end(12);
    content_box.set_margin_top(8);
    content_box.set_margin_bottom(0);

    // ── Deck options frame ─────────────────────────────────────────────────────
    let options_frame = Frame::new(Some("Deck options"));
    options_frame.add_css_class("marco-tabs-frame");

    let options_inner = Box::new(Orientation::Horizontal, 8);
    options_inner.set_margin_start(8);
    options_inner.set_margin_end(8);
    options_inner.set_margin_top(8);
    options_inner.set_margin_bottom(8);
    options_inner.set_valign(Align::Center);

    let timer_label = Label::new(Some("Auto-advance timer:"));
    timer_label.set_halign(Align::Start);

    // SpinButton: 0 = disabled, 1-60 seconds.
    let timer_spin = SpinButton::with_range(0.0, 60.0, 1.0);
    timer_spin.set_value(3.0);
    timer_spin.set_width_chars(4);
    timer_spin.set_tooltip_text(Some("0 = timer disabled"));

    // Display "0" as "off" / display non-zero as numeric via output signal.
    {
        let timer_seconds_clone = timer_seconds.clone();
        timer_spin.connect_value_changed(move |spin| {
            let v = spin.value().round() as u32;
            timer_seconds_clone.set(v);
        });
    }

    let seconds_label = Label::new(Some("seconds  (0 = off)"));
    seconds_label.set_halign(Align::Start);

    options_inner.append(&timer_label);
    options_inner.append(&timer_spin);
    options_inner.append(&seconds_label);
    options_frame.set_child(Some(&options_inner));
    content_box.append(&options_frame);

    // ── Slides list frame ──────────────────────────────────────────────────────
    let slides_frame = Frame::new(Some("Slides"));
    slides_frame.add_css_class("marco-tabs-frame");

    let slides_inner = Box::new(Orientation::Vertical, 8);
    slides_inner.set_margin_start(8);
    slides_inner.set_margin_end(8);
    slides_inner.set_margin_top(8);
    slides_inner.set_margin_bottom(8);

    let slides_factory = SignalListItemFactory::new();

    let list_view = ListView::new(Some(selection.clone()), Some(slides_factory.clone()));
    list_view.add_css_class("marco-tabs-listview");
    list_view.set_hexpand(true);
    list_view.set_vexpand(true);

    let slides_scroller = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(130)
        .hexpand(true)
        .vexpand(true)
        .build();
    slides_scroller.set_child(Some(&list_view));

    // Empty-state placeholder shown when the store has no items.
    let empty_label = Label::new(Some("No slides yet — click + Add Slide"));
    empty_label.set_halign(Align::Center);
    empty_label.set_valign(Align::Center);
    empty_label.add_css_class("marco-tabs-empty-label");

    let slides_stack = Stack::new();
    slides_stack.set_hexpand(true);
    slides_stack.set_vexpand(true);
    slides_stack.add_named(&empty_label, Some("empty"));
    slides_stack.add_named(&slides_scroller, Some("list"));
    slides_stack.set_visible_child_name("empty");

    slides_inner.append(&slides_stack);

    let add_button = Button::with_label("+ Add Slide");
    add_button.add_css_class("marco-btn");
    add_button.add_css_class("marco-btn-blue");
    add_button.add_css_class("marco-tabs-action-btn");

    let duplicate_button = Button::with_label("Duplicate");
    duplicate_button.add_css_class("marco-btn");
    duplicate_button.add_css_class("marco-btn-blue");
    duplicate_button.add_css_class("marco-tabs-action-btn");

    let slides_actions = Box::new(Orientation::Horizontal, 8);
    slides_actions.append(&add_button);
    slides_actions.append(&duplicate_button);
    slides_inner.append(&slides_actions);

    slides_frame.set_child(Some(&slides_inner));
    content_box.append(&slides_frame);

    // ── Content frame ──────────────────────────────────────────────────────────
    let content_frame = Frame::new(None);
    content_frame.add_css_class("marco-tabs-frame");

    let content_inner = Box::new(Orientation::Vertical, 6);
    content_inner.set_margin_start(8);
    content_inner.set_margin_end(8);
    content_inner.set_margin_top(8);
    content_inner.set_margin_bottom(8);

    let content_title = Label::new(Some("Content for: -"));
    content_title.set_halign(Align::Start);
    content_title.set_xalign(0.0);
    content_title.add_css_class("marco-tabs-content-title");

    let content_scroller = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Always)
        .min_content_height(120)
        .hexpand(true)
        .vexpand(true)
        .build();
    content_scroller.add_css_class("marco-textfield-scroll");

    let content_view = TextView::new();
    content_view.set_wrap_mode(gtk4::WrapMode::WordChar);
    content_view.set_hexpand(true);
    content_view.set_vexpand(true);
    content_view.add_css_class("marco-textfield-view");
    content_scroller.set_child(Some(&content_view));

    content_inner.append(&content_title);
    content_inner.append(&content_scroller);
    content_frame.set_child(Some(&content_inner));
    content_box.append(&content_frame);

    // ── Bottom action bar ──────────────────────────────────────────────────────
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

    root.append(&content_box);
    root.append(&bottom_frame);
    dialog.set_child(Some(&root));

    // ── Syncing guard ──────────────────────────────────────────────────────────
    let is_syncing_content = Rc::new(Cell::new(false));

    // ── refresh_ui_state ──────────────────────────────────────────────────────
    // Keeps button sensitivity, empty/list stack page, and content frame in
    // sync with the current model state.
    let refresh_ui_state: Rc<dyn Fn()> = {
        let store = store.clone();
        let selection = selection.clone();
        let add_button = add_button.clone();
        let duplicate_button = duplicate_button.clone();
        let insert_button = insert_button.clone();
        let slides_stack = slides_stack.clone();
        let content_title = content_title.clone();
        let content_buffer = content_view.buffer();
        let is_syncing_content = is_syncing_content.clone();

        Rc::new(move || {
            let count = store.n_items();
            let selected = selected_position(&selection);

            add_button.set_sensitive(count < MAX_SLIDES);
            duplicate_button.set_sensitive(count < MAX_SLIDES && selected.is_some());
            // Disable insert when there are no slides to avoid inserting an empty deck.
            insert_button.set_sensitive(count > 0);

            // Switch between empty-state placeholder and the real list.
            if count == 0 {
                slides_stack.set_visible_child_name("empty");
            } else {
                slides_stack.set_visible_child_name("list");
            }

            if let Some(pos) = selected {
                content_title.set_text(&format!("Content for: {}", slide_name(pos)));

                if let Some(item_obj) = slide_object_at(&store, pos) {
                    let item = item_obj.borrow::<SlideItem>();
                    is_syncing_content.set(true);
                    content_buffer.set_text(&item.content);
                    is_syncing_content.set(false);
                }
            } else {
                content_title.set_text("Content for:");
                is_syncing_content.set(true);
                content_buffer.set_text("");
                is_syncing_content.set(false);
            }
        })
    };

    // ── Row factory ───────────────────────────────────────────────────────────
    slides_factory.connect_setup({
        let store = store.clone();
        let selection = selection.clone();
        let refresh_ui_state = refresh_ui_state.clone();
        move |_factory, list_item| {
            let row = Box::new(Orientation::Horizontal, 6);
            row.add_css_class("marco-tabs-row");

            // Slide label (non-editable; name is positional).
            let name_label = Label::new(None);
            name_label.set_hexpand(true);
            name_label.set_halign(Align::Start);
            name_label.set_valign(Align::Center);
            name_label.add_css_class("marco-tabs-name-entry"); // reuse same style

            let delete_button = Button::with_label("🗑");
            delete_button.add_css_class("marco-tabs-delete-btn");

            row.append(&name_label);
            row.append(&delete_button);
            list_item.set_child(Some(&row));

            // Clicking anywhere on the row selects it.
            let list_item_weak = list_item.downgrade();
            let selection_for_click = selection.clone();
            let click = gtk4::GestureClick::new();
            click.set_propagation_phase(gtk4::PropagationPhase::Bubble);
            click.connect_pressed(move |_gesture, _n_press, _x, _y| {
                let Some(list_item) = list_item_weak.upgrade() else {
                    return;
                };
                let position = list_item.position();
                if position == gtk4::INVALID_LIST_POSITION {
                    return;
                }
                selection_for_click.set_selected(position);
            });
            row.add_controller(click);

            // Delete button handler.
            let list_item_weak = list_item.downgrade();
            let store_for_delete = store.clone();
            let selection_for_delete = selection.clone();
            let refresh_for_delete = refresh_ui_state.clone();
            delete_button.connect_clicked(move |_| {
                let Some(list_item) = list_item_weak.upgrade() else {
                    return;
                };
                let position = list_item.position();
                if position == gtk4::INVALID_LIST_POSITION {
                    return;
                }
                if position >= store_for_delete.n_items() {
                    return;
                }

                store_for_delete.remove(position);

                let count = store_for_delete.n_items();
                if count == 0 {
                    selection_for_delete.set_selected(gtk4::INVALID_LIST_POSITION);
                } else if position >= count {
                    selection_for_delete.set_selected(count - 1);
                } else {
                    selection_for_delete.set_selected(position);
                }

                refresh_for_delete();
            });
        }
    });

    // Bind: set the label text based on the item's position (positional "Slide N").
    slides_factory.connect_bind(move |_factory, list_item| {
        let Some(row_widget) = list_item.child() else {
            return;
        };
        let Ok(row) = row_widget.downcast::<Box>() else {
            return;
        };
        let Some(first) = row.first_child() else {
            return;
        };
        let Ok(name_label) = first.downcast::<Label>() else {
            return;
        };

        let position = list_item.position();
        name_label.set_text(&slide_name(position));
    });

    // ── Selection changed ─────────────────────────────────────────────────────
    {
        let refresh_ui_state = refresh_ui_state.clone();
        selection.connect_selected_notify(move |_| {
            refresh_ui_state();
        });
    }

    // ── Add slide ─────────────────────────────────────────────────────────────
    {
        let store = store.clone();
        let selection = selection.clone();
        let refresh_ui_state = refresh_ui_state.clone();
        add_button.connect_clicked(move |_| {
            if store.n_items() >= MAX_SLIDES {
                refresh_ui_state();
                return;
            }

            let index = store.n_items();
            store.append(&glib::BoxedAnyObject::new(SlideItem {
                content: String::new(),
            }));
            selection.set_selected(index);
            refresh_ui_state();
        });
    }

    // ── Duplicate slide ───────────────────────────────────────────────────────
    {
        let store = store.clone();
        let selection = selection.clone();
        let refresh_ui_state = refresh_ui_state.clone();
        duplicate_button.connect_clicked(move |_| {
            if store.n_items() >= MAX_SLIDES {
                refresh_ui_state();
                return;
            }

            let Some(position) = selected_position(&selection) else {
                refresh_ui_state();
                return;
            };
            let Some(item_obj) = slide_object_at(&store, position) else {
                refresh_ui_state();
                return;
            };

            let content = {
                let item = item_obj.borrow::<SlideItem>();
                item.content.clone()
            };

            let insert_pos = (position + 1).min(store.n_items());
            store.insert(
                insert_pos,
                &glib::BoxedAnyObject::new(SlideItem { content }),
            );
            selection.set_selected(insert_pos);
            refresh_ui_state();
        });
    }

    // ── Content text changes ──────────────────────────────────────────────────
    {
        let store = store.clone();
        let selection = selection.clone();
        let is_syncing_content = is_syncing_content.clone();
        let content_buffer = content_view.buffer();
        content_buffer.connect_changed(move |buf| {
            if is_syncing_content.get() {
                return;
            }
            let Some(position) = selected_position(&selection) else {
                return;
            };
            let Some(item_obj) = slide_object_at(&store, position) else {
                return;
            };

            let start = buf.start_iter();
            let end = buf.end_iter();
            let text = buf.text(&start, &end, false).to_string();

            let mut item = item_obj.borrow_mut::<SlideItem>();
            item.content = text;
        });
    }

    // Initial state refresh.
    refresh_ui_state();

    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    // ── Insert button ─────────────────────────────────────────────────────────
    {
        let dialog_weak = dialog_weak.clone();
        let store = store.clone();
        let timer_seconds = timer_seconds.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        insert_button.connect_clicked(move |_| {
            let slides = collect_slides(&store);
            let timer = timer_seconds.get();
            let markdown = generate_sliders_markdown(&slides, timer);
            insert_sliders_at_cursor(&editor_buffer, &editor_view, &markdown);

            if let Some(dialog) = dialog_weak.upgrade() {
                dialog.close();
            }
        });
    }

    // ── Cancel / close ────────────────────────────────────────────────────────
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

    // Start focus on the Add Slide button: the dialog opens with an empty store,
    // so the natural first action is adding slides rather than editing content.
    add_button.grab_focus();
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store(count: u32) -> gio::ListStore {
        let store = gio::ListStore::new::<glib::BoxedAnyObject>();
        for _ in 0..count {
            store.append(&glib::BoxedAnyObject::new(SlideItem {
                content: String::new(),
            }));
        }
        store
    }

    // ── slide_name ───────────────────────────────────────────────────────────

    #[test]
    fn smoke_slide_name_first() {
        assert_eq!(slide_name(0), "Slide 1");
    }

    #[test]
    fn smoke_slide_name_third() {
        assert_eq!(slide_name(2), "Slide 3");
    }

    // ── collect_slides ───────────────────────────────────────────────────────

    #[test]
    fn smoke_collect_slides_empty() {
        let store = make_store(0);
        assert!(collect_slides(&store).is_empty());
    }

    #[test]
    fn smoke_collect_slides_count() {
        let store = make_store(3);
        assert_eq!(collect_slides(&store).len(), 3);
    }

    // ── generate_sliders_markdown ────────────────────────────────────────────

    #[test]
    fn smoke_generate_no_timer() {
        let slides = vec![
            SlideItem {
                content: "Hello".to_string(),
            },
            SlideItem {
                content: "World".to_string(),
            },
        ];
        let md = generate_sliders_markdown(&slides, 0);
        assert!(md.starts_with("@slidestart\n"));
        assert!(md.contains("\n---\n"));
        assert!(md.ends_with("@slideend"));
        assert!(!md.contains(":t"));
    }

    #[test]
    fn smoke_generate_with_timer() {
        let slides = vec![SlideItem {
            content: "Intro".to_string(),
        }];
        let md = generate_sliders_markdown(&slides, 5);
        assert!(md.starts_with("@slidestart:t5\n"));
        assert!(md.ends_with("@slideend"));
    }

    #[test]
    fn smoke_generate_empty_slides_fallback() {
        // No slides: should still produce a valid, parsable deck.
        let md = generate_sliders_markdown(&[], 0);
        assert!(md.starts_with("@slidestart"));
        assert!(md.ends_with("@slideend"));
    }

    #[test]
    fn smoke_generate_separator_count() {
        // 3 slides → 2 separators.
        let slides = vec![
            SlideItem {
                content: "A".to_string(),
            },
            SlideItem {
                content: "B".to_string(),
            },
            SlideItem {
                content: "C".to_string(),
            },
        ];
        let md = generate_sliders_markdown(&slides, 0);
        let sep_count = md.matches("\n---\n").count();
        assert_eq!(sep_count, 2);
    }
}
