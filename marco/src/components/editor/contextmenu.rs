//! Editor context menu.
//!
//! Uses `GtkPopoverMenu` (built from a `GMenuModel`) with
//! `PopoverMenuFlags::NESTED` so that submenus appear as native floating
//! panels to the right of their parent item — no manual geometry or
//! coordinate translation is needed.
//!
//! All operations are wired through `GSimpleAction` objects registered on
//! the `GtkApplication`.  `set_enabled()` is called on each action just
//! before the menu is shown, giving correct greyed-out states.  The
//! bookmark label ("Add Bookmark" / "Remove Bookmark") is updated by
//! replacing the item in a dedicated mutable `gio::Menu` section.

use gtk4::{gdk, gio, prelude::*, PopoverMenuFlags, TextBuffer};
use std::{cell::RefCell, rc::Rc};

use crate::components::bookmarks::BookmarkManager;
use crate::components::editor::table_edit;
use crate::ui::menu_items::FileOperations;

// ── action names ─────────────────────────────────────────────────────────────

/// Reuse the existing bookmark action name that the rest of the app expects.
const TOGGLE_BOOKMARK_ACTION: &str = "toggle_bookmark_current_line";

const CTX_UNDO: &str = "ctx-undo";
const CTX_REDO: &str = "ctx-redo";

const CTX_CUT: &str = "ctx-cut";
const CTX_COPY: &str = "ctx-copy";
const CTX_PASTE: &str = "ctx-paste";
const CTX_DELETE: &str = "ctx-delete";

const CTX_SELECT_ALL: &str = "ctx-select-all";

const CTX_INDENT_INC: &str = "ctx-indent-increase";
const CTX_INDENT_DEC: &str = "ctx-indent-decrease";

const CTX_FORMAT_TABLE: &str = "ctx-format-table";
const CTX_INSERT_ROW_ABOVE: &str = "ctx-insert-row-above";
const CTX_INSERT_ROW_BELOW: &str = "ctx-insert-row-below";
const CTX_DELETE_ROW: &str = "ctx-delete-row";
const CTX_MOVE_ROW_UP: &str = "ctx-move-row-up";
const CTX_MOVE_ROW_DOWN: &str = "ctx-move-row-down";

const CTX_INSERT_COL_LEFT: &str = "ctx-insert-col-left";
const CTX_INSERT_COL_RIGHT: &str = "ctx-insert-col-right";
const CTX_DELETE_COL: &str = "ctx-delete-col";
const CTX_MOVE_COL_LEFT: &str = "ctx-move-col-left";
const CTX_MOVE_COL_RIGHT: &str = "ctx-move-col-right";

const CTX_ALIGN_COL_LEFT: &str = "ctx-align-col-left";
const CTX_ALIGN_COL_CENTER: &str = "ctx-align-col-center";
const CTX_ALIGN_COL_RIGHT: &str = "ctx-align-col-right";

// ── GMenuModel helpers ────────────────────────────────────────────────────────

/// Create a `GMenuItem` for `action` (prefixed with `app.`).
fn item(label: &str, action: &str) -> gio::MenuItem {
    gio::MenuItem::new(Some(label), Some(&format!("app.{action}")))
}

/// Create a `GMenuItem` with an accelerator hint label.
fn item_accel(label: &str, action: &str, accel: &str) -> gio::MenuItem {
    let mi = gio::MenuItem::new(Some(label), Some(&format!("app.{action}")));
    mi.set_attribute_value("accel", Some(&accel.to_variant()));
    mi
}

// ── text-buffer operations ────────────────────────────────────────────────────

fn delete_selection_or_next_char(tb: &TextBuffer) {
    if let Some((mut s, mut e)) = tb.selection_bounds() {
        tb.begin_user_action();
        tb.delete(&mut s, &mut e);
        tb.end_user_action();
        return;
    }
    let mut s = tb.iter_at_offset(tb.cursor_position());
    let mut e = s;
    if e.forward_char() {
        tb.begin_user_action();
        tb.delete(&mut s, &mut e);
        tb.end_user_action();
    }
}

fn increase_indent(tb: &TextBuffer) {
    let (sl, el) = selection_or_cursor_lines(tb);
    tb.begin_user_action();
    for line in (sl..=el).rev() {
        if let Some(mut it) = tb.iter_at_line(line) {
            tb.insert(&mut it, "\t");
        }
    }
    tb.end_user_action();
}

