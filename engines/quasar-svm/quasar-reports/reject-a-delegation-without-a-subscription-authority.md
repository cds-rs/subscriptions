## Reject a delegation without a subscription authority — PASS

> creating a delegation before initializing the authority is refused

### Alice tries to create a delegation with no authority in place

**CreateFixedDelegation (no authority): structured CPI tree**

```text

── subscriptions::CreateFixedDelegation ────────────────────
Transaction  signers=[alice]
└── subscriptions::CreateFixedDelegation [1] ✗ 357cu  signer=alice
    └── Error: FixedDelegationExpiryInPast (0x12d)
Error: custom program error: 0x12d
Compute Units (this run): 357
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```
