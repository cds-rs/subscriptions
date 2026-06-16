## Reject a delegation without a subscription authority — PASS

> creating a delegation before initializing the authority is refused

### Alice tries to create a delegation with no authority in place

**CreateFixedDelegation (no authority): structured CPI tree**

```text

Transaction  signers=[alice]
└── subscriptions::CreateFixedDelegation [1] ✗ 351cu  signer=alice
    └── Error: FixedDelegationExpiryInPast
Error: InstructionError(0, Custom(301))
Compute Units (this run): 351
Fee: 5000 lamports
Legend (2):
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