fn decrease_indent(tb: &TextBuffer) {
    let (sl, el) = selection_or_cursor_lines(tb);
    tb.begin_user_action();
    for line in (sl..=el).rev() {
        if let Some(mut line_start) = tb.iter_at_line(line) {
            let mut next = line_start;
            if next.forward_char() {
                let first = tb.text(&line_start, &next, false).to_string();
                if first == "\t" {
                    tb.delete(&mut line_start, &mut next);
                    continue;
                }
                if first == " " {
                    let mut remove_end = line_start;
                    let mut removed = 0usize;
                    while removed < 4 {
                        let mut probe = remove_end;
                        if !probe.forward_char() {
                            break;
                        }
                        if tb.text(&remove_end, &probe, false).as_str() != " " {
                            break;
                        }
                        remove_end = probe;
                        removed += 1;
                    }
                    if removed > 0 {
                        tb.delete(&mut line_start, &mut remove_end);
                    }
                }
            }
        }
    }
    tb.end_user_action();
}

fn selection_or_cursor_lines(tb: &TextBuffer) -> (i32, i32) {
    if let Some((s, e)) = tb.selection_bounds() {
        let sl = s.line().max(0);
        let mut el = e.line().max(0);
        if e.starts_line() && el > sl {
            el -= 1;
        }
        (sl, el.max(sl))
    } else {
        let l = tb.iter_at_offset(tb.cursor_position()).line().max(0);
        (l, l)
    }
}

fn bookmark_state(
    bookmark_manager: &BookmarkManager,
    file_operations_rc: &Rc<RefCell<FileOperations>>,
    editor_buffer: &sourceview5::Buffer,
) -> Option<bool> {
    let path = file_operations_rc
        .borrow()
        .buffer
        .borrow()
        .get_file_path()
        .map(|p| p.to_path_buf())?;
    let line = editor_buffer
        .iter_at_offset(editor_buffer.cursor_position())
        .line()
        .max(0) as u32;
    Some(bookmark_manager.is_bookmarked(path, line))
}

// ── action set ────────────────────────────────────────────────────────────────

/// Holds every `GSimpleAction` and the mutable bookmark section so we can
/// update enabled-states and labels before each popup.
struct CtxActions {
    undo: gio::SimpleAction,
    redo: gio::SimpleAction,
    cut: gio::SimpleAction,
    copy: gio::SimpleAction,
    paste: gio::SimpleAction,
    delete: gio::SimpleAction,
    select_all: gio::SimpleAction,
    indent_inc: gio::SimpleAction,
    indent_dec: gio::SimpleAction,
    format_table: gio::SimpleAction,
    insert_row_above: gio::SimpleAction,
    insert_row_below: gio::SimpleAction,
    delete_row: gio::SimpleAction,
    move_row_up: gio::SimpleAction,
    move_row_down: gio::SimpleAction,
    insert_col_left: gio::SimpleAction,
    insert_col_right: gio::SimpleAction,
    delete_col: gio::SimpleAction,
    move_col_left: gio::SimpleAction,
    move_col_right: gio::SimpleAction,
    align_col_left: gio::SimpleAction,
    align_col_center: gio::SimpleAction,
    align_col_right: gio::SimpleAction,
    toggle_bookmark: gio::SimpleAction,
    /// Mutable section so the bookmark label updates before each popup.
    bookmark_section: gio::Menu,
}

impl CtxActions {
    fn new() -> Self {
        let sa = |n: &str| gio::SimpleAction::new(n, None);
        Self {
            undo: sa(CTX_UNDO),
            redo: sa(CTX_REDO),
            cut: sa(CTX_CUT),
            copy: sa(CTX_COPY),
            paste: sa(CTX_PASTE),
            delete: sa(CTX_DELETE),
            select_all: sa(CTX_SELECT_ALL),
            indent_inc: sa(CTX_INDENT_INC),
            indent_dec: sa(CTX_INDENT_DEC),
            format_table: sa(CTX_FORMAT_TABLE),
            insert_row_above: sa(CTX_INSERT_ROW_ABOVE),
            insert_row_below: sa(CTX_INSERT_ROW_BELOW),
            delete_row: sa(CTX_DELETE_ROW),
            move_row_up: sa(CTX_MOVE_ROW_UP),
            move_row_down: sa(CTX_MOVE_ROW_DOWN),
            insert_col_left: sa(CTX_INSERT_COL_LEFT),
            insert_col_right: sa(CTX_INSERT_COL_RIGHT),
            delete_col: sa(CTX_DELETE_COL),
            move_col_left: sa(CTX_MOVE_COL_LEFT),
            move_col_right: sa(CTX_MOVE_COL_RIGHT),
            align_col_left: sa(CTX_ALIGN_COL_LEFT),
            align_col_center: sa(CTX_ALIGN_COL_CENTER),
            align_col_right: sa(CTX_ALIGN_COL_RIGHT),
            toggle_bookmark: sa(TOGGLE_BOOKMARK_ACTION),
            bookmark_section: gio::Menu::new(),
        }
    }

