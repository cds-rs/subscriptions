## Signer accounts must sign — PASS

> flipping any required signer to non-signer is rejected

**InitSubscriptionAuthority (owner forced non-signer): structured CPI tree**

```text

Transaction  signers=[sponsor]
└── subscriptions::InitSubscriptionAuthority [1] ✗ 228cu
    └── Error: NotSigner
Error: InstructionError(0, Custom(100))
Compute Units (this run): 228
Fee: 5000 lamports
Legend (2):
  sponsor       = 47cncVPgU4mK37H7VvxLCCsoDEKYaVhNLHHp4MbnEwvx
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```
