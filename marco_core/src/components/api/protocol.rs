//! Marco ↔ Polo IPC Protocol
//!
//! This module defines the RON-serializable data structures for communication
//! between Marco (editor) and Polo (viewer) via IPC (Named Pipes/Unix Sockets).
//!
//! # Message Format
//!
//! All messages are serialized in RON (Rusty Object Notation) and transmitted
//! over local IPC channels (Unix sockets on Linux/macOS, Named Pipes on Windows).
//!
//! # Security
//!
//! - Session keys are UUID v4 (cryptographically random)
//! - Keys are ephemeral (per-session, not persisted)
//! - All requests must include valid session key
//! - Keys are never logged (use `[REDACTED]` in logs)

use serde::{Deserialize, Serialize};

/// Request sent from Polo → Marco
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiRequest {
    /// Get session information
    ///
    /// Polo sends this on startup to fetch document path, themes, etc.
    GetSession { session_key: String },

    /// Request to refresh the current view
    ///
    /// Polo can request Marco to re-parse the document if needed
    RefreshView { session_key: String },

    /// Notify Marco that Polo is closing
    ///
    /// Allows clean session cleanup
    CloseSession { session_key: String },
}

/// Response sent from Marco → Polo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiResponse {
    /// Session data returned successfully
    SessionData(SessionInfo),

    /// Command executed successfully
    Success,

    /// Error occurred during request processing
    Error { message: String },
}

/// Session information shared with Polo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionInfo {
    /// Path to the markdown document being edited
    pub document_path: Option<String>,

    /// HTML preview theme (e.g., "github.css", "marco.css")
    pub theme: String,

    /// Editor theme/scheme (e.g., "marco-dark", "marco-light")
    pub editor_theme: String,

    /// Whether Polo should be read-only (currently always true)
    pub read_only: bool,
}

/// Push notification from Marco → Polo (future use for bidirectional updates)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiNotification {
    /// Theme changed in Marco, Polo should update
    ThemeUpdated {
        theme: String,
        editor_theme: String,
    },

    /// Document content was modified
    DocumentModified,

    /// Marco is shutting down, Polo should close
    ServerClosing,
}

/// Commands sent from Marco → Polo (push-based updates)
///
/// These are proactive commands from Marco to Polo, not responses to requests.
/// Marco pushes content updates, theme changes, and scroll sync to Polo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerCommand {
    /// Refresh Polo's content with new HTML
    RefreshContent {
        /// Rendered HTML content
        html: String,
        /// Optional scroll position to restore (0.0 = top, 1.0 = bottom)
        scroll_position: Option<f64>,
    },

    /// Update Polo's theme
    UpdateTheme {
        /// HTML preview theme (e.g., "github.css")
        theme: String,
        /// Editor theme (e.g., "dark", "light")
        editor_theme: String,
    },

    /// Scroll Polo's view to specific position
    ScrollTo {
        /// Scroll position (0.0 = top, 1.0 = bottom)
        position: f64,
    },

    /// Gracefully shutdown Polo
    Shutdown,
}

/// Response from Polo after processing a ServerCommand
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommandResponse {
    /// Command executed successfully
    Ok,
    
    /// Command failed
    Error { message: String },
}

impl ServerCommand {
    /// Create a RefreshContent command
    pub fn refresh_content(html: String, scroll_position: Option<f64>) -> Self {
        ServerCommand::RefreshContent {
            html,
            scroll_position,
        }
    }

    /// Create an UpdateTheme command
    pub fn update_theme(theme: String, editor_theme: String) -> Self {
        ServerCommand::UpdateTheme {
            theme,
            editor_theme,
        }
    }

    /// Create a ScrollTo command
    pub fn scroll_to(position: f64) -> Self {
        ServerCommand::ScrollTo { position }
    }

    /// Create a Shutdown command
    pub fn shutdown() -> Self {
        ServerCommand::Shutdown
    }
}

