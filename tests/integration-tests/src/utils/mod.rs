//! The litesvm engine binding plus the harness surface re-export. The generic
//! harness lives in the `scenarios` crate; this module names the one
//! concrete-engine item (`make_backend`) and re-exports `scenarios`' top-level
//! harness types so dependents reaching for `tests_subscriptions::utils::*`
//! (the observed twin's `ObservedResultExt`) keep resolving.

pub mod backend;

pub use backend::make_backend;
pub use scenarios::world::{as_pubkey, ObservedResultExt, StagedSubscription, World};
