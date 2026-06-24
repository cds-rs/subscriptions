//! `resume_subscription`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::resume_subscription` (engine-neutral,
//! generic over `B: TestSVM`); this is the quasar binding, the analogue of the
//! litesvm `test_resume_subscription_bound.rs`. `bind_scenarios!` emits one
//! `#[test]` per scenario, each calling the generic body with a fresh
//! `make_quasar_backend()`.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    resume_subscription;
    resume_subscription_happy_path,
    resume_subscription_rejected_at_cancelled_period_end,
    resume_subscription_not_cancelled_rejected,
    resume_subscription_non_subscriber_rejected,
    resume_subscription_plan_mismatch_rejected,
    resume_subscription_rejected_after_cancelled_period_elapsed,
    resume_subscription_rejected_when_plan_expired,
    resume_subscription_allows_when_plan_sunset,
    resume_subscription_rejected_when_plan_deleted,
    resume_subscription_cancel_resume_cancel_across_period_boundary,
    resume_subscription_version_mismatch,
);
