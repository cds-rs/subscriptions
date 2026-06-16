## Reject a forged token program — PASS

> passing a non-token account in the token-program slot is rejected

### Alice passes her own pubkey in the token-program slot

**InitSubscriptionAuthority (forged token program): structured CPI tree**

```text

Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✗ 272cu  signer=[alice, alice]
    └── Error: InvalidTokenProgram
Error: InstructionError(0, Custom(105))
Compute Units (this run): 272
Fee: 5000 lamports
Legend (2):
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

- [x] the transaction is refused: `false`
