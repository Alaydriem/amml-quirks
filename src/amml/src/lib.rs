//! @alaydriem/quirks as an AMML mod: hooks the client update and the router, reads the manager events and
//! the game state, and drives quirks-core with them. No input is ever sent.

#![deny(unsafe_code)]

pub mod detours;
pub mod input;
pub mod quirks_mod;
pub mod services;
pub mod traits;
pub mod types;

pub use quirks_mod::QuirksMod;

amml_sdk::amml_mod!(QuirksMod);
