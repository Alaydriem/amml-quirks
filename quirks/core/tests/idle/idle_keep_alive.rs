use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use quirks_core::idle::IdleKeepAlive;

// a keep-alive whose fire counts its calls and succeeds while `ready` is true
fn rig() -> (IdleKeepAlive, Arc<AtomicU32>, Arc<AtomicBool>) {
    let calls = Arc::new(AtomicU32::new(0));
    let ready = Arc::new(AtomicBool::new(true));
    let (counted, gate) = (Arc::clone(&calls), Arc::clone(&ready));
    let keep_alive = IdleKeepAlive::new(Box::new(move || {
        counted.fetch_add(1, Ordering::SeqCst);
        gate.load(Ordering::SeqCst)
    }));
    (keep_alive, calls, ready)
}

#[test]
fn it_fires_once_per_interval_while_on() {
    let (keep_alive, calls, _) = rig();
    let start = Instant::now();

    keep_alive.tick_at(start + IdleKeepAlive::INTERVAL * 2);
    assert_eq!(calls.load(Ordering::SeqCst), 0, "off: nothing fires");

    keep_alive.on();
    keep_alive.tick_at(Instant::now() + Duration::from_secs(30));
    assert_eq!(calls.load(Ordering::SeqCst), 0, "before the interval");

    let late = Instant::now() + IdleKeepAlive::INTERVAL + Duration::from_secs(1);
    keep_alive.tick_at(late);
    keep_alive.tick_at(late + Duration::from_secs(10));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(keep_alive.fired(), 1);

    keep_alive.tick_at(late + IdleKeepAlive::INTERVAL);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn a_game_that_is_not_ready_is_asked_again_in_a_second() {
    let (keep_alive, calls, ready) = rig();
    ready.store(false, Ordering::SeqCst);
    keep_alive.on();
    let first = Instant::now() + IdleKeepAlive::INTERVAL + Duration::from_secs(1);

    keep_alive.tick_at(first);
    keep_alive.tick_at(first + Duration::from_millis(500));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    ready.store(true, Ordering::SeqCst);
    keep_alive.tick_at(first + IdleKeepAlive::RETRY);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(keep_alive.fired(), 1);
}
