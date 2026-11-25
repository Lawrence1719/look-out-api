use look_out_api::state::{advance, State, Status, Transition};

#[test]
fn flapping_does_not_alert_and_recovery_requires_two_successes() {
    let mut state = State::default();
    assert_eq!(advance(&mut state, true, 3, 2), None);
    assert_eq!(state.status, Status::Up);
    for ok in [false, false, true, false, true] {
        assert_eq!(advance(&mut state, ok, 3, 2), None);
    }
    assert_eq!(advance(&mut state, false, 3, 2), None);
    assert_eq!(advance(&mut state, false, 3, 2), None);
    assert_eq!(advance(&mut state, false, 3, 2), Some(Transition::Down));
    assert_eq!(advance(&mut state, false, 3, 2), None);
    assert_eq!(advance(&mut state, true, 3, 2), None);
    assert_eq!(advance(&mut state, true, 3, 2), Some(Transition::Recovered));
    assert_eq!(advance(&mut state, true, 3, 2), None);
}

#[test]
fn startup_failure_uses_threshold_and_restart_keeps_counters() {
    let mut state = State::default();
    advance(&mut state, false, 3, 2);
    let saved = serde_json::to_string(&state).expect("serialize");
    let mut restored = serde_json::from_str(&saved).expect("deserialize");
    assert_eq!(advance(&mut restored, false, 3, 2), None);
    assert_eq!(advance(&mut restored, false, 3, 2), Some(Transition::Down));
}