impl CommandResponse {
    /// Create an Ok response
    pub fn ok() -> Self {
        CommandResponse::Ok
    }

    /// Create an Error response
    pub fn error(message: impl Into<String>) -> Self {
        CommandResponse::Error {
            message: message.into(),
        }
    }
}

impl ApiRequest {
    /// Extract the session key from any request variant
    pub fn session_key(&self) -> &str {
        match self {
            ApiRequest::GetSession { session_key } => session_key,
            ApiRequest::RefreshView { session_key } => session_key,
            ApiRequest::CloseSession { session_key } => session_key,
        }
    }

    /// Create a GetSession request
    pub fn get_session(session_key: String) -> Self {
        ApiRequest::GetSession { session_key }
    }

    /// Create a RefreshView request
    pub fn refresh_view(session_key: String) -> Self {
        ApiRequest::RefreshView { session_key }
    }

    /// Create a CloseSession request
    pub fn close_session(session_key: String) -> Self {
        ApiRequest::CloseSession { session_key }
    }
}

impl ApiResponse {
    /// Create a success response
    pub fn success() -> Self {
        ApiResponse::Success
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        ApiResponse::Error {
            message: message.into(),
        }
    }

    /// Create a session data response
    pub fn session_data(info: SessionInfo) -> Self {
        ApiResponse::SessionData(info)
    }
}

