use alaydriem_quirks::QuirksMod;
use amm_core::structs::event::ModEvent;
use amml_api::amml_abi::AmmlRouteVia;
use amml_api::testing::FakeActivity;
use amml_api::{AmmlMod, Area};

use crate::support::QuirksRig;

const ON: &str = "idle {\n  enabled = true\n}\n";
const OFF: &str = "idle {\n  enabled = false\n}\n";

#[test]
fn the_manifest_uses_block_is_the_uses_of_the_mod() {
    let manifest = include_str!("../amml.hcl").replace("\r\n", "\n");
    assert_eq!(QuirksMod::USES.write_into(&manifest), manifest);
}

#[test]
fn the_manager_link_has_every_row_it_needs() {
    amm_client::link::LinkUses::check(&QuirksMod::USES).unwrap();
}

#[test]
fn the_features_of_the_mod_are_required() {
    let rows = QuirksMod::USES.rows();
    let required = |name: &str| {
        rows.iter().any(|row| {
            // SAFETY: the rows of a `Uses` are built by `amml_uses!` from `&'static str` names
            let row_name = unsafe { row.name.as_str() }.unwrap_or("");
            row_name == name && row.required != 0
        })
    };
    for name in ["client.tick", "client.idle_screen_open", Area::CLIENT_ACTIVITY.id, "registry.provider_ready"] {
        assert!(required(name), "{name} is not a required row");
    }
}

#[test]
fn the_idle_screen_is_refused_without_a_manager() {
    let rig = QuirksRig::without_manager();

    assert!(rig.idle_screen(AmmlRouteVia::PUSH));
    assert!(rig.idle_screen(AmmlRouteVia::REPLACE));
    assert!(rig.logged("needs @amml/modmanager"));
    assert!(rig.logged("idle guard: refused the idle screen via=push"));
}

#[test]
fn the_manager_setting_turns_the_fix_off_and_on() {
    let rig = QuirksRig::with_manager(OFF);
    rig.tick();
    assert!(rig.logged("attached to the mod manager"));
    assert!(!rig.idle_screen(AmmlRouteVia::PUSH));
    assert!(rig.logged_with("idle keep-alive: off", "why=setting off"));

    amm_client::test_support::FakeTable::set_config(ON);
    rig.manager_event(ModEvent::SettingsChanged { paths: 1 });
    assert!(rig.idle_screen(AmmlRouteVia::PUSH));
}

#[test]
fn a_manager_disable_keeps_the_fix_off_until_enabled() {
    let rig = QuirksRig::with_manager(ON);
    rig.tick();
    assert!(rig.idle_screen(AmmlRouteVia::PUSH));

    rig.manager_event(ModEvent::Disabled);
    assert!(!rig.idle_screen(AmmlRouteVia::PUSH));
    amm_client::test_support::FakeTable::set_config(ON);
    rig.manager_event(ModEvent::SettingsChanged { paths: 1 });
    assert!(!rig.idle_screen(AmmlRouteVia::PUSH), "a setting does not undo the manager's disable");

    rig.manager_event(ModEvent::Enabled);
    assert!(rig.idle_screen(AmmlRouteVia::PUSH));
}

#[test]
fn the_manager_going_away_keeps_the_last_value() {
    let rig = QuirksRig::with_manager(ON);
    rig.tick();
    rig.manager_gone();
    assert!(rig.idle_screen(AmmlRouteVia::PUSH));
}

#[test]
fn the_keep_alive_does_not_fire_before_its_interval() {
    let rig = QuirksRig::with_manager(ON);
    rig.activity(FakeActivity::Ok);
    for _ in 0..10 {
        rig.tick();
    }
    assert_eq!(rig.activity_calls(), 0);
    assert!(rig.logged_with("idle keep-alive: on", "every_seconds=60"));
}

#[test]
fn disable_turns_the_fix_off() {
    let rig = QuirksRig::with_manager(ON);
    rig.tick();
    rig.game.disable::<QuirksMod>();
    assert!(rig.logged_with("idle keep-alive: off", "why=disabled"));
}
