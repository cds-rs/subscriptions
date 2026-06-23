//! The one place the concrete engine is named. The generic harness lives in the
//! `scenarios` crate (engine-free, generic over `B: TestSVM`); binding a
//! different `TestSVM` engine later is a swap here, not in the World.

use litesvm_utils::{LiteSVM, LiteSvmBackend, TestSVM};
use std::path::Path;

use scenarios::tests::constants::PROGRAM_ID;

/// Build the concrete litesvm backend the suite runs on: a fresh `LiteSVM` with
/// the subscriptions program loaded from its built `.so`. [`World::new`]
/// (`scenarios::tests::utils::World`) takes whatever backend it is handed.
pub fn make_backend() -> LiteSvmBackend {
    let mut backend = LiteSvmBackend::new(LiteSVM::new());
    let so = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/subscriptions_program.so");
    let bytes = std::fs::read(so).unwrap();
    backend.deploy_program(PROGRAM_ID, &bytes);
    backend
}
