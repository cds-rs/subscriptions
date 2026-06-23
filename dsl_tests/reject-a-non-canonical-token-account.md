## Reject a non-canonical token account — PASS

> an auxiliary (non-ATA) token account is rejected where the canonical ATA is required

### Alice points the instruction at an auxiliary token account, not her ATA

**InitSubscriptionAuthority (non-canonical ATA): structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✗ 8054cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Error: InvalidAssociatedTokenAccountDerivedAddress (0x6c)
Error: InstructionError(0, Custom(108))
Compute Units (this run): 8054
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```
