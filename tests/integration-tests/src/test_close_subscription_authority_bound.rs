//! `close_subscription_authority`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::close_subscription_authority`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits one `#[test]` per scenario, each calling the
//! generic body with a fresh `make_backend()`. The rendered reports are
//! byte-identical to the old `test_close_subscription_authority.rs`, since the
//! titles, intents, and bodies are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    close_subscription_authority;
    close_subscription_authority,
    non_owner_cannot_close,
    writable_accounts_must_be_writable,
    signer_accounts_must_be_signers,
    close_returns_rent_to_sponsor,
    close_without_receiver_when_sponsor_funded_fails,
    close_with_wrong_receiver_unauthorized,
    idempotent_init_preserves_original_payer,
    closed_account_is_zeroed,
);
