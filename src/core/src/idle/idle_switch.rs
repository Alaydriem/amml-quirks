use std::time::Instant;

use crate::idle::{IdleGuard, IdleKeepAlive};

// The idle fix as one switch: the route guard and the keep-alive go on and off together.
pub struct IdleSwitch {
    guard: IdleGuard,
    keep_alive: IdleKeepAlive,
}

impl IdleSwitch {
    // `fire` runs the game's any-input handler; false when the game is not ready
    pub fn new(fire: Box<dyn Fn() -> bool + Send + Sync>) -> Self {
        Self { guard: IdleGuard::new(), keep_alive: IdleKeepAlive::new(fire) }
    }

    // does nothing when the switch already has this value
    pub fn apply(&self, on: bool) {
        if on == self.is_on() {
            return;
        }
        if on {
            self.guard.on();
            self.keep_alive.on();
        } else {
            self.guard.off();
            self.keep_alive.off("setting off");
        }
    }

    pub fn is_on(&self) -> bool {
        self.guard.is_on()
    }

    pub fn guard(&self) -> &IdleGuard {
        &self.guard
    }

    pub fn tick_at(&self, now: Instant) {
        self.keep_alive.tick_at(now);
    }

    pub fn tick(&self) {
        self.keep_alive.tick();
    }
}
