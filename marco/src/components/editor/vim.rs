//! Vim emulation for the editor.
//!
//! Wraps `sourceview5::VimIMContext`, the vim emulation shipped with
//! GtkSourceView. The context is plugged into the source view through an
//! `EventControllerKey` in the capture phase, so it sees every key before the
//! text view does. In Insert mode the context lets printable keys through, so
//! normal typing is unaffected.
//!
//! Vim mode is a global switch: every editor registers itself with [`attach`],
//! and [`set_vim_mode_globally`] toggles all of them at once. The current state
//! is also kept in [`is_vim_mode_enabled`] so editors created later start in the
//! right state.

use gtk4::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Commands typed after `:` that close the window or the application.
const QUIT_COMMANDS: &[&str] = &["q", "q!", "qa", "qa!", "wq", "wqa", "x", "xa"];

/// Commands typed after `:` that save the document before quitting.
const SAVE_AND_QUIT_COMMANDS: &[&str] = &["wq", "wqa", "x", "xa"];

/// CSS class of the vim status label shown under the editor.
const STATUS_CSS_CLASS: &str = "vim-status";

/// Whether vim mode is currently enabled for the application.
static VIM_MODE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Callback that switches one editor to the given vim state.
type ToggleCallback = Rc<dyn Fn(bool)>;

thread_local! {
    /// One toggle callback per attached editor (GTK runs on a single thread).
    static TOGGLE_CALLBACKS: RefCell<Vec<ToggleCallback>> = const { RefCell::new(Vec::new()) };
}

/// Returns `true` if vim mode is currently enabled.
pub fn is_vim_mode_enabled() -> bool {
    VIM_MODE_ENABLED.load(Ordering::Relaxed)
}

/// Enable or disable vim mode in every attached editor.
///
/// Called at startup from the saved settings and whenever the user flips the
/// switch in Preferences.
pub fn set_vim_mode_globally(enabled: bool) {
    VIM_MODE_ENABLED.store(enabled, Ordering::Relaxed);
    log::debug!("[vim] enabled = {enabled}");

    // Clone the callbacks out first so a callback can never re-enter the borrow.
    let callbacks: Vec<ToggleCallback> = TOGGLE_CALLBACKS.with(|c| c.borrow().clone());
    for callback in callbacks {
        callback(enabled);
    }
}

/// Attach vim emulation to `source_view`.
///
/// `container` is the box that holds the editor; a one line status label that
/// shows the pending command or the `:` / `/` command line is appended to it.
/// The editor starts in the state given by [`is_vim_mode_enabled`] and follows
/// every later call to [`set_vim_mode_globally`].
pub fn attach(source_view: &sourceview5::View, container: &gtk4::Box) {
    let im_context = sourceview5::VimIMContext::new();

    let key_controller = gtk4::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk4::PropagationPhase::Capture);
    key_controller.set_im_context(Some(&im_context));

    let status_label = gtk4::Label::new(None);
    status_label.add_css_class(STATUS_CSS_CLASS);
    status_label.set_xalign(0.0);
    status_label.set_margin_start(10);
    status_label.set_margin_end(10);
    status_label.set_selectable(false);
    status_label.set_visible(false);
    container.append(&status_label);

    connect_status_updates(&im_context, &status_label);
    connect_ex_commands(&im_context, source_view);

    let source_view = source_view.clone();
    // GTK asserts (and may crash) when a controller is removed from a widget it
    // is not attached to, so only add/remove it on a real state change.
    let attached = Cell::new(false);
    let apply = move |enabled: bool| {
        if enabled == attached.get() {
            return;
        }
        attached.set(enabled);

        if enabled {
            im_context.set_client_widget(Some(&source_view));
            source_view.add_controller(key_controller.clone());
        } else {
            source_view.remove_controller(&key_controller);
            im_context.set_client_widget(None::<&gtk4::Widget>);
            // Leave the block cursor of Normal mode behind.
            source_view.set_overwrite(false);
        }
        status_label.set_visible(enabled);
        status_label.set_text("");
    };

    if is_vim_mode_enabled() {
        apply(true);
    }
    TOGGLE_CALLBACKS.with(|c| c.borrow_mut().push(Rc::new(apply)));
}

/// Keep `label` in sync with the command line and the pending normal command.
fn connect_status_updates(im_context: &sourceview5::VimIMContext, label: &gtk4::Label) {
    let refresh = {
        let label = label.clone();
        move |im: &sourceview5::VimIMContext| {
            let text = _status_text(
                Some(im.command_bar_text().as_str()),
                Some(im.command_text().as_str()),
            );
            label.set_text(&text);
        }
    };

    im_context.connect_command_bar_text_notify(refresh.clone());
    im_context.connect_command_text_notify(refresh);
}

/// Pick what the status label shows: the command line wins over pending keys.
fn _status_text(command_bar: Option<&str>, pending: Option<&str>) -> String {
    match (command_bar, pending) {
        (Some(bar), _) if !bar.is_empty() => bar.to_string(),
        (_, Some(keys)) if !keys.is_empty() => keys.to_string(),
        _ => String::new(),
    }
}

/// Route `:w`, `:q` and friends to the application actions.
fn connect_ex_commands(im_context: &sourceview5::VimIMContext, source_view: &sourceview5::View) {
    // `:w` and `:w file`. The editor always saves through the "save" action,
    // which asks for a path when the document has none.
    im_context.connect_write(|_im, _view, _path| {
        _activate_app_action("save");
    });

    let source_view = source_view.clone();
    im_context.connect_execute_command(move |_im, command| {
        let command = command.trim();
        if !QUIT_COMMANDS.contains(&command) {
            return false;
        }
        if SAVE_AND_QUIT_COMMANDS.contains(&command) {
            _activate_app_action("save");
        }
        _quit(&source_view);
        true
    });
}

/// Close the window that holds `source_view`, which quits the application.
fn _quit(source_view: &sourceview5::View) {
    let Some(root) = source_view.root() else {
        return;
    };
    if let Ok(window) = root.downcast::<gtk4::Window>() {
        window.close();
    }
}

/// Activate an action of the running application by name.
fn _activate_app_action(name: &str) {
    let Some(app) = gtk4::gio::Application::default() else {
        log::warn!("[vim] no running application, cannot run action '{name}'");
        return;
    };
    app.activate_action(name, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_text_prefers_command_bar() {
        assert_eq!(_status_text(Some(":w"), Some("2d")), ":w");
    }

    #[test]
    fn status_text_falls_back_to_pending_keys() {
        assert_eq!(_status_text(Some(""), Some("2d")), "2d");
        assert_eq!(_status_text(None, Some("2d")), "2d");
    }

    #[test]
    fn status_text_empty_when_idle() {
        assert_eq!(_status_text(None, None), "");
        assert_eq!(_status_text(Some(""), Some("")), "");
    }
}
