## Transfer subscription: multiple pulls within a period — PASS

> two pulls within the same period accumulate up to the period limit

### Stage: Alice's authority, the merchant's plan, Alice subscribed

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 9242cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 9242
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
    alice ->> subscriptions: InitSubscriptionAuthority (9242cu)
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
    subscriptions -->>- alice: ok (9242cu)
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
    CCgEmw61_TpZi[("CCgEmw61…TpZi")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| CCgEmw61_TpZi
    Token -->|writes| CCgEmw61_TpZi
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
    CCgEmw61_TpZi[("CCgEmw61…TpZi")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| CCgEmw61_TpZi
```

**CreatePlan: structured CPI tree**

```text

── subscriptions::CreatePlan ───────────────────────────────
Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✓ 3468cu  signer=merchant
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 3468
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```

**CreatePlan: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->> subscriptions: CreatePlan (3468cu)
    subscriptions ->> System: CreateAccount
```

**CreatePlan: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->>+ subscriptions: CreatePlan
    subscriptions ->>+ System: CreateAccount
    System -->>- subscriptions: ok
    subscriptions -->>- merchant: ok (3468cu)
```

**CreatePlan: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    merchant([merchant]):::signer
    Plan([Plan]):::signer
    System[System]:::program
    merchant -->|signs| subscriptions
    merchant -->|signs| System
    Plan -->|signs| System
    subscriptions -->|writes| Plan
```

**CreatePlan: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    merchant[(merchant)]:::account
    subscriptions[subscriptions]:::owner
    Plan[(Plan)]:::account
    System -->|owns| merchant
    subscriptions -->|owns| Plan
```

### The merchant pulls 20 tokens (first pull)

**TransferSubscription (first pull): structured CPI tree**

```text

── subscriptions::TransferSubscription ─────────────────────
Transaction  signers=[merchant]
└── subscriptions::TransferSubscription [1] ✓ 7442cu  signer=merchant
    ├── Token::TransferChecked [2] ✓ 113cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
Compute Units (this run): 7442
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```

**TransferSubscription (first pull): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->> subscriptions: TransferSubscription (7442cu)
    subscriptions ->> Token: TransferChecked (113cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferSubscription (first pull): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->>+ subscriptions: TransferSubscription
    subscriptions ->>+ Token: TransferChecked
    Token -->>- subscriptions: ok (113cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- merchant: ok (7442cu)
```

**TransferSubscription (first pull): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    Subscription[(Subscription)]:::writable
    CCgEmw61_TpZi[("CCgEmw61…TpZi")]:::writable
    B3RKAdvn_g1gr[("B3RKAdvn…g1gr")]:::writable
    merchant([merchant]):::signer
    Token[Token]:::program
    SubAuthority([SubAuthority]):::signer
    EventAuthority([EventAuthority]):::signer
    merchant -->|signs| subscriptions
    SubAuthority -->|signs| Token
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| Subscription
    subscriptions -->|writes| CCgEmw61_TpZi
    subscriptions -->|writes| B3RKAdvn_g1gr
    Token -->|writes| CCgEmw61_TpZi
    Token -->|writes| B3RKAdvn_g1gr
```

**TransferSubscription (first pull): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    Subscription[(Subscription)]:::account
    Token[Token]:::owner
    CCgEmw61_TpZi[("CCgEmw61…TpZi")]:::account
    B3RKAdvn_g1gr[("B3RKAdvn…g1gr")]:::account
    System[System]:::owner
    merchant[(merchant)]:::account
    subscriptions -->|owns| Subscription
    Token -->|owns| CCgEmw61_TpZi
    Token -->|owns| B3RKAdvn_g1gr
    System -->|owns| merchant
```

- [x] the merchant has 20 tokens: `20000000`

### The merchant pulls another 20 tokens (second pull)

**TransferSubscription (second pull): structured CPI tree**

```text

── subscriptions::TransferSubscription ─────────────────────
Transaction  signers=[merchant]
└── subscriptions::TransferSubscription [1] ✓ 7442cu  signer=merchant
    ├── Token::TransferChecked [2] ✓ 113cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
Compute Units (this run): 7442
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```

**TransferSubscription (second pull): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->> subscriptions: TransferSubscription (7442cu)
    subscriptions ->> Token: TransferChecked (113cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferSubscription (second pull): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->>+ subscriptions: TransferSubscription
    subscriptions ->>+ Token: TransferChecked
    Token -->>- subscriptions: ok (113cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- merchant: ok (7442cu)
```

**TransferSubscription (second pull): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    Subscription[(Subscription)]:::writable
    CCgEmw61_TpZi[("CCgEmw61…TpZi")]:::writable
    B3RKAdvn_g1gr[("B3RKAdvn…g1gr")]:::writable
    merchant([merchant]):::signer
    Token[Token]:::program
    SubAuthority([SubAuthority]):::signer
    EventAuthority([EventAuthority]):::signer
    merchant -->|signs| subscriptions
    SubAuthority -->|signs| Token
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| Subscription
    subscriptions -->|writes| CCgEmw61_TpZi
    subscriptions -->|writes| B3RKAdvn_g1gr
    Token -->|writes| CCgEmw61_TpZi
    Token -->|writes| B3RKAdvn_g1gr
```

**TransferSubscription (second pull): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    Subscription[(Subscription)]:::account
    Token[Token]:::owner
    CCgEmw61_TpZi[("CCgEmw61…TpZi")]:::account
    B3RKAdvn_g1gr[("B3RKAdvn…g1gr")]:::account
    System[System]:::owner
    merchant[(merchant)]:::account
    subscriptions -->|owns| Subscription
    Token -->|owns| CCgEmw61_TpZi
    Token -->|owns| B3RKAdvn_g1gr
    System -->|owns| merchant
```

- [x] the merchant has 40 tokens: `40000000`
- [x] the pulled-in-period is 40 tokens: `40000000`
