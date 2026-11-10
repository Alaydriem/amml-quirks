use std::path::Path;

use amm_client::link::{ApiTable, EventDrain, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use quirks_core::config::QuirksConfig;

use crate::services::{QuirksDescriptor, RawSettings, SettingsReader};

// the registration with the mod manager and the settings read from it; `events` holds at most
// MAX_EVENTS_PER_TICK and is cleared at the start of every drain
pub struct QuirksLink {
    link: ManagerLink,
    config: QuirksConfig,
    events: Vec<ModEvent>,
}

impl QuirksLink {
    pub const MAX_DRAINS_PER_TICK: usize = 4;
    pub const MAX_EVENTS_PER_TICK: usize = Self::MAX_DRAINS_PER_TICK * EventDrain::CAPACITY;

    pub fn new(table: ApiTable, config_dir: &Path) -> Result<Self, LinkError> {
        let link = ManagerLink::without_frame(table, QuirksDescriptor::TEXT, config_dir)?;
        Self::from_link(link)
    }

    pub fn from_link(link: ManagerLink) -> Result<Self, LinkError> {
        let mut this =
            Self { config: QuirksConfig::default(), link, events: Vec::with_capacity(Self::MAX_EVENTS_PER_TICK) };
        this.config = this.read_settings();
        Ok(this)
    }

    pub fn config(&self) -> &QuirksConfig {
        &self.config
    }

    // reads the settings again
    pub fn reload(&mut self) -> &QuirksConfig {
        self.config = self.read_settings();
        &self.config
    }

    // reads the queued manager events, at most MAX_EVENTS_PER_TICK; the rest wait for the next tick
    pub fn drain(&mut self) -> Vec<ModEvent> {
        self.events.clear();
        for _ in 0..Self::MAX_DRAINS_PER_TICK {
            let before = self.events.len();
            self.events.extend(self.link.events());
            if self.events.len() - before < EventDrain::CAPACITY {
                break;
            }
        }
        self.events.clone()
    }

    fn read_settings(&self) -> QuirksConfig {
        match self.link.config::<RawSettings>() {
            Ok(raw) => SettingsReader::convert(&raw),
            Err(error) => {
                curia::warn!("config could not be read; using the defaults", { error: error.to_string() });
                QuirksConfig::default()
            }
        }
    }
}
