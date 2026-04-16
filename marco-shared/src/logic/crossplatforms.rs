//! Cross-platform OS helpers for Marco

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_variants() {
        let _ = Platform::Linux;
        let _ = Platform::Windows;
        let _ = Platform::Unknown;
    }

    #[test]
    fn test_detect_platform_and_dark_mode() {
        let _ = detect_platform();
        let _ = is_dark_mode_supported();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Windows,
    Unknown,
}

pub fn detect_platform() -> Platform {
    if cfg!(target_os = "linux") {
        Platform::Linux
    } else if cfg!(target_os = "windows") {
        Platform::Windows
    } else {
        Platform::Unknown
    }
}

pub fn is_dark_mode_supported() -> bool {
    matches!(
        detect_platform(),
        Platform::Linux | Platform::Windows
    )
}
