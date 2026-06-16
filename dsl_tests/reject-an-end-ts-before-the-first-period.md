## Reject an end_ts before the first period — PASS

> a plan whose end_ts lands before its first period closes is rejected

### The merchant sets an end_ts before the first 720-hour period closes

**CreatePlan (end_ts before first period): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 391cu  signer=merchant
    └── Error: InvalidEndTs
Error: InstructionError(0, Custom(511))
Compute Units (this run): 391
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
