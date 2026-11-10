use std::path::Path;

use alaydriem_quirks::services::QuirksLink;
use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;

fn link() -> QuirksLink {
    QuirksLink::new(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap()
}

#[test]
fn reload_reads_a_changed_setting() {
    FakeTable::set_config("idle {\n  enabled = true\n}\n");
    let mut link = link();
    assert!(link.config().idle_enabled);

    FakeTable::set_config("idle {\n  enabled = false\n}\n");
    link.reload();

    assert!(!link.config().idle_enabled);
}

#[test]
fn a_drain_reads_at_most_the_per_tick_bound_and_defers_the_rest_in_order() {
    let mut link = link();
    let bound = QuirksLink::MAX_EVENTS_PER_TICK;
    let extra = 10;
    let event = |index: usize| ModEvent::Key { vk: index as u32, down: true, repeat: false };
    for index in 0..bound + extra {
        FakeTable::push_event(event(index));
    }

    let first = link.drain();
    assert_eq!(first.len(), bound);
    assert_eq!(first.first().copied(), Some(event(0)));
    assert_eq!(first.last().copied(), Some(event(bound - 1)));

    let second = link.drain();
    assert_eq!(second, (bound..bound + extra).map(event).collect::<Vec<_>>());

    assert!(link.drain().is_empty());
}

#[test]
fn a_drain_of_exactly_the_bound_loses_nothing() {
    let mut link = link();
    for index in 0..QuirksLink::MAX_EVENTS_PER_TICK {
        FakeTable::push_event(ModEvent::Key { vk: index as u32, down: true, repeat: false });
    }

    assert_eq!(link.drain().len(), QuirksLink::MAX_EVENTS_PER_TICK);
    assert!(link.drain().is_empty());
}
