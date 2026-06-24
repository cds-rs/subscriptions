//! `revoke_abandoned_delegation`, bound to the litesvm engine.
//!
//! The bodies live in `scenarios::suite::revoke_abandoned_delegation`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits one `#[test]` per scenario, each calling the
//! generic body with a fresh `make_backend()`. The rendered reports are
//! byte-identical to the old `test_revoke_abandoned_delegation.rs`, since the
//! titles, intents, and bodies are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    revoke_abandoned_delegation;
    sponsor_recovers_no_expiry_fixed_delegation_after_authority_closed,
    payer_recovers_no_expiry_recurring_delegation_after_authority_closed,
    payer_recovers_delegation_after_authority_reinit_bumps_init_id,
    revoke_abandoned_rejects_live_delegation,
    revoke_abandoned_rejects_non_sponsor_caller,
    revoke_abandoned_rejects_unbound_authority_account,
);
