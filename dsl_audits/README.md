# DSL audit reproductions

These are narration tests that reproduce, in the anchor-litesvm DSL, the four
**High**-severity findings from the Cantina "Multi Delegator" security review
(the report and signed baseline live in [`../audits/`](../audits/)). The point
is to validate the audit's conclusions in our own vocabulary: not "the auditor
says there's a drain" but a test that performs the drain and a report that shows
it land, frame by frame, in the structured CPI tree and the authority/ownership
graphs.

The four reports here are the **before**: each exploit succeeds. They are static
copies committed on `turbin3` (the fixed branch) for reference; the live,
runnable reproduction lives on the `dsl-audit` branch (see Methodology).

## Before and after — the showcase

The same exploit, run against the vulnerable program (where it lands) and the
fixed program (where it is refused), **co-located in one report each** with the
full observability surface — the structured CPI trees and the authority/ownership
graphs, so the attack and its absence are both legible:

- [3.1.1 — a recurring pull before the delegation starts](./before-after-3-1-1.md)
- [3.1.2 — a ghost plan drains the subscriber](./before-after-3-1-2.md) — the 50,000× drain
- [3.1.3 — a pre-funded PDA blocks creation](./before-after-3-1-3.md)
- [3.1.4 — a ghost plan siphons past the agreement](./before-after-3-1-4.md)

## Findings

| # | finding | exploit | fix | refused (after) with |
|---|---|---|---|---|
| [3.1.1](./audit-3-1-1--a-recurring-pull-lands-before-the-delegation-starts.md) | missing start-time validation | a recurring pull lands before `start_ts` (`saturating_sub` floors it to 0) | PR6 | `DelegationNotStarted` |
| [3.1.2](./audit-3-1-2--a-ghost-plan-with-an-inflated-amount-drains-the-subscriber.md) | ghost-plan inflated amount | delete + recreate the plan at the same id with a huge per-period amount; the transfer reads the live amount, not the consented snapshot | PR4 | `PlanTermsMismatch` |
| [3.1.3](./audit-3-1-3--a-pre-funded-pda-blocks-authority-creation.md) | pre-funded PDA blocks creation | front-run the deterministic PDA with lamports; the unconditional `CreateAccount` is rejected (a permanent DoS) | PR5 | creation succeeds |
| [3.1.4](./audit-3-1-4--a-ghost-plan-with-an-extended-end-ts-siphons-past-the-agreement.md) | ghost-plan extended `end_ts` | recreate the plan with a far-future `end_ts` to keep pulling past the original end | PR4 | `PlanTermsMismatch` |

## Methodology

How these were set up, and why it works the way it does.

### The problem: the audited baseline can't run the DSL

The natural instinct is to branch off the audited commit (`18a50bc`, where the
bugs are live) and write the tests there. That doesn't work: the audited baseline
is **solana 2.x / litesvm 0.7**, and the DSL (the `World`/observability surface)
needs **solana 3.x / litesvm 0.12**. Grafting the DSL onto the old baseline would
mean porting the audited program itself to solana 3.x, at which point it is no
longer the audited code. So the audited baseline stays a read-only reference (the
`audit-baseline-18a50bc` branch); the reproduction happens on the modern stack.

### The trick: re-introduce each bug surgically, on the fixed code

Every High finding was fixed in a small, identifiable change (PR4 through PR6).
So on a branch off `turbin3` (which has both the DSL *and* the fixed program), we
reverse those fixes to bring the vulnerabilities back on a stack the DSL can run.
The shape is a clean before/after that any reviewer can re-walk:

1. **Revert all four fixes in one commit** (`revert(audit): re-introduce all four
   Cantina HIGH findings`). The program now carries the vulnerable behavior:
   - 3.1.1 (`transfer_validation.rs`): drop the `current_ts < period_start` guard.
   - 3.1.2 / 3.1.4 (`transfer_subscription.rs`): drop `check_plan_terms` and read
     the live plan amount instead of the consented snapshot.
   - 3.1.3 (`helpers/program.rs`): always `CreateAccount`, so a pre-funded PDA
     blocks creation.
2. **Exploit all** in a *separate* commit: one narration test per finding
   (`tests/integration-tests/src/audit_high_*.rs`), each performing the attack
   and rendering the surface. Keeping the tests out of the revert commit is what
   lets the next step keep them.
3. **Branch off the exploited state** (`dsl-audit-fixed`) and **`git revert` the
   revert** to restore all four fixes in place.
4. **Rerun the same tests.** They now fail, because each exploit is refused
   (`DelegationNotStarted`, `PlanTermsMismatch`, creation-succeeds). That failure
   is the regression proof: the test that demonstrated the drain can no longer
   drain.

### The branches

| branch | state | the four exploit tests |
|---|---|---|
| `dsl-audit` | fixes reverted (vulnerable) | **pass** — exploits succeed; reports here are written from this branch |
| `dsl-audit-fixed` | `git revert` of the revert (fixed) | **fail** — exploits refused, the named errors above |
| `audit-baseline-18a50bc` | the signed solana-2 audited baseline | (reference only; no DSL) |

### Regenerating

The reports are byte-reproducible (deterministic actors, pinned clock,
single-threaded), so they diff clean. To regenerate the **before** set:

```bash
git checkout dsl-audit
cargo build-sbf --manifest-path program/Cargo.toml   # the vulnerable .so
cargo test -p tests-subscriptions audit_high          # writes dsl_audits/
```

On `dsl-audit` the branch's `.cargo/config` points `MD_REPORTS_DIR` at
`dsl_audits/`; on `turbin3` the same tests run against the *fixed* program, so
the exploits are refused and the copies here are kept static.
