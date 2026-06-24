//! Cantina HIGH finding 3.1.3, bound to the quasar-svm engine.
//!
//! The body lives in `scenarios::suite::audit_regression_3_1_3`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `audit_regression_3_1_3_bound.rs`. `bind_scenarios!`
//! emits the `#[test]` calling the generic body with a fresh
//! `make_quasar_backend()`.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    audit_regression_3_1_3;
    finding_3_1_3_prefunded_pda_blocks_creation,
);
