//! Cantina HIGH findings 3.1.2 and 3.1.4 (the ghost-plan attack), bound to the
//! quasar-svm engine.
//!
//! The bodies live in `scenarios::suite::audit_regression_ghost_plan`
//! (engine-neutral, generic over `B: TestSVM`); this is the quasar binding, the
//! analogue of the litesvm `audit_regression_ghost_plan_bound.rs`.
//! `bind_scenarios!` emits one `#[test]` per scenario, each calling the generic
//! body with a fresh `make_quasar_backend()`.

use subscriptions_quasar_spike::make_quasar_backend;

scenarios::bind_scenarios!(
    make_quasar_backend;
    audit_regression_ghost_plan;
    finding_3_1_2_ghost_plan_inflated_amount_drains,
    finding_3_1_4_ghost_plan_extended_end_ts_siphons,
);
