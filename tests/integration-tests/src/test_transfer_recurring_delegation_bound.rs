//! `transfer_recurring_delegation`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::transfer_recurring_delegation`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits one `#[test]` per scenario, each calling the
//! generic body with a fresh `make_backend()`. The rendered reports are
//! byte-identical to the old `test_transfer_recurring_delegation.rs`.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    transfer_recurring_delegation;
    test_recurring_transfer_success,
    test_recurring_transfer_exceeds_period_limit,
    test_recurring_transfer_expired,
    test_recurring_transfer_multiple_periods,
    test_recurring_transfer_skip_multiple_periods,
    test_recurring_transfer_skip_period_cannot_double_claim,
    recurring_delegation_rejects_transfer_with_different_mint_authority,
    recurring_transfer_rejects_approved_non_canonical_source,
    writable_accounts_must_be_writable,
    signer_accounts_must_be_signers,
    test_recurring_transfer_delegator_mismatch_exploit,
    test_recurring_transfer_token_revoke,
    test_recurring_transfer_to_third_party,
    test_recurring_transfer_version_mismatch,
    test_recurring_transfer_stale_subscription_authority,
    test_recurring_transfer_not_started,
    test_recurring_transfer_within_drift_window,
    test_recurring_rollover_blocked_at_expiry_boundary,
    test_recurring_transfer_past_drift_window,
    test_recurring_transfer_token_2022_transfer_fee,
    test_recurring_transfer_token_2022_confidential_transfer_public_balance,
    test_recurring_transfer_token_2022_unconfigured_transfer_hook,
);
