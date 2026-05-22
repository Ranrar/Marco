// Menu and titlebar components
//
//! # Menu Module
//!
//! Creates and manages Polo's custom titlebar with text-only menu bar.
//!
//! ## Titlebar Layout
//!
//! ### Left Side
//! - **App Icon**: Polo favicon (16x16)
//! - **[File] menu button**: Popover with Open / Quit
//! - **[View] menu button**: Popover with themes, mode toggle, TOC toggle
//!
//! ### Center
//! - **Title Label**: Shows "Polo" or "Polo - filename.md"
//!
//! ### Right Side
//! - **Window Controls**: Minimize, Maximize/Restore, Close buttons
//!
//! ## Functions
//!
//! - **`create_custom_titlebar`**: Builds the complete titlebar

use crate::components::dialog::show_open_file_dialog;
use crate::components::toc_panel::TocPanelHandle;
use crate::components::utils::{apply_gtk_theme_preference, list_available_themes_from_path};
use crate::components::viewer::platform_webview::PlatformWebView;
use crate::components::viewer::{load_and_render_markdown, show_empty_state_with_theme};
use gtk4::{
    gdk, gio, prelude::*, Align, ApplicationWindow, Box as GtkBox, Button, EventControllerMotion,
    HeaderBar, Image, Label, Orientation, Picture, Popover, Separator, WindowHandle,
};
use marco_shared::logic::loaders::icon_loader::{window_icon_svg, WindowIcon};
use marco_shared::logic::swanson::SettingsManager;
use rsvg::{CairoRenderer, Loader};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::time::Duration;

// ── Shared hover-switch state for the menu bar ────────────────────────────
//
// Mirrors Marco's `HoverMenuSwitchState`: while any popover is open, hovering
// another menu button switches to it after a short delay.

#[derive(Clone)]
struct PoloMenuState {
    is_open: Rc<RefCell<bool>>,
    current: Rc<RefCell<Option<gtk4::Popover>>>,
    pending: Rc<RefCell<Option<gtk4::glib::SourceId>>>,
}

impl PoloMenuState {
    fn new() -> Self {
        Self {
            is_open: Rc::new(RefCell::new(false)),
            current: Rc::new(RefCell::new(None)),
            pending: Rc::new(RefCell::new(None)),
        }
    }

    fn menu_open(&self) -> bool {
        *self.is_open.borrow()
    }

    fn current(&self) -> Option<gtk4::Popover> {
        self.current.borrow().clone()
    }

    fn is_current(&self, p: &gtk4::Popover) -> bool {
        self.current().is_some_and(|cur| cur == *p)
    }

    /// Called from `popover.connect_closed` — clears state for this popover.
    fn on_closed(&self, p: &gtk4::Popover) {
        if self.is_current(p) {
            *self.is_open.borrow_mut() = false;
            *self.current.borrow_mut() = None;
        }
    }

    fn cancel_pending(&self) {
        if let Some(id) = self.pending.borrow_mut().take() {
            id.remove();
        }
    }

    /// Switch immediately: close the old popover and open the target.
    fn switch_to(&self, target: gtk4::Popover) {
        self.cancel_pending();
        let prev = self.current();
        *self.is_open.borrow_mut() = true;
        *self.current.borrow_mut() = Some(target.clone());
        if let Some(prev) = prev {
            if prev != target {
                prev.popdown();
            }
        }
        target.popup();
    }

    /// Toggle open/close; if a different popover is open, switch to target.
    fn toggle_or_open(&self, target: gtk4::Popover) {
        self.cancel_pending();
        if self.is_current(&target) && self.menu_open() {
            target.popdown();
        } else {
            self.switch_to(target);
        }
    }

    /// Schedule a hover-switch after 140 ms (matches Marco's HOVER_SWITCH_DELAY_MS).
    fn schedule_hover_switch(&self, target: gtk4::Popover) {
        self.cancel_pending();
        let state = self.clone();
        let id = gtk4::glib::timeout_add_local(Duration::from_millis(140), move || {
            let _ = state.pending.borrow_mut().take();
            if !state.menu_open() {
                return gtk4::glib::ControlFlow::Break;
            }
            if state.is_current(&target) {
                return gtk4::glib::ControlFlow::Break;
            }
            state.switch_to(target.clone());
            gtk4::glib::ControlFlow::Break
        });
        *self.pending.borrow_mut() = Some(id);
    }
}

