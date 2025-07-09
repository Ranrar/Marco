use crate::editor::core::MarkdownEditor;
use gtk4::prelude::*;
use gtk4::{
    Dialog, Entry, FileChooserAction, FileChooserDialog, FileFilter, Grid, Label, ListBox,
    ListBoxRow, ResponseType, ScrolledWindow, SearchEntry,
};

/// URL-encode a file path or URL to handle spaces and special characters
fn url_encode_path(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        // For HTTP URLs, only encode spaces - other characters are likely already encoded
        path.replace(' ', "%20")
    } else {
        // For local file paths, encode spaces and other characters that can cause issues
        path.chars()
            .map(|c| match c {
                ' ' => "%20".to_string(),
                '(' => "%28".to_string(),
                ')' => "%29".to_string(),
                '[' => "%5B".to_string(),
                ']' => "%5D".to_string(),
                '{' => "%7B".to_string(),
                '}' => "%7D".to_string(),
                c if c.is_ascii_alphanumeric() || ".-_~/:".contains(c) => c.to_string(),
                c => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}

impl MarkdownEditor {
    /// Show dialog to select programming language for fenced code block with smart search
    pub(crate) fn show_fenced_code_dialog(&self) {
        // Get the window from the source view's root
        let window = if let Some(widget) = self.source_view.root() {
            if let Ok(window) = widget.downcast::<gtk4::Window>() {
                Some(window)
            } else {
                None
            }
        } else {
            None
        };

        // Create the dialog with proper parent window
        let dialog = Dialog::with_buttons(
            Some("Insert Fenced Code Block"),
            window.as_ref(),
            gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Insert", ResponseType::Accept),
            ],
        );

        // Create the main container
        let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);

        // Title label
        let title_label = Label::new(Some("Choose a programming language:"));
        title_label.set_halign(gtk4::Align::Start);
        title_label.add_css_class("heading");
        main_box.append(&title_label);

        // Search entry
        let search_entry = SearchEntry::new();
        search_entry.set_placeholder_text(Some(
            "Type to search languages (e.g., rust, javascript, python...)",
        ));
        search_entry.set_hexpand(true);
        main_box.append(&search_entry);

        // Scrolled window for the language list
        let scrolled_window = ScrolledWindow::new();
        scrolled_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        scrolled_window.set_min_content_height(200);
        scrolled_window.set_max_content_height(300);

        // List box for displaying filtered languages
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        list_box.add_css_class("rich-list");
        scrolled_window.set_child(Some(&list_box));
        main_box.append(&scrolled_window);

        // Get all available languages from syntect
        let code_manager = self.code_language_manager.borrow().clone();

        // Selected language storage
        let selected_language = std::rc::Rc::new(std::cell::RefCell::new(String::new()));

        // Function to populate the list
        let populate_list = {
            let list_box = list_box.clone();
            let selected_language = selected_language.clone();

            move |filter: &str| {
                // Clear existing items
                while let Some(child) = list_box.first_child() {
                    list_box.remove(&child);
                }

                // Add "(none)" option
                let none_row = ListBoxRow::new();
                let none_label = Label::new(Some("(none) - Plain text"));
                none_label.set_halign(gtk4::Align::Start);
                none_label.set_margin_top(8);
                none_label.set_margin_bottom(8);
                none_label.set_margin_start(12);
                none_label.set_margin_end(12);
                none_row.set_child(Some(&none_label));
                none_row.set_activatable(true);
                list_box.append(&none_row);

                // Use smart language suggestions from syntect_highlight
                let matching_languages = code_manager.get_smart_language_suggestions(filter);

                // Limit to 20 results for performance
                for lang in matching_languages.iter().take(20) {
                    let row = ListBoxRow::new();
                    let label = Label::new(Some(lang));
                    label.set_halign(gtk4::Align::Start);
                    label.set_margin_top(8);
                    label.set_margin_bottom(8);
                    label.set_margin_start(12);
                    label.set_margin_end(12);
                    row.set_child(Some(&label));
                    row.set_activatable(true);
                    list_box.append(&row);
                }

                // Auto-select first item if filter matches exactly
                if !filter.is_empty() && !matching_languages.is_empty() {
                    if let Some(first_lang) = matching_languages.get(0) {
                        if first_lang.to_lowercase() == filter.to_lowercase() {
                            if let Some(row) = list_box.row_at_index(1) {
                                // Skip "(none)" row
                                list_box.select_row(Some(&row));
                                *selected_language.borrow_mut() = first_lang.clone();
                            }
                        }
                    }
                }
            }
        };

        // Initial population
        populate_list("");

        // Search functionality
        let search_populate = populate_list.clone();
        search_entry.connect_search_changed(move |entry| {
            let text = entry.text();
            search_populate(&text);
        });

        // List box selection
        let selection_language = selected_language.clone();
        list_box.connect_row_selected(move |_list_box, row| {
            if let Some(row) = row {
                if let Some(child) = row.child() {
                    if let Ok(label) = child.downcast::<Label>() {
                        let text = label.text();
                        let lang = if text.starts_with("(none)") {
                            String::new()
                        } else {
                            text.to_string()
                        };
                        *selection_language.borrow_mut() = lang;
                    }
                }
            }
        });

        // Double-click to insert
        let dialog_close = dialog.clone();
        let buffer_clone = self.source_buffer.clone();
        list_box.connect_row_activated(move |_list_box, row| {
            if let Some(child) = row.child() {
                if let Ok(label) = child.downcast::<Label>() {
                    let text = label.text();
                    let language = if text.starts_with("(none)") {
                        String::new()
                    } else {
                        text.to_string()
                    };

                    let code_block = if language.is_empty() {
                        "\n```\ncode goes here\n```\n".to_string()
                    } else {
                        format!("\n```{}\ncode goes here\n```\n", language.to_lowercase())
                    };

                    let gtk_buffer = buffer_clone.upcast_ref::<gtk4::TextBuffer>();
                    let cursor_mark = gtk_buffer.get_insert();
                    let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
                    buffer_clone.insert(&mut cursor_iter, &code_block);

                    dialog_close.close();
                }
            }
        });

        // Add content to dialog
        dialog.content_area().append(&main_box);

        // Set default button
        if let Some(insert_button) = dialog.widget_for_response(ResponseType::Accept) {
            insert_button.add_css_class("suggested-action");
        }

        // Focus on search entry
        search_entry.grab_focus();

        // Connect response
        let final_buffer_clone = self.source_buffer.clone();
        let final_selected_language = selected_language.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let language = final_selected_language.borrow().clone();

                let code_block = if language.is_empty() {
                    "\n```\ncode goes here\n```\n".to_string()
                } else {
                    format!("\n```{}\ncode goes here\n```\n", language.to_lowercase())
                };

                let gtk_buffer = final_buffer_clone.upcast_ref::<gtk4::TextBuffer>();
                let cursor_mark = gtk_buffer.get_insert();
                let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
                final_buffer_clone.insert(&mut cursor_iter, &code_block);
            }
            dialog.close();
        });

        dialog.present();

        // Focus editor after dialog is shown
        let editor_clone = self.clone();
        dialog.connect_close(move |_| {
            editor_clone.focus_editor_and_position_cursor();
        });
    }

    pub(crate) fn show_link_dialog(&self) {
        // Get the window from the source view's root
        let window = if let Some(widget) = self.source_view.root() {
            if let Ok(window) = widget.downcast::<gtk4::Window>() {
                Some(window)
            } else {
                None
            }
        } else {
            None
        };

        // Create the dialog with proper parent window for modal behavior
        let dialog = Dialog::with_buttons(
            Some("Insert Link"),
            window.as_ref(),
            gtk4::DialogFlags::MODAL | gtk4::DialogFlags::DESTROY_WITH_PARENT,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Insert", ResponseType::Accept),
            ],
        );

        // Create the grid layout
        let grid = Grid::new();
        grid.set_row_spacing(10);
        grid.set_column_spacing(10);
        grid.set_margin_top(20);
        grid.set_margin_bottom(20);
        grid.set_margin_start(20);
        grid.set_margin_end(20);

        // URL input
        let url_label = Label::new(Some("URL:"));
        url_label.set_halign(gtk4::Align::Start);
        let url_entry = Entry::new();
        url_entry.set_placeholder_text(Some("https://example.com"));
        url_entry.set_hexpand(true);

        // Link text input
        let text_label = Label::new(Some("Link Text:"));
        text_label.set_halign(gtk4::Align::Start);
        let text_entry = Entry::new();
        text_entry.set_placeholder_text(Some("Link description"));
        text_entry.set_hexpand(true);

        // Check if we have selected text to use as default
        let buffer = &self.source_buffer;
        let gtk_buffer = buffer.upcast_ref::<gtk4::TextBuffer>();
        if gtk_buffer.has_selection() {
            if let Some((start, end)) = gtk_buffer.selection_bounds() {
                let selected_text = gtk_buffer.text(&start, &end, false);
                text_entry.set_text(&selected_text);
            }
        }

        // Add to grid
        grid.attach(&url_label, 0, 0, 1, 1);
        grid.attach(&url_entry, 1, 0, 1, 1);
        grid.attach(&text_label, 0, 1, 1, 1);
        grid.attach(&text_entry, 1, 1, 1, 1);

        // Add grid to dialog
        dialog.content_area().append(&grid);

        // Focus on URL entry
        url_entry.grab_focus();

        // Connect response
        let buffer_clone = self.source_buffer.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                let url = url_entry.text();
                let text = text_entry.text();

                if !url.is_empty() {
                    let link_text = if text.is_empty() {
                        url.to_string()
                    } else {
                        text.to_string()
                    };

                    let link = format!("[{}]({})", link_text, url);

                    // Insert at cursor or replace selection
                    let gtk_buffer = buffer_clone.upcast_ref::<gtk4::TextBuffer>();
                    if gtk_buffer.has_selection() {
                        // Replace selection
                        if let Some((start, end)) = gtk_buffer.selection_bounds() {
                            let mut start_mut = start;
                            let mut end_mut = end;
                            buffer_clone.delete(&mut start_mut, &mut end_mut);
                            let mut insert_iter = start;
                            buffer_clone.insert(&mut insert_iter, &link);
                        }
                    } else {
                        // Insert at cursor
                        let cursor_mark = gtk_buffer.get_insert();
                        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
                        buffer_clone.insert(&mut cursor_iter, &link);
                    }
                }
            }
            dialog.close();
        });

        dialog.present();

        // Focus editor after dialog is shown
        let editor_clone = self.clone();
        dialog.connect_close(move |_| {
            editor_clone.focus_editor_and_position_cursor();
        });
    }

    pub(crate) fn show_image_dialog(&self) {
        // Get the window from the source view's root
        let window = if let Some(widget) = self.source_view.root() {
            if let Ok(window) = widget.downcast::<gtk4::Window>() {
                Some(window)
            } else {
                None
            }
        } else {
            None
        };

        // Create file chooser dialog with proper parent window
        let dialog = FileChooserDialog::new(
            Some("Select Image"),
            window.as_ref(),
            FileChooserAction::Open,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Open", ResponseType::Accept),
            ],
        );

        // Add image file filters
        let filter = FileFilter::new();
        filter.set_name(Some("Image Files"));
        filter.add_mime_type("image/*");
        filter.add_pattern("*.png");
        filter.add_pattern("*.jpg");
        filter.add_pattern("*.jpeg");
        filter.add_pattern("*.gif");
        filter.add_pattern("*.bmp");
        filter.add_pattern("*.svg");
        filter.add_pattern("*.webp");
        dialog.add_filter(&filter);

        let all_filter = FileFilter::new();
        all_filter.set_name(Some("All Files"));
        all_filter.add_pattern("*");
        dialog.add_filter(&all_filter);

        // Connect response
        let buffer_clone = self.source_buffer.clone();
        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Accept {
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        let path_str = path.to_string_lossy();
                        let filename = path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("image");

                        // URL-encode the path to handle spaces and special characters
                        let encoded_path = url_encode_path(&path_str);

                        let image_markdown = format!("![{}]({})", filename, encoded_path);

                        // Insert at cursor
                        let gtk_buffer = buffer_clone.upcast_ref::<gtk4::TextBuffer>();
                        let cursor_mark = gtk_buffer.get_insert();
                        let mut cursor_iter = gtk_buffer.iter_at_mark(&cursor_mark);
                        buffer_clone.insert(&mut cursor_iter, &image_markdown);
                    }
                }
            }
            dialog.close();
        });

        dialog.present();

        // Focus editor after dialog is shown
        let editor_clone = self.clone();
        dialog.connect_close(move |_| {
            editor_clone.focus_editor_and_position_cursor();
        });
    }

    pub fn show_find_dialog(&self, window: &gtk4::Window) {
        let dialog = Dialog::builder()
            .title("Find")
            .transient_for(window)
            .modal(true)
            .build();

        let content_area = dialog.content_area();
        let grid = Grid::new();
        grid.set_margin_top(12);
        grid.set_margin_bottom(12);
        grid.set_margin_start(12);
        grid.set_margin_end(12);
        grid.set_row_spacing(6);
        grid.set_column_spacing(6);

        let find_label = Label::new(Some("Find:"));
        let find_entry = Entry::new();
        find_entry.set_hexpand(true);

        // Add case-sensitive checkbox
        let case_sensitive_check = gtk4::CheckButton::builder().label("Case sensitive").build();

        grid.attach(&find_label, 0, 0, 1, 1);
        grid.attach(&find_entry, 1, 0, 1, 1);
        grid.attach(&case_sensitive_check, 1, 1, 1, 1);

        content_area.append(&grid);

        dialog.add_button("Cancel", ResponseType::Cancel);
        let find_next_button = dialog.add_button("Find Next", ResponseType::Ok);
        find_next_button.set_css_classes(&["suggested-action"]);

        let source_buffer = self.source_buffer.clone();
        let source_view = self.source_view.clone();

        // Set focus to the entry
        find_entry.grab_focus();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok {
                let entry = dialog
                    .content_area()
                    .first_child()
                    .and_then(|grid| grid.downcast::<Grid>().ok())
                    .and_then(|grid| grid.child_at(1, 0))
                    .and_then(|entry| entry.downcast::<Entry>().ok());

                let case_check = dialog
                    .content_area()
                    .first_child()
                    .and_then(|grid| grid.downcast::<Grid>().ok())
                    .and_then(|grid| grid.child_at(1, 1))
                    .and_then(|check| check.downcast::<gtk4::CheckButton>().ok());

                if let (Some(entry), Some(case_check)) = (entry, case_check) {
                    let search_text = entry.text();
                    if !search_text.is_empty() {
                        // Perform search from cursor position
                        let cursor_mark = source_buffer.get_insert();
                        let cursor_iter = source_buffer.iter_at_mark(&cursor_mark);
                        let end_iter = source_buffer.end_iter();
                        let text = source_buffer.text(&cursor_iter, &end_iter, false);
                        let text_str = text.as_str();
                        let search_str = search_text.as_str();

                        let found_pos = if case_check.is_active() {
                            text_str.find(search_str)
                        } else {
                            text_str.to_lowercase().find(&search_str.to_lowercase())
                        };

                        if let Some(pos) = found_pos {
                            let mut search_start = cursor_iter;
                            search_start.forward_chars(pos as i32);
                            let mut search_end = search_start;
                            search_end.forward_chars(search_str.len() as i32);
                            source_buffer.select_range(&search_start, &search_end);

                            // Scroll to the found text
                            let mut scroll_iter = search_start;
                            source_view.scroll_to_iter(&mut scroll_iter, 0.0, false, 0.0, 0.0);
                        } else {
                            // Not found from cursor, search from beginning
                            let (start, _) = source_buffer.bounds();
                            let text = source_buffer.text(&start, &cursor_iter, false);
                            let text_str = text.as_str();

                            let found_pos = if case_check.is_active() {
                                text_str.find(search_str)
                            } else {
                                text_str.to_lowercase().find(&search_str.to_lowercase())
                            };

                            if let Some(pos) = found_pos {
                                let mut search_start = start;
                                search_start.forward_chars(pos as i32);
                                let mut search_end = search_start;
                                search_end.forward_chars(search_str.len() as i32);
                                source_buffer.select_range(&search_start, &search_end);
                                let mut scroll_iter = search_start;
                                source_view.scroll_to_iter(&mut scroll_iter, 0.0, false, 0.0, 0.0);
                            }
                        }
                    }
                }
            } else {
                dialog.close();
            }
        });

        dialog.present();
    }

    pub fn show_replace_dialog(&self, window: &gtk4::Window) {
        let dialog = Dialog::builder()
            .title("Find and Replace")
            .transient_for(window)
            .modal(true)
            .build();

        let content_area = dialog.content_area();
        let grid = Grid::new();
        grid.set_margin_top(12);
        grid.set_margin_bottom(12);
        grid.set_margin_start(12);
        grid.set_margin_end(12);
        grid.set_row_spacing(6);
        grid.set_column_spacing(6);

        let find_label = Label::new(Some("Find:"));
        let find_entry = Entry::new();
        find_entry.set_hexpand(true);

        let replace_label = Label::new(Some("Replace:"));
        let replace_entry = Entry::new();
        replace_entry.set_hexpand(true);

        // Add case-sensitive checkbox
        let case_sensitive_check = gtk4::CheckButton::builder().label("Case sensitive").build();

        grid.attach(&find_label, 0, 0, 1, 1);
        grid.attach(&find_entry, 1, 0, 1, 1);
        grid.attach(&replace_label, 0, 1, 1, 1);
        grid.attach(&replace_entry, 1, 1, 1, 1);
        grid.attach(&case_sensitive_check, 1, 2, 1, 1);

        content_area.append(&grid);

        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.add_button("Replace All", ResponseType::Apply);
        let replace_button = dialog.add_button("Replace", ResponseType::Ok);
        replace_button.set_css_classes(&["suggested-action"]);

        // Set focus to the find entry
        find_entry.grab_focus();

        let source_buffer = self.source_buffer.clone();
        let source_view = self.source_view.clone();

        dialog.connect_response(move |dialog, response| {
            if response == ResponseType::Ok || response == ResponseType::Apply {
                let content_area = dialog.content_area();
                let grid = content_area
                    .first_child()
                    .and_then(|grid| grid.downcast::<Grid>().ok());

                if let Some(grid) = grid {
                    let find_entry = grid
                        .child_at(1, 0)
                        .and_then(|entry| entry.downcast::<Entry>().ok());
                    let replace_entry = grid
                        .child_at(1, 1)
                        .and_then(|entry| entry.downcast::<Entry>().ok());
                    let case_check = grid
                        .child_at(1, 2)
                        .and_then(|check| check.downcast::<gtk4::CheckButton>().ok());

                    if let (Some(find_entry), Some(replace_entry), Some(case_check)) =
                        (find_entry, replace_entry, case_check)
                    {
                        let find_text = find_entry.text();
                        let replace_text = replace_entry.text();

                        if !find_text.is_empty() {
                            let (start, end) = source_buffer.bounds();
                            let text = source_buffer.text(&start, &end, false);
                            let text_str = text.as_str();
                            let find_str = find_text.as_str();
                            let replace_str = replace_text.as_str();

                            if response == ResponseType::Apply {
                                // Replace all
                                let new_text = if case_check.is_active() {
                                    text_str.replace(find_str, replace_str)
                                } else {
                                    // Case-insensitive replace all
                                    let mut result = String::new();
                                    let mut last_end = 0;
                                    let text_lower = text_str.to_lowercase();
                                    let find_lower = find_str.to_lowercase();

                                    for (start, _) in text_lower.match_indices(&find_lower) {
                                        result.push_str(&text_str[last_end..start]);
                                        result.push_str(replace_str);
                                        last_end = start + find_str.len();
                                    }
                                    result.push_str(&text_str[last_end..]);
                                    result
                                };
                                source_buffer.set_text(&new_text);
                            } else {
                                // Replace next occurrence from current cursor position
                                let cursor_mark = source_buffer.get_insert();
                                let cursor_iter = source_buffer.iter_at_mark(&cursor_mark);
                                let end_iter = source_buffer.end_iter();
                                let text_from_cursor =
                                    source_buffer.text(&cursor_iter, &end_iter, false);
                                let text_from_cursor_str = text_from_cursor.as_str();

                                let found_pos = if case_check.is_active() {
                                    text_from_cursor_str.find(find_str)
                                } else {
                                    text_from_cursor_str
                                        .to_lowercase()
                                        .find(&find_str.to_lowercase())
                                };

                                if let Some(pos) = found_pos {
                                    let mut search_start = cursor_iter;
                                    search_start.forward_chars(pos as i32);
                                    let mut search_end = search_start;
                                    search_end.forward_chars(find_str.len() as i32);

                                    // Create a mark to preserve the position before deletion
                                    let replace_mark =
                                        source_buffer.create_mark(None, &search_start, false);

                                    // Replace the found text
                                    let mut search_start_mut = search_start;
                                    let mut search_end_mut = search_end;
                                    source_buffer
                                        .delete(&mut search_start_mut, &mut search_end_mut);

                                    // Get fresh iterator from mark for insertion
                                    let mut insert_iter = source_buffer.iter_at_mark(&replace_mark);
                                    source_buffer.insert(&mut insert_iter, replace_str);

                                    // Position cursor after replacement and scroll to it
                                    let new_start = source_buffer.iter_at_mark(&replace_mark);
                                    let mut new_pos = new_start;
                                    new_pos.forward_chars(replace_str.len() as i32);
                                    source_buffer.place_cursor(&new_pos);
                                    let mut scroll_iter = new_pos;
                                    source_view.scroll_to_iter(
                                        &mut scroll_iter,
                                        0.0,
                                        false,
                                        0.0,
                                        0.0,
                                    );

                                    // Clean up the temporary mark
                                    source_buffer.delete_mark(&replace_mark);
                                } else {
                                    // Not found from cursor, search from beginning
                                    let (start, _) = source_buffer.bounds();
                                    let text_to_cursor =
                                        source_buffer.text(&start, &cursor_iter, false);
                                    let text_to_cursor_str = text_to_cursor.as_str();

                                    let found_pos = if case_check.is_active() {
                                        text_to_cursor_str.find(find_str)
                                    } else {
                                        text_to_cursor_str
                                            .to_lowercase()
                                            .find(&find_str.to_lowercase())
                                    };

                                    if let Some(pos) = found_pos {
                                        let mut search_start = start;
                                        search_start.forward_chars(pos as i32);
                                        let mut search_end = search_start;
                                        search_end.forward_chars(find_str.len() as i32);

                                        // Replace the found text
                                        let replace_mark =
                                            source_buffer.create_mark(None, &search_start, false);

                                        let mut search_start_mut = search_start;
                                        let mut search_end_mut = search_end;
                                        source_buffer
                                            .delete(&mut search_start_mut, &mut search_end_mut);

                                        // Get fresh iterator from mark for insertion
                                        let mut insert_iter =
                                            source_buffer.iter_at_mark(&replace_mark);
                                        source_buffer.insert(&mut insert_iter, replace_str);

                                        // Position cursor after replacement and scroll to it
                                        let new_start = source_buffer.iter_at_mark(&replace_mark);
                                        let mut new_pos = new_start;
                                        new_pos.forward_chars(replace_str.len() as i32);
                                        source_buffer.place_cursor(&new_pos);
                                        let mut scroll_iter = new_pos;
                                        source_view.scroll_to_iter(
                                            &mut scroll_iter,
                                            0.0,
                                            false,
                                            0.0,
                                            0.0,
                                        );

                                        // Clean up the temporary mark
                                        source_buffer.delete_mark(&replace_mark);
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                dialog.close();
            }
        });

        dialog.present();
    }

    /// Show save confirmation dialog and handle the response asynchronously
    /// Returns true immediately if no unsaved changes, false if dialog is shown (to prevent immediate quit)
    pub fn show_unsaved_changes_dialog_and_quit<F>(
        &self,
        parent: Option<&gtk4::Window>,
        on_confirm_quit: F,
    ) -> bool
    where
        F: Fn() + 'static,
    {
        if !self.is_modified() {
            println!("DEBUG: Document not modified, proceeding immediately");
            return true; // Not modified, safe to proceed immediately
        }

        // Check if document is effectively empty (even if it was modified) - if so, don't show save dialog
        if self.is_empty() {
            println!(
                "DEBUG: Document is empty after edits, proceeding immediately without save prompt"
            );
            return true; // Empty document, safe to proceed immediately
        }

        println!("DEBUG: Document is modified and not empty, showing unsaved changes dialog");

        // Create confirmation dialog
        let title = crate::language::tr("dialog.unsaved_changes.title");
        let message = crate::language::tr("dialog.unsaved_changes.message");
        let cancel_text = crate::language::tr("dialog.unsaved_changes.cancel");
        let discard_text = crate::language::tr("dialog.unsaved_changes.discard");
        let save_text = crate::language::tr("dialog.unsaved_changes.save");

        println!("DEBUG: Dialog strings - Title: '{}', Message: '{}', Cancel: '{}', Discard: '{}', Save: '{}'", 
                 title, message, cancel_text, discard_text, save_text);

        let dialog = gtk4::MessageDialog::builder()
            .transient_for(parent.unwrap_or(&gtk4::Window::new()))
            .modal(true)
            .message_type(gtk4::MessageType::Question)
            .text(&title)
            .secondary_text(&message)
            .build();

        dialog.add_button(&cancel_text, ResponseType::Cancel);
        dialog.add_button(&discard_text, ResponseType::No);
        dialog.add_button(&save_text, ResponseType::Yes);

        // Set default response to Save
        dialog.set_default_response(ResponseType::Yes);

        println!(
            "DEBUG: Dialog created with buttons - Cancel: {:?}, Discard: {:?}, Save: {:?}",
            ResponseType::Cancel,
            ResponseType::No,
            ResponseType::Yes
        );

        // Handle dialog response asynchronously
        let editor_weak = self.clone();
        let parent_window = parent.map(|w| w.clone());

        println!("DEBUG: Setting up dialog response callback");

        // Clone the callback for the save case
        let on_confirm_quit_for_save = std::rc::Rc::new(on_confirm_quit);
        let on_confirm_quit_for_discard = on_confirm_quit_for_save.clone();

        // Use a flag to ensure the dialog response is only handled once
        let response_handled = std::rc::Rc::new(std::cell::RefCell::new(false));
        let response_handled_clone = response_handled.clone();

        dialog.connect_response(move |dialog, response| {
            println!("DEBUG: Dialog response received: {:?}", response);

            // Check if response was already handled
            if *response_handled_clone.borrow() {
                println!("DEBUG: Dialog response already handled, ignoring");
                return;
            }

            match response {
                ResponseType::Yes => {
                    // User wants to save before quitting
                    println!("DEBUG: User clicked Save button");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();

                    // Use the callback-based save method to only quit if save is successful
                    let quit_callback = on_confirm_quit_for_save.clone();
                    editor_weak.save_current_file_with_callback(
                        parent_window.as_ref(),
                        move || {
                            println!("DEBUG: Save completed successfully, calling quit callback");
                            quit_callback();
                            println!("DEBUG: on_confirm_quit callback completed");
                        },
                    );
                }
                ResponseType::No => {
                    // User wants to discard changes and quit
                    println!("DEBUG: User clicked Don't Save button");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                    println!("DEBUG: Dialog closed, about to call quit callback");
                    (*on_confirm_quit_for_discard)();
                    println!("DEBUG: on_confirm_quit callback completed");
                }
                ResponseType::Cancel => {
                    // User explicitly clicked Cancel button
                    println!("DEBUG: User clicked Cancel button");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                }
                ResponseType::DeleteEvent => {
                    // Dialog was closed by window manager (X button) - treat as cancel
                    println!("DEBUG: Dialog closed by window manager, treating as cancel");
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                }
                _ => {
                    // Other responses - treat as cancel
                    println!(
                        "DEBUG: Other dialog response: {:?}, treating as cancel",
                        response
                    );
                    *response_handled_clone.borrow_mut() = true;
                    dialog.close();
                }
            }
        });

        // Show the dialog
        println!("DEBUG: Presenting dialog to user");
        dialog.present();

        // Return false to indicate that quit should not proceed immediately
        // The actual quit will happen in the dialog response callback
        println!("DEBUG: Returning false - quit should wait for dialog response");
        false
    }
}
