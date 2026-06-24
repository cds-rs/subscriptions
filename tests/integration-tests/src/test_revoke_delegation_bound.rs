//! `revoke_delegation` (and `revoke_subscription`), bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::revoke_delegation` (engine-neutral,
//! generic over `B: TestSVM`). This module is the litesvm binding:
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_backend()`. The rendered reports are byte-identical to
//! the old `test_revoke_delegation.rs`, since the titles, intents, and bodies are
//! unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    revoke_delegation;
    revoke_fixed_delegation,
    revoke_recurring_delegation,
    non_delegator_cannot_revoke,
    closed_account_is_zeroed,
    revoke_with_wrong_receiver_returns_unauthorized,
    writable_accounts_must_be_writable,
    signer_accounts_must_be_signers,
    revoke_subscription_without_cancel_rejected,
    revoke_subscription_after_cancel_succeeds,
    revoke_subscription_with_future_expires_at_ts_rejected,
    test_revoke_fixed_version_agnostic,
    test_revoke_recurring_version_agnostic,
    test_revoke_subscription_version_mismatch,
    sponsor_can_revoke_expired_fixed_delegation,
    sponsor_can_revoke_expired_recurring_delegation,
    sponsor_cannot_revoke_non_expired_fixed_delegation,
    sponsor_cannot_revoke_non_expired_recurring_delegation,
    sponsor_cannot_revoke_no_expiry_delegation,
    sponsor_cannot_revoke_within_drift_window,
    delegator_can_revoke_sponsor_funded_before_expiry,
    attacker_cannot_revoke_sponsor_funded_delegation,
    sponsor_revoke_subscription_when_plan_ended,
    sponsor_revoke_subscription_when_plan_closed,
    sponsor_revoke_subscription_when_plan_recreated_with_different_terms,
    sponsor_revoke_subscription_when_cancelled_and_expired,
    sponsor_revoke_active_subscription_rejected,
    sponsor_revoke_subscription_with_wrong_plan_pda_rejected,
    attacker_cannot_revoke_sponsor_funded_subscription,
    subscriber_revoke_routes_rent_to_sponsor,
);
