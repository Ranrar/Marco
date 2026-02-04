use crate::logic::menu_items::file::SaveChangesResult;
use gtk4::{glib, prelude::*, Align, Box, Button, Label, Orientation, Window};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

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
    // Custom Titlebar
    // ========================================================================

    let headerbar = gtk4::HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.add_css_class("marco-titlebar");
    headerbar.set_show_title_buttons(false);

    // Title label
    let title_label = Label::new(Some("Save Changes?"));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    headerbar.set_title_widget(Some(&title_label));

    // Close button with SVG icon
    use crate::ui::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
    use core::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
    use gio;
    use gtk4::gdk;
    use rsvg::{CairoRenderer, Loader};

    fn render_svg_icon(icon: WindowIcon, color: &str, icon_size: f64) -> gdk::MemoryTexture {
        let svg = window_icon_svg(icon).replace("currentColor", color);
        let bytes = glib::Bytes::from_owned(svg.into_bytes());
        let stream = gio::MemoryInputStream::from_bytes(&bytes);

        let handle =
            match Loader::new().read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE) {
                Ok(h) => h,
                Err(e) => {
                    log::error!("load SVG handle: {}", e);
                    // Fallback tiny transparent texture
                    let bytes = glib::Bytes::from_owned(vec![0u8, 0u8, 0u8, 0u8]);
                    return gdk::MemoryTexture::new(
                        1,
                        1,
                        gdk::MemoryFormat::B8g8r8a8Premultiplied,
                        &bytes,
                        4,
                    );
                }
            };

        let display_scale = gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_then(|m| m.downcast::<gdk::Monitor>().ok())
            .map(|m| m.scale_factor() as f64)
            .unwrap_or(1.0);

        let render_scale = display_scale * 2.0;
        let render_size = (icon_size * render_scale) as i32;

        let mut surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size)
                .expect("create surface");
        {
            let cr = cairo::Context::new(&surface).expect("create context");
            cr.scale(render_scale, render_scale);

            let renderer = CairoRenderer::new(&handle);
            let viewport = cairo::Rectangle::new(0.0, 0.0, icon_size, icon_size);
            renderer
                .render_document(&cr, &viewport)
                .expect("render SVG");
        }

        let data = surface.data().expect("get surface data").to_vec();
        let bytes = glib::Bytes::from_owned(data);
        gdk::MemoryTexture::new(
            render_size,
            render_size,
            gdk::MemoryFormat::B8g8r8a8Premultiplied,
            &bytes,
            (render_size * 4) as usize,
        )
    }

    fn svg_icon_button(
        window: &Window,
        icon: WindowIcon,
        tooltip: &str,
        color: &str,
        icon_size: f64,
    ) -> Button {
        let pic = gtk4::Picture::new();
        let texture = render_svg_icon(icon, color, icon_size);
        pic.set_paintable(Some(&texture));
        pic.set_size_request(icon_size as i32, icon_size as i32);
        pic.set_can_shrink(false);
        pic.set_halign(gtk4::Align::Center);
        pic.set_valign(gtk4::Align::Center);

        let btn = Button::new();
        btn.set_child(Some(&pic));
        btn.set_tooltip_text(Some(tooltip));
        btn.set_valign(gtk4::Align::Center);
        btn.set_margin_start(1);
        btn.set_margin_end(1);
        btn.set_focusable(false);
        btn.set_can_focus(false);
        btn.set_has_frame(false);
        btn.add_css_class("topright-btn");
        btn.add_css_class("window-control-btn");
        btn.set_width_request((icon_size + 6.0) as i32);
        btn.set_height_request((icon_size + 6.0) as i32);

        // Add hover and click interactions
        {
            let pic_hover = pic.clone();
            let normal_color = color.to_string();
            let is_dark = window.has_css_class("marco-theme-dark");
            let hover_color = if is_dark {
                DARK_PALETTE.control_icon_hover.to_string()
            } else {
                LIGHT_PALETTE.control_icon_hover.to_string()
            };
            let active_color = if is_dark {
                DARK_PALETTE.control_icon_active.to_string()
            } else {
                LIGHT_PALETTE.control_icon_active.to_string()
            };

            let motion_controller = gtk4::EventControllerMotion::new();
            let icon_for_enter = icon;
            let hover_color_enter = hover_color.clone();
            motion_controller.connect_enter(move |_ctrl, _x, _y| {
                let texture = render_svg_icon(icon_for_enter, &hover_color_enter, icon_size);
                pic_hover.set_paintable(Some(&texture));
            });

            let pic_leave = pic.clone();
            let icon_for_leave = icon;
            let normal_color_leave = normal_color.clone();
            motion_controller.connect_leave(move |_ctrl| {
                let texture = render_svg_icon(icon_for_leave, &normal_color_leave, icon_size);
                pic_leave.set_paintable(Some(&texture));
            });
            btn.add_controller(motion_controller);

            let gesture = gtk4::GestureClick::new();
            let pic_pressed = pic.clone();
            let icon_for_pressed = icon;
            let active_color_pressed = active_color.clone();
            gesture.connect_pressed(move |_gesture, _n, _x, _y| {
                let texture = render_svg_icon(icon_for_pressed, &active_color_pressed, icon_size);
                pic_pressed.set_paintable(Some(&texture));
            });

            let pic_released = pic.clone();
            let icon_for_released = icon;
            gesture.connect_released(move |_gesture, _n, _x, _y| {
                let texture = render_svg_icon(icon_for_released, &hover_color, icon_size);
                pic_released.set_paintable(Some(&texture));
            });
            btn.add_controller(gesture);
        }

        btn
    }

    let icon_color: std::borrow::Cow<'static, str> = if dialog.has_css_class("marco-theme-dark") {
        std::borrow::Cow::from(DARK_PALETTE.control_icon)
    } else {
        std::borrow::Cow::from(LIGHT_PALETTE.control_icon)
    };

    let btn_close_titlebar = svg_icon_button(&dialog, WindowIcon::Close, "Close", &icon_color, 8.0);
    headerbar.pack_end(&btn_close_titlebar);
    dialog.set_titlebar(Some(&headerbar));

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
    let primary_message = Label::new(Some(&format!(
        "Save changes to \"{}\" before {}?",
        document_name, action
    )));
    primary_message.add_css_class("marco-dialog-title");
    primary_message.set_halign(Align::Start);
    primary_message.set_valign(Align::Start);
    primary_message.set_wrap(true);
    primary_message.set_xalign(0.0);
    primary_message.set_max_width_chars(45); // Constrain width to button area
    vbox.append(&primary_message);

    // Secondary message
    let secondary_message = Label::new(Some("Your changes will be lost if you don't save them."));
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
    let btn_discard = Button::with_label("Close without Saving");
    btn_discard.add_css_class("marco-dialog-button");
    btn_discard.add_css_class("destructive-action");
    btn_discard.set_tooltip_text(Some("Discard changes and close"));
    button_box.append(&btn_discard);

    // Cancel button (warning action)
    let btn_cancel = Button::with_label("Cancel");
    btn_cancel.add_css_class("marco-dialog-button");
    btn_cancel.add_css_class("warning-action");
    btn_cancel.set_tooltip_text(Some("Return to editing"));
    button_box.append(&btn_cancel);

    // Save button (suggested action - primary)
    let btn_save = Button::with_label("Save As...");
    btn_save.add_css_class("marco-dialog-button");
    btn_save.add_css_class("suggested-action");
    btn_save.set_tooltip_text(Some("Save changes"));
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
