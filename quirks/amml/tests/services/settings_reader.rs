use alaydriem_quirks::services::SettingsReader;
use quirks_core::config::QuirksConfig;

#[test]
fn idle_off_is_read() {
    assert!(!SettingsReader::read("idle { enabled = false }").idle_enabled);
}

#[test]
fn missing_value_keeps_the_default() {
    assert_eq!(SettingsReader::read("idle {}"), QuirksConfig::default());
}

#[test]
fn unreadable_text_gives_defaults() {
    assert_eq!(SettingsReader::read("idle = {"), QuirksConfig::default());
}
