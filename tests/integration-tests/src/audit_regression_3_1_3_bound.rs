//! Cantina HIGH finding 3.1.3, bound to the litesvm engine.
//!
//! The body lives in `scenarios::suite::audit_regression_3_1_3`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits the `#[test]` calling the generic body with a
//! fresh `make_backend()`. The rendered report is byte-identical to the old
//! `audit_regression_3_1_3.rs`.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    audit_regression_3_1_3;
    finding_3_1_3_prefunded_pda_blocks_creation,
);
