//! The litesvm binding of the subscriptions suite. The harness (World, scenario
//! verbs, account builders, PDA/IDL/CU helpers) is the engine-free `scenarios`
//! crate; this crate re-exports it under the same `crate::...` paths the 21
//! `test_*.rs` files were written against, then layers the one concrete-engine
//! item, `make_backend`, on top.

// The program items (`instructions`, `state`, `SubscriptionsError`, `Header`,
// ...) under `crate::...`, the paths the 21 `test_*.rs` files use. Taken
// straight from `subscriptions` rather than through `scenarios`' glob so the
// `tests` / `world` / `helpers` modules `scenarios` adds do not collide with the
// facade this crate defines below.
pub use subscriptions::*;

/// The litesvm engine binding (`make_backend`) plus the re-exported
/// `tests_subscriptions::utils::*` harness surface.
pub mod utils;

/// The harness facade, mirroring `scenarios::tests` so every `crate::tests::...`
/// path in the test files resolves. The only addition is the engine binding:
/// `tests::utils::make_backend` comes from this crate (`crate::utils`), not from
/// the engine-free harness.
pub mod tests {
    pub use scenarios::tests::{constants, cu_tracker, idl, pda};

    pub mod utils {
        pub use crate::utils::make_backend;
        pub use scenarios::tests::utils::*;
    }
}

#[cfg(test)]
mod audit_regression_3_1_1;
#[cfg(test)]
mod behavior_transfer_subscription_alt;
#[cfg(test)]
mod audit_regression_3_1_3;
#[cfg(test)]
mod audit_regression_ghost_plan;
#[cfg(test)]
mod test_cancel_subscription;
#[cfg(test)]
mod test_close_subscription_authority;
#[cfg(test)]
mod test_create_fixed_delegation;
#[cfg(test)]
mod test_create_plan;
#[cfg(test)]
mod test_create_recurring_delegation;
#[cfg(test)]
mod test_delete_plan;
#[cfg(test)]
mod test_initialize_subscription_authority;
#[cfg(test)]
mod test_resume_subscription;
#[cfg(test)]
mod test_revoke_abandoned_delegation;
#[cfg(test)]
mod test_subscribe_bound;
#[cfg(test)]
mod test_revoke_delegation;
#[cfg(test)]
mod test_revoke_subscription_authority;
#[cfg(test)]
mod test_transfer_fixed_delegation;
#[cfg(test)]
mod test_transfer_recurring_delegation;
#[cfg(test)]
mod test_transfer_subscription;
#[cfg(test)]
mod test_update_plan;
