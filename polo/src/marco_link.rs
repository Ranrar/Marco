// marco_link - Finding and reaching the Marco editor from Polo
//
//! # Reaching Marco from Polo
//!
//! Polo's "open in Marco" action has to work in two very different worlds, and
//! this module is the seam between them.
//!
//! - **Off Flatpak** (the `.deb`, Windows, `cargo run`) Marco is a sibling
//!   binary. Finding it means looking next to our own executable and then on
//!   `PATH`; reaching it means spawning it.
//! - **Under Flatpak** Marco is a *separate application* in its own sandbox.
//!   There is no binary to spawn. Finding it means asking the session bus
//!   whether its name is activatable; reaching it means calling
//!   `org.freedesktop.Application.Open`, which `GApplication` exports for free.
//!
//! The second world also introduces a state the first one never had: **Marco
//! may simply not be installed.** Polo ships as its own Flatpak and can be
//! installed alone, so the action must stop being something that silently
//! fails. [`availability`] reports that state and [`install`] offers the way
//! out of it, sending the user to Marco's store page.
//!
//! Two alternatives were rejected for *launching*. `flatpak-spawn --host`
//! requires `--talk-name=org.freedesktop.Flatpak`, which is effectively host
//! command execution and which Flathub reviewers push back on. The OpenURI
//! portal (`gio::AppInfo::launch_default_for_uri`) opens the user's *default*
//! Markdown handler -- which may well be Polo itself, making it circular. The
//! portal is still the right call for arbitrary external links, and [`install`]
//! uses it for the store page.

use gtk4::prelude::*;
use gtk4::{gio, glib};

/// Marco's application ID, and the bus name it registers under.
///
/// Must match `APP_ID` in `marco/src/main.rs`; `GApplication` derives the bus
/// name from it. Under Flatpak it is also the Flatpak ref, which is what makes
/// it activatable.
const MARCO_APP_ID: &str = "io.github.ranrar.Marco";

/// The `org.freedesktop.Application` object path, which `GApplication` derives
/// from the application ID by replacing `.` with `/`.
const MARCO_OBJECT_PATH: &str = "/io/github/ranrar/Marco";

/// Preferred install URI. GNOME Software and KDE Discover both claim the
/// `appstream:` scheme and land directly on the app's page.
const MARCO_APPSTREAM_URI: &str = "appstream://io.github.ranrar.Marco";

/// Fallback for when nothing claims `appstream:` — the browser can always open
/// this.
const MARCO_FLATHUB_URL: &str = "https://flathub.org/apps/io.github.ranrar.Marco";

/// How long to wait for the bus daemon to list its activatable names.
///
/// This is a local round trip to the bus (or, inside Flatpak, to
/// `xdg-dbus-proxy`), so it answers in well under a millisecond in practice.
/// The cap only exists so a wedged bus cannot freeze the UI: [`availability`]
/// is called from the main thread, where a stall would be a frozen window.
const LIST_NAMES_TIMEOUT_MS: i32 = 500;

/// How long to wait for Marco to accept an `Open` call.
///
/// Generous on purpose: if Marco is not already running this call *starts* it,
/// and a cold GTK + WebKit start is not instant. Nothing blocks on this — the
/// call is asynchronous — so a long ceiling costs nothing.
const OPEN_TIMEOUT_MS: i32 = 30_000;

/// Whether Marco can be reached, and if not, what to offer instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marco {
    /// Marco is present and can be opened right now.
    Available,
    /// Marco is absent but installable from here — offer [`install`].
    ///
    /// Only ever returned under Flatpak, where we know both that Marco is
    /// published on Flathub and that a store front-end exists to install it.
    Installable,
    /// Marco is absent and Polo has no way to get it. Off Flatpak this means a
    /// packaging question we cannot answer from inside the app: the user
    /// installed Polo some other way and has to install Marco the same way.
    Missing,
}

/// Whether this process is running inside a Flatpak sandbox.
///
/// Same test as `crate::app_id` uses to pick the application ID — Flatpak
/// always mounts `/.flatpak-info` inside the sandbox and never outside it.
fn in_flatpak() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new("/.flatpak-info").exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Report whether Marco can be reached from here.
///
/// Cheap enough to call on every toolbar build and again on click, which is
/// what makes installing Marco while Polo is running take effect without a
/// restart.
pub fn availability() -> Marco {
    if in_flatpak() {
        if marco_on_bus() {
            Marco::Available
        } else {
            // We are a Flatpak, so Marco is a Flatpak too, so it is on Flathub.
            Marco::Installable
        }
    } else if marco_executable().is_some() {
        Marco::Available
    } else {
        Marco::Missing
    }
}

/// Ask Marco to open `file_path`, calling `on_done` with the outcome.
///
/// The callback exists because the Flatpak path can take seconds: calling
/// `Open` on a name that is activatable but not running *starts* Marco, and
/// waiting for that on the main thread would freeze Polo's window. So the call
/// is asynchronous and `on_done` runs later, on the main loop.
///
/// That also fixes an ordering hazard. "DualView" closes Polo once Marco has
/// the document; doing that synchronously after firing the call could tear down
/// the bus connection with the message still buffered, and the document would
/// never arrive. Callers should close Polo *from* `on_done`, not beside it.
///
/// Off Flatpak the spawn is immediate and `on_done` is called before this
/// function returns.
pub fn open<F>(file_path: &str, on_done: F)
where
    F: FnOnce(Result<(), String>) + 'static,
{
    if in_flatpak() {
        open_over_dbus(file_path, on_done);
    } else {
        on_done(spawn_sibling(file_path));
    }
}

