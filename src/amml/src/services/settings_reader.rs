use quirks_core::config::QuirksConfig;
use serde::Deserialize;

// the part of config.hcl the mod reads; a missing value keeps its default
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RawSettings {
    idle: RawIdle,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawIdle {
    enabled: Option<bool>,
}

// turns the checked config.hcl into an QuirksConfig
pub struct SettingsReader;

impl SettingsReader {
    // text that cannot be read gives the defaults, with one warning
    pub fn read(text: &str) -> QuirksConfig {
        match hcl::from_str::<RawSettings>(text) {
            Ok(raw) => Self::convert(&raw),
            Err(error) => {
                curia::warn!("config could not be read; using the defaults", { error: error.to_string() });
                QuirksConfig::default()
            }
        }
    }

    pub fn convert(raw: &RawSettings) -> QuirksConfig {
        let defaults = QuirksConfig::default();
        QuirksConfig { idle_enabled: raw.idle.enabled.unwrap_or(defaults.idle_enabled) }
    }
}
