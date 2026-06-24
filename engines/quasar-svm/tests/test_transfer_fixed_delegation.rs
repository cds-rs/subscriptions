//! `transfer_fixed_delegation`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::transfer_fixed_delegation`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `test_transfer_fixed_delegation_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_quasar_backend()`.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    transfer_fixed_delegation;
    test_fixed_transfer_success,
    test_fixed_transfer_token_2022_transfer_fee,
    test_fixed_transfer_token_2022_confidential_transfer_public_balance,
    test_fixed_transfer_token_2022_unconfigured_transfer_hook,
    test_fixed_transfer_token_2022_active_transfer_hook,
    active_hook_transfer_without_validation_pda_fails,
    test_fixed_transfer_multiple_times,
    test_fixed_transfer_exceeds_amount,
    test_fixed_transfer_expired,
    test_fixed_transfer_wrong_signer,
    test_fixed_transfer_to_third_party,
    fixed_delegation_rejects_transfer_with_different_mint_authority,
    fixed_transfer_rejects_approved_non_canonical_source,
    writable_accounts_must_be_writable,
    signer_accounts_must_be_signers,
    test_fixed_transfer_delegator_mismatch_exploit,
    test_fixed_transfer_version_mismatch,
    test_fixed_transfer_stale_subscription_authority,
    test_close_subscription_authority_blocks_all_transfers,
    test_fixed_transfer_within_drift_window,
    test_fixed_transfer_past_drift_window,
);
