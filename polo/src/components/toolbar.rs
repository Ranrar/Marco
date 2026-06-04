// Polo icon toolbar
//
//! # Toolbar Module
//!
//! Creates the icon-based toolbar displayed below the titlebar.
//!
//! ## Buttons (left to right)
//!
//! - **Open file** – opens the file picker dialog
//! - *separator*
//! - **Open in Editor** – opens the current file in Marco editor (disabled when
//!   no file is loaded)
//! - **TOC** – toggles the Table-of-Contents side panel
//! - *separator*
//! - **Light / Dark mode** – toggles between light and dark colour mode
//!
//! ## Icons
//!
//! All icons are inline SVG strings rendered via `rsvg` + `cairo`.  Hover and
//! active states are handled through `EventControllerMotion` / `GestureClick`.

use crate::components::dialog::show_open_file_dialog;
use crate::components::menu::{render_svg_texture, toggle_color_mode};
use crate::components::toc_panel::TocPanelHandle;
use crate::components::viewer::platform_webview::PlatformWebView;
use gtk4::{prelude::*, Align, Box as GtkBox, Button, Orientation, Picture, Separator};
use marco_shared::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
use marco_shared::logic::swanson::SettingsManager;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

// ── Inline SVG icons ──────────────────────────────────────────────────────

/// Open-file (folder-open) icon - Tabler Icons `icon-tabler-folder-open`
const SVG_OPEN_FILE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 19l2.757 -7.351a1 1 0 0 1 .936 -.649h12.307a1 1 0 0 1 .986 1.164l-.996 5.211a2 2 0 0 1 -1.964 1.625h-14.026a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2h4l3 3h7a2 2 0 0 1 2 2v2"/></svg>"#;

/// Print icon - Tabler Icons `icon-tabler-printer`
const SVG_PRINT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M17 17h2a2 2 0 0 0 2 -2v-4a2 2 0 0 0 -2 -2h-14a2 2 0 0 0 -2 2v4a2 2 0 0 0 2 2h2"/><path d="M17 9v-4a2 2 0 0 0 -2 -2h-6a2 2 0 0 0 -2 2v4"/><path d="M7 15a2 2 0 0 1 2 -2h6a2 2 0 0 1 2 2v4a2 2 0 0 1 -2 2h-6a2 2 0 0 1 -2 -2l0 -4"/></svg>"#;

/// Open-in-editor (pencil) icon - Tabler Icons `icon-tabler-pencil`
const SVG_OPEN_EDITOR: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M4 20h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4"/><path d="M13.5 6.5l4 4"/></svg>"#;

/// Table-of-contents (stack-2) icon - Tabler Icons `icon-tabler-stack-2`
const SVG_TOC: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 4l-8 4l8 4l8 -4l-8 -4"/><path d="M4 12l8 4l8 -4"/><path d="M4 16l8 4l8 -4"/></svg>"#;

/// Logical icon size in pixels (will be rendered at 2× for HiDPI)
const TOOLBAR_ICON_SIZE: f64 = 12.0;

// ── Public types ──────────────────────────────────────────────────────────

/// State returned from `create_polo_toolbar`.
pub struct PoloToolbarState {
    /// The toolbar widget to insert into the window layout.
    pub toolbar: GtkBox,
    /// The "Open in Editor" button; enable it when a file is loaded.
    pub open_editor_btn: Button,
}

// ── Public constructor ────────────────────────────────────────────────────

