//! Marco Beacon - IPC Server for Marco ↔ Polo Communication
//!
//! The Beacon is Marco's IPC server that listens for connections from Polo viewers.
//! It handles session validation and command processing using RON-serialized messages.
//!
//! # Platform Support
//!
//! - **Linux/macOS**: Unix domain sockets
//! - **Windows**: Named pipes
//!
//! # Architecture
//!
//! The Beacon runs in Marco's main thread (GTK single-threaded model) and accepts
//! connections from Polo instances. Each connection is handled synchronously via
//! message-passing.

use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions};
use interprocess::TryClone;
use log::{debug, error, info, warn};
use marco_core::components::api::protocol::ServerCommand;
use marco_core::{ApiRequest, ApiResponse, SessionManager};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};

/// Marco IPC server (Beacon)
pub struct Beacon {
    /// The IPC listener (Unix socket or Named Pipe)
    listener: LocalSocketListener,
    
    /// Session manager for validating requests
    session_manager: Arc<SessionManager>,
    
    /// Socket name for logging
    socket_name: String,
    
    /// Active connections to Polo instances (session_key -> stream)
    /// Wrapped in Arc<Mutex<>> for thread-safe access from background thread
    connections: Arc<Mutex<HashMap<String, LocalSocketStream>>>,
}