    /// Register every action on the application (called once at startup).
    fn register_on(&self, app: &gtk4::Application) {
        app.add_action(&self.undo);
        app.add_action(&self.redo);
        app.add_action(&self.cut);
        app.add_action(&self.copy);
        app.add_action(&self.paste);
        app.add_action(&self.delete);
        app.add_action(&self.select_all);
        app.add_action(&self.indent_inc);
        app.add_action(&self.indent_dec);
        app.add_action(&self.format_table);
        app.add_action(&self.insert_row_above);
        app.add_action(&self.insert_row_below);
        app.add_action(&self.delete_row);
        app.add_action(&self.move_row_up);
        app.add_action(&self.move_row_down);
        app.add_action(&self.insert_col_left);
        app.add_action(&self.insert_col_right);
        app.add_action(&self.delete_col);
        app.add_action(&self.move_col_left);
        app.add_action(&self.move_col_right);
        app.add_action(&self.align_col_left);
        app.add_action(&self.align_col_center);
        app.add_action(&self.align_col_right);
        app.add_action(&self.toggle_bookmark);
    }
}

// ── GMenuModel builder ────────────────────────────────────────────────────────

/// Build the complete menu model.  `bookmark_section` is kept as a live
/// reference so its contents can be updated before each popup.
fn build_model(bookmark_section: &gio::Menu) -> gio::Menu {
    let root = gio::Menu::new();

    // History
    let history = gio::Menu::new();
    history.append_item(&item_accel("Undo", CTX_UNDO, "<Ctrl>z"));
    history.append_item(&item_accel("Redo", CTX_REDO, "<Ctrl>y"));
    root.append_section(None, &history);

    // Clipboard
    let clipboard = gio::Menu::new();
    clipboard.append_item(&item_accel("Cut", CTX_CUT, "<Ctrl>x"));
    clipboard.append_item(&item_accel("Copy", CTX_COPY, "<Ctrl>c"));
    clipboard.append_item(&item_accel("Paste", CTX_PASTE, "<Ctrl>v"));
    clipboard.append_item(&item("Delete", CTX_DELETE));
    root.append_section(None, &clipboard);

    // Selection
    let selection = gio::Menu::new();
    selection.append_item(&item_accel("Select All", CTX_SELECT_ALL, "<Ctrl>a"));
    root.append_section(None, &selection);

    // Indent
    let indent = gio::Menu::new();
    indent.append_item(&item_accel("Increase Indent", CTX_INDENT_INC, "Tab"));
    indent.append_item(&item_accel("Decrease Indent", CTX_INDENT_DEC, "<Shift>Tab"));
    root.append_section(None, &indent);

    // Table operations with nested submenus
    let table = gio::Menu::new();
    table.append_item(&item("Format Table", CTX_FORMAT_TABLE));

    let rows = gio::Menu::new();
    rows.append_item(&item("Insert Row Above", CTX_INSERT_ROW_ABOVE));
    rows.append_item(&item("Insert Row Below", CTX_INSERT_ROW_BELOW));
    rows.append_item(&item("Delete Row", CTX_DELETE_ROW));
    rows.append_item(&item("Move Row Up", CTX_MOVE_ROW_UP));
    rows.append_item(&item("Move Row Down", CTX_MOVE_ROW_DOWN));
    table.append_submenu(Some("Rows"), &rows);

    let cols = gio::Menu::new();
    cols.append_item(&item("Insert Column Left", CTX_INSERT_COL_LEFT));
    cols.append_item(&item("Insert Column Right", CTX_INSERT_COL_RIGHT));
    cols.append_item(&item("Delete Column", CTX_DELETE_COL));
    cols.append_item(&item("Move Column Left", CTX_MOVE_COL_LEFT));
    cols.append_item(&item("Move Column Right", CTX_MOVE_COL_RIGHT));
    table.append_submenu(Some("Columns"), &cols);

    let align = gio::Menu::new();
    align.append_item(&item("Align Left", CTX_ALIGN_COL_LEFT));
    align.append_item(&item("Align Center", CTX_ALIGN_COL_CENTER));
    align.append_item(&item("Align Right", CTX_ALIGN_COL_RIGHT));
    table.append_submenu(Some("Alignment"), &align);

    root.append_section(None, &table);

    // Bookmark (mutable section - populated before each popup)
    bookmark_section.remove_all();
    bookmark_section.append_item(&item("Add/Remove Bookmark", TOGGLE_BOOKMARK_ACTION));
    root.append_section(None, bookmark_section);

    root
}

