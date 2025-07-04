use gtk4::prelude::*;
use gtk4::{Application, gio, MessageDialog, MessageType, ResponseType};
use crate::{editor, language};

/// Show a dialog notifying the user that text must be selected for the styling function
fn show_text_selection_required_dialog(parent: &gtk4::Window, feature_name: &str) {
    let title = "Text Selection Required";
    let message = format!("Please select text in the editor before applying {} formatting.", feature_name);
    
    let dialog = MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .message_type(MessageType::Info)
        .text(title)
        .secondary_text(&message)
        .build();
    
    dialog.add_button("OK", ResponseType::Ok);
    dialog.set_default_response(ResponseType::Ok);
    
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    
    dialog.present();
}

pub fn add_advanced_menu(menu_model: &gio::Menu) {
    // Advanced Menu (Markdown Hacks from markdownguide.org/hacks/)
    let advanced_menu = gio::Menu::new();
    
    // Text styling hacks
    let text_styling_menu = gio::Menu::new();
    text_styling_menu.append(Some("🔤 Underline"), Some("app.insert_underline"));
    text_styling_menu.append(Some("📐 Center Text"), Some("app.insert_center_text"));
    text_styling_menu.append(Some("🎨 Colored Text"), Some("app.insert_colored_text"));
    text_styling_menu.append(Some("📝 Indent Text"), Some("app.insert_indent_text"));
    advanced_menu.append_submenu(Some(&language::tr("advanced.text_styling")), &text_styling_menu);
    
    // Comments and admonitions
    let comments_menu = gio::Menu::new();
    comments_menu.append(Some("💬 Comment"), Some("app.insert_comment"));
    comments_menu.append(Some("⚠️ Admonition"), Some("app.insert_admonition"));
    advanced_menu.append_submenu(Some(&language::tr("advanced.comments_admonitions")), &comments_menu);
    
    // Enhanced images and links
    let media_menu = gio::Menu::new();
    media_menu.append(Some("🖼️ Image with Size"), Some("app.insert_image_with_size"));
    media_menu.append(Some("🖼️ Image with Caption"), Some("app.insert_image_with_caption"));
    media_menu.append(Some("🔗 Link Open New"), Some("app.insert_link_open_new"));
    media_menu.append(Some("📹 YouTube Video"), Some("app.insert_youtube_video"));
    advanced_menu.append_submenu(Some(&language::tr("advanced.enhanced_media")), &media_menu);
    
    // Add Table of Contents directly to Advanced menu
    advanced_menu.append(Some("📑 Table of Contents"), Some("app.insert_table_of_contents"));
    
    menu_model.append_submenu(Some(&language::tr("menu.advanced")), &advanced_menu);
}

pub fn create_advanced_actions(app: &Application, editor: &editor::MarkdownEditor) {
    // Text styling actions - only work when text is selected
    let insert_underline_action = gio::ActionEntry::builder("insert_underline")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if editor.has_text_selection() {
                    if let Some(window) = app.active_window() {
                        super::show_underline_dialog(&window, &editor);
                    }
                } else {
                    // Show a notification dialog that text must be selected
                    if let Some(window) = app.active_window() {
                        show_text_selection_required_dialog(&window, "underline");
                    }
                }
            }
        })
        .build();

    let insert_center_text_action = gio::ActionEntry::builder("insert_center_text")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if editor.has_text_selection() {
                    if let Some(window) = app.active_window() {
                        super::show_center_text_dialog(&window, &editor);
                    }
                } else {
                    // Show a notification dialog that text must be selected
                    if let Some(window) = app.active_window() {
                        show_text_selection_required_dialog(&window, "center text");
                    }
                }
            }
        })
        .build();

    let insert_colored_text_action = gio::ActionEntry::builder("insert_colored_text")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if editor.has_text_selection() {
                    if let Some(window) = app.active_window() {
                        super::show_colored_text_dialog(&window, &editor);
                    }
                } else {
                    // Show a notification dialog that text must be selected
                    if let Some(window) = app.active_window() {
                        show_text_selection_required_dialog(&window, "colored text");
                    }
                }
            }
        })
        .build();

    let insert_indent_text_action = gio::ActionEntry::builder("insert_indent_text")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                if editor.has_text_selection() {
                    if let Some(selected_text) = editor.get_selected_text() {
                        editor.insert_indented_text(&selected_text, 1);
                    }
                } else {
                    // For indent, we can work on the current line even without selection
                    editor.insert_indented_text("indented text", 1);
                }
            }
        })
        .build();

    // Comments and admonitions actions
    let insert_comment_action = gio::ActionEntry::builder("insert_comment")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    super::show_comment_dialog(&window, &editor);
                }
            }
        })
        .build();

    let insert_admonition_action = gio::ActionEntry::builder("insert_admonition")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    super::show_admonition_dialog(&window, &editor);
                }
            }
        })
        .build();

    // Enhanced media actions
    let insert_image_with_size_action = gio::ActionEntry::builder("insert_image_with_size")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    super::show_image_with_size_dialog(&window, &editor);
                }
            }
        })
        .build();

    let insert_image_with_caption_action = gio::ActionEntry::builder("insert_image_with_caption")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    crate::menu::dialogs::advanced::image_with_caption::show_image_with_caption_dialog(&window, &editor);
                }
            }
        })
        .build();

    let insert_link_open_new_action = gio::ActionEntry::builder("insert_link_open_new")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    super::show_link_open_new_dialog(&window, &editor);
                }
            }
        })
        .build();

    let insert_youtube_video_action = gio::ActionEntry::builder("insert_youtube_video")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    super::show_youtube_video_dialog(&window, &editor);
                }
            }
        })
        .build();

    // Table of Contents action
    let insert_table_of_contents_action = gio::ActionEntry::builder("insert_table_of_contents")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.insert_table_of_contents();
            }
        })
        .build();

    app.add_action_entries([
        insert_underline_action, insert_center_text_action, insert_colored_text_action,
        insert_indent_text_action, insert_comment_action, insert_admonition_action,
        insert_image_with_size_action, insert_image_with_caption_action, insert_link_open_new_action,
        insert_youtube_video_action, insert_table_of_contents_action
    ]);
}