#[allow(clippy::too_many_arguments)]
/// Create custom titlebar with text menu bar (File / View) and window controls.
///
/// Returns `(WindowHandle, Label)` where:
/// - `WindowHandle`: The titlebar handle to set on the window
/// - `Label`: The title label (update when the displayed file changes)
///
/// The `open_editor_btn` parameter is the toolbar's "Open in Editor" button;
/// it is enabled automatically when a file is successfully opened via the
/// File → Open menu item.
pub fn create_custom_titlebar(
    window: &ApplicationWindow,
    filename: &str,
    initial_theme: &str,
    settings_manager: Arc<SettingsManager>,
    webview: PlatformWebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
    toc_handle: Option<TocPanelHandle>,
    open_editor_btn: gtk4::Button,
) -> (WindowHandle, Label) {
    let handle = WindowHandle::new();

    let headerbar = HeaderBar::new();
    headerbar.add_css_class("titlebar");
    headerbar.add_css_class("polo-titlebar");
    headerbar.set_show_title_buttons(false);

    // ── App icon ──────────────────────────────────────────────────────────
    let icon_path = asset_root.join("icons/icon_64x64_polo.png");
    let icon = Image::from_file(&icon_path);
    icon.set_pixel_size(16);
    icon.set_halign(Align::Start);
    icon.set_margin_start(5);
    icon.set_margin_end(5);
    icon.set_valign(Align::Center);
    icon.set_tooltip_text(Some("Polo - Markdown Viewer"));
    headerbar.pack_start(&icon);

    // ── Title label (center) ──────────────────────────────────────────────
    let title_text = if filename == "Untitled" {
        "Polo".to_string()
    } else {
        format!("Polo - {}", filename)
    };
    let title_label = Label::new(Some(&title_text));
    title_label.set_valign(Align::Center);
    title_label.add_css_class("title-label");
    title_label.add_css_class("polo-title-label");
    headerbar.set_title_widget(Some(&title_label));

    // ── Menu bar ──────────────────────────────────────────────────────────
    let menu_bar = GtkBox::new(Orientation::Horizontal, 0);
    // Shared hover-switch state (mirrors Marco's HoverMenuSwitchState)
    let menu_state = PoloMenuState::new();

    // ── File menu ────────────────────────────────────────────────────────
    // Create the button FIRST so we can parent the popover to it.
    let file_btn = Button::with_label("File");
    file_btn.add_css_class("polo-menu-btn");
    file_btn.set_valign(Align::Center);
    file_btn.set_focusable(false);
    file_btn.set_has_frame(false);

    let file_popover = build_file_popover(
        window,
        webview.clone(),
        settings_manager.clone(),
        current_file_path.clone(),
        asset_root,
        toc_handle.clone(),
        open_editor_btn,
        title_label.clone(),
        initial_theme,
    );
    // Parent to the button (not the container) so the arrow points at the button
    file_popover.set_parent(&file_btn);
    // Unparent when the button is destroyed to avoid GTK finalization warnings.
    {
        let pop = file_popover.clone();
        file_btn.connect_destroy(move |_| pop.unparent());
    }

    // Track close events
    {
        let state = menu_state.clone();
        let pop = file_popover.clone();
        file_popover.connect_closed(move |_| state.on_closed(&pop));
    }
    // Hover-switch: entering this button while another menu is open switches to it
    {
        let state = menu_state.clone();
        let pop = file_popover.clone();
        let motion = EventControllerMotion::new();
        let state2 = state.clone();
        let pop2 = pop.clone();
        motion.connect_enter(move |_, _, _| {
            if state2.menu_open() && !state2.is_current(&pop2) {
                state2.schedule_hover_switch(pop2.clone());
            }
        });
        motion.connect_leave(move |_| state.cancel_pending());
        file_btn.add_controller(motion);
    }
    // Toggle: click opens; click on an already-open menu closes it
    {
        let state = menu_state.clone();
        let pop = file_popover.clone();
        file_btn.connect_clicked(move |_| state.toggle_or_open(pop.clone()));
    }
    menu_bar.append(&file_btn);

    // ── View menu ────────────────────────────────────────────────────────
    // Create the button FIRST so we can parent the popover to it.
    let view_btn = Button::with_label("View");
    view_btn.add_css_class("polo-menu-btn");
    view_btn.set_valign(Align::Center);
    view_btn.set_focusable(false);
    view_btn.set_has_frame(false);

    let view_popover = Popover::new();
    view_popover.add_css_class("polo-menu-popover");
    // Parent to the button — arrow points at the View button, matching Marco
    view_popover.set_parent(&view_btn);
    // Unparent when the button is destroyed to avoid GTK finalization warnings.
    {
        let pop = view_popover.clone();
        view_btn.connect_destroy(move |_| pop.unparent());
    }
    view_popover.set_position(gtk4::PositionType::Bottom);

    // Rebuild View popover content every time it opens so state is current
    {
        let view_popover_ref = view_popover.clone();
        let settings_clone = settings_manager.clone();
        let webview_clone = webview.clone();
        let cfp_clone = current_file_path.clone();
        let asset_root_buf = asset_root.to_path_buf();
        let toc_for_view = toc_handle.clone();
        let window_clone = window.clone();

        view_popover.connect_show(move |_| {
            let content = build_view_popover_content(
                &window_clone,
                settings_clone.clone(),
                webview_clone.clone(),
                cfp_clone.clone(),
                &asset_root_buf,
                toc_for_view.clone(),
                &view_popover_ref,
            );
            view_popover_ref.set_child(Some(&content));
        });
    }
    // Track close events
    {
        let state = menu_state.clone();
        let pop = view_popover.clone();
        view_popover.connect_closed(move |_| state.on_closed(&pop));
    }
    // Hover-switch
    {
        let state = menu_state.clone();
        let pop = view_popover.clone();
        let motion = EventControllerMotion::new();
        let state2 = state.clone();
        let pop2 = pop.clone();
        motion.connect_enter(move |_, _, _| {
            if state2.menu_open() && !state2.is_current(&pop2) {
                state2.schedule_hover_switch(pop2.clone());
            }
        });
        motion.connect_leave(move |_| state.cancel_pending());
        view_btn.add_controller(motion);
    }
    // Toggle
    {
        let state = menu_state.clone();
        let pop = view_popover.clone();
        view_btn.connect_clicked(move |_| state.toggle_or_open(pop.clone()));
    }
    menu_bar.append(&view_btn);

    headerbar.pack_start(&menu_bar);

    // ── Window controls (right) ───────────────────────────────────────────
    let (btn_min, btn_max_toggle, btn_close) = create_window_controls(window, &settings_manager);
    headerbar.pack_end(&btn_close);
    headerbar.pack_end(&btn_max_toggle);
    headerbar.pack_end(&btn_min);

    handle.set_child(Some(&headerbar));
    (handle, title_label)
}

