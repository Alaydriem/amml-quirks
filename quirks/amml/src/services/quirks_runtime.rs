use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError, RwLock};

use amm_client::link::ApiTable;
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use amml_game::GameEntries;
use amml_sdk::{HookHandle, ModRuntime};
use quirks_core::config::QuirksConfig;
use quirks_core::idle::IdleSwitch;

use crate::services::QuirksLink;
use crate::types::LinkState;

// what the hooks share: the idle switch, the game client and entries, the manager link state and the installed
// hooks. The mod manager is reached on the first ClientInstance::update after enable, when every mod of the host
// has loaded; `hooks` holds at most HOOK_CAPACITY handles
pub struct QuirksRuntime {
    idle: IdleSwitch,
    client: AtomicUsize,
    entries: RwLock<Option<GameEntries>>,
    config_dir: PathBuf,
    link_state: AtomicU8,
    link: Mutex<Option<QuirksLink>>,
    // the manager turned the mod off: the idle fix stays off, whatever the setting
    manager_disabled: AtomicBool,
    hooks: Mutex<Vec<HookHandle>>,
}

static RUNTIME: OnceLock<QuirksRuntime> = OnceLock::new();

impl QuirksRuntime {
    pub const HOOK_CAPACITY: usize = 8;

    pub fn new(config_dir: &Path, fire: Box<dyn Fn() -> bool + Send + Sync>) -> Self {
        Self {
            idle: IdleSwitch::new(fire),
            client: AtomicUsize::new(0),
            entries: RwLock::new(None),
            config_dir: config_dir.to_path_buf(),
            link_state: AtomicU8::new(LinkState::Disabled.to_u8()),
            link: Mutex::new(None),
            manager_disabled: AtomicBool::new(false),
            hooks: Mutex::new(Vec::with_capacity(Self::HOOK_CAPACITY)),
        }
    }

    // the runtime of this DLL; the first call wins
    pub fn install(config_dir: &Path, fire: Box<dyn Fn() -> bool + Send + Sync>) -> &'static Self {
        RUNTIME.get_or_init(|| Self::new(config_dir, fire))
    }

    pub fn get() -> Option<&'static Self> {
        RUNTIME.get()
    }

    pub fn idle(&self) -> &IdleSwitch {
        &self.idle
    }

    pub fn link_state(&self) -> LinkState {
        LinkState::from_u8(self.link_state.load(Ordering::SeqCst))
    }

    pub fn client(&self) -> usize {
        self.client.load(Ordering::Relaxed)
    }

    // keeps the first non-null client; later calls change nothing. The primary client is created at start-up and
    // lives until the game closes, while split-screen clients come and go
    pub fn capture(&self, client: usize) {
        if client != 0 {
            let _ = self.client.compare_exchange(0, client, Ordering::Relaxed, Ordering::Relaxed);
        }
    }

    pub fn set_entries(&self, entries: GameEntries) {
        *self.entries.write().unwrap_or_else(PoisonError::into_inner) = Some(entries);
    }

    // None without the entries
    pub fn with_entries<R>(&self, body: impl FnOnce(&GameEntries) -> R) -> Option<R> {
        let entries = self.entries.read().unwrap_or_else(PoisonError::into_inner);
        entries.as_ref().map(body)
    }

    // enable: the runtime starts Disabled, so a tick before this call does nothing. The idle fix starts from the default setting, so it works without a manager, and the next tick
    // reaches the manager again
    pub fn enable_link(&self) {
        self.manager_disabled.store(false, Ordering::SeqCst);
        self.link_state.store(LinkState::Waiting.to_u8(), Ordering::SeqCst);
        self.idle.apply(QuirksConfig::default().idle_enabled);
    }

    // once per ClientInstance::update on the game thread, after the original
    pub fn tick(&self, client: usize) {
        self.capture(client);
        self.tick_with(ApiTable::resolve);
    }

    // reaches the manager when the link is Waiting, then handles its events and runs the keep-alive
    pub fn tick_with(&self, table: impl FnOnce() -> Result<ApiTable, LinkError>) {
        if self.link_state() == LinkState::Waiting {
            self.attach_with(table());
        }
        if self.link_state() == LinkState::Attached {
            self.run();
        }
        self.idle.tick();
    }

    // registers with the manager the table belongs to; on failure the idle fix keeps its default and one line is
    // logged. Only a Waiting runtime takes the result: if disable ran meanwhile, a new link is dropped again
    fn attach_with(&self, table: Result<ApiTable, LinkError>) {
        let result = table.and_then(|table| QuirksLink::new(table, &self.config_dir));
        let (state, link) = match result {
            Ok(link) => (LinkState::Attached, Some(link)),
            Err(error) => {
                Self::log_link_error(&error);
                (LinkState::Off, None)
            }
        };
        let waiting = LinkState::Waiting.to_u8();
        let mut slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        if self.link_state.compare_exchange(waiting, state.to_u8(), Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return;
        }
        if let Some(link) = link {
            self.apply_setting(link.config());
            *slot = Some(link);
            curia::info!("@alaydriem/quirks attached to the mod manager");
        }
    }

    // disable: the guard and the keep-alive go off before the hooks come out, and the mod unregisters
    pub fn detach(&self) {
        self.link_state.store(LinkState::Disabled.to_u8(), Ordering::SeqCst);
        drop(self.link.lock().unwrap_or_else(PoisonError::into_inner).take());
        self.idle.apply(false);
    }

    // keeps an installed hook; false, keeping nothing, when HOOK_CAPACITY hooks are already kept
    pub fn keep_hook(&self, hook: HookHandle) -> bool {
        let mut hooks = self.hooks.lock().unwrap_or_else(PoisonError::into_inner);
        if hooks.len() >= Self::HOOK_CAPACITY {
            return false;
        }
        hooks.push(hook);
        true
    }

    // the installed hooks, newest first, leaving none behind
    pub fn take_hooks(&self) -> Vec<HookHandle> {
        let mut hooks = self.hooks.lock().unwrap_or_else(PoisonError::into_inner);
        let mut taken = std::mem::take(&mut *hooks);
        taken.reverse();
        taken
    }

    fn apply_setting(&self, config: &QuirksConfig) {
        self.idle.apply(config.idle_enabled && !self.manager_disabled.load(Ordering::SeqCst));
    }

    fn run(&self) {
        let Ok(mut slot) = self.link.lock() else { return };
        let Some(link) = slot.as_mut() else { return };
        for event in link.drain() {
            match event {
                ModEvent::SettingsChanged { .. } => {
                    let config = link.reload().clone();
                    self.apply_setting(&config);
                }
                ModEvent::Disabled => {
                    self.manager_disabled.store(true, Ordering::SeqCst);
                    self.idle.apply(false);
                }
                ModEvent::Enabled => {
                    self.manager_disabled.store(false, Ordering::SeqCst);
                    let config = link.config().clone();
                    self.apply_setting(&config);
                }
                _ => {}
            }
        }
    }

    fn log_link_error(error: &LinkError) {
        let Some(runtime) = ModRuntime::get() else { return };
        match error {
            LinkError::ManagerMissing | LinkError::AbiMismatch { .. } => {
                runtime
                    .logger()
                    .error("@alaydriem/quirks needs @amml/modmanager 0.2.x; the idle setting uses its default");
            }
            other => {
                runtime.logger().error(&format!(
                    "@amml/modmanager refused @alaydriem/quirks ({other}); the idle setting uses its default"
                ));
            }
        }
    }
}
