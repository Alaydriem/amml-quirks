use amm_client::link::{LinkState, ManagerLink};
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use amml_api::Ctx;
use quirks_core::config::QuirksConfig;

use crate::services::{QuirksDescriptor, RawSettings, SettingsReader};

// the registration with the mod manager and the settings read from it. `events` is the drain buffer, cleared at the
// start of every drain; the link itself keeps a bounded number of unread events
pub struct QuirksLink {
    link: ManagerLink,
    events: Vec<ModEvent>,
}

impl QuirksLink {
    // the link waits for the manager's tables; `state` says when it is connected
    pub fn start(ctx: &Ctx) -> Result<Self, LinkError> {
        let link = ManagerLink::start_without_frame(ctx, QuirksDescriptor::TEXT)?;
        Ok(Self { link, events: Vec::new() })
    }

    pub fn state(&self) -> LinkState {
        self.link.state()
    }

    // the settings of the manager's config.hcl; a text that cannot be read gives the defaults, with one warning
    pub fn settings(&self) -> QuirksConfig {
        match self.link.config::<RawSettings>() {
            Ok(raw) => SettingsReader::convert(&raw),
            Err(error) => {
                curia::warn!("config could not be read; using the defaults", { error: error.to_string() });
                QuirksConfig::default()
            }
        }
    }

    // reads every queued manager event
    pub fn drain(&mut self) -> &[ModEvent] {
        self.events.clear();
        self.events.extend(self.link.events());
        &self.events
    }
}
