//! `create_plan`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::create_plan` (engine-neutral, generic
//! over `B: TestSVM`); this is the quasar binding, the analogue of the litesvm
//! `test_create_plan_bound.rs`. `bind_scenarios!` emits one `#[test]` per
//! scenario, each calling the generic body with a fresh `make_quasar_backend()`.
//! The same titles run here; the rendered surface differs by engine, hence the
//! separate report dir set in `.cargo/config.toml`.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
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
