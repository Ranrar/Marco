/// A background thread that processes settings persistence operations sequentially.
///
/// This prevents race conditions and ensures proper resource cleanup when saving
/// settings from UI callbacks.
pub struct SettingsThreadPool {
    pub tx: std::sync::mpsc::Sender<Box<dyn FnOnce() + Send>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Default for SettingsThreadPool {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsThreadPool {
    /// Create a new settings thread pool with a single background worker.
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel::<Box<dyn FnOnce() + Send>>();
        let handle = std::thread::spawn(move || {
            while let Ok(task) = rx.recv() {
                task();
            }
            log::debug!("Settings thread pool shutting down");
        });
        Self {
            tx,
            handle: Some(handle),
        }
    }

    /// Shut down the background thread by dropping the sender side and
    /// joining the worker thread.
    pub fn shutdown(&mut self) {
        // Replace the sender with a dummy to close the channel
        let (dummy_tx, _) = std::sync::mpsc::channel();
        let _ = std::mem::replace(&mut self.tx, dummy_tx);

        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.join() {
                log::error!("Failed to join settings thread: {:?}", e);
            } else {
                log::debug!("Settings thread cleaned up successfully");
            }
        }
    }
}
