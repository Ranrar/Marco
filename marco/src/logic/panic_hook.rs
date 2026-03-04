/// Install a panic hook that logs the panic message and flushes the file logger
/// before delegating to the default panic handler.
pub fn install_panic_hook() {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let panic_msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            _ => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_str(),
                _ => "Unknown panic payload",
            },
        };
        let location = if let Some(location) = info.location() {
            format!("{}:{}", location.file(), location.line())
        } else {
            "unknown:0".to_string()
        };
        log::error!("PANIC at {}: {}", location, panic_msg);
        // Try to flush and shutdown the file logger cleanly
        core::logic::logger::shutdown_file_logger();
        // Call the default hook so we preserve existing behavior (printing to stderr)
        default_panic(info);
    }));
}
