# Same program, two engines: a fingerprint comparison

> **Disclaimer.** `Subscriptions` is an audited Solana Foundation protocol, and is
> treated here as correct: it is a fixture, not the subject. The subject under test
> is an experimental testing framework of my own, which is unverified and may be
> wrong. Every difference reported below is therefore a question about that framework
> or the engines beneath it, not a defect in Subscriptions. Nothing in this document
> is a claim about the protocol's correctness.

## What is and is not under test

`Subscriptions` is a new, audited pinocchio protocol from the Solana Foundation. In
this experiment it is a fixture, not the subject. I treat it as correct (it has been
audited) and use it as a known-good program to exercise something else.

The subject under test is my own work, the part that might be wrong: an experimental
observability and testing framework, and the layer that lets one test suite run on
more than one execution engine. Subscriptions's behavior is the constant here; my
framework's rendering of that behavior is the variable. So when two reports of the
same transaction disagree, the discrepancy is a fact about my framework or the
engines beneath it, never about the protocol. Nothing in this document speaks to
whether Subscriptions is correct; by assumption, it is.

This is experimental and not verified for correctness; I am prioritizing the system
design to get to the ballpark, and the careful analysis comes after my capstone.

What I did: converted Subscriptions's test suite to the observability framework (for
richer execution data), parametrized the framework over the execution backend, ran
the **same program** on two engines (litesvm and quasar-svm), and compared the
rendered results. They look both equivalent and not.

Scope: this is the **engine axis**, not a program rewrite. The program binary
(`target/deploy/subscriptions_program.so`, built once from the pinocchio source)
is byte-identical on both engines; only the backend differs. There is no
"quasar version" of the program here, just the same `.so` under a second engine.

## What a fingerprint is, and what this compares

Strictly, a fingerprint is a Merkle hash of a transaction's *normalized*
behavioral signature: take the structured record of what the transaction did,
normalize it (resolve addresses to the roles that derived them, decide which
fields count), and hash. The result is a scalar that answers one question, are two
executions the same under this notion of identity, and nothing else.

This analysis works one step before the hash, on the rendered record itself: the
human-readable surface the fingerprint would be computed from. That record is the
structured CPI tree (every program invoked, nested, each with its compute), a
sequence diagram, an authority graph (who signed what), an ownership graph (what
owns what once the transaction settles), the decoded events with their fields, and
the transaction's fee and total compute. The actors are deterministic and the clock
is pinned, so a given scenario renders the same bytes on every run of one engine; a
difference between two records is a difference between two executions, not harness
noise.

Comparing the records directly, rather than their fingerprints, is deliberate, and
it is the stricter test. A fingerprint normalizes before it hashes, so it folds away
whatever a normalization chooses to ignore (an unaliased address that differs only
because a counter is not in lockstep across two runs). The record folds away
nothing. That is why 0 of 225 match, and it is the right place to start: you cannot
decide what a fingerprint should normalize until you have seen, un-normalized,
everything that actually differs.

## How the comparison focuses attention

Two records of the same scenario, diffed line by line, localize every place the two
executions disagree: a differing line is a differing field. One diff is a curiosity.
The leverage comes from doing it across all 225 scenarios and tallying which fields
differ, which turns a wall of diffs into a ranking. A field that differs in nearly
every scenario is a systematic difference between the engines; a field that differs
in a handful is incidental to particular tests. The systematic ones are where to
look first, because they are a property of the engine pair rather than of any one
scenario. The table below is that ranking, and it is the input a fingerprint's
design needs: a difference that is incidental and noisy is a candidate to normalize
away; one that is systematic and meaningful is a field to keep.

What follows is measurements: what differs, and how widely. No interpretation of the
causes; sorting each difference into its bucket is the task at the end.

## Result

- litesvm: 226 / 226 scenarios pass.
- quasar-svm: 226 / 226 scenarios pass.
- Rendered reports compared scenario-by-scenario (n = 225): **0 byte-identical.**

## What differs

Count of scenarios whose report differs in each field (n = 225):

| field                              | scenarios differing |
|------------------------------------|---------------------|
| Fee                                | 225                 |
| Compute units                      | 183                 |
| Single-instruction banner label    | 183                 |
| Unaliased account address          | 181                 |
| Graph edges                        | 181                 |

- 12 of 225 differ in the Fee line alone.

Example values (litesvm | quasar):

| field            | litesvm                              | quasar-svm                  |
|------------------|--------------------------------------|-----------------------------|
| Fee              | `5000`                               | `0`                         |
| banner label     | `subscriptions::InitSubscriptionAuthority` | `subscriptions::Approve` |
| unaliased addr   | `EkP9qmWM…utYb`                      | `6rTuoZtJ…jXyx`             |

Compute units differ in both direction and magnitude across scenarios:

| scenario                   | litesvm | quasar-svm |
|----------------------------|---------|------------|
| cancel (happy path)        | 15242   | 12020      |
| subscribe (happy path)     | 9242    | 10520      |
| create fixed delegation    | 6242    | 22520      |

For the banner difference, the in-tree frame line is identical on both engines
(`subscriptions::InitSubscriptionAuthority [1]`); only the header banner differs.

## Example pair to examine

One scenario (create a fixed delegation), rendered on each engine:

- [litesvm](examples/create-fixed-delegation.litesvm.md)
- [quasar-svm](examples/create-fixed-delegation.quasar-svm.md)

Diff: 40 lines.

## Browse all reports

Every scenario, rendered per engine (one report each, same 225 scenarios):

- litesvm: [`dsl_tests/`](../../dsl_tests/README.md)
- quasar-svm: [`quasar-reports/`](quasar-reports/README.md)

## Status

It worked: the same suite, unchanged, runs green on both engines. The reports do
not match. The differences above are measured. They are not yet explained.

## Hypothesis

Lossy execution fidelity: my framework, projecting one (correct) execution through
two engines, renders and meters it two different ways. The loss is mine to find, in
the framework or the engine adapters, not in the protocol. To be investigated.

## Task: explain each measured difference

1. **Fee.** Confirm Fee is `0` on quasar for every transaction and `quasar`
   reports `capabilities().fees == false`. Determine which other measured fields
   change as a function of the fee.
2. **Compute units.** Tabulate per-frame CU on both engines for one scenario.
   Determine whether the per-instruction delta is fixed or input-dependent.
   Identify the CU source on each engine.
3. **Banner label.** Locate where the single-instruction banner label is derived.
   Capture its input (`message.instructions[0]`) on both engines. Compare.
4. **Unaliased addresses.** Identify which accounts render unaliased. Check
   whether the address is stable across repeated runs of the same engine.
   Identify the address source.
5. **Graph edges.** Enumerate the node and edge set on both engines for one
   scenario. Map each added or removed element to the field it derives from.
6. **Invariants.** Enumerate the fields expected to be engine-invariant (account
   state deltas, CPI tree structure, decoded events, error codes). Confirm
   whether they match across all 225.

## Reproduce

```
cargo test -p tests-subscriptions          # litesvm reports -> dsl_tests/
cd engines/quasar-svm && cargo test         # quasar reports  -> quasar-reports/
# then diff dsl_tests/<slug>.md against engines/quasar-svm/quasar-reports/<slug>.md
```
