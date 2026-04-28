use gtk4::{glib, prelude::*, Align, Box, Button, Label, Orientation, Window};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// What the user decided in the open-local-file confirmation dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenLocalFileChoice {
    /// User cancelled — do nothing.
    Cancel,
    /// No unsaved changes; user confirmed to open.
    Open,
    /// User had unsaved changes and chose to save first, then open.
    SaveAndOpen,
    /// User had unsaved changes and chose to discard them, then open.
    DiscardAndOpen,
}

/// Show a styled confirmation dialog for opening a local Markdown file from a preview link.
///
/// Detects the current theme from the parent window's CSS classes and applies
/// `marco-dialog` / `marco-theme-*` styling automatically.
///
/// # Arguments
/// * `parent` — Parent window used for modality and theme detection.
/// * `filename` — Display name of the target file (shown in the dialog title).
/// * `has_unsaved` — Whether the current document has unsaved changes.
/// * `current_doc` — Display name of the current document (used only when `has_unsaved`).
///
/// # Returns
/// An [`OpenLocalFileChoice`] describing what the user decided.
pub async fn show_open_local_file_dialog<W: IsA<Window>>(
    parent: &W,
    filename: &str,
    has_unsaved: bool,
    current_doc: &str,
) -> OpenLocalFileChoice {
    // ====================================================================
    // Theme detection
    // ====================================================================

    let theme_class = if parent
        .dynamic_cast_ref::<gtk4::Widget>()
        .is_some_and(|w| w.has_css_class("marco-theme-dark"))
    {
        "marco-theme-dark"
    } else {
        "marco-theme-light"
    };

    // ====================================================================
    // Dialog window
    // ====================================================================

    let dialog = Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(360)
        .resizable(false)
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(theme_class);

    // ====================================================================
    // Custom titlebar
    // ====================================================================

    let titlebar = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        "Open File",
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );
    let btn_close_titlebar = titlebar
        .close_button
        .as_ref()
        .expect("open-file dialog requires a close button");
    dialog.set_titlebar(Some(&titlebar.headerbar));

    // ====================================================================
    // ESC key → Cancel
    // ====================================================================

    let key_ctrl = gtk4::EventControllerKey::new();
    dialog.add_controller(key_ctrl.clone());

    // ====================================================================
    // Shared result state
    // ====================================================================

    let result: Rc<RefCell<OpenLocalFileChoice>> =
        Rc::new(RefCell::new(OpenLocalFileChoice::Cancel));

    // ====================================================================
    // Content area
    // ====================================================================

    let vbox = Box::new(Orientation::Vertical, 0);
    vbox.add_css_class("marco-dialog-content");

    // Primary message
    let primary_text = format!("Open \"{}\" in the editor?", filename);
    let primary = Label::new(Some(&primary_text));
    primary.add_css_class("marco-dialog-title");
    primary.set_halign(Align::Start);
    primary.set_wrap(true);
    primary.set_xalign(0.0);
    primary.set_max_width_chars(45);
    vbox.append(&primary);

    // Secondary message — only when there are unsaved changes
    if has_unsaved {
        let secondary_text = format!(
            "\"{}\" has unsaved changes. Choose how to proceed:",
            current_doc
        );
        let secondary = Label::new(Some(&secondary_text));
        secondary.add_css_class("marco-dialog-message");
        secondary.set_halign(Align::Start);
        secondary.set_wrap(true);
        secondary.set_xalign(0.0);
        secondary.set_max_width_chars(45);
        vbox.append(&secondary);
    }

    // ====================================================================
    // Button row
    // ====================================================================

    let button_box = Box::new(Orientation::Horizontal, 8);
    button_box.add_css_class("marco-dialog-button-box");
    button_box.set_halign(Align::End);
    button_box.set_valign(Align::End);

    let dialog_weak = dialog.downgrade();

    if has_unsaved {
        // Discard & Open (destructive — red)
        let btn_discard = Button::with_label("Discard & Open");
        btn_discard.add_css_class("marco-btn");
        btn_discard.add_css_class("marco-btn-red");
        btn_discard.set_tooltip_text(Some("Discard unsaved changes and open the file"));
        button_box.append(&btn_discard);

        // Cancel (yellow)
        let btn_cancel = Button::with_label("Cancel");
        btn_cancel.add_css_class("marco-btn");
        btn_cancel.add_css_class("marco-btn-yellow");
        btn_cancel.set_tooltip_text(Some("Cancel and stay in the current document"));
        button_box.append(&btn_cancel);

        // Save & Open (primary — blue)
        let btn_save_open = Button::with_label("Save & Open");
        btn_save_open.add_css_class("marco-btn");
        btn_save_open.add_css_class("marco-btn-blue");
        btn_save_open.set_tooltip_text(Some("Save the current document, then open the file"));
        button_box.append(&btn_save_open);

        // Wire buttons
        let result_d = result.clone();
        let dw_d = dialog_weak.clone();
        btn_discard.connect_clicked(move |_| {
            *result_d.borrow_mut() = OpenLocalFileChoice::DiscardAndOpen;
            if let Some(d) = dw_d.upgrade() {
                d.close();
            }
        });

        let result_c = result.clone();
        let dw_c = dialog_weak.clone();
        btn_cancel.connect_clicked(move |_| {
            *result_c.borrow_mut() = OpenLocalFileChoice::Cancel;
            if let Some(d) = dw_c.upgrade() {
                d.close();
            }
        });

        let result_s = result.clone();
        let dw_s = dialog_weak.clone();
        btn_save_open.connect_clicked(move |_| {
            *result_s.borrow_mut() = OpenLocalFileChoice::SaveAndOpen;
            if let Some(d) = dw_s.upgrade() {
                d.close();
            }
        });
    } else {
        // Cancel (yellow)
        let btn_cancel = Button::with_label("Cancel");
        btn_cancel.add_css_class("marco-btn");
        btn_cancel.add_css_class("marco-btn-yellow");
        btn_cancel.set_tooltip_text(Some("Cancel and stay in the current document"));
        button_box.append(&btn_cancel);

        // Open (primary — blue)
        let btn_open = Button::with_label("Open");
        btn_open.add_css_class("marco-btn");
        btn_open.add_css_class("marco-btn-blue");
        btn_open.set_tooltip_text(Some("Open the file in the editor"));
        button_box.append(&btn_open);

        // Wire buttons
        let result_c = result.clone();
        let dw_c = dialog_weak.clone();
        btn_cancel.connect_clicked(move |_| {
            *result_c.borrow_mut() = OpenLocalFileChoice::Cancel;
            if let Some(d) = dw_c.upgrade() {
                d.close();
            }
        });

        let result_o = result.clone();
        let dw_o = dialog_weak.clone();
        btn_open.connect_clicked(move |_| {
            *result_o.borrow_mut() = OpenLocalFileChoice::Open;
            if let Some(d) = dw_o.upgrade() {
                d.close();
            }
        });
    }

    vbox.append(&button_box);
    dialog.set_child(Some(&vbox));

    // ====================================================================
    // Titlebar close button → Cancel
    // ====================================================================

    let result_tb = result.clone();
    let dw_tb = dialog_weak.clone();
    btn_close_titlebar.connect_clicked(move |_| {
        *result_tb.borrow_mut() = OpenLocalFileChoice::Cancel;
        if let Some(d) = dw_tb.upgrade() {
            d.close();
        }
    });

    // ====================================================================
    // ESC handler → Cancel
    // ====================================================================

    let result_esc = result.clone();
    let dw_esc = dialog_weak.clone();
    key_ctrl.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            *result_esc.borrow_mut() = OpenLocalFileChoice::Cancel;
            if let Some(d) = dw_esc.upgrade() {
                d.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });

    // ====================================================================
    // Show dialog
    // ====================================================================

    dialog.present();

    // ====================================================================
    // Async wait — DialogFuture pattern (same as save.rs)
    // ====================================================================

    struct DialogFuture {
        completed: Rc<RefCell<bool>>,
        waker: Rc<RefCell<Option<Waker>>>,
    }

    impl Future for DialogFuture {
        type Output = ();
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
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

    let completed_close = completed.clone();
    let waker_close = waker.clone();
    dialog.connect_close_request(move |_| {
        *completed_close.borrow_mut() = true;
        if let Some(w) = waker_close.borrow_mut().take() {
            w.wake();
        }
        glib::Propagation::Proceed
    });

    DialogFuture { completed, waker }.await;

    let choice = *result.borrow();
    choice
}
