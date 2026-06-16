## Reject a mint-mismatch attack — PASS

> a forged instruction whose embedded mint differs from the passed mint account is rejected

### The merchant embeds a mint in the instruction data that differs from the passed mint account

**CreatePlan (mint mismatch): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 497cu  signer=merchant
    └── Error: MintMismatch
Error: InstructionError(0, Custom(125))
Compute Units (this run): 497
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
