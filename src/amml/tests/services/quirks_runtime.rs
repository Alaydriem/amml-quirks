use alaydriem_quirks::services::QuirksRuntime;
use amml_api::ApiError;
use amml_api::amml_abi::AmmlStatus;
use quirks_core::idle::FireResult;

#[test]
fn a_call_the_game_ran_is_a_fire() {
    assert_eq!(QuirksRuntime::fire_result(Ok(())), FireResult::Fired);
}

#[test]
fn a_build_without_the_call_turns_the_keep_alive_off() {
    assert_eq!(QuirksRuntime::fire_result(Err(ApiError::NotAvailable)), FireResult::NotAvailable);
}

#[test]
fn a_refused_or_early_call_is_tried_again() {
    for error in [ApiError::Refused, ApiError::NotReady, ApiError::Stale, ApiError::Loader(AmmlStatus::FAILED)] {
        assert_eq!(QuirksRuntime::fire_result(Err(error)), FireResult::NotReady);
    }
}
