//! `revoke_abandoned_delegation`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::revoke_abandoned_delegation`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `test_revoke_abandoned_delegation_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_quasar_backend()`. The rent-recovery assertions use a
//! 10_000-lamport tolerance, which holds at Fee:0, so none are fee-gated.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    revoke_abandoned_delegation;
    sponsor_recovers_no_expiry_fixed_delegation_after_authority_closed,
    payer_recovers_no_expiry_recurring_delegation_after_authority_closed,
    payer_recovers_delegation_after_authority_reinit_bumps_init_id,
    revoke_abandoned_rejects_live_delegation,
    revoke_abandoned_rejects_non_sponsor_caller,
    revoke_abandoned_rejects_unbound_authority_account,
);
