## AUDIT 3.1.3 (regression): the fix survives a pre-funded PDA — PASS

> the PR5 fix tops up a pre-funded PDA so Alice can still initialize (Cantina HIGH 3.1.3)

### Mallory pre-funds Alice's authority PDA address

### Alice tries to initialize her authority

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::Approve ──────────────────────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 12796cu  signer=alice
    ├── System::Transfer (alice -> SubAuthority) 628,640 lamports [2] ✓ (no cu)
    ├── System::Allocate [2] ✓ (no cu)
    ├── System::Assign [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 2904cu
Compute Units (this run): 12796
Fee: 0 lamports
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
    alice ->> subscriptions: InitSubscriptionAuthority (12796cu)
    subscriptions ->> System: Transfer (alice → SubAuthority) 628,640 lamports
    subscriptions ->> System: Allocate
    subscriptions ->> System: Assign
    subscriptions ->> Token: Approve (2904cu)
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
    subscriptions ->>+ System: Transfer (alice → SubAuthority) 628,640 lamports
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Allocate
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Assign
    System -->>- subscriptions: ok
    subscriptions ->>+ Token: Approve
    Token -->>- subscriptions: ok (2904cu)
    subscriptions -->>- alice: ok (12796cu)
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
    FHRJVU4Q_pic7[("FHRJVU4Q…pic7")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| FHRJVU4Q_pic7
    System -->|writes| SubAuthority
    Token -->|writes| FHRJVU4Q_pic7
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
    FHRJVU4Q_pic7[("FHRJVU4Q…pic7")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| FHRJVU4Q_pic7
```

Finding 3.1.3: the PDA address was pre-funded, so the unconditional CreateAccount on this branch is rejected by the System program and Alice's authority cannot be created — a permanent denial of service against any user whose (deterministic) PDA Mallory front-runs. On the fixed code (PR5) the program tops up the rent and Allocate/Assign-s in place, creation succeeds, and the check below confirms it — this regression guards the fix.

- [x] the fix survives the pre-funded PDA (creation succeeds): `false`
