//! `create_recurring_delegation`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::create_recurring_delegation`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `test_create_recurring_delegation_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_quasar_backend()`.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    create_recurring_delegation;
    create_recurring_delegation,
    create_recurring_delegation_rejects_stale_subscription_authority_generation,
    create_recurring_delegation_with_past_start_ts,
    create_recurring_delegation_with_zero_period,
    create_recurring_delegation_with_start_ts_greater_than_expiry_ts,
    create_recurring_delegation_with_period_exceeding_max,
    create_recurring_delegation_with_sentinel_start_starts_at_landing,
    create_recurring_delegation_sentinel_start_requires_expiry,
    create_recurring_delegation_sentinel_start_rejects_elapsed_expiry,
    create_recurring_delegation_with_zero_expiry,
);
