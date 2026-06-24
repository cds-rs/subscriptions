//! `create_fixed_delegation`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::create_fixed_delegation` (engine-neutral,
//! generic over `B: TestSVM`). This module is the litesvm binding:
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_backend()`. The rendered reports are byte-identical to
//! the old `test_create_fixed_delegation.rs`, since the titles, intents, and
//! bodies are unchanged (litesvm models fees, so the fee-gated "Alice paid the
//! revoke fee" check runs here exactly as before).

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    create_fixed_delegation;
    create_fixed_delegation_with_sponsor,
    create_fixed_delegation,
    create_fixed_delegation_rejects_stale_subscription_authority_generation,
    create_fixed_delegation_with_prefunded_pda,
    create_delegation_without_subscription_authority,
    create_delegation_wrong_pda,
    create_delegation_duplicate_nonce,
    create_multiple_delegations_different_nonces,
    writable_accounts_must_be_writable,
    signer_accounts_must_be_signers,
    create_fixed_delegation_with_expiry_in_past,
    create_fixed_delegation_with_zero_expiry,
);
