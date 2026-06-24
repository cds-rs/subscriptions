//! Cantina HIGH finding 3.1.1, bound to the litesvm engine.
//!
//! The body lives in `scenarios::suite::audit_regression_3_1_1` (engine-neutral,
//! generic over `B: TestSVM`). This module is the litesvm binding:
//! `bind_scenarios!` emits the `#[test]` calling the generic body with a fresh
//! `make_backend()`. The rendered report is byte-identical to the old
//! `audit_regression_3_1_1.rs`, since the title, intent, and body are unchanged.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    audit_regression_3_1_1;
    finding_3_1_1_recurring_pull_before_start_ts,
);
