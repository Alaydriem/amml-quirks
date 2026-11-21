use std::sync::{Arc, Mutex, PoisonError};

use amml_api::amml_abi::{
    ClientTick, IdleScreenOpen, InjectedInputReleased, InputLeaseLost, InputLeaseResumed, InputLeaseSuspended,
    ModsLoaded, ProviderGone, ProviderReady, ScreenshotTaken, ScreenshotWritten,
};
use amml_api::{AmmlMod, ApiError, Area, Ctx, DisableCtx, Uses};

use crate::services::{QuirksLink, QuirksListeners, QuirksRuntime};

// what the enabled mod owns. `disable` takes it, which drops the listeners and closes the link
static RUNNING: Mutex<Option<QuirksListeners>> = Mutex::new(None);

// the AMML entry point: enable starts the manager link and the listeners, disable turns the idle fix off and leaves
// the mod manager
pub struct QuirksMod;

impl AmmlMod for QuirksMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "quirks";
    const USES: Uses = amml_api::amml_uses! {
        feature "idle" {
            required ClientTick;
            required ModsLoaded;
            required ProviderReady;
            required ProviderGone;
            // The manager link listens to these when the loader serves them; quirks takes no lease and no screenshot
            optional InputLeaseSuspended;
            optional InputLeaseResumed;
            optional InputLeaseLost;
            optional InjectedInputReleased;
            optional ScreenshotTaken;
            optional ScreenshotWritten;
            optional area Area::INPUT_LEASE;
        }
        feature "idle_screen" {
            required IdleScreenOpen;
        }
        feature "keep_alive" {
            required area Area::CLIENT_ACTIVITY;
        }
    };

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        let log = ctx.log();
        // without the link the idle fix still runs with its default setting
        let link = QuirksLink::start(ctx)
            .inspect_err(|error| {
                log.error(
                    "the mod manager link did not start; the idle setting uses its default",
                    &[("error", &error.to_string())],
                );
            })
            .ok();
        let runtime = Arc::new(QuirksRuntime::new());
        runtime.attach(link);
        let listeners = QuirksListeners::start(ctx, Arc::clone(&runtime)).inspect_err(|error| {
            runtime.detach();
            log.error("quirks off", &[("error", &error.to_string())]);
        })?;
        *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(listeners);
        Ok(())
    }

    fn disable(_ctx: &Ctx, _how: &DisableCtx) {
        let running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(listeners) = running {
            listeners.stop();
        }
    }
}
