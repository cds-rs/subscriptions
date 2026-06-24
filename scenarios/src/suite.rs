//! The engine-neutral scenario bodies, one module per instruction family.
//!
//! Each `pub fn name<B: TestSVM>(backend: B)` is a lifted test body: it builds a
//! `World` over the handed backend and runs the same flow the engine workspaces
//! bind through [`crate::bind_scenarios`]. The bodies are byte-identical to the
//! `test_*.rs` files they were lifted from (titles, intents, asserts), save for
//! receiving the backend as a parameter instead of naming a concrete engine, and
//! gating fee-dependent assertions on [`crate::tests::utils::World::capabilities`].

pub mod audit_regression_3_1_1;
pub mod audit_regression_3_1_3;
pub mod audit_regression_ghost_plan;
pub mod behavior_transfer_subscription_alt;
pub mod cancel_subscription;
pub mod close_subscription_authority;
pub mod create_fixed_delegation;
pub mod create_plan;
pub mod create_recurring_delegation;
pub mod delete_plan;
pub mod initialize_subscription_authority;
pub mod resume_subscription;
pub mod revoke_abandoned_delegation;
pub mod revoke_delegation;
pub mod revoke_subscription_authority;
pub mod subscribe;
pub mod transfer_fixed_delegation;
pub mod transfer_recurring_delegation;
pub mod transfer_subscription;
pub mod update_plan;
