// What one fire of the keep-alive gave: the game ran its any-input handler, the game is not ready yet (no client,
// input handler or registered handler; asked again in a second), or this game build cannot run it at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireResult {
    Fired,
    NotReady,
    NotAvailable,
}
