//! Simple View Key Generation and Validation
//!
//! Provides secure key generation for Polo's simple view mode.
//! Keys are time-limited tokens that grant access to the minimal UI.

use std::time::{SystemTime, UNIX_EPOCH};

/// Secret salt for key generation (in production, this should be more secure)
const SECRET_SALT: &str = "marco_polo_simple_view_2025";

/// Key validity duration in seconds (5 minutes)
const KEY_VALIDITY_SECONDS: u64 = 300;

/// Generate a simple view key valid for the current time window
///
/// The key is generated using a combination of:
/// - Current time (rounded to nearest time window)
/// - Secret salt
/// - Simple hash function
///
/// # Returns
///
/// A hex string representing the key (e.g., "a3f5c9d2e1b4")
pub fn generate_simple_view_key() -> String {
    let timestamp = get_current_time_window();
    let data = format!("{}{}", timestamp, SECRET_SALT);
    simple_hash(&data)
}

/// Validate a simple view key
///
/// Checks if the key matches the current time window or the previous one
/// (allows for slight clock skew and key reuse within validity period)
///
/// # Arguments
///
/// * `key` - The key to validate
///
/// # Returns
///
/// `true` if the key is valid, `false` otherwise
pub fn validate_simple_view_key(key: &str) -> bool {
    // Check current time window
    let current_timestamp = get_current_time_window();
    let current_key = simple_hash(&format!("{}{}", current_timestamp, SECRET_SALT));
    
    if key == current_key {
        return true;
    }
    
    // Check previous time window (for clock skew tolerance)
    if current_timestamp >= KEY_VALIDITY_SECONDS {
        let previous_timestamp = current_timestamp - KEY_VALIDITY_SECONDS;
        let previous_key = simple_hash(&format!("{}{}", previous_timestamp, SECRET_SALT));
        
        if key == previous_key {
            return true;
        }
    }
    
    false
}

/// Get current time rounded to the nearest time window
fn get_current_time_window() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Round to nearest KEY_VALIDITY_SECONDS
    (now / KEY_VALIDITY_SECONDS) * KEY_VALIDITY_SECONDS
}

/// Simple hash function for key generation
///
/// This is a basic hash for demonstration. In production, use a proper
/// HMAC or cryptographic hash function.
fn simple_hash(data: &str) -> String {
    let mut hash: u64 = 5381;
    
    for byte in data.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    
    format!("{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn smoke_test_key_generation() {
        let key1 = generate_simple_view_key();
        let key2 = generate_simple_view_key();
        
        // Keys should be consistent within the same time window
        assert_eq!(key1, key2);
        assert!(!key1.is_empty());
        
        // Key should be valid
        assert!(validate_simple_view_key(&key1));
    }
    
    #[test]
    fn smoke_test_key_validation() {
        let valid_key = generate_simple_view_key();
        let invalid_key = "invalid123";
        
        assert!(validate_simple_view_key(&valid_key));
        assert!(!validate_simple_view_key(invalid_key));
    }
    
    #[test]
    fn smoke_test_time_window() {
        let window1 = get_current_time_window();
        let window2 = get_current_time_window();
        
        // Windows should be consistent
        assert_eq!(window1, window2);
        
        // Window should be divisible by validity period
        assert_eq!(window1 % KEY_VALIDITY_SECONDS, 0);
    }
}
