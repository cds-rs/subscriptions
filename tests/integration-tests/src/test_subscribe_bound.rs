//! `subscribe`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::subscribe` (engine-neutral, generic over
//! `B: TestSVM`). This module is the litesvm binding: `bind_scenarios!` emits one
//! `#[test]` per scenario, each calling the generic body with a fresh
//! `make_backend()`. The rendered reports are byte-identical to the old
//! `test_subscribe.rs`, since the titles, intents, and bodies are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    subscribe;
    subscribe_happy_path,
    subscribe_plan_sunset_rejected,
    subscribe_plan_expired_rejected,
    subscribe_mint_mismatch_rejected,
    subscribe_non_subscriber_subscription_authority_rejected,
    subscribe_no_subscription_authority_rejected,
    subscribe_with_sponsor,
    subscribe_duplicate_rejected,
    subscribe_rejects_stale_subscription_authority_generation,
    subscribe_rejects_stale_expected_terms,
);
