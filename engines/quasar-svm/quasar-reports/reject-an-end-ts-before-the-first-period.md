## Reject an end_ts before the first period — PASS

> a plan whose end_ts lands before its first period closes is rejected

### The merchant sets an end_ts before the first 720-hour period closes

**CreatePlan (end_ts before first period): structured CPI tree**

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
