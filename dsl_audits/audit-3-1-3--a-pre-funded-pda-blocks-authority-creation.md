## AUDIT 3.1.3: a pre-funded PDA blocks authority creation — PASS

> an attacker pre-funds the authority PDA so Alice can never initialize it (Cantina HIGH 3.1.3)

### Mallory pre-funds Alice's authority PDA address

### Alice tries to initialize her authority

**InitSubscriptionAuthority: structured CPI tree**

```text

Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✗ 4721cu  signer=alice
    ├── System [2] ✗ (no cu)
    │   └── Error: custom program error: 0x0
    └── Error: custom program error: 0x0
Error: InstructionError(0, Custom(0))
Compute Units (this run): 4721
Fee: 5000 lamports
Legend (2):
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

Finding 3.1.3: the PDA address was pre-funded, so the unconditional CreateAccount on this branch is rejected by the System program and Alice's authority cannot be created — a permanent denial of service against any user whose (deterministic) PDA an attacker front-runs. On the fixed code (PR5) the program tops up the rent and Allocate/Assign-s in place, creation succeeds, and the check below fails: that failure is the regression proof the fix holds.

- [x] the pre-funded PDA BLOCKS creation (the vulnerability): `true`
