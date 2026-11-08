use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use quirks_core::idle::{IdleKeepAlive, IdleSwitch};

// a switch whose fire counts its calls and succeeds
fn rig() -> (IdleSwitch, Arc<AtomicU32>) {
    let calls = Arc::new(AtomicU32::new(0));
    let counted = Arc::clone(&calls);
    let switch = IdleSwitch::new(Box::new(move || {
        counted.fetch_add(1, Ordering::SeqCst);
        true
    }));
    (switch, calls)
}

#[test]
fn apply_turns_guard_and_keep_alive_on_and_off() {
    let (switch, calls) = rig();
    assert!(!switch.is_on());
    switch.apply(true);
    assert!(switch.is_on());
    assert!(switch.guard().refuse(b"/idle-route"));
    switch.tick_at(Instant::now() + IdleKeepAlive::INTERVAL + Duration::from_secs(1));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    switch.apply(false);
    assert!(!switch.is_on());
    assert!(!switch.guard().refuse(b"/idle-route"));
    switch.tick_at(Instant::now() + IdleKeepAlive::INTERVAL * 3);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn applying_the_same_value_changes_nothing() {
    let (switch, calls) = rig();
    let start = Instant::now();
    switch.apply(true);
    std::thread::sleep(Duration::from_millis(50));
    // must not move the keep-alive's next fire
    switch.apply(true);
    switch.tick_at(start + IdleKeepAlive::INTERVAL + Duration::from_millis(10));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
