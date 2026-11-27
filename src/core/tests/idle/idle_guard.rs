use quirks_core::idle::IdleGuard;

#[test]
fn the_idle_screen_is_refused_only_while_on() {
    let guard = IdleGuard::new();
    assert!(!guard.refuse("push"));

    guard.on();
    assert!(guard.refuse("push"));
    assert!(guard.refuse("replace"));
    assert_eq!(guard.refused(), 2);

    guard.off();
    assert!(!guard.refuse("push"));
}

#[test]
fn a_new_on_counts_from_zero() {
    let guard = IdleGuard::new();
    guard.on();
    assert!(guard.refuse("push"));
    guard.off();

    guard.on();
    assert_eq!(guard.refused(), 0);
}
