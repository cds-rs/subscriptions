//! Cantina HIGH finding 3.1.1, bound to the quasar-svm engine.
//!
//! The body lives in `scenarios::suite::audit_regression_3_1_1` (engine-neutral,
//! generic over `B: TestSVM`); this is the quasar binding, the analogue of the
//! litesvm `audit_regression_3_1_1_bound.rs`. `bind_scenarios!` emits the
//! `#[test]` calling the generic body with a fresh `make_quasar_backend()`.

use subscriptions_quasar_svm::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    audit_regression_3_1_1;
    finding_3_1_1_recurring_pull_before_start_ts,
);
