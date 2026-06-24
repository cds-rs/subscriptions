//! `delete_plan`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::delete_plan` (engine-neutral, generic
//! over `B: TestSVM`); this is the quasar binding, the analogue of the litesvm
//! `test_delete_plan_bound.rs`. `bind_scenarios!` emits one `#[test]` per
//! scenario, each calling the generic body with a fresh `make_quasar_backend()`.
//!
//! The one balance-shaped assertion (`delete_plan_happy_path`'s "the merchant's
//! balance grew") holds via the rent the merchant reclaims, independent of the
//! transaction fee, so it needs no gate even though quasar models `Fee: 0`.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
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
