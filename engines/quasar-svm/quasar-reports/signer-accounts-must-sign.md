## Signer accounts must sign — PASS

> flipping any required signer to non-signer is rejected

**InitSubscriptionAuthority (owner forced non-signer): structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[sponsor]
└── subscriptions::InitSubscriptionAuthority [1] ✗ 234cu
    └── Error: NotSigner (0x64)
Error: custom program error: 0x64
Compute Units (this run): 234
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  sponsor       = 47cncVPgU4mK37H7VvxLCCsoDEKYaVhNLHHp4MbnEwvx
```
