//! Robust settings loader/saver using RON and Serde
use crate::logic::settings_struct::Settings;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

// AppearanceSettings is now a sub-struct of Settings

pub fn load_settings<P: AsRef<Path>>(settings_path: P) -> Result<Settings> {
    let contents = fs::read_to_string(&settings_path)
        .with_context(|| format!("Failed to read settings file: {:?}", settings_path.as_ref()))?;
    let settings: Settings = ron::from_str(&contents)
        .with_context(|| "Failed to parse settings.ron as RON")?;
    Ok(settings)
}

pub fn save_settings<P: AsRef<Path>>(settings_path: P, settings: &Settings) -> Result<()> {
    let ron = ron::ser::to_string_pretty(settings, ron::ser::PrettyConfig::default())
        .context("Failed to serialize settings to RON")?;
    fs::write(&settings_path, ron)
        .with_context(|| format!("Failed to write settings file: {:?}", settings_path.as_ref()))?;
    Ok(())
}
