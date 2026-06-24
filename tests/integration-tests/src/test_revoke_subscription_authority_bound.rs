//! `revoke_subscription_authority`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::revoke_subscription_authority`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits one `#[test]` per scenario, each calling the
//! generic body with a fresh `make_backend()`. The rendered reports are
//! byte-identical to the old `test_revoke_subscription_authority.rs`, since the
//! titles, intents, and bodies are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    revoke_subscription_authority;
    revoke_subscription_authority_clears_delegate,
    revoke_subscription_authority_clears_delegate_token_2022,
    revoke_subscription_authority_works_after_close,
    revoke_subscription_authority_rejects_ata_mint_mismatch,
);
