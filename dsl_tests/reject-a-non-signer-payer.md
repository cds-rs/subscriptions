## Reject a non-signer payer — PASS

> a trailing non-signer account in the optional payer slot is rejected

### Alice appends a non-signer in the optional payer slot

**InitSubscriptionAuthority (non-signer payer): structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✗ 397cu  signer=alice
    └── Error: NotSigner (0x64)
Error: InstructionError(0, Custom(100))
Compute Units (this run): 397
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```
