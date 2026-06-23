//! Engine-neutral scenario vocabulary for the subscriptions suite.
//!
//! One set of test bodies, expressed against the `testsvm` neutral types,
//! driving the program through its wire interface. Each engine workspace binds
//! a concrete `B: TestSVM` via a generated `#[test]` shim.
//!
//! The program's own items (`instructions`, `state`, `event_engine`,
//! `SubscriptionsError`, `Header`, ...) are re-exported so the harness's
//! `crate::...` paths resolve here exactly as they did in the engine workspace
//! it was lifted from; the engine binding (`make_backend`) is the one piece
//! that stays with the concrete-engine crate.

pub use subscriptions::*;

pub mod constants;
pub mod cu_tracker;
pub mod helpers;
pub mod idl;
pub mod pda;
pub mod suite;
pub mod test_helpers;
pub mod world;

pub use helpers::{days, minutes, rent_exempt_lamports};

/// The harness's internal modules reach for `crate::tests::pda`,
/// `crate::tests::utils::World`, etc. (the path shape it carried over from the
/// engine workspace). Replicating that facade here keeps every `crate::tests::`
/// path in the moved code resolving unchanged.
pub mod tests {
    pub use crate::{constants, cu_tracker, idl, pda};

    pub mod utils {
        pub use crate::test_helpers::*;
        pub use crate::world::{as_pubkey, ObservedResultExt, StagedSubscription, World};
    }
}

/// Bind a set of engine-neutral scenario bodies to a concrete backend, emitting
/// one `#[test] fn` per scenario.
///
/// Each scenario in [`suite`] is a `pub fn name<B: TestSVM>(backend: B)`; this
/// macro generates the `#[test]` shim that calls it with a fresh backend from
/// `$factory()`. An engine workspace invokes it once per instruction family:
///
/// ```ignore
/// scenarios::bind_scenarios!(
///     crate::tests::utils::make_backend;
///     subscribe;
///     subscribe_happy_path,
///     subscribe_plan_sunset_rejected,
/// );
/// ```
///
/// `$factory` is any path to a `fn() -> B` (the one place a workspace names its
/// concrete engine); `$module` is the [`suite`] submodule the names live in; the
/// trailing list is the bare scenario fn names. The generated test name matches
/// the scenario fn name, so a run reads the same on every engine.
#[macro_export]
macro_rules! bind_scenarios {
    ($factory:path; $module:ident; $($scenario:ident),+ $(,)?) => {
        $(
            #[test]
            fn $scenario() {
                $crate::suite::$module::$scenario($factory());
            }
        )+
    };
}
