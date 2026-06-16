# Converting the integration suite to the anchor-litesvm DSL

This document records how the subscriptions integration tests were converted from
raw `litesvm` calls to the anchor-litesvm testing DSL: the World/scenario pattern,
where every test draws named actors from a fixed cast, runs each on-chain action
through an observing backend, and leaves a narrative report (a structured CPI
tree, sequence diagrams, and authority/ownership graphs) under `dsl_tests/`. It is
written for someone who will maintain or extend the suite, so it leans on the
mechanics rather than the motivation.

Scope: the 16 test files under `tests/integration-tests/src/`, the harness in
`utils/world.rs`, the builder changes in `utils/test_helpers.rs`, and the
dependency and determinism wiring. Out of scope: the program itself, and the
audit reproductions (those have their own write-up in `dsl_audits/`).

## 1. The starting point

Before the conversion, a test looked like this: `setup()` returned a bare
`LiteSVM` plus a default payer; builder structs (`CreatePlan`, `Subscribe`,
`TransferDelegation`, ...) constructed an `Instruction` and sent it through
`build_and_send_transaction`, which called `litesvm.send_transaction` directly;
the return was `litesvm::types::TransactionResult` (an alias for
`Result<TransactionMetadata, FailedTransactionMetadata>`); and assertions went
through a `TransactionResultExt` trait with `assert_ok` / `assert_err`.

That works, but the test reads in the machine's vocabulary: an ordered
`Vec<AccountMeta>`, base58 pubkeys, slot positions. Nothing tells the reader who
signed, who merely funded, what moved, or what the program actually did inside
the transaction. There is no captured execution record to render.

## 2. The target: World and scenario

The DSL replaces the bare `LiteSVM` with a `World` (`utils/world.rs`): one object
holding the observing backend, the narrative `Report`, and the cast. The shape of
a converted test is:

```rust
let mut world = World::new("Subscribe with a sponsor", "the sponsor pays; Alice is untouched");
let alice = world.actor("alice");           // deterministic, funded, aliased
let mint = world.usdc_mint(&alice);          // fabrication (not the focus)
world.md().step("Alice subscribes; the sponsor pays");
let ix = Subscribe::new(world.svm_mut(), &alice, ...).instruction();
world.send_ok(&[ix], &[&sponsor, &alice], "Subscribe");   // observed; renders the surface
world.md().check("Alice's lamports are untouched", before, after);
```

Three vocabulary terms carry the rest of this document:

- **actor**: a named signer, drawn from the cast by role (`world.actor("alice")`),
  derived deterministically, funded once, and aliased so every rendered output
  names it.
- **prop**: a named non-signing account (a PDA, an ATA, a mint), registered with
  `world.prop(pubkey, "name")` so it too renders by name.
- **observed send**: `world.send` / `send_ok` / `send_err`, which route the
  transaction through `LiteSvmBackend` (so the per-frame trace is captured) and
  append the rendered surface to the report.

## 3. Step one: catalogue the cast

The first move was empirical, not architectural: grep every signer binding across
the suite (the `setup()` tuple positions, `Keypair::new()`, `init_wallet`) and see
what roles actually recur. The roughly dozen ad-hoc names collapsed onto six:

| cast member | role | collapsed from |
|---|---|---|
| `alice` | the principal: subscriber, delegator, mint authority | `alice`, `subscriber`, `user` |
| `merchant` | plan owner / payee | `merchant` |
| `bob` | delegatee / payee (delegation family) | `bob`, `delegatee` |
| `charlie` | second counterparty | `charlie` |
| `sponsor` (+`sponsor2`) | rent and fee payer, not the beneficiary | `sponsor`, `sponsor_a/_b`, `fee_payer` |
| `mallory` | the adversary | `attacker`, `eve`, `random_signer`, unauthorized `puller` |

Two casting calls are worth flagging. The adversary was deliberately collapsed:
`attacker`, `eve`, `random_signer` all play the identical beat (an unprivileged
signer hitting a guarded instruction), so unifying them as `mallory` makes the
authority graph legible (the same lane signs where it has no right, every time).
And `bob` and `merchant` were kept distinct even though both are "the account that
receives Alice's tokens", because the domain genuinely separates a merchant
offering a plan from a delegatee granted a pull, and both names were entrenched.

`actor(name)` derives `deterministic_keypair("subscriptions", name)`, funds it
once (tracked in a `HashSet` so repeat calls do not double-fund), and registers
its alias. Determinism here is not incidental; see section 7.

## 4. Step two: the World harness

