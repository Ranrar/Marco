// Plugin interception trait
pub trait DiagnosticsInterceptor {
    fn intercept(&mut self, event: &Event);
}

// Diagnostics: error/warning event collection and reporting
use crate::logic::parser::event::{Event, SourcePos};

pub struct Diagnostics {
    pub errors: Vec<(String, Option<SourcePos>)>,
    pub warnings: Vec<(String, Option<SourcePos>)>,
    pub unsupported: Vec<(String, Option<SourcePos>)>,
    pub plugin: Option<Box<dyn DiagnosticsInterceptor>>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            unsupported: Vec::new(),
            plugin: None,
        }
    }
    pub fn collect(&mut self, event: &Event) {
        match event {
            Event::Error(msg, pos) => self.errors.push((msg.clone(), pos.clone())),
            Event::Warning(msg, pos) => self.warnings.push((msg.clone(), pos.clone())),
            Event::Unsupported(msg, pos) => self.unsupported.push((msg.clone(), pos.clone())),
            _ => {}
        }
        if let Some(plugin) = &mut self.plugin {
            plugin.intercept(event);
        }
    }
    pub fn set_plugin(&mut self, plugin: Box<dyn DiagnosticsInterceptor>) {
        self.plugin = Some(plugin);
    }
    pub fn report(&self) {
        for (msg, pos) in &self.errors {
            eprintln!("Error at {:?}: {}", pos, msg);
        }
        for (msg, pos) in &self.warnings {
            println!("Warning at {:?}: {}", pos, msg);
        }
        for (msg, pos) in &self.unsupported {
            println!("Unsupported at {:?}: {}", pos, msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::parser::event::{Event, SourcePos};
    use std::sync::{Arc, Mutex};

    struct TestPlugin {
        called: Arc<Mutex<bool>>,
    }
    impl DiagnosticsInterceptor for TestPlugin {
        fn intercept(&mut self, _event: &Event) {
            let mut called = self.called.lock().unwrap();
            *called = true;
        }
    }

    #[test]
    fn collects_error_warning_unsupported() {
        let mut diag = Diagnostics::new();
        let pos = Some(SourcePos { line: 1, column: 2 });
        diag.collect(&Event::Error("err".into(), pos.clone()));
        diag.collect(&Event::Warning("warn".into(), pos.clone()));
        diag.collect(&Event::Unsupported("unsup".into(), pos.clone()));
        assert_eq!(diag.errors.len(), 1);
        assert_eq!(diag.warnings.len(), 1);
        assert_eq!(diag.unsupported.len(), 1);
        assert_eq!(diag.errors[0].0, "err");
        assert_eq!(diag.warnings[0].0, "warn");
        assert_eq!(diag.unsupported[0].0, "unsup");
    }

    #[test]
    fn plugin_intercept_called() {
        let mut diag = Diagnostics::new();
        let called = Arc::new(Mutex::new(false));
        let plugin = TestPlugin { called: Arc::clone(&called) };
        diag.set_plugin(Box::new(plugin));
        diag.collect(&Event::Error("err".into(), None));
        assert!(*called.lock().unwrap(), "Plugin should be called");
    }

    #[test]
    fn report_prints_output() {
        let mut diag = Diagnostics::new();
        diag.errors.push(("err".into(), None));
        diag.warnings.push(("warn".into(), None));
        diag.unsupported.push(("unsup".into(), None));
        // Just call report; output not captured, but should not panic
        diag.report();
    }
}
// Example plugin implementing DiagnosticsInterceptor
pub struct LoggingDiagnosticsPlugin;

impl DiagnosticsInterceptor for LoggingDiagnosticsPlugin {
    fn intercept(&mut self, event: &crate::logic::parser::event::Event) {
        match event {
            crate::logic::parser::event::Event::Error(msg, pos) => {
                eprintln!("[PLUGIN] Error at {:?}: {}", pos, msg);
            }
            crate::logic::parser::event::Event::Warning(msg, pos) => {
                println!("[PLUGIN] Warning at {:?}: {}", pos, msg);
            }
            crate::logic::parser::event::Event::Unsupported(msg, pos) => {
                println!("[PLUGIN] Unsupported at {:?}: {}", pos, msg);
            }
            _ => {}
        }
    }
}
