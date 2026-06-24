//! Cantina HIGH findings 3.1.2 and 3.1.4 (the ghost-plan attack), bound to the
//! litesvm engine.
//!
//! The bodies live in `scenarios::suite::audit_regression_ghost_plan`
//! (engine-neutral, generic over `B: TestSVM`). This module is the litesvm
//! binding: `bind_scenarios!` emits one `#[test]` per scenario, each calling the
//! generic body with a fresh `make_backend()`. The rendered reports are
//! byte-identical to the old `audit_regression_ghost_plan.rs`.

scenarios::bind_scenarios!(
    crate::tests::utils::make_backend;
    audit_regression_ghost_plan;
    finding_3_1_2_ghost_plan_inflated_amount_drains,
    finding_3_1_4_ghost_plan_extended_end_ts_siphons,
);
