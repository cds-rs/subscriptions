//! The `transfer_subscription` authorization-alt behavior report, bound to the
//! litesvm engine.
//!
//! The body lives in `scenarios::suite::behavior_transfer_subscription_alt`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits the `#[test]` calling the generic body with a
//! fresh `make_backend()`. The rendered report is byte-identical to the old
//! `behavior_transfer_subscription_alt.rs`.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    behavior_transfer_subscription_alt;
    transfer_subscription_the_authorization_alt,
);
