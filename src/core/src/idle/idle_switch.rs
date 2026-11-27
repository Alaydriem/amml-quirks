use std::time::Instant;

use crate::idle::{FireResult, IdleGuard, IdleKeepAlive};

// The idle fix as one switch: the idle-screen guard and the keep-alive go on and off together.
pub struct IdleSwitch {
    guard: IdleGuard,
    keep_alive: IdleKeepAlive,
}

impl IdleSwitch {
    pub fn new() -> Self {
        Self { guard: IdleGuard::new(), keep_alive: IdleKeepAlive::new() }
    }

    // the setting: does nothing when the switch already has this value
    pub fn apply(&self, on: bool) {
        if on {
            self.on();
        } else {
            self.off("setting off");
        }
    }

    pub fn on(&self) {
        if self.is_on() {
            return;
        }
        self.guard.on();
        self.keep_alive.on();
    }

    // turns both off and logs `why`; does nothing when off
    pub fn off(&self, why: &str) {
        if !self.is_on() {
            return;
        }
        self.guard.off();
        self.keep_alive.off(why);
    }

    pub fn is_on(&self) -> bool {
        self.guard.is_on()
    }

    pub fn guard(&self) -> &IdleGuard {
        &self.guard
    }

    pub fn keep_alive(&self) -> &IdleKeepAlive {
        &self.keep_alive
    }

    pub fn tick_at(&self, now: Instant, fire: impl FnOnce() -> FireResult) {
        self.keep_alive.tick_at(now, fire);
    }

    pub fn tick(&self, fire: impl FnOnce() -> FireResult) {
        self.keep_alive.tick(fire);
    }
}

impl Default for IdleSwitch {
    fn default() -> Self {
        Self::new()
    }
}
