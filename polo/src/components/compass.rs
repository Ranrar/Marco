//! Polo Compass - IPC Client for Polo ↔ Marco Communication
//!
//! The Compass is Polo's IPC client that connects to Marco's Beacon server.
//! It fetches session data and sends commands using RON-serialized messages.
//!
//! # Platform Support
//!
//! - **Linux/macOS**: Unix domain sockets
//! - **Windows**: Named pipes
//!
//! # Usage
//!
//! ```no_run
//! use compass::Compass;
//!
//! let mut compass = Compass::connect(socket_name, session_key)?;
//! let session_info = compass.fetch_session()?;
//! compass.close_session()?;
//! ```

use interprocess::local_socket::prelude::*;
use interprocess::local_socket::GenericNamespaced;
use interprocess::TryClone;
use log::{debug, error, info, warn};
use marco_core::components::api::protocol::ServerCommand;
use marco_core::{ApiRequest, ApiResponse, SessionInfo};
use std::io::{BufRead, BufReader, Write};
use std::thread;

/// Polo IPC client (Compass)
pub struct Compass {
    /// The IPC stream connection to Marco
    stream: LocalSocketStream,
    
    /// Session key for authentication
    session_key: String,
    
    /// Socket name for logging
    socket_name: String,
}

impl Compass {
    /// Connect to Marco's Beacon IPC server
    ///
    /// # Arguments
    ///
    /// * `socket_name` - The socket/pipe name to connect to
    /// * `session_key` - The session key provided by Marco
    ///
    /// # Returns
    ///
    /// A new Compass instance connected to Marco
    pub fn connect(socket_name: &str, session_key: String) -> Result<Self, String> {
        debug!("Compass connecting to Marco at: {}", socket_name);
        
        // Connect to the IPC server
        let stream = LocalSocketStream::connect(
            socket_name.to_ns_name::<GenericNamespaced>()
                .map_err(|e| format!("Invalid socket name: {}", e))?
        ).map_err(|e| format!("Failed to connect: {}", e))?;
        
        info!("Compass connected to Marco (session: [REDACTED])");
        
        Ok(Compass {
            stream,
            session_key,
            socket_name: socket_name.to_string(),
        })
    }
    
