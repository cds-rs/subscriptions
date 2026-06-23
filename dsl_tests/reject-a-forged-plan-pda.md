## Reject a forged plan PDA — PASS

> passing a non-canonical plan PDA is rejected

### The merchant points the instruction at a forged plan PDA

**CreatePlan (wrong PDA): structured CPI tree**

```text

── subscriptions::CreatePlan ───────────────────────────────
Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 2079cu  signer=merchant
    └── Error: InvalidPlanPda (0x1f6)
Error: InstructionError(0, Custom(502))
Compute Units (this run): 2079
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```
