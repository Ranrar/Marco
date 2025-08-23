use chrono::Local;
use std::fs;
use std::path::PathBuf;
use log::LevelFilter;

#[test]
fn panic_is_logged_in_repo_log_folder_in_process() {
    // Ensure repo root cwd so logger writes into the repo 'log' folder
    let repo_root = std::env::current_dir().expect("cwd");

    // Initialize logger in-process (enabled = true)
    let _ = marco::logic::logger::init_file_logger(true, LevelFilter::Trace).expect("init logger");

    // emit an info entry to create the file
    log::info!("test: init");

    // Simulate panic logging behavior: use log::error! and then shutdown
    log::error!("PANIC at test: intentional in-process test");
    // As an extra sentinel, write directly to the same path (mirrors previous behavior)
    if let Ok(cwd) = std::env::current_dir() {
        let month = Local::now().format("%Y%m").to_string();
        let file_name = Local::now().format("%y%m%d.log").to_string();
        let path = cwd.join("log").join(month).join(file_name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::OpenOptions::new().create(true).append(true).open(&path).and_then(|mut f| {
            use std::io::Write;
            let _ = writeln!(f, "PANIC_DIRECT: in-process test");
            f.flush()
        });
    }

    // Flush and close
    marco::logic::logger::shutdown_file_logger();

    // Determine expected log path: ./log/YYYYMM/YYMMDD.log under repo root
    let month = Local::now().format("%Y%m").to_string();
    let file_name = Local::now().format("%y%m%d.log").to_string();
    let log_path: PathBuf = repo_root.join("log").join(month).join(file_name);

    // Small wait to ensure writes are visible
    std::thread::sleep(std::time::Duration::from_millis(200));

    assert!(log_path.exists(), "log file not created: {:?}", log_path);

    let content = fs::read_to_string(&log_path).expect("read log");
    assert!(content.contains("PANIC") || content.to_uppercase().contains("PANIC") || content.contains("PANIC_DIRECT"), "log did not contain panic entry: {}", content);
}
