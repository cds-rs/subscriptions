//! `close_subscription_authority`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::close_subscription_authority`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `test_close_subscription_authority_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_quasar_backend()`.
//!
//! The balance-shaped assertions hold via the rent returned by the close,
//! independent of the transaction fee, so they need no gate under quasar's
//! `Fee: 0`.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
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
