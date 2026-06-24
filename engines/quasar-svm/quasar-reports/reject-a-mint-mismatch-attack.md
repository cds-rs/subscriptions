## Reject a mint-mismatch attack — PASS

> a forged instruction whose embedded mint differs from the passed mint account is rejected

### The merchant embeds a mint in the instruction data that differs from the passed mint account

**CreatePlan (mint mismatch): structured CPI tree**

```text

── subscriptions::CreatePlan ───────────────────────────────
Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✗ 503cu  signer=merchant
    └── Error: MintMismatch (0x7d)
Error: custom program error: 0x7d
Compute Units (this run): 503
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```
