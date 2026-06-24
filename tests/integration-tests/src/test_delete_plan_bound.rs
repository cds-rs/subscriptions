//! `delete_plan`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::delete_plan` (engine-neutral, generic
//! over `B: TestSVM`). This module is the litesvm binding: `bind_scenarios!`
//! emits one `#[test]` per scenario, each calling the generic body with a fresh
//! `make_backend()`. The rendered reports are byte-identical to the old
//! `test_delete_plan.rs`, since the titles, intents, and bodies are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    delete_plan;
    delete_plan_happy_path,
    delete_plan_not_owner,
    delete_active_expired_plan,
    delete_active_not_expired_fails,
    delete_sunset_not_expired_fails,
    delete_sunset_exactly_at_end_ts_fails,
    delete_plan_double_delete_fails,
    delete_plan_data_zeroed,
);
