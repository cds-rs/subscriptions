//! `create_recurring_delegation`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::create_recurring_delegation`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits one `#[test]` per scenario, each calling the
//! generic body with a fresh `make_backend()`. The rendered reports are
//! byte-identical to the old `test_create_recurring_delegation.rs`, since the
//! titles, intents, and bodies are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
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
