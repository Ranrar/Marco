use crate::logic::menu_items::file::SaveChangesResult;
use anyhow::Result;
use gtk4::{
    glib, prelude::*, Align, Box, Button, Label, Orientation, Window,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};
use std::pin::Pin;
use std::future::Future;

/// Shows a "Save Changes?" confirmation dialog
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
/// ```
/// use crate::ui::dialogs::save::show_save_changes_dialog;
/// 
/// match show_save_changes_dialog(&window, "Untitled.md", "closing").await? {
///     SaveChangesResult::Save => save_document(),
///     SaveChangesResult::Discard => close_without_saving(),
///     SaveChangesResult::Cancel => return,
/// }
/// ```
pub async fn show_save_changes_dialog<W: IsA<Window>>(
    parent: &W,
    document_name: &str,
    action: &str,
) -> Result<SaveChangesResult> {
    // Get current theme mode from parent window
    // Cast to Widget to access has_css_class method
    let theme_class = if let Some(widget) = parent.dynamic_cast_ref::<gtk4::Widget>() {
        if widget.has_css_class("marco-theme-dark") {
            "marco-theme-dark"
        } else {
            "marco-theme-light"
        }
    } else {
        "marco-theme-light" // Default to light theme
    };
    
    // Create a Window instead of deprecated MessageDialog
    let dialog = Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(500)
        .default_height(180)
        .resizable(false)
        .build();
    
    // Apply CSS classes for theming
    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(theme_class);
    
    // Create custom titlebar matching marco's style
    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.add_css_class("marco-titlebar");
    headerbar.set_show_title_buttons(false);
    
    // Set title in headerbar
    let title_label = Label::new(Some("Save Changes?"));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    headerbar.set_title_widget(Some(&title_label));
    
    // Create custom close button with icon font
    let close_label = Label::new(None);
    close_label.set_markup("<span font_family='icomoon'>\u{39}</span>"); // \u{39} = marco-close icon
    close_label.set_valign(Align::Center);
    close_label.add_css_class("icon-font");
    
    let btn_close_titlebar = Button::new();
    btn_close_titlebar.set_child(Some(&close_label));
    btn_close_titlebar.set_tooltip_text(Some("Close"));
    btn_close_titlebar.set_valign(Align::Center);
    btn_close_titlebar.set_margin_start(1);
    btn_close_titlebar.set_margin_end(1);
    btn_close_titlebar.set_focusable(false);
    btn_close_titlebar.set_can_focus(false);
    btn_close_titlebar.set_has_frame(false);
    btn_close_titlebar.add_css_class("topright-btn");
    btn_close_titlebar.add_css_class("window-control-btn");
    
    // Shared state for dialog result
    let result = Rc::new(RefCell::new(SaveChangesResult::Cancel));
    
    // Wire up close button (Cancel action)
    let dialog_weak_for_close = dialog.downgrade();
    let result_for_close = result.clone();
    btn_close_titlebar.connect_clicked(move |_| {
        *result_for_close.borrow_mut() = SaveChangesResult::Cancel;
        if let Some(dialog) = dialog_weak_for_close.upgrade() {
            dialog.close();
        }
    });
    
    // Add close button to right side of headerbar
    headerbar.pack_end(&btn_close_titlebar);
    
    dialog.set_titlebar(Some(&headerbar));
    
    // Create main content container
    let vbox = Box::new(Orientation::Vertical, 0);
    vbox.add_css_class("marco-dialog-content");
    
    // Primary message
    let primary_message = Label::new(Some(&format!(
        "Save changes to \"{}\" before {}?",
        document_name, action
    )));
    primary_message.add_css_class("marco-dialog-title");
    primary_message.set_halign(Align::Start);
    primary_message.set_wrap(true);
    vbox.append(&primary_message);
    
    // Secondary message
    let secondary_message = Label::new(Some("Your changes will be lost if you don't save them."));
    secondary_message.add_css_class("marco-dialog-message");
    secondary_message.set_halign(Align::Start);
    secondary_message.set_wrap(true);
    vbox.append(&secondary_message);
    
    // Create button container (horizontal layout)
    let button_box = Box::new(Orientation::Horizontal, 8);
    button_box.add_css_class("marco-dialog-button-box");
    button_box.set_halign(Align::End);
    
    // Close without Saving button (destructive action)
    let btn_discard = Button::with_label("Close without Saving");
    btn_discard.add_css_class("marco-dialog-button");
    btn_discard.add_css_class("destructive-action");
    btn_discard.set_tooltip_text(Some("Discard changes and close"));
    button_box.append(&btn_discard);
    
    // Cancel button
    let btn_cancel = Button::with_label("Cancel");
    btn_cancel.add_css_class("marco-dialog-button");
    button_box.append(&btn_cancel);
    
    // Save As... button (suggested action)
    let btn_save = Button::with_label("Save As...");
    btn_save.add_css_class("marco-dialog-button");
    btn_save.add_css_class("suggested-action");
    btn_save.set_tooltip_text(Some("Save changes"));
    button_box.append(&btn_save);
    
    vbox.append(&button_box);
    
    dialog.set_child(Some(&vbox));
    
    // Handle button clicks
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
    
    // Show dialog and wait for it to close
    dialog.present();
    
    // Create a future that completes when the dialog closes
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
    
    let final_result = *result.borrow();
    log::info!("[SaveDialog] Save changes dialog result: {:?}", final_result);
    Ok(final_result)
}
