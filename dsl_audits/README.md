# DSL audit reproductions

Narration tests that reproduce the four **High**-severity findings from the
Cantina "Multi Delegator" security review (`audits/`), validating the audit's
conclusions in the anchor-litesvm DSL.

Each test performs the finding's exploit against a deliberately-vulnerable
program: on this `dsl-audit` branch the four fixes are reverted in one commit
(`29bd8f1`), and each report below renders the full observability surface (the
structured CPI tree, the sequence diagrams, the authority and ownership graphs)
showing the attack land.

These are the **before**: the exploit succeeds. The **after** is the
`dsl-audit-fixed` branch (a `git revert` of `29bd8f1`, restoring all four fixes),
where the *same* tests fail because each exploit is now refused — that failure is
the regression proof. The named errors there are `DelegationNotStarted` (3.1.1),
`PlanTermsMismatch` (3.1.2 / 3.1.4), and creation simply succeeding (3.1.3).

Regenerate: `cargo test -p tests-subscriptions audit_high` (reports land here via
`MD_REPORTS_DIR`, set to `dsl_audits` in `.cargo/config.toml` on this branch).

## Findings

- **3.1.1** — [a recurring pull lands before the delegation starts](./audit-3-1-1--a-recurring-pull-lands-before-the-delegation-starts.md)
  Missing start-time validation: `saturating_sub` floors a pre-start timestamp to
  0, so the full per-period budget is pullable before `start_ts`. Fixed in PR6.
- **3.1.2** — [a ghost plan with an inflated amount drains the subscriber](./audit-3-1-2--a-ghost-plan-with-an-inflated-amount-drains-the-subscriber.md)
  Delete and recreate the plan at the same id (same PDA) with a huge per-period
  amount; `transfer_subscription` reads the live amount, not the consented
  snapshot. Fixed in PR4 (`check_plan_terms`).
- **3.1.3** — [a pre-funded PDA blocks authority creation](./audit-3-1-3--a-pre-funded-pda-blocks-authority-creation.md)
  An attacker pre-funds the deterministic PDA address; the unconditional
  `CreateAccount` is rejected by the System program, a permanent denial of
  service. Fixed in PR5 (top-up + `Allocate`/`Assign` in place).
- **3.1.4** — [a ghost plan with an extended end_ts siphons past the agreement](./audit-3-1-4--a-ghost-plan-with-an-extended-end-ts-siphons-past-the-agreement.md)
  Recreate the plan with a far-future `end_ts` to keep pulling past the original
  end. Same root cause and fix as 3.1.2 (PR4).
