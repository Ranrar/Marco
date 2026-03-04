use crate::components::bookmarks::{bookmark_menu_label_with_path, BookmarkManager};
use crate::components::language::MenuTranslations;
use gtk4::{gio, prelude::*};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const MAX_BOOKMARK_ACTIONS: usize = 256;

fn clear_menu(menu: &gio::Menu) {
    while menu.n_items() > 0 {
        menu.remove(0);
    }
}

#[allow(clippy::too_many_arguments)]
fn update_bookmarks_menu_actions(
    app: &gtk4::Application,
    bookmarks_menu: &gio::Menu,
    bookmark_manager: &Rc<BookmarkManager>,
    current_file_provider: &Rc<dyn Fn() -> Option<PathBuf>>,
    jump_to_bookmark: &Rc<dyn Fn(PathBuf, u32)>,
    menu_translations: &MenuTranslations,
) {
    clear_menu(bookmarks_menu);

    for i in 0..MAX_BOOKMARK_ACTIONS {
        let action_name = format!("open_bookmark_{}", i);
        if app.lookup_action(&action_name).is_some() {
            app.remove_action(&action_name);
        }
    }

    let current_file = current_file_provider();
    let (current_doc, other_docs) = bookmark_manager.grouped_by_current(current_file.as_deref());

    let mut flattened = Vec::with_capacity(current_doc.len() + other_docs.len());

    if !current_doc.is_empty() {
        for entry in &current_doc {
            let line_label = format!("Line {}", entry.line + 1).replace('_', "__");
            let action_name = format!("app.open_bookmark_{}", flattened.len());
            bookmarks_menu.append(Some(&line_label), Some(&action_name));
            flattened.push((entry.file_path.clone(), entry.line));
        }
    }

    if !other_docs.is_empty() {
        let section = gio::Menu::new();
        for entry in &other_docs {
            let label = bookmark_menu_label_with_path(entry);
            let action_name = format!("app.open_bookmark_{}", flattened.len());
            section.append(Some(&label), Some(&action_name));
            flattened.push((entry.file_path.clone(), entry.line));
        }
        bookmarks_menu.append_section(None, &section);
    }

    if flattened.is_empty() {
        bookmarks_menu.append(Some(&menu_translations.no_bookmarks), None);
        return;
    }

    for (i, (path, line)) in flattened.into_iter().enumerate().take(MAX_BOOKMARK_ACTIONS) {
        let action_name = format!("open_bookmark_{}", i);
        let action = gio::SimpleAction::new(&action_name, None);
        let jump_to_bookmark = Rc::clone(jump_to_bookmark);
        action.connect_activate(move |_, _| {
            jump_to_bookmark(path.clone(), line);
        });
        app.add_action(&action);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_bookmark_actions(
    app: &gtk4::Application,
    bookmarks_menu: &gio::Menu,
    bookmark_manager: Rc<BookmarkManager>,
    current_file_provider: Rc<dyn Fn() -> Option<PathBuf>>,
    jump_to_bookmark: Rc<dyn Fn(PathBuf, u32)>,
    menu_translations: Rc<RefCell<MenuTranslations>>,
) {
    let app_owned = app.clone();
    let bookmarks_menu_owned = bookmarks_menu.clone();

    update_bookmarks_menu_actions(
        &app_owned,
        &bookmarks_menu_owned,
        &bookmark_manager,
        &current_file_provider,
        &jump_to_bookmark,
        &menu_translations.borrow(),
    );

    {
        let app_owned = app_owned.clone();
        let bookmarks_menu_owned = bookmarks_menu_owned.clone();
        let bookmark_manager_owned = bookmark_manager.clone();
        let current_file_provider_owned = current_file_provider.clone();
        let jump_to_bookmark_owned = jump_to_bookmark.clone();
        let menu_translations_owned = menu_translations.clone();
        bookmark_manager.register_changed_callback(move || {
            update_bookmarks_menu_actions(
                &app_owned,
                &bookmarks_menu_owned,
                &bookmark_manager_owned,
                &current_file_provider_owned,
                &jump_to_bookmark_owned,
                &menu_translations_owned.borrow(),
            );
        });
    }
}

pub fn refresh_bookmark_menu(
    app: &gtk4::Application,
    bookmarks_menu: &gio::Menu,
    bookmark_manager: &Rc<BookmarkManager>,
    current_file_provider: &Rc<dyn Fn() -> Option<PathBuf>>,
    jump_to_bookmark: &Rc<dyn Fn(PathBuf, u32)>,
    menu_translations: &MenuTranslations,
) {
    update_bookmarks_menu_actions(
        app,
        bookmarks_menu,
        bookmark_manager,
        current_file_provider,
        jump_to_bookmark,
        menu_translations,
    );
}