`World::new(title, intent)` builds the backend and registers the whole vocabulary
once, so individual tests never re-register anything:

- The program is loaded from `../../target/deploy/subscriptions_program.so` and the
  clock pinned to a fixed `NOW` (`1_700_000_000`).
- The svm is wrapped in `LiteSvmBackend`, which installs the inspect-hook trace
  recorder; every `send` then captures the full per-frame execution record.
- Instruction names: all 16 instructions plus `EmitEvent`, keyed by their
  one-byte discriminator.
- Error names: the full `SubscriptionsError` table (75 variants), registered
  through a small `errors!` macro that expands `(SubscriptionsError::V as u32,
  "V")`. The macro reads the discriminant from the enum rather than transcribing
  numbers, which matters because the enum is not a dense run (it has explicit
  anchors at 100, 300, 400, 500, 600). A hand-typed "106" would have mislabelled a
  finding; `as u32` cannot.
- Event decoders: the self-CPI events (`SubscriptionCreated`, `RecurringTransfer`)
  are registered with `register_cpi_event`, keyed on the 8-byte event tag plus the
  one-byte event discriminator, since the program emits events as a self-CPI whose
  data is `EVENT_IX_TAG ++ disc ++ fields` (there is no `Program data:` log to
  read; the payload rides on the traced frame).

The send surface:

- `send(ixs, signers, label)` routes through `backend.send`, then appends the full
  surface to the report on success (CPI tree, sequence diagram with and without
  lifelines, authority graph, ownership graph) and only the refused tree on
  failure (which already carries the named error). `signers[0]` is the fee payer,
  so a sponsor is ordered first when it should pay.
- `send_ok` asserts success; `send_err` asserts a specific `SubscriptionsError`.

Two scenario verbs cover the common multi-step setups: `init_authority` (Alice's
SubscriptionAuthority, optionally sponsored) and `stage_subscription` (a live
plan plus Alice subscribed), each observed so the staging renders into the report
too.

Finally, `ObservedResultExt` keeps the assertion surface familiar: `.assert_ok()`
delegates to the rich result's `assert_success`, and `.assert_err(SubscriptionsError::X)`
to `assert_error_code(X as u32)`, so converted tests read the same as the
originals.

## 5. Step three: split construction from sending in the builders

This is the load-bearing change. The builders sent raw through
`build_and_send_transaction`, discarding the `Message`; to observe a send, the
World needs the `Instruction` so it can route it through `backend.send` (which
assembles the message, trace, and registries into the rich result).

So every builder gained an `instruction()` that returns the built-but-unsent
`Instruction`, with `execute()` refactored to delegate to it (no serialization
drift). The data-bearing builders keep variants that also return the derived PDA:
`CreateDelegation::fixed_ix` / `recurring_ix` return `(Instruction, Pubkey)`,
`CreatePlan` exposes `plan_pda()`, `Subscribe` exposes `subscription_pda()`. The
`PlanData` / `UpdatePlanData` serialization (a raw `from_raw_parts` over the packed
struct) was reused verbatim rather than re-implemented; getting those bytes wrong
is exactly the kind of subtle break worth avoiding.

A converted observed beat then reads as a one-liner with full per-call
visibility:

```rust
world.send_ok(
    &[CreatePlan::new(world.svm_mut(), &merchant, mint).plan_id(1).amount(50_000_000).instruction()],
    &[&merchant],
    "CreatePlan",
);
```

One borrow-checker note: `CreatePlan::new(world.svm_mut(), ...)` borrows the World
mutably, and `world.send_ok` needs it mutably again, so the builder must be
dropped before the send. Building the instruction inside the argument expression
(or a `{ }` block) does that; the builder's `&mut LiteSVM` borrow ends as soon as
`instruction()` returns the owned `Instruction`.

## 6. Step four: the per-test conversion

With the harness in place, each test followed the same recipe:

1. `setup()` becomes `World::new(title, intent)`; the payer/`alice`/`user` becomes
   `world.actor("alice")`, and so on per the cast table.
2. Fabrication (`init_mint`, `init_ata`, `set_account`, `CreateSubscription`'s
   account injection) runs unchanged through `world.svm_mut()`: it is setup, not
   the action under test, so it does not need observing. Reads use `world.svm()`.
3. Each on-chain action under test routes through `world.send_ok` / `send_err`
   (or `send` for a generic "must fail" check).
4. `current_ts()` becomes `world.now()`; `move_clock_forward` becomes
   `world.warp`.
