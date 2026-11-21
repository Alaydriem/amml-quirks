use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use crate::idle::FireResult;
use crate::idle::keep_alive_state::KeepAliveState;

// After 900 s with no input the game goes idle: it asks for the idle screen (IdleGuard refuses that) and its update
// drops to about 10 per second. The timer still runs behind the refused screen, so the game stays throttled. While
// on, once every `INTERVAL` this runs the fire the caller gives: the game's any-input handler, the code the game runs
// for each input event. No key, button or pointer event is made, so nothing in the game reacts to it.
//
// Game thread only (`on`, `off` and `tick` run in the game update).
pub struct IdleKeepAlive {
    state: Mutex<KeepAliveState>,
}

impl IdleKeepAlive {
    pub const INTERVAL: Duration = Duration::from_secs(60);
    // the wait after a fire that found the game not ready
    pub const RETRY: Duration = Duration::from_secs(1);

    pub fn new() -> Self {
        Self { state: Mutex::new(KeepAliveState { on: false, next: Instant::now(), fired: 0, unavailable: false }) }
    }

    // turns the keep-alive on and logs it; does nothing when on, or when this game build cannot run the fire
    pub fn on(&self) {
        let mut state = self.lock();
        if state.on || state.unavailable {
            return;
        }
        state.on = true;
        state.fired = 0;
        state.next = Instant::now() + Self::INTERVAL;
        curia::info!("idle keep-alive: on", { every_seconds: Self::INTERVAL.as_secs() });
    }

    // turns it off and logs why; does nothing when off
    pub fn off(&self, why: &str) {
        let mut state = self.lock();
        if !state.on {
            return;
        }
        state.on = false;
        curia::info!("idle keep-alive: off", { why: why, fired: state.fired });
    }

    pub fn is_on(&self) -> bool {
        self.lock().on
    }

    // each game update: runs `fire` when `INTERVAL` has passed since the last fire
    pub fn tick(&self, fire: impl FnOnce() -> FireResult) {
        self.tick_at(Instant::now(), fire);
    }

    pub fn tick_at(&self, now: Instant, fire: impl FnOnce() -> FireResult) {
        let mut state = self.lock();
        if !state.on || now < state.next {
            return;
        }
        // late (a stall) or early (no client yet): the next one is a full interval away, except when the game was not
        // ready, which is tried again in a second
        match fire() {
            FireResult::Fired => {
                state.fired += 1;
                if state.fired == 1 {
                    curia::info!("idle keep-alive: fired", { count: state.fired });
                } else {
                    curia::debug!("idle keep-alive: fired", { count: state.fired });
                }
                state.next = now + Self::INTERVAL;
            }
            FireResult::NotReady => state.next = now + Self::RETRY,
            FireResult::NotAvailable => {
                state.on = false;
                state.unavailable = true;
                curia::warn!("idle keep-alive: off", { why: "not on this build", fired: state.fired });
            }
        }
    }

    pub fn fired(&self) -> u32 {
        self.lock().fired
    }

    fn lock(&self) -> MutexGuard<'_, KeepAliveState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for IdleKeepAlive {
    fn default() -> Self {
        Self::new()
    }
}
