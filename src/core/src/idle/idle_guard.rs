use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// While on, the game cannot open the idle screen: each time the game asks for it, the mod cancels the ask, and the
// game is told the route was not taken. Every other screen goes through (the loader raises nothing for them). The
// game's idle timer is not changed; it may ask again, and each ask is refused.
//
// `on` and `off` run on the game thread; `refuse` reads only atomics.
pub struct IdleGuard {
    on: AtomicBool,
    // idle-screen asks refused since `on`
    refused: AtomicU64,
}

impl IdleGuard {
    pub fn new() -> Self {
        Self { on: AtomicBool::new(false), refused: AtomicU64::new(0) }
    }

    // starts refusing the idle screen; does nothing when on
    pub fn on(&self) {
        if self.on.load(Ordering::SeqCst) {
            return;
        }
        self.refused.store(0, Ordering::SeqCst);
        self.on.store(true, Ordering::SeqCst);
    }

    pub fn off(&self) {
        if !self.on.swap(false, Ordering::SeqCst) {
            return;
        }
        let refused = self.refused.load(Ordering::SeqCst);
        if refused > 0 {
            curia::info!("idle guard: idle-screen asks refused", { count: refused });
        }
    }

    pub fn is_on(&self) -> bool {
        self.on.load(Ordering::SeqCst)
    }

    // one ask of the game for the idle screen, `via` the way it asked (for the log): true when it must be refused
    pub fn refuse(&self, via: &str) -> bool {
        if !self.on.load(Ordering::SeqCst) {
            return false;
        }
        // the game may ask again each update while it stays idle: log the first
        if self.refused.fetch_add(1, Ordering::SeqCst) == 0 {
            curia::info!("idle guard: refused the idle screen", { via: via });
        }
        true
    }

    // idle-screen asks refused since the last `on`
    pub fn refused(&self) -> u64 {
        self.refused.load(Ordering::SeqCst)
    }
}

impl Default for IdleGuard {
    fn default() -> Self {
        Self::new()
    }
}
