## Survive a pre-funded PDA — PASS

> a griefer pre-funds the authority PDA; Alice can still initialize it

### Despite the pre-funded PDA, Alice initializes her authority

**InitSubscriptionAuthority: structured CPI tree**

```text

Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 13018cu  signer=alice
    ├── System [2] ✓ (no cu)
    ├── System [2] ✓ (no cu)
    ├── System [2] ✓ (no cu)
    └── Token [2] ✓ 126cu
Compute Units (this run): 13018
Fee: 5000 lamports
Legend (2):
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

**InitSubscriptionAuthority: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    participant Token
    alice ->> subscriptions: InitSubscriptionAuthority (13018cu)
    subscriptions ->> System: unnamed
    subscriptions ->> System: unnamed
    subscriptions ->> System: unnamed
    subscriptions ->> Token: unnamed (126cu)
```

**InitSubscriptionAuthority: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    participant Token
    alice ->>+ subscriptions: InitSubscriptionAuthority
    subscriptions ->>+ System: unnamed
    System -->>- subscriptions: ok
    subscriptions ->>+ System: unnamed
    System -->>- subscriptions: ok
    subscriptions ->>+ System: unnamed
    System -->>- subscriptions: ok
    subscriptions ->>+ Token: unnamed
    Token -->>- subscriptions: ok (126cu)
    subscriptions -->>- alice: ok (13018cu)
```

**InitSubscriptionAuthority: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    alice([alice]):::signer
    SubAuthority[(SubAuthority)]:::writable
    2wZuJCG2_fdZR[("2wZuJCG2…fdZR")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 2wZuJCG2_fdZR
```

**InitSubscriptionAuthority: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    alice[(alice)]:::account
    subscriptions[subscriptions]:::owner
    SubAuthority[(SubAuthority)]:::account
    Token[Token]:::owner
    2wZuJCG2_fdZR[("2wZuJCG2…fdZR")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 2wZuJCG2_fdZR
```
