//! `subscribe`, bound to the quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::subscribe` (engine-neutral, generic over
//! `B: TestSVM`); this is the quasar binding, the analogue of the litesvm
//! `test_subscribe_bound.rs`. `bind_scenarios!` emits one `#[test]` per scenario,
//! each calling the generic body with a fresh `make_quasar_backend()`. The same
//! titles run here, so a quasar run reads the same as the litesvm one (the
//! rendered surface differs by engine, hence the separate report dir set in
//! `.cargo/config.toml`).
//!
//! Fee-dependent assertions are gated inside the bodies on
//! `world.capabilities().fees`. quasar-svm models fees, so on this engine they
//! run; the gate exists for signature-less engines. The one balance-shaped
//! assertion (`subscribe_with_sponsor`'s "the sponsor was charged") holds via the
//! rent the sponsor pays for the Subscription PDA, independent of the fee, so it
//! needs no gate.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    subscribe;
    subscribe_happy_path,
    subscribe_plan_sunset_rejected,
    subscribe_plan_expired_rejected,
    subscribe_mint_mismatch_rejected,
    subscribe_non_subscriber_subscription_authority_rejected,
    subscribe_no_subscription_authority_rejected,
    subscribe_with_sponsor,
    subscribe_duplicate_rejected,
    subscribe_rejects_stale_subscription_authority_generation,
    subscribe_rejects_stale_expected_terms,
);
