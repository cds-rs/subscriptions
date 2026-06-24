## Reject an over-long period — PASS

> a plan whose period exceeds the max is rejected

### The merchant tries a period of 8761 hours (over the max)

**CreatePlan (period_hours over max): structured CPI tree**

```text

── subscriptions::CreatePlan ───────────────────────────────
Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 379cu  signer=merchant
    └── Error: InvalidPeriodLength (0x192)
Error: custom program error: 0x192
Compute Units (this run): 379
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```
