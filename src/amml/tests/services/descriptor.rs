use alaydriem_quirks::services::{QuirksDescriptor, SettingsReader};
use amm_core::services::DescriptorChecker;
use amm_core::services::settings::SchemaChecker;
use amm_core::structs::settings::HclDocument;
use quirks_core::config::QuirksConfig;

#[test]
fn descriptor_passes_manager_checker() {
    let d = DescriptorChecker::new(128, 256).parse(QuirksDescriptor::TEXT).unwrap();

    assert_eq!(d.id(), "@alaydriem/quirks");
    assert!(d.claims.is_empty());
    assert!(d.keybinds.is_empty());
}

#[test]
fn defaults_match_spec() {
    let d = DescriptorChecker::new(128, 256).parse(QuirksDescriptor::TEXT).unwrap();
    let checked = SchemaChecker::check(&d.settings, HclDocument::empty());

    assert_eq!(SettingsReader::read(&checked.document.text()), QuirksConfig::default());
}