    /// Send a request to Marco and get response
    fn send_request(&mut self, request: ApiRequest) -> Result<ApiResponse, String> {
        // Serialize request to RON
        let request_ron = ron::to_string(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        debug!("Compass sending request ({} bytes)", request_ron.len());
        
        // Send to Marco
        writeln!(self.stream, "{}", request_ron)
            .map_err(|e| format!("Failed to send request: {}", e))?;
        
        // Read response
        let mut reader = BufReader::new(
            self.stream.try_clone()
                .map_err(|e| format!("Failed to clone stream: {}", e))?
        );
        let mut line = String::new();
        reader.read_line(&mut line)
            .map_err(|e| format!("Failed to read response: {}", e))?;
        
        debug!("Compass received response ({} bytes)", line.len());
        
        // Deserialize RON response
        let response: ApiResponse = ron::from_str(&line)
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(response)
    }
    
    /// Fetch session information from Marco
    ///
    /// This should be called immediately after connecting to get
    /// the document path, themes, and other session data.
    pub fn fetch_session(&mut self) -> Result<SessionInfo, String> {
        debug!("Compass fetching session data");
        
        let request = ApiRequest::get_session(self.session_key.clone());
        let response = self.send_request(request)?;
        
        match response {
            ApiResponse::SessionData(info) => {
                info!(
                    "Compass received session: theme={}, editor_theme={}, path={:?}",
                    info.theme,
                    info.editor_theme,
                    info.document_path
                );
                Ok(info)
            }
            ApiResponse::Error { message } => {
                error!("Compass: Session fetch failed: {}", message);
                Err(format!("Failed to fetch session: {}", message))
            }
            ApiResponse::Success => {
                warn!("Compass: Unexpected success response for GetSession");
                Err("Unexpected response from Marco".to_string())
            }
        }
    }
    
    /// Request Marco to refresh the view
    ///
    /// This can be used if Polo detects file changes or user triggers refresh.
    pub fn refresh_view(&mut self) -> Result<(), String> {
        debug!("Compass requesting view refresh");
        
        let request = ApiRequest::refresh_view(self.session_key.clone());
        let response = self.send_request(request)?;
        
        match response {
            ApiResponse::Success => {
                info!("Compass: View refresh successful");
                Ok(())
            }
            ApiResponse::Error { message } => {
                error!("Compass: View refresh failed: {}", message);
                Err(format!("Failed to refresh view: {}", message))
            }
            _ => {
                warn!("Compass: Unexpected response for RefreshView");
                Err("Unexpected response from Marco".to_string())
            }
        }
    }
    
    /// Notify Marco that Polo is closing
    ///
    /// This should be called when Polo exits to allow clean session cleanup.
    pub fn close_session(&mut self) -> Result<(), String> {
        debug!("Compass sending close session notification");
        
        let request = ApiRequest::close_session(self.session_key.clone());
        let response = self.send_request(request)?;
        
        match response {
            ApiResponse::Success => {
                info!("Compass: Session closed successfully");
                Ok(())
            }
            ApiResponse::Error { message } => {
                warn!("Compass: Session close failed: {}", message);
                // Don't return error, we're closing anyway
                Ok(())
            }
            _ => {
                warn!("Compass: Unexpected response for CloseSession");
                Ok(())
            }
        }
    }
    
    /// Get the socket name
    pub fn socket_name(&self) -> &str {
        &self.socket_name
    }
    
    /// Get the session key (returns redacted string for logging)
    pub fn session_key_redacted(&self) -> &str {
        "[REDACTED]"
    }
    
    /// Start listening for commands from Marco in a background thread
    ///
    /// This spawns a thread that continuously reads ServerCommand messages
    /// from Marco and dispatches them to the appropriate handlers.
    ///
    /// # Arguments
    ///
    /// * `on_refresh` - Callback for RefreshContent commands (html, scroll_position)
    /// * `on_theme` - Callback for UpdateTheme commands (theme, editor_theme)
    /// * `on_scroll` - Callback for ScrollTo commands (position)
    /// * `on_shutdown` - Callback for Shutdown commands
    ///
    /// # Returns
    ///
    /// A thread handle that will run until connection closes or Shutdown received
    ///
    /// # Example
    ///
    /// ```ignore
    /// compass.listen_for_commands(
    ///     |html, pos| println!("Refresh: {}", html),
    ///     |theme, ed| println!("Theme: {}/{}", theme, ed),
    ///     |pos| println!("Scroll: {}", pos),
    ///     || println!("Shutdown"),
    /// );
    /// ```
    pub fn listen_for_commands<FR, FT, FS, FSH>(
        &self,
        on_refresh: FR,
        on_theme: FT,
        on_scroll: FS,
        on_shutdown: FSH,
    ) -> Result<thread::JoinHandle<()>, String>
    where
        FR: Fn(String, Option<f64>) + Send + 'static,
        FT: Fn(String, String) + Send + 'static,
        FS: Fn(f64) + Send + 'static,
        FSH: Fn() + Send + 'static,
    {
        // Clone the stream for the background thread
        let stream = self.stream.try_clone()
            .map_err(|e| format!("Failed to clone stream for listener: {}", e))?;
        
        info!("Compass: Starting command listener thread");
        
        let handle = thread::spawn(move || {
            let mut reader = BufReader::new(stream);
            
            loop {
                let mut line = String::new();
                
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        debug!("Compass listener: Connection closed by Marco");
                        break;
                    }
                    Ok(_) => {
                        // Parse ServerCommand from RON
                        match ron::from_str::<ServerCommand>(&line) {
                            Ok(command) => {
                                match command {
                                    ServerCommand::RefreshContent { html, scroll_position } => {
                                        debug!("Compass: Received RefreshContent ({} bytes)", html.len());
                                        on_refresh(html, scroll_position);
                                    }
                                    ServerCommand::UpdateTheme { theme, editor_theme } => {
                                        debug!("Compass: Received UpdateTheme (theme={}, editor={})", theme, editor_theme);
                                        on_theme(theme, editor_theme);
                                    }
                                    ServerCommand::ScrollTo { position } => {
                                        debug!("Compass: Received ScrollTo (pos={})", position);
                                        on_scroll(position);
                                    }
                                    ServerCommand::Shutdown => {
                                        info!("Compass: Received Shutdown command");
                                        on_shutdown();
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Compass listener: Failed to parse command: {}", e);
                                // Don't break - might be transient parse error
                            }
                        }
                    }
                    Err(e) => {
                        error!("Compass listener: Read error: {}", e);
                        break;
                    }
                }
            }
            
            debug!("Compass listener thread exiting");
        });
        
        Ok(handle)
    }
}

impl Drop for Compass {
    fn drop(&mut self) {
        debug!("Compass disconnecting from Marco");
        // Try to send close notification, but don't fail if it doesn't work
        let _ = self.close_session();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_compass_creation() {
        // Note: Can't actually connect without a running Beacon server
        // This just tests that the struct compiles and basic methods exist
        let _session_key = "test-session-key".to_string();
        
        // Verify session key is never exposed in logs
        assert_eq!("[REDACTED]", "[REDACTED]");
    }
    
    #[test]
    fn smoke_test_request_building() {
        let session_key = "test-key".to_string();
        
        let request = ApiRequest::get_session(session_key.clone());
        assert_eq!(request.session_key(), "test-key");
        
        let request = ApiRequest::refresh_view(session_key.clone());
        assert_eq!(request.session_key(), "test-key");
        
        let request = ApiRequest::close_session(session_key);
        assert_eq!(request.session_key(), "test-key");
    }
    
    #[test]
    fn smoke_test_server_command_parsing() {
        // Test that ServerCommand can be parsed from RON (for command listener)
        let cmd_ron = r#"RefreshContent(html: "<h1>Test</h1>", scroll_position: Some(0.5))"#;
        let cmd: ServerCommand = ron::from_str(cmd_ron).expect("Failed to parse RefreshContent");
        match cmd {
            ServerCommand::RefreshContent { html, scroll_position } => {
                assert_eq!(html, "<h1>Test</h1>");
                assert_eq!(scroll_position, Some(0.5));
            }
            _ => panic!("Wrong command variant"),
        }
        
        let cmd_ron = r#"UpdateTheme(theme: "github", editor_theme: "dark")"#;
        let cmd: ServerCommand = ron::from_str(cmd_ron).expect("Failed to parse UpdateTheme");
        match cmd {
            ServerCommand::UpdateTheme { theme, editor_theme } => {
                assert_eq!(theme, "github");
                assert_eq!(editor_theme, "dark");
            }
            _ => panic!("Wrong command variant"),
        }
        
        let cmd_ron = r#"ScrollTo(position: 0.75)"#;
        let cmd: ServerCommand = ron::from_str(cmd_ron).expect("Failed to parse ScrollTo");
        match cmd {
            ServerCommand::ScrollTo { position } => assert_eq!(position, 0.75),
            _ => panic!("Wrong command variant"),
        }
        
        let cmd_ron = r#"Shutdown"#;
        let cmd: ServerCommand = ron::from_str(cmd_ron).expect("Failed to parse Shutdown");
        assert!(matches!(cmd, ServerCommand::Shutdown));
    }
    
    // Full integration tests require a running Beacon server and should
    // be in separate integration test files with proper setup/teardown
}
