## Transfer subscription: an authorized puller pulls — PASS

> a whitelisted puller (not the merchant) pulls against the subscription

### Stage: Alice's authority, the merchant's plan, Alice subscribed

**InitSubscriptionAuthority: structured CPI tree**

```text

Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 6242cu  signer=alice
    ├── System [2] ✓ (no cu)
    └── Token [2] ✓ 126cu
Compute Units (this run): 6242
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
    alice ->> subscriptions: InitSubscriptionAuthority (6242cu)
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
    subscriptions ->>+ Token: unnamed
    Token -->>- subscriptions: ok (126cu)
    subscriptions -->>- alice: ok (6242cu)
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
    5Dk5CH8Y_8RDg[("5Dk5CH8Y…8RDg")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 5Dk5CH8Y_8RDg
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
    5Dk5CH8Y_8RDg[("5Dk5CH8Y…8RDg")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 5Dk5CH8Y_8RDg
```

**CreatePlan: structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✓ 3468cu  signer=merchant
    └── System [2] ✓ (no cu)
Compute Units (this run): 3468
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

**CreatePlan: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->> subscriptions: CreatePlan (3468cu)
    subscriptions ->> System: unnamed
```

**CreatePlan: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->>+ subscriptions: CreatePlan
    subscriptions ->>+ System: unnamed
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
    Plan[(Plan)]:::writable
    System[System]:::program
    merchant -->|signs| subscriptions
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

### The whitelisted puller pulls 10 tokens to the merchant ATA

**TransferSubscription (authorized puller): structured CPI tree**

```text

Transaction  signers=[puller]
└── subscriptions::TransferSubscription [1] ✓ 5955cu  signer=puller
    ├── Token [2] ✓ 113cu
    └── subscriptions [2] ✓ 137cu
Compute Units (this run): 5955
Fee: 5000 lamports
Legend (2):
  puller        = 6MyHPNpi5TVyR6mNQ3RbqUpt3XjpABEFQXirP3Fyosot
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

**TransferSubscription (authorized puller): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant puller
    participant subscriptions
    participant Token
    puller ->> subscriptions: TransferSubscription (5955cu)
    subscriptions ->> Token: unnamed (113cu)
    subscriptions ->> subscriptions: unnamed (137cu)
```

**TransferSubscription (authorized puller): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant puller
    participant subscriptions
    participant Token
    puller ->>+ subscriptions: TransferSubscription
    subscriptions ->>+ Token: unnamed
    Token -->>- subscriptions: ok (113cu)
    subscriptions ->>+ subscriptions: unnamed
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- puller: ok (5955cu)
```

**TransferSubscription (authorized puller): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    Subscription[(Subscription)]:::writable
    5Dk5CH8Y_8RDg[("5Dk5CH8Y…8RDg")]:::writable
    5u3C1UQF_f8p9[("5u3C1UQF…f8p9")]:::writable
    puller([puller]):::signer
    Token[Token]:::program
    puller -->|signs| subscriptions
    subscriptions -->|writes| Subscription
    subscriptions -->|writes| 5Dk5CH8Y_8RDg
    subscriptions -->|writes| 5u3C1UQF_f8p9
```

**TransferSubscription (authorized puller): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    Subscription[(Subscription)]:::account
    Token[Token]:::owner
    5Dk5CH8Y_8RDg[("5Dk5CH8Y…8RDg")]:::account
    5u3C1UQF_f8p9[("5u3C1UQF…f8p9")]:::account
    System[System]:::owner
    puller[(puller)]:::account
    subscriptions -->|owns| Subscription
    Token -->|owns| 5Dk5CH8Y_8RDg
    Token -->|owns| 5u3C1UQF_f8p9
    System -->|owns| puller
```

- [x] the merchant received 10 tokens: `10000000`
