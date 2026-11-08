use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use crate::idle::keep_alive_state::KeepAliveState;

// After 900 s with no input the game goes idle: it pushes `/idle-route` (IdleGuard refuses that) and its update drops to
// about 10 per second. The timer still runs behind the refused screen, so the game stays throttled. While on, once every
// `INTERVAL` this runs the game's any-input handler (`InputHandler::mAnyInputHandler`), the code the game runs for each
// input event. No key, button or pointer event is made, so nothing in the game or the studio reacts to it.
//
// Game thread only (`on`, `off` and `tick` run in the game update).
pub struct IdleKeepAlive {
    // runs the any-input handler of the primary client; false when the game has no client, input handler or registered
    // handler yet
    fire: Box<dyn Fn() -> bool + Send + Sync>,
    state: Mutex<KeepAliveState>,
}

impl IdleKeepAlive {
    pub const INTERVAL: Duration = Duration::from_secs(60);
    // the wait after a fire that found the game not ready
    pub const RETRY: Duration = Duration::from_secs(1);

    pub fn new(fire: Box<dyn Fn() -> bool + Send + Sync>) -> Self {
        Self { fire, state: Mutex::new(KeepAliveState { on: false, next: Instant::now(), fired: 0 }) }
    }

    // turns the keep-alive on and logs it; does nothing when on
    pub fn on(&self) {
        let mut state = self.lock();
        if state.on {
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

    // each game update: fires when `INTERVAL` has passed since the last one
    pub fn tick(&self) {
        self.tick_at(Instant::now());
    }

    pub fn tick_at(&self, now: Instant) {
        let mut state = self.lock();
        if !state.on || now < state.next {
            return;
        }
        // late (a stall) or early (no client yet): the next one is a full interval away, except when the game was not
        // ready, which is tried again in a second
        let fired = (self.fire)();
        if fired {
            state.fired += 1;
            curia::debug!("idle keep-alive: fired", { count: state.fired });
        }
        state.next = now + if fired { Self::INTERVAL } else { Self::RETRY };
    }

    pub fn fired(&self) -> u32 {
        self.lock().fired
    }

    fn lock(&self) -> MutexGuard<'_, KeepAliveState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
