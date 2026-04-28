//! "Exporting…" progress dialog.
//!
//! Modal, button-less dialog displayed while a long-running export operation
//! (PDF / HTML) is in progress.  Shows an indeterminate (pulsing) progress
//! bar and can only be closed by the user via the titlebar's `X` button or
//! programmatically by calling [`ExportingDialog::close`].
//!
//! The dialog uses the same custom titlebar / theme classes as the other
//! Marco dialogs (`marco-dialog`, `marco-theme-light` / `marco-theme-dark`).

use gtk4::{glib, prelude::*, Align, Box as GtkBox, Label, Orientation, ProgressBar, Window};

use crate::components::viewer::export_pipeline::{CancelToken, ExportPhase, ProgressReporter};

/// Handle to a live "Exporting…" dialog.
///
/// Holds the dialog [`Window`] together with its pulse-animation source so
/// callers can close the dialog and stop the animation in a single call to
/// [`Self::close`].
///
/// Use [`Self::reporter`] to get a [`ProgressReporter`] that updates the
/// phase label, and [`Self::cancel_token`] to get a [`CancelToken`] the
/// pipeline and the X/ESC handlers share.
///
/// The handle does **not** itself implement `Drop`-based cleanup — callers
/// are expected to invoke [`Self::close`] when their export operation
/// finishes (or fails).  This keeps closure semantics explicit and avoids
/// surprising teardown order interactions with GTK signal handlers.
pub struct ExportingDialog {
    window: Window,
    pulse_source: Option<glib::SourceId>,
    phase_label: Label,
    cancel_token: CancelToken,
}

/// [`ProgressReporter`] backed by the phase label inside an [`ExportingDialog`].
///
/// Obtained via [`ExportingDialog::reporter`].  Clones the `Label` handle
/// (cheap GTK ref-count) so it stays valid as long as the dialog window is
/// alive.
pub struct ExportDialogReporter {
    phase_label: Label,
}

impl ProgressReporter for ExportDialogReporter {
    fn set_phase(&self, phase: ExportPhase) {
        self.phase_label.set_text(phase.label());
    }
}

impl ExportingDialog {
    /// Returns a [`ProgressReporter`] that updates the dialog's phase label.
    pub fn reporter(&self) -> ExportDialogReporter {
        ExportDialogReporter {
            phase_label: self.phase_label.clone(),
        }
    }

    /// Returns a clone of the shared [`CancelToken`].
    ///
    /// Pass this to [`crate::components::viewer::export_pipeline::run_export`]
    /// so the pipeline checks for cancellation on every polling cycle.
    pub fn cancel_token(&self) -> CancelToken {
        self.cancel_token.clone()
    }

    /// Programmatically close the dialog and stop its pulse animation.
    pub fn close(mut self) {
        if let Some(src) = self.pulse_source.take() {
            src.remove();
        }
        self.window.close();
    }
}

/// Show the "Exporting…" progress dialog.
///
/// * `parent`  - parent window used for transient/modal positioning and
///   theme-class detection.
/// * `title`   - text shown in the custom titlebar (e.g. `"Exporting PDF…"`).
/// * `message` - body label shown above the progress bar
///   (e.g. `"Generating PDF, please wait…"`).
///
/// The returned [`ExportingDialog`] keeps the GTK [`Window`] alive; the
/// caller must invoke [`ExportingDialog::close`] to dismiss it.
///
/// The dialog has **no** action buttons — only the titlebar `X` button is
/// available so the user can cancel/dismiss the wait.  Pressing `Esc` also
/// closes the window (matches the rest of Marco's dialogs).
pub fn show_exporting_dialog<W: IsA<Window>>(
    parent: &W,
    title: &str,
    message: &str,
) -> ExportingDialog {
    // ── Cancel token shared between the X/ESC handler and run_export ──────
    let cancel_token = CancelToken::new();
    // ── Theme detection (mirrors save.rs / export.rs pattern) ─────────────
    let theme_class = if let Some(widget) = parent.dynamic_cast_ref::<gtk4::Widget>() {
        if widget.has_css_class("marco-theme-dark") {
            "marco-theme-dark"
        } else {
            "marco-theme-light"
        }
    } else {
        "marco-theme-light"
    };

    // ── Dialog window ─────────────────────────────────────────────────────
    let dialog = Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(360)
        .default_height(140)
        .resizable(false)
        .deletable(false) // titlebar X is provided by our custom headerbar
        .build();

    dialog.add_css_class("marco-dialog");
    dialog.add_css_class(theme_class);

    // ── Custom titlebar (X-only, matches other Marco dialogs) ─────────────
    let titlebar_controls = crate::ui::titlebar::create_custom_titlebar_with_buttons(
        &dialog,
        title,
        crate::ui::titlebar::TitlebarButtons {
            close: true,
            minimize: false,
            maximize: false,
        },
    );

    let btn_close_titlebar = titlebar_controls
        .close_button
        .as_ref()
        .expect("Exporting dialog requires a close button");

    dialog.set_titlebar(Some(&titlebar_controls.headerbar));

    // Titlebar X cancels the export and closes the dialog window.
    // The pipeline will detect the cancellation on its next poll cycle,
    // restore the live preview, and then main.rs calls ExportingDialog::close.
    let dialog_weak_for_close = dialog.downgrade();
    let cancel_for_close = cancel_token.clone();
    btn_close_titlebar.connect_clicked(move |_| {
        log::info!("[ExportingDialog] User clicked titlebar close");
        cancel_for_close.cancel();
        if let Some(d) = dialog_weak_for_close.upgrade() {
            d.close();
        }
    });

    // ESC also cancels (consistent with Marco dialogs)
    let key_controller = gtk4::EventControllerKey::new();
    let dialog_weak_for_esc = dialog.downgrade();
    let cancel_for_esc = cancel_token.clone();
    key_controller.connect_key_pressed(move |_c, key, _code, _state| {
        if key == gtk4::gdk::Key::Escape {
            log::info!("[ExportingDialog] User pressed ESC");
            cancel_for_esc.cancel();
            if let Some(d) = dialog_weak_for_esc.upgrade() {
                d.close();
            }
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    dialog.add_controller(key_controller);

    // ── Body content ──────────────────────────────────────────────────────
    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.add_css_class("marco-dialog-content");

    let primary = Label::new(Some(message));
    primary.add_css_class("marco-dialog-title");
    primary.set_halign(Align::Start);
    primary.set_valign(Align::Start);
    primary.set_wrap(true);
    primary.set_xalign(0.0);
    primary.set_max_width_chars(45);
    vbox.append(&primary);

    // Secondary phase label — updated by ExportDialogReporter::set_phase.
    let phase_label = Label::new(Some(ExportPhase::Preparing.label()));
    phase_label.add_css_class("marco-dialog-description");
    phase_label.set_halign(Align::Start);
    phase_label.set_xalign(0.0);
    vbox.append(&phase_label);

    let progress = ProgressBar::new();
    progress.set_show_text(false);
    progress.set_hexpand(true);
    progress.set_valign(Align::Center);
    progress.set_pulse_step(0.1);
    vbox.append(&progress);

    dialog.set_child(Some(&vbox));

    // ── Pulse animation (~100 ms ticks, indeterminate progress) ───────────
    let progress_weak = progress.downgrade();
    let pulse_source = glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if let Some(p) = progress_weak.upgrade() {
            p.pulse();
            glib::ControlFlow::Continue
        } else {
            glib::ControlFlow::Break
        }
    });

    dialog.present();

    ExportingDialog {
        window: dialog,
        pulse_source: Some(pulse_source),
        phase_label,
        cancel_token,
    }
}
