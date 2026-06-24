//! The `transfer_subscription` authorization-alt behavior report, bound to the
//! quasar-svm engine.
//!
//! The body lives in `scenarios::suite::behavior_transfer_subscription_alt`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `behavior_transfer_subscription_alt_bound.rs`.
//! `bind_scenarios!` emits the `#[test]` calling the generic body with a fresh
//! `make_quasar_backend()`.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    behavior_transfer_subscription_alt;
    transfer_subscription_the_authorization_alt,
);
