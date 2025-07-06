use gtk4::prelude::*;
use gtk4::{Application, gio};
use crate::{editor, language};

pub fn add_file_menu(menu_model: &gio::Menu) {
    let file_menu = gio::Menu::new();
    file_menu.append(Some(&language::tr("menu.new")), Some("app.new"));
    file_menu.append(Some(&language::tr("menu.open")), Some("app.open"));
    file_menu.append(Some(&language::tr("menu.save")), Some("app.save"));
    file_menu.append(Some(&language::tr("menu.save_as")), Some("app.save_as"));
    file_menu.append(Some(&language::tr("menu.quit")), Some("app.quit"));
    
    menu_model.append_submenu(Some(&language::tr("menu.file")), &file_menu);
}

pub fn add_edit_menu(menu_model: &gio::Menu) {
    let edit_menu = gio::Menu::new();
    edit_menu.append(Some(&language::tr("menu.undo")), Some("app.undo"));
    edit_menu.append(Some(&language::tr("menu.redo")), Some("app.redo"));
    edit_menu.append(Some(&language::tr("menu.cut")), Some("app.cut"));
    edit_menu.append(Some(&language::tr("menu.copy")), Some("app.copy"));
    edit_menu.append(Some(&language::tr("menu.paste")), Some("app.paste"));
    edit_menu.append(Some(&language::tr("menu.select_all")), Some("app.select_all"));
    edit_menu.append(Some(&language::tr("menu.find")), Some("app.find"));
    edit_menu.append(Some(&language::tr("menu.replace")), Some("app.replace"));
    
    menu_model.append_submenu(Some(&language::tr("menu.edit")), &edit_menu);
}

pub fn create_file_actions(app: &Application, editor: &editor::MarkdownEditor) {
    let new_action = gio::ActionEntry::builder("new")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.new_file();
            }
        })
        .build();
    
    let open_action = gio::ActionEntry::builder("open")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.open_file_from_menu(&window);
                }
            }
        })
        .build();
    
    let save_action = gio::ActionEntry::builder("save")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.save_file_from_menu(&window);
                }
            }
        })
        .build();
    
    let save_as_action = gio::ActionEntry::builder("save_as")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.save_as_file_from_menu(&window);
                }
            }
        })
        .build();
    
    let quit_action = gio::ActionEntry::builder("quit")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                println!("Quit clicked");
                
                // Get the main window to use as parent for the dialog
                let window = app.active_window();
                let app_clone = app.clone();
                
                // Check if there are unsaved changes and show confirmation if needed
                let should_quit_immediately = editor.show_unsaved_changes_dialog_and_quit(
                    window.as_ref(), 
                    move || {
                        println!("DEBUG: Confirmed quit, calling app.quit()");
                        app_clone.quit();
                    }
                );
                
                if should_quit_immediately {
                    // No unsaved changes, quit immediately
                    app.quit();
                }
                // Otherwise, the quit will happen in the dialog callback
            }
        })
        .build();
    
    app.add_action_entries([new_action, open_action, save_action, save_as_action, quit_action]);
    
    // Set keyboard accelerators for File menu actions
    app.set_accels_for_action("app.new", &["<Ctrl>n"]);
    app.set_accels_for_action("app.open", &["<Ctrl>o"]);
    app.set_accels_for_action("app.save", &["<Ctrl>s"]);
    app.set_accels_for_action("app.save_as", &["<Ctrl><Shift>s"]);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);
}

pub fn create_edit_actions(app: &Application, editor: &editor::MarkdownEditor) {
    let undo_action = gio::ActionEntry::builder("undo")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.undo();
            }
        })
        .build();
    
    let redo_action = gio::ActionEntry::builder("redo")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.redo();
            }
        })
        .build();
    
    let cut_action = gio::ActionEntry::builder("cut")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.cut();
            }
        })
        .build();
    
    let copy_action = gio::ActionEntry::builder("copy")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.copy();
            }
        })
        .build();
    
    let paste_action = gio::ActionEntry::builder("paste")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                editor.paste();
            }
        })
        .build();
    
    let find_action = gio::ActionEntry::builder("find")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.show_find_dialog(&window);
                }
            }
        })
        .build();
     let replace_action = gio::ActionEntry::builder("replace")
        .activate({
            let editor = editor.clone();
            move |app: &Application, _action, _param| {
                if let Some(window) = app.active_window() {
                    editor.show_replace_dialog(&window);
                }
            }
        })
        .build();
    
    let select_all_action = gio::ActionEntry::builder("select_all")
        .activate({
            let editor = editor.clone();
            move |_app: &Application, _action, _param| {
                let source_buffer = editor.source_buffer();
                let gtk_buffer = source_buffer.upcast_ref::<gtk4::TextBuffer>();
                let start_iter = gtk_buffer.start_iter();
                let end_iter = gtk_buffer.end_iter();
                gtk_buffer.select_range(&start_iter, &end_iter);
            }
        })
        .build();

    app.add_action_entries([undo_action, redo_action, cut_action, copy_action, paste_action, find_action, replace_action, select_all_action]);
    
    // Set keyboard accelerators for Edit menu actions
    app.set_accels_for_action("app.undo", &["<Ctrl>z"]);
    app.set_accels_for_action("app.redo", &["<Ctrl>y"]);
    app.set_accels_for_action("app.cut", &["<Ctrl>x"]);
    app.set_accels_for_action("app.copy", &["<Ctrl>c"]);
    app.set_accels_for_action("app.paste", &["<Ctrl>v"]);
    app.set_accels_for_action("app.select_all", &["<Ctrl>a"]);
    app.set_accels_for_action("app.find", &["<Ctrl>f"]);
    app.set_accels_for_action("app.replace", &["<Ctrl>h"]);
}
