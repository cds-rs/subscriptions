## Idempotent init preserves the original payer — PASS

> a second init by a different sponsor leaves the stored payer untouched

### Sponsor A initializes the authority

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[sponsor, alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 7764cu  signer=[alice, sponsor]
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 7764
Fee: 10000 lamports
Legend (3):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  sponsor       = 47cncVPgU4mK37H7VvxLCCsoDEKYaVhNLHHp4MbnEwvx
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
    alice ->> subscriptions: InitSubscriptionAuthority (7764cu)
    subscriptions ->> System: CreateAccount
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
    subscriptions ->>+ System: CreateAccount
    System -->>- subscriptions: ok
    subscriptions ->>+ Token: Approve
    Token -->>- subscriptions: ok (126cu)
    subscriptions -->>- alice: ok (7764cu)
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
    EjSY49xJ_kxHY[("EjSY49xJ…kxHY")]:::writable
    sponsor([sponsor]):::signer
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    sponsor -->|signs| subscriptions
    sponsor -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| EjSY49xJ_kxHY
    Token -->|writes| EjSY49xJ_kxHY
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
    EjSY49xJ_kxHY[("EjSY49xJ…kxHY")]:::account
    sponsor[(sponsor)]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| EjSY49xJ_kxHY
    System -->|owns| sponsor
```

### Sponsor B re-runs init

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[sponsor2, alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 6302cu  signer=[alice, sponsor2]
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 6302
Fee: 10000 lamports
Legend (3):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  sponsor2      = CRhhd9p4sqTfae2hJTknFKWimstLVHZRyYMK8GZLMKqz
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**InitSubscriptionAuthority: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant Token
    alice ->> subscriptions: InitSubscriptionAuthority (6302cu)
    subscriptions ->> Token: Approve (126cu)
```

**InitSubscriptionAuthority: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant Token
    alice ->>+ subscriptions: InitSubscriptionAuthority
    subscriptions ->>+ Token: Approve
    Token -->>- subscriptions: ok (126cu)
    subscriptions -->>- alice: ok (6302cu)
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
    EjSY49xJ_kxHY[("EjSY49xJ…kxHY")]:::writable
    sponsor2([sponsor2]):::signer
    Token[Token]:::program
    alice -->|signs| subscriptions
    sponsor2 -->|signs| subscriptions
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| EjSY49xJ_kxHY
    Token -->|writes| EjSY49xJ_kxHY
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
    EjSY49xJ_kxHY[("EjSY49xJ…kxHY")]:::account
    sponsor2[(sponsor2)]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| EjSY49xJ_kxHY
    System -->|owns| sponsor2
```

- [x] the stored payer is still sponsor A: `47cncVPgU4mK37H7VvxLCCsoDEKYaVhNLHHp4MbnEwvx`