/// Build the Polo icon toolbar.
///
/// The returned `PoloToolbarState.open_editor_btn` should be passed to
/// `create_custom_titlebar` so that the File → Open action can enable it
/// when a file is successfully opened.
#[allow(clippy::type_complexity)]
pub fn create_polo_toolbar(
    window: &gtk4::ApplicationWindow,
    webview: PlatformWebView,
    settings_manager: Arc<SettingsManager>,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
    toc_handle: TocPanelHandle,
    toc_on_file_opened: Option<Rc<dyn Fn(&str) + 'static>>,
) -> PoloToolbarState {
    let toolbar = GtkBox::new(Orientation::Horizontal, 0);
    toolbar.add_css_class("polo-toolbar");
    toolbar.set_valign(Align::Center);

    // ── Open file button ───────────────────────────────────────────────
    // Created first; wired to click AFTER open_editor_btn is created.
    let open_btn = make_icon_btn(window, SVG_OPEN_FILE, "Open file", TOOLBAR_ICON_SIZE);

    // ── Separator ──────────────────────────────────────────────────────
    let sep1 = make_separator();

    // ── Open in Editor ────────────────────────────────────────────────
    let open_editor_btn = make_icon_btn(
        window,
        SVG_OPEN_EDITOR,
        "Open in Marco editor",
        TOOLBAR_ICON_SIZE,
    );
    let has_file = current_file_path
        .read()
        .ok()
        .and_then(|g| g.as_ref().cloned())
        .is_some();
    open_editor_btn.set_sensitive(has_file);

    {
        let win_weak = window.downgrade();
        let cfp = current_file_path.clone();
        open_editor_btn.connect_clicked(move |_| {
            if let Some(w) = win_weak.upgrade() {
                if let Ok(guard) = cfp.read() {
                    if let Some(ref path) = *guard {
                        crate::components::dialog::show_open_in_editor_dialog(&w, path);
                    }
                }
            }
        });
    }

    // ── TOC ────────────────────────────────────────────────────────────
    let toc_btn = make_icon_btn(
        window,
        SVG_TOC,
        "Toggle Table of Contents",
        TOOLBAR_ICON_SIZE,
    );
    {
        let th = toc_handle.clone();
        toc_btn.connect_clicked(move |_| th.toggle());
    }

    // ── Separator ──────────────────────────────────────────────────────
    let sep2 = make_separator();

    // ── Print ─────────────────────────────────────────────────────────
    let print_btn = make_icon_btn(window, SVG_PRINT, "Print (Ctrl+P)", TOOLBAR_ICON_SIZE);
    {
        let wv_print = webview.clone();
        let win_weak = window.downgrade();
        print_btn.connect_clicked(move |_| {
            let parent = win_weak.upgrade();
            wv_print.print(parent.as_ref().map(|w| w.upcast_ref()));
        });
    }

    // ── Separator ──────────────────────────────────────────────────────
    let sep3 = make_separator();

    // ── Light / Dark mode ─────────────────────────────────────────────
    let mode_btn = make_mode_btn(
        window,
        settings_manager.clone(),
        webview.clone(),
        current_file_path.clone(),
        asset_root,
    );

    // ── Wire Open-file click (needs open_editor_btn reference) ────────
    {
        let win_weak = window.downgrade();
        let wv = webview.clone();
        let sm = settings_manager.clone();
        let cfp = current_file_path.clone();
        let asset = asset_root.to_path_buf();
        let oe_btn = open_editor_btn.clone();
        let toc_cb = toc_on_file_opened;

        open_btn.connect_clicked(move |_| {
            if let Some(w) = win_weak.upgrade() {
                // Title label is owned by the titlebar; use a throwaway label
                // here since the toolbar open button is a secondary shortcut.
                // The proper title update happens via the menu's File → Open.
                let dummy_label = gtk4::Label::new(None);
                show_open_file_dialog(
                    &w,
                    wv.clone(),
                    sm.clone(),
                    cfp.clone(),
                    &oe_btn,
                    &dummy_label,
                    &asset,
                    toc_cb.clone(),
                );
            }
        });
    }

    // ── Assemble toolbar ───────────────────────────────────────────────
    toolbar.append(&open_btn);
    toolbar.append(&sep1);
    toolbar.append(&open_editor_btn);
    toolbar.append(&toc_btn);
    toolbar.append(&sep2);
    toolbar.append(&print_btn);
    toolbar.append(&sep3);
    toolbar.append(&mode_btn);

    PoloToolbarState {
        toolbar,
        open_editor_btn,
    }
}

// ── Mode toggle button ────────────────────────────────────────────────────

fn make_mode_btn(
    window: &gtk4::ApplicationWindow,
    settings_manager: Arc<SettingsManager>,
    webview: PlatformWebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
) -> Button {
    use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};

    // Determine initial mode
    let is_dark = {
        let s = settings_manager.get_settings();
        s.appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.contains("dark"))
            .unwrap_or(false)
    };

    let icon_color = if is_dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    };
    let icon_svg = window_icon_svg(if is_dark {
        WindowIcon::Sun
    } else {
        WindowIcon::Moon
    });
    let tooltip = if is_dark {
        "Switch to Light Mode"
    } else {
        "Switch to Dark Mode"
    };

    // Build mode button without state-flags tracking (SVG changes on toggle).
    let mode_btn = make_icon_btn_from_svg(window, icon_svg, tooltip, TOOLBAR_ICON_SIZE, false);

    // Override the initial color using the foreground palette color.
    if let Some(pic) = mode_btn.child().and_then(|c| c.downcast::<Picture>().ok()) {
        let t = render_svg_texture(icon_svg, icon_color, TOOLBAR_ICON_SIZE);
        pic.set_paintable(Some(&t));
    }

    // Retrieve the Picture child so we can update it on click
    let pic = mode_btn
        .child()
        .and_then(|c| c.downcast::<Picture>().ok())
        .expect("mode button has Picture child");

    let sm = settings_manager.clone();
    let wv = webview.clone();
    let cfp = current_file_path.clone();
    let asset = asset_root.to_path_buf();
    let window_clone = window.clone();
    let pic_clone = pic.clone();

    mode_btn.connect_clicked(move |_| {
        toggle_color_mode(
            &window_clone,
            sm.clone(),
            wv.clone(),
            cfp.clone(),
            &asset,
            Some((&pic_clone, TOOLBAR_ICON_SIZE)),
        );
        // Update tooltip to reflect new state
        // (toggle_color_mode already updated the picture)
    });

    mode_btn
}

