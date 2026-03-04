//! Insert Tabs Dialog
//!
//! Provides a tab-block builder dialog using GTK4 ListView.

use gtk4::{
    gio, glib, prelude::*, Align, Box, Button, Entry, Frame, Label, ListView, Orientation,
    PolicyType, ScrolledWindow, SignalListItemFactory, SingleSelection, Stack, TextView, Window,
};
use sourceview5::{Buffer, View};
use std::{cell::Cell, rc::Rc};

const MAX_TABS: u32 = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TabItem {
    name: String,
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

fn tab_object_at(store: &gio::ListStore, position: u32) -> Option<glib::BoxedAnyObject> {
    store
        .item(position)
        .and_then(|obj| obj.downcast::<glib::BoxedAnyObject>().ok())
}

fn default_tab_name(position: u32) -> String {
    format!("Tab {}", position + 1)
}

fn next_tab_name(store: &gio::ListStore) -> String {
    let mut max_seen = 0u32;

    for idx in 0..store.n_items() {
        let Some(item_obj) = tab_object_at(store, idx) else {
            continue;
        };

        let item = item_obj.borrow::<TabItem>();
        let trimmed = item.name.trim();

        if let Some(rest) = trimmed.strip_prefix("Tab ") {
            if let Ok(n) = rest.parse::<u32>() {
                max_seen = max_seen.max(n);
            }
        }
    }

    format!("Tab {}", max_seen + 1)
}

fn collect_tabs(store: &gio::ListStore) -> Vec<TabItem> {
    let mut tabs = Vec::with_capacity(store.n_items() as usize);
    for idx in 0..store.n_items() {
        if let Some(item_obj) = tab_object_at(store, idx) {
            let item = item_obj.borrow::<TabItem>();
            tabs.push(item.clone());
        }
    }
    tabs
}

fn generate_tabs_markdown(tabs: &[TabItem]) -> String {
    let source_tabs = if tabs.is_empty() {
        vec![TabItem {
            name: "Tab 1".to_string(),
            content: String::new(),
        }]
    } else {
        tabs.to_vec()
    };

    let mut lines = vec![":::tab".to_string()];

    for (idx, tab) in source_tabs.iter().enumerate() {
        let name = if tab.name.trim().is_empty() {
            default_tab_name(idx as u32)
        } else {
            tab.name.trim().to_string()
        };

        lines.push(format!("@tab {}", name));

        if tab.content.trim().is_empty() {
            lines.push(String::new());
        } else {
            lines.extend(tab.content.lines().map(|line| line.to_string()));
        }

        if idx + 1 < source_tabs.len() {
            lines.push(String::new());
        }
    }

    lines.push(":::".to_string());
    lines.join("\n")
}

fn insert_tabs_at_cursor(buffer: &Buffer, view: &View, tabs_markdown: &str) {
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
        tabs_markdown.to_string()
    } else {
        tabs_markdown
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

    // Marks are invalidated after a buffer mutation; re-fetch to find the new end position.
    let end_mark = buffer.get_insert();
    let mut end_iter = buffer.iter_at_mark(&end_mark);
    view.scroll_to_iter(&mut end_iter, 0.15, true, 0.0, 0.35);
    view.grab_focus();
}

pub fn show_insert_tabs_dialog(parent: &Window, editor_buffer: &Buffer, editor_view: &View) {
    let theme_class = if parent.has_css_class("marco-theme-dark") {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    let dialog = Window::builder()
        // Non-modal: user can scroll/read the document while building the tab block.
        .modal(false)
        .transient_for(parent)
        .default_width(520)
        .default_height(420)
        .resizable(false)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class("marco-dialog-compact");
    dialog.add_css_class(theme_class);

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Insert Tabs",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let close_button = titlebar_controls
        .close_button
        .as_ref()
        .expect("Insert Tabs dialog requires a close button");
    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    let store = gio::ListStore::new::<glib::BoxedAnyObject>();

    let selection = SingleSelection::new(Some(store.clone()));
    selection.set_autoselect(true);
    selection.set_can_unselect(false);
    selection.set_selected(0);

    let root = Box::new(Orientation::Vertical, 0);

    let content_box = Box::new(Orientation::Vertical, 8);
    content_box.add_css_class("marco-dialog-content");
    content_box.set_margin_start(12);
    content_box.set_margin_end(12);
    content_box.set_margin_top(8);
    content_box.set_margin_bottom(0);

    let tabs_frame = Frame::new(Some("Tabs"));
    tabs_frame.add_css_class("marco-tabs-frame");

    let tabs_inner = Box::new(Orientation::Vertical, 8);
    tabs_inner.set_margin_start(8);
    tabs_inner.set_margin_end(8);
    tabs_inner.set_margin_top(8);
    tabs_inner.set_margin_bottom(8);

    let tabs_factory = SignalListItemFactory::new();

    let list_view = ListView::new(Some(selection.clone()), Some(tabs_factory.clone()));
    list_view.add_css_class("marco-tabs-listview");
    list_view.set_hexpand(true);
    list_view.set_vexpand(true);

    let tabs_scroller = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .min_content_height(130)
        .hexpand(true)
        .vexpand(true)
        .build();
    tabs_scroller.set_child(Some(&list_view));

    // Empty-state overlay shown when the store has no items.
    let empty_label = Label::new(Some("No tabs yet — click + Add Tab"));
    empty_label.set_halign(Align::Center);
    empty_label.set_valign(Align::Center);
    empty_label.add_css_class("marco-tabs-empty-label");

    let tabs_stack = Stack::new();
    tabs_stack.set_hexpand(true);
    tabs_stack.set_vexpand(true);
    tabs_stack.add_named(&empty_label, Some("empty"));
    tabs_stack.add_named(&tabs_scroller, Some("list"));
    // Start on the empty page; refresh_ui_state will switch as needed.
    tabs_stack.set_visible_child_name("empty");

    tabs_inner.append(&tabs_stack);

    let add_button = Button::with_label("+ Add Tab");
    add_button.add_css_class("marco-btn");
    add_button.add_css_class("marco-btn-blue");
    add_button.add_css_class("marco-tabs-action-btn");

    let duplicate_button = Button::with_label("Duplicate");
    duplicate_button.add_css_class("marco-btn");
    duplicate_button.add_css_class("marco-btn-blue");
    duplicate_button.add_css_class("marco-tabs-action-btn");

    let tabs_actions = Box::new(Orientation::Horizontal, 8);
    tabs_actions.append(&add_button);
    tabs_actions.append(&duplicate_button);
    tabs_inner.append(&tabs_actions);

    tabs_frame.set_child(Some(&tabs_inner));
    content_box.append(&tabs_frame);

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
        .min_content_height(150)
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

    let is_syncing_content = Rc::new(Cell::new(false));

    let refresh_ui_state: Rc<dyn Fn()> = {
        let store = store.clone();
        let selection = selection.clone();
        let add_button = add_button.clone();
        let duplicate_button = duplicate_button.clone();
        let insert_button = insert_button.clone();
        let tabs_stack = tabs_stack.clone();
        let content_title = content_title.clone();
        let content_buffer = content_view.buffer();
        let is_syncing_content = is_syncing_content.clone();

        Rc::new(move || {
            let count = store.n_items();
            let selected = selected_position(&selection);

            add_button.set_sensitive(count < MAX_TABS);
            duplicate_button.set_sensitive(count < MAX_TABS && selected.is_some());
            // Disable insert when there are no tabs to avoid inserting a phantom Tab 1.
            insert_button.set_sensitive(count > 0);

            // Switch between empty-state placeholder and the real list.
            if count == 0 {
                tabs_stack.set_visible_child_name("empty");
            } else {
                tabs_stack.set_visible_child_name("list");
            }

            if let Some(pos) = selected {
                if let Some(item_obj) = tab_object_at(&store, pos) {
                    let item = item_obj.borrow::<TabItem>();
                    content_title.set_text(&format!("Content for: {}", item.name));

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

    tabs_factory.connect_setup({
        let store = store.clone();
        let selection = selection.clone();
        let refresh_ui_state = refresh_ui_state.clone();
        move |_factory, list_item| {
            let row = Box::new(Orientation::Horizontal, 6);
            row.add_css_class("marco-tabs-row");

            let name_entry = Entry::new();
            name_entry.set_hexpand(true);
            name_entry.add_css_class("marco-textfield-entry");
            name_entry.add_css_class("marco-tabs-name-entry");

            let delete_button = Button::with_label("🗑");
            delete_button.add_css_class("marco-tabs-delete-btn");

            row.append(&name_entry);
            row.append(&delete_button);
            list_item.set_child(Some(&row));

            let list_item_weak = list_item.downgrade();
            let store_for_name = store.clone();
            let selection_for_name = selection.clone();
            let refresh_for_name = refresh_ui_state.clone();
            name_entry.connect_changed(move |entry| {
                let Some(list_item) = list_item_weak.upgrade() else {
                    return;
                };
                let position = list_item.position();
                if position == gtk4::INVALID_LIST_POSITION {
                    return;
                }
                let Some(item_obj) = tab_object_at(&store_for_name, position) else {
                    return;
                };

                {
                    let mut item = item_obj.borrow_mut::<TabItem>();
                    item.name = entry.text().to_string();
                }

                if selected_position(&selection_for_name) == Some(position) {
                    refresh_for_name();
                }
            });

            // Select the corresponding row whenever the user interacts with the
            // title entry — whether by clicking with a mouse or by Tab-key focus.
            //
            // `connect_has_focus_notify` handles keyboard navigation (Tab key).
            //
            // `GestureClick` with `PropagationPhase::Capture` handles mouse clicks.
            // The capture phase fires *before* the Entry processes the event, so it
            // never competes with normal text-editing gestures. Without it,
            // clicking an entry that already held focus from a previous interaction
            // would not trigger a focus-notify (the property doesn't change) and
            // the row would stay unselected.
            let list_item_weak = list_item.downgrade();
            let selection_for_focus = selection.clone();
            name_entry.connect_has_focus_notify(move |entry| {
                if !entry.has_focus() {
                    return;
                }
                let Some(list_item) = list_item_weak.upgrade() else {
                    return;
                };
                let position = list_item.position();
                if position == gtk4::INVALID_LIST_POSITION {
                    return;
                }
                selection_for_focus.set_selected(position);
            });

            let list_item_weak = list_item.downgrade();
            let selection_for_click = selection.clone();
            let click = gtk4::GestureClick::new();
            click.set_propagation_phase(gtk4::PropagationPhase::Capture);
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
            name_entry.add_controller(click);

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

    tabs_factory.connect_bind(move |_factory, list_item| {
        let Some(row_widget) = list_item.child() else {
            return;
        };
        let Ok(row) = row_widget.downcast::<Box>() else {
            return;
        };
        let Some(first) = row.first_child() else {
            return;
        };
        let Ok(name_entry) = first.downcast::<Entry>() else {
            return;
        };

        let Some(item_obj) = list_item
            .item()
            .and_then(|obj| obj.downcast::<glib::BoxedAnyObject>().ok())
        else {
            return;
        };

        // IMPORTANT: `set_text()` emits `changed`, which may borrow this item mutably
        // from the row handler. Clone the value first so the immutable borrow is dropped
        // before emitting signals.
        let name = {
            let item = item_obj.borrow::<TabItem>();
            item.name.clone()
        };
        name_entry.set_text(&name);
    });

    {
        let refresh_ui_state = refresh_ui_state.clone();
        selection.connect_selected_notify(move |_| {
            refresh_ui_state();
        });
    }

    {
        let store = store.clone();
        let selection = selection.clone();
        let refresh_ui_state = refresh_ui_state.clone();
        add_button.connect_clicked(move |_| {
            if store.n_items() >= MAX_TABS {
                refresh_ui_state();
                return;
            }

            let index = store.n_items();
            let name = next_tab_name(&store);
            store.append(&glib::BoxedAnyObject::new(TabItem {
                name,
                content: String::new(),
            }));
            selection.set_selected(index);
            refresh_ui_state();
        });
    }

    {
        let store = store.clone();
        let selection = selection.clone();
        let refresh_ui_state = refresh_ui_state.clone();
        duplicate_button.connect_clicked(move |_| {
            if store.n_items() >= MAX_TABS {
                refresh_ui_state();
                return;
            }

            let Some(position) = selected_position(&selection) else {
                refresh_ui_state();
                return;
            };
            let Some(item_obj) = tab_object_at(&store, position) else {
                refresh_ui_state();
                return;
            };

            let (name, content) = {
                let item = item_obj.borrow::<TabItem>();
                (item.name.clone(), item.content.clone())
            };
            // Strip any existing " Copy" suffixes before appending, so repeated
            // duplications don't accumulate "Copy Copy Copy" chains.
            let base_name = name.trim_end_matches(" Copy").to_string();
            let duplicated = TabItem {
                name: format!("{} Copy", base_name),
                content,
            };

            let insert_pos = (position + 1).min(store.n_items());
            store.insert(insert_pos, &glib::BoxedAnyObject::new(duplicated));
            selection.set_selected(insert_pos);
            refresh_ui_state();
        });
    }

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
            let Some(item_obj) = tab_object_at(&store, position) else {
                return;
            };

            let start = buf.start_iter();
            let end = buf.end_iter();
            let text = buf.text(&start, &end, false).to_string();

            let mut item = item_obj.borrow_mut::<TabItem>();
            item.content = text;
        });
    }

    refresh_ui_state();

    let editor_buffer = editor_buffer.clone();
    let editor_view = editor_view.clone();
    let dialog_weak = dialog.downgrade();

    {
        let dialog_weak = dialog_weak.clone();
        let store = store.clone();
        let editor_buffer = editor_buffer.clone();
        let editor_view = editor_view.clone();
        insert_button.connect_clicked(move |_| {
            let tabs = collect_tabs(&store);
            let markdown = generate_tabs_markdown(&tabs);
            insert_tabs_at_cursor(&editor_buffer, &editor_view, &markdown);

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

    // Start focus on the Add Tab button: the dialog opens with an empty store,
    // so the natural first action is adding tabs rather than editing content.
    add_button.grab_focus();
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── next_tab_name ────────────────────────────────────────────────────────

    fn make_store(names: &[&str]) -> gio::ListStore {
        let store = gio::ListStore::new::<glib::BoxedAnyObject>();
        for name in names {
            store.append(&glib::BoxedAnyObject::new(TabItem {
                name: name.to_string(),
                content: String::new(),
            }));
        }
        store
    }

    #[test]
    fn smoke_next_tab_name_empty_store() {
        let store = make_store(&[]);
        assert_eq!(next_tab_name(&store), "Tab 1");
    }

    #[test]
    fn smoke_next_tab_name_single_item() {
        let store = make_store(&["Tab 1"]);
        assert_eq!(next_tab_name(&store), "Tab 2");
    }

    #[test]
    fn smoke_next_tab_name_gap_in_sequence() {
        // Has Tab 1 and Tab 3 but not Tab 2 — next should be Tab 4 (max+1).
        let store = make_store(&["Tab 1", "Tab 3"]);
        assert_eq!(next_tab_name(&store), "Tab 4");
    }

    #[test]
    fn smoke_next_tab_name_all_renamed() {
        // No "Tab N" names exist — first auto-name should be Tab 1.
        let store = make_store(&["Alpha", "Beta", "Gamma"]);
        assert_eq!(next_tab_name(&store), "Tab 1");
    }

    #[test]
    fn smoke_next_tab_name_ignores_non_numeric_suffix() {
        // "Tab foo" is not a valid "Tab N" entry and must be ignored.
        let store = make_store(&["Tab foo", "Tab 2"]);
        assert_eq!(next_tab_name(&store), "Tab 3");
    }

    // ── collect_tabs ─────────────────────────────────────────────────────────

    #[test]
    fn smoke_collect_tabs_order_preserved() {
        let store = make_store(&["First", "Second", "Third"]);
        let tabs = collect_tabs(&store);
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0].name, "First");
        assert_eq!(tabs[1].name, "Second");
        assert_eq!(tabs[2].name, "Third");
    }

    #[test]
    fn smoke_collect_tabs_empty_store() {
        let store = make_store(&[]);
        let tabs = collect_tabs(&store);
        assert!(tabs.is_empty());
    }

    // ── duplicate copy logic ─────────────────────────────────────────────────

    #[test]
    fn smoke_duplicate_copy_strips_existing_suffix() {
        // Simulate what the duplicate handler does with trim_end_matches.
        let name = "Tab 1 Copy Copy".to_string();
        let base_name = name.trim_end_matches(" Copy").to_string();
        let result = format!("{} Copy", base_name);
        assert_eq!(result, "Tab 1 Copy");
    }

    #[test]
    fn smoke_duplicate_copy_first_duplication() {
        let name = "Tab 1".to_string();
        let base_name = name.trim_end_matches(" Copy").to_string();
        let result = format!("{} Copy", base_name);
        assert_eq!(result, "Tab 1 Copy");
    }

    // ── generate_tabs_markdown ───────────────────────────────────────────────

    #[test]
    fn smoke_test_generate_tabs_markdown() {
        let tabs = vec![
            TabItem {
                name: "Windows".to_string(),
                content: "Install with PowerShell".to_string(),
            },
            TabItem {
                name: "Linux".to_string(),
                content: "Use your package manager".to_string(),
            },
        ];

        let markdown = generate_tabs_markdown(&tabs);
        assert!(markdown.starts_with(":::tab"));
        assert!(markdown.contains("@tab Windows"));
        assert!(markdown.contains("@tab Linux"));
        assert!(markdown.ends_with(":::"));
    }

    #[test]
    fn smoke_test_generate_tabs_markdown_empty_name_fallback() {
        let tabs = vec![TabItem {
            name: "".to_string(),
            content: "content".to_string(),
        }];

        let markdown = generate_tabs_markdown(&tabs);
        assert!(markdown.contains("@tab Tab 1"));
    }
}
