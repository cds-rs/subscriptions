## Survive a pre-funded PDA — PASS

> a griefer pre-funds the authority PDA; Alice can still initialize it

### Despite the pre-funded PDA, Alice initializes her authority

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 17518cu  signer=alice
    ├── System::Transfer (alice -> SubAuthority) 1,627,640 lamports [2] ✓ (no cu)
    ├── System::Allocate [2] ✓ (no cu)
    ├── System::Assign [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 17518
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**InitSubscriptionAuthority: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    participant Token
    alice ->> subscriptions: InitSubscriptionAuthority (17518cu)
    subscriptions ->> System: Transfer (alice → SubAuthority) 1,627,640 lamports
    subscriptions ->> System: Allocate
    subscriptions ->> System: Assign
    subscriptions ->> Token: Approve (126cu)
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
    subscriptions ->>+ System: Transfer (alice → SubAuthority) 1,627,640 lamports
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Allocate
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Assign
    System -->>- subscriptions: ok
    subscriptions ->>+ Token: Approve
    Token -->>- subscriptions: ok (126cu)
    subscriptions -->>- alice: ok (17518cu)
```

**InitSubscriptionAuthority: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    alice([alice]):::signer
    SubAuthority([SubAuthority]):::signer
    6NQM5qwU_Z2vo[("6NQM5qwU…Z2vo")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 6NQM5qwU_Z2vo
    System -->|writes| SubAuthority
    Token -->|writes| 6NQM5qwU_Z2vo
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
    6NQM5qwU_Z2vo[("6NQM5qwU…Z2vo")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 6NQM5qwU_Z2vo
```
