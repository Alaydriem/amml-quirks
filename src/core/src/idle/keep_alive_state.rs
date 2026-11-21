use std::time::Instant;

// The state of the idle keep-alive: whether it is on, when the next fire is due, how many fires ran and whether this
// game build cannot run the fire at all.
pub(crate) struct KeepAliveState {
    pub(crate) on: bool,
    pub(crate) next: Instant,
    pub(crate) fired: u32,
    pub(crate) unavailable: bool,
}
