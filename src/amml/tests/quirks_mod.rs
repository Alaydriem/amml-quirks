use alaydriem_quirks::QuirksMod;
use alaydriem_quirks::detours::with_route_text;
use amml_game::HookOrder;
use amml_sdk::AmmlMod;

#[test]
fn module_matches_the_mod_id() {
    assert_eq!(QuirksMod::MODULE, format!("@{}/{}", QuirksMod::NAMESPACE, QuirksMod::NAME));
    assert_eq!(QuirksMod::id(), QuirksMod::MODULE);
}

#[test]
fn only_the_client_update_is_required() {
    let required: Vec<_> = QuirksMod::hooks().iter().filter(|h| h.required).map(|h| h.entry).collect();
    assert_eq!(required, ["ClientInstance.update"]);
}

#[test]
fn route_hooks_are_writers() {
    for spec in QuirksMod::hooks().iter().filter(|h| h.entry.starts_with("OreUIRouter.")) {
        assert_eq!(spec.priority, HookOrder::WRITE);
    }
}

#[test]
fn amml_hcl_names_the_entries_the_mod_uses() {
    let document: hcl::Value = hcl::from_str(include_str!("../amml.hcl")).unwrap();
    let hcl::Value::Object(map) = document else { panic!("the document is not an object") };
    let Some(hcl::Value::Array(listed)) = map.get("entries") else { panic!("no entries") };
    let mut listed: Vec<&str> = listed.iter().filter_map(hcl::Value::as_str).collect();
    let mut used: Vec<&str> = QuirksMod::hooks().iter().map(|h| h.entry).collect();
    used.extend(QuirksMod::wanted().iter().map(|(id, _)| id.as_str()));
    listed.sort_unstable();
    used.sort_unstable();
    assert_eq!(listed, used);
}

#[test]
fn a_null_route_is_not_read() {
    // SAFETY: 0 is the null route, which `with_route_text` does not read
    let read = unsafe { with_route_text(0, |_| ()) };
    assert!(read.is_none());
}
