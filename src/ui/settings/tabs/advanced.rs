use gtk4::Box;

use crate::ui::settings::tabs::window::flavor_info::FLAVOR_INFO_HTML;

pub fn build_advanced_tab() -> Box {
use gtk4::{Button, Dialog, ResponseType, ScrolledWindow};
use webkit6::prelude::*;
    use gtk4::{Label, Box as GtkBox, Orientation, Align};

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_margin_top(24);
    container.set_margin_bottom(24);
    container.set_margin_start(32);
    container.set_margin_end(32);

    // Helper for bold label
    let bold_label = |text: &str| {
        let l = Label::new(Some(text));
        l.set_halign(Align::Start);
        l.set_xalign(0.0);
        l.set_markup(&format!("<b>{}</b>", glib::markup_escape_text(text)));
        l
    };

    // Helper for normal description
    let desc_label = |text: &str| {
        let l = Label::new(Some(text));
        l.set_halign(Align::Start);
        l.set_xalign(0.0);
        l.set_wrap(true);
        l
    };


    // Markdown Variant Toggle Compatibility List
    let variants_vbox = GtkBox::new(Orientation::Vertical, 8);
    let variants_title = bold_label("Markdown Variant Toggle Compatibility List");
    variants_vbox.append(&variants_title);


    use gtk4::CheckButton;
    // List of variants and their compatibilities
    let variants = [
        ("CommonMark", &["GFM", "Marco"] as &[&str]),
        ("GFM", &["CommonMark", "Marco"]),
        ("Pandoc", &["Obsidian", "Typora", "Marco"]),
        ("Obsidian", &["Pandoc", "Typora", "Marco"]),
        ("Typora", &["Pandoc", "Obsidian", "Marco"]),
        ("Marco", &["All variants (Marco is compatible with everything)"])
    ];

    use std::collections::HashMap;
    use std::rc::Rc;
    use std::cell::RefCell;

    // Store checkbuttons and their compatibilities for dynamic access
    let checkboxes: Rc<RefCell<HashMap<String, CheckButton>>> = Rc::new(RefCell::new(HashMap::new()));
    let compat_map: HashMap<String, Vec<String>> = variants.iter()
        .map(|(v, c)| (v.to_string(), c.iter().map(|s| s.to_string()).collect::<Vec<_>>()))
        .collect();

    // Build UI and store checkbuttons, and also collect variant names for later
    let mut variant_names = Vec::new();
    for (variant, _) in variants.iter() {
        let hbox = GtkBox::new(Orientation::Horizontal, 0);
        let check = CheckButton::with_label(variant);
        check.set_halign(Align::Start);
        hbox.append(&check);
        hbox.set_margin_bottom(8);
        variants_vbox.append(&hbox);
        checkboxes.borrow_mut().insert(variant.to_string(), check.clone());
        variant_names.push(variant.to_string());
    }
    container.append(&variants_vbox);

    // Info button and dialog
    let info_button = Button::with_label("Info");
    info_button.set_halign(Align::End);
    info_button.set_margin_bottom(12);
    // Handler for info button
    info_button.connect_clicked(move |_| {
        // Find the root window (parent)
        let parent_window = gtk4::Window::list_toplevels()
            .into_iter()
            .find_map(|w| w.downcast::<gtk4::Window>().ok().filter(|win| win.is_visible()));
        let dialog = if let Some(parent) = parent_window {
            Dialog::builder()
                .transient_for(&parent)
                .modal(true)
                .title("Markdown Variant Selection Info")
                .build()
        } else {
            Dialog::builder()
                .modal(true)
                .title("Markdown Variant Selection Info")
                .build()
        };
        dialog.add_button("Close", ResponseType::Close);
        let content_area = dialog.content_area();
        use webkit6::WebView;
        let webview = WebView::new();
        webview.load_html(FLAVOR_INFO_HTML, None);
        webview.set_hexpand(true);
        webview.set_vexpand(true);
        let scrolled = ScrolledWindow::builder()
            .min_content_width(850)
            .min_content_height(600)
            .child(&webview)
            .build();
        content_area.append(&scrolled);
        dialog.connect_response(|d, _| d.close());
        dialog.show();
    });
    container.append(&info_button);

    // After all checkboxes are created, connect signal handlers in a second pass
    // Store previous selection before Marco is enabled
    use std::cell::RefCell as StdRefCell;
    let previous_selection: Rc<StdRefCell<Option<Vec<String>>>> = Rc::new(StdRefCell::new(None));
    // Shared tracker for Marco's last state
    thread_local! {
        static LAST_MARCO_ACTIVE: std::cell::RefCell<bool> = std::cell::RefCell::new(false);
    }

    for variant in variant_names {
        let checkboxes_rc = checkboxes.clone();
        let compat_map = compat_map.clone();
        let variant_name = variant.clone();
        let previous_selection = previous_selection.clone();
        // Clone the CheckButton for this variant
        let check_opt = checkboxes_rc.borrow().get(&variant_name).cloned();
        if let Some(check_orig) = check_opt {
            check_orig.connect_toggled({
                let check = check_orig.clone();
                let checkboxes_rc = checkboxes_rc.clone();
                let compat_map = compat_map.clone();
                let variant_name = variant_name.clone();
                let previous_selection = previous_selection.clone();
                move |_| {
                    let checkboxes = checkboxes_rc.clone();
                    let is_active = check.is_active();
                    // Count how many are currently active
                    let active_count = checkboxes.borrow().values().filter(|b| b.is_active()).count();
                    // If this is the only active checkbox and user tries to turn it off, prevent it
                    if !is_active && active_count == 0 {
                        check.set_active(true);
                        return;
                    }
                    // If Marco is being toggled ON, save the current selection (excluding Marco)
                    if variant_name == "Marco" && is_active {
                        // Only save previous selection if Marco was previously off
                        let mut should_save = false;
                        LAST_MARCO_ACTIVE.with(|last| {
                            let was = *last.borrow();
                            if !was {
                                should_save = true;
                            }
                            *last.borrow_mut() = true;
                        });
                        if should_save {
                            let selected: Vec<String> = checkboxes.borrow().iter()
                                .filter(|(v, btn)| btn.is_active() && *v != "Marco")
                                .map(|(v, _)| v.clone())
                                .collect();
                            *previous_selection.borrow_mut() = Some(selected);
                        }
                    } else if variant_name == "Marco" && !is_active {
                        // Update the static tracker
                        LAST_MARCO_ACTIVE.with(|last| {
                            *last.borrow_mut() = false;
                        });
                    }
                    // If Marco is being toggled OFF, restore previous selection if available
                    let marco_btn = checkboxes.borrow().get("Marco").cloned();
                    let marco_active = marco_btn.as_ref().map(|b| b.is_active()).unwrap_or(false);
                    if variant_name == "Marco" && !marco_active {
                        if let Some(prev) = previous_selection.borrow().as_ref() {
                            // First, uncheck everything except Marco
                            for (v, btn) in checkboxes.borrow().iter() {
                                if v == "Marco" {
                                    btn.set_sensitive(true);
                                    btn.set_active(false);
                                } else {
                                    btn.set_sensitive(true);
                                    btn.set_active(false);
                                }
                            }
                            // Then, restore previous selection (even if empty)
                            for v in prev.iter() {
                                if let Some(btn) = checkboxes.borrow().get(v) {
                                    btn.set_active(true);
                                }
                            }
                            // If after restoring, none are active, fallback to CommonMark+GFM
                            let still_active = checkboxes.borrow().values().any(|b| b.is_active());
                            if !still_active {
                                for (v, btn) in checkboxes.borrow().iter() {
                                    if v == "CommonMark" || v == "GFM" {
                                        btn.set_active(true);
                                    }
                                }
                            }
                            // After restoring, re-apply compatibility logic
                            // Use the first variant from prev as the active variant, if any
                            let mut any_on = false;
                            let mut active_variant = None;
                            if let Some(first) = prev.iter().find(|v| {
                                checkboxes.borrow().get(*v).map(|b| b.is_active()).unwrap_or(false) && *v != "Marco"
                            }) {
                                any_on = true;
                                active_variant = Some(first.clone());
                            } else {
                                // fallback: find any active variant (shouldn't happen)
                                for (v, btn) in checkboxes.borrow().iter() {
                                    if btn.is_active() && v != "Marco" {
                                        any_on = true;
                                        active_variant = Some(v.clone());
                                        break;
                                    }
                                }
                            }
                            if any_on {
                                let active = active_variant.unwrap();
                                let mut allowed = compat_map.get(&active).cloned().unwrap_or_default();
                                allowed.push(active.clone());
                                allowed.push("Marco".to_string());
                                for (v, btn) in checkboxes.borrow().iter() {
                                    if allowed.contains(v) {
                                        btn.set_sensitive(true);
                                    } else {
                                        btn.set_sensitive(false);
                                        btn.set_active(false);
                                    }
                                }
                            } else {
                                for btn in checkboxes.borrow().values() {
                                    btn.set_sensitive(true);
                                }
                            }
                        } else {
                            // No previous selection, fallback to CommonMark+GFM
                            for (v, btn) in checkboxes.borrow().iter() {
                                if v == "CommonMark" || v == "GFM" {
                                    btn.set_sensitive(true);
                                    btn.set_active(true);
                                } else if v == "Marco" {
                                    btn.set_sensitive(true);
                                    btn.set_active(false);
                                } else {
                                    btn.set_sensitive(false);
                                    btn.set_active(false);
                                }
                            }
                        }
                        return;
                    }
                    // If Marco is active, all are enabled
                    if marco_active {
                        for btn in checkboxes.borrow().values() {
                            btn.set_sensitive(true);
                        }
                        return;
                    }
                    // If any other is toggled on, only compatible + Marco remain enabled
                    let mut any_on = false;
                    let mut active_variant = None;
                    for (v, btn) in checkboxes.borrow().iter() {
                        if btn.is_active() && v != "Marco" {
                            any_on = true;
                            active_variant = Some(v.clone());
                            break;
                        }
                    }
                    if any_on {
                        let active = active_variant.unwrap();
                        let mut allowed = compat_map.get(&active).cloned().unwrap_or_default();
                        allowed.push(active.clone()); // allow self
                        allowed.push("Marco".to_string()); // Marco always allowed
                        for (v, btn) in checkboxes.borrow().iter() {
                            if allowed.contains(v) {
                                btn.set_sensitive(true);
                            } else {
                                btn.set_sensitive(false);
                                btn.set_active(false);
                            }
                        }
                    } else {
                        // If none are on, all are enabled
                        for btn in checkboxes.borrow().values() {
                            btn.set_sensitive(true);
                        }
                    }
                }
            });
        }
    }

    // Your own plugins/extensions (info only)
    let plugins_vbox = GtkBox::new(Orientation::Vertical, 2);
    let plugins_title = bold_label("Your own plugins/extensions");
    let plugins_desc = desc_label("Add, remove, or configure custom Markdown plugins and extensions here. (Coming soon)");
    plugins_vbox.append(&plugins_title);
    plugins_vbox.append(&plugins_desc);
    plugins_vbox.set_spacing(4);
    plugins_vbox.set_margin_bottom(24);
    container.append(&plugins_vbox);

    container
}
