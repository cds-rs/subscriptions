//! `update_plan`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::update_plan` (engine-neutral, generic
//! over `B: TestSVM`); this is the quasar binding, the analogue of the litesvm
//! `test_update_plan_bound.rs`. `bind_scenarios!` emits one `#[test]` per
//! scenario, each calling the generic body with a fresh `make_quasar_backend()`.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    update_plan;
    update_plan_happy_path,
    update_plan_preserves_immutable_fields,
    update_plan_not_owner,
    update_plan_invalid_status,
    update_plan_end_ts_in_past,
    update_plan_clear_end_ts,
    update_plan_sunset_is_terminal,
    update_plan_no_op,
    update_plan_sunset_requires_end_ts,
    update_plan_at_exact_expiry_boundary,
    update_plan_expired,
    update_plan_add_pullers,
    update_plan_remove_pullers_owner_still_authorized,
    update_plan_replace_pullers,
    update_plan_max_pullers,
    update_plan_rejects_near_immediate_end_ts,
);
