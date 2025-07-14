use crate::editor::core::MarkdownEditor;
use gtk4::prelude::*;
use gtk4::{gdk, EventControllerKey};

impl MarkdownEditor {
    /// Set up keyboard shortcuts for common formatting operations
    pub(crate) fn setup_keyboard_shortcuts(&self) {
        let controller = EventControllerKey::new();
        let editor_clone = self.clone();

        controller.connect_key_pressed(move |_, keyval, _, state| {
            // Check for Ctrl modifier
            if state.contains(gdk::ModifierType::CONTROL_MASK) {
                match keyval {
                    gdk::Key::n => {
                        return glib::Propagation::Proceed;
                    }
                    gdk::Key::o => {
                        return glib::Propagation::Proceed;
                    }
                    gdk::Key::s => {
                        return glib::Propagation::Proceed;
                    }
                    gdk::Key::q => {
                        return glib::Propagation::Proceed;
                    }
                    gdk::Key::z => {
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::y => {
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::x => {
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::c => {
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::v => {
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::f => {
                        return glib::Propagation::Proceed;
                    }
                    gdk::Key::h => {
                        return glib::Propagation::Proceed;
                    }
                    gdk::Key::b => {
                        editor_clone.insert_bold();
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::i => {
                        editor_clone.insert_italic();
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::u => {
                        editor_clone.insert_underline("");
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::k => {
                        editor_clone.insert_link();
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::_1 => {
                        editor_clone.insert_heading(1);
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::_2 => {
                        editor_clone.insert_heading(2);
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::_3 => {
                        editor_clone.insert_heading(3);
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::_4 => {
                        editor_clone.insert_heading(4);
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::_5 => {
                        editor_clone.insert_heading(5);
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::_6 => {
                        // Ctrl+6 for heading 6
                        editor_clone.insert_heading(6);
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::grave => {
                        // Ctrl+` for inline code
                        editor_clone.insert_inline_code();
                        return glib::Propagation::Stop;
                    }
                    gdk::Key::period => {
                        // Ctrl+. for emoji picker
                        crate::editor::emoji::show_emoji_picker_shortcut(&editor_clone);
                        return glib::Propagation::Stop;
                    }
                    _ => {}
                }

                // Check for Ctrl+Shift combinations
                if state.contains(gdk::ModifierType::SHIFT_MASK) {
                    match keyval {
                        gdk::Key::period => {
                            editor_clone.insert_numbered_list();
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::_8 => {
                            editor_clone.insert_bullet_list();
                            return glib::Propagation::Stop;
                        }
                        gdk::Key::_7 => {
                            editor_clone.insert_blockquote();
                            return glib::Propagation::Stop;
                        }
                        _ => {}
                    }
                }
            }

            // Check for function keys (F4, F5, F6)
            match keyval {
                gdk::Key::F4 => {
                    // F4: Toggle between current view modes (context menu would show options)
                    if let Some(ref context_menu) = *editor_clone.context_menu.borrow() {
                        context_menu.show_at_cursor(&editor_clone);
                        return glib::Propagation::Stop;
                    }
                }
                gdk::Key::F5 => {
                    // F5: Switch to HTML preview
                    editor_clone.set_view_mode("html");
                    let prefs = crate::settings::get_app_preferences();
                    prefs.set_view_mode("html");
                    return glib::Propagation::Stop;
                }
                gdk::Key::F6 => {
                    // F6: Switch to Code preview
                    editor_clone.set_view_mode("code");
                    let prefs = crate::settings::get_app_preferences();
                    prefs.set_view_mode("code");
                    return glib::Propagation::Stop;
                }
                _ => {}
            }

            // Check for Shift+F10 (context menu at cursor)
            if state.contains(gdk::ModifierType::SHIFT_MASK) && keyval == gdk::Key::F10 {
                if let Some(ref context_menu) = *editor_clone.context_menu.borrow() {
                    context_menu.show_at_cursor(&editor_clone);
                    return glib::Propagation::Stop;
                }
            }

            // Pass through unhandled keys
            glib::Propagation::Proceed
        });

        self.source_view.add_controller(controller);
    }

    /// Undo the last action in the buffer
    pub fn undo(&self) {
        if self.source_buffer.can_undo() {
            self.source_buffer.undo();
        }
    }

    /// Redo the last undone action in the buffer
    pub fn redo(&self) {
        if self.source_buffer.can_redo() {
            self.source_buffer.redo();
        }
    }

    /// Cut selected text to clipboard
    pub fn cut(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();
            self.source_buffer.cut_clipboard(&clipboard, true);
        }
    }

    /// Copy selected text to clipboard
    pub fn copy(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();
            self.source_buffer.copy_clipboard(&clipboard);
        }
    }

    /// Paste text from clipboard
    pub fn paste(&self) {
        if let Some(display) = gdk::Display::default() {
            let clipboard = display.clipboard();
            self.source_buffer.paste_clipboard(&clipboard, None, true);
        }
    }
}