// ── public entry point ────────────────────────────────────────────────────────

/// Install editor context menu actions and right-click popover on the editor view.
///
/// Uses `GtkPopoverMenu::from_model_full` with `PopoverMenuFlags::NESTED` so
/// that submenus are displayed as native cascading panels positioned by GTK —
/// no manual coordinate translation is required.
pub fn setup_editor_context_menu(
    app: &gtk4::Application,
    editor_source_view: &sourceview5::View,
    editor_buffer: &sourceview5::Buffer,
    bookmark_manager: Rc<BookmarkManager>,
    file_operations_rc: Rc<RefCell<FileOperations>>,
) {
    // Create and register all actions.
    let actions = Rc::new(CtxActions::new());
    actions.register_on(app);

    // ── wire bookmark toggle ─────────────────────────────────────────────────
    {
        let bm = bookmark_manager.clone();
        let fop = file_operations_rc.clone();
        let buf = editor_buffer.clone();
        actions.toggle_bookmark.connect_activate(move |_, _| {
            let Some(path) = fop
                .borrow()
                .buffer
                .borrow()
                .get_file_path()
                .map(|p| p.to_path_buf())
            else {
                return;
            };
            let line = buf.iter_at_offset(buf.cursor_position()).line().max(0) as u32;
            bm.toggle(path, line);
        });
    }

    // ── wire all other operations ────────────────────────────────────────────
    {
        let buf = editor_buffer.clone();
        actions.undo.connect_activate(move |_, _| {
            if buf.can_undo() {
                buf.undo();
            }
        });
    }
    {
        let buf = editor_buffer.clone();
        actions.redo.connect_activate(move |_, _| {
            if buf.can_redo() {
                buf.redo();
            }
        });
    }
    {
        let buf = editor_buffer.clone();
        let sv = editor_source_view.clone();
        actions.cut.connect_activate(move |_, _| {
            if let Some(d) = gdk::Display::default() {
                buf.cut_clipboard(&d.clipboard(), sv.is_editable());
            }
        });
    }
    {
        let buf = editor_buffer.clone();
        actions.copy.connect_activate(move |_, _| {
            if let Some(d) = gdk::Display::default() {
                buf.copy_clipboard(&d.clipboard());
            }
        });
    }
    {
        let buf = editor_buffer.clone();
        let sv = editor_source_view.clone();
        actions.paste.connect_activate(move |_, _| {
            if let Some(d) = gdk::Display::default() {
                buf.paste_clipboard(&d.clipboard(), None, sv.is_editable());
            }
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.delete.connect_activate(move |_, _| {
            delete_selection_or_next_char(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.select_all.connect_activate(move |_, _| {
            tb.select_range(&tb.start_iter(), &tb.end_iter());
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.indent_inc.connect_activate(move |_, _| {
            increase_indent(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.indent_dec.connect_activate(move |_, _| {
            decrease_indent(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.format_table.connect_activate(move |_, _| {
            table_edit::format_table_at_cursor(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.insert_row_above.connect_activate(move |_, _| {
            table_edit::insert_row_above(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.insert_row_below.connect_activate(move |_, _| {
            table_edit::insert_row_below(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.delete_row.connect_activate(move |_, _| {
            table_edit::delete_current_row(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.move_row_up.connect_activate(move |_, _| {
            table_edit::move_current_row_up(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.move_row_down.connect_activate(move |_, _| {
            table_edit::move_current_row_down(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.insert_col_left.connect_activate(move |_, _| {
            table_edit::insert_column_left(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.insert_col_right.connect_activate(move |_, _| {
            table_edit::insert_column_right(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.delete_col.connect_activate(move |_, _| {
            table_edit::delete_current_column(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.move_col_left.connect_activate(move |_, _| {
            table_edit::move_current_column_left(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.move_col_right.connect_activate(move |_, _| {
            table_edit::move_current_column_right(&tb);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.align_col_left.connect_activate(move |_, _| {
            table_edit::align_current_column(&tb, table_edit::ColumnAlignment::Left);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.align_col_center.connect_activate(move |_, _| {
            table_edit::align_current_column(&tb, table_edit::ColumnAlignment::Center);
        });
    }
    {
        let tb: TextBuffer = editor_buffer.clone().upcast();
        actions.align_col_right.connect_activate(move |_, _| {
            table_edit::align_current_column(&tb, table_edit::ColumnAlignment::Right);
        });
    }

    // ── build menu model and GtkPopoverMenu ──────────────────────────────────
    //
    // PopoverMenuFlags::NESTED makes each submenu appear as a floating panel
    // anchored to its parent item — GTK handles all positioning internally.
    let model = build_model(&actions.bookmark_section);
    let popover = gtk4::PopoverMenu::from_model_full(&model, PopoverMenuFlags::NESTED);
    let popover_base: gtk4::Popover = popover.clone().upcast();
    popover.set_parent(editor_source_view);
    crate::ui::popover_state::enforce_dismiss_behavior(&popover_base);
    // Keep parent context menu open while navigating into a submenu.
    popover.set_cascade_popdown(false);
    // Add the same CSS class used by the rest of the app's popovers.
    popover.add_css_class("editor-context-menu");

    // ── right-click gesture ───────────────────────────────────────────────────
    let secondary_click = gtk4::GestureClick::new();
    secondary_click.set_button(gdk::BUTTON_SECONDARY);
    secondary_click.set_propagation_phase(gtk4::PropagationPhase::Capture);

    // Claim the press so SourceView does not react to it.
    secondary_click.connect_pressed(|gesture, _n, _x, _y| {
        gesture.set_state(gtk4::EventSequenceState::Claimed);
    });

    {
        let actions = actions.clone();
        let editor_buffer = editor_buffer.clone();
        let bookmark_manager = bookmark_manager.clone();
        let file_operations_rc = file_operations_rc.clone();
        let popover = popover.clone();

        secondary_click.connect_released(move |_gesture, _n, x, y| {
            // ── update action enabled states ──────────────────────────────────
            actions.undo.set_enabled(editor_buffer.can_undo());
            actions.redo.set_enabled(editor_buffer.can_redo());

            let has_sel = editor_buffer.selection_bounds().is_some();
            actions.cut.set_enabled(has_sel);
            actions.copy.set_enabled(has_sel);
            actions
                .delete
                .set_enabled(has_sel || editor_buffer.char_count() > 0);

            let tb: TextBuffer = editor_buffer.clone().upcast();

            let can_outdent = {
                let (sl, el) = selection_or_cursor_lines(&tb);
                (sl..=el).any(|line| {
                    tb.iter_at_line(line).is_some_and(|s| {
                        let mut e = s;
                        e.forward_char() && matches!(tb.text(&s, &e, false).as_str(), "\t" | " ")
                    })
                })
            };
            actions.indent_dec.set_enabled(can_outdent);

            let caps = table_edit::table_action_availability(&tb);
            actions.format_table.set_enabled(caps.in_table);
            actions.insert_row_above.set_enabled(caps.in_table);
            actions.insert_row_below.set_enabled(caps.in_table);
            actions.delete_row.set_enabled(caps.can_delete_row);
            actions.move_row_up.set_enabled(caps.can_move_row_up);
            actions.move_row_down.set_enabled(caps.can_move_row_down);
            actions.insert_col_left.set_enabled(caps.in_table);
            actions.insert_col_right.set_enabled(caps.in_table);
            actions.delete_col.set_enabled(caps.can_delete_column);
            actions.move_col_left.set_enabled(caps.can_move_column_left);
            actions
                .move_col_right
                .set_enabled(caps.can_move_column_right);
            actions.align_col_left.set_enabled(caps.can_align_column);
            actions.align_col_center.set_enabled(caps.can_align_column);
            actions.align_col_right.set_enabled(caps.can_align_column);

            // Table submenu triggers: greyed out as a group when not in a table.
            let in_table = caps.in_table;
            actions.format_table.set_enabled(in_table);

            // ── update bookmark label ─────────────────────────────────────────
            let (label, bm_enabled) =
                match bookmark_state(&bookmark_manager, &file_operations_rc, &editor_buffer) {
                    Some(true) => ("Remove Bookmark", true),
                    Some(false) => ("Add Bookmark", true),
                    None => ("Add/Remove Bookmark", false),
                };
            actions.toggle_bookmark.set_enabled(bm_enabled);
            actions.bookmark_section.remove_all();
            actions
                .bookmark_section
                .append_item(&item(label, TOGGLE_BOOKMARK_ACTION));

            // ── position and show ─────────────────────────────────────────────
            // Navigate back to the root page first so re-opening always shows
            // the top-level menu even if a submenu was last visible.
            popover.set_visible_submenu(None::<&str>);
            popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            popover.popup();
        });
    }

    editor_source_view.add_controller(secondary_click);
}
