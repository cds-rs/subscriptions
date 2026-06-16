## Reject an over-long period — PASS

> a plan whose period exceeds the max is rejected

### The merchant tries a period of 8761 hours (over the max)

**CreatePlan (period_hours over max): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 373cu  signer=merchant
    └── Error: InvalidPeriodLength
Error: InstructionError(0, Custom(402))
Compute Units (this run): 373
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
