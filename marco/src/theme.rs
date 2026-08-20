thread_local! {}

// Removed duplicate save_appearance_settings; use Swanson.rs only
use dark_light::Mode as SystemMode;
#[cfg(target_os = "linux")]
use gtk4::Settings as GtkSettings;
use marco_shared::logic::swanson::{AppearanceSettings, SettingsManager};
use sourceview5::{StyleScheme, StyleSchemeManager};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// List all available HTML preview themes (*.css) in /themes/
pub fn list_preview_themes(theme_dir: &Path) -> Vec<String> {
    fs::read_dir(theme_dir)
        .map(|entries| {
            entries
                .filter_map(|e| {
                    let e = e.ok()?;
                    let path = e.path();
                    if path.extension().is_some_and(|ext| ext == "css") {
                        path.file_name()?.to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Value of `appearance.color_mode` meaning "follow the operating system".
pub const COLOR_MODE_SYSTEM: &str = "system";
/// Value of `appearance.color_mode` meaning "always light".
pub const COLOR_MODE_LIGHT: &str = "light";
/// Value of `appearance.color_mode` meaning "always dark".
pub const COLOR_MODE_DARK: &str = "dark";

/// The style scheme ID used for each of the two concrete modes. These are the
/// only values ever written to `appearance.editor_mode`.
pub const SCHEME_LIGHT: &str = "marco-light";
/// See [`SCHEME_LIGHT`].
pub const SCHEME_DARK: &str = "marco-dark";

/// The scheme ID for a light/dark choice.
pub fn scheme_id_for_dark(is_dark: bool) -> &'static str {
    if is_dark {
        SCHEME_DARK
    } else {
        SCHEME_LIGHT
    }
}

/// Ask the OS whether it is currently in dark mode.
///
/// `None` when the OS has no preference or cannot be asked — a desktop with no
/// XDG settings portal, for instance. Callers keep their current theme in that
/// case rather than forcing light, so a portal that is briefly unavailable
/// cannot flash the whole window to the wrong theme.
///
/// This is a blocking D-Bus round trip on Linux and a registry read on
/// Windows. Both are fast, but call it at startup and on change rather than
/// per frame.
pub fn detect_system_dark() -> Option<bool> {
    match dark_light::detect() {
        Ok(SystemMode::Dark) => Some(true),
        Ok(SystemMode::Light) => Some(false),
        Ok(SystemMode::Unspecified) => {
            log::debug!("[theme] OS reports no colour-scheme preference");
            None
        }
        Err(e) => {
            log::debug!("[theme] could not detect OS colour scheme: {e}");
            None
        }
    }
}

/// The colour mode stored in settings, defaulting to what `editor_mode` says
/// for a settings file written before the option existed.
pub fn color_mode_from_settings(appearance: Option<&AppearanceSettings>) -> String {
    if let Some(mode) = appearance.and_then(|a| a.color_mode.as_deref()) {
        let mode = mode.trim().to_lowercase();
        if mode == COLOR_MODE_SYSTEM || mode == COLOR_MODE_DARK || mode == COLOR_MODE_LIGHT {
            return mode;
        }
    }
    let is_dark = appearance
        .and_then(|a| a.editor_mode.as_deref())
        .is_some_and(|m| m.contains("dark"));
    if is_dark {
        COLOR_MODE_DARK
    } else {
        COLOR_MODE_LIGHT
    }
    .to_string()
}

/// The scheme ID a colour-mode preference resolves to right now.
///
/// `current_is_dark` is the theme in use, returned unchanged when the mode is
/// `"system"` and the OS declines to answer.
pub fn scheme_id_for_color_mode(color_mode: &str, current_is_dark: bool) -> &'static str {
    match color_mode.trim().to_lowercase().as_str() {
        COLOR_MODE_SYSTEM => scheme_id_for_dark(detect_system_dark().unwrap_or(current_is_dark)),
        COLOR_MODE_DARK => SCHEME_DARK,
        _ => SCHEME_LIGHT,
    }
}

/// Determines the effective color mode (light/dark) based on settings and system.
///
/// Accepts either a colour-mode value (`"light"` / `"dark"` / `"system"`) or a
/// style scheme ID (`"marco-dark"`), because callers hold one or the other.
pub fn resolve_effective_mode(color_mode: &str) -> String {
    let lowered = color_mode.trim().to_lowercase();
    match lowered.as_str() {
        COLOR_MODE_SYSTEM | "system default" => {
            if detect_system_dark().unwrap_or(false) {
                "dark".to_string()
            } else {
                "light".to_string()
            }
        }
        // Scheme IDs (`marco-dark`) and bare modes (`dark`) alike — every other
        // reader in the codebase tests for the substring, so match that.
        other if other.contains("dark") => "dark".to_string(),
        _ => "light".to_string(),
    }
}

/// Applies the HTML preview theme by loading the correct CSS file.
/// Returns the path to the CSS file to load.
pub fn select_preview_theme(settings: &AppearanceSettings, theme_dir: &Path) -> Option<PathBuf> {
    let theme_file = settings.preview_theme.as_deref().unwrap_or("standard.css");
    let path = theme_dir.join(theme_file);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Synchronizes the HTML preview theme context (e.g., sets data-theme attribute)
pub fn get_preview_data_theme(color_mode: &str) -> &'static str {
    match resolve_effective_mode(color_mode).as_str() {
        "dark" => "dark",
        "light" => "light",
        _ => "light",
    }
}

/// ThemeManager: manages current theme state and applies/synchronizes themes
pub struct ThemeManager {
    pub ui_theme_dir: PathBuf,
    pub preview_theme_dir: PathBuf,
    pub editor_theme_dir: PathBuf,
    style_scheme_manager: StyleSchemeManager,
    settings_manager: Arc<SettingsManager>,
}

impl ThemeManager {
    pub fn new(
        settings_manager: Arc<SettingsManager>,
        ui_theme_dir: PathBuf,
        preview_theme_dir: PathBuf,
        editor_theme_dir: PathBuf,
    ) -> Self {
        // Initialize StyleSchemeManager and add our custom themes directory
        let style_scheme_manager = StyleSchemeManager::new();

        // Add our custom editor themes directory to the search path
        let current_paths = style_scheme_manager.search_path();
        let editor_path_str = editor_theme_dir.to_string_lossy();
        let mut paths: Vec<&str> = current_paths.iter().map(|s| s.as_str()).collect();
        paths.push(&editor_path_str);
        style_scheme_manager.set_search_path(&paths);

        // Ensure editor_mode is set to a valid style scheme ID
        let _ = settings_manager.update_settings(|settings| {
            if let Some(appearance) = settings.appearance.as_mut() {
                if appearance.editor_mode.is_none() {
                    // Default to light theme if nothing is set
                    appearance.editor_mode = Some("marco-light".to_string());
                }
            }
        });

        ThemeManager {
            ui_theme_dir,
            preview_theme_dir,
            editor_theme_dir,
            style_scheme_manager,
            settings_manager: settings_manager.clone(),
        }
    }

    /// List available preview themes
    pub fn available_preview_themes(&self) -> Vec<String> {
        list_preview_themes(&self.preview_theme_dir)
    }

    /// List available editor style schemes
    pub fn available_editor_schemes(
        &self,
    ) -> Vec<marco_shared::logic::loaders::theme_loader::ThemeEntry> {
        marco_shared::logic::loaders::theme_loader::list_editor_style_schemes(
            &self.editor_theme_dir,
        )
    }

    /// Get the current editor style scheme ID
    pub fn current_editor_scheme_id(&self) -> String {
        let settings = self.settings_manager.get_settings();
        settings
            .appearance
            .as_ref()
            .and_then(|a| a.editor_mode.as_ref())
            .cloned()
            .unwrap_or_else(|| "marco-light".to_string())
    }

    /// Get the current editor style scheme object
    pub fn current_editor_scheme(&self) -> Option<StyleScheme> {
        let scheme_id = self.current_editor_scheme_id();
        self.style_scheme_manager.scheme(&scheme_id)
    }

    /// Get editor style scheme by ID
    pub fn get_editor_scheme(&self, scheme_id: &str) -> Option<StyleScheme> {
        self.style_scheme_manager.scheme(scheme_id)
    }

    /// Get the path to the current preview theme CSS file
    pub fn current_preview_theme_path(&self) -> Option<PathBuf> {
        let settings = self.settings_manager.get_settings();
        let appearance = settings.appearance.as_ref()?;
        select_preview_theme(appearance, &self.preview_theme_dir)
    }

    /// Get the data-theme value for the HTML preview ("light" or "dark")
    ///
    /// Reads the resolved scheme rather than the colour-mode preference, so
    /// `"system"` costs no OS round trip here: the scheme is already whatever
    /// the OS last reported.
    pub fn preview_data_theme(&self) -> &'static str {
        get_preview_data_theme(&self.current_editor_scheme_id())
    }

    /// The colour mode the user selected: `"light"`, `"dark"` or `"system"`.
    pub fn current_color_mode(&self) -> String {
        let settings = self.settings_manager.get_settings();
        color_mode_from_settings(settings.appearance.as_ref())
    }

    /// Whether the theme in use right now is the dark one.
    pub fn current_is_dark(&self) -> bool {
        self.current_editor_scheme_id().contains("dark")
    }

    /// Record the colour-mode preference and return the scheme ID it resolves
    /// to now. The caller applies that scheme via [`Self::set_editor_scheme`].
    pub fn set_color_mode(&mut self, color_mode: &str) -> &'static str {
        let scheme_id = scheme_id_for_color_mode(color_mode, self.current_is_dark());
        let stored = color_mode.trim().to_lowercase();
        if let Err(e) = self.settings_manager.update_settings(|settings| {
            let mut appearance = settings.appearance.clone().unwrap_or_default();
            appearance.color_mode = Some(stored.clone());
            settings.appearance = Some(appearance);
        }) {
            log::error!("[theme] failed to save color_mode: {e}");
        }
        scheme_id
    }

    /// Get preview theme mode from scheme ID
    pub fn preview_theme_mode_from_scheme(&self, scheme_id: &str) -> String {
        let theme_mode = if scheme_id.contains("dark") {
            "dark"
        } else {
            "light"
        };
        format!("theme-{}", theme_mode)
    }

    /// Change editor style scheme and update themes
    pub fn set_editor_scheme(&mut self, scheme_id: &str, _settings_path: &Path) {
        if let Err(e) = self.settings_manager.update_settings(|settings| {
            let mut appearance = settings.appearance.clone().unwrap_or_default();
            appearance.editor_mode = Some(scheme_id.to_string());
            settings.appearance = Some(appearance);
        }) {
            eprintln!("[ERROR] Failed to save editor_scheme: {}", e);
        }

        self.sync_platform_theme_preference();
    }

    /// Tell the platform's own theming which mode we are in.
    ///
    /// Separate from [`Self::set_editor_scheme`] because it also has to run at
    /// startup, where the scheme is not changing but GTK has not been told
    /// about it yet. Without that, launching in dark mode leaves GTK's native
    /// rendering — file choosers, native menus, anything not styled by Marco's
    /// own CSS — light until the first theme change.
    ///
    /// Windows has no equivalent knob: there the theme is carried entirely by
    /// the `marco-theme-dark` / `marco-theme-light` CSS classes on the window.
    pub fn sync_platform_theme_preference(&self) {
        #[cfg(target_os = "linux")]
        {
            let prefer_dark = self.current_is_dark();
            if let Some(settings_obj) = GtkSettings::default() {
                settings_obj.set_gtk_application_prefer_dark_theme(prefer_dark);
            }
        }
    }

    /// Change preview theme (filename)
    pub fn set_preview_theme(&mut self, theme: String, _settings_path: &Path) {
        if let Err(e) = self.settings_manager.update_settings(|settings| {
            let mut appearance = settings.appearance.clone().unwrap_or_default();
            appearance.preview_theme = Some(theme.clone());
            settings.appearance = Some(appearance);
        }) {
            eprintln!("[ERROR] Failed to save preview_theme: {}", e);
        }

        // HTML preview reload should be triggered by the UI layer
    }

    /// Get current settings from SettingsManager
    pub fn get_settings(&self) -> marco_shared::logic::swanson::Settings {
        self.settings_manager.get_settings()
    }
}

/// Keeps a background OS-theme watcher running; dropping it shuts the watcher
/// down.
///
/// Shutdown closes the channel, which ends the main-thread task immediately so
/// no further callbacks fire. The two background threads unwind more lazily:
/// the forwarding thread is parked inside `dark_light`'s blocking receiver and
/// only notices the closed channel when the OS next reports a change, at which
/// point it drops the `dark_light` watcher (closing the registry handle on
/// Windows, detaching the D-Bus signal thread on Linux). They hold no GTK
/// state and cost nothing while parked, so the delay is harmless — but it does
/// mean the threads can outlive this guard.
pub struct SystemColorModeWatcher {
    receiver: async_channel::Receiver<bool>,
}

impl Drop for SystemColorModeWatcher {
    fn drop(&mut self) {
        // Closes the channel for every clone, including the one the
        // main-thread task is awaiting.
        self.receiver.close();
    }
}

/// Call `on_change` on the GTK main thread whenever the OS switches between
/// light and dark, passing `true` for dark.
///
/// Returns `None` when the platform cannot report changes — a Linux desktop
/// with no XDG settings portal, most commonly. That is not an error worth
/// bothering the user about: "System default" still resolves correctly at
/// startup through [`detect_system_dark`], it just stops tracking live.
///
/// `dark_light` runs its own watcher thread on both platforms (a D-Bus signal
/// stream on Linux, `RegNotifyChangeKeyValue` on Windows) and hands changes
/// over a `std::sync::mpsc` channel, which cannot be awaited. A second thread
/// forwards them onto an async channel that `glib` can await, so the callback
/// lands on the main thread where touching widgets is legal.
pub fn watch_system_color_mode<F>(on_change: F) -> Option<SystemColorModeWatcher>
where
    F: Fn(bool) + 'static,
{
    let watcher = match dark_light::subscribe() {
        Ok(watcher) => watcher,
        Err(e) => {
            log::info!("[theme] OS colour-scheme changes will not be followed live: {e}");
            return None;
        }
    };

    let (sender, receiver) = async_channel::unbounded::<bool>();
    std::thread::Builder::new()
        .name("marco-theme-watch".to_string())
        .spawn(move || {
            for mode in watcher.iter() {
                let is_dark = match mode {
                    SystemMode::Dark => true,
                    SystemMode::Light => false,
                    // Do not guess when the OS withdraws its preference; the
                    // next real change will arrive on its own.
                    SystemMode::Unspecified => continue,
                };
                if sender.send_blocking(is_dark).is_err() {
                    break;
                }
            }
        })
        .ok()?;

    let receiver_for_task = receiver.clone();
    glib::spawn_future_local(async move {
        while let Ok(is_dark) = receiver_for_task.recv().await {
            log::info!(
                "[theme] OS switched to {} mode",
                if is_dark { "dark" } else { "light" }
            );
            on_change(is_dark);
        }
    });

    Some(SystemColorModeWatcher { receiver })
}

#[cfg(test)]
mod color_mode_tests {
    use super::*;

    fn appearance(color_mode: Option<&str>, editor_mode: Option<&str>) -> AppearanceSettings {
        AppearanceSettings {
            editor_mode: editor_mode.map(str::to_string),
            color_mode: color_mode.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn smoke_color_mode_falls_back_to_the_scheme_for_older_settings() {
        // No `color_mode` key: a settings file written before the option
        // existed must keep the theme it already had, not reset to light.
        let dark = appearance(None, Some("marco-dark"));
        assert_eq!(color_mode_from_settings(Some(&dark)), COLOR_MODE_DARK);
        let light = appearance(None, Some("marco-light"));
        assert_eq!(color_mode_from_settings(Some(&light)), COLOR_MODE_LIGHT);
        assert_eq!(color_mode_from_settings(None), COLOR_MODE_LIGHT);
    }

    #[test]
    fn smoke_stored_color_mode_wins_over_the_scheme() {
        let following = appearance(Some("system"), Some("marco-dark"));
        assert_eq!(
            color_mode_from_settings(Some(&following)),
            COLOR_MODE_SYSTEM
        );
    }

    #[test]
    fn smoke_unrecognised_color_mode_falls_back_to_the_scheme() {
        let garbage = appearance(Some("chartreuse"), Some("marco-dark"));
        assert_eq!(color_mode_from_settings(Some(&garbage)), COLOR_MODE_DARK);
    }

    #[test]
    fn smoke_explicit_modes_resolve_without_asking_the_os() {
        assert_eq!(
            scheme_id_for_color_mode(COLOR_MODE_DARK, false),
            SCHEME_DARK
        );
        assert_eq!(
            scheme_id_for_color_mode(COLOR_MODE_LIGHT, true),
            SCHEME_LIGHT
        );
    }

    #[test]
    fn smoke_scheme_ids_resolve_to_their_own_mode() {
        // The regression this guards: `marco-dark` used to fall through to
        // "light", so the preview rendered light against a dark editor.
        assert_eq!(resolve_effective_mode(SCHEME_DARK), "dark");
        assert_eq!(resolve_effective_mode(SCHEME_LIGHT), "light");
        assert_eq!(get_preview_data_theme(SCHEME_DARK), "dark");
        assert_eq!(get_preview_data_theme(SCHEME_LIGHT), "light");
        assert_eq!(get_preview_data_theme("dark"), "dark");
    }
}
