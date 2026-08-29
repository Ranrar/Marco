// marco_link - Finding and reaching the Marco editor from Polo
//
//! # Reaching Marco from Polo
//!
//! Marco is a sibling binary next to Polo. Finding it means looking beside our
//! own executable and then along `PATH`; reaching it means spawning it with the
//! document as an argument.
//!
//! Marco may not be installed at all — Polo is packaged and installed on its
//! own, so "open in Marco" has to report that state rather than silently fail.
//! [`availability`] answers that question, and the caller is expected to ask
//! before offering the action.

/// Find Marco's executable next to our own, then on `PATH`.
///
/// The installed name differs from the crate name on Linux — upstream the
/// editor binary is `marco`, but that collides with the MATE window manager on
/// Debian, so the package installs it as `markdowncomposer`. A development
/// build always produces `marco`. Try both, so this works from an installed
/// package and from `cargo run` alike.
fn marco_executable() -> Option<std::path::PathBuf> {
    let candidates = [
        marco_shared::paths::COMPOSER_EXE_NAME,
        marco_shared::paths::COMPOSER_DEV_EXE_NAME,
    ];

    let sibling_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf));

    if let Some(dir) = sibling_dir {
        if let Some(path) = candidates
            .iter()
            .map(|name| dir.join(name))
            .find(|p| p.is_file())
        {
            return Some(path);
        }
    }

    // Nothing alongside us — look along PATH under both names.
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .flat_map(|dir| candidates.iter().map(move |name| dir.join(name)))
        .find(|p| p.is_file())
}

/// Whether Marco can be reached, and if not, what to say about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marco {
    /// Marco is present and can be opened right now.
    Available,
    /// Marco is absent, and Polo has no way to get it: that is a packaging
    /// question we cannot answer from inside the app. The user installed Polo
    /// some way, and has to install Marco the same way.
    Missing,
}

/// Report whether Marco can be reached from here.
///
/// Cheap enough to call on every toolbar build and again on click, which is
/// what makes installing Marco while Polo is running take effect without a
/// restart.
pub fn availability() -> Marco {
    if marco_executable().is_some() {
        Marco::Available
    } else {
        Marco::Missing
    }
}

/// Ask Marco to open `file_path`.
///
/// Marco is a single-instance `GApplication`, so a second invocation forwards
/// the document to the running instance and exits rather than opening a second
/// editor.
///
/// The spawn returns as soon as the child process exists, not when it has read
/// the document — but the child holds the path itself, so a caller that closes
/// Polo on `Ok(())` cannot lose the handover.
pub fn open(file_path: &str) -> Result<(), String> {
    let command =
        marco_executable().ok_or_else(|| "Marco is not installed on this system".to_string())?;

    std::process::Command::new(&command)
        .arg(file_path)
        .spawn()
        .map_err(|e| format!("Failed to spawn Marco: {e}"))?;

    log::info!("Launched Marco: {} {}", command.display(), file_path);
    Ok(())
}
