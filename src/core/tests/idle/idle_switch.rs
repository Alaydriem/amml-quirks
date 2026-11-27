use std::cell::Cell;
use std::time::{Duration, Instant};

use quirks_core::idle::{FireResult, IdleKeepAlive, IdleSwitch};

fn fire(calls: &Cell<u32>) -> impl FnOnce() -> FireResult + '_ {
    move || {
        calls.set(calls.get() + 1);
        FireResult::Fired
    }
}

#[test]
fn apply_turns_guard_and_keep_alive_on_and_off() {
    let switch = IdleSwitch::new();
    let calls = Cell::new(0);
    assert!(!switch.is_on());
    switch.apply(true);
    assert!(switch.is_on());
    assert!(switch.guard().refuse("push"));
    switch.tick_at(Instant::now() + IdleKeepAlive::INTERVAL + Duration::from_secs(1), fire(&calls));
    assert_eq!(calls.get(), 1);
    switch.apply(false);
    assert!(!switch.is_on());
    assert!(!switch.guard().refuse("push"));
    switch.tick_at(Instant::now() + IdleKeepAlive::INTERVAL * 3, fire(&calls));
    assert_eq!(calls.get(), 1);
}

#[test]
fn applying_the_same_value_changes_nothing() {
    let switch = IdleSwitch::new();
    let calls = Cell::new(0);
    let start = Instant::now();
    switch.apply(true);
    std::thread::sleep(Duration::from_millis(50));
    // must not move the keep-alive's next fire
    switch.apply(true);
    switch.tick_at(start + IdleKeepAlive::INTERVAL + Duration::from_millis(10), fire(&calls));
    assert_eq!(calls.get(), 1);
}

#[test]
fn off_turns_both_off_whatever_the_reason() {
    let switch = IdleSwitch::new();
    switch.on();
    switch.off("disabled");
    assert!(!switch.is_on());
    assert!(!switch.keep_alive().is_on());
}