// ── Private helpers ────────────────────────────────────────────────────────

/// Determine toolbar button color based on GTK state flags and current theme.
fn toolbar_color_for_flags(btn: &gtk4::Button, flags: gtk4::StateFlags) -> &'static str {
    use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
    let is_dark = btn
        .root()
        .and_then(|r| r.downcast::<gtk4::Window>().ok())
        .map(|w| w.has_css_class("marco-theme-dark"))
        .unwrap_or(false);
    if flags.contains(gtk4::StateFlags::ACTIVE) {
        if is_dark {
            DARK_PALETTE.control_icon_active
        } else {
            LIGHT_PALETTE.control_icon_active
        }
    } else if flags.contains(gtk4::StateFlags::PRELIGHT) {
        if is_dark {
            DARK_PALETTE.control_icon_hover
        } else {
            LIGHT_PALETTE.control_icon_hover
        }
    } else if is_dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    }
}

/// Create an icon button from an SVG string using Marco's state-flags approach.
///
/// Icon color updates automatically on hover/active/normal via
/// `connect_state_flags_changed` and on theme changes via `connect_map`.
/// Pass `track_theme = false` for the mode-toggle button, whose SVG is managed
/// separately by `toggle_color_mode`.
fn make_icon_btn(window: &gtk4::ApplicationWindow, svg: &str, tooltip: &str, size: f64) -> Button {
    make_icon_btn_from_svg(window, svg, tooltip, size, true)
}

fn make_icon_btn_from_svg(
    window: &gtk4::ApplicationWindow,
    svg: &str,
    tooltip: &str,
    size: f64,
    track_theme: bool,
) -> Button {
    use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};

    let is_dark = window.style_context().has_class("marco-theme-dark");
    let initial_color = if is_dark {
        DARK_PALETTE.control_icon
    } else {
        LIGHT_PALETTE.control_icon
    };

    let pic = Picture::new();
    let texture = render_svg_texture(svg, initial_color, size);
    pic.set_paintable(Some(&texture));
    pic.set_size_request(size as i32, size as i32);
    pic.set_can_shrink(false);
    pic.set_halign(Align::Center);
    pic.set_valign(Align::Center);

    let btn = Button::new();
    btn.set_child(Some(&pic));
    btn.set_tooltip_text(Some(tooltip));
    btn.set_valign(Align::Center);
    btn.set_focusable(false);
    btn.set_can_focus(false);
    btn.set_has_frame(false);
    btn.set_width_request((size + 2.0) as i32);
    btn.set_height_request((size + 2.0) as i32);
    btn.add_css_class("polo-toolbar-btn");

    let svg_owned = svg.to_string();

    // Recompute icon color whenever button state changes (hover / active / normal).
    // Guard with is_mapped() to avoid snapshotting before first allocation.
    if track_theme {
        let pic_state = pic.clone();
        let s = svg_owned.clone();
        let btn_ref = btn.clone();
        btn.connect_state_flags_changed(move |btn, _| {
            if btn.is_mapped() {
                let flags = btn.state_flags();
                let color = toolbar_color_for_flags(&btn_ref, flags);
                let t = render_svg_texture(&s, color, size);
                pic_state.set_paintable(Some(&t));
            }
        });

        // Re-render after map so the root window's theme class is available.
        let pic_map = pic.clone();
        let s2 = svg_owned.clone();
        let btn_ref2 = btn.clone();
        btn.connect_map(move |_| {
            let flags = btn_ref2.state_flags();
            let color = toolbar_color_for_flags(&btn_ref2, flags);
            let t = render_svg_texture(&s2, color, size);
            pic_map.set_paintable(Some(&t));
        });

        // Also sync after click activation.
        let pic_click = pic.clone();
        let s3 = svg_owned.clone();
        let btn_ref3 = btn.clone();
        btn.connect_clicked(move |_| {
            let flags = btn_ref3.state_flags();
            let color = toolbar_color_for_flags(&btn_ref3, flags);
            let t = render_svg_texture(&s3, color, size);
            pic_click.set_paintable(Some(&t));
        });
    }

    btn
}

/// Create a styled vertical separator for the toolbar.
fn make_separator() -> Separator {
    let sep = Separator::new(Orientation::Vertical);
    sep.add_css_class("polo-toolbar-separator");
    sep
}