impl Beacon {
    /// Create a new Beacon IPC server
    ///
    /// # Arguments
    ///
    /// * `socket_name` - The socket/pipe name to bind to
    /// * `session_manager` - Shared session manager for validation
    ///
    /// # Socket Naming
    ///
    /// - **Linux/macOS**: `/tmp/marco-{session_key}.sock`
    /// - **Windows**: `\\.\pipe\marco-{session_key}`
    ///
    /// # Returns
    ///
    /// A new Beacon instance ready to accept connections
    pub fn new(socket_name: &str, session_manager: Arc<SessionManager>) -> Result<Self, String> {
        debug!("Creating Beacon IPC server at: {}", socket_name);
        
        // Create the listener using the correct API for interprocess 2.x
        // On Unix: creates a socket file, on Windows: creates a named pipe
        let listener = ListenerOptions::new()
            .name(socket_name.to_ns_name::<GenericNamespaced>()
                .map_err(|e| format!("Invalid socket name: {}", e))?)
            .create_sync()
            .map_err(|e| format!("Failed to bind IPC listener: {}", e))?;
        
        info!("Beacon IPC server listening at: {}", socket_name);
        
        Ok(Beacon {
            listener,
            session_manager,
            socket_name: socket_name.to_string(),
            connections: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Accept a single connection and handle it
    ///
    /// This is a blocking call that waits for a Polo client to connect.
    /// Once connected, it processes one request and closes the connection.
    ///
    /// For continuous listening, call this in a loop or use GTK timeouts.
    pub fn accept_connection(&self) -> Result<(), String> {
        debug!("Beacon waiting for connection...");
        
        match self.listener.accept() {
            Ok(stream) => {
                debug!("Beacon accepted connection");
                self.handle_client(stream)?;
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                return Err(format!("Accept failed: {}", e));
            }
        }
        
        Ok(())
    }
    
    /// Handle a client connection
    ///
    /// For GetSession requests, the connection is stored in the registry for
    /// sending future ServerCommands. Other requests are handled and closed.
    fn handle_client(&self, stream: LocalSocketStream) -> Result<(), String> {
        let mut reader = BufReader::new(&stream);
        let mut writer = stream.try_clone()
            .map_err(|e| format!("Failed to clone stream: {}", e))?;
        
        // Read RON message from Polo
        let mut line = String::new();
        reader.read_line(&mut line)
            .map_err(|e| format!("Failed to read request: {}", e))?;
        
        debug!("Beacon received request ({} bytes)", line.len());
        
        // Deserialize RON request
        let request: ApiRequest = match ron::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse RON request: {}", e);
                let error_response = ApiResponse::error("Malformed request");
                let response_ron = ron::to_string(&error_response)
                    .map_err(|e| format!("Failed to serialize error response: {}", e))?;
                writeln!(writer, "{}", response_ron)
                    .map_err(|e| format!("Failed to write error response: {}", e))?;
                return Ok(());
            }
        };
        
        let session_key = request.session_key().to_string();
        let is_get_session = matches!(request, ApiRequest::GetSession { .. });
        
        // Handle the request
        let response = self.handle_request(request);
        
        // Serialize RON response
        let response_ron = ron::to_string(&response)
            .map_err(|e| format!("Failed to serialize response: {}", e))?;
        writeln!(writer, "{}", response_ron)
            .map_err(|e| format!("Failed to write response: {}", e))?;
        
        debug!("Beacon sent response ({} bytes)", response_ron.len());
        
        // If this was a GetSession request and it succeeded, store the connection
        if is_get_session && matches!(response, ApiResponse::SessionData(_)) {
            let stream_clone = stream.try_clone()
                .map_err(|e| format!("Failed to clone stream for storage: {}", e))?;
            
            let mut connections = self.connections.lock()
                .map_err(|e| format!("Failed to lock connections: {}", e))?;
            connections.insert(session_key.clone(), stream_clone);
            info!("Beacon: Stored persistent connection for session");
        }
        
        Ok(())
    }
    
    /// Handle an API request
    fn handle_request(&self, request: ApiRequest) -> ApiResponse {
        let session_key = request.session_key();
        
        // Validate session
        if !self.session_manager.validate_session(session_key) {
            warn!("Beacon: Invalid session key in request");
            return ApiResponse::error("Invalid session");
        }
        
        // Handle specific commands
        match request {
            ApiRequest::GetSession { .. } => {
                debug!("Beacon: Handling GetSession request");
                match self.session_manager.get_session_info(session_key) {
                    Some(info) => {
                        info!("Beacon: Sent session data (theme: {}, editor_theme: {})", 
                              info.theme, info.editor_theme);
                        ApiResponse::session_data(info)
                    }
                    None => ApiResponse::error("Session not found"),
                }
            }
            
            ApiRequest::RefreshView { .. } => {
                debug!("Beacon: Handling RefreshView request");
                // TODO: Implement document refresh logic
                info!("Beacon: RefreshView requested");
                ApiResponse::success()
            }
            
            ApiRequest::CloseSession { .. } => {
                debug!("Beacon: Handling CloseSession request");
                self.session_manager.remove_session(session_key);
                
                // Remove the stored connection
                self.remove_connection(session_key);
                
                info!("Beacon: Session closed");
                ApiResponse::success()
            }
        }
    }
    
    /// Send a command to a specific Polo instance
    ///
    /// # Arguments
    ///
    /// * `session_key` - The session key identifying the target Polo instance
    /// * `command` - The ServerCommand to send
    ///
    /// # Returns
    ///
    /// Result indicating success or error message
    ///
    /// # Example
    ///
    /// ```ignore
    /// let html = "<h1>Updated Content</h1>".to_string();
    /// beacon.send_command(&session_key, ServerCommand::refresh_content(html, None))?;
    /// ```
    pub fn send_command(&self, session_key: &str, command: ServerCommand) -> Result<(), String> {
        let mut connections = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        
        let stream = match connections.get_mut(session_key) {
            Some(s) => s,
            None => {
                warn!("Beacon: No connection found for session");
                return Err("Connection not found".to_string());
            }
        };
        
        // Serialize command to RON
        let command_ron = ron::to_string(&command)
            .map_err(|e| format!("Failed to serialize command: {}", e))?;
        
        // Send to Polo
        match writeln!(stream, "{}", command_ron) {
            Ok(_) => {
                debug!("Beacon: Sent command ({} bytes)", command_ron.len());
                Ok(())
            }
            Err(e) => {
                error!("Beacon: Failed to send command: {}", e);
                // Remove broken connection
                connections.remove(session_key);
                Err(format!("Failed to send command: {}", e))
            }
        }
    }
    
    /// Remove a connection from the registry
    ///
    /// Called when a session is closed or a connection fails
    pub fn remove_connection(&self, session_key: &str) {
        if let Ok(mut connections) = self.connections.lock() {
            if connections.remove(session_key).is_some() {
                debug!("Beacon: Removed connection for session");
            }
        }
    }
    
    /// Get count of active connections
    pub fn connection_count(&self) -> usize {
        self.connections.lock()
            .map(|c| c.len())
            .unwrap_or(0)
    }
    
    /// Get the socket name
    pub fn socket_name(&self) -> &str {
        &self.socket_name
    }
}

impl Drop for Beacon {
    fn drop(&mut self) {
        debug!("Beacon shutting down (socket: {})", self.socket_name);
    }
}

/// Generate platform-specific socket name for a session
///
/// # Arguments
///
/// * `session_key` - The UUID session key
///
/// # Returns
///
/// Platform-appropriate socket name:
/// - Linux/macOS: `/tmp/marco-{session_key}.sock`
/// - Windows: `\\.\pipe\marco-{session_key}`
pub fn generate_socket_name(session_key: &str) -> String {
    if cfg!(windows) {
        format!(r"\\.\pipe\marco-{}", session_key)
    } else {
        format!("/tmp/marco-{}.sock", session_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use marco_core::SessionData;
    use std::path::PathBuf;
    
    #[test]
    fn smoke_test_socket_name_generation() {
        let session_key = "test-key-123";
        let socket_name = generate_socket_name(session_key);
        
        if cfg!(windows) {
            assert!(socket_name.contains(r"\\.\pipe\marco-"));
        } else {
            assert!(socket_name.starts_with("/tmp/marco-"));
            assert!(socket_name.ends_with(".sock"));
        }
        
        assert!(socket_name.contains(session_key));
    }
    
    // Note: Full IPC tests require actually binding sockets, which can
    // conflict with existing test infrastructure. Integration tests
    // should be run separately with proper cleanup.
    
    #[test]
    fn smoke_test_request_handling() {
        let session_manager = Arc::new(SessionManager::new());
        
        // Create a session
        let session_key = SessionManager::generate_session_key();
        let session_data = SessionData::new(
            session_key.clone(),
            Some(PathBuf::from("/test.md")),
            "github.css".to_string(),
            "marco-dark".to_string(),
            true,
        );
        session_manager.create_session(session_data);
        
        // Note: Can't create actual Beacon without binding socket
        // This just tests the session manager integration
        
        assert!(session_manager.validate_session(&session_key));
        let info = session_manager.get_session_info(&session_key).unwrap();
        assert_eq!(info.theme, "github.css");
    }
    
    #[test]
    fn smoke_test_connection_registry() {
        let session_manager = Arc::new(SessionManager::new());
        let socket_name = "/tmp/test-beacon-registry.sock";
        
        // Create beacon (will fail to bind, but that's ok for this test)
        // We just want to test the connection registry logic
        let beacon = match Beacon::new(socket_name, session_manager) {
            Ok(b) => b,
            Err(_) => {
                // If socket already exists, skip test
                // (Integration tests should clean up properly)
                return;
            }
        };
        
        // Test initial state
        assert_eq!(beacon.connection_count(), 0);
        
        // Test remove on non-existent connection (should not panic)
        beacon.remove_connection("nonexistent");
        assert_eq!(beacon.connection_count(), 0);
    }
    
    #[test]
    fn smoke_test_send_command_error() {
        let session_manager = Arc::new(SessionManager::new());
        let socket_name = "/tmp/test-beacon-command.sock";
        
        let beacon = match Beacon::new(socket_name, session_manager) {
            Ok(b) => b,
            Err(_) => return, // Skip if socket exists
        };
        
        // Test sending command to non-existent connection
        let result = beacon.send_command(
            "nonexistent-session", 
            ServerCommand::shutdown()
        );
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Connection not found"));
    }
}
