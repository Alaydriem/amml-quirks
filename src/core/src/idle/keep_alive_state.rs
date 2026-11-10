use std::time::Instant;

// The state of the idle keep-alive: whether it is on, when the next fire is due and how many fires ran.
pub(crate) struct KeepAliveState {
    pub(crate) on: bool,
    pub(crate) next: Instant,
    pub(crate) fired: u32,
}
