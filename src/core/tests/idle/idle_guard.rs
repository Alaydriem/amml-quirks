use quirks_core::idle::IdleGuard;

#[test]
fn the_idle_route_is_refused_only_while_on() {
    let guard = IdleGuard::new();
    assert!(!guard.refuse(b"/idle-route"));

    guard.on();
    assert!(guard.refuse(b"/idle-route"));
    // the router may carry a prefix, so the end is matched
    assert!(guard.refuse(b"/screens/idle-route"));
    assert!(!guard.refuse(b"/hud"));
    assert!(!guard.refuse(b"/idle-route/more"));

    guard.off();
    assert!(!guard.refuse(b"/idle-route"));
}
