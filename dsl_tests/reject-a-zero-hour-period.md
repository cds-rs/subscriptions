## Reject a zero-hour period — PASS

> a plan with period_hours 0 is rejected

### The merchant tries to create a plan with a zero-hour period

**CreatePlan (period_hours 0): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 379cu  signer=merchant
    └── Error: InvalidPeriodLength
Error: InstructionError(0, Custom(402))
Compute Units (this run): 379
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
