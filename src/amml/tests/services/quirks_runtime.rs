use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alaydriem_quirks::services::QuirksRuntime;
use alaydriem_quirks::types::LinkState;
use amm_client::test_support::FakeTable;
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use amml_sdk::HookHandle;

const ROUTE: &[u8] = b"/idle-route";

fn runtime() -> QuirksRuntime {
    let fired = Arc::new(AtomicUsize::new(0));
    QuirksRuntime::new(
        Path::new(FakeTable::CONFIG_DIR),
        Box::new(move || {
            fired.fetch_add(1, Ordering::SeqCst);
            true
        }),
    )
}

fn settings(enabled: bool) {
    FakeTable::set_config(&format!("idle {{\n  enabled = {enabled}\n}}\n"));
}

fn changed() -> ModEvent {
    ModEvent::SettingsChanged { paths: 1 }
}

fn attached(enabled: bool) -> QuirksRuntime {
    settings(enabled);
    let runtime = runtime();
    runtime.enable_link();
    runtime.tick_with(|| Ok(FakeTable::table()));
    assert_eq!(runtime.link_state(), LinkState::Attached);
    runtime
}

#[test]
fn idle_is_on_without_a_manager() {
    let runtime = runtime();
    runtime.enable_link();

    runtime.tick_with(|| Err(LinkError::ManagerMissing));

    assert!(runtime.idle().is_on());
    assert_eq!(runtime.link_state(), LinkState::Off);
}

#[test]
fn the_saved_setting_is_applied_on_attach() {
    let runtime = attached(false);

    assert!(!runtime.idle().is_on());
}

#[test]
fn a_settings_change_is_applied_on_the_next_tick() {
    let runtime = attached(true);
    assert!(runtime.idle().is_on());

    settings(false);
    FakeTable::push_event(changed());
    runtime.tick_with(|| Err(LinkError::ManagerMissing));
    assert!(!runtime.idle().is_on());

    settings(true);
    FakeTable::push_event(changed());
    runtime.tick_with(|| Err(LinkError::ManagerMissing));
    assert!(runtime.idle().is_on());
}

#[test]
fn manager_disable_turns_idle_off_and_enable_restores_it() {
    let runtime = attached(true);

    FakeTable::push_event(ModEvent::Disabled);
    runtime.tick_with(|| Err(LinkError::ManagerMissing));
    assert!(!runtime.idle().is_on());

    FakeTable::push_event(ModEvent::Enabled);
    runtime.tick_with(|| Err(LinkError::ManagerMissing));
    assert!(runtime.idle().is_on());
}

#[test]
fn a_settings_change_while_the_manager_disabled_the_mod_keeps_idle_off() {
    let runtime = attached(true);
    FakeTable::push_event(ModEvent::Disabled);
    runtime.tick_with(|| Err(LinkError::ManagerMissing));

    FakeTable::push_event(changed());
    runtime.tick_with(|| Err(LinkError::ManagerMissing));

    assert!(!runtime.idle().is_on());
}

#[test]
fn detach_turns_idle_off() {
    let runtime = attached(true);
    assert!(runtime.idle().guard().refuse(ROUTE));

    runtime.detach();

    assert!(!runtime.idle().is_on());
    assert!(!runtime.idle().guard().refuse(ROUTE));
    assert_eq!(runtime.link_state(), LinkState::Disabled);
}

#[test]
fn a_disabled_runtime_does_not_reach_the_manager_until_enable() {
    let runtime = attached(true);
    runtime.detach();

    runtime.tick_with(|| Ok(FakeTable::table()));
    assert!(!runtime.idle().is_on());

    runtime.enable_link();
    assert!(runtime.idle().is_on());
}

#[test]
fn capture_keeps_the_first_non_null_client() {
    let runtime = runtime();
    runtime.capture(0);
    assert_eq!(runtime.client(), 0);
    runtime.capture(0x10);
    runtime.capture(0x20);
    assert_eq!(runtime.client(), 0x10);
}

#[test]
fn hooks_are_bounded_and_taken_newest_first() {
    let runtime = runtime();
    for id in 0..QuirksRuntime::HOOK_CAPACITY as u64 {
        assert!(runtime.keep_hook(HookHandle(id)));
    }
    assert!(!runtime.keep_hook(HookHandle(99)));

    let taken = runtime.take_hooks();
    let ids: Vec<u64> = taken.iter().map(|hook| hook.0).collect();
    assert_eq!(ids, vec![7, 6, 5, 4, 3, 2, 1, 0]);
    assert!(runtime.take_hooks().is_empty());
}

#[test]
fn a_tick_before_enable_does_nothing() {
    let runtime = runtime();
    let called = AtomicBool::new(false);

    runtime.tick_with(|| {
        called.store(true, Ordering::SeqCst);
        Err(LinkError::ManagerMissing)
    });

    assert!(!called.load(Ordering::SeqCst));
    assert!(!runtime.idle().is_on());
    assert_eq!(runtime.link_state(), LinkState::Disabled);
}
