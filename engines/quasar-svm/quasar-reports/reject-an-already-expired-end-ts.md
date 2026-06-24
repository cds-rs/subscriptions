## Reject an already-expired end_ts — PASS

> a plan whose end_ts is in the past is rejected

### The merchant tries an end_ts of 1_000 (in the past)

**CreatePlan (expired end_ts): structured CPI tree**

```text

── subscriptions::CreatePlan ───────────────────────────────
Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 397cu  signer=merchant
    └── Error: InvalidEndTs (0x1ff)
Error: custom program error: 0x1ff
Compute Units (this run): 397
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```
