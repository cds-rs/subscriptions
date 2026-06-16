//! The smallest observed test: a bare System transfer, no program of our own.
//!
//! It proves two things at once: the dependency graph resolves (the forked
//! litesvm unifying with `litesvm-token` and the path-dep'd `litesvm-utils`), and
//! the render path works end to end on a single frame. It is also the simplest
//! possible shape of a DSL test: build the backend, `send`, turn the record into
//! a `TransactionResult`, render.

use litesvm_utils::{Aliases, Keypair, LiteSVM, LiteSvmBackend, Pubkey, Signer, TestSVM, TransactionResult};
use solana_instruction::{AccountMeta, Instruction};

#[test]
fn observed_backend_renders_a_system_transfer() {
    let mut backend = LiteSvmBackend::new(LiteSVM::new());
    let payer = Keypair::new();
    backend.fund_sol(&payer.pubkey(), 1_000_000_000);

    // A bare System transfer (instruction 2 = Transfer): a frame both the logs
    // and the per-frame trace witness, with no program of our own deployed.
    let dest = Pubkey::new_unique();
    let mut data = vec![2u8, 0, 0, 0];
    data.extend_from_slice(&2_000_000u64.to_le_bytes());
    let ix = Instruction {
        program_id: Pubkey::default(), // System program is the all-zero address.
        accounts: vec![AccountMeta::new(payer.pubkey(), true), AccountMeta::new(dest, false)],
        data,
    };

    let record = backend.send(&[ix], &[&payer]);
    assert!(record.error.is_none(), "transfer should succeed: {:?}", record.error);
    assert!(record.trace.is_some(), "the in-memory backend captured the per-frame trace");

    let result: TransactionResult = record.into();
    let rendered = result.with_aliases(Aliases::with_well_known()).logs_structured_string();
    println!("{rendered}");
    assert!(rendered.contains("System"), "aliased render should name the System program:\n{rendered}");
}

