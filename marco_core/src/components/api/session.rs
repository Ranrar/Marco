//! Session Management for Marco ↔ Polo IPC
//!
//! This module provides session key generation, validation, and session data storage.
//! Sessions are ephemeral and stored in memory only.

use super::protocol::SessionInfo;
use log::{info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use uuid::Uuid;

/// Session data stored in Marco
#[derive(Debug, Clone)]
pub struct SessionData {
    /// Unique session key (UUID v4)
    pub key: String,

    /// Document being edited (if any)
    pub document_path: Option<PathBuf>,

    /// Current HTML preview theme
    pub theme: String,

    /// Current editor theme/scheme
    pub editor_theme: String,

    /// Read-only flag (Polo is always read-only)
    pub read_only: bool,

    /// Polo process ID (for monitoring)
    pub polo_pid: Option<u32>,

    /// Session creation timestamp
    pub created_at: SystemTime,
}

impl SessionData {
    /// Create new session data
    pub fn new(
        key: String,
        document_path: Option<PathBuf>,
        theme: String,
        editor_theme: String,
        read_only: bool,
    ) -> Self {
        SessionData {
            key,
            document_path,
            theme,
            editor_theme,
            read_only,
            polo_pid: None,
            created_at: SystemTime::now(),
        }
    }

    /// Convert to SessionInfo for API response
    pub fn to_session_info(&self) -> SessionInfo {
        SessionInfo {
            document_path: self
                .document_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            theme: self.theme.clone(),
            editor_theme: self.editor_theme.clone(),
            read_only: self.read_only,
        }
    }
}

/// Thread-safe session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a new cryptographically secure session key
    pub fn generate_session_key() -> String {
        Uuid::new_v4().to_string()
    }

    /// Create a new session and return the key
    ///
    /// # Security Note
    ///
    /// The session key is never logged. Use `[REDACTED]` in log messages.
    pub fn create_session(&self, data: SessionData) -> String {
        let key = data.key.clone();
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(key.clone(), data);
        info!("Session created (key: [REDACTED], total sessions: {})", sessions.len());
        key
    }

    /// Get session data by key
    pub fn get_session(&self, key: &str) -> Option<SessionData> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(key).cloned()
    }

    /// Validate that a session key exists
    pub fn validate_session(&self, key: &str) -> bool {
        let sessions = self.sessions.read().unwrap();
        let exists = sessions.contains_key(key);
        if !exists {
            warn!("Invalid session key attempt (key: [REDACTED])");
        }
        exists
    }

    /// Get session info for API response
    pub fn get_session_info(&self, key: &str) -> Option<SessionInfo> {
        self.get_session(key).map(|data| data.to_session_info())
    }

    /// Update Polo PID for a session
    pub fn set_polo_pid(&self, key: &str, pid: u32) {
        let mut sessions = self.sessions.write().unwrap();
        if let Some(session) = sessions.get_mut(key) {
            session.polo_pid = Some(pid);
            info!("Updated session with Polo PID: {} (key: [REDACTED])", pid);
        }
    }

    /// Remove a session
    pub fn remove_session(&self, key: &str) -> Option<SessionData> {
        let mut sessions = self.sessions.write().unwrap();
        let removed = sessions.remove(key);
        if removed.is_some() {
            info!("Session closed (key: [REDACTED], remaining: {})", sessions.len());
        }
        removed
    }

    /// Get all session keys (for debugging/cleanup)
    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().unwrap();
        sessions.keys().cloned().collect()
    }

    /// Remove all sessions (called on Marco shutdown)
    pub fn clear_all_sessions(&self) {
        let mut sessions = self.sessions.write().unwrap();
        let count = sessions.len();
        sessions.clear();
        if count > 0 {
            info!("Cleared all {} sessions on shutdown", count);
        }
    }

    /// Get total number of active sessions
    pub fn session_count(&self) -> usize {
        let sessions = self.sessions.read().unwrap();
        sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test_session_key_generation() {
        let key1 = SessionManager::generate_session_key();
        let key2 = SessionManager::generate_session_key();

        // Keys should be unique
        assert_ne!(key1, key2);

        // Keys should be valid UUIDs (36 characters with hyphens)
        assert_eq!(key1.len(), 36);
        assert_eq!(key2.len(), 36);

        // Should contain hyphens at expected positions
        assert_eq!(key1.chars().nth(8), Some('-'));
        assert_eq!(key1.chars().nth(13), Some('-'));
    }

    #[test]
    fn smoke_test_session_creation() {
        let manager = SessionManager::new();

        let key = SessionManager::generate_session_key();
        let data = SessionData::new(
            key.clone(),
            Some(PathBuf::from("/test/doc.md")),
            "github.css".to_string(),
            "marco-dark".to_string(),
            true,
        );

        let created_key = manager.create_session(data);
        assert_eq!(created_key, key);

        // Should be able to retrieve session
        let retrieved = manager.get_session(&key);
        assert!(retrieved.is_some());

        let session = retrieved.unwrap();
        assert_eq!(session.theme, "github.css");
        assert_eq!(session.editor_theme, "marco-dark");
        assert_eq!(session.read_only, true);
    }

    #[test]
    fn smoke_test_session_validation() {
        let manager = SessionManager::new();

        let key = SessionManager::generate_session_key();
        let data = SessionData::new(
            key.clone(),
            None,
            "theme.css".to_string(),
            "theme".to_string(),
            true,
        );

        manager.create_session(data);

        // Valid key should pass
        assert!(manager.validate_session(&key));

        // Invalid key should fail
        assert!(!manager.validate_session("invalid-key"));
    }

    #[test]
    fn smoke_test_session_info_conversion() {
        let manager = SessionManager::new();

        let key = SessionManager::generate_session_key();
        let data = SessionData::new(
            key.clone(),
            Some(PathBuf::from("/test.md")),
            "github.css".to_string(),
            "marco-light".to_string(),
            false,
        );

        manager.create_session(data);

        let info = manager.get_session_info(&key).unwrap();
        assert_eq!(info.document_path, Some("/test.md".to_string()));
        assert_eq!(info.theme, "github.css");
        assert_eq!(info.editor_theme, "marco-light");
        assert_eq!(info.read_only, false);
    }

    #[test]
    fn smoke_test_session_removal() {
        let manager = SessionManager::new();

        let key = SessionManager::generate_session_key();
        let data = SessionData::new(
            key.clone(),
            None,
            "theme".to_string(),
            "scheme".to_string(),
            true,
        );

        manager.create_session(data);
        assert_eq!(manager.session_count(), 1);

        // Remove session
        let removed = manager.remove_session(&key);
        assert!(removed.is_some());
        assert_eq!(manager.session_count(), 0);

        // Should no longer validate
        assert!(!manager.validate_session(&key));
    }

    #[test]
    fn smoke_test_multiple_sessions() {
        let manager = SessionManager::new();

        let key1 = SessionManager::generate_session_key();
        let key2 = SessionManager::generate_session_key();

        let data1 = SessionData::new(
            key1.clone(),
            Some(PathBuf::from("/doc1.md")),
            "theme1.css".to_string(),
            "scheme1".to_string(),
            true,
        );
        let data2 = SessionData::new(
            key2.clone(),
            Some(PathBuf::from("/doc2.md")),
            "theme2.css".to_string(),
            "scheme2".to_string(),
            false,
        );

        manager.create_session(data1);
        manager.create_session(data2);

        assert_eq!(manager.session_count(), 2);

        // Both should validate
        assert!(manager.validate_session(&key1));
        assert!(manager.validate_session(&key2));

        // Clear all
        manager.clear_all_sessions();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn smoke_test_polo_pid_update() {
        let manager = SessionManager::new();

        let key = SessionManager::generate_session_key();
        let data = SessionData::new(
            key.clone(),
            None,
            "theme".to_string(),
            "scheme".to_string(),
            true,
        );

        manager.create_session(data);

        // Initially no PID
        let session = manager.get_session(&key).unwrap();
        assert_eq!(session.polo_pid, None);

        // Set PID
        manager.set_polo_pid(&key, 12345);

        // Should now have PID
        let session = manager.get_session(&key).unwrap();
        assert_eq!(session.polo_pid, Some(12345));
    }
}
