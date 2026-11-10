use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// While on, the game cannot open the idle screen: the router detours refuse a push or replace of `/idle-route` (they
// return false, as for a route the router does not take). Every other route goes through. The game's idle timer is not
// changed; it may ask again, and each ask is refused. The detours stay installed between enable and disable.
//
// `on` and `off` run on the game thread; the detour bodies read only atomics.
pub struct IdleGuard {
    on: AtomicBool,
    // idle-screen pushes refused since `on`
    refused: AtomicU64,
}

impl IdleGuard {
    // the route of the idle screen (`OreUI::EntryPoints::Idle::ROUTE`)
    pub const IDLE_ROUTE: &'static [u8] = b"/idle-route";

    pub fn new() -> Self {
        Self { on: AtomicBool::new(false), refused: AtomicU64::new(0) }
    }

    // starts refusing the idle route; does nothing when on
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
            curia::info!("idle guard: idle-screen pushes refused", { count: refused });
        }
    }

    pub fn is_on(&self) -> bool {
        self.on.load(Ordering::SeqCst)
    }

    // the router detours: true when the push of `route` must be refused
    pub fn refuse(&self, route: &[u8]) -> bool {
        if !self.on.load(Ordering::SeqCst) {
            return false;
        }
        // the route may carry the router's prefix, so match its end
        if !route.ends_with(Self::IDLE_ROUTE) {
            return false;
        }
        // the game may ask again each update while it stays idle: log the first
        if self.refused.fetch_add(1, Ordering::SeqCst) == 0 {
            curia::info!("idle guard: refused the idle screen", { route: String::from_utf8_lossy(route).into_owned() });
        }
        true
    }
}

impl Default for IdleGuard {
    fn default() -> Self {
        Self::new()
    }
}
