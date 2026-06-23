//! SPIKE crate: binds the engine-neutral `scenarios` harness to the
//! quasar-svm engine. The one place the concrete engine is named for quasar,
//! mirroring `tests/integration-tests/src/utils/backend.rs` (litesvm).

use scenarios::tests::constants::PROGRAM_ID;
use std::path::Path;
use testsvm::TestSVM;
use testsvm_quasar::QuasarBackend;

/// Build the quasar backend the spike runs on: a fresh `QuasarBackend` with the
/// subscriptions program loaded from its built `.so`. `World::new`
/// (`scenarios::world::World`) takes whatever `B: TestSVM` it is handed, and
/// registers the program-id alias + instruction/error/event vocabulary itself.
pub fn make_quasar_backend() -> QuasarBackend {
    let mut backend = QuasarBackend::new();
    let so = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/subscriptions_program.so");
    let bytes = std::fs::read(&so).unwrap_or_else(|e| panic!("read {}: {e}", so.display()));
    backend.deploy_program(PROGRAM_ID, &bytes);
    backend
}
