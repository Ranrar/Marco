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

// Plugin interception trait
pub trait DiagnosticsInterceptor {
    fn intercept(&mut self, event: &Event);
}
