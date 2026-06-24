# Same program, two engines: a fingerprint comparison

`subscriptions` is a new pinocchio protocol from the Solana Foundation.
Experimentally, and not verified for correctness (I am prioritizing the system
design to get to the ballpark): I converted its test suite to our observability
framework for richer execution data, parametrized the framework over the
execution backend, ran the **same program** on two engines (litesvm and
quasar-svm), and compared the rendered results. They look both equivalent and
not. Deeper analysis after the capstone.

Scope: this is the **engine axis**, not a program rewrite. The program binary
(`target/deploy/subscriptions_program.so`, built once from the pinocchio source)
is byte-identical on both engines; only the backend differs. There is no
"quasar version" of the program here, just the same `.so` under a second engine.

Measurements only. No interpretation.

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

Lossy execution fidelity between the two engines. To be investigated.

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
