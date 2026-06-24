//! `revoke_subscription_authority`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::revoke_subscription_authority`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `test_revoke_subscription_authority_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_quasar_backend()`.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    revoke_subscription_authority;
    revoke_subscription_authority_clears_delegate,
    revoke_subscription_authority_clears_delegate_token_2022,
    revoke_subscription_authority_works_after_close,
    revoke_subscription_authority_rejects_ata_mint_mismatch,
);
