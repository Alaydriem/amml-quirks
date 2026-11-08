// The game functions the quirks mod hooks. Each body calls the original or refuses through one
// QuirksRuntime query; bool results cross as bytes, so no bool invariant is assumed of the game registers

use amml_msvc::MsvcString;
use amml_sdk::amml_detour;

use crate::services::QuirksRuntime;

// hands the bytes of the `std::string const&` at `route` to `body`, borrowed for the call only; None for a null
// route, which is not read
///
/// # Safety
///
/// `route` is 0 or points to a live MSVC `std::string` for the whole call.
#[allow(unsafe_code)]
pub unsafe fn with_route_text<R>(route: usize, body: impl FnOnce(&[u8]) -> R) -> Option<R> {
    // SAFETY: the caller guarantees `route` is 0 or a live string; `as_ref` turns 0 into None
    let text = unsafe { (route as *const MsvcString).as_ref() }?;
    Some(body(text.as_bytes()))
}

// whether the idle guard refuses the route at `route`
#[allow(unsafe_code)]
fn refuses(route: usize) -> bool {
    let Some(runtime) = QuirksRuntime::get() else { return false };
    // SAFETY: the router passes a live `std::string const&`, or null, and the call ends before the router returns
    unsafe { with_route_text(route, |text| runtime.idle().guard().refuse(text)) == Some(true) }
}

// bool ClientInstance::update(bool isInitFinished) (26.60: update(bool, bool)): once per frame on the game thread
// The third argument is carried as a raw u64 so the register reaches the original bit for bit
amml_detour!(pub mod client_update(this: usize, init_finished: u8, extra: u64) -> u8 |original| {
    let result = original(this, init_finished, extra);
    if let Some(runtime) = QuirksRuntime::get() {
        runtime.tick(this);
    }
    result
});

// bool OreUI::Router::_pushRoute(std::string const& route, RouterPushMode mode): a refused push returns false, as
// for a route the router does not take
amml_detour!(pub mod route_push(this: usize, route: usize, mode: u32) -> u8 |original| {
    if refuses(route) {
        return 0;
    }
    original(this, route, mode)
});

// bool OreUI::Router::replaceRoute(std::string const& route): a refused replace returns false
amml_detour!(pub mod route_replace(this: usize, route: usize) -> u8 |original| {
    if refuses(route) {
        return 0;
    }
    original(this, route)
});
