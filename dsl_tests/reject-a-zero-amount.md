## Reject a zero amount — PASS

> a plan with a zero amount is rejected

### The merchant tries to create a plan with a zero amount

**CreatePlan (amount 0): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 375cu  signer=merchant
    └── Error: InvalidAmount
Error: InstructionError(0, Custom(129))
Compute Units (this run): 375
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
