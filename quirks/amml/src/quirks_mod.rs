use std::ffi::c_void;

use amml_game::{EntryId, EntryNeed, EntryResolver, GameEntries, GameError, HookOrder};
use amml_sdk::{AmmlMod, Api, ModDirs, SdkError};

use crate::detours::{client_update, route_push, route_replace};
use crate::input::AnyInput;
use crate::services::{AmmlHost, HookInstaller, QuirksRuntime};
use crate::traits::GameHost;
use crate::types::HookSpec;

// the loader a game host stands in for, as the resolver the game entries are read with
struct HostResolver<'a, H: GameHost>(&'a H);

impl<H: GameHost> EntryResolver for HostResolver<'_, H> {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.0.resolve(id)
    }
}

// the AMML entry point: load makes the runtime, enable installs the hooks, disable turns the idle fix off and then
// removes them. The manager link is made on the first ClientInstance::update after enable
pub struct QuirksMod;

impl QuirksMod {
    // the id as the module string the loader and the hook host take; a test checks it against `AmmlMod::id`
    pub const MODULE: &'static str = "@alaydriem/quirks";
    pub const UPDATE: &'static str = "ClientInstance.update";
    pub const PUSH_ROUTE: &'static str = "OreUIRouter.pushRoute";
    pub const REPLACE_ROUTE: &'static str = "OreUIRouter.replaceRoute";

    const SCREEN_FEATURE: &'static str = "the idle screen block";

    // every hook, in install order; the client update carries the setting, the route hooks are writers that may
    // refuse a push
    pub fn hooks() -> [HookSpec; 3] {
        [
            HookSpec {
                entry: Self::PUSH_ROUTE,
                detour: route_push::detour as *const c_void,
                original: route_push::GUARD.original(),
                required: false,
                priority: HookOrder::WRITE,
                feature: Self::SCREEN_FEATURE,
            },
            HookSpec {
                entry: Self::REPLACE_ROUTE,
                detour: route_replace::detour as *const c_void,
                original: route_replace::GUARD.original(),
                required: false,
                priority: HookOrder::WRITE,
                feature: Self::SCREEN_FEATURE,
            },
            HookSpec {
                entry: Self::UPDATE,
                detour: client_update::detour as *const c_void,
                original: client_update::GUARD.original(),
                required: true,
                priority: 0,
                feature: "the idle setting",
            },
        ]
    }

    // the game reads: the input handler getter serves the keep-alive and is optional
    pub fn wanted() -> [(EntryId, EntryNeed); 1] {
        [(EntryId::CLIENT_INSTANCE_GET_MINECRAFT_INPUT, EntryNeed::Optional { feature: "the idle keep-alive" })]
    }

    // the body of enable: the game entries and the required hook entries must resolve, then the hooks go in and
    // the next tick reaches the manager
    pub fn enable_with<H: GameHost>(host: &H, runtime: &QuirksRuntime) -> Result<(), SdkError> {
        let entries = GameEntries::resolve(&HostResolver(host), &Self::wanted()).map_err(|error| match error {
            GameError::MissingEntry { id } => SdkError::MissingEntry(id),
            other => SdkError::Mod(other.to_string()),
        })?;
        HookInstaller::install(host, runtime, &Self::hooks())?;
        for (feature, cause) in entries.off_features() {
            host.warn(&format!("{feature} is off ({cause})"));
        }
        runtime.set_entries(entries);
        runtime.enable_link();
        Ok(())
    }

    // the keep-alive fire: the game any-input handler of the client the update detour noted. False before the
    // first update or with the entry off
    #[allow(unsafe_code)]
    fn fire_any_input() -> bool {
        let Some(runtime) = QuirksRuntime::get() else { return false };
        let client = runtime.client();
        if client == 0 {
            return false;
        }
        runtime
            .with_entries(|entries| {
                // SAFETY: the client is the live primary ClientInstance the update detour noted, and the keep-alive
                // runs on the game thread
                AnyInput::new(entries).is_some_and(|input| unsafe { input.fire(client as *mut c_void) })
            })
            .unwrap_or(false)
    }
}

impl AmmlMod for QuirksMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "quirks";

    fn load(api: &Api) -> Result<(), SdkError> {
        let dirs = ModDirs::new(api, Self::MODULE)?;
        QuirksRuntime::install(&dirs.config_dir, Box::new(Self::fire_any_input));
        Ok(())
    }

    fn enable(api: &Api) -> Result<(), SdkError> {
        let runtime = QuirksRuntime::get().ok_or_else(|| SdkError::Mod("load did not run".to_string()))?;
        Self::enable_with(&AmmlHost::new(api, Self::MODULE), runtime)
    }

    fn disable(api: &Api) {
        let Some(runtime) = QuirksRuntime::get() else { return };
        runtime.detach();
        HookInstaller::remove_all(&AmmlHost::new(api, Self::MODULE), runtime);
    }
}
