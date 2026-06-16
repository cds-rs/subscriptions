## Reject an already-expired end_ts — PASS

> a plan whose end_ts is in the past is rejected

### The merchant tries an end_ts of 1_000 (in the past)

**CreatePlan (expired end_ts): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 397cu  signer=merchant
    └── Error: InvalidEndTs
Error: InstructionError(0, Custom(511))
Compute Units (this run): 397
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
