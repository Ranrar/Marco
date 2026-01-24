// MIT License - Marco Project
// GTK4 EventLoopWaker implementation for Servo

use embedder_traits::EventLoopWaker;
use glib::MainContext;

/// GTK4 event loop waker that integrates Servo with GTK's main loop.
///
/// This wakes the GTK main thread when Servo has work to do, ensuring
/// that `Servo::spin_event_loop()` gets called promptly.
#[derive(Clone)]
pub struct GtkEventLoopWaker;

impl GtkEventLoopWaker {
    pub fn new() -> Self {
        Self
    }
}

impl EventLoopWaker for GtkEventLoopWaker {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        Box::new(self.clone())
    }

    fn wake(&self) {
        // Queue an idle callback on GTK's main loop
        // This ensures servo.spin_event_loop() gets called
        MainContext::default().invoke(|| {
            // The actual spin will happen in the widget's tick callback
        });
    }
}
