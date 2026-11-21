use std::sync::Arc;

use amml_api::amml_abi::{ClientTick, IdleScreenOpen, ModsLoaded};
use amml_api::{ApiError, Cancel, Ctx, HookOrder, IdleScreenOpenExt, Listener, RouteVia};
use quirks_core::idle::FireResult;

use crate::services::QuirksRuntime;

// the loader listeners of quirks; each only hands its event to the runtime. Dropping them removes them
pub struct QuirksListeners {
    listeners: Vec<Listener>,
    runtime: Arc<QuirksRuntime>,
}

impl QuirksListeners {
    pub fn start(ctx: &Ctx, runtime: Arc<QuirksRuntime>) -> Result<Self, ApiError> {
        let events = ctx.events();
        let mut listeners = Vec::with_capacity(3);
        let ticking = Arc::clone(&runtime);
        listeners.push(events.listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            ticking.tick(|| match at.client() {
                Some(client) => QuirksRuntime::fire_result(client.note_activity()),
                None => FireResult::NotReady,
            });
        })?);
        // a writer: it cancels the ask before the readers see it
        let guarding = Arc::clone(&runtime);
        listeners.push(events.listen::<IdleScreenOpen>(HookOrder::WRITE, move |event, _| {
            let via = match event.via() {
                RouteVia::Push => "push",
                RouteVia::Replace => "replace",
                RouteVia::Other(_) => "other",
            };
            if guarding.refuse_idle_screen(via) {
                event.cancel();
            }
        })?);
        let loaded = Arc::clone(&runtime);
        listeners.push(events.listen::<ModsLoaded>(HookOrder::LAST, move |_, _| loaded.mods_loaded())?);
        Ok(Self { listeners, runtime })
    }

    // the listeners go first, so no listener runs while the runtime lets go of the link
    pub fn stop(self) {
        let Self { listeners, runtime } = self;
        drop(listeners);
        runtime.detach();
    }
}
