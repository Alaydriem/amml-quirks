//! @alaydriem/quirks as an AMML mod: listens to the loader's client tick and idle-screen events, reads the manager
//! events and drives quirks-core with them. No input is ever sent.

#![deny(unsafe_code)]

#[forbid(unsafe_code)]
pub mod quirks_mod;
#[forbid(unsafe_code)]
pub mod services;

pub use quirks_mod::QuirksMod;

amml_api::amml_mod!(QuirksMod);