// ── Helper ────────────────────────────────────────────────────────────────

/// Create a menu button whose label is left-aligned.
/// GTK `Button::with_label` centres text by default; replacing the child
/// with an explicit `Label` and `set_halign(Align::Start)` achieves proper
/// left alignment without relying on the unsupported `text-align` CSS property.
fn menu_btn(text: &str) -> Button {
    let btn = Button::new();
    let lbl = Label::new(Some(text));
    lbl.set_halign(Align::Start);
    btn.set_child(Some(&lbl));
    btn
}

// ── File popover ──────────────────────────────────────────────────────────
//
// Uses gio::Menu + PopoverMenu so that "Open Recent" is a native GTK4
// submenu (slides in as a panel, back-navigation included) — identical to
// how Marco renders its File menu.  Two win-scoped GIO actions handle file
// loading:
//   win.polo-open-file        — opens the file-chooser dialog
//   win.polo-open-recent      — opens a specific recent file (string param)
//   win.polo-clear-recent     — clears the recent files list
//   win.polo-quit             — closes the window

#[allow(clippy::too_many_arguments)]
fn build_file_popover(
    window: &ApplicationWindow,
    webview: PlatformWebView,
    settings_manager: Arc<SettingsManager>,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
    toc_handle: Option<TocPanelHandle>,
    open_editor_btn: gtk4::Button,
    title_label: Label,
    _initial_theme: &str,
) -> Popover {
    use gtk4::glib;

    // ── GIO menu model ────────────────────────────────────────────────────
    let file_menu = gio::Menu::new();
    let recent_menu = gio::Menu::new();

    // Section 1: Open + Open Recent submenu
    let open_section = gio::Menu::new();
    open_section.append(Some("Open..."), Some("win.polo-open-file"));
    let recent_item = gio::MenuItem::new(Some("Open Recent"), None);
    recent_item.set_submenu(Some(&recent_menu));
    open_section.append_item(&recent_item);
    file_menu.append_section(None, &open_section);

    // Section 2: Print
    let print_section = gio::Menu::new();
    print_section.append(Some("Print\u{2026}"), Some("win.polo-print"));
    file_menu.append_section(None, &print_section);

    // Section 3: Quit
    let quit_section = gio::Menu::new();
    quit_section.append(Some("Quit"), Some("win.polo-quit"));
    file_menu.append_section(None, &quit_section);

    // ── PopoverMenu widget ────────────────────────────────────────────────
    let popover = gtk4::PopoverMenu::from_model(Some(&file_menu));
    popover.add_css_class("polo-menu-popover");
    popover.set_position(gtk4::PositionType::Bottom);

    // ── Register window actions (once; skip if already registered) ────────

    // polo-open-file: opens the file-chooser dialog
    if window.lookup_action("polo-open-file").is_none() {
        let open_action = gio::SimpleAction::new("polo-open-file", None);
        let window_weak = window.downgrade();
        let wv = webview.clone();
        let sm = settings_manager.clone();
        let cfp = current_file_path.clone();
        let asset = asset_root.to_path_buf();
        let oe_btn = open_editor_btn.clone();
        let title = title_label.clone();
        #[allow(clippy::type_complexity)]
        let on_file_opened: Option<Rc<dyn Fn(&str) + 'static>> = toc_handle.clone().map(|h| {
            Rc::new(move |path: &str| {
                if let Ok(text) =
                    marco_shared::cache::cached::read_to_string(std::path::Path::new(path))
                {
                    h.update_from_text_async(text);
                }
            }) as Rc<dyn Fn(&str) + 'static>
        });
        open_action.connect_activate(move |_, _| {
            if let Some(w) = window_weak.upgrade() {
                show_open_file_dialog(
                    &w,
                    wv.clone(),
                    sm.clone(),
                    cfp.clone(),
                    &oe_btn,
                    &title,
                    &asset,
                    on_file_opened.clone(),
                );
            }
        });
        window.add_action(&open_action);
    }

    // polo-open-recent: opens a specific file by path (string variant param)
    if window.lookup_action("polo-open-recent").is_none() {
        let open_recent_action =
            gio::SimpleAction::new("polo-open-recent", Some(glib::VariantTy::STRING));
        let wv = webview.clone();
        let sm = settings_manager.clone();
        let cfp = current_file_path.clone();
        let asset = asset_root.to_path_buf();
        let oe_btn = open_editor_btn.clone();
        let title = title_label.clone();
        let toc = toc_handle.clone();
        let window_weak = window.downgrade();
        open_recent_action.connect_activate(move |_, param| {
            let path_str = match param.and_then(|v| v.str().map(|s| s.to_owned())) {
                Some(s) => s,
                None => return,
            };
            let path_owned = std::path::PathBuf::from(&path_str);
            let theme = sm
                .get_settings()
                .appearance
                .as_ref()
                .and_then(|a| a.preview_theme.clone())
                .unwrap_or_else(|| "github.css".to_string());
            load_and_render_markdown(&wv, &path_str, &theme, &sm, &asset);
            if let Ok(mut g) = cfp.write() {
                *g = Some(path_str.clone());
            }
            oe_btn.set_sensitive(true);
            oe_btn.set_tooltip_text(Some("Open this file in Marco editor"));
            if let Some(fname) = path_owned.file_name() {
                let t = format!("Polo - {}", fname.to_string_lossy());
                if let Some(w) = window_weak.upgrade() {
                    w.set_title(Some(&t));
                }
                title.set_text(&t);
            }
            if let Some(ref h) = toc {
                if let Ok(text) = marco_shared::cache::cached::read_to_string(&path_owned) {
                    h.update_from_text_async(text);
                }
            }
            let _ = sm.update_settings(|s| {
                if s.polo.is_none() {
                    s.polo = Some(marco_shared::logic::swanson::PoloSettings::default());
                }
                if let Some(ref mut polo) = s.polo {
                    polo.last_opened_file = Some(std::path::PathBuf::from(&path_str));
                }
                s.add_polo_recent_file(&path_str);
            });
        });
        window.add_action(&open_recent_action);
    }

    // polo-clear-recent: clears the recent files list
    if window.lookup_action("polo-clear-recent").is_none() {
        let clear_action = gio::SimpleAction::new("polo-clear-recent", None);
        let sm = settings_manager.clone();
        clear_action.connect_activate(move |_, _| {
            let _ = sm.update_settings(|s| s.clear_polo_recent_files());
        });
        window.add_action(&clear_action);
    }

    // polo-quit: closes the window
    if window.lookup_action("polo-quit").is_none() {
        let quit_action = gio::SimpleAction::new("polo-quit", None);
        let window_weak = window.downgrade();
        quit_action.connect_activate(move |_, _| {
            if let Some(w) = window_weak.upgrade() {
                w.close();
            }
        });
        window.add_action(&quit_action);
    }

    // polo-print: opens the native print dialog
    if window.lookup_action("polo-print").is_none() {
        let print_action = gio::SimpleAction::new("polo-print", None);
        let wv = webview.clone();
        let window_weak = window.downgrade();
        print_action.connect_activate(move |_, _| {
            let parent = window_weak.upgrade();
            wv.print(parent.as_ref().map(|w| w.upcast_ref()));
        });
        window.add_action(&print_action);
        // Register Ctrl+P accelerator
        if let Some(app) = window.application() {
            app.set_accels_for_action("win.polo-print", &["<Control>p"]);
        }
    }

    // ── Rebuild recent submenu each time the menu opens ───────────────────
    let recent_menu_ref = recent_menu.clone();
    let sm_for_show = settings_manager.clone();
    popover.connect_show(move |_| {
        // Clear the existing recent items
        while recent_menu_ref.n_items() > 0 {
            recent_menu_ref.remove(0);
        }

        let recent_files = sm_for_show.get_settings().get_polo_recent_files();
        if recent_files.is_empty() {
            // Non-actionable placeholder (no action target → item is insensitive)
            recent_menu_ref.append(Some("No recent files"), None);
        } else {
            for path in &recent_files {
                let display = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .replace('_', "__"); // escape GTK mnemonic underscores
                let item = gio::MenuItem::new(Some(&display), None);
                item.set_action_and_target_value(
                    Some("win.polo-open-recent"),
                    Some(&path.to_string_lossy().as_ref().to_variant()),
                );
                recent_menu_ref.append_item(&item);
            }
            // Separator section + Clear
            let clear_section = gio::Menu::new();
            clear_section.append(Some("Clear Recent Files"), Some("win.polo-clear-recent"));
            recent_menu_ref.append_section(None, &clear_section);
        }
    });

    popover.upcast::<Popover>()
}

