//! `transfer_subscription`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::test_transfer_subscription`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `test_transfer_subscription_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_quasar_backend()`.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    test_transfer_subscription;
    test_transfer_subscription_success,
    test_transfer_subscription_puller_authorized,
    test_transfer_subscription_unauthorized_caller,
    test_transfer_subscription_multiple_pulls_within_period,
    test_transfer_subscription_exceeds_period_limit,
    test_transfer_subscription_period_rollover,
    test_transfer_subscription_plan_expired,
    test_transfer_subscription_subscription_cancelled,
    test_transfer_subscription_cancelled_allows_current_period,
    test_transfer_subscription_cancelled_blocks_next_period,
    test_transfer_subscription_destination_valid,
    test_transfer_subscription_destination_invalid,
    test_transfer_subscription_no_destinations_any_receiver,
    test_transfer_subscription_wrong_subscription_for_plan,
    test_transfer_subscription_zero_amount,
    test_transfer_subscription_sunset_allows_transfer,
    test_transfer_subscription_plan_closed,
    writable_accounts_must_be_writable,
    signer_accounts_must_be_signers,
    test_subscription_transfer_version_mismatch,
    test_subscription_transfer_stale_subscription_authority,
    test_transfer_subscription_ghost_plan_rejected,
);
