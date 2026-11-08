use std::path::Path;

use alaydriem_quirks::QuirksMod;
use alaydriem_quirks::services::{HookInstaller, QuirksRuntime};
use amm_client::test_support::FakeTable;

use crate::support::fake_host::FakeHost;

fn runtime() -> QuirksRuntime {
    QuirksRuntime::new(Path::new(FakeTable::CONFIG_DIR), Box::new(|| false))
}

#[test]
fn missing_route_entries_turn_off_only_the_screen_block() {
    let host = FakeHost::new(&[(QuirksMod::UPDATE, 0x1000)]);

    let result = HookInstaller::install(&host, &runtime(), &QuirksMod::hooks());

    assert!(result.is_ok());
    assert_eq!(host.hooked(), [QuirksMod::UPDATE]);
    let warnings = host.warnings();
    assert!(!warnings.is_empty());
    assert!(warnings.iter().all(|line| line.contains("the idle screen block")));
}

#[test]
fn missing_update_entry_fails_enable() {
    let host = FakeHost::new(&[(QuirksMod::PUSH_ROUTE, 0x2000), (QuirksMod::REPLACE_ROUTE, 0x3000)]);

    let result = QuirksMod::enable_with(&host, &runtime());

    assert!(result.is_err());
    assert!(host.hooked().is_empty());
}