// ── View popover content (rebuilt on each show) ────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_view_popover_content(
    window: &ApplicationWindow,
    settings_manager: Arc<SettingsManager>,
    webview: PlatformWebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
    toc_handle: Option<TocPanelHandle>,
    popover: &Popover,
) -> GtkBox {
    let vbox = GtkBox::new(Orientation::Vertical, 0);

    // ── Theme section header ──────────────────────────────────────────
    let themes_label = Label::new(Some("Theme"));
    themes_label.add_css_class("polo-menu-item");
    themes_label.set_halign(Align::Start);
    themes_label.set_sensitive(false);
    vbox.append(&themes_label);

    // Current theme from settings
    let current_theme = {
        let s = settings_manager.get_settings();
        s.appearance
            .as_ref()
            .and_then(|a| a.preview_theme.clone())
            .unwrap_or_else(|| "marco.css".to_string())
    };

    // List available themes
    let themes = list_available_themes_from_path(asset_root);
    for theme_name in &themes {
        let current_stem = current_theme.trim_end_matches(".css");
        let display = if current_stem == theme_name.as_str() {
            format!("✓  {}", theme_name)
        } else {
            format!("    {}", theme_name)
        };
        let theme_btn = menu_btn(&display);
        theme_btn.add_css_class("polo-theme-item");
        theme_btn.set_halign(Align::Fill);

        let popover_clone = popover.clone();
        let sm_clone = settings_manager.clone();
        let webview_clone = webview.clone();
        let cfp_clone = current_file_path.clone();
        let asset_root_buf = asset_root.to_path_buf();
        let theme_filename = format!("{}.css", theme_name);

        theme_btn.connect_clicked(move |_| {
            popover_clone.popdown();
            let theme_to_save = theme_filename.clone();
            let _ = sm_clone.update_settings(move |s| {
                if s.appearance.is_none() {
                    s.appearance =
                        Some(marco_shared::logic::swanson::AppearanceSettings::default());
                }
                if let Some(ref mut a) = s.appearance {
                    a.preview_theme = Some(theme_to_save.clone());
                }
            });
            if let Ok(path_guard) = cfp_clone.read() {
                if let Some(ref path) = *path_guard {
                    load_and_render_markdown(
                        &webview_clone,
                        path,
                        &theme_filename,
                        &sm_clone,
                        &asset_root_buf,
                    );
                } else {
                    show_empty_state_with_theme(&webview_clone, &sm_clone);
                }
            }
        });

        vbox.append(&theme_btn);
    }

    // Separator
    let sep1 = Separator::new(Orientation::Horizontal);
    sep1.add_css_class("polo-menu-separator");
    vbox.append(&sep1);

    // ── Mode toggle ────────────────────────────────────────────────────
    let current_mode = {
        let s = settings_manager.get_settings();
        let mode = s
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.as_str())
            .unwrap_or("marco-light");
        if mode.contains("dark") {
            "dark"
        } else {
            "light"
        }
    };
    let mode_label = if current_mode == "dark" {
        "Switch to Light Mode"
    } else {
        "Switch to Dark Mode"
    };
    let mode_btn = menu_btn(mode_label);
    mode_btn.add_css_class("polo-menu-item");
    mode_btn.set_halign(Align::Fill);

    {
        let popover_clone = popover.clone();
        let sm_clone = settings_manager.clone();
        let webview_clone = webview.clone();
        let cfp_clone = current_file_path.clone();
        let asset_root_buf = asset_root.to_path_buf();
        let window_clone = window.clone();

        mode_btn.connect_clicked(move |_| {
            popover_clone.popdown();
            toggle_color_mode(
                &window_clone,
                sm_clone.clone(),
                webview_clone.clone(),
                cfp_clone.clone(),
                &asset_root_buf,
                None,
            );
        });
    }
    vbox.append(&mode_btn);

    // Separator
    let sep2 = Separator::new(Orientation::Horizontal);
    sep2.add_css_class("polo-menu-separator");
    vbox.append(&sep2);

    // ── TOC toggle ─────────────────────────────────────────────────────
    if let Some(toc) = toc_handle {
        let toc_label = if toc.is_visible() {
            "Hide Table of Contents"
        } else {
            "Show Table of Contents"
        };
        let toc_btn = menu_btn(toc_label);
        toc_btn.add_css_class("polo-menu-item");
        toc_btn.set_halign(Align::Fill);

        let popover_clone = popover.clone();
        toc_btn.connect_clicked(move |_| {
            popover_clone.popdown();
            toc.toggle();
        });

        vbox.append(&toc_btn);
    }

    vbox
}

