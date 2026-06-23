//! Neutral assertions on the engine-neutral [`testsvm::model::Transaction`], the
//! record every `TestSVM::send` returns. The World-path send returns this model
//! directly (no litesvm `TransactionResult` in the loop), so the converted tests
//! assert against it through this extension trait.
//!
//! These mirror the litesvm `TransactionResult` helpers byte-for-byte:
//! `assert_error_code` formats the same `"custom program error: 0x{:x}"`
//! substring the runtime emits and matches it against the model's `logs` (where
//! the runtime prints `Program ... failed: custom program error: 0x<n>`) and its
//! `error` field. The model's `error` is the `TransactionError`'s `Debug`
//! (`InstructionError(0, Custom(130))`), so the substring lands in the logs, not
//! the error field; checking both matches what the litesvm `assert_error` does.
//! The success/failure panics carry the same `error` + `logs` context.
//!
//! Hoist-to-testsvm note: these four are generic over no engine and read only
//! public `model::Transaction` fields, so they belong on `testsvm::model`
//! itself (as inherent methods or a blessed trait). They live here until the
//! upstream crate grows the neutral assertion surface; once it does, drop this
//! file and re-export from `testsvm`.

use litesvm_utils::model;

/// Ergonomic assertions and accessors on the engine-neutral transaction record.
pub trait ModelTxExt {
    /// Whether the transaction succeeded (no error attached).
    fn is_success(&self) -> bool;

    /// The compute units the transaction consumed.
    fn compute_units(&self) -> u64;

    /// Assert the transaction succeeded; returns `self` for chaining. Panics with
    /// the error and logs when it failed.
    fn assert_success(self) -> Self;

    /// Assert the transaction failed with a custom program error `code`, matched
    /// by the `"custom program error: 0x{:x}"` substring the runtime emits in the
    /// logs (or, defensively, the error field). Returns `self` for chaining;
    /// panics with the actual error and logs otherwise.
    fn assert_error_code(self, code: u32) -> Self;
}

impl ModelTxExt for model::Transaction {
    fn is_success(&self) -> bool {
        self.error.is_none()
    }

    fn compute_units(&self) -> u64 {
        self.compute_units
    }

    fn assert_success(self) -> Self {
        assert!(
            self.error.is_none(),
            "Transaction failed: {}\nLogs:\n{}",
            self.error.as_deref().unwrap_or("Unknown error"),
            self.logs.join("\n")
        );
        self
    }

    fn assert_error_code(self, code: u32) -> Self {
        let needle = format!("custom program error: 0x{:x}", code);
        let in_logs = self.logs.iter().any(|l| l.contains(&needle));
        let in_error = self.error.as_ref().map(|e| e.contains(&needle)).unwrap_or(false);
        assert!(
            in_logs || in_error,
            "Expected error containing '{}'.\nError: {:?}\nLogs:\n{}",
            needle,
            self.error,
            self.logs.join("\n")
        );
        self
    }
}
