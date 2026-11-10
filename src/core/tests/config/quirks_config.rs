use quirks_core::config::QuirksConfig;

#[test]
fn config_default_has_idle_on() {
    assert!(QuirksConfig::default().idle_enabled);
}