// ── Shared mode toggle logic ───────────────────────────────────────────────

/// Toggle between light and dark color mode.
///
/// Updates settings, applies CSS class to window, optionally updates a
/// Picture widget with the new Sun/Moon icon, and reloads the webview.
///
/// `mode_pic_to_update`: optional `(Picture, icon_logical_size)` to update
/// the mode toggle icon after the switch (used by the toolbar button).
pub(crate) fn toggle_color_mode(
    window: &ApplicationWindow,
    settings_manager: Arc<SettingsManager>,
    webview: PlatformWebView,
    current_file_path: Arc<RwLock<Option<String>>>,
    asset_root: &std::path::Path,
    mode_pic_to_update: Option<(&Picture, f64)>,
) {
    let new_mode = {
        let s = settings_manager.get_settings();
        let cur = s
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .map(|m| m.as_str())
            .unwrap_or("marco-light");
        if cur.contains("dark") {
            "marco-light"
        } else {
            "marco-dark"
        }
    };

    log::info!("Toggling color mode to: {}", new_mode);

    let new_mode_owned = new_mode.to_string();
    let _ = settings_manager.update_settings(move |s| {
        if s.appearance.is_none() {
            s.appearance = Some(marco_shared::logic::swanson::AppearanceSettings::default());
        }
        if let Some(ref mut a) = s.appearance {
            a.editor_mode = Some(new_mode_owned.clone());
        }
    });

    let is_dark = new_mode.contains("dark");

    apply_gtk_theme_preference(&settings_manager);

    let (old_class, new_class) = if is_dark {
        ("marco-theme-light", "marco-theme-dark")
    } else {
        ("marco-theme-dark", "marco-theme-light")
    };
    window.remove_css_class(old_class);
    // add_css_class fires the `css-classes` notify, which re-renders all
    // static toolbar icons (open, open-editor, TOC) to the new theme color.
    window.add_css_class(new_class);
    log::debug!("Switched CSS class from {} to {}", old_class, new_class);

    // Update mode button icon AFTER the class change so this render wins
    // over the notify-triggered re-render that fired above.
    if let Some((pic, icon_size)) = mode_pic_to_update {
        use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
        let icon_color = if is_dark {
            DARK_PALETTE.control_icon
        } else {
            LIGHT_PALETTE.control_icon
        };
        let icon = if is_dark {
            WindowIcon::Sun
        } else {
            WindowIcon::Moon
        };
        let texture = render_svg_texture(window_icon_svg(icon), icon_color, icon_size);
        pic.set_paintable(Some(&texture));
    }

    // Reload webview
    let asset_root_buf = asset_root.to_path_buf();
    if let Ok(path_guard) = current_file_path.read() {
        if let Some(ref path) = *path_guard {
            let theme = {
                let s = settings_manager.get_settings();
                s.appearance
                    .as_ref()
                    .and_then(|a| a.preview_theme.clone())
                    .unwrap_or_else(|| "marco.css".to_string())
            };
            load_and_render_markdown(&webview, path, &theme, &settings_manager, &asset_root_buf);
        } else {
            show_empty_state_with_theme(&webview, &settings_manager);
        }
    }
}

