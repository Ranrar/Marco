use gtk4::prelude::{PopoverExt, WidgetExt};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    static GLOBAL_ROOT_POPOVER_STATE: RefCell<Option<RootPopoverState>> = const { RefCell::new(None) };
}

/// Shared state for root-level popover interaction across menu and toolbar.
#[derive(Clone, Default)]
pub struct RootPopoverState {
    /// Whether any root-level popover tree is currently open.
    menu_open: Rc<Cell<bool>>,
    /// The currently active root popover.
    current_popover: Rc<RefCell<Option<gtk4::Popover>>>,
}

impl RootPopoverState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_root_open(&self) -> bool {
        self.menu_open.get()
    }

    pub fn set_current(&self, popover: Option<gtk4::Popover>) {
        *self.current_popover.borrow_mut() = popover;
    }

    pub fn current(&self) -> Option<gtk4::Popover> {
        self.current_popover.borrow().clone()
    }

    pub fn set_open(&self, open: bool) {
        self.menu_open.set(open);
    }

    /// Close the whole active root-level tree.
    /// With cascade-popdown enabled on root popovers, children are closed too.
    pub fn close_root_tree(&self) {
        if let Some(current) = self.current() {
            current.popdown();
        }
        self.set_open(false);
        self.set_current(None);
    }
}

pub fn set_global_root_popover_state(state: RootPopoverState) {
    GLOBAL_ROOT_POPOVER_STATE.with(|slot| {
        *slot.borrow_mut() = Some(state);
    });
}

pub fn is_toolbar_interaction_blocked() -> bool {
    GLOBAL_ROOT_POPOVER_STATE.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|s| s.is_root_open())
            .unwrap_or(false)
    })
}

/// Ensure popovers close with Escape and outside clicks consistently.
pub fn enforce_dismiss_behavior(popover: &gtk4::Popover) {
    popover.set_autohide(true);
    popover.set_can_focus(true);

    let escape_key = gtk4::EventControllerKey::new();
    let popover_for_escape = popover.clone();
    escape_key.connect_key_pressed(move |_, key, _, _| {
        if key == gtk4::gdk::Key::Escape {
            popover_for_escape.popdown();
            return gtk4::glib::Propagation::Stop;
        }
        gtk4::glib::Propagation::Proceed
    });
    popover.add_controller(escape_key);
}