/// Send the user somewhere they can install Marco.
///
/// Tries the `appstream:` URI first so a store front-end can open Marco's page
/// directly, and falls back to the Flathub web page if nothing handles it.
/// Both go through `GAppInfo`, which inside a sandbox routes to the OpenURI
/// portal automatically — no extra permission needed.
pub fn install() -> Result<(), String> {
    let launch =
        |uri: &str| gio::AppInfo::launch_default_for_uri(uri, None::<&gio::AppLaunchContext>);

    match launch(MARCO_APPSTREAM_URI) {
        Ok(()) => {
            log::info!("Opened Marco's store page via {MARCO_APPSTREAM_URI}");
            Ok(())
        }
        Err(e) => {
            log::debug!("No handler for {MARCO_APPSTREAM_URI} ({e}); falling back to Flathub");
            launch(MARCO_FLATHUB_URL)
                .map(|()| log::info!("Opened {MARCO_FLATHUB_URL}"))
                .map_err(|e| format!("Failed to open Marco's install page: {e}"))
        }
    }
}

/// Whether Marco's bus name is activatable, or already taken.
///
/// `ListActivatableNames` covers an installed-but-not-running Marco, which is
/// the common case; `ListNames` covers a running one whose service file is
/// missing for some reason. Inside Flatpak both lists are filtered by
/// `xdg-dbus-proxy` down to the names we are allowed to see, which is exactly
/// why this works as an install check: the `--talk-name=io.github.ranrar.Marco`
/// grant makes a present Marco visible and leaves an absent one invisible.
fn marco_on_bus() -> bool {
    let connection = match gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) {
        Ok(connection) => connection,
        Err(e) => {
            log::warn!("No session bus, cannot look for Marco: {e}");
            return false;
        }
    };

    ["ListActivatableNames", "ListNames"].iter().any(|method| {
        bus_names(&connection, method)
            .iter()
            .any(|n| n == MARCO_APP_ID)
    })
}

/// Call one of the bus daemon's name-listing methods, returning an empty list
/// rather than an error — a failure to ask is indistinguishable, for our
/// purposes, from an answer of "not there".
fn bus_names(connection: &gio::DBusConnection, method: &str) -> Vec<String> {
    let reply = connection.call_sync(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        method,
        None,
        None,
        gio::DBusCallFlags::NONE,
        LIST_NAMES_TIMEOUT_MS,
        gio::Cancellable::NONE,
    );

    match reply {
        Ok(reply) => reply
            .get::<(Vec<String>,)>()
            .map(|(names,)| names)
            .unwrap_or_else(|| {
                log::warn!(
                    "{method} returned an unexpected signature: {}",
                    reply.type_()
                );
                Vec::new()
            }),
        Err(e) => {
            log::debug!("{method} failed: {e}");
            Vec::new()
        }
    }
}

/// Hand the document to Marco over `org.freedesktop.Application.Open`.
///
/// `GApplication` exports this interface itself, so there is no code on the
/// Marco side of this call. Marco's desktop file carries `DBusActivatable=true`
/// so the bus starts it on demand if it is not already running, and forwards to
/// the running instance if it is.
fn open_over_dbus<F>(file_path: &str, on_done: F)
where
    F: FnOnce(Result<(), String>) + 'static,
{
    let connection = match gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) {
        Ok(connection) => connection,
        Err(e) => {
            on_done(Err(format!("No session bus to reach Marco on: {e}")));
            return;
        }
    };

    // Open takes URIs, not paths.
    let uri = gio::File::for_path(file_path).uri();

    let parameters = glib::Variant::tuple_from_iter([
        vec![uri.to_string()].to_variant(),
        // platform-data: nothing to say, but the signature requires it.
        glib::VariantDict::new(None).end(),
    ]);

    log::info!("Asking Marco to open {uri}");

    connection.call(
        Some(MARCO_APP_ID),
        MARCO_OBJECT_PATH,
        "org.freedesktop.Application",
        "Open",
        Some(&parameters),
        None,
        gio::DBusCallFlags::NONE,
        OPEN_TIMEOUT_MS,
        gio::Cancellable::NONE,
        move |result| {
            on_done(
                result
                    .map(|_| ())
                    .map_err(|e| format!("Marco did not accept the document: {e}")),
            );
        },
    );
}

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

/// Launch Marco as a child process. Marco is a single-instance `GApplication`,
/// so a second invocation forwards the document to the running instance and
/// exits rather than opening a second editor.
fn spawn_sibling(file_path: &str) -> Result<(), String> {
    let command =
        marco_executable().ok_or_else(|| "Marco is not installed on this system".to_string())?;

    std::process::Command::new(&command)
        .arg(file_path)
        .spawn()
        .map_err(|e| format!("Failed to spawn Marco: {e}"))?;

    log::info!("Launched Marco: {} {}", command.display(), file_path);
    Ok(())
}