// ── SVG rendering helpers ─────────────────────────────────────────────────

/// Render an SVG string to a `gdk::MemoryTexture` at the given logical size.
///
/// Replaces `currentColor` with `color` before rendering.
pub(crate) fn render_svg_texture(svg: &str, color: &str, size: f64) -> gdk::MemoryTexture {
    let svg = svg.replace("currentColor", color);
    let bytes = gtk4::glib::Bytes::from_owned(svg.into_bytes());
    let stream = gio::MemoryInputStream::from_bytes(&bytes);

    let handle = Loader::new()
        .read_stream(&stream, None::<&gio::File>, gio::Cancellable::NONE)
        .expect("load SVG handle");

    let display_scale = gdk::Display::default()
        .and_then(|d| d.monitors().item(0))
        .and_then(|m| m.downcast::<gdk::Monitor>().ok())
        .map(|m| m.scale_factor() as f64)
        .unwrap_or(1.0);

    let render_scale = display_scale * 2.0;
    let render_size = (size * render_scale) as i32;

    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, render_size, render_size)
        .expect("create surface");
    {
        let cr = cairo::Context::new(&surface).expect("create context");
        cr.scale(render_scale, render_scale);
        let renderer = CairoRenderer::new(&handle);
        let viewport = cairo::Rectangle::new(0.0, 0.0, size, size);
        renderer
            .render_document(&cr, &viewport)
            .expect("render SVG");
    }

    let data = surface.data().expect("get surface data").to_vec();
    let bytes = gtk4::glib::Bytes::from_owned(data);
    gdk::MemoryTexture::new(
        render_size,
        render_size,
        gdk::MemoryFormat::B8g8r8a8Premultiplied,
        &bytes,
        (render_size * 4) as usize,
    )
}

