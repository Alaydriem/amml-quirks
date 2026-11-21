use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use amm_client::link::LinkState;
use amm_core::structs::event::ModEvent;
use amml_api::ApiError;
use quirks_core::config::QuirksConfig;
use quirks_core::idle::{FireResult, IdleSwitch};

use crate::services::QuirksLink;

// what the listeners share: the idle switch, the manager link and what the manager last said. The idle fix starts
// from the default setting at enable, so it works without a manager; the settings are read on the first tick that
// finds the link connected, and again after each reconnect
pub struct QuirksRuntime {
    idle: IdleSwitch,
    link: Mutex<Option<QuirksLink>>,
    // the link was connected at the last tick
    connected: AtomicBool,
    // the manager turned the mod off: the idle fix stays off, whatever the setting
    manager_disabled: AtomicBool,
}

impl QuirksRuntime {
    pub fn new() -> Self {
        Self {
            idle: IdleSwitch::new(),
            link: Mutex::new(None),
            connected: AtomicBool::new(false),
            manager_disabled: AtomicBool::new(false),
        }
    }

    pub fn idle(&self) -> &IdleSwitch {
        &self.idle
    }

    // enable: takes the link (none when it could not start) and turns the idle fix on with the default setting
    pub fn attach(&self, link: Option<QuirksLink>) {
        let mut slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        drop(slot.take());
        *slot = link;
        self.connected.store(false, Ordering::SeqCst);
        self.manager_disabled.store(false, Ordering::SeqCst);
        if cfg!(feature = "test-no-keep-alive") {
            curia::warn!("idle keep-alive: this test build never fires it");
        }
        self.idle.apply(QuirksConfig::default().idle_enabled);
    }

    // disable: the guard and the keep-alive go off, then the link closes
    pub fn detach(&self) {
        let mut slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        self.idle.off("disabled");
        self.connected.store(false, Ordering::SeqCst);
        drop(slot.take());
    }

    // once per client tick on the game thread: the manager's settings and events, then the keep-alive with `fire`
    pub fn tick(&self, fire: impl FnOnce() -> FireResult) {
        self.read_manager();
        if !cfg!(feature = "test-no-keep-alive") {
            self.idle.tick(fire);
        }
    }

    // one ask of the game for the idle screen: true when it must be refused
    pub fn refuse_idle_screen(&self, via: &str) -> bool {
        self.idle.guard().refuse(via)
    }

    // after every mod loaded: a link that is not connected now has no manager, and the setting keeps its default
    pub fn mods_loaded(&self) {
        let slot = self.link.lock().unwrap_or_else(PoisonError::into_inner);
        let connected = slot.as_ref().is_some_and(|link| link.state() == LinkState::Connected);
        if !connected {
            curia::error!("@alaydriem/quirks needs @amml/modmanager; the idle setting uses its default");
        }
    }

    // what a call of the game's any-input handler gave, as a keep-alive fire: a refused or early call is tried again,
    // a build without the call turns the keep-alive off
    pub fn fire_result(result: Result<(), ApiError>) -> FireResult {
        match result {
            Ok(()) => FireResult::Fired,
            Err(ApiError::NotAvailable) => FireResult::NotAvailable,
            Err(_) => FireResult::NotReady,
        }
    }

    fn read_manager(&self) {
        let Ok(mut slot) = self.link.lock() else { return };
        let Some(link) = slot.as_mut() else { return };
        let connected = link.state() == LinkState::Connected;
        let was_connected = self.connected.swap(connected, Ordering::SeqCst);
        let mut reload = connected && !was_connected;
        if reload {
            curia::info!("@alaydriem/quirks attached to the mod manager");
        }
        for event in link.drain() {
            match event {
                ModEvent::SettingsChanged { .. } => reload = true,
                ModEvent::Disabled => {
                    self.manager_disabled.store(true, Ordering::SeqCst);
                    self.idle.off("the manager disabled the mod");
                }
                ModEvent::Enabled => {
                    self.manager_disabled.store(false, Ordering::SeqCst);
                    reload = true;
                }
                _ => {}
            }
        }
        if reload && connected {
            let config = link.settings();
            self.idle.apply(config.idle_enabled && !self.manager_disabled.load(Ordering::SeqCst));
        }
    }
}

impl Default for QuirksRuntime {
    fn default() -> Self {
        Self::new()
    }
}