5. The hand-built malformed-instruction negative tests (flip a writable flag, a
   forged token program, a non-signer payer) keep their `Instruction` construction
   and just send it through `world.send_err(&[ix], signers, label,
   SubscriptionsError::X)`, so they render the refused tree with the named error.
6. Meaningful assertions become `world.md().check(label, expected, actual)`,
   which renders a PASS/FAIL row in the report; the rest stay as plain `assert!`.

The titles must be unique per test, because the report filename is a slug of the
title; for `rstest` matrices, the case is folded into the title.

## 7. Determinism: the reports are committed snapshots

The reports are committed as byte-reproducible regression artifacts (a `Report`
flushes on `Drop` to `<MD_REPORTS_DIR>/<slug>.md`), so they must not churn on every
run. Two execution-order artifacts otherwise rewrite them:

1. `Pubkey::new_unique()` props come off a process-global counter, so parallel test
   order shuffles the addresses (and any PDA or alias derived from them). This was
   the bulk: ~167 of 220 files differed between two parallel runs. The fix is the
   cast plus deterministic props (and one stray `Keypair::new()` puller became
   `world.actor("puller")`).
2. The program's one-time JIT-compile cost is charged to whichever test runs first
   in the process (we measured 10,742 cu cold versus 6,242 cu warm on the same
   instruction), so the first-payer flips per run.

A fixed test order pins both, so `.cargo/config.toml` sets `RUST_TEST_THREADS = 1`
alongside `MD_REPORTS_DIR = { value = "dsl_tests", relative = true }`. Verified: two
default runs now differ in zero of 220 reports. The general, parallel-safe fix (a
title-seeded `world.unique()` plus a JIT warmup) is tracked upstream as
cds-rs/anchor-litesvm#9; the thread pin is the documented escape hatch until then.

## 8. Dependencies, and the version trap

The suite depends on the framework by git, not path:
`litesvm-utils = { git = "https://github.com/cds-rs/anchor-litesvm", branch = "turbin3" }`,
so it tracks what the cohort consumes. The trap worth knowing: `litesvm-token`
pulls its own `litesvm`, and if that version differs from the framework's, the
token helpers (`CreateMint`, `MintTo`) reject `world.svm()` as the wrong
`LiteSVM` type. The fix is a `[patch.crates-io]` that points `litesvm` at the
cds-rs fork (`branch = "main"`), the same source the framework git-deps, so cargo
unifies them into one. Keep `litesvm-token` at the matching major (`0.12`).

## 9. The DRY pass

After the mechanical conversion, three patterns recurred enough to factor out, all
swept in-editor (`:argdo` / grug-far) with regex capture:

- `world.usdc_mint(&owner)` collapsed 83 copies of the canonical
  `init_mint(world.svm_mut(), TOKEN_PROGRAM_ID, MINT_DECIMALS, 1_000_000_000, Some(X.pubkey()), &[])`.
- `world.fund_ata(mint, &owner, n)` collapsed 121 `init_ata` calls.
- `as_pubkey([u8; 32])` (one shared helper) replaced 7 local copies, so report
  rows comparing a program-side `Address` render base58 rather than a byte array.

A separate sweep dropped 116 redundant `send_ok(...).assert_ok()` (since `send_ok`
already asserts), and `cargo fix` removed the imports the helpers made dead (the
crate denies unused imports).

## 10. Scaling the conversion

Sixteen files is too many to convert by hand in one sitting, so after locking the
pattern on a reference file (`test_initialize_subscription_authority.rs`), the
remaining fifteen were converted by a workflow: one agent per file, each given the
reference, the World API, the cast table, and the builder `instruction()` catalog.

The one constraint is that the crate compiles as a unit, so a half-converted file
breaks every other file's `cargo test`. The workflow therefore ran the files
sequentially (each agent verified its file green before the next started), which
keeps the crate buildable throughout at the cost of wall-clock.

## 11. What a report shows, and validation

Each converted test writes one report reading a single transaction up to five
ways: the structured CPI tree (with self-CPI events decoded to aliased,
column-aligned fields), the sequence diagram with and without lifelines, the
authority graph (who signed, who funded, which PDA the program signed for), and
the ownership graph (account owner edges). Failure paths render the refused tree
with the named error.

The converted suite passes 219 of 221, with the two failures pre-existing and
environmental: `test_transfer_fixed_delegation` has two tests that
`include_bytes!` an external transfer-hook program `.so` absent from the checkout,
unrelated to the conversion. The 220 reports are byte-reproducible and live in
`dsl_tests/` with a generated `README.md` table of contents grouped by
instruction.
