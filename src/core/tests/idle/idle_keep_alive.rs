use std::cell::Cell;
use std::time::{Duration, Instant};

use quirks_core::idle::{FireResult, IdleKeepAlive};

// a fire that counts its calls and gives `result`
fn fire<'a>(calls: &'a Cell<u32>, result: FireResult) -> impl FnOnce() -> FireResult + 'a {
    move || {
        calls.set(calls.get() + 1);
        result
    }
}

#[test]
fn it_fires_once_per_interval_while_on() {
    let keep_alive = IdleKeepAlive::new();
    let calls = Cell::new(0);
    let start = Instant::now();

    keep_alive.tick_at(start + IdleKeepAlive::INTERVAL * 2, fire(&calls, FireResult::Fired));
    assert_eq!(calls.get(), 0, "off: nothing fires");

    keep_alive.on();
    keep_alive.tick_at(Instant::now() + Duration::from_secs(30), fire(&calls, FireResult::Fired));
    assert_eq!(calls.get(), 0, "before the interval");

    let late = Instant::now() + IdleKeepAlive::INTERVAL + Duration::from_secs(1);
    keep_alive.tick_at(late, fire(&calls, FireResult::Fired));
    keep_alive.tick_at(late + Duration::from_secs(10), fire(&calls, FireResult::Fired));
    assert_eq!(calls.get(), 1);
    assert_eq!(keep_alive.fired(), 1);

    keep_alive.tick_at(late + IdleKeepAlive::INTERVAL, fire(&calls, FireResult::Fired));
    assert_eq!(calls.get(), 2);
}

#[test]
fn a_game_that_is_not_ready_is_asked_again_in_a_second() {
    let keep_alive = IdleKeepAlive::new();
    let calls = Cell::new(0);
    keep_alive.on();
    let first = Instant::now() + IdleKeepAlive::INTERVAL + Duration::from_secs(1);

    keep_alive.tick_at(first, fire(&calls, FireResult::NotReady));
    keep_alive.tick_at(first + Duration::from_millis(500), fire(&calls, FireResult::NotReady));
    assert_eq!(calls.get(), 1);

    keep_alive.tick_at(first + IdleKeepAlive::RETRY, fire(&calls, FireResult::Fired));
    assert_eq!(calls.get(), 2);
    assert_eq!(keep_alive.fired(), 1);
}

#[test]
fn a_build_without_the_fire_turns_it_off_for_good() {
    let keep_alive = IdleKeepAlive::new();
    let calls = Cell::new(0);
    keep_alive.on();
    let first = Instant::now() + IdleKeepAlive::INTERVAL + Duration::from_secs(1);

    keep_alive.tick_at(first, fire(&calls, FireResult::NotAvailable));
    assert!(!keep_alive.is_on());

    keep_alive.on();
    assert!(!keep_alive.is_on(), "a later on does not arm it again");
    keep_alive.tick_at(first + IdleKeepAlive::INTERVAL * 2, fire(&calls, FireResult::Fired));
    assert_eq!(calls.get(), 1);
}
