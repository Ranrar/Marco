//! "Export complete" success dialog.
//!
//! Displayed *after* a PDF or HTML export finishes successfully.  Offers the
//! user three choices, returned via [`ExportCompleteAction`]:
//!
//! * Open the exported document in the system's default app.
//! * Open the parent folder in the system's file manager.
//! * Just close the dialog.
//!
//! Styling matches the rest of Marco's modal dialogs (custom titlebar,
//! `marco-dialog` / `marco-theme-{light,dark}` CSS classes, button-coloring
//! convention from `save.rs`).

use gtk4::{glib, prelude::*, Align, Box as GtkBox, Button, Label, Orientation, Window};
use std::cell::RefCell;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// User's choice in the "Export complete" dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportCompleteAction {
    /// Open the exported document in the system default app.
    OpenDocument,
    /// Reveal the file's containing folder in the OS file manager.
    OpenFolder,
    /// Just close the dialog.
    Close,
}

/// Show the "Export complete" dialog and await the user's choice.
///
/// `parent`        — parent window, used for transient/modal positioning and
///                   theme-class detection.
/// `title`         — titlebar text (e.g. `"PDF Export Complete"`).
/// `message`       — primary message label (e.g. `"PDF exported successfully."`).
/// `output_path`   — path of the file just written; displayed as the secondary
///                   description and used for "Open document" / "Open folder".
pub async fn show_export_complete_dialog<W: IsA<Window>>(
    parent: &W,
    title: &str,
    message: &str,
    output_path: &Path,
) -> ExportCompleteAction {
    // ── Theme detection (mirrors save.rs / exporting.rs) ──────────────────
    let theme_class = if let Some(widget) = parent.dynamic_cast_ref::<gtk4::Widget>() {
        if widget.has_css_class("marco-theme-dark") {
            "marco-theme-dark"
        } else {
            "marco-theme-light"
        }
    } else {
        "marco-theme-light"
    };

    let dialog = Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(420)
        .default_height(180)
        .resizable(false)
        .build();
    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(theme_class);

    // ── Custom titlebar (X-only) ──────────────────────────────────────────
    let titlebar = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        title,
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );
    let btn_close_titlebar = titlebar
        .close_button
        .as_ref()
        .expect("Export complete dialog requires a close button");
    dialog.set_titlebar(Some(&titlebar.headerbar));

    // ── Shared result ─────────────────────────────────────────────────────
    let result: Rc<RefCell<ExportCompleteAction>> =
        Rc::new(RefCell::new(ExportCompleteAction::Close));

    // Titlebar X = Close
    let dialog_weak_for_close = dialog.downgrade();
    let result_for_close = result.clone();
    btn_close_titlebar.connect_clicked(move |_| {
        *result_for_close.borrow_mut() = ExportCompleteAction::Close;
        if let Some(d) = dialog_weak_for_close.upgrade() {
            d.close();
        }
    });

    // ESC = Close
    let key_ctrl = gtk4::EventControllerKey::new();
    let dialog_weak_for_esc = dialog.downgrade();
    let result_for_esc = result.clone();
    key_ctrl.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            *result_for_esc.borrow_mut() = ExportCompleteAction::Close;
            if let Some(d) = dialog_weak_for_esc.upgrade() {
                d.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    dialog.add_controller(key_ctrl);

    // ── Body ──────────────────────────────────────────────────────────────
    let vbox = GtkBox::new(Orientation::Vertical, 0);
    vbox.add_css_class("marco-dialog-content");

    let primary = Label::new(Some(message));
    primary.add_css_class("marco-dialog-title");
    primary.set_halign(Align::Start);
    primary.set_xalign(0.0);
    primary.set_wrap(true);
    primary.set_max_width_chars(55);
    vbox.append(&primary);

    let path_label = Label::new(Some(&output_path.display().to_string()));
    path_label.add_css_class("marco-dialog-description");
    path_label.set_halign(Align::Start);
    path_label.set_xalign(0.0);
    path_label.set_wrap(true);
    path_label.set_max_width_chars(55);
    path_label.set_selectable(true);
    vbox.append(&path_label);

    // ── Buttons ───────────────────────────────────────────────────────────
    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.add_css_class("marco-dialog-button-box");
    button_box.set_halign(Align::Start);
    button_box.set_valign(Align::End);

    let btn_close = Button::with_label("Close");
    btn_close.add_css_class("marco-btn");
    btn_close.add_css_class("marco-btn-yellow");
    button_box.append(&btn_close);

    let btn_folder = Button::with_label("Open folder");
    btn_folder.add_css_class("marco-btn");
    btn_folder.add_css_class("marco-btn-blue");
    button_box.append(&btn_folder);

    let btn_doc = Button::with_label("Open document");
    btn_doc.add_css_class("marco-btn");
    btn_doc.add_css_class("marco-btn-blue");
    button_box.append(&btn_doc);

    vbox.append(&button_box);
    dialog.set_child(Some(&vbox));

    let dialog_weak = dialog.downgrade();
    {
        let result = result.clone();
        let dw = dialog_weak.clone();
        btn_close.connect_clicked(move |_| {
            *result.borrow_mut() = ExportCompleteAction::Close;
            if let Some(d) = dw.upgrade() {
                d.close();
            }
        });
    }
    {
        let result = result.clone();
        let dw = dialog_weak.clone();
        btn_folder.connect_clicked(move |_| {
            *result.borrow_mut() = ExportCompleteAction::OpenFolder;
            if let Some(d) = dw.upgrade() {
                d.close();
            }
        });
    }
    {
        let result = result.clone();
        let dw = dialog_weak.clone();
        btn_doc.connect_clicked(move |_| {
            *result.borrow_mut() = ExportCompleteAction::OpenDocument;
            if let Some(d) = dw.upgrade() {
                d.close();
            }
        });
    }

    dialog.present();

    // ── Await close ───────────────────────────────────────────────────────
    struct DialogFuture {
        completed: Rc<RefCell<bool>>,
        waker: Rc<RefCell<Option<Waker>>>,
    }
    impl Future for DialogFuture {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if *self.completed.borrow() {
                Poll::Ready(())
            } else {
                *self.waker.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }

    let completed = Rc::new(RefCell::new(false));
    let waker: Rc<RefCell<Option<Waker>>> = Rc::new(RefCell::new(None));
    let completed_clone = completed.clone();
    let waker_clone = waker.clone();
    dialog.connect_close_request(move |_| {
        *completed_clone.borrow_mut() = true;
        if let Some(w) = waker_clone.borrow_mut().take() {
            w.wake();
        }
        glib::Propagation::Proceed
    });

    DialogFuture { completed, waker }.await;

    let action = *result.borrow();
    action
}

/// Helper: open a file or folder in the OS default handler.
///
/// * **Linux**: uses `gio::AppInfo::launch_default_for_uri` with a `file://`
///   URI (xdg-open / mimeapps).
/// * **Windows**: spawns `explorer.exe <path>` directly. The `file://` URI
///   path doesn't reliably open directories on Windows because directories
///   aren't registered as a URI handler — Explorer is the canonical tool
///   and also handles regular files via the default association.
pub fn open_path(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        // Use explorer.exe so both files and directories open in the
        // expected app (file → default association, directory → Explorer).
        match std::process::Command::new("explorer.exe").arg(path).spawn() {
            Ok(_) => return,
            Err(e) => {
                log::warn!(
                    "[ExportComplete] explorer.exe {:?} failed: {}; falling back to gio",
                    path,
                    e
                );
            }
        }
    }

    let uri = gtk4::glib::filename_to_uri(path, None);
    let uri = match uri {
        Ok(u) => u,
        Err(e) => {
            log::warn!(
                "[ExportComplete] filename_to_uri failed for {:?}: {}",
                path,
                e
            );
            return;
        }
    };
    if let Err(e) = gtk4::gio::AppInfo::launch_default_for_uri(
        uri.as_str(),
        None::<&gtk4::gio::AppLaunchContext>,
    ) {
        log::warn!(
            "[ExportComplete] launch_default_for_uri({}) failed: {}",
            uri,
            e
        );
    }
}

/// Convenience: returns the parent directory of `output_path`, falling back
/// to the path itself if it has no parent.
pub fn parent_dir(output_path: &Path) -> PathBuf {
    output_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| output_path.to_path_buf())
}
