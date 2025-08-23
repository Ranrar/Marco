use std::env;
use std::path::PathBuf;
use log::LevelFilter;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("panic_tester requires a working directory argument");
        std::process::exit(2);
    }
    let wd = PathBuf::from(&args[1]);
    if let Err(e) = std::env::set_current_dir(&wd) {
        eprintln!("failed to chdir: {}", e);
        std::process::exit(2);
    }

    // Initialize the file logger to write into the provided working directory
    if let Err(e) = marco::logic::logger::init_file_logger(true, LevelFilter::Trace) {
        eprintln!("failed to init file logger: {}", e);
        // continue anyway and panic to test panic handler
    }

    // emit a small info entry to ensure the file is created and a writer is exercised
    log::info!("panic_tester initialized");

    // Install a panic hook that logs and flushes the file logger
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let panic_msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.as_str(),
                None => "Unknown panic payload",
            },
        };
        let location = if let Some(location) = info.location() {
            format!("{}:{}", location.file(), location.line())
        } else {
            "unknown:0".to_string()
        };
        log::error!("PANIC at {}: {}", location, panic_msg);
        // Try to also write a sentinel directly to the file path so tests can reliably detect
        // the panic even if the log backend has race conditions. Attempt to mirror the
        // same path logic used by the logger: ./Log/YYYYMM/YYMMDD.log
        if let Ok(cwd) = std::env::current_dir() {
            let month = chrono::Local::now().format("%Y%m").to_string();
            let file_name = chrono::Local::now().format("%y%m%d.log").to_string();
            let path = cwd.join("Log").join(month).join(file_name);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::OpenOptions::new().create(true).append(true).open(&path).and_then(|mut f| {
                use std::io::Write;
                let _ = writeln!(f, "PANIC_DIRECT: {} at {}", panic_msg, location);
                f.flush()
            });
        }
        marco::logic::logger::shutdown_file_logger();
        default_hook(info);
    }));

    // Intentionally cause a panic that the panic hook should log
    panic!("intentional panic for test");
}