// ── Window controls ────────────────────────────────────────────────────────

fn create_window_controls(
    window: &ApplicationWindow,
    _settings_manager: &Arc<SettingsManager>,
) -> (Button, Button, Button) {
    const ICON_SIZE: f64 = 8.0;

    fn make_svg_btn(
        window: &ApplicationWindow,
        icon: WindowIcon,
        tooltip: &str,
        icon_size: f64,
    ) -> Button {
        use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};

        let is_dark = window.style_context().has_class("marco-theme-dark");
        let normal_color = if is_dark {
            DARK_PALETTE.control_icon
        } else {
            LIGHT_PALETTE.control_icon
        };
        let hover_color = if is_dark {
            DARK_PALETTE.control_icon_hover
        } else {
            LIGHT_PALETTE.control_icon_hover
        };
        let active_color = if is_dark {
            DARK_PALETTE.control_icon_active
        } else {
            LIGHT_PALETTE.control_icon_active
        };

        let pic = Picture::new();
        let texture = render_svg_texture(window_icon_svg(icon), normal_color, icon_size);
        pic.set_paintable(Some(&texture));
        pic.set_size_request(icon_size as i32, icon_size as i32);
        pic.set_can_shrink(false);
        pic.set_halign(Align::Center);
        pic.set_valign(Align::Center);

        let btn = Button::new();
        btn.set_child(Some(&pic));
        btn.set_tooltip_text(Some(tooltip));
        btn.set_valign(Align::Center);
        btn.set_margin_start(1);
        btn.set_margin_end(1);
        btn.set_focusable(false);
        btn.set_can_focus(false);
        btn.set_has_frame(false);
        btn.set_width_request((icon_size + 6.0) as i32);
        btn.set_height_request((icon_size + 6.0) as i32);
        btn.add_css_class("topright-btn");
        btn.add_css_class("window-control-btn");

        {
            let motion = gtk4::EventControllerMotion::new();
            let pic_enter = pic.clone();
            let hov = hover_color.to_string();
            motion.connect_enter(move |_, _, _| {
                let t = render_svg_texture(window_icon_svg(icon), &hov, icon_size);
                pic_enter.set_paintable(Some(&t));
            });
            let pic_leave = pic.clone();
            let nor = normal_color.to_string();
            motion.connect_leave(move |_| {
                let t = render_svg_texture(window_icon_svg(icon), &nor, icon_size);
                pic_leave.set_paintable(Some(&t));
            });
            btn.add_controller(motion);

            let gesture = gtk4::GestureClick::new();
            let pic_press = pic.clone();
            let act = active_color.to_string();
            gesture.connect_pressed(move |_, _, _, _| {
                let t = render_svg_texture(window_icon_svg(icon), &act, icon_size);
                pic_press.set_paintable(Some(&t));
            });
            let pic_rel = pic.clone();
            let hov2 = hover_color.to_string();
            gesture.connect_released(move |_, _, _, _| {
                let t = render_svg_texture(window_icon_svg(icon), &hov2, icon_size);
                pic_rel.set_paintable(Some(&t));
            });
            btn.add_controller(gesture);
        }

        btn
    }

    let btn_min = make_svg_btn(window, WindowIcon::Minimize, "Minimize", ICON_SIZE);
    let btn_close = make_svg_btn(window, WindowIcon::Close, "Close", ICON_SIZE);

    // Maximize / restore toggle
    let is_dark = window.style_context().has_class("marco-theme-dark");
    let normal_color: &str = if is_dark {
        crate::components::css::constants::DARK_PALETTE.control_icon
    } else {
        crate::components::css::constants::LIGHT_PALETTE.control_icon
    };

    let max_pic = Picture::new();
    max_pic.set_size_request(ICON_SIZE as i32, ICON_SIZE as i32);
    max_pic.set_can_shrink(false);
    max_pic.set_halign(Align::Center);
    max_pic.set_valign(Align::Center);

    {
        let t = render_svg_texture(
            window_icon_svg(WindowIcon::Maximize),
            normal_color,
            ICON_SIZE,
        );
        max_pic.set_paintable(Some(&t));
    }

    let btn_max = Button::new();
    btn_max.set_child(Some(&max_pic));
    btn_max.set_tooltip_text(Some("Maximize"));
    btn_max.set_valign(Align::Center);
    btn_max.set_margin_start(1);
    btn_max.set_margin_end(1);
    btn_max.set_focusable(false);
    btn_max.set_can_focus(false);
    btn_max.set_has_frame(false);
    btn_max.set_width_request((ICON_SIZE + 6.0) as i32);
    btn_max.set_height_request((ICON_SIZE + 6.0) as i32);
    btn_max.add_css_class("topright-btn");
    btn_max.add_css_class("window-control-btn");

    // Update icon when window maximize state changes
    {
        let color = normal_color.to_string();
        let pic_ref = max_pic.clone();
        let btn_ref = btn_max.clone();
        window.connect_maximized_notify(move |w| {
            let icon = if w.is_maximized() {
                WindowIcon::Restore
            } else {
                WindowIcon::Maximize
            };
            let t = render_svg_texture(window_icon_svg(icon), &color, ICON_SIZE);
            pic_ref.set_paintable(Some(&t));
            btn_ref.set_tooltip_text(Some(if w.is_maximized() {
                "Restore"
            } else {
                "Maximize"
            }));
        });
    }

    // Hover / active states for maximize button
    {
        use crate::components::css::constants::{DARK_PALETTE, LIGHT_PALETTE};
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
        let nor = normal_color.to_string();

        let motion = gtk4::EventControllerMotion::new();
        let pic_e = max_pic.clone();
        let hov = hover_color.clone();
        motion.connect_enter(move |_, _, _| {
            let t = render_svg_texture(window_icon_svg(WindowIcon::Maximize), &hov, ICON_SIZE);
            pic_e.set_paintable(Some(&t));
        });
        let pic_l = max_pic.clone();
        motion.connect_leave(move |_| {
            let t = render_svg_texture(window_icon_svg(WindowIcon::Maximize), &nor, ICON_SIZE);
            pic_l.set_paintable(Some(&t));
        });
        btn_max.add_controller(motion);

        let gesture = gtk4::GestureClick::new();
        let pic_p = max_pic.clone();
        let act = active_color;
        gesture.connect_pressed(move |_, _, _, _| {
            let t = render_svg_texture(window_icon_svg(WindowIcon::Maximize), &act, ICON_SIZE);
            pic_p.set_paintable(Some(&t));
        });
        let pic_r = max_pic.clone();
        gesture.connect_released(move |_, _, _, _| {
            let t = render_svg_texture(
                window_icon_svg(WindowIcon::Maximize),
                &hover_color,
                ICON_SIZE,
            );
            pic_r.set_paintable(Some(&t));
        });
        btn_max.add_controller(gesture);
    }

    // Click actions
    {
        let w = window.clone();
        btn_min.connect_clicked(move |_| w.minimize());
    }
    {
        let w = window.clone();
        btn_max.connect_clicked(move |_| {
            if w.is_maximized() {
                w.unmaximize();
            } else {
                w.maximize();
            }
        });
    }
    {
        let w = window.clone();
        btn_close.connect_clicked(move |_| w.close());
    }

    (btn_min, btn_max, btn_close)
}