impl SessionInfo {
    /// Create new session info
    pub fn new(
        document_path: Option<String>,
        theme: String,
        editor_theme: String,
        read_only: bool,
    ) -> Self {
        SessionInfo {
            document_path,
            theme,
            editor_theme,
            read_only,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_request_serialization() {
        let request = ApiRequest::GetSession {
            session_key: "test-key-123".to_string(),
        };

        // Serialize to RON
        let ron = ron::to_string(&request).expect("Failed to serialize");
        assert!(ron.contains("test-key-123"));

        // Deserialize back
        let parsed: ApiRequest = ron::from_str(&ron).expect("Failed to deserialize");
        assert_eq!(request, parsed);
    }

    #[test]
    fn smoke_test_response_serialization() {
        let response = ApiResponse::SessionData(SessionInfo {
            document_path: Some("/test/doc.md".to_string()),
            theme: "github.css".to_string(),
            editor_theme: "marco-dark".to_string(),
            read_only: true,
        });

        // Serialize to RON
        let ron = ron::to_string(&response).expect("Failed to serialize");
        assert!(ron.contains("github.css"));

        // Deserialize back
        let parsed: ApiResponse = ron::from_str(&ron).expect("Failed to deserialize");
        assert_eq!(response, parsed);
    }

    #[test]
    fn smoke_test_error_response() {
        let response = ApiResponse::Error {
            message: "Invalid session".to_string(),
        };

        let ron = ron::to_string(&response).expect("Failed to serialize");
        let parsed: ApiResponse = ron::from_str(&ron).expect("Failed to deserialize");
        assert_eq!(response, parsed);
    }

    #[test]
    fn smoke_test_session_key_extraction() {
        let request = ApiRequest::GetSession {
            session_key: "key-123".to_string(),
        };
        assert_eq!(request.session_key(), "key-123");

        let request = ApiRequest::RefreshView {
            session_key: "key-456".to_string(),
        };
        assert_eq!(request.session_key(), "key-456");

        let request = ApiRequest::CloseSession {
            session_key: "key-789".to_string(),
        };
        assert_eq!(request.session_key(), "key-789");
    }

    #[test]
    fn smoke_test_request_builders() {
        let request = ApiRequest::get_session("key-1".to_string());
        assert_eq!(request.session_key(), "key-1");

        let request = ApiRequest::refresh_view("key-2".to_string());
        assert_eq!(request.session_key(), "key-2");

        let request = ApiRequest::close_session("key-3".to_string());
        assert_eq!(request.session_key(), "key-3");
    }

    #[test]
    fn smoke_test_response_builders() {
        let response = ApiResponse::success();
        assert!(matches!(response, ApiResponse::Success));

        let response = ApiResponse::error("test error");
        match response {
            ApiResponse::Error { message } => assert_eq!(message, "test error"),
            _ => panic!("Expected error response"),
        }

        let info = SessionInfo::new(
            Some("/test.md".to_string()),
            "theme.css".to_string(),
            "editor-theme".to_string(),
            false,
        );
        let response = ApiResponse::session_data(info.clone());
        match response {
            ApiResponse::SessionData(returned_info) => assert_eq!(returned_info, info),
            _ => panic!("Expected session data response"),
        }
    }

    #[test]
    fn smoke_test_server_command_builders() {
        let cmd = ServerCommand::refresh_content("<h1>Test</h1>".to_string(), Some(0.5));
        match cmd {
            ServerCommand::RefreshContent { html, scroll_position } => {
                assert_eq!(html, "<h1>Test</h1>");
                assert_eq!(scroll_position, Some(0.5));
            }
            _ => panic!("Wrong variant"),
        }

        let cmd = ServerCommand::update_theme("github".to_string(), "dark".to_string());
        match cmd {
            ServerCommand::UpdateTheme { theme, editor_theme } => {
                assert_eq!(theme, "github");
                assert_eq!(editor_theme, "dark");
            }
            _ => panic!("Wrong variant"),
        }

        let cmd = ServerCommand::scroll_to(0.75);
        match cmd {
            ServerCommand::ScrollTo { position } => assert_eq!(position, 0.75),
            _ => panic!("Wrong variant"),
        }

        let cmd = ServerCommand::shutdown();
        assert!(matches!(cmd, ServerCommand::Shutdown));
    }

    #[test]
    fn smoke_test_server_command_serialization() {
        let cmd = ServerCommand::refresh_content("<p>Content</p>".to_string(), None);
        let ron_string = ron::to_string(&cmd).expect("Failed to serialize");
        let deserialized: ServerCommand = ron::from_str(&ron_string).expect("Failed to deserialize");
        assert_eq!(cmd, deserialized);

        let cmd = ServerCommand::update_theme("marco".to_string(), "light".to_string());
        let ron_string = ron::to_string(&cmd).expect("Failed to serialize");
        let deserialized: ServerCommand = ron::from_str(&ron_string).expect("Failed to deserialize");
        assert_eq!(cmd, deserialized);

        let cmd = ServerCommand::scroll_to(0.25);
        let ron_string = ron::to_string(&cmd).expect("Failed to serialize");
        let deserialized: ServerCommand = ron::from_str(&ron_string).expect("Failed to deserialize");
        assert_eq!(cmd, deserialized);

        let cmd = ServerCommand::shutdown();
        let ron_string = ron::to_string(&cmd).expect("Failed to serialize");
        let deserialized: ServerCommand = ron::from_str(&ron_string).expect("Failed to deserialize");
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn smoke_test_command_response() {
        let resp = CommandResponse::ok();
        assert!(matches!(resp, CommandResponse::Ok));

        let resp = CommandResponse::error("test error");
        match resp {
            CommandResponse::Error { message } => assert_eq!(message, "test error"),
            _ => panic!("Wrong variant"),
        }

        // Test serialization
        let resp = CommandResponse::ok();
        let ron_string = ron::to_string(&resp).expect("Failed to serialize");
        let deserialized: CommandResponse = ron::from_str(&ron_string).expect("Failed to deserialize");
        assert_eq!(resp, deserialized);

        let resp = CommandResponse::error("connection lost");
        let ron_string = ron::to_string(&resp).expect("Failed to serialize");
        let deserialized: CommandResponse = ron::from_str(&ron_string).expect("Failed to deserialize");
        assert_eq!(resp, deserialized);
    }
}
