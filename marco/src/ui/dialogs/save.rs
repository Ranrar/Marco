use crate::components::language::DialogTranslations;
use crate::logic::menu_items::file::SaveChangesResult;
use gtk4::{glib, prelude::*, Align, Box, Button, Label, Orientation, Window};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

/// Shows a "Save changes?" confirmation dialog
///
/// # Arguments
/// * `parent` - Parent window for the dialog
/// * `document_name` - Name of the document with unsaved changes
/// * `action` - What the user is trying to do (e.g., "closing the document")
///
/// # Returns
/// * `Ok(SaveChangesResult::Save)` - User wants to save
/// * `Ok(SaveChangesResult::Discard)` - User wants to discard changes
/// * `Ok(SaveChangesResult::Cancel)` - User cancelled the operation
/// * `Err(anyhow::Error)` - Dialog failed to show
///
/// # Example
/// ```no_run
/// use crate::ui::dialogs::save::show_save_changes_dialog;
/// use crate::components::language::SimpleLocalizationManager;
///
/// let translations = SimpleLocalizationManager::new()?.translations();
/// match show_save_changes_dialog(
///     &window,
///     "Untitled.md",
///     "closing",
///     &translations.dialog,
///     None,
/// ).await? {
///     SaveChangesResult::Save => save_document(),
///     SaveChangesResult::Discard => close_without_saving(),
///     SaveChangesResult::Cancel => return,
/// }
/// ```
pub async fn show_save_changes_dialog<W: IsA<Window>>(
    parent: &W,
    document_name: &str,
    action: &str,
    translations: &DialogTranslations,
) -> Result<SaveChangesResult, std::boxed::Box<dyn std::error::Error>> {
    // ========================================================================
    // Dialog Window Setup
    // ========================================================================

    // Get current theme mode from parent window
    let theme_class = if let Some(widget) = parent.dynamic_cast_ref::<gtk4::Widget>() {
        if widget.has_css_class("marco-theme-dark") {
            "marco-theme-dark"
        } else {
            "marco-theme-light"
        }
    } else {
        "marco-theme-light" // Default to light theme
    };

    // Create dialog window
    let dialog = Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(380)
        .default_height(180)
        .resizable(false)
        .build();

    // Apply theme CSS classes
    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(theme_class);

    // ========================================================================
    // Custom Titlebar (shared component)
    // ========================================================================

    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        &translations.save_changes_title,
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let btn_close_titlebar = titlebar_controls
        .close_button
        .as_ref()
        .expect("Save Changes dialog requires a close button");

    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    // ========================================================================
    // Shared Result State
    // ========================================================================

    let result = Rc::new(RefCell::new(SaveChangesResult::Cancel));

    // ========================================================================
    // Event Handlers - Titlebar & Keyboard
    // ========================================================================

    // Titlebar close button (Cancel action)
    let dialog_weak_for_close = dialog.downgrade();
    let result_for_close = result.clone();
    btn_close_titlebar.connect_clicked(move |_| {
        log::info!("[SaveDialog] User clicked titlebar close (cancel)");
        *result_for_close.borrow_mut() = SaveChangesResult::Cancel;
        if let Some(dialog) = dialog_weak_for_close.upgrade() {
            dialog.close();
        }
    });

    // ESC key handler (Cancel action)
    let key_controller = gtk4::EventControllerKey::new();
    let dialog_weak_for_esc = dialog.downgrade();
    let result_for_esc = result.clone();
    key_controller.connect_key_pressed(move |_controller, key, _code, _state| {
        if key == gtk4::gdk::Key::Escape {
            log::info!("[SaveDialog] User pressed ESC (cancel)");
            *result_for_esc.borrow_mut() = SaveChangesResult::Cancel;
            if let Some(dialog) = dialog_weak_for_esc.upgrade() {
                dialog.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    dialog.add_controller(key_controller);

    // ========================================================================
    // Create main content container with structured layout
    // ========================================================================

    let vbox = Box::new(Orientation::Vertical, 0);
    vbox.add_css_class("marco-dialog-content");

    // ------------------------------------------------------------------------
    // Message Section
    // ------------------------------------------------------------------------

    // Primary message
    let primary_text = translations
        .save_changes_prompt
        .replace("{document}", document_name)
        .replace("{action}", action);
    let primary_message = Label::new(Some(&primary_text));
    primary_message.add_css_class("marco-dialog-title");
    primary_message.set_halign(Align::Start);
    primary_message.set_valign(Align::Start);
    primary_message.set_wrap(true);
    primary_message.set_xalign(0.0);
    primary_message.set_max_width_chars(45); // Constrain width to button area
    vbox.append(&primary_message);

    // Secondary message
    let secondary_message = Label::new(Some(&translations.save_changes_secondary));
    secondary_message.add_css_class("marco-dialog-message");
    secondary_message.set_halign(Align::Start);
    secondary_message.set_valign(Align::Start);
    secondary_message.set_wrap(true);
    secondary_message.set_xalign(0.0);
    secondary_message.set_max_width_chars(45); // Constrain width to button area
    vbox.append(&secondary_message);

    // ------------------------------------------------------------------------
    // Button Section (left-aligned to match text)
    // ------------------------------------------------------------------------

    let button_box = Box::new(Orientation::Horizontal, 8);
    button_box.add_css_class("marco-dialog-button-box");
    button_box.set_halign(Align::Start);
    button_box.set_valign(Align::End);

    // Discard button (destructive action)
    let btn_discard = Button::with_label(&translations.save_without_saving);
    btn_discard.add_css_class("marco-btn");
    btn_discard.add_css_class("marco-btn-red");
    btn_discard.set_tooltip_text(Some(&translations.discard_tooltip));
    button_box.append(&btn_discard);

    // Cancel button (warning action)
    let btn_cancel = Button::with_label(&translations.cancel_button);
    btn_cancel.add_css_class("marco-btn");
    btn_cancel.add_css_class("marco-btn-yellow");
    btn_cancel.set_tooltip_text(Some(&translations.cancel_tooltip));
    button_box.append(&btn_cancel);

    // Save button (suggested action - primary)
    let btn_save = Button::with_label(&translations.save_as_button);
    btn_save.add_css_class("marco-btn");
    btn_save.add_css_class("marco-btn-blue");
    btn_save.set_tooltip_text(Some(&translations.save_tooltip));
    button_box.append(&btn_save);

    vbox.append(&button_box);

    dialog.set_child(Some(&vbox));

    // ========================================================================
    // Event Handlers - Button Actions
    // ========================================================================

    let dialog_weak = dialog.downgrade();

    // Discard button - close without saving
    let result_for_discard = result.clone();
    let dialog_weak_for_discard = dialog_weak.clone();
    btn_discard.connect_clicked(move |_| {
        log::info!("[SaveDialog] User chose to discard changes");
        *result_for_discard.borrow_mut() = SaveChangesResult::Discard;
        if let Some(dialog) = dialog_weak_for_discard.upgrade() {
            dialog.close();
        }
    });

    // Cancel button - cancel operation
    let result_for_cancel = result.clone();
    let dialog_weak_for_cancel = dialog_weak.clone();
    btn_cancel.connect_clicked(move |_| {
        log::info!("[SaveDialog] User cancelled");
        *result_for_cancel.borrow_mut() = SaveChangesResult::Cancel;
        if let Some(dialog) = dialog_weak_for_cancel.upgrade() {
            dialog.close();
        }
    });

    // Save button - save changes
    let result_for_save = result.clone();
    let dialog_weak_for_save = dialog_weak.clone();
    btn_save.connect_clicked(move |_| {
        log::info!("[SaveDialog] User chose to save");
        *result_for_save.borrow_mut() = SaveChangesResult::Save;
        if let Some(dialog) = dialog_weak_for_save.upgrade() {
            dialog.close();
        }
    });

    // ========================================================================
    // Show Dialog and Wait for Result
    // ========================================================================

    dialog.present();

    // ========================================================================
    // Async Future - Wait for Dialog to Close
    // ========================================================================

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
        if let Some(waker) = waker_clone.borrow_mut().take() {
            waker.wake();
        }
        glib::Propagation::Proceed
    });

    // Wait for the dialog to close
    DialogFuture { completed, waker }.await;

    // Return the result
    let final_result = *result.borrow();
    log::info!("[SaveDialog] Dialog closed with result: {:?}", final_result);
    Ok(final_result)
}
