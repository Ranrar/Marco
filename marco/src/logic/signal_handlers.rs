/// Register OS-level signal handlers for graceful shutdown.
///
/// On Linux, hooks SIGINT and SIGTERM via `glib::source::unix_signal_add_local`.
/// On Windows, hooks Ctrl-C via `ctrlc` crate and polls an `AtomicBool` on the
/// GLib main loop.
pub fn setup_signal_handlers(app: &gtk4::Application) {
    #[cfg(target_os = "linux")]
    {
        use glib::ControlFlow;
        use glib_unix::unix_signal_add_local;
        use gtk4::gio::prelude::ApplicationExt;

        // POSIX signal numbers (stable across Unix platforms).
        const SIGINT: i32 = 2;
        const SIGTERM: i32 = 15;

        let app_for_sigint = app.clone();
        unix_signal_add_local(SIGINT, move || {
            log::warn!("Received SIGINT, requesting graceful shutdown...");
            marco_core::logic::logger::shutdown_file_logger();
            app_for_sigint.quit();
            ControlFlow::Break
        });

        let app_for_sigterm = app.clone();
        unix_signal_add_local(SIGTERM, move || {
            log::warn!("Received SIGTERM, requesting graceful shutdown...");
            marco_core::logic::logger::shutdown_file_logger();
            app_for_sigterm.quit();
            ControlFlow::Break
        });
    }

    // Windows: use ctrlc handler for graceful shutdown. Use an AtomicBool + polling
    // timeout so the handler is Send and we stay on the main thread.
    #[cfg(target_os = "windows")]
    {
        use gtk4::glib;
        use gtk4::prelude::ApplicationExt;

        let ctrlc_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let ctrlc_flag_handler = std::sync::Arc::clone(&ctrlc_flag);
        ctrlc::set_handler(move || {
            ctrlc_flag_handler.store(true, std::sync::atomic::Ordering::SeqCst);
        })
        .expect("Failed to set Ctrl-C handler");

        // Poll the flag on the main loop and perform shutdown when set
        let app_for_poll = app.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if ctrlc_flag.load(std::sync::atomic::Ordering::SeqCst) {
                log::warn!("Received Ctrl-C, requesting graceful shutdown...");
                marco_core::logic::logger::shutdown_file_logger();
                app_for_poll.quit();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }
}
