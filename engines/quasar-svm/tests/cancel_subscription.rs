//! `cancel_subscription`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::cancel_subscription` (engine-neutral,
//! generic over `B: TestSVM`); this is the quasar binding, the analogue of the
//! litesvm `test_cancel_subscription_bound.rs`. `bind_scenarios!` emits one
//! `#[test]` per scenario, each calling the generic body with a fresh
//! `make_quasar_backend()`.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    cancel_subscription;
    cancel_subscription_happy_path,
    cancel_at_exact_end_ts_keeps_final_period_billable,
    cancel_subscription_non_subscriber_rejected,
    cancel_subscription_already_cancelled_rejected,
    test_cancel_subscription_version_mismatch,
    cancel_subscription_ghost_plan_expires_immediately,
    cancel_subscription_caps_at_plan_end_ts,
    cancel_subscription_after_plan_expired_allows_immediate_revoke,
);
