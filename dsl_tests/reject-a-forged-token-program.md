## Reject a forged token program — PASS

> passing a non-token account in the token-program slot is rejected

### Alice passes her own pubkey in the token-program slot

**InitSubscriptionAuthority (forged token program): structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✗ 278cu  signer=[alice, alice]
    └── Error: InvalidTokenProgram (0x69)
Error: InstructionError(0, Custom(105))
Compute Units (this run): 278
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

- [x] the transaction is refused: `false`
