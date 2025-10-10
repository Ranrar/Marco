# Marco ↔ Polo IPC Architecture

## Overview

This document describes the IPC (Inter-Process Communication) architecture for secure, real-time communication between Marco (editor) and Polo (viewer) using cross-platform Named Pipes and Unix Domain Sockets.

---

## Technology Stack

### IPC Layer: `interprocess` Crate

* **Crate:** [`interprocess`](https://crates.io/crates/interprocess)
* **Version:** `2.x`
* **Platform Support:**
  * **Linux/macOS:** Unix Domain Sockets
  * **Windows:** Named Pipes
  * **API:** Unified cross-platform interface

### Why `interprocess`?

| Feature | Benefit |
|---------|---------|
| **Cross-platform** | Single codebase for Linux, macOS, Windows |
| **Local-only** | No network exposure, no TCP/IP stack |
| **Fast** | Lower latency than HTTP (no protocol overhead) |
| **Secure** | OS-level permissions, no public access |
| **Stream-based** | Supports bidirectional communication |
| **No dependencies** | Lightweight, no async runtime required |

### Message Format: RON (Rusty Object Notation)

* **Crate:** `ron` (already in workspace)
* **Why RON:**
  * Human-readable for debugging
  * Native Rust syntax
  * Excellent Serde integration
  * Compact compared to JSON
  * Type-safe serialization

### Session Keys: UUID v4

* **Crate:** `uuid` with `v4` feature
* **Purpose:** Cryptographically secure session authentication
* **Lifetime:** Ephemeral (generated per Polo launch, discarded on exit)

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Marco (Editor)                          │
│                                                                 │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐ │
│  │   Layout     │      │     IPC      │      │   Session    │ │
│  │   Manager    │─────▶│   Server     │◀─────│   Manager    │ │
│  └──────────────┘      └──────────────┘      └──────────────┘ │
│         │                      │                      │        │
│         │                      │                      │        │
│         │              Named Pipe (Windows)           │        │
│         │              Unix Socket (Linux/macOS)      │        │
│         │                      │                      │        │
│         ▼                      ▼                      ▼        │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 │ IPC Connection
                                 │ (RON messages)
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                          Polo (Viewer)                          │
│                                                                 │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────────┐ │
│  │   Startup    │─────▶│     IPC      │◀─────│   WebView    │ │
│  │   Handler    │      │   Client     │      │   Renderer   │ │
│  └──────────────┘      └──────────────┘      └──────────────┘ │
│                                │                               │
│                                │                               │
│                        Receives Commands:                      │
│                        • GetSession                            │
│                        • UpdateTheme                           │
│                        • RefreshView                           │
│                        • CloseSession                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## IPC Implementation Details

### Server Side (Marco)

**Location:** `marco/src/components/api_server.rs`

```rust
use interprocess::local_socket::{LocalSocketListener, LocalSocketStream};
use std::io::{BufRead, BufReader, Write};

pub struct IpcServer {
    listener: LocalSocketListener,
    session_manager: Arc<SessionManager>,
}

impl IpcServer {
    pub fn new(socket_name: &str, session_manager: Arc<SessionManager>) -> Result<Self> {
        // On Unix: creates socket at /tmp/marco-{uuid}.sock
        // On Windows: creates pipe at \\.\pipe\marco-{uuid}
        let listener = LocalSocketListener::bind(socket_name)?;
        
        Ok(IpcServer {
            listener,
            session_manager,
        })
    }
    
    pub fn accept_connection(&self) -> Result<()> {
        let stream = self.listener.accept()?;
        self.handle_client(stream)?;
        Ok(())
    }
    
    fn handle_client(&self, stream: LocalSocketStream) -> Result<()> {
        let mut reader = BufReader::new(&stream);
        let mut writer = stream.try_clone()?;
        
        // Read RON message
        let mut line = String::new();
        reader.read_line(&mut line)?;
        
        // Deserialize RON request
        let request: ApiRequest = ron::from_str(&line)?;
        
        // Validate session and handle command
        let response = self.handle_request(request);
        
        // Serialize RON response
        let response_ron = ron::to_string(&response)?;
        writeln!(writer, "{}", response_ron)?;
        
        Ok(())
    }
}
```

### Client Side (Polo)

**Location:** `polo/src/components/api_client.rs`

```rust
use interprocess::local_socket::LocalSocketStream;
use std::io::{BufRead, BufReader, Write};

pub struct IpcClient {
    stream: LocalSocketStream,
}

impl IpcClient {
    pub fn connect(socket_name: &str) -> Result<Self> {
        let stream = LocalSocketStream::connect(socket_name)?;
        Ok(IpcClient { stream })
    }
    
    pub fn send_request(&mut self, request: ApiRequest) -> Result<ApiResponse> {
        // Serialize request to RON
        let request_ron = ron::to_string(&request)?;
        
        // Send to Marco
        writeln!(self.stream, "{}", request_ron)?;
        
        // Read response
        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        
        // Deserialize RON response
        let response: ApiResponse = ron::from_str(&line)?;
        Ok(response)
    }
    
    pub fn fetch_session(&mut self, session_key: String) -> Result<SessionInfo> {
        let request = ApiRequest::GetSession { session_key };
        let response = self.send_request(request)?;
        
        match response {
            ApiResponse::SessionData(info) => Ok(info),
            ApiResponse::Error { message } => Err(anyhow::anyhow!(message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }
}
```

---

## Data Structures (RON-Serializable)

**Location:** `marco_core/src/logic/api.rs`

```rust
use serde::{Deserialize, Serialize};

/// Request from Polo → Marco
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiRequest {
    /// Get session information
    GetSession { 
        session_key: String 
    },
    
    /// Request to refresh the current view
    RefreshView { 
        session_key: String 
    },
    
    /// Notify Marco that Polo is closing
    CloseSession { 
        session_key: String 
    },
}

/// Response from Marco → Polo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiResponse {
    /// Session data returned
    SessionData(SessionInfo),
    
    /// Command succeeded
    Success,
    
    /// Error occurred
    Error { 
        message: String 
    },
}

/// Session information shared with Polo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Path to the markdown document
    pub document_path: Option<String>,
    
    /// HTML preview theme (e.g., "github.css")
    pub theme: String,
    
    /// Editor theme (e.g., "marco-dark")
    pub editor_theme: String,
    
    /// Whether Polo should be read-only
    pub read_only: bool,
}

/// Server-to-client push notifications (future use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiNotification {
    /// Theme changed in Marco
    ThemeUpdated { 
        theme: String,
        editor_theme: String,
    },
    
    /// Document content changed
    DocumentModified,
    
    /// Marco is shutting down
    ServerClosing,
}
```

---

## Session Management

**Location:** `marco_core/src/logic/api.rs`

```rust
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use uuid::Uuid;

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

pub struct SessionData {
    /// Unique session key (UUID v4)
    pub key: String,
    
    /// Document being edited
    pub document_path: Option<PathBuf>,
    
    /// Current HTML preview theme
    pub theme: String,
    
    /// Current editor theme
    pub editor_theme: String,
    
    /// Read-only flag
    pub read_only: bool,
    
    /// Polo process ID (for monitoring)
    pub polo_pid: Option<u32>,
    
    /// Session creation time
    pub created_at: SystemTime,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Generate a new session key
    pub fn generate_session_key() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Create a new session
    pub fn create_session(&self, data: SessionData) -> String {
        let key = data.key.clone();
        let mut sessions = self.sessions.write().unwrap();
        sessions.insert(key.clone(), data);
        log::info!("Session created (key: [REDACTED])");
        key
    }
    
    /// Get session data
    pub fn get_session(&self, key: &str) -> Option<SessionData> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(key).cloned()
    }
    
    /// Validate session key
    pub fn validate_session(&self, key: &str) -> bool {
        let sessions = self.sessions.read().unwrap();
        sessions.contains_key(key)
    }
    
    /// Remove a session
    pub fn remove_session(&self, key: &str) {
        let mut sessions = self.sessions.write().unwrap();
        if sessions.remove(key).is_some() {
            log::info!("Session closed (key: [REDACTED])");
        }
    }
    
    /// Get session info for API response
    pub fn get_session_info(&self, key: &str) -> Option<SessionInfo> {
        self.get_session(key).map(|data| SessionInfo {
            document_path: data.document_path.map(|p| p.to_string_lossy().to_string()),
            theme: data.theme,
            editor_theme: data.editor_theme,
            read_only: data.read_only,
        })
    }
}
```

---

## Launch Flow

### 1. Marco Prepares Session

**File:** `marco/src/components/layout_manager.rs`

```rust
fn apply_editor_and_view_separate(&self) {
    // Generate unique session key
    let session_key = SessionManager::generate_session_key();
    
    // Create IPC socket name (platform-specific)
    let socket_name = if cfg!(windows) {
        format!(r"\\.\pipe\marco-{}", session_key)
    } else {
        format!("/tmp/marco-{}.sock", session_key)
    };
    
    // Create session data
    let session_data = SessionData {
        key: session_key.clone(),
        document_path: self.current_document.borrow().clone(),
        theme: self.get_current_theme(),
        editor_theme: self.get_current_editor_theme(),
        read_only: false,
        polo_pid: None,
        created_at: SystemTime::now(),
    };
    
    // Register session
    self.session_manager.create_session(session_data);
    
    // Start IPC server
    let server = IpcServer::new(&socket_name, self.session_manager.clone())?;
    self.ipc_server.replace(Some(server));
    
    // Launch Polo with session info
    self.launch_polo(&socket_name, &session_key)?;
}
```

### 2. Marco Launches Polo

```rust
fn launch_polo(&self, socket_name: &str, session_key: &str) -> Result<()> {
    let polo_path = self.find_polo_binary()?;
    
    let child = Command::new(&polo_path)
        .arg("--session")
        .arg(session_key)
        .arg("--socket")
        .arg(socket_name)
        .spawn()?;
    
    let child_id = child.id();
    log::info!("Polo launched (PID: {}, session: [REDACTED])", child_id);
    
    // Update session with PID
    if let Some(mut session) = self.session_manager.get_session(session_key) {
        session.polo_pid = Some(child_id);
    }
    
    *self.polo_process.borrow_mut() = Some(child);
    Ok(())
}
```

### 3. Polo Connects and Authenticates

**File:** `polo/src/main.rs`

```rust
fn main() {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    let session_key = parse_arg(&args, "--session")?;
    let socket_name = parse_arg(&args, "--socket")?;
    
    // Connect to Marco's IPC server
    let mut client = IpcClient::connect(&socket_name)?;
    
    // Fetch session data
    let session_info = client.fetch_session(session_key.clone())?;
    
    log::info!("Connected to Marco (session: [REDACTED])");
    
    // Load document from session
    if let Some(path) = session_info.document_path {
        load_and_render_markdown(&webview, &path, &session_info.theme);
    }
    
    // Apply themes
    apply_theme(&session_info.theme, &session_info.editor_theme);
    
    // Store client for future commands
    store_ipc_client(client);
}
```

### 4. Polo Sends Close Notification

```rust
fn on_window_close() {
    // Notify Marco before exiting
    if let Some(mut client) = get_ipc_client() {
        let request = ApiRequest::CloseSession {
            session_key: get_session_key(),
        };
        let _ = client.send_request(request);
    }
    
    log::info!("Sent CloseSession to Marco");
}
```

---

## Command Implementations

### GetSession (Polo → Marco)

**Purpose:** Fetch session data on Polo startup

**Flow:**
1. Polo sends `ApiRequest::GetSession { session_key }`
2. Marco validates session key
3. Marco returns `ApiResponse::SessionData(SessionInfo)` with document path and themes
4. Polo loads document and applies themes

**RON Example:**

Request:
```ron
GetSession(
    session_key: "a7f3c9e1-4b2d-4f8a-9c3d-1e5f6a7b8c9d"
)
```

Response:
```ron
SessionData((
    document_path: Some("/home/user/docs/README.md"),
    theme: "github.css",
    editor_theme: "marco-dark",
    read_only: false,
))
```

---

### UpdateTheme (Marco → Polo)

**Purpose:** Synchronize theme changes from Marco to Polo

**Flow:**
1. User changes theme in Marco
2. Marco sends `ApiNotification::ThemeUpdated` to Polo via IPC
3. Polo receives notification
4. Polo applies new theme to WebView

**Implementation Note:** Requires bidirectional communication or periodic polling

---

### RefreshView (Polo → Marco)

**Purpose:** Request document refresh (e.g., after external file change)

**Flow:**
1. Polo detects file change or user triggers refresh
2. Polo sends `ApiRequest::RefreshView { session_key }`
3. Marco re-parses document
4. Marco sends updated HTML to Polo (future enhancement)

---

### CloseSession (Polo → Marco)

**Purpose:** Clean shutdown notification

**Flow:**
1. Polo window closes
2. Polo sends `ApiRequest::CloseSession { session_key }`
3. Marco validates session
4. Marco removes session from SessionManager
5. Marco closes IPC listener for that session
6. Marco returns `ApiResponse::Success`

**RON Example:**

Request:
```ron
CloseSession(
    session_key: "a7f3c9e1-4b2d-4f8a-9c3d-1e5f6a7b8c9d"
)
```

Response:
```ron
Success
```

---

## Security Model

### Session Key Properties

| Property | Implementation |
|----------|----------------|
| **Generation** | `uuid::Uuid::new_v4()` (cryptographically secure) |
| **Lifetime** | Ephemeral (per Polo launch) |
| **Storage** | In-memory only (never persisted) |
| **Transmission** | Via command-line args (OS protects process arguments) |
| **Validation** | Every API request checks `SessionManager` |
| **Logging** | Always redacted (`[REDACTED]` in logs) |

### IPC Security

| Aspect | Protection |
|--------|------------|
| **Linux/macOS** | Unix socket with file permissions (700) |
| **Windows** | Named pipe with restricted ACLs |
| **Network Exposure** | None (local-only) |
| **Authentication** | Session key required for all operations |
| **Lifetime** | Socket/pipe deleted on Marco shutdown |

### Threat Model

| Threat | Mitigation |
|--------|-----------|
| **Unauthorized access** | Session key validation on every request |
| **Session hijacking** | Short-lived keys, in-memory storage |
| **Man-in-the-middle** | Not applicable (local IPC, no network) |
| **Key leakage** | Never logged, only in command-line args |
| **Denial of service** | Limited to local user (OS-level protection) |

---

## Error Handling

### Connection Errors

```rust
// Marco side
match listener.accept() {
    Ok(stream) => self.handle_client(stream),
    Err(e) => {
        log::error!("IPC accept failed: {}", e);
        // Continue listening for other connections
    }
}

// Polo side
match IpcClient::connect(socket_name) {
    Ok(client) => { /* proceed */ },
    Err(e) => {
        log::error!("Failed to connect to Marco: {}", e);
        show_error_dialog("Cannot connect to Marco editor");
        std::process::exit(1);
    }
}
```

### Invalid Session

```rust
// Marco side
fn handle_request(&self, request: ApiRequest) -> ApiResponse {
    let session_key = match &request {
        ApiRequest::GetSession { session_key } => session_key,
        ApiRequest::RefreshView { session_key } => session_key,
        ApiRequest::CloseSession { session_key } => session_key,
    };
    
    if !self.session_manager.validate_session(session_key) {
        log::warn!("Invalid session key attempt");
        return ApiResponse::Error {
            message: "Invalid session".to_string()
        };
    }
    
    // Handle valid request...
}
```

### RON Parsing Errors

```rust
match ron::from_str::<ApiRequest>(&line) {
    Ok(request) => self.handle_request(request),
    Err(e) => {
        log::error!("Failed to parse RON request: {}", e);
        ApiResponse::Error {
            message: "Malformed request".to_string()
        }
    }
}
```

---

## Testing Strategy

### Unit Tests

**File:** `marco_core/src/logic/api.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_session_manager() {
        let manager = SessionManager::new();
        
        // Create session
        let key = SessionManager::generate_session_key();
        let data = SessionData {
            key: key.clone(),
            document_path: Some(PathBuf::from("/test.md")),
            theme: "github.css".to_string(),
            editor_theme: "marco-dark".to_string(),
            read_only: false,
            polo_pid: Some(1234),
            created_at: SystemTime::now(),
        };
        
        manager.create_session(data);
        
        // Validate
        assert!(manager.validate_session(&key));
        
        // Get info
        let info = manager.get_session_info(&key).unwrap();
        assert_eq!(info.theme, "github.css");
        
        // Remove
        manager.remove_session(&key);
        assert!(!manager.validate_session(&key));
    }
    
    #[test]
    fn smoke_test_ron_serialization() {
        // Test request
        let request = ApiRequest::GetSession {
            session_key: "test-key".to_string()
        };
        let ron = ron::to_string(&request).unwrap();
        let parsed: ApiRequest = ron::from_str(&ron).unwrap();
        
        // Test response
        let response = ApiResponse::SessionData(SessionInfo {
            document_path: Some("/test.md".to_string()),
            theme: "github.css".to_string(),
            editor_theme: "marco-dark".to_string(),
            read_only: false,
        });
        let ron = ron::to_string(&response).unwrap();
        let parsed: ApiResponse = ron::from_str(&ron).unwrap();
    }
}
```

### Integration Tests

**File:** `tests/integration/ipc_test.rs`

```rust
#[test]
fn test_marco_polo_ipc_communication() {
    // Start Marco IPC server
    let session_manager = Arc::new(SessionManager::new());
    let socket_name = "/tmp/marco-test.sock";
    let server = IpcServer::new(socket_name, session_manager.clone()).unwrap();
    
    // Create session
    let session_key = SessionManager::generate_session_key();
    let session_data = SessionData {
        key: session_key.clone(),
        document_path: Some(PathBuf::from("/test.md")),
        theme: "github.css".to_string(),
        editor_theme: "marco-dark".to_string(),
        read_only: false,
        polo_pid: None,
        created_at: SystemTime::now(),
    };
    session_manager.create_session(session_data);
    
    // Spawn server thread
    let server_thread = std::thread::spawn(move || {
        server.accept_connection().unwrap();
    });
    
    // Connect as Polo client
    std::thread::sleep(Duration::from_millis(100));
    let mut client = IpcClient::connect(socket_name).unwrap();
    
    // Fetch session
    let info = client.fetch_session(session_key.clone()).unwrap();
    assert_eq!(info.theme, "github.css");
    assert_eq!(info.document_path, Some("/test.md".to_string()));
    
    server_thread.join().unwrap();
}
```

---

## Dependencies Summary

### Workspace `Cargo.toml` Additions

```toml
[workspace.dependencies]
interprocess = "2.2"
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### `marco_core/Cargo.toml`

```toml
[dependencies]
interprocess = { workspace = true }
uuid = { workspace = true }
ron = { workspace = true }
serde = { workspace = true }
```

### `marco/Cargo.toml`

```toml
[dependencies]
marco_core = { path = "../marco_core" }
interprocess = { workspace = true }
```

### `polo/Cargo.toml`

```toml
[dependencies]
marco_core = { path = "../marco_core" }
interprocess = { workspace = true }
```

---

## Advantages Over HTTP

| Aspect | HTTP (localhost) | IPC (interprocess) |
|--------|------------------|--------------------|
| **Latency** | ~1-2ms | <0.5ms |
| **Port conflicts** | Possible | None |
| **Dependencies** | HTTP server crate (tiny_http/hyper) | Single `interprocess` crate |
| **Network exposure** | Localhost only (requires binding config) | None (OS-level local) |
| **Debugging** | Can use curl/browser | Requires custom tools |
| **Cross-platform** | Yes | Yes (unified API) |
| **Security** | Requires careful binding | OS-level permissions |

---

## Implementation Checklist

- [ ] Add `interprocess` and `uuid` to workspace dependencies
- [ ] Define RON data structures in `marco_core/src/logic/api.rs`
- [ ] Implement `SessionManager` in `marco_core`
- [ ] Create `marco/src/components/api_server.rs` (IPC server)
- [ ] Create `polo/src/components/api_client.rs` (IPC client)
- [ ] Integrate IPC server with `layout_manager.rs`
- [ ] Modify Polo startup to accept `--session` and `--socket` args
- [ ] Implement `GetSession` command
- [ ] Implement `CloseSession` command
- [ ] Implement `RefreshView` command (optional)
- [ ] Implement `UpdateTheme` notification (optional)
- [ ] Add smoke tests for all components
- [ ] Add integration tests for end-to-end flow
- [ ] Update logging to redact session keys
- [ ] Test on Linux and Windows (cross-platform validation)
- [ ] Document API in user guide

---

## Future Enhancements

### Bidirectional Push Notifications

Currently, communication is request-response only. For real-time theme updates, consider:

1. **Separate notification channel:** Marco opens a second IPC connection for push messages
2. **Polling:** Polo periodically checks for updates (simpler, higher latency)
3. **Async IPC:** Use `interprocess` async API with Tokio (more complex)

### Multiple Polo Instances

Support multiple viewer windows:

- Marco maintains multiple sessions (one per Polo)
- Each Polo gets unique session key and socket name
- SessionManager tracks all active sessions
- Theme updates broadcast to all connected Polo instances

### File Watching Integration

Automatically refresh Polo when document changes on disk:

- Marco uses `notify` crate to watch open file
- On change, Marco sends `DocumentModified` notification to Polo
- Polo triggers refresh automatically

---

## Conclusion

The `interprocess`-based IPC architecture provides:

✅ **Cross-platform** support (Linux, macOS, Windows)  
✅ **Secure** local-only communication  
✅ **Fast** low-latency message passing  
✅ **Simple** RON-based serialization  
✅ **Lightweight** minimal dependencies  

This design aligns with Marco's architectural principles: pure Rust, minimal dependencies, and platform-agnostic implementation.

---

**Document Version:** 1.0  
**Date:** October 10, 2025  
**Author:** Marco Development Team  
**Status:** Architecture Design (Ready for Implementation)
