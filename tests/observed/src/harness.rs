//! Shared harness for the observed (DSL) twins of the subscriptions suite.
//!
//! Two pieces, reused by every test in this crate:
//!
//!   - `world()` builds the backend (the program loaded from its built `.so`,
//!     the clock pinned to a fixed `NOW` so every PDA and timestamp is
//!     byte-reproducible), and hands it back ready for the suite's own account
//!     builders, which take `&mut LiteSVM` and so run against `backend.svm_mut()`.
//!
//!   - `render_all(md, result, svm, caption)` is the centerpiece: it appends
//!     every observability form anchor-litesvm can render of a single
//!     transaction to the report. Each converted test ends by calling it, so the
//!     report shows one `TransactionResult` read five ways: the structured CPI
//!     tree, the sequence diagram (with and without lifelines), and the
//!     authority and ownership graphs.

use litesvm_utils::{LiteSVM, LiteSvmBackend, MarkdownBlock, Report, TestSVM, TransactionResult};

use tests_subscriptions::tests::constants::PROGRAM_ID;

/// A fixed wall-clock. Pinned so decoded timestamps (and thus the whole report)
/// are byte-reproducible across runs.
pub const NOW: i64 = 1_700_000_000;

/// Build the observed backend: the program loaded from its built `.so`, the
/// clock pinned to `NOW`, wrapped in `LiteSvmBackend` so every `send` captures a
/// per-frame trace. The suite's account builders run against `backend.svm_mut()`.
pub fn world() -> LiteSvmBackend {
    let mut svm = LiteSVM::new();
    let so = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/subscriptions_program.so");
    svm.add_program_from_file(PROGRAM_ID.to_bytes(), so).unwrap();
    let mut backend = LiteSvmBackend::new(svm);
    backend.warp_to_timestamp(NOW);
    backend
}

/// Append every render form of one transaction to the report, under a caption.
///
/// This is what the crate exists to show: one `TransactionResult`, read five
/// ways. The CPI tree is plain text (fenced as `text`); the four graphs are
/// already `` ```mermaid ``-fenced fragments, so they go in verbatim as `Raw`
/// (fencing them again would bury the diagram inside a text block, the report
/// bug this crate's predecessor flushed out).
pub fn render_all(md: &mut Report, result: &TransactionResult, _backend: &LiteSvmBackend, caption: &str) {
    md.block(
        format!("{caption}: structured CPI tree"),
        MarkdownBlock::Fenced { lang: "text".into(), body: result.logs_structured_string() },
    );
    md.block(
        format!("{caption}: sequence diagram"),
        MarkdownBlock::Raw(result.mermaid_string()),
    );
    md.block(
        format!("{caption}: sequence diagram, with lifelines"),
        MarkdownBlock::Raw(result.mermaid_string_with_lifelines()),
    );
    md.block(
        format!("{caption}: authority graph"),
        MarkdownBlock::Raw(result.authority_graph_string()),
    );
    md.block(
        format!("{caption}: ownership graph"),
        MarkdownBlock::Raw(result.ownership_graph_string()),
    );
}

