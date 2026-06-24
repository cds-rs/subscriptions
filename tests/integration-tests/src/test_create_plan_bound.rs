//! `create_plan`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::create_plan` (engine-neutral, generic
//! over `B: TestSVM`). This module is the litesvm binding: `bind_scenarios!`
//! emits one `#[test]` per scenario, each calling the generic body with a fresh
//! `make_backend()`. The rendered reports are byte-identical to the old
//! `test_create_plan.rs`, since the titles, intents, and bodies are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    create_plan;
    create_plan_happy_path,
    create_plan_no_expiry,
    create_plan_period_hours_zero,
    create_plan_period_hours_exceeds_max,
    create_plan_amount_zero,
    create_plan_no_destinations,
    create_plan_expired_end_ts,
    create_plan_end_ts_before_first_period,
    create_plan_wrong_pda,
    create_plan_mint_mismatch_attack,
    create_plan_prefunded_pda,
    create_plan_duplicate_plan_id,
);
